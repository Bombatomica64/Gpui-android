use gpui::{
    AppContext as _, ClipboardItem, Context, Entity, IntoElement, ParentElement, Render,
    SharedString, Styled, Subscription, Window, div, prelude::*,
};
use gpui_kit::component::{
    IconName, WindowExt as _,
    button::Button,
    clipboard::Clipboard,
    h_flex,
    input::{
        Input, InputEvent, InputGroup, InputGroupAddon, InputGroupButton, InputGroupText,
        InputState, MaskPattern,
    },
    label::{HighlightsMatch, Label},
    v_flex,
};

use crate::ui::{self, EventLog, prelude::*};

pub const UNICODE_SAMPLE: &str = "English Italiano àèéìòù 日本語 中文测试 한국어 😀🚀🦀";

struct Field {
    name: &'static str,
    state: Entity<InputState>,
}

pub struct TextInputScreen {
    fields: Vec<Field>,
    active: usize,
    log: EventLog,
    _subscriptions: Vec<Subscription>,
}

impl TextInputScreen {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let make = |name: &'static str,
                    window: &mut Window,
                    cx: &mut Context<Self>,
                    build: &dyn Fn(InputState) -> InputState| Field {
            name,
            state: cx.new(|cx| build(InputState::new(window, cx))),
        };
        let fields = vec![
            make("unicode", window, cx, &|s| s.default_value(UNICODE_SAMPLE)),
            make("empty", window, cx, &|s| s.placeholder("Type here…").clean_on_escape()),
            make("password", window, cx, &|s| s.masked(true).placeholder("Password")),
            make("phone mask", window, cx, &|s| {
                s.mask_pattern("(999) 999-9999").placeholder("(555) 123-4567")
            }),
            make("amount mask", window, cx, &|s| {
                s.mask_pattern(MaskPattern::Number {
                    separator: Some(','),
                    fraction: Some(2),
                })
                .placeholder("1,234.56")
            }),
            make("digits only", window, cx, &|s| {
                s.pattern(regex::Regex::new(r"^\d*$").expect("valid regex"))
                    .placeholder("0-9 only")
            }),
            make("disabled", window, cx, &|s| s.default_value("Disabled input")),
            make("url", window, cx, &|s| s.default_value("https://gpui-kit.com/docs/mobile/")),
            make("group", window, cx, &|s| s.placeholder("username")),
            make("highlight", window, cx, &|s| s.default_value("Hello")),
            make("long", window, cx, &|s| {
                s.default_value(
                    "A deliberately long single-line value that is far wider than a phone \
                     screen, to test horizontal caret scrolling and selection handles near the edge.",
                )
            }),
        ];
        let subscriptions = fields
            .iter()
            .enumerate()
            .map(|(ix, field)| {
                cx.subscribe_in(&field.state, window, move |this, state, event: &InputEvent, _, cx| {
                    let name = this.fields[ix].name;
                    this.active = ix;
                    let message = match event {
                        InputEvent::Change => format!("{name}: change -> {:?}", truncate(&state.read(cx).value())),
                        InputEvent::PressEnter { secondary, shift } => {
                            format!("{name}: enter (secondary={secondary}, shift={shift})")
                        }
                        InputEvent::Focus => format!("{name}: focus"),
                        InputEvent::Blur => format!("{name}: blur"),
                    };
                    this.log.push(message);
                    cx.notify();
                })
            })
            .collect();
        Self {
            fields,
            active: 0,
            log: EventLog::default(),
            _subscriptions: subscriptions,
        }
    }

    fn field(&self, name: &str) -> &Entity<InputState> {
        &self.fields.iter().find(|f| f.name == name).expect("known field").state
    }

    fn active_state(&self) -> Entity<InputState> {
        self.fields[self.active].state.clone()
    }
}

fn truncate(value: &str) -> String {
    let mut out: String = value.chars().take(40).collect();
    if value.chars().count() > 40 {
        out.push('…');
    }
    out
}

impl Render for TextInputScreen {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let active = self.active_state();
        let (value, selection, cursor) = {
            let state = active.read(cx);
            (
                state.value(),
                state.selected_text().to_string(),
                state.cursor(),
            )
        };
        let focused = window.focused_input(cx).is_some();
        let (keyboard, keyboard_px) = crate::diagnostics::keyboard();
        let highlight = self.field("highlight").read(cx).value();

