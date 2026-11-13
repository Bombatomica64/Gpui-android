//! Pinch to zoom and swipe actions on list rows, built on GPUI's own
//! gestures: `on_pinch` for two fingers, `on_drag` for a row's sideways swipe.

use std::time::{Duration, Instant};

use gpui::{
    AnyElement, AppContext as _, Bounds, ClickEvent, Context, DragMoveEvent, Empty,
    InteractiveElement, IntoElement, MouseButton, MouseDownEvent, ParentElement, PinchEvent,
    Pixels, Point, Render, StatefulInteractiveElement, Styled, TouchPhase, Window, canvas, div,
    point, prelude::FluentBuilder as _, px,
};
use gpui_kit::component::{button::Button, h_flex, v_flex};

use crate::ui::{self, EventLog, prelude::*};

const TILES: usize = 8;
const TILE: f32 = 36.;
const MIN_ZOOM: f32 = 0.5;
const MAX_ZOOM: f32 = 6.;
/// How far a row must travel before letting go deletes or archives it.
const SWIPE_COMMIT: f32 = 110.;
/// Where a committed row slides to: past the edge of a phone screen.
const SWIPE_OUT: f32 = 420.;
const SETTLE: Duration = Duration::from_millis(200);

const MESSAGES: [(&str, &str); 8] = [
    ("Ada", "Notes from the gesture review"),
    ("Build bot", "APK ready: gpui-mobile-lab"),
    ("Grace", "Lunch on Friday?"),
    ("Linus", "Re: pinch deltas"),
    ("Margaret", "Flight plan attached"),
    ("Alan", "The table columns move now"),
    ("Barbara", "Swipe left to delete"),
    ("Edsger", "Swipe right to archive"),
];

/// The value a row carries while it is being swiped.
#[derive(Clone, Copy)]
struct SwipedRow(usize);

/// Swiping draws the row itself under the finger, so the drag has no preview.
impl Render for SwipedRow {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}

struct Swipe {
    row: usize,
    start_x: Pixels,
    offset: Pixels,
}

/// A released row on its way back to rest, or off the screen.
struct Settling {
    row: usize,
    from: Pixels,
    to: Pixels,
    start: Instant,
    /// What the row's removal logs once it is off the screen.
    action: Option<&'static str>,
}

impl Settling {
    fn progress(&self) -> f32 {
        (self.start.elapsed().as_secs_f32() / SETTLE.as_secs_f32()).min(1.)
    }

    /// Eases out, so the row leaves the finger at speed and slows to rest.
    fn offset(&self) -> Pixels {
        let t = 1. - (1. - self.progress()).powi(3);
        self.from + (self.to - self.from) * t
    }
}

pub struct GesturesScreen {
    zoom: f32,
    /// Where the grid's top-left corner sits in the pad, before any zoom.
    origin: Point<Pixels>,
    pad_bounds: Bounds<Pixels>,
    last_center: Option<Point<Pixels>>,
    pinches: usize,
    /// Indices into `MESSAGES` still in the inbox.
    inbox: Vec<usize>,
    swipe: Option<Swipe>,
    settling: Vec<Settling>,
    log: EventLog,
}

impl GesturesScreen {
    pub fn new(_: &mut Window, _: &mut Context<Self>) -> Self {
        Self {
            zoom: 1.,
            origin: point(px(16.), px(16.)),
            pad_bounds: Bounds::default(),
            last_center: None,
            pinches: 0,
            inbox: (0..MESSAGES.len()).collect(),
            swipe: None,
            settling: Vec::new(),
            log: EventLog::default(),
        }
    }

    /// Zooms about the point between the fingers, and pans as that point moves.
    fn pinch(&mut self, event: &PinchEvent, cx: &mut Context<Self>) {
        let center = event.position - self.pad_bounds.origin;
        match event.phase {
            TouchPhase::Started => {
                self.pinches += 1;
                self.last_center = Some(center);
            }
            TouchPhase::Moved => {
                let zoom = (self.zoom * (1. + event.delta)).clamp(MIN_ZOOM, MAX_ZOOM);
                let scale = zoom / self.zoom;
                let last = self.last_center.unwrap_or(center);
                self.origin = center - (last - self.origin) * scale;
                self.zoom = zoom;
                self.last_center = Some(center);
            }
            TouchPhase::Ended | TouchPhase::Cancelled => {
                self.last_center = None;
                self.log.push(format!("pinch ended at {:.2}×", self.zoom));
            }
        }
        cx.notify();
    }

