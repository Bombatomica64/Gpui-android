use gpui::{
    Context, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement, Styled,
    Window, div,
};
use gpui_kit::component::{
    Icon, IconName, accordion::Accordion, button::Button, collapsible::Collapsible, h_flex, v_flex,
};

use crate::ui::{self, EventLog, prelude::*};

pub struct DisclosureScreen {
    open: Vec<usize>,
    multiple: bool,
    bordered: bool,
    collapsible_open: [bool; 3],
    log: EventLog,
}

impl DisclosureScreen {
    pub fn new(_: &mut Window, _: &mut Context<Self>) -> Self {
        Self {
            open: vec![0],
            multiple: false,
            bordered: true,
            collapsible_open: [true, false, false],
            log: EventLog::default(),
        }
    }
}

crate::hot_render!(DisclosureScreen);

impl DisclosureScreen {
    fn render_view(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let open = self.open.clone();
        let accordion = Accordion::new("faq")
            .multiple(self.multiple)
            .bordered(self.bordered)
            .item(|item| item.open(open.contains(&0)).icon(IconName::Info).title("Is it accessible?").child("Yes. It follows the WAI-ARIA disclosure pattern."))
            .item(|item| {
                item.open(open.contains(&1)).title("Does it animate?").child(
                    v_flex().gap_1().children((1..=6).map(|i| div().child(format!("Spring-based reveal, line {i}.")))),
                )
            })
            .item(|item| item.open(open.contains(&2)).title("Disabled item").disabled(true).child("Unreachable."))
            .item(|item| {
                item.open(open.contains(&3))
                    .title("A very long title that must wrap on a narrow phone screen without clipping the chevron")
                    .child("Body text.")
            })
            .on_toggle_click(cx.listener(|this, open: &[usize], _, cx| {
                this.open = open.to_vec();
                this.log.push(format!("accordion open -> {open:?}"));
                cx.notify();
            }));

        let collapsibles = (0..3).map(|ix| {
            let is_open = self.collapsible_open[ix];
            Collapsible::new()
                .open(is_open)
                .motion_id(("collapsible", ix))
                .child(
                    h_flex()
                        .id(("collapsible-trigger", ix))
                        .justify_between()
                        .py_2()
                        .cursor_pointer()
                        .child(format!("Section {}", ix + 1))
                        .child(Icon::new(if is_open {
                            IconName::ChevronDown
                        } else {
                            IconName::ChevronRight
                        }))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.collapsible_open[ix] = !this.collapsible_open[ix];
                            this.log.push(format!(
                                "collapsible {} -> {}",
                                ix + 1,
                                this.collapsible_open[ix]
                            ));
                            cx.notify();
                        })),
                )
                .content(
                    v_flex()
                        .pb_2()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .children((1..=3).map(move |line| {
                            div().child(format!("Content {} line {line}", ix + 1))
                        })),
                )
        });

        v_flex()
            .gap_3()
            .child(
                ui::section("Accordion", cx)
                    .child(
                        h_flex()
                            .gap_1()
                            .child(
                                Button::new("acc-multiple")
                                    .small()
                                    .outline()
                                    .selected(self.multiple)
                                    .label("Multiple")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.multiple = !this.multiple;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("acc-bordered")
                                    .small()
                                    .outline()
                                    .selected(self.bordered)
                                    .label("Bordered")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.bordered = !this.bordered;
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(accordion)
                    .child(ui::value_row("Open items", format!("{:?}", self.open), cx)),
            )
            .child(
                ui::section("Collapsible", cx)
                    .children(collapsibles)
                    .child(ui::value_row(
                        "Open",
                        format!("{:?}", self.collapsible_open),
                        cx,
                    )),
            )
            .child(self.log.render(cx))
    }
}
