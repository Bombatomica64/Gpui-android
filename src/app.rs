//! Root view: the catalog, and one open screen at a time under a persistent header.

use gpui::{
    AnyView, App, AppContext as _, Context, Entity, FocusHandle, Focusable, InteractiveElement,
    IntoElement, KeyBinding, ParentElement, Pixels, Render, ScrollHandle, SharedString,
    StatefulInteractiveElement, Styled, Subscription, Window, actions, div, prelude::*, px,
};
use gpui_kit::component::{
    IconName,
    button::Button,
    h_flex,
    input::{AnyInputState, Input, InputEvent, InputState},
    v_flex,
};

use crate::matrix::{self, Status};
use crate::screens::{self, ScreenDef};
use crate::ui::{self, prelude::*};

actions!(lab, [GoBack]);

const CONTEXT: &str = "LabApp";

pub fn init(cx: &mut App) {
    // Android back arrives as "escape". Kit overlays and inputs bind escape in
    // deeper contexts, so they close first; only unhandled presses reach GoBack.
    cx.bind_keys([KeyBinding::new("escape", GoBack, Some(CONTEXT))]);
}

struct OpenScreen {
    def: &'static ScreenDef,
    view: AnyView,
}

pub struct LabApp {
    focus: FocusHandle,
    search: Entity<InputState>,
    query: SharedString,
    filter: Option<Status>,
    screen: Option<OpenScreen>,
    catalog_scroll: ScrollHandle,
    screen_scroll: ScrollHandle,
    /// Focused input and viewport height last revealed by keyboard avoidance.
    revealed: Option<(String, Pixels)>,
    _subscriptions: Vec<Subscription>,
}