    fn reset_zoom(&mut self) {
        self.zoom = 1.;
        self.origin = point(px(16.), px(16.));
    }

    /// Ends a swipe: past the threshold the row slides off and is deleted
    /// (left) or archived (right); short of it the row slides back.
    fn settle(&mut self, row: usize, cx: &mut Context<Self>) {
        let Some(swipe) = self.swipe.take_if(|swipe| swipe.row == row) else {
            return;
        };
        let offset = f32::from(swipe.offset);
        if offset == 0. {
            return;
        }
        let (to, action) = if offset.abs() < SWIPE_COMMIT {
            (0., None)
        } else if offset < 0. {
            (-SWIPE_OUT, Some("deleted"))
        } else {
            (SWIPE_OUT, Some("archived"))
        };
        self.settling.push(Settling {
            row,
            from: swipe.offset,
            to: px(to),
            start: Instant::now(),
            action,
        });
        cx.notify();
    }

    /// Finishes the rows that have arrived, and asks for another frame while
    /// any is still moving.
    fn advance_settling(&mut self, window: &mut Window) {
        let (done, moving): (Vec<_>, Vec<_>) = std::mem::take(&mut self.settling)
            .into_iter()
            .partition(|settling| settling.progress() >= 1.);
        self.settling = moving;
        for settling in done {
            if let Some(action) = settling.action {
                self.inbox.retain(|ix| *ix != settling.row);
                self.log
                    .push(format!("{action} “{}”", MESSAGES[settling.row].1));
            }
        }
        if !self.settling.is_empty() {
            window.request_animation_frame();
        }
    }

    fn render_zoom_pad(&self, cx: &mut Context<Self>) -> AnyElement {
        let view = cx.entity().downgrade();
        let tile = px(TILE * self.zoom);
        let grid = v_flex()
            .absolute()
            .left(self.origin.x)
            .top(self.origin.y)
            .children((0..TILES).map(|row| {
                h_flex().children((0..TILES).map(move |col| {
                    let hue = ((row * TILES + col) as f32 / (TILES * TILES) as f32) * 0.8;
                    div()
                        .size(tile)
                        .flex_none()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(11. * self.zoom))
                        .text_color(gpui::white())
                        .bg(gpui::hsla(
                            hue,
                            0.55,
                            0.45 + 0.1 * ((row + col) % 2) as f32,
                            1.,
                        ))
                        .child(format!("{}{}", (b'A' + row as u8) as char, col + 1))
                }))
            }));