        let tools = h_flex()
            .flex_wrap()
            .gap_1()
            .child(Button::new("select-all").small().outline().label("Select all").on_click(cx.listener(
                |this, _, window, cx| {
                    let state = this.active_state();
                    state.update(cx, |state, cx| {
                        state.focus(window, cx);
                        state.select_all(window, cx);
                    });
                    this.log.push(format!("select all in {}", this.fields[this.active].name));
                    cx.notify();
                },
            )))
            .child(Button::new("copy-sel").small().outline().label("Copy selection").on_click(cx.listener(
                |this, _, _, cx| {
                    let text = this.active_state().read(cx).selected_text().to_string();
                    cx.write_to_clipboard(ClipboardItem::new_string(text.clone()));
                    this.log.push(format!("copied {:?}", truncate(&text)));
                    cx.notify();
                },
            )))
            .child(Button::new("read-clip").small().outline().label("Read clipboard").on_click(cx.listener(
                |this, _, _, cx| {
                    let text = cx.read_from_clipboard().and_then(|item| item.text());
                    this.log.push(format!("clipboard = {:?}", text.as_deref().map(truncate)));
                    cx.notify();
                },
            )))
            .child(Button::new("insert-sample").small().outline().label("Set Unicode sample").on_click(
                cx.listener(|this, _, window, cx| {
                    this.active_state()
                        .update(cx, |state, cx| state.set_value(UNICODE_SAMPLE, window, cx));
                    this.log.push("value set programmatically (no Change event expected)");
                    cx.notify();
                }),
            ))
            .child(Button::new("blur").small().outline().label("Blur (hide keyboard)").on_click(cx.listener(
                |this, _, window, cx| {
                    window.blur(cx);
                    this.log.push("window.blur()");
                    cx.notify();
                },
            )));

        v_flex()
            .gap_3()
            .child(
                ui::section("Live state of the last-used field", cx)
                    .child(ui::value_row("Field", self.fields[self.active].name, cx))
                    .child(ui::value_row("Value", truncate(&value), cx))
                    .child(ui::value_row("Chars / bytes", format!("{} / {}", value.chars().count(), value.len()), cx))
                    .child(ui::value_row("Cursor (byte offset)", cursor.to_string(), cx))
                    .child(ui::value_row("Selection", format!("{:?}", truncate(&selection)), cx))
                    .child(ui::value_row("Input focused", focused.to_string(), cx))
                    .child(ui::value_row(
                        "Keyboard (host)",
                        format!("visible={keyboard} height={keyboard_px}px"),
                        cx,
                    ))
                    .child(ui::value_row(
                        "Viewport / visual viewport",
                        format!(
                            "{:.0}×{:.0} / {:.0}×{:.0}",
                            window.viewport_size().width.as_f32(),
                            window.viewport_size().height.as_f32(),
                            window.visual_viewport_bounds().size.width.as_f32(),
                            window.visual_viewport_bounds().size.height.as_f32()
                        ),
                        cx,
                    ))
                    .child(tools),
            )
            .child(
                ui::section("Input", cx)
                    .child(ui::hint(
                        "Try: tap to focus, type, backspace, long-press / double-tap to select a \
                         word, drag the handles, then Cut/Copy/Paste/Select All from the edit menu. \
                         Use a CJK or Korean IME to test composition.",
                        cx,
                    ))
                    .child(Input::new(self.field("unicode")).cleanable(true))
                    .child(Input::new(self.field("empty")).cleanable(true))
                    .child(Input::new(self.field("long")))
                    .child(
                        Input::new(self.field("password"))
                            .mask_toggle()
                            .prefix(gpui_kit::component::Icon::new(IconName::EyeOff).small()),
                    ),
            )
            .child(
                ui::section("Formatted input", cx)
                    .child(Input::new(self.field("phone mask")))
                    .child(Input::new(self.field("amount mask")))
                    .child(Input::new(self.field("digits only")))
                    .child(Input::new(self.field("disabled")).disabled(true)),
            )
            .child(
                ui::section("Prefix / suffix / Clipboard", cx)
                    .child(
                        Input::new(self.field("url")).suffix(
                            Clipboard::new("copy-url")
                                .value_fn({
                                    let state = self.field("url").clone();
                                    move |_, cx| state.read(cx).value()
                                })
                                .on_copied(|value, window, cx| {
                                    window.push_notification(format!("Copied {value}"), cx)
                                }),
                        ),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(Clipboard::new("copy-sample").value(UNICODE_SAMPLE).on_copied({
                                let view = cx.entity();
                                move |value: SharedString, _, cx| {
                                    view.update(cx, |this, cx| {
                                        this.log.push(format!("Clipboard copied {:?}", truncate(&value)));
                                        cx.notify();
                                    })
                                }
                            }))
                            .child(ui::hint("Copies the Unicode sample; paste it into any field.", cx)),
                    ),
            )
            .child(
                ui::section("InputGroup", cx).child(
                    InputGroup::new("group")
                        .addon(InputGroupAddon::new("at").child(InputGroupText::new().child("@")))
                        .input(Input::new(self.field("group")))
                        .addon(
                            InputGroupAddon::new("go").child(
                                InputGroupButton::new("go-btn")
                                    .label("Check")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        let name = this.field("group").read(cx).value();
                                        this.log.push(format!("check username {name:?}"));
                                        cx.notify();
                                    })),
                            ),
                        ),
                ),
            )
            .child(
                ui::section("Label with highlights", cx)
                    .child(Input::new(self.field("highlight")))
                    .child(Label::new("Hello World, hello again").highlights(HighlightsMatch::Full(highlight.clone())))
                    .child(Label::new("Hello prefix only").highlights(HighlightsMatch::Prefix(highlight)))
                    .child(Label::new("Card number").secondary("(optional)"))
                    .child(Label::new("secret").masked(true)),
            )
            .child(self.log.render(cx))
            .child(div().h_64().child(ui::hint(
                "Spacer: the last fields sit low on the page so the keyboard has to push them up.",
                cx,
            )))
    }
}
