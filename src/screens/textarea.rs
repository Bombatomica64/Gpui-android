use gpui::{
    AppContext as _, Context, Entity, IntoElement, ParentElement, Render, Styled, Subscription,
    Window, px,
};
use gpui_kit::component::{
    IconName,
    button::Button,
    h_flex,
    input::{
        Editor, EditorState, InlineToken, InputContent, InputEvent, InputToken, Textarea,
        TextareaState,
    },
    v_flex,
};

use crate::screens::text_input::UNICODE_SAMPLE;
use crate::ui::{self, EventLog, prelude::*};

const CODE: &str = r#"// Rust sample: long lines test horizontal scrolling in the editor.
use std::collections::HashMap;

fn main() {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for word in "the quick brown fox jumps over the lazy dog the end".split_whitespace() {
        *counts.entry(word).or_default() += 1;
    }
    let mut pairs: Vec<_> = counts.into_iter().collect();
    pairs.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    for (word, count) in pairs {
        println!("{word:>8} {count}");
    }
}
"#;

pub struct TextareaScreen {
    plain: Entity<TextareaState>,
    grow: Entity<TextareaState>,
    tokens: Entity<TextareaState>,
    editor: Entity<EditorState>,
    lines: usize,
    log: EventLog,
    _subscriptions: Vec<Subscription>,
}

fn token_content() -> InputContent {
    let text = "Ask @Alice to review #1234 before Friday.";
    let alice = text.find("@Alice").unwrap_or(0);
    let issue = text.find("#1234").unwrap_or(0);
    let content = InputContent::new(text);
    content
        .clone()
        .with_token(alice..alice + "@Alice".len(), InlineToken::new("user-alice", "@Alice"))
        .and_then(|c| c.with_token(issue..issue + "#1234".len(), InlineToken::new("issue-1234", "#1234")))
        .unwrap_or_else(|err| {
            log::error!("building inline tokens failed: {err:?}");
            content
        })
}

impl TextareaScreen {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let plain = cx.new(|cx| {
            TextareaState::new(window, cx)
                .rows(5)
                .default_value(format!("{UNICODE_SAMPLE}\n\nSecond paragraph.\nThird line."))
        });
        let grow = cx.new(|cx| {
            TextareaState::new(window, cx)
                .auto_grow(1, 6)
                .placeholder("Auto-grows from 1 to 6 rows…")
        });
        let tokens = cx.new(|cx| {
            let mut state = TextareaState::new(window, cx).rows(3);
            state.set_value(token_content(), window, cx);
            state
        });
        let editor = cx.new(|cx| {
            EditorState::new(window, cx)
                .line_number(true)
                .default_value(CODE)
        });
        let mut subscriptions = Vec::new();
        for (name, state) in [("plain", &plain), ("auto-grow", &grow), ("tokens", &tokens)] {
            subscriptions.push(cx.subscribe(state, move |this, state, event: &InputEvent, cx| {
                this.log_event(name, event, state.read(cx).value().len(), cx)
            }));
        }
        subscriptions.push(cx.subscribe(&editor, |this, state, event: &InputEvent, cx| {
            this.lines = state.read(cx).value().lines().count();
            this.log_event("editor", event, state.read(cx).value().len(), cx)
        }));
        Self {
            plain,
            grow,
            tokens,
            editor,
            lines: CODE.lines().count(),
            log: EventLog::default(),
            _subscriptions: subscriptions,
        }
    }

    fn log_event(&mut self, name: &str, event: &InputEvent, len: usize, cx: &mut Context<Self>) {
        let message = match event {
            InputEvent::Change => format!("{name}: change ({len} bytes)"),
            InputEvent::PressEnter { secondary, shift } => {
                format!("{name}: enter secondary={secondary} shift={shift}")
            }
            InputEvent::Focus => format!("{name}: focus"),
            InputEvent::Blur => format!("{name}: blur"),
        };
        self.log.push(message);
        cx.notify();
    }
}

impl Render for TextareaScreen {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let plain_value = self.plain.read(cx).value();
        v_flex()
            .gap_3()
            .child(
                ui::section("Textarea", cx)
                    .child(Textarea::new(&self.plain))
                    .child(ui::value_row(
                        "Lines / chars",
                        format!("{} / {}", plain_value.lines().count(), plain_value.chars().count()),
                        cx,
                    ))
                    .child(ui::hint(
                        "Enter inserts a newline; test selection across lines, long-press, \
                         and scrolling inside the field once text exceeds 5 rows.",
                        cx,
                    )),
            )
            .child(
                ui::section("Auto-grow (1–6 rows)", cx)
                    .child(Textarea::new(&self.grow))
                    .child(ui::hint("Type several lines; the field should grow, then scroll.", cx)),
            )
            .child(
                ui::section("Inline tokens", cx)
                    .child(
                        Textarea::new(&self.tokens)
                            .token(|context, _, _| InputToken::new(context).icon(IconName::User))
                            .on_token_click(cx.listener(|this, _, _, cx| {
                                this.log.push("token clicked");
                                cx.notify();
                            })),
                    )
                    .child(
                        h_flex().gap_1().child(
                            Button::new("reset-tokens").small().outline().label("Reset tokens").on_click(
                                cx.listener(|this, _, window, cx| {
                                    this.tokens.update(cx, |state, cx| state.set_value(token_content(), window, cx));
                                    this.log.push("tokens reset");
                                    cx.notify();
                                }),
                            ),
                        ),
                    )
                    .child(ui::hint(
                        "@Alice and #1234 are atomic: backspace should delete a whole token.",
                        cx,
                    )),
            )
            .child(
                ui::section("Editor (no tree-sitter)", cx)
                    .child(Editor::new(&self.editor).h(px(260.)))
                    .child(ui::value_row("Lines", self.lines.to_string(), cx))
                    .child(ui::hint(
                        "Scroll inside the editor both ways; the page should not scroll while \
                         the editor has more content in the gesture direction.",
                        cx,
                    )),
            )
            .child(self.log.render(cx))
    }
}
