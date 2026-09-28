use std::rc::Rc;
use std::time::Duration;

use gpui::{
    AppContext as _, Context, Entity, IntoElement, ParentElement, Render, SharedString, Styled,
    Window, div, prelude::*, px,
};
use gpui_kit::component::{
    Icon, IconName, Placement, WindowExt as _,
    button::{Button, ButtonVariant},
    h_flex,
    input::{Input, InputState},
    notification::{Notification, NotificationType},
    v_flex,
};

use crate::ui::{self, EventLog, prelude::*};

pub struct DialogsScreen {
    input: Entity<InputState>,
    opened: usize,
    closed: usize,
    log: EventLog,
}

impl DialogsScreen {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            input: cx.new(|cx| InputState::new(window, cx).placeholder("Type inside the overlay")),
            opened: 0,
            closed: 0,
            log: EventLog::default(),
        }
    }

    fn opened(&mut self, what: &str, cx: &mut Context<Self>) {
        self.opened += 1;
        self.log.push(format!("open {what}"));
        cx.notify();
    }

    fn on_close_handler(&self, what: &'static str, cx: &mut Context<Self>) -> Rc<dyn Fn(&mut Window, &mut gpui::App)> {
        let view = cx.entity();
        Rc::new(move |_, cx| {
            view.update(cx, |this, cx| {
                this.closed += 1;
                this.log.push(format!("closed {what}"));
                cx.notify();
            })
        })
    }

    fn open_basic(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let on_close = self.on_close_handler("basic dialog", cx);
        window.open_dialog(cx, move |dialog, _, _| {
            dialog
                .title("Basic dialog")
                .child("Tap outside, press back, or use the close button.")
                .on_close({
                    let on_close = on_close.clone();
                    move |_, window, cx| on_close(window, cx)
                })
        });
        self.opened("basic dialog", cx);
    }

    fn open_form(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let input = self.input.clone();
        let view = cx.entity();
        window.open_dialog(cx, move |dialog, _, _| {
            let view = view.clone();
            let input = input.clone();
            dialog
                .title("Dialog with input")
                .child(
                    v_flex()
                        .gap_2()
                        .child("The keyboard must not cover this field.")
                        .child(Input::new(&input)),
                )
                .footer(
                    h_flex().gap_2().justify_end().child(
                        Button::new("dialog-save").primary().label("Save").on_click({
                            let view = view.clone();
                            let input = input.clone();
                            move |_, window, cx| {
                                let value = input.read(cx).value();
                                view.update(cx, |this, cx| {
                                    this.log.push(format!("dialog saved {value:?}"));
                                    cx.notify();
                                });
                                window.close_dialog(cx);
                            }
                        }),
                    ),
                )
        });
        self.opened("dialog with input", cx);
    }

    fn open_long(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        window.open_dialog(cx, move |dialog, _, _| {
            dialog.title("Long content").child(
                div()
                    .id("dialog-scroll")
                    .max_h(px(360.))
                    .overflow_y_scroll()
                    .child(v_flex().gap_2().children((1..=60).map(|i| {
                        div().child(format!("Row {i}: the page behind must not scroll with this list."))
                    }))),
            )
        });
        self.opened("long dialog", cx);
    }

    fn open_nested(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        window.open_dialog(cx, move |dialog, _, _| {
            dialog.title("First dialog").child(
                Button::new("open-second").outline().label("Open a second dialog").on_click(|_, window, cx| {
                    window.open_dialog(cx, |dialog, _, _| {
                        dialog.title("Second dialog").child("Back closes only this one.")
                    });
                }),
            )
        });
        self.opened("nested dialog", cx);
    }

    fn open_alert(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let view = cx.entity();
        window.open_alert_dialog(cx, move |alert, _, cx| {
            let ok_view = view.clone();
            let cancel_view = view.clone();
            alert
                .icon(Icon::new(IconName::TriangleAlert).text_color(cx.theme().danger))
                .title("Delete file?")
                .description("This action cannot be undone. The file will be removed permanently.")
                .confirm()
                .ok_text("Delete")
                .ok_variant(ButtonVariant::Danger)
                .on_ok(move |_, window, cx| {
                    ok_view.update(cx, |this, cx| {
                        this.log.push("alert: OK");
                        cx.notify();
                    });
                    window.push_notification(Notification::success("Deleted"), cx);
                    true
                })
                .on_cancel(move |_, _, cx| {
                    cancel_view.update(cx, |this, cx| {
                        this.log.push("alert: Cancel");
                        cx.notify();
                    });
                    true
                })
        });
        self.opened("alert dialog", cx);
    }

    fn open_sheet(&mut self, placement: Placement, window: &mut Window, cx: &mut Context<Self>) {
        let input = self.input.clone();
        let on_close = self.on_close_handler("sheet", cx);
        let size = match placement {
            Placement::Left | Placement::Right => px(300.),
            _ => px(360.),
        };
        window.open_sheet_at(placement, cx, move |sheet, _, _| {
            let on_close = on_close.clone();
            sheet
                .title(format!("Sheet ({placement:?})"))
                .size(size)
                .on_close(move |_, window, cx| on_close(window, cx))
                .child(
                    v_flex()
                        .gap_2()
                        .child("Swipe/tap outside to dismiss, or press back.")
                        .child(Input::new(&input))
                        .children((1..=20).map(|i| div().child(format!("Sheet row {i}")))),
                )
        });
        self.opened(&format!("sheet {placement:?}"), cx);
    }

    fn notify(&mut self, kind: NotificationType, window: &mut Window, cx: &mut Context<Self>) {
        let message: SharedString = format!("{kind:?} notification at {}", ui::timestamp()).into();
        window.push_notification((kind, message), cx);
        self.log.push(format!("notification {kind:?}"));
        cx.notify();
    }

    fn sequential(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Open and close five dialogs, one per 300 ms, to catch leaks and stale focus.
        self.log.push("sequential open/close x5 started");
        cx.spawn_in(window, async move |this, cx| {
            for i in 0..5 {
                let _ = cx.update(|window, cx| {
                    window.open_dialog(cx, move |dialog, _, _| {
                        dialog.title(format!("Sequential {}", i + 1))
                    })
                });
                cx.background_executor().timer(Duration::from_millis(300)).await;
                let _ = cx.update(|window, cx| window.close_dialog(cx));
                cx.background_executor().timer(Duration::from_millis(150)).await;
            }
            let _ = this.update(cx, |this, cx| {
                this.opened += 5;
                this.closed += 5;
                this.log.push("sequential open/close x5 finished");
                cx.notify();
            });
        })
        .detach();
    }
}