const HEADER_HEIGHT: f32 = 52.;

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
        // Requests from the Java thread (IME dismissal, intent-requested screens) are
        // picked up here, on the GPUI thread.
        cx.spawn_in(window, async move |this, cx| loop {
            cx.background_executor().timer(std::time::Duration::from_millis(150)).await;
            if crate::diagnostics::take_blur_request() {
                let _ = cx.update(|window, cx| window.blur(cx));
            }
            if let Some(screen) = crate::diagnostics::take_requested_screen() {
                let result = this.update_in(cx, |this, window, cx| this.open(&screen, window, cx));
                if result.is_err() {
                    break;
                }
            }
        })
        .detach();
        Self {
            focus,
            search,
            query: SharedString::default(),
            filter: None,
            screen: None,
            catalog_scroll: ScrollHandle::new(),
            screen_scroll: ScrollHandle::new(),
            revealed: None,
            _subscriptions: vec![subscription],
        }
    }

    pub fn open(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        let Some(def) = screens::find(id) else {
            log::error!("no demo screen named {id:?}");
            return;
        };
        log::info!("open screen: {}", def.title);
        let view = (def.build)(window, cx);
        self.screen = Some(OpenScreen { def, view });
        self.screen_scroll = ScrollHandle::new();
        self.focus.focus(window, cx);
        cx.notify();
    }

    fn close_screen(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(screen) = self.screen.take() {
            log::info!("close screen: {}", screen.def.title);
        }
        self.focus.focus(window, cx);
        cx.notify();
    }

    fn go_back(&mut self, _: &GoBack, window: &mut Window, cx: &mut Context<Self>) {
        if self.screen.is_some() {
            self.close_screen(window, cx);
        } else {
            log::info!("back at catalog root: moving task to background");
            #[cfg(target_os = "android")]
            crate::host::move_task_to_back();
        }
    }

    fn render_header(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let title: SharedString = self
            .screen
            .as_ref()
            .map(|s| s.def.title.into())
            .unwrap_or_else(|| "GPUI Kit Mobile Lab".into());
        h_flex()
            .w_full()
            .h(px(52.))
            .flex_none()
            .px_2()
            .gap_2()
            .border_b_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().title_bar)
            .when(self.screen.is_some(), |this| {
                this.child(
                    Button::new("back")
                        .ghost()
                        .icon(IconName::ChevronLeft)
                        .label("Catalog")
                        .on_click(cx.listener(|this, _, window, cx| this.close_screen(window, cx))),
                )
            })
            .child(div().flex_1().min_w_0().truncate().font_semibold().child(title))
            .when(self.screen.is_none(), |this| {
                this.child(
                    Button::new("open-themes")
                        .ghost()
                        .small()
                        .icon(IconName::Palette)
                        .on_click(cx.listener(|this, _, window, cx| this.open("Themes", window, cx))),
                )
            })
            .child(div().text_xs().text_color(cx.theme().muted_foreground).child(format!(
                "{:.0}×{:.0}",
                window.viewport_size().width.as_f32(),
                window.viewport_size().height.as_f32()
            )))
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
                    this.filter = if this.filter == Some(status) { None } else { Some(status) };
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
                    .on_click(cx.listener(move |this, _, window, cx| this.open(id, window, cx)))
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
            // Without this GPUI turns a horizontal pan into vertical scrolling of a
            // vertical-only container (a mouse-wheel convenience that is wrong on touch).
            .restrict_scroll_to_axis()
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
        let has_screen = screens::find(row.screen).is_some();
        h_flex()
            .id(SharedString::from(format!("row-{}-{}", row.category, row.component)))
            .w_full()
            .min_h(px(48.))
            .px_3()
            .py_2()
            .gap_2()
            .rounded(cx.theme().radius)
            .border_1()
            .border_color(cx.theme().border)
            .when(has_screen, |this| {
                this.active(|this| this.bg(cx.theme().accent)).on_click(cx.listener(
                    move |this, _, window, cx| this.open(row.screen, window, cx),
                ))
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

    /// Keyboard avoidance. adjustResize shrinks GPUI's viewport when the IME opens,
    /// but neither GPUI nor Kit scrolls a focused input back into view (Kit's mobile
    /// guide leaves keyboard avoidance to the host). Reveal it once per change of
    /// viewport height or focused field, so later manual scrolling is not undone.
    fn reveal_focused_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let focused = window.focused_input(cx);
        let bounds = match &focused {
            Some(AnyInputState::Input(state)) => Some(state.read(cx).input_bounds()),
            Some(AnyInputState::Textarea(state)) => Some(state.read(cx).input_bounds()),
            Some(AnyInputState::Editor(state)) => Some(state.read(cx).input_bounds()),
            _ => None,
        };
        let height = window.viewport_size().height;
        let key = focused.as_ref().map(|input| (format!("{input:?}"), height));
        if key == self.revealed {
            return;
        }
        self.revealed = key;
        let Some(bounds) = bounds.filter(|b| b.size.height > px(0.)) else {
            return;
        };
        let top_limit = px(HEADER_HEIGHT + 8.);
        let bottom_limit = height - px(16.);
        // Tall fields (textareas) only need their top part visible.
        let bottom = bounds.bottom().min(bounds.top() + px(120.));
        let delta = if bottom > bottom_limit {
            bottom - bottom_limit
        } else if bounds.top() < top_limit {
            bounds.top() - top_limit
        } else {
            return;
        };
        let handle = if self.screen.is_some() { &self.screen_scroll } else { &self.catalog_scroll };
        let mut offset = handle.offset();
        offset.y -= delta;
        log::info!("keyboard avoidance: scrolling focused input by {:.0} pt", delta.as_f32());
        handle.set_offset(offset);
        cx.notify();
        // Overlays anchored to the input (Kit's touch edit menu) read its bounds from
        // the previous layout; one more full refresh re-anchors them after the scroll.
        window.on_next_frame(|window, _| window.refresh());
    }

    fn render_screen(&self, screen: &OpenScreen, cx: &mut Context<Self>) -> impl IntoElement {
        crate::diagnostics::set_screen_scroll(-self.screen_scroll.offset().y.as_f32());
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
                .restrict_scroll_to_axis()
                .track_scroll(&self.screen_scroll)
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

impl Render for LabApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Back needs a focused node inside this key context. When a focused input
        // is removed or blurs, nothing is focused and back would be dropped.
        if window.focused(cx).is_none() {
            self.focus.focus(window, cx);
        }
        self.reveal_focused_input(window, cx);
        let body = match &self.screen {
            Some(screen) => self.render_screen(screen, cx).into_any_element(),
            None => self.render_catalog(window, cx).into_any_element(),
        };
        v_flex()
            .key_context(CONTEXT)
            .track_focus(&self.focus)
            .on_action(cx.listener(Self::go_back))
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .font_family(cx.theme().font_family.clone())
            .child(self.render_header(window, cx))
            .child(body)
    }
}
