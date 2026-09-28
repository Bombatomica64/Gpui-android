//! Raw gesture observation: what GPUI's touch pipeline delivers on this device.

use std::time::Instant;

use gpui::{
    Bounds, ClickEvent, Context, InteractiveElement, IntoElement, LongPressEvent, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, ParentElement, PinchEvent, Pixels, Point, Render, StatefulInteractiveElement,
    Styled, TouchDragEvent, TouchPhase, Window, canvas, div, point, prelude::*, px,
};
use gpui_kit::component::{button::Button, h_flex, v_flex};

use crate::ui::{self, EventLog, prelude::*};

#[derive(Default)]
struct Counters {
    touch_move: usize,
    mouse_down: usize,
    mouse_up: usize,
    clicks: usize,
    double: usize,
    long_press: usize,
    drag: usize,
    pinch: usize,
}

pub struct TouchScreen {
    counters: Counters,
    pad_bounds: Bounds<Pixels>,
    marker: Point<Pixels>,
    last_touch: Option<Point<Pixels>>,
    rapid: usize,
    rapid_started: Option<Instant>,
    log: EventLog,
}

impl TouchScreen {
    pub fn new(_: &mut Window, _: &mut Context<Self>) -> Self {
        Self {
            counters: Counters::default(),
            pad_bounds: Bounds::default(),
            marker: point(px(60.), px(60.)),
            last_touch: None,
            rapid: 0,
            rapid_started: None,
            log: EventLog::default(),
        }
    }

    fn local(&self, position: Point<Pixels>) -> Point<Pixels> {
        position - self.pad_bounds.origin
    }
}

impl Render for TouchScreen {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity().downgrade();
        let c = &self.counters;
        let rate = self.rapid_started.map(|start| {
            let secs = start.elapsed().as_secs_f32().max(0.001);
            self.rapid as f32 / secs
        });

        // Observe every event kind inside the pad without claiming any of them, so
        // the counters show exactly what the platform + GPUI recognizer produced.
        let observer = canvas(
            |bounds, _, _| bounds,
            move |bounds, _, window, _| {
                let inside = move |p: Point<Pixels>| bounds.contains(&p);
                let v = view.clone();
                // Elements never see raw TouchEvents: GPUI turns a touch into mouse
                // down/move/up (pressed) plus recognized gestures.
                window.on_mouse_event(move |e: &MouseMoveEvent, phase, _, cx| {
                    if !phase.bubble() || !inside(e.position) || e.pressed_button.is_none() {
                        return;
                    }
                    let _ = v.update(cx, |this, cx| {
                        this.pad_bounds = bounds;
                        this.counters.touch_move += 1;
                        this.marker = this.local(e.position);
                        this.last_touch = Some(this.marker);
                        cx.notify();
                    });
                });
                let v = view.clone();
                window.on_mouse_event(move |e: &MouseDownEvent, phase, _, cx| {
                    if phase.bubble() && inside(e.position) {
                        let _ = v.update(cx, |this, cx| {
                            this.pad_bounds = bounds;
                            this.counters.mouse_down += 1;
                            this.marker = this.local(e.position);
                            this.last_touch = Some(this.marker);
                            this.log.push(format!("down at {:?} (click_count {})", this.marker, e.click_count));
                            cx.notify();
                        });
                    }
                });
                let v = view.clone();
                window.on_mouse_event(move |e: &MouseUpEvent, phase, _, cx| {
                    if phase.bubble() && inside(e.position) {
                        let _ = v.update(cx, |this, cx| {
                            this.counters.mouse_up += 1;
                            cx.notify();
                        });
                    }
                });
                let v = view.clone();
                window.on_mouse_event(move |e: &LongPressEvent, phase, window, cx| {
                    if phase.bubble() && inside(e.start_position) {
                        // GPUI only continues a long press (Moved/Ended) that a
                        // handler claims; unclaimed, the touch ends as a tap.
                        if matches!(e.phase, TouchPhase::Started) {
                            window.prevent_default();
                        }
                        let _ = v.update(cx, |this, cx| {
                            if matches!(e.phase, TouchPhase::Started) {
                                this.counters.long_press += 1;
                            }
                            if !matches!(e.phase, TouchPhase::Moved) {
                                this.log.push(format!("long press {:?} at {:?}", e.phase, this.local(e.position)));
                            }
                            cx.notify();
                        });
                    }
                });
                let v = view.clone();
                window.on_mouse_event(move |e: &TouchDragEvent, phase, _, cx| {
                    if phase.bubble() && inside(e.start_position) {
                        let _ = v.update(cx, |this, cx| {
                            this.counters.drag += 1;
                            cx.notify();
                        });
                    }
                });
                let v = view.clone();
                window.on_mouse_event(move |e: &PinchEvent, phase, _, cx| {
                    if phase.bubble() && inside(e.position) {
                        let _ = v.update(cx, |this, cx| {
                            this.counters.pinch += 1;
                            this.log.push(format!("pinch delta {:.3}", e.delta));
                            cx.notify();
                        });
                    }
                });
            },
        )
        .absolute()
        .size_full();