impl Render for DialogsScreen {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let has_dialog = window.has_active_dialog(cx);
        let has_sheet = window.has_active_sheet(cx);
        let button = |id: &'static str, label: &'static str| Button::new(id).outline().label(label);
        v_flex()
            .gap_3()
            .child(
                ui::section("Dialog / AlertDialog", cx)
                    .child(
                        h_flex()
                            .flex_wrap()
                            .gap_2()
                            .child(button("basic", "Basic").on_click(cx.listener(|this, _, w, cx| this.open_basic(w, cx))))
                            .child(button("form", "With input").on_click(cx.listener(|this, _, w, cx| this.open_form(w, cx))))
                            .child(button("long", "Long content").on_click(cx.listener(|this, _, w, cx| this.open_long(w, cx))))
                            .child(button("nested", "Nested").on_click(cx.listener(|this, _, w, cx| this.open_nested(w, cx))))
                            .child(button("alert", "AlertDialog").on_click(cx.listener(|this, _, w, cx| this.open_alert(w, cx))))
                            .child(button("sequential", "Open/close ×5").on_click(cx.listener(|this, _, w, cx| this.sequential(w, cx)))),
                    )
                    .child(ui::value_row(
                        "Opened / closed",
                        format!("{} / {} (dialog active: {has_dialog}, sheet active: {has_sheet})", self.opened, self.closed),
                        cx,
                    )),
            )
            .child(
                ui::section("Sheet", cx).child(
                    h_flex()
                        .flex_wrap()
                        .gap_2()
                        .child(button("sheet-bottom", "Bottom").on_click(cx.listener(|this, _, w, cx| this.open_sheet(Placement::Bottom, w, cx))))
                        .child(button("sheet-top", "Top").on_click(cx.listener(|this, _, w, cx| this.open_sheet(Placement::Top, w, cx))))
                        .child(button("sheet-left", "Left").on_click(cx.listener(|this, _, w, cx| this.open_sheet(Placement::Left, w, cx))))
                        .child(button("sheet-right", "Right").on_click(cx.listener(|this, _, w, cx| this.open_sheet(Placement::Right, w, cx)))),
                ),
            )
            .child(
                ui::section("Notification", cx)
                    .child(
                        h_flex()
                            .flex_wrap()
                            .gap_2()
                            .child(button("n-info", "Info").on_click(cx.listener(|this, _, w, cx| this.notify(NotificationType::Info, w, cx))))
                            .child(button("n-success", "Success").on_click(cx.listener(|this, _, w, cx| this.notify(NotificationType::Success, w, cx))))
                            .child(button("n-warning", "Warning").on_click(cx.listener(|this, _, w, cx| this.notify(NotificationType::Warning, w, cx))))
                            .child(button("n-error", "Error").on_click(cx.listener(|this, _, w, cx| this.notify(NotificationType::Error, w, cx))))
                            .child(button("n-sticky", "Sticky + action").on_click(cx.listener(|this, _, window, cx| {
                                let view = cx.entity();
                                window.push_notification(
                                    Notification::new()
                                        .title("Upload finished")
                                        .message("Tap the notification or its action.")
                                        .autohide(false)
                                        .on_click(move |_, _, cx| {
                                            view.update(cx, |this, cx| {
                                                this.log.push("sticky notification tapped");
                                                cx.notify();
                                            })
                                        }),
                                    cx,
                                );
                                this.log.push("sticky notification");
                                cx.notify();
                            })))
                            .child(button("n-clear", "Clear all").on_click(cx.listener(|this, _, window, cx| {
                                window.clear_notifications(cx);
                                this.log.push("notifications cleared");
                                cx.notify();
                            }))),
                    )
                    .child(ui::hint("Toasts are placed top-right by default; check they fit a 411 pt screen.", cx)),
            )
            .child(self.log.render(cx))
            .child(
                ui::section("Background content", cx)
                    .child(ui::hint("Rows below make the page scroll, to verify overlays block it.", cx))
                    .children((1..=30).map(|i| div().text_sm().child(format!("Background row {i}")))),
            )
    }
}
