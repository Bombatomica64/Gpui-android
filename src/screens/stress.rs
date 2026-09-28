//! Many elements plus continuously changing state, to surface rendering cost on-device.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use gpui::{
    AppContext as _, Context, Entity, IntoElement, ParentElement, Render, SharedString, Styled,
    Window, div, px, uniform_list,
};
use gpui_kit::component::{
    button::Button,
    chart::LineChart,
    h_flex,
    input::{Input, InputState},
    progress::Progress,
    v_flex,
};

use crate::ui::{self, prelude::*};

#[derive(Clone)]
struct Point {
    x: SharedString,
    y: f64,
}

pub struct StressScreen {
    running: bool,
    frame: u64,
    started: Option<Instant>,
    render_times: VecDeque<Duration>,
    intervals: VecDeque<Duration>,
    last_frame: Option<Instant>,
    series: Vec<Point>,
    inputs: Vec<Entity<InputState>>,
}

impl StressScreen {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            running: false,
            frame: 0,
            started: None,
            render_times: VecDeque::new(),
            intervals: VecDeque::new(),
            last_frame: None,
            series: (0..60)
                .map(|i| Point {
                    x: i.to_string().into(),
                    y: 50.0,
                })
                .collect(),
            inputs: (0..4)
                .map(|i| {
                    cx.new(|cx| {
                        InputState::new(window, cx).default_value(format!("Input {i}: 日本語 😀"))
                    })
                })
                .collect(),
        }
    }

    fn reset(&mut self) {
        self.frame = 0;
        self.started = None;
        self.render_times.clear();
        self.intervals.clear();
        self.last_frame = None;
        self.series = (0..60)
            .map(|i| Point {
                x: i.to_string().into(),
                y: 50.0,
            })
            .collect();
    }

    fn stats(samples: &VecDeque<Duration>) -> String {
        if samples.is_empty() {
            return "—".into();
        }
        let ms: Vec<f32> = samples.iter().map(|d| d.as_secs_f32() * 1000.).collect();
        let avg = ms.iter().sum::<f32>() / ms.len() as f32;
        let worst = ms.iter().cloned().fold(0., f32::max);
        format!("avg {avg:.1} ms · worst {worst:.1} ms")
    }
}

impl Render for StressScreen {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let render_start = Instant::now();
        if self.running {
            let now = Instant::now();
            if let Some(last) = self.last_frame {
                self.intervals.push_back(now - last);
                while self.intervals.len() > 120 {
                    self.intervals.pop_front();
                }
            }
            self.last_frame = Some(now);
            self.frame += 1;
            // Deterministic signal: sum of two sines, one sample per frame.
            let t = self.frame as f64 / 10.0;
            self.series.remove(0);
            self.series.push(Point {
                x: self.frame.to_string().into(),
                y: 50.0 + 30.0 * t.sin() + 10.0 * (t * 3.7).cos(),
            });
            window.request_animation_frame();
        }
        let fps = if self.intervals.is_empty() {
            None
        } else {
            let avg = self.intervals.iter().map(|d| d.as_secs_f32()).sum::<f32>()
                / self.intervals.len() as f32;
            Some(1.0 / avg)
        };
        let frame = self.frame;
        let hue_shift = (frame % 360) as f32 / 360.0;

        let controls = h_flex()
            .gap_2()
            .child(
                Button::new("start")
                    .primary()
                    .label("Start")
                    .disabled(self.running)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.running = true;
                        this.started.get_or_insert_with(Instant::now);
                        this.last_frame = None;
                        log::info!("event: stress start");
                        cx.notify();
                    })),
            )
            .child(
                Button::new("stop")
                    .outline()
                    .label("Stop")
                    .disabled(!self.running)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.running = false;
                        log::info!("event: stress stop after {} frames", this.frame);
                        cx.notify();
                    })),
            )
            .child(
                Button::new("reset")
                    .outline()
                    .label("Reset")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.running = false;
                        this.reset();
                        log::info!("event: stress reset");
                        cx.notify();
                    })),
            );

        let buttons = (0..120).map(|i| {
            Button::new(("stress-btn", i))
                .xsmall()
                .outline()
                .selected((frame as usize + i) % 17 == 0)
                .label(format!("{}", (i as u64 + frame) % 1000))
        });
        let texts = (0..400).map(|i| {
            div()
                .text_xs()
                .text_color(gpui::hsla(
                    (hue_shift + i as f32 / 400.) % 1.,
                    0.6,
                    0.45,
                    1.,
                ))
                .child(format!("t{}", (i as u64 * 7 + frame) % 997))
        });

        let body = v_flex()
            .gap_3()
            .child(
                ui::section("Controls", cx)
                    .child(controls)
                    .child(ui::value_row("Frames", frame.to_string(), cx))
                    .child(ui::value_row(
                        "FPS (render cadence)",
                        fps.map(|f| format!("{f:.1}")).unwrap_or_else(|| "—".into()),
                        cx,
                    ))
                    .child(ui::value_row(
                        "Frame interval",
                        Self::stats(&self.intervals),
                        cx,
                    ))
                    .child(ui::value_row(
                        "StressScreen::render()",
                        Self::stats(&self.render_times),
                        cx,
                    ))
                    .child(ui::value_row(
                        "Elapsed",
                        self.started
                            .map(|s| format!("{:.1} s", s.elapsed().as_secs_f32()))
                            .unwrap_or_else(|| "—".into()),
                        cx,
                    ))
                    .child(Progress::new("stress-progress").value((frame % 100) as f32)),
            )
            .child(
                ui::section("Live chart (60 points, one per frame)", cx).child(
                    div().h(px(160.)).child(
                        LineChart::new(self.series.clone())
                            .x(|p| p.x.clone())
                            .y(|p| p.y)
                            .stroke(cx.theme().primary)
                            .y_domain(0., 100.)
                            .x_axis(false)
                            .linear()
                            .id("stress-chart"),
                    ),
                ),
            )
            .child(
                ui::section("120 buttons", cx)
                    .child(h_flex().flex_wrap().gap_1().children(buttons)),
            )
            .child(
                ui::section("400 text nodes", cx)
                    .child(h_flex().flex_wrap().gap_1().children(texts)),
            )
            .child(
                ui::section("Inputs", cx)
                    .children(self.inputs.iter().map(|input| Input::new(input))),
            )
            .child(
                ui::section("Long list (5,000 rows, virtualized, fixed height)", cx).child(
                    uniform_list(
                        "stress-list",
                        5_000,
                        cx.processor(move |this, range: std::ops::Range<usize>, _, cx| {
                            range
                                .map(|i| {
                                    h_flex()
                                        .h(px(28.))
                                        .px_2()
                                        .text_xs()
                                        .border_b_1()
                                        .border_color(cx.theme().border)
                                        .child(format!("row {i} · frame {}", this.frame))
                                })
                                .collect()
                        }),
                    )
                    .h(px(240.)),
                ),
            );

        let elapsed = render_start.elapsed();
        if self.running {
            self.render_times.push_back(elapsed);
            while self.render_times.len() > 120 {
                self.render_times.pop_front();
            }
        }
        body
    }
}
