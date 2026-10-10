use gpui::{
    AppContext as _, Context, Entity, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, Window, div, point, px,
};
use gpui_kit::component::{
    IconName,
    button::Button,
    h_flex,
    hover_card::HoverCard,
    input::{Input, InputState},
    menu::{ContextMenuExt as _, DropdownMenu as _, PopupMenuItem},
    native_menu::NativeMenu,
    popover::Popover,
    tooltip::Tooltip,
    v_flex,
};

use crate::ui::{self, EventLog, prelude::*};

gpui::actions!(lab_menu, [NativeCut, NativeCopy, NativePaste]);

pub struct PopoversScreen {
    input: Entity<InputState>,
    word_wrap: bool,
    show_hidden: bool,
    controlled_open: bool,
    focus: gpui::FocusHandle,
    log: EventLog,
}

impl PopoversScreen {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            input: cx.new(|cx| InputState::new(window, cx).placeholder("Input inside a popover")),
            word_wrap: true,
            show_hidden: false,
            controlled_open: false,
            focus: cx.focus_handle(),
            log: EventLog::default(),
        }
    }
}

crate::hot_render!(PopoversScreen);

impl PopoversScreen {
    fn render_view(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity();
        let word_wrap = self.word_wrap;
        let show_hidden = self.show_hidden;

        let dropdown = Button::new("dropdown")
            .outline()
            .label("Dropdown menu")
            .dropdown_menu({
                let view = view.clone();
                move |menu, window, cx| {
                    let item = |label: &'static str| {
                        let view = view.clone();
                        PopupMenuItem::new(label).on_click(move |_, _, cx| {
                            view.update(cx, |this, cx| {
                                this.log.push(format!("menu: {label}"));
                                cx.notify();
                            })
                        })
                    };
                    let wrap_view = view.clone();
                    let hidden_view = view.clone();
                    menu.label("File")
                        .item(item("New").icon(IconName::Plus))
                        .item(item("Open…").icon(IconName::FolderOpen))
                        .item(item("Disabled item").disabled(true))
                        .separator()
                        .item(PopupMenuItem::new("Word wrap").checked(word_wrap).on_click(
                            move |_, _, cx| {
                                wrap_view.update(cx, |this, cx| {
                                    this.word_wrap = !this.word_wrap;
                                    this.log.push(format!("word wrap -> {}", this.word_wrap));
                                    cx.notify();
                                })
                            },
                        ))
                        .item(
                            PopupMenuItem::new("Show hidden")
                                .checked(show_hidden)
                                .on_click(move |_, _, cx| {
                                    hidden_view.update(cx, |this, cx| {
                                        this.show_hidden = !this.show_hidden;
                                        this.log
                                            .push(format!("show hidden -> {}", this.show_hidden));
                                        cx.notify();
                                    })
                                }),
                        )
                        .separator()
                        .submenu("Open recent", window, cx, {
                            let view = view.clone();
                            move |menu, _, _| {
                                let mut menu = menu;
                                for name in ["project-alpha", "project-beta", "project-gamma"] {
                                    let view = view.clone();
                                    menu = menu.item(PopupMenuItem::new(name).on_click(
                                        move |_, _, cx| {
                                            view.update(cx, |this, cx| {
                                                this.log.push(format!("recent: {name}"));
                                                cx.notify();
                                            })
                                        },
                                    ));
                                }
                                menu
                            }
                        })
                        .link("GPUI Kit website", "https://gpui-kit.com")
                }
            });

        let long_menu = Button::new("long-menu")
            .outline()
            .label("Long menu (40 items)")
            .dropdown_menu({
                let view = view.clone();
                move |menu, _, _| {
                    let mut menu = menu.scrollable(true).max_h(px(300.));
                    for i in 1..=40 {
                        let view = view.clone();
                        menu = menu.item(PopupMenuItem::new(format!("Item {i}")).on_click(
                            move |_, _, cx| {
                                view.update(cx, |this, cx| {
                                    this.log.push(format!("long menu item {i}"));
                                    cx.notify();
                                })
                            },
                        ));
                    }
                    menu
                }
            });

