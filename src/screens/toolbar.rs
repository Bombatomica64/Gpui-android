use gpui::{Context, IntoElement, ParentElement, Render, Styled, Window, div, prelude::*, px};
use gpui_kit::component::{
    IconName, Size,
    button::{Button, Toggle},
    h_flex,
    separator::Separator,
    toolbar::{Toolbar, ToolbarGroup},
    v_flex,
};

use crate::ui::{self, EventLog, prelude::*};

pub struct ToolbarScreen {
    size: Size,
    disabled: bool,
    bold: bool,
    italic: bool,
    undo_depth: usize,
    log: EventLog,
}

impl ToolbarScreen {
    pub fn new(_: &mut Window, _: &mut Context<Self>) -> Self {
        Self {
            size: Size::Small,
            disabled: false,
            bold: false,
            italic: false,
            undo_depth: 3,
            log: EventLog::default(),
        }
    }

    fn action(&mut self, name: &str, cx: &mut Context<Self>) {
        self.log.push(format!("toolbar: {name}"));
        cx.notify();
    }
}

impl Render for ToolbarScreen {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let disabled = self.disabled;
        let toolbar = Toolbar::new("doc-toolbar")
            .with_size(self.size)
            .disabled(disabled)
            .child(
                Button::new("new")
                    .icon(IconName::Plus)
                    .label("New")
                    .disabled(disabled)
                    .on_click(cx.listener(|this, _, _, cx| this.action("New", cx))),
            )
            .content(Separator::vertical().h_5())
            .child(
                ToolbarGroup::new("history")
                    .label("History")
                    .child(
                        Button::new("undo")
                            .icon(IconName::Undo2)
                            .tooltip("Undo")
                            .disabled(disabled || self.undo_depth == 0)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.undo_depth = this.undo_depth.saturating_sub(1);
                                this.action("Undo", cx)
                            })),
                    )
                    .child(
                        Button::new("redo")
                            .icon(IconName::Redo2)
                            .tooltip("Redo")
                            .disabled(disabled)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.undo_depth += 1;
                                this.action("Redo", cx)
                            })),
                    ),
            )
            .content(Separator::vertical().h_5())
            .child(
                ToolbarGroup::new("format")
                    .label("Formatting")
                    .child(
                        Toggle::new("bold")
                            .icon(gpui_kit::assets::IconName::Bold)
                            .checked(self.bold)
                            .disabled(disabled)
                            .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                this.bold = *checked;
                                this.action(&format!("Bold -> {checked}"), cx)
                            })),
                    )
                    .child(
                        Toggle::new("italic")
                            .icon(gpui_kit::assets::IconName::Italic)
                            .checked(self.italic)
                            .disabled(disabled)
                            .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                this.italic = *checked;
                                this.action(&format!("Italic -> {checked}"), cx)
                            })),
                    ),
            )
            .content(div().flex_1())
            .child(
                Button::new("more")
                    .icon(IconName::Ellipsis)
                    .tooltip("More options")
                    .disabled(disabled)
                    .on_click(cx.listener(|this, _, _, cx| this.action("More", cx))),
            );

        let sizes = [("XSmall", Size::XSmall), ("Small", Size::Small), ("Medium", Size::Medium)]
            .into_iter()
            .map(|(label, size)| {
                Button::new(label)
                    .small()
                    .label(label)
                    .selected(self.size == size)
                    .outline()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.size = size;
                        this.log.push(format!("size -> {label}"));
                        cx.notify();
                    }))
            });

        v_flex()
            .gap_3()
            .child(
                ui::section("Toolbar with groups (new in 0.7.0)", cx)
                    .child(
                        div()
                            .w_full()
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded(px(6.))
                            .p_1()
                            .child(toolbar),
                    )
                    .child(h_flex().gap_1().children(sizes))
                    .child(
                        Button::new("toggle-toolbar-disabled")
                            .small()
                            .outline()
                            .label(if disabled { "Enable toolbar" } else { "Disable toolbar" })
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.disabled = !this.disabled;
                                this.log.push(format!("toolbar disabled = {}", this.disabled));
                                cx.notify();
                            })),
                    )
                    .child(ui::value_row(
                        "State",
                        format!("bold={} italic={} undo depth={}", self.bold, self.italic, self.undo_depth),
                        cx,
                    ))
                    .child(ui::hint(
                        "Icon-only buttons carry tooltips as accessible names; Kit suppresses the \
                         tooltip overlay on Android, so nothing should pop up on tap.",
                        cx,
                    )),
            )
            .child(
                ui::section("Narrow toolbar (overflow)", cx)
                    .child(
                        div().w(px(220.)).overflow_hidden().child(
                            Toolbar::new("narrow")
                                .children((0..8usize).map(|i| {
                                    Button::new(("narrow", i)).label(format!("B{i}")).on_click(cx.listener(
                                        move |this, _, _, cx| this.action(&format!("narrow B{i}"), cx),
                                    ))
                                })),
                        ),
                    )
                    .child(ui::hint("8 buttons in a 220 px box: check clipping at the edge.", cx)),
            )
            .child(self.log.render(cx))
    }
}
