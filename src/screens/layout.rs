use gpui::{
    Context, InteractiveElement, IntoElement, ParentElement, Render, ScrollHandle,
    StatefulInteractiveElement, Styled, Window, div, prelude::*, px,
};
use gpui_kit::component::{
    h_flex,
    resizable::{h_resizable, resizable_panel, v_resizable},
    scroll::ScrollableElement as _,
    separator::Separator,
    v_flex,
};

use crate::ui::{self, EventLog, prelude::*};

pub struct LayoutScreen {
    horizontal: ScrollHandle,
    sizes: String,
    log: EventLog,
}

impl LayoutScreen {
    pub fn new(_: &mut Window, _: &mut Context<Self>) -> Self {
        Self {
            horizontal: ScrollHandle::new(),
            sizes: "not resized yet".into(),
            log: EventLog::default(),
        }
    }
}

fn panel(label: &'static str, cx: &Context<LayoutScreen>) -> gpui::Div {
    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .bg(cx.theme().muted)
        .text_sm()
        .child(label)
}

impl Render for LayoutScreen {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let offset = self.horizontal.offset();
        let view = cx.entity();
        v_flex()
            .gap_3()
            .child(
                ui::section("Separator", cx)
                    .child(Separator::horizontal())
                    .child(Separator::horizontal().label("With label"))
                    .child(Separator::horizontal_dashed())
                    .child(
                        h_flex()
                            .h(px(32.))
                            .gap_3()
                            .child("Left")
                            .child(Separator::vertical())
                            .child("Middle")
                            .child(Separator::vertical_dashed())
                            .child("Right"),
                    ),
            )
            .child(
                ui::section("Horizontal scroll + scrollbar", cx)
                    .child(
                        div()
                            .id("h-scroll")
                            .w_full()
                            .overflow_x_scroll()
                            .track_scroll(&self.horizontal)
                            .child(
                                h_flex().gap_2().py_2().children((1..=30).map(|i| {
                                    div()
                                        .flex_none()
                                        .w(px(96.))
                                        .h(px(64.))
                                        .rounded(cx.theme().radius)
                                        .bg(cx.theme().accent)
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .child(format!("Card {i}"))
                                })),
                            )
                            .horizontal_scrollbar(&self.horizontal),
                    )
                    .child(ui::value_row("Scroll x", format!("{:.0}", -offset.x.as_f32()), cx))
                    .child(ui::hint(
                        "Swipe horizontally inside the strip; a vertical swipe that starts on it \
                         must scroll the page instead (one scroll owner per axis).",
                        cx,
                    )),
            )
            .child(
                ui::section("Vertical scrollable (overflow_y_scrollbar)", cx)
                    .child(
                        div()
                            .id("v-scroll")
                            .h(px(180.))
                            .border_1()
                            .border_color(cx.theme().border)
                            .child(
                                v_flex()
                                    .p_2()
                                    .children((1..=40).map(|i| div().text_sm().child(format!("Inner row {i}")))),
                            )
                            .overflow_y_scrollbar(),
                    )
                    .child(ui::hint(
                        "Nested scroll area: scroll it to the end, keep dragging — the page should \
                         only take over at the edge.",
                        cx,
                    )),
            )
            .child(
                ui::section("Resizable panels", cx)
                    .child(
                        div().h(px(260.)).border_1().border_color(cx.theme().border).child(
                            v_resizable("lab-resizable")
                                .on_resize(move |state, _, cx| {
                                    let sizes: Vec<String> =
                                        state.read(cx).sizes().iter().map(|s| format!("{:.0}", s.as_f32())).collect();
                                    view.update(cx, |this, cx| {
                                        this.sizes = sizes.join(", ");
                                        this.log.push(format!("resized -> [{}]", this.sizes));
                                        cx.notify();
                                    })
                                })
                                .child(
                                    h_resizable("lab-resizable-top")
                                        .child(resizable_panel().size(px(120.)).size_range(px(80.)..px(240.)).child(panel("Left", cx)))
                                        .child(resizable_panel().child(panel("Center", cx))),
                                )
                                .child(resizable_panel().size(px(80.)).size_range(px(60.)..px(200.)).child(panel("Bottom", cx))),
                        ),
                    )
                    .child(ui::value_row("Vertical sizes", self.sizes.clone(), cx))
                    .child(ui::hint("Drag the dividers with a finger.", cx)),
            )
            .child(
                ui::section("h_flex / v_flex at phone width", cx).child(
                    h_flex()
                        .w_full()
                        .gap_2()
                        .child(div().flex_1().min_w_0().truncate().child("flex_1 + truncate: this text is far too long to fit and must end with an ellipsis"))
                        .child(div().flex_none().px_2().bg(cx.theme().muted).child("fixed")),
                ),
            )
            .child(self.log.render(cx))
    }
}
