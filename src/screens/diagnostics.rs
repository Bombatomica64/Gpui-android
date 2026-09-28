//! Runtime facts that are actually observable; nothing here is estimated.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use gpui::{Context, IntoElement, ParentElement, Render, Styled, Task, Window, div, prelude::*, px};
use gpui_kit::component::{WindowExt as _, button::Button, h_flex, v_flex};

use crate::diagnostics as host;
use crate::ui::{self, prelude::*};

pub struct DiagnosticsScreen {
    show_hud: bool,
    /// Measure render cadence by requesting a frame after every frame.
    animating: bool,
    frames: VecDeque<Instant>,
    refresh: Option<Task<()>>,
}

impl DiagnosticsScreen {
    pub fn new(_: &mut Window, cx: &mut Context<Self>) -> Self {
        // Re-render twice a second so platform-reported values stay current without
        // forcing continuous frames.
        let refresh = cx.spawn(async move |this, cx| loop {
            cx.background_executor().timer(Duration::from_millis(500)).await;
            if this.update(cx, |_, cx| cx.notify()).is_err() {
                break;
            }
        });
        Self { show_hud: true, animating: false, frames: VecDeque::new(), refresh: Some(refresh) }
    }

    fn cadence(&self) -> Option<(f32, f32, f32)> {
        if self.frames.len() < 3 {
            return None;
        }
        let intervals: Vec<f32> = self
            .frames
            .iter()
            .zip(self.frames.iter().skip(1))
            .map(|(a, b)| (*b - *a).as_secs_f32() * 1000.)
            .collect();
        let avg = intervals.iter().sum::<f32>() / intervals.len() as f32;
        let worst = intervals.iter().cloned().fold(0., f32::max);
        Some((1000. / avg, avg, worst))
    }
}

impl Render for DiagnosticsScreen {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let _ = &self.refresh;
        if self.animating {
            self.frames.push_back(Instant::now());
            while self.frames.len() > 120 {
                self.frames.pop_front();
            }
            window.request_animation_frame();
        }
        let viewport = window.viewport_size();
        let visual = window.visual_viewport_bounds();
        let (keyboard, keyboard_px) = host::keyboard();
        let focused_input = window.focused_input(cx).is_some();
        let focused = window.focused(cx).map(|handle| format!("{handle:?}"));
        let touch = host::last_touch();
        let cadence = self.cadence();

        let facts: Vec<(&str, String)> = vec![
            ("GPUI Kit / GPUI", format!("{} / {}", crate::GPUI_KIT_VERSION, crate::GPUI_VERSION)),
            ("gpui-mobile", crate::GPUI_MOBILE_REVISION.into()),
            ("Android API", host::api_level().map(|l| l.to_string()).unwrap_or_else(|| "unknown".into())),
            ("Viewport (logical)", format!("{:.1} × {:.1}", viewport.width.as_f32(), viewport.height.as_f32())),
            ("Scale factor", format!("{:.3}", window.scale_factor())),
            (
                "Viewport (physical)",
                format!(
                    "{:.0} × {:.0} px",
                    viewport.width.as_f32() * window.scale_factor(),
                    viewport.height.as_f32() * window.scale_factor()
                ),
            ),
            (
                "Visual viewport",
                format!(
                    "origin {:.0},{:.0} size {:.0} × {:.0}",
                    visual.origin.x.as_f32(),
                    visual.origin.y.as_f32(),
                    visual.size.width.as_f32(),
                    visual.size.height.as_f32()
                ),
            ),
            ("Keyboard (host)", format!("visible={keyboard}, {keyboard_px} px")),
            ("Focused text input", focused_input.to_string()),
            ("Focused handle", focused.unwrap_or_else(|| "none".into())),
            ("Screen scroll y", format!("{} pt", host::screen_scroll())),
            ("Activity foreground", host::is_foreground().to_string()),
            ("Window active (GPUI)", window.is_window_active().to_string()),
            ("Appearance", format!("{:?}", window.appearance())),
            ("Host MotionEvents", host::touch_events().to_string()),
            (
                "Last touch (physical px)",
                touch
                    .map(|t| format!("action {} · {} ptr · {:.0},{:.0}", t.action, t.pointers, t.x, t.y))
                    .unwrap_or_else(|| "none".into()),
            ),
            (
                "Render cadence",
                match (self.animating, cadence) {
                    (true, Some((fps, avg, worst))) => {
                        format!("{fps:.1} fps · avg {avg:.1} ms · worst {worst:.1} ms (last {} frames)", self.frames.len())
                    }
                    (true, None) => "measuring…".into(),
                    (false, _) => "off (start measuring below)".into(),
                },
            ),
        ];

        v_flex()
            .relative()
            .gap_3()
            .min_h(px(520.))
            .child(
                ui::section("Runtime", cx)
                    .children(facts.into_iter().map(|(label, value)| ui::value_row(label, value, cx)))
                    .child(ui::hint(
                        "Values refresh every 0.5 s. Open the keyboard from Text Input and come back \
                         to compare the host keyboard height with GPUI's viewport.",
                        cx,
                    )),
            )
            .child(
                ui::section("Frame timing", cx)
                    .child(
                        h_flex()
                            .flex_wrap()
                            .gap_1()
                            .child(Button::new("measure").small().outline().selected(self.animating).label(
                                if self.animating { "Stop measuring" } else { "Measure render cadence" },
                            ).on_click(cx.listener(|this, _, window, cx| {
                                this.animating = !this.animating;
                                this.frames.clear();
                                log::info!("event: render cadence measurement -> {}", this.animating);
                                window.request_animation_frame();
                                cx.notify();
                            })))
                            .child(Button::new("hud").small().outline().selected(self.show_hud).label("gpui-fps HUD").on_click(
                                cx.listener(|this, _, _, cx| {
                                    this.show_hud = !this.show_hud;
                                    cx.notify();
                                }),
                            )),
                    )
                    .child(ui::hint(
                        "Cadence = interval between consecutive renders while continuously \
                         requesting frames (vsync-paced by gpui-mobile). The HUD (top-right of \
                         this card area) is gpui-fps: presented frames and Window::draw time from \
                         GPUI's frame trace. Its GPU/memory rows have no Android backend.",
                        cx,
                    )),
            )
            .when(self.show_hud, |this| {
                this.child(div().absolute().top_0().right_0().size_full().child(gpui_fps::fps_monitor(window, cx)))
            })
    }
}
