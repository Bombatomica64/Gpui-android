//! Root view: the catalog, and a stack of screens over it under a persistent
//! header. Pushes and pops slide; on Android 14+ a back swipe previews the pop.
//!
//! Back is not `escape`: gpui-mobile reports it through `host::on_back`, and the
//! lab decides. An open Kit overlay (dialog, sheet, popover, menu) gets an
//! `escape` first, since that is how Kit closes them; otherwise the top screen
//! pops. With no screen open the lab tells the host it does not want Back, so the
//! system handles it (back to home, with its own predictive animation).

use std::time::Instant;

use gpui::{
    AnyView, AnyWindowHandle, App, AppContext as _, Context, Entity, FocusHandle, Focusable,
    Global, InteractiveElement, IntoElement, ParentElement, ScrollHandle, SharedString,
    StatefulInteractiveElement, Styled, Subscription, WeakEntity, Window, div, prelude::*, px,
    relative,
};
use gpui_kit::component::{
    IconName,
    button::Button,
    h_flex,
    input::{Input, InputEvent, InputState},
    v_flex,
};

use crate::matrix::{self, Status};
use crate::nav::{self, Motion, Preview, Route, Transition};
use crate::screens::{self, ScreenDef};
use crate::ui::{self, prelude::*};

/// Where Back and screen requests go: the catalog of the newest window.
struct Navigator {
    app: WeakEntity<LabApp>,
    window: AnyWindowHandle,
}

impl Global for Navigator {}

pub fn init(_: &mut App) {
    #[cfg(target_os = "android")]
    gpui_mobile::android::host::on_back(|_, event, cx| on_back(event, cx));
}

/// What a back request means right now, worked out outside the catalog's own
/// update so that overlay `escape` handlers may touch any entity.
#[cfg(target_os = "android")]
fn on_back(event: gpui_mobile::android::host::BackEvent, cx: &mut App) {
    use gpui::Keystroke;
    use gpui_kit::component::WindowExt as _;
    use gpui_mobile::android::host::{BackEdge, BackEvent};

    let Some(navigator) = cx.try_global::<Navigator>() else {
        log::warn!("back: no catalog window");
        return;
    };
    let (app, handle) = (navigator.app.clone(), navigator.window);
    let result = handle.update(cx, |_, window, cx| {
        let overlay = window.has_active_dialog(cx) || window.has_active_sheet(cx);
        let back = match event {
            BackEvent::Started(at) => Back::Started {
                progress: at.progress,
                from_right: at.edge == BackEdge::Right,
                overlay,
            },
            BackEvent::Progressed(at) => Back::Progressed(at.progress),
            BackEvent::Cancelled => Back::Cancelled,
            BackEvent::Invoked => {
                // Kit closes its overlays, and fields their selection handles, on
                // escape; a plain field lets it through, so this is `false` then.
                let escape = Keystroke::parse("escape").expect("valid keystroke");
                let closed = window.dispatch_keystroke(escape, cx);
                Back::Invoked {
                    handled: closed || overlay,
                }
            }
        };
        app.update(cx, |this, cx| this.back(back, window, cx))
    });
    if let Err(err) = result {
        log::warn!("back: catalog window gone: {err}");
    }
}

/// A back request, platform-independent.
#[derive(Clone, Copy, Debug)]
enum Back {
    Started {
        progress: f32,
        from_right: bool,
        /// A Kit dialog or sheet is open: Back is for it, not for the screen.
        overlay: bool,
    },
    Progressed(f32),
    Cancelled,
    Invoked {
        /// An overlay took it (or is still open and refused to close).
        handled: bool,
    },
}

/// Push a screen by title onto the stack of the catalog window.
pub fn push(title: &str, window: &mut Window, cx: &mut App) {
    with_app(cx, |this, cx| this.push(title, true, window, cx));
}

/// Pop the top screen, as the back button does.
pub fn pop(window: &mut Window, cx: &mut App) {
    with_app(cx, |this, cx| this.pop(0.0, window, cx));
}

/// Pop every screen, back to the catalog.
pub fn pop_to_catalog(window: &mut Window, cx: &mut App) {
    with_app(cx, |this, cx| this.route(Route::Catalog, window, cx));
}

