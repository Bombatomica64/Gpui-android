use gpui::{
    AppContext as _, Context, Entity, IntoElement, ParentElement, Render, Styled, Window, div,
    prelude::*, px,
};
use gpui_kit::component::{
    IconName, IndexPath, WindowExt as _,
    button::Button,
    command::{Command, CommandGroup, CommandItem, CommandState},
    v_flex,
};

use crate::ui::{self, EventLog, prelude::*};

const GROUPS: &[(&str, &[(&str, IconName)])] = &[
    (
        "Files",
        &[
            ("New file", IconName::Plus),
            ("Open folder", IconName::FolderOpen),
            ("Search files", IconName::Search),
        ],
    ),
    (
        "Settings",
        &[
            ("Toggle theme", IconName::Moon),
            ("Keyboard shortcuts", IconName::SquareTerminal),
            ("Preferences", IconName::Settings),
        ],
    ),
    (
        "Help",
        &[("Documentation", IconName::BookOpen), ("Report an issue", IconName::Github)],
    ),
];

fn label_at(ix: IndexPath) -> &'static str {
    GROUPS
        .get(ix.section)
        .and_then(|(_, items)| items.get(ix.row))
        .map(|(label, _)| *label)
        .unwrap_or("?")
}

fn build_command(state: &Entity<CommandState>, view: Entity<CommandScreen>) -> Command {
    let mut command = Command::new(state).placeholder("Type a command…");
    for (group, items) in GROUPS {
        let mut g = CommandGroup::new().label(*group);
        for (label, icon) in *items {
            g = g.item(CommandItem::new().label(*label).icon(icon.clone()).keywords([*group]));
        }
        command = command.group(g);
    }
    let select_view = view.clone();
    let confirm_view = view.clone();
    let query_view = view;
    command
        .on_query(move |query, _, cx| {
            let query = query.to_string();
            query_view.update(cx, |this, cx| {
                this.query = query;
                cx.notify();
            })
        })
        .on_select(move |ix, _, cx| {
            select_view.update(cx, |this, cx| {
                this.highlighted = label_at(ix);
                cx.notify();
            })
        })
        .on_confirm(move |ix, window, cx| {
            confirm_view.update(cx, |this, cx| {
                this.log.push(format!("confirm {}", label_at(ix)));
                cx.notify();
            });
            window.close_dialog(cx);
        })
        .empty(|_, _, _| div().p_4().child("No command matches."))
}

pub struct CommandScreen {
    inline: Entity<CommandState>,
    palette: Entity<CommandState>,
    query: String,
    highlighted: &'static str,
    log: EventLog,
}

impl CommandScreen {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            inline: cx.new(|cx| CommandState::new(window, cx)),
            palette: cx.new(|cx| CommandState::new(window, cx)),
            query: String::new(),
            highlighted: "",
            log: EventLog::default(),
        }
    }
}

impl Render for CommandScreen {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity();
        v_flex()
            .gap_3()
            .child(
                ui::section("Inline Command", cx)
                    .child(build_command(&self.inline, view.clone()).max_h(px(320.)))
                    .child(ui::value_row("Query", format!("{:?}", self.query), cx))
                    .child(ui::value_row("Highlighted", self.highlighted, cx))
                    .child(ui::hint("Type to filter; tap an item to confirm it.", cx)),
            )
            .child(
                ui::section("Command palette in a Dialog", cx).child(
                    Button::new("open-palette").primary().label("Open palette").on_click(cx.listener(
                        |this, _, window, cx| {
                            let state = this.palette.clone();
                            let view = cx.entity();
                            window.open_dialog(cx, move |dialog, _, _| {
                                let state = state.clone();
                                let view = view.clone();
                                dialog.close_button(false).p_0().content(move |content, _, _| {
                                    content.child(build_command(&state, view.clone()).bordered(false))
                                })
                            });
                            this.palette.update(cx, |state, cx| state.focus(window, cx));
                            this.log.push("palette opened");
                            cx.notify();
                        },
                    )),
                ),
            )
            .child(self.log.render(cx))
    }
}