        let pad = div()
            .id("touch-pad")
            .relative()
            .h(px(260.))
            .w_full()
            .rounded(cx.theme().radius_lg)
            .border_2()
            .border_dashed()
            .border_color(cx.theme().border)
            .bg(cx.theme().muted)
            .on_click(cx.listener(|this, event: &ClickEvent, _, cx| {
                this.counters.clicks += 1;
                if event.click_count() >= 2 {
                    this.counters.double += 1;
                    this.log.push(format!("double tap (click_count={})", event.click_count()));
                } else {
                    this.log.push("tap");
                }
                cx.notify();
            }))
            .child(observer)
            .child(
                div()
                    .absolute()
                    .left(self.marker.x - px(16.))
                    .top(self.marker.y - px(16.))
                    .size(px(32.))
                    .rounded_full()
                    .bg(cx.theme().primary.opacity(0.7)),
            )
            .child(
                div()
                    .absolute()
                    .bottom_2()
                    .left_2()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child("Tap · double-tap · long-press · drag · pinch"),
            );

        let row = |label: &str, value: usize| ui::value_row(label.to_string(), value.to_string(), cx);

        v_flex()
            .gap_3()
            .child(ui::section("Gesture pad", cx).child(pad).child(ui::value_row(
                "Last touch (pad-local)",
                format!("{:?}", self.last_touch),
                cx,
            )))
            .child(
                ui::section("Counters", cx)
                    .child(
                        h_flex()
                            .flex_wrap()
                            .gap_x_4()
                            .child(row("pressed move", c.touch_move))
                            .child(row("mouse down", c.mouse_down))
                            .child(row("mouse up", c.mouse_up))
                            .child(row("click", c.clicks))
                            .child(row("double tap", c.double))
                            .child(row("long press", c.long_press))
                            .child(row("touch drag", c.drag))
                            .child(row("pinch", c.pinch)),
                    )
                    .child(ui::value_row(
                        "Host MotionEvents (whole app)",
                        crate::diagnostics::touch_events().to_string(),
                        cx,
                    ))
                    .child(Button::new("reset-counters").small().outline().label("Reset").on_click(cx.listener(
                        |this, _, _, cx| {
                            this.counters = Counters::default();
                            this.rapid = 0;
                            this.rapid_started = None;
                            this.log.clear();
                            cx.notify();
                        },
                    ))),
            )
            .child(
                ui::section("Rapid taps", cx)
                    .child(
                        Button::new("rapid")
                            .primary()
                            .large()
                            .label(format!("Tap fast ({})", self.rapid))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.rapid += 1;
                                this.rapid_started.get_or_insert_with(Instant::now);
                                cx.notify();
                            })),
                    )
                    .child(ui::value_row(
                        "Taps / rate",
                        format!("{} / {}", self.rapid, rate.map(|r| format!("{r:.1} per s")).unwrap_or_else(|| "—".into())),
                        cx,
                    ))
                    .child(ui::hint("Every physical tap should count exactly once.", cx)),
            )
            .child(
                ui::section("Tap targets near edges", cx).child(
                    h_flex()
                        .justify_between()
                        .children(["left", "center", "right"].map(|edge| {
                            div()
                                .id(edge)
                                .px_3()
                                .py_2()
                                .border_1()
                                .border_color(cx.theme().border)
                                .rounded(cx.theme().radius)
                                .active(|this| this.bg(cx.theme().accent))
                                .child(edge)
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.log.push(format!("edge target {edge}"));
                                    cx.notify();
                                }))
                        })),
                ),
            )
            .child(self.log.render(cx))
            .child(ui::hint(
                "Fling / stop-fling / nested scrolling are exercised on the Scroll Stress screen.",
                cx,
            ))
    }
}