/// Titles of the open screens, bottom first.
pub fn stack(cx: &App) -> Vec<&'static str> {
    cx.try_global::<Navigator>()
        .and_then(|navigator| navigator.app.upgrade())
        .map(|app| app.read(cx).stack.iter().map(|s| s.def.title).collect())
        .unwrap_or_default()
}

fn with_app(cx: &mut App, f: impl FnOnce(&mut LabApp, &mut Context<LabApp>)) {
    let app = cx
        .try_global::<Navigator>()
        .and_then(|navigator| navigator.app.upgrade());
    match app {
        Some(app) => app.update(cx, f),
        None => log::warn!("navigation: no catalog window"),
    }
}

struct OpenScreen {
    def: &'static ScreenDef,
    view: AnyView,
    scroll: ScrollHandle,
}

pub struct LabApp {
    focus: FocusHandle,
    search: Entity<InputState>,
    query: SharedString,
    filter: Option<Status>,
    /// Open screens over the catalog, bottom first.
    stack: Vec<OpenScreen>,
    /// The push, pop or settle under way. A popping screen stays on the stack
    /// until its motion ends.
    transition: Option<Transition>,
    /// A back swipe in progress (Android 14+), before it commits or cancels.
    preview: Option<Preview>,
    catalog_scroll: ScrollHandle,
    /// Last bar appearance sent to the host, so it is only sent on change.
    light_system_bars: Option<bool>,
    _subscriptions: Vec<Subscription>,
}