        v_flex()
            .gap_3()
            .child(
                ui::section("Popover", cx)
                    .child(
                        h_flex()
                            .flex_wrap()
                            .gap_2()
                            .child(
                                Popover::new("basic-popover")
                                    .trigger(Button::new("popover-trigger").outline().label("Basic"))
                                    .on_open_change({
                                        let view = view.clone();
                                        move |open, _, cx| {
                                            view.update(cx, |this, cx| {
                                                this.log.push(format!("popover open -> {open}"));
                                                cx.notify();
                                            })
                                        }
                                    })
                                    .child("Hello from a popover.")
                                    .child("Tap outside to dismiss."),
                            )
                            .child(
                                Popover::new("input-popover")
                                    .trigger(Button::new("popover-input").outline().label("With input"))
                                    .content({
                                        let input = self.input.clone();
                                        move |_, _, _| {
                                            v_flex()
                                                .w(px(260.))
                                                .gap_2()
                                                .child("Keyboard should appear:")
                                                .child(Input::new(&input))
                                        }
                                    }),
                            )
                            .child(
                                Popover::new("controlled-popover")
                                    .open(self.controlled_open)
                                    .on_open_change(cx.listener(|this, open: &bool, _, cx| {
                                        this.controlled_open = *open;
                                        this.log.push(format!("controlled popover -> {open}"));
                                        cx.notify();
                                    }))
                                    .trigger(Button::new("popover-controlled").outline().label("Controlled"))
                                    .child("Controlled: open state lives in the screen."),
                            ),
                    )
                    .child(
                        h_flex().justify_end().child(
                            Popover::new("edge-popover")
                                .trigger(Button::new("popover-edge").outline().label("Right edge"))
                                .child("This popover opens near the right screen edge and must not be clipped."),
                        ),
                    )
                    .child(ui::value_row("Controlled open", self.controlled_open.to_string(), cx)),
            )
            .child(
                ui::section("HoverCard (tap to open on mobile)", cx)
                    .child(
                        HoverCard::new("hover-card")
                            .trigger(div().text_color(cx.theme().primary).child("@gpui-kit"))
                            .on_open_change({
                                let view = view.clone();
                                move |open, _, cx| {
                                    view.update(cx, |this, cx| {
                                        this.log.push(format!("hover card open -> {open}"));
                                        cx.notify();
                                    })
                                }
                            })
                            .child(
                                v_flex()
                                    .w(px(260.))
                                    .gap_1()
                                    .child(div().font_semibold().child("GPUI Kit"))
                                    .child("UI components for GPUI. Tap outside to close."),
                            ),
                    )
                    .child(ui::hint("Kit switches HoverCard to tap-to-open on Android.", cx)),
            )
            .child(
                ui::section("Tooltip", cx)
                    .child(
                        h_flex()
                            .gap_2()
                            .child(Button::new("tooltip-kit").outline().label("Kit tooltip").tooltip("Kit-managed tooltip"))
                            .child(
                                div()
                                    .id("raw-tooltip")
                                    .px_2()
                                    .py_1()
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded(cx.theme().radius)
                                    .child("Raw GPUI .tooltip()")
                                    .tooltip(|window, cx| Tooltip::new("Raw GPUI tooltip").build(window, cx)),
                            ),
                    )
                    .child(ui::hint(
                        "Expected on Android: the Kit tooltip never shows (disabled by Base); the \
                         raw GPUI tooltip may show on long press.",
                        cx,
                    )),
            )
            .child(
                ui::section("PopupMenu", cx)
                    .child(h_flex().flex_wrap().gap_2().child(dropdown).child(long_menu))
                    .child(ui::value_row(
                        "Checked items",
                        format!("word wrap={word_wrap} show hidden={show_hidden}"),
                        cx,
                    )),
            )
            .child(
                ui::section("Context menu (ContextMenuExt)", cx)
                    .child(
                        div()
                            .id("context-target")
                            .h(px(80.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded(cx.theme().radius)
                            .border_1()
                            .border_dashed()
                            .border_color(cx.theme().border)
                            .child("Long-press / right-click here")
                            .context_menu({
                                let view = view.clone();
                                move |menu, _, _| {
                                    let item = |label: &'static str| {
                                        let view = view.clone();
                                        PopupMenuItem::new(label).on_click(move |_, _, cx| {
                                            view.update(cx, |this, cx| {
                                                this.log.push(format!("context: {label}"));
                                                cx.notify();
                                            })
                                        })
                                    };
                                    menu.item(item("Copy")).item(item("Paste")).separator().item(item("Delete"))
                                }
                            }),
                    )
                    .child(ui::hint(
                        "Right-click or long-press opens the menu.",
                        cx,
                    )),
            )
            .child(
                ui::section("NativeMenu (GPUI-drawn fallback)", cx).child(
                    Button::new("native-menu")
                        .outline()
                        .label("Show NativeMenu")
                        .on_click(cx.listener(|this, event: &gpui::ClickEvent, window, cx| {
                            // Menu actions dispatch along the focus path; focus this
                            // screen so its on_action handlers receive them.
                            this.focus.focus(window, cx);
                            let position = event.position();
                            NativeMenu::new()
                                .menu("Cut", Box::new(NativeCut))
                                .menu("Copy", Box::new(NativeCopy))
                                .separator()
                                .menu("Paste", Box::new(NativePaste))
                                .show(if position == point(px(0.), px(0.)) { point(px(40.), px(200.)) } else { position }, window, cx);
                            this.log.push("NativeMenu shown");
                            cx.notify();
                        })),
                ),
            )
            .child(self.log.render(cx))
            .track_focus(&self.focus)
            .on_action(cx.listener(|this, _: &NativeCut, _, cx| {
                this.log.push("native menu: Cut");
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &NativeCopy, _, cx| {
                this.log.push("native menu: Copy");
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &NativePaste, _, cx| {
                this.log.push("native menu: Paste");
                cx.notify();
            }))
    }
}