        div()
            .id("zoom-pad")
            .relative()
            .h(px(280.))
            .w_full()
            .overflow_hidden()
            .rounded(cx.theme().radius_lg)
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().muted)
            .on_pinch(cx.listener(|this, event: &PinchEvent, _, cx| this.pinch(event, cx)))
            .on_click(cx.listener(|this, event: &ClickEvent, _, cx| {
                if event.click_count() >= 2 {
                    this.reset_zoom();
                    this.log.push("double tap: zoom reset");
                    cx.notify();
                }
            }))
            .child(grid)
            .child(
                // Records where the pad is, to turn the pinch centre into pad
                // coordinates.
                canvas(
                    |bounds, _, _| bounds,
                    move |bounds, _, _, cx| {
                        let _ = view.update(cx, |this, _| this.pad_bounds = bounds);
                    },
                )
                .absolute()
                .size_full(),
            )
            .child(
                div()
                    .absolute()
                    .bottom_2()
                    .right_2()
                    .px_2()
                    .rounded(cx.theme().radius)
                    .bg(cx.theme().background.opacity(0.85))
                    .text_xs()
                    .child(format!("{:.2}×", self.zoom)),
            )
            .into_any_element()
    }

    fn render_row(&self, row: usize, cx: &mut Context<Self>) -> AnyElement {
        let (from, subject) = MESSAGES[row];
        let offset = self
            .swipe
            .as_ref()
            .filter(|swipe| swipe.row == row)
            .map(|swipe| swipe.offset)
            .or_else(|| {
                self.settling
                    .iter()
                    .find(|settling| settling.row == row)
                    .map(Settling::offset)
            })
            .unwrap_or(px(0.));
        let committing = f32::from(offset).abs() >= SWIPE_COMMIT;
        let action = |label: &'static str, color: gpui::Hsla| {
            div()
                .px_4()
                .text_sm()
                .text_color(gpui::white())
                .when(committing, |this| this.font_semibold())
                .bg(color)
                .h_full()
                .flex()
                .items_center()
                .child(label)
        };

        div()
            .id(("swipe-row", row))
            .relative()
            .h(px(60.))
            .w_full()
            .overflow_hidden()
            .border_b_1()
            .border_color(cx.theme().border)
            // What letting go would do, under the row.
            .child(
                h_flex()
                    .absolute()
                    .inset_0()
                    .justify_between()
                    .when(offset > px(0.), |this| {
                        this.bg(cx.theme().success)
                            .child(action("Archive", cx.theme().success))
                    })
                    .when(offset < px(0.), |this| {
                        this.bg(cx.theme().danger)
                            .child(div())
                            .child(action("Delete", cx.theme().danger))
                    }),
            )
            .child(
                v_flex()
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .left(offset)
                    .w_full()
                    .justify_center()
                    .px_3()
                    .bg(cx.theme().background)
                    .child(div().text_sm().font_semibold().child(from))
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(subject),
                    ),
            )
            // A sideways move drags the row; an up or down move still
            // scrolls the page, as GPUI yields a drag to the scroll axis.
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, event: &MouseDownEvent, _, _| {
                    // A finger catching a row on its way back holds it there.
                    let offset = this
                        .settling
                        .iter()
                        .position(|settling| settling.row == row && settling.action.is_none())
                        .map_or(px(0.), |ix| this.settling.remove(ix).offset());
                    this.swipe = Some(Swipe {
                        row,
                        start_x: event.position.x - offset,
                        offset,
                    });
                }),
            )
            .on_drag(SwipedRow(row), |row, _, _, cx| cx.new(|_| *row))
            .on_drag_move(
                cx.listener(move |this, event: &DragMoveEvent<SwipedRow>, _, cx| {
                    if event.drag(cx).0 != row {
                        return;
                    }
                    if let Some(swipe) = this.swipe.as_mut().filter(|swipe| swipe.row == row) {
                        swipe.offset = event.event.position.x - swipe.start_x;
                        cx.notify();
                    }
                }),
            )
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| this.settle(row, cx)),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| this.settle(row, cx)),
            )
            .on_click(cx.listener(move |this, _, _, cx| {
                this.log.push(format!("open “{}”", MESSAGES[row].1));
                cx.notify();
            }))
            .into_any_element()
    }
}

#[gpui_hot::hot]
impl Render for GesturesScreen {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.advance_settling(window);
        let rows: Vec<_> = self
            .inbox
            .clone()
            .into_iter()
            .map(|row| self.render_row(row, cx))
            .collect();

        v_flex()
            .gap_3()
            .child(
                ui::section("Pinch to zoom", cx)
                    .child(self.render_zoom_pad(cx))
                    .child(ui::value_row(
                        "Zoom / pinches",
                        format!("{:.2}× / {}", self.zoom, self.pinches),
                        cx,
                    ))
                    .child(ui::hint(
                        "Two fingers zoom about the point between them and pan as \
                         they move; one finger scrolls the page. Double-tap resets.",
                        cx,
                    ))
                    .child(
                        Button::new("reset-zoom")
                            .small()
                            .outline()
                            .label("Reset zoom")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.reset_zoom();
                                cx.notify();
                            })),
                    ),
            )
            .child(
                ui::section("Swipe actions", cx)
                    .child(
                        v_flex()
                            .w_full()
                            .rounded(cx.theme().radius_lg)
                            .border_1()
                            .border_color(cx.theme().border)
                            .overflow_hidden()
                            .children(rows)
                            .when(self.inbox.is_empty(), |this| {
                                this.child(
                                    div()
                                        .p_4()
                                        .text_sm()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("Inbox empty."),
                                )
                            }),
                    )
                    .child(ui::hint(
                        "Swipe a row left to delete it or right to archive it; let go \
                         before the label turns bold to keep it. A tap opens it.",
                        cx,
                    ))
                    .child(
                        Button::new("restore-rows")
                            .small()
                            .outline()
                            .label("Restore all")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.inbox = (0..MESSAGES.len()).collect();
                                this.swipe = None;
                                this.settling.clear();
                                cx.notify();
                            })),
                    ),
            )
            .child(ui::section("Table columns", cx).child(ui::hint(
                "On the Tables screen's DataTable, drag a column's edge in the \
                     header to resize it, or long-press a header and drag to move \
                     the column.",
                cx,
            )))
            .child(self.log.render(cx))
    }
}