impl LabApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| InputState::new(window, cx).placeholder("Search components…"));
        let subscription = cx.subscribe(&search, |this, state, event: &InputEvent, cx| {
            if let InputEvent::Change = event {
                this.query = state.read(cx).value();
                cx.notify();
            }
        });
        let focus = cx.focus_handle();
        focus.focus(window, cx);
        cx.set_global(Navigator {
            app: cx.weak_entity(),
            window: window.window_handle(),
        });
        // Requests from the Java thread (intent-requested screens, deep links) are
        // picked up here, on the GPUI thread.
        cx.spawn_in(window, async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(150))
                    .await;
                if let Some(route) = crate::diagnostics::take_requested_route() {
                    let result =
                        this.update_in(cx, |this, window, cx| this.route(route, window, cx));
                    if result.is_err() {
                        break;
                    }
                }
            }
        })
        .detach();
        let appearance = cx.observe_window_appearance(window, |_, window, cx| {
            screens::themes::follow_system(window, cx)
        });
        Self {
            focus,
            search,
            query: SharedString::default(),
            filter: None,
            stack: Vec::new(),
            transition: None,
            preview: None,
            catalog_scroll: ScrollHandle::new(),
            light_system_bars: None,
            _subscriptions: vec![subscription, appearance],
        }
    }

    /// Open a screen over the current one. `animate` is off for the screens under
    /// the top one of a route, which are never seen arriving.
    pub fn push(&mut self, id: &str, animate: bool, window: &mut Window, cx: &mut Context<Self>) {
        let Some(def) = screens::find(id) else {
            log::error!("no demo screen named {id:?}");
            return;
        };
        self.finish_transition();
        self.preview = None;
        log::info!(
            "push screen: {} (depth {})",
            def.title,
            self.stack.len() + 1
        );
        let view = (def.build)(window, cx);
        self.stack.push(OpenScreen {
            def,
            view,
            scroll: ScrollHandle::new(),
        });
        if animate {
            self.transition = Some(Transition::push(Instant::now()));
        }
        self.remember_top();
        self.focus.focus(window, cx);
        cx.notify();
    }

    /// Slide the top screen out from `from` (where a back preview left it, or 0).
    fn pop(&mut self, from: f32, window: &mut Window, cx: &mut Context<Self>) {
        if self.stack.is_empty() {
            return;
        }
        let from = match self.transition.take() {
            Some(transition) if transition.motion == Motion::Pop => {
                self.transition = Some(transition);
                return;
            }
            // Popping a screen still sliding in starts from where it is.
            Some(transition) => transition.sample(Instant::now()).0,
            None => from,
        };
        if let Some(top) = self.stack.last() {
            log::info!("pop screen: {}", top.def.title);
        }
        self.transition = Some(Transition::pop(Instant::now(), from));
        self.focus.focus(window, cx);
        cx.notify();
    }

    /// Replace the stack with a route's screens; only the top one slides in.
    fn route(&mut self, route: Route, window: &mut Window, cx: &mut Context<Self>) {
        self.finish_transition();
        self.preview = None;
        match route {
            Route::Catalog => {
                // Drop what is underneath and slide the top one out over the catalog.
                let keep = self.stack.len().saturating_sub(1);
                self.stack.drain(..keep);
                self.pop(0.0, window, cx);
            }
            Route::Screens(titles) => {
                log::info!("route to {titles:?}");
                self.stack.clear();
                let last = titles.len().saturating_sub(1);
                for (ix, title) in titles.iter().enumerate() {
                    self.push(title, ix == last, window, cx);
                }
            }
        }
        self.remember_top();
        cx.notify();
    }

    fn back(&mut self, back: Back, window: &mut Window, cx: &mut Context<Self>) {
        let now = Instant::now();
        match back {
            Back::Started {
                progress,
                from_right,
                overlay,
            } => {
                self.finish_transition();
                self.preview = (!overlay && !self.stack.is_empty()).then_some(Preview {
                    progress,
                    from_right,
                });
            }
            Back::Progressed(progress) => {
                if let Some(preview) = &mut self.preview {
                    preview.progress = progress;
                }
            }
            Back::Cancelled => {
                if let Some(preview) = self.preview.take() {
                    self.transition = Some(Transition::settle(now, preview.offset()));
                }
            }
            Back::Invoked { handled } => {
                let from = self.preview.take().map(|preview| preview.offset());
                if handled {
                    log::info!("back: an overlay took it");
                    if let Some(from) = from {
                        self.transition = Some(Transition::settle(now, from));
                    }
                } else if self.stack.is_empty() {
                    // The host only sends Back while a screen is open; this is a
                    // race with the last pop.
                    log::info!("back at catalog root: moving task to background");
                    #[cfg(target_os = "android")]
                    crate::host::move_task_to_back();
                } else {
                    self.pop(from.unwrap_or(0.0), window, cx);
                }
            }
        }
        cx.notify();
    }

    /// End the running motion at once: a popping screen is dropped.
    fn finish_transition(&mut self) {
        if let Some(transition) = self.transition.take()
            && transition.motion == Motion::Pop
        {
            self.stack.pop();
            self.remember_top();
        }
    }

    /// A reload (`cargo gpui`) reopens the top screen: LabActivity reads this extra.
    fn remember_top(&self) {
        gpui_hot::remember("screen", self.stack.last().map(|s| s.def.title));
    }

    /// The top page's offset this frame, ending a finished motion.
    fn step(&mut self, window: &mut Window) -> f32 {
        let preview = self.preview.map(|p| p.offset()).unwrap_or(0.0);
        let Some(transition) = self.transition else {
            return preview;
        };
        let (offset, done) = transition.sample(Instant::now());
        if done {
            self.finish_transition();
            0.0
        } else {
            window.request_animation_frame();
            offset
        }
    }

    fn render_header(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let title: SharedString = self
            .stack
            .last()
            .map(|s| s.def.title.into())
            .unwrap_or_else(|| "GPUI Kit Mobile Lab".into());
        let back_label = match self.stack.len() {
            0 | 1 => "Catalog",
            n => self.stack[n - 2].def.title,
        };
        h_flex()
            .w_full()
            .h(px(52.))
            .flex_none()
            .px_2()
            .gap_2()
            .border_b_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().title_bar)
            .when(!self.stack.is_empty(), |this| {
                this.child(
                    Button::new("back")
                        .ghost()
                        .icon(IconName::ChevronLeft)
                        .label(back_label)
                        .on_click(cx.listener(|this, _, window, cx| this.pop(0.0, window, cx))),
                )
            })
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .font_semibold()
                    .child(title),
            )
            .when(self.stack.is_empty(), |this| {
                this.child(
                    Button::new("open-themes")
                        .ghost()
                        .small()
                        .icon(IconName::Palette)
                        .on_click(
                            cx.listener(|this, _, window, cx| {
                                this.push("Themes", true, window, cx)
                            }),
                        ),
                )
            })
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!(
                        "{:.0}×{:.0}",
                        window.viewport_size().width.as_f32(),
                        window.viewport_size().height.as_f32()
                    )),
            )
    }

    fn render_summary(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let viewport = window.viewport_size();
        let facts = [
            ("GPUI Kit", crate::GPUI_KIT_VERSION.to_string()),
            ("GPUI", crate::GPUI_VERSION.to_string()),
            ("gpui-mobile", crate::GPUI_MOBILE_REVISION.to_string()),
            (
                "Android API",
                crate::diagnostics::api_level()
                    .map(|level| level.to_string())
                    .unwrap_or_else(|| "unknown".into()),
            ),
            (
                "Viewport",
                format!(
                    "{:.0}×{:.0} pt @ {:.2}x",
                    viewport.width.as_f32(),
                    viewport.height.as_f32(),
                    window.scale_factor()
                ),
            ),
        ];
        let counters = [
            Status::Working,
            Status::Partial,
            Status::Broken,
            Status::NotApplicable,
            Status::Untested,
        ]
        .into_iter()
        .filter(|status| *status != Status::Untested || matrix::count(Status::Untested) > 0)
        .map(|status| {
            let selected = self.filter == Some(status);
            Button::new(SharedString::from(format!("filter-{}", status.label())))
                .small()
                .outline()
                .selected(selected)
                .label(format!("{} {}", matrix::count(status), status.label()))
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.filter = if this.filter == Some(status) {
                        None
                    } else {
                        Some(status)
                    };
                    cx.notify();
                }))
        });
        v_flex()
            .gap_2()
            .child(
                v_flex().gap_0p5().children(
                    facts
                        .into_iter()
                        .map(|(label, value)| ui::value_row(label, value, cx)),
                ),
            )
            .child(h_flex().flex_wrap().gap_1().children(counters))
    }

    fn render_catalog(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let query = self.query.to_lowercase();
        let matches = |row: &matrix::Row| {
            (query.is_empty()
                || row.component.to_lowercase().contains(&query)
                || row.screen.to_lowercase().contains(&query)
                || row.category.to_lowercase().contains(&query))
                && self.filter.is_none_or(|status| row.status == status)
        };

        let tools: Vec<_> = screens::TOOLS
            .iter()
            .map(|id| {
                let id: &'static str = id;
                Button::new(SharedString::from(format!("tool-{id}")))
                    .outline()
                    .small()
                    .label(id)
                    .on_click(
                        cx.listener(move |this, _, window, cx| this.push(id, true, window, cx)),
                    )
            })
            .collect();

        let mut sections = Vec::new();
        for category in matrix::categories() {
            let rows: Vec<&matrix::Row> = matrix::rows()
                .iter()
                .filter(|row| row.category == category && matches(row))
                .collect();
            if rows.is_empty() {
                continue;
            }
            let mut items = Vec::new();
            for row in rows {
                items.push(self.render_row(row, cx).into_any_element());
            }
            sections.push(
                v_flex()
                    .gap_1()
                    .child(
                        div()
                            .pt_3()
                            .text_xs()
                            .font_semibold()
                            .text_color(cx.theme().muted_foreground)
                            .child(category.to_uppercase()),
                    )
                    .children(items)
                    .into_any_element(),
            );
        }
        let empty = sections.is_empty();

        div()
            .id("catalog")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .track_scroll(&self.catalog_scroll)
            .child(
                v_flex()
                    .p_3()
                    .gap_3()
                    .child(self.render_summary(window, cx))
                    .child(Input::new(&self.search).cleanable(true))
                    .child(
                        v_flex()
                            .gap_1()
                            .child(div().text_xs().font_semibold().child("LAB TOOLS"))
                            .child(h_flex().flex_wrap().gap_1().children(tools)),
                    )
                    .children(sections)
                    .when(empty, |this| {
                        this.child(ui::hint("No component matches the search and filter.", cx))
                    })
                    .child(div().h(px(24.))),
            )
    }

    fn render_row(&self, row: &'static matrix::Row, cx: &mut Context<Self>) -> impl IntoElement {
        let has_screen = row.included && screens::find(row.screen).is_some();
        h_flex()
            .id(SharedString::from(format!(
                "row-{}-{}",
                row.category, row.component
            )))
            .w_full()
            .min_h(px(48.))
            .px_3()
            .py_2()
            .gap_2()
            .rounded(cx.theme().radius)
            .border_1()
            .border_color(cx.theme().border)
            .when(has_screen, |this| {
                this.active(|this| this.bg(cx.theme().accent)).on_click(
                    cx.listener(move |this, _, window, cx| this.push(row.screen, true, window, cx)),
                )
            })
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .child(div().text_sm().child(row.component))
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(if has_screen {
                                SharedString::from(format!("→ {}", row.screen))
                            } else {
                                SharedString::from(row.notes)
                            }),
                    ),
            )
            .child(ui::status_tag(row.status))
    }

    fn render_screen(&self, screen: &OpenScreen, cx: &mut Context<Self>) -> impl IntoElement {
        let covered: Vec<_> = matrix::for_screen(screen.def.title).collect();
        let header = (!covered.is_empty()).then(|| {
            h_flex()
                .flex_wrap()
                .gap_1()
                .px_3()
                .pt_2()
                .children(covered.into_iter().map(|row| {
                    h_flex()
                        .gap_1()
                        .text_xs()
                        .child(
                            div()
                                .size_2()
                                .rounded_full()
                                .bg(ui::status_color(row.status, cx)),
                        )
                        .child(row.component)
                }))
        });
        if screen.def.owns_scroll {
            // Screens with virtualized lists own their scrolling; nesting them in
            // another scroll container would create two scroll owners.
            v_flex()
                .flex_1()
                .min_h_0()
                .children(header)
                .child(div().flex_1().min_h_0().child(screen.view.clone()))
                .into_any_element()
        } else {
            div()
                .id(SharedString::from(format!("screen-{}", screen.def.title)))
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .track_scroll(&screen.scroll)
                .child(
                    v_flex()
                        .children(header)
                        .child(div().p_3().child(screen.view.clone()))
                        .child(div().h(px(48.))),
                )
                .into_any_element()
        }
    }
}

