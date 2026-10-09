use std::time::Duration;

use gpui::{
    Anchor, Context, IntoElement, Keystroke, ParentElement, Render, SharedString, Styled, Window,
    actions,
};
use gpui_kit::component::{
    IconName, Size,
    button::{Button, ButtonGroup, DropdownButton, Toggle, ToggleGroup},
    h_flex,
    kbd::Kbd,
    link::Link,
    menu::PopupMenuItem,
    v_flex,
};

use crate::ui::{self, EventLog, prelude::*};

actions!(lab_buttons, [MenuNew, MenuOpen, MenuDelete]);

pub struct ButtonsScreen {
    clicks: usize,
    disabled: bool,
    loading: bool,
    group_selected: Vec<usize>,
    bold: bool,
    italic: bool,
    underline: bool,
    toggle: bool,
    log: EventLog,
}

impl ButtonsScreen {
    pub fn new(_: &mut Window, _: &mut Context<Self>) -> Self {
        Self {
            clicks: 0,
            disabled: false,
            loading: false,
            group_selected: vec![0],
            bold: true,
            italic: false,
            underline: false,
            toggle: false,
            log: EventLog::default(),
        }
    }

    fn start_loading(&mut self, cx: &mut Context<Self>) {
        self.loading = true;
        self.log.push("loading started (2s)");
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(Duration::from_secs(2)).await;
            let _ = this.update(cx, |this, cx| {
                this.loading = false;
                this.log.push("loading finished");
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }
}

#[gpui_hot::hot]
impl Render for ButtonsScreen {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let variants = [
            ("Primary", 0),
            ("Secondary", 1),
            ("Danger", 2),
            ("Warning", 3),
            ("Success", 4),
            ("Info", 5),
            ("Ghost", 6),
            ("Link", 7),
            ("Outline", 8),
        ]
        .into_iter()
        .map(|(label, kind)| {
            let button = Button::new(SharedString::from(format!("variant-{label}"))).label(label);
            let button = match kind {
                0 => button.primary(),
                1 => button.secondary(),
                2 => button.danger(),
                3 => button.warning(),
                4 => button.success(),
                5 => button.info(),
                6 => button.ghost(),
                7 => button.link(),
                _ => button.outline(),
            };
            button
                .disabled(self.disabled)
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.clicks += 1;
                    this.log.push(format!("{label} clicked"));
                    cx.notify();
                }))
        });

        let sizes = [
            ("XSmall", Size::XSmall),
            ("Small", Size::Small),
            ("Medium", Size::Medium),
            ("Large", Size::Large),
        ]
        .into_iter()
        .map(|(label, size)| {
            Button::new(SharedString::from(format!("size-{label}")))
                .primary()
                .with_size(size)
                .label(label)
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.log.push(format!("{label} size clicked"));
                    cx.notify();
                }))
        });

        let group_selected = self.group_selected.clone();
        let formatting_states = [self.bold, self.italic, self.underline];

        v_flex()
            .gap_3()
            .child(
                ui::section("Variants", cx)
                    .child(h_flex().flex_wrap().gap_2().children(variants))
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("toggle-disabled")
                                    .small()
                                    .outline()
                                    .label(if self.disabled { "Enable all" } else { "Disable all" })
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.disabled = !this.disabled;
                                        this.log.push(format!("disabled = {}", this.disabled));
                                        cx.notify();
                                    })),
                            )
                            .child(ui::value_row("Clicks", self.clicks.to_string(), cx)),
                    )
                    .child(ui::hint(
                        "Tap each variant; with 'Disable all' on, taps must not register.",
                        cx,
                    )),
            )
            .child(ui::section("Sizes & icons", cx).child(h_flex().flex_wrap().gap_2().children(sizes)).child(
                h_flex()
                    .flex_wrap()
                    .gap_2()
                    .child(Button::new("icon-only").icon(IconName::Heart).outline().on_click(
                        cx.listener(|this, _, _, cx| {
                            this.log.push("icon button clicked");
                            cx.notify();
                        }),
                    ))
                    .child(Button::new("icon-label").icon(IconName::Plus).label("New item"))
                    .child(Button::new("compact").compact().outline().label("Compact"))
                    .child(Button::new("selected").outline().selected(true).label("Selected")),
            ))
            .child(
                ui::section("Loading", cx)
                    .child(
                        Button::new("loading")
                            .primary()
                            .loading(self.loading)
                            .label(if self.loading { "Saving…" } else { "Save (2s)" })
                            .on_click(cx.listener(|this, _, _, cx| this.start_loading(cx))),
                    )
                    .child(ui::value_row("Loading", self.loading.to_string(), cx))
                    .child(ui::hint("While loading, extra taps must be ignored.", cx)),
            )
            .child(
                ui::section("ButtonGroup (multiple)", cx)
                    .child(
                        ButtonGroup::new("group")
                            .outline()
                            .multiple(true)
                            .child(Button::new("g0").label("Day").selected(group_selected.contains(&0)))
                            .child(Button::new("g1").label("Week").selected(group_selected.contains(&1)))
                            .child(Button::new("g2").label("Month").selected(group_selected.contains(&2)))
                            .on_click(cx.listener(|this, selected: &Vec<usize>, _, cx| {
                                this.group_selected = selected.clone();
                                this.log.push(format!("group -> {selected:?}"));
                                cx.notify();
                            })),
                    )
                    .child(ui::value_row("Selected", format!("{:?}", self.group_selected), cx)),
            )
            .child(
                ui::section("DropdownButton", cx).child(
                    DropdownButton::new("dropdown")
                        .button(Button::new("dropdown-main").label("Export").on_click(cx.listener(
                            |this, _, _, cx| {
                                this.log.push("Export (main action)");
                                cx.notify();
                            },
                        )))
                        .dropdown_menu_with_anchor(Anchor::TopRight, {
                            let view = cx.entity();
                            move |menu, _, _| {
                                let item = |label: &'static str| {
                                    let view = view.clone();
                                    PopupMenuItem::new(label).on_click(move |_, _, cx| {
                                        view.update(cx, |this, cx| {
                                            this.log.push(format!("menu: {label}"));
                                            cx.notify();
                                        })
                                    })
                                };
                                menu.item(item("Export CSV"))
                                    .item(item("Export PDF"))
                                    .separator()
                                    .item(item("Print…"))
                            }
                        }),
                ),
            )
            .child(
                ui::section("Toggle / ToggleGroup", cx)
                    .child(
                        ToggleGroup::new("format")
                            .segmented()
                            .outline()
                            .child(Toggle::new(0).label("Bold").checked(formatting_states[0]))
                            .child(Toggle::new(1).label("Italic").checked(formatting_states[1]))
                            .child(Toggle::new(2).label("Underline").checked(formatting_states[2]))
                            .on_click(cx.listener(|this, states: &Vec<bool>, _, cx| {
                                this.bold = states[0];
                                this.italic = states[1];
                                this.underline = states[2];
                                this.log.push(format!("format -> {states:?}"));
                                cx.notify();
                            })),
                    )
                    .child(
                        Toggle::new("single")
                            .icon(IconName::Star)
                            .label("Favorite")
                            .checked(self.toggle)
                            .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                this.toggle = *checked;
                                this.log.push(format!("favorite -> {checked}"));
                                cx.notify();
                            })),
                    )
                    .child(ui::value_row(
                        "State",
                        format!(
                            "bold={} italic={} underline={} favorite={}",
                            self.bold, self.italic, self.underline, self.toggle
                        ),
                        cx,
                    )),
            )
            .child(
                ui::section("Link", cx)
                    .child(
                        Link::new("link-local")
                            .child("Link with on_click (no navigation)")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.log.push("link clicked");
                                cx.notify();
                            })),
                    )
                    .child(Link::new("link-web").href("https://gpui-kit.com").child("https://gpui-kit.com (opens browser)"))
                    .child(ui::hint(
                        "The href link calls cx.open_url; gpui-mobile implements it with an ACTION_VIEW intent.",
                        cx,
                    )),
            )
            .child(
                ui::section("Kbd", cx)
                    .child(
                        h_flex()
                            .flex_wrap()
                            .gap_2()
                            .children(["ctrl-c", "ctrl-shift-p", "escape", "alt-enter"].into_iter().filter_map(
                                |stroke| Keystroke::parse(stroke).ok().map(Kbd::new),
                            )),
                    )
                    .child(ui::hint("Display-only; rendered with the Linux/Windows style labels.", cx)),
            )
            .child(self.log.render(cx))
    }
}
