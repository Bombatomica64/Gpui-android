use std::time::Duration;

use gpui::{Context, IntoElement, ParentElement, Render, Styled, Task, Window, div, px};
use gpui_kit::component::{
    IconName, Size,
    alert::Alert,
    button::Button,
    h_flex,
    marker::{Marker, MarkerContent, MarkerLoadingStyle, MarkerVariant},
    progress::{Progress, ProgressCircle},
    separator::Separator,
    shimmer::ShimmerText,
    skeleton::Skeleton,
    spinner::Spinner,
    status_bar::StatusBar,
    v_flex,
};

use crate::ui::{self, EventLog, prelude::*};

pub struct FeedbackScreen {
    progress: f32,
    indeterminate: bool,
    running: Option<Task<()>>,
    closed_alerts: Vec<&'static str>,
    loading: bool,
    log: EventLog,
}

impl FeedbackScreen {
    pub fn new(_: &mut Window, _: &mut Context<Self>) -> Self {
        Self {
            progress: 35.,
            indeterminate: false,
            running: None,
            closed_alerts: vec![],
            loading: true,
            log: EventLog::default(),
        }
    }

    fn toggle_run(&mut self, cx: &mut Context<Self>) {
        if self.running.take().is_some() {
            self.log.push("progress stopped");
        } else {
            self.log.push("progress started");
            self.running = Some(cx.spawn(async move |this, cx| {
                loop {
                    cx.background_executor()
                        .timer(Duration::from_millis(100))
                        .await;
                    let alive = this.update(cx, |this, cx| {
                        this.progress = if this.progress >= 100. {
                            0.
                        } else {
                            this.progress + 2.
                        };
                        cx.notify();
                    });
                    if alive.is_err() {
                        break;
                    }
                }
            }));
        }
        cx.notify();
    }
}

impl Render for FeedbackScreen {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let alert = |this: &Self, id: &'static str, alert: Alert, cx: &mut Context<Self>| {
            alert
                .visible(!this.closed_alerts.contains(&id))
                .on_close(cx.listener(move |this, _, _, cx| {
                    this.closed_alerts.push(id);
                    this.log.push(format!("alert {id} closed"));
                    cx.notify();
                }))
        };
        v_flex()
            .gap_3()
            .child(
                // Unstyled, exactly as in Kit's Shimmer story (longbridge/gpui-kit#3327):
                // in dark mode the first line shows no sweep on Kit 0.7.0.
                ui::section("ShimmerText defaults", cx)
                    .child(ShimmerText::new("Thinking…"))
                    .child(ShimmerText::new("Searching the current project…").text_color(cx.theme().muted_foreground))
                    .child(ui::hint("Default text color, then muted. Switch to dark mode to compare.", cx)),
            )
            .child(
                ui::section("Alert", cx)
                    .child(alert(self, "info", Alert::info("info", "Maintenance starts Friday at 22:00 UTC.").title("Scheduled maintenance"), cx))
                    .child(alert(self, "success", Alert::success("success", "Your changes were saved."), cx))
                    .child(alert(self, "warning", Alert::warning("warning", "Storage is 90% full; consider cleaning up old files before uploading more."), cx))
                    .child(alert(self, "error", Alert::error("error", "Upload failed: network unreachable."), cx))
                    .child(Alert::info("banner", "Banner alert spans the full width.").banner())
                    .child(
                        Button::new("restore-alerts").small().outline().label("Restore closed alerts").on_click(cx.listener(
                            |this, _, _, cx| {
                                this.closed_alerts.clear();
                                this.log.push("alerts restored");
                                cx.notify();
                            },
                        )),
                    ),
            )
            .child(
                ui::section("Progress", cx)
                    .child(Progress::new("progress").value(self.progress).loading(self.indeterminate))
                    .child(
                        h_flex()
                            .gap_4()
                            .child(ProgressCircle::new("circle").value(self.progress).loading(self.indeterminate).with_size(Size::Large))
                            .child(ProgressCircle::new("circle-small").value(self.progress).small())
                            .child(ui::value_row("Value", format!("{:.0}%", self.progress), cx)),
                    )
                    .child(
                        h_flex()
                            .gap_1()
                            .child(Button::new("run").small().outline().label(if self.running.is_some() { "Stop" } else { "Animate" }).on_click(
                                cx.listener(|this, _, _, cx| this.toggle_run(cx)),
                            ))
                            .child(Button::new("indeterminate").small().outline().selected(self.indeterminate).label("Indeterminate").on_click(
                                cx.listener(|this, _, _, cx| {
                                    this.indeterminate = !this.indeterminate;
                                    this.log.push(format!("indeterminate -> {}", this.indeterminate));
                                    cx.notify();
                                }),
                            )),
                    ),
            )
            .child(
                ui::section("Spinner", cx).child(
                    h_flex()
                        .gap_4()
                        .child(Spinner::new().small())
                        .child(Spinner::new())
                        .child(Spinner::new().large().color(cx.theme().primary))
                        .child(Spinner::new().with_size(Size::Size(px(40.))).icon(IconName::LoaderCircle)),
                ),
            )
            .child(
                ui::section("Skeleton / ShimmerText", cx)
                    .child(
                        h_flex()
                            .gap_3()
                            .child(Skeleton::new().size_12().rounded_full())
                            .child(v_flex().flex_1().gap_2().child(Skeleton::new().w_full().h_4()).child(Skeleton::new().secondary().w(px(160.)).h_4())),
                    )
                    // Kit's default highlight mixes the text color toward `foreground`,
                    // which is invisible on `foreground` text in dark mode.
                    .child(
                        ShimmerText::new("Thinking about your question…")
                            .text_color(cx.theme().muted_foreground)
                            .highlight_color(cx.theme().foreground),
                    )
                    .child(
                        ShimmerText::new("Shimmer once")
                            .once(true)
                            .id("shimmer-once")
                            .text_color(cx.theme().muted_foreground)
                            .highlight_color(cx.theme().foreground),
                    ),
            )
            .child(
                ui::section("Marker", cx)
                    .child(Marker::new().with_variant(MarkerVariant::Separator).content(MarkerContent::new().child("Earlier messages")))
                    .child(Marker::new().loading(self.loading).content(MarkerContent::new().child("Generating answer")))
                    .child(
                        Marker::new()
                            .loading(self.loading)
                            .with_loading_style(MarkerLoadingStyle::Shimmer)
                            .with_variant(MarkerVariant::Border)
                            .content(MarkerContent::new().child("Searching the web")),
                    )
                    .child(Button::new("marker-loading").small().outline().label("Toggle loading").on_click(cx.listener(
                        |this, _, _, cx| {
                            this.loading = !this.loading;
                            cx.notify();
                        },
                    ))),
            )
            .child(
                ui::section("StatusBar", cx).child(
                    div().border_1().border_color(cx.theme().border).child(
                        StatusBar::new()
                            .left(Button::new("branch").ghost().xsmall().icon(IconName::Github).label("main"))
                            .left(Separator::vertical().h_3())
                            .left(div().text_xs().child("0 errors"))
                            .right(Button::new("position").ghost().xsmall().label("Ln 12, Col 34").on_click(cx.listener(
                                |this, _, _, cx| {
                                    this.log.push("status bar item tapped");
                                    cx.notify();
                                },
                            ))),
                    ),
                ),
            )
            .child(self.log.render(cx))
    }
}