impl Focusable for LabApp {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

#[gpui_hot::hot]
impl Render for LabApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Keyboard input needs a focused node. When a focused input is removed or
        // blurs, nothing is focused; keep focus inside the app.
        if window.focused(cx).is_none() {
            self.focus.focus(window, cx);
        }
        let light = !cx.theme().is_dark();
        if self.light_system_bars != Some(light) {
            self.light_system_bars = Some(light);
            #[cfg(target_os = "android")]
            crate::host::set_light_system_bars(light);
        }
        let offset = self.step(window);
        // Android decides at the start of a gesture who gets Back: the lab while a
        // screen is open, the system (back to home) at the catalog.
        #[cfg(target_os = "android")]
        gpui_mobile::android::host::set_back_enabled(!self.stack.is_empty());
        if let Some(top) = self.stack.last() {
            crate::diagnostics::set_screen_scroll(-top.scroll.offset().y.as_f32());
        }
        let moving = offset != 0.0 || self.transition.is_some();
        let depth = self.stack.len();
        let background = cx.theme().background;
        let page = |depth: usize,
                    offset: f32,
                    this: &Self,
                    window: &mut Window,
                    cx: &mut Context<Self>| {
            let content = match depth {
                0 => this.render_catalog(window, cx).into_any_element(),
                n => this
                    .render_screen(&this.stack[n - 1], cx)
                    .into_any_element(),
            };
            v_flex()
                .id(("page", depth))
                .absolute()
                .top_0()
                .bottom_0()
                .w_full()
                .left(relative(offset))
                .bg(background)
                .child(content)
        };
        let mut body = div().relative().flex_1().min_h_0().overflow_hidden();
        if moving && depth > 0 {
            body = body
                .child(page(depth - 1, nav::under_offset(offset), self, window, cx))
                .child(
                    div()
                        .absolute()
                        .inset_0()
                        .bg(gpui::black().opacity(nav::scrim(offset))),
                );
        }
        body = body.child(
            page(depth, offset, self, window, cx).when(moving, |page| page.shadow_lg().occlude()),
        );
        // The window is edge to edge: keep the content inside the part GPUI knows
        // is clear of the system bars, the cutout and the keyboard, with the
        // title bar color underneath them.
        let viewport = window.viewport_size();
        let visible = window.fully_visible_bounds();
        div()
            .size_full()
            .pt(visible.top())
            .pb(viewport.height - visible.bottom())
            .pl(visible.left())
            .pr(viewport.width - visible.right())
            .bg(cx.theme().title_bar)
            .child(
                v_flex()
                    .track_focus(&self.focus)
                    .size_full()
                    .bg(cx.theme().background)
                    .text_color(cx.theme().foreground)
                    .font_family(cx.theme().font_family.clone())
                    .child(self.render_header(window, cx))
                    .child(body),
            )
    }
}
