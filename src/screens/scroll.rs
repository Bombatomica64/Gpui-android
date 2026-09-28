use gpui::{
    Context, InteractiveElement, IntoElement, ParentElement, Render, ScrollHandle, SharedString,
    StatefulInteractiveElement, Styled, UniformListScrollHandle, Window, div, px, uniform_list,
};
use gpui_kit::component::{
    button::Button,
    h_flex,
    scroll::ScrollableElement as _,
    tab::{Tab, TabBar},
    v_flex,
};

use crate::ui::{self, prelude::*};

const VIRTUAL_ROWS: usize = 10_000;

pub struct ScrollScreen {
    tab: usize,
    plain: ScrollHandle,
    virtual_list: UniformListScrollHandle,
    mixed: ScrollHandle,
    text: ScrollHandle,
}

impl ScrollScreen {
    pub fn new(_: &mut Window, _: &mut Context<Self>) -> Self {
        Self {
            tab: 0,
            plain: ScrollHandle::new(),
            virtual_list: UniformListScrollHandle::new(),
            mixed: ScrollHandle::new(),
            text: ScrollHandle::new(),
        }
    }
}

fn long_text() -> SharedString {
    let sentence = "The quick brown fox jumps over the lazy dog; 敏捷的棕色狐狸跳过了懒狗; 素早い茶色の狐がのろまな犬を飛び越える. ";
    sentence.repeat(400).into()
}

impl Render for ScrollScreen {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let offset = match self.tab {
            0 => self.plain.offset(),
            1 => self.virtual_list.0.borrow().base_handle.offset(),
            2 => self.mixed.offset(),
            _ => self.text.offset(),
        };
        let body = match self.tab {
            0 => div()
                .id("plain-100")
                .size_full()
                .overflow_y_scroll()
                .track_scroll(&self.plain)
                .child(v_flex().children((1..=100).map(|i| {
                    h_flex()
                        .h(px(52.))
                        .px_3()
                        .gap_2()
                        .border_b_1()
                        .border_color(cx.theme().border)
                        .child(div().size_8().rounded_full().bg(cx.theme().accent))
                        .child(format!("Plain row {i} (not virtualized)"))
                })))
                .vertical_scrollbar(&self.plain)
                .into_any_element(),
            1 => uniform_list(
                "virtual-10k",
                VIRTUAL_ROWS,
                cx.processor(|_, range: std::ops::Range<usize>, _, cx| {
                    range
                        .map(|i| {
                            h_flex()
                                .id(i)
                                .h(px(44.))
                                .px_3()
                                .border_b_1()
                                .border_color(cx.theme().border)
                                .active(|this| this.bg(cx.theme().accent))
                                .child(format!("Virtual row {i} of {VIRTUAL_ROWS}"))
                                .on_click(move |_, _, _| {
                                    log::info!("event: virtual row {i} tapped")
                                })
                        })
                        .collect()
                }),
            )
            .track_scroll(&self.virtual_list)
            .size_full()
            .into_any_element(),
            2 => div()
                .id("mixed")
                .size_full()
                .overflow_y_scroll()
                .restrict_scroll_to_axis()
                .track_scroll(&self.mixed)
                .child(
                    v_flex()
                        .gap_3()
                        .p_3()
                        .children((1..=12usize).map(|section| {
                            v_flex()
                                .gap_1()
                                .child(div().font_semibold().child(format!("Section {section}")))
                                .child(
                                    div()
                                        .id(("strip", section))
                                        .flex()
                                        .overflow_x_scroll()
                                        .restrict_scroll_to_axis()
                                        .child(h_flex().flex_none().gap_2().children(
                                            (1..=15).map(|card| {
                                                div()
                                                    .flex_none()
                                                    .w(px(110.))
                                                    .h(px(80.))
                                                    .rounded(cx.theme().radius)
                                                    .bg(cx.theme().muted)
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .child(format!("{section}.{card}"))
                                            }),
                                        )),
                                )
                        })),
                )
                .into_any_element(),
            _ => div()
                .id("long-text")
                .size_full()
                .overflow_y_scroll()
                .track_scroll(&self.text)
                .p_3()
                .child(div().text_sm().child(long_text()))
                .into_any_element(),
        };
        let hints = [
            "100 plain rows in one scroll container. Fling, then touch to stop the fling.",
            "10,000 rows via uniform_list: only visible rows are built. Fling far.",
            "Vertical page with horizontal strips: each gesture should pick one axis.",
            "One huge wrapped text block (~40 KB) in a single scroll container.",
        ];
        v_flex()
            .size_full()
            .child(
                TabBar::new("scroll-kind")
                    .underline()
                    .selected_index(self.tab)
                    .child(Tab::new().label("100 rows"))
                    .child(Tab::new().label("10k virtual"))
                    .child(Tab::new().label("Nested axes"))
                    .child(Tab::new().label("Long text"))
                    .on_click(cx.listener(|this, ix: &usize, _, cx| {
                        this.tab = *ix;
                        cx.notify();
                    })),
            )
            .child(
                h_flex()
                    .px_3()
                    .py_1()
                    .gap_2()
                    .justify_between()
                    .child(ui::value_row(
                        "Offset y",
                        format!("{:.0}", -offset.y.as_f32()),
                        cx,
                    ))
                    .child(Button::new("top").xsmall().outline().label("Top").on_click(
                        cx.listener(|this, _, _, cx| {
                            this.plain.set_offset(Default::default());
                            this.mixed.set_offset(Default::default());
                            this.text.set_offset(Default::default());
                            this.virtual_list
                                .scroll_to_item(0, gpui::ScrollStrategy::Top);
                            cx.notify();
                        }),
                    )),
            )
            .child(div().px_3().child(ui::hint(hints[self.tab.min(3)], cx)))
            .child(div().flex_1().min_h_0().child(body))
    }
}
