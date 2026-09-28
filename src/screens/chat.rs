use std::cell::RefCell;
use std::rc::Rc;

use gpui::{
    AppContext as _, Axis, Context, Entity, IntoElement, ParentElement, Render, Styled,
    Subscription, Window, div, prelude::*,
};
use gpui_kit::base::TextView;
use gpui_kit::component::{
    Icon, IconName,
    attachment::{
        Attachment, AttachmentContent, AttachmentDescription, AttachmentMedia, AttachmentStatus,
        AttachmentTitle,
    },
    avatar::Avatar,
    bubble::{Bubble, BubbleContent, BubbleVariant},
    button::Button,
    h_flex,
    input::{InputEvent, Textarea, TextareaState},
    message::{Message, MessageAlignment, MessageContent, MessageFooter, MessageHeader},
    message_scroller::{MessageScroller, MessageScrollerState},
    v_flex,
};

use crate::ui::{self, prelude::*};

#[derive(Clone)]
struct ChatMessage {
    mine: bool,
    text: String,
    variant: BubbleVariant,
}

fn seed() -> Vec<ChatMessage> {
    let variants = [
        BubbleVariant::Filled,
        BubbleVariant::Secondary,
        BubbleVariant::Muted,
        BubbleVariant::Tinted,
        BubbleVariant::Outline,
        BubbleVariant::Ghost,
    ];
    (0..40)
        .map(|i| ChatMessage {
            mine: i % 3 == 0,
            text: match i % 5 {
                0 => format!("Message {i}: short."),
                1 => format!("Message {i}: **Markdown** with `code` and a [link](https://gpui-kit.com)."),
                2 => format!("Message {i}: English Italiano àèéìòù 日本語 中文测试 한국어 😀🚀🦀"),
                3 => format!("Message {i}: a longer paragraph that wraps across several lines on a phone, to check bubble width limits and text selection by long press."),
                _ => format!("Message {i}:\n\n```rust\nfn answer() -> u32 {{ 42 }}\n```"),
            },
            variant: variants[i % variants.len()].clone(),
        })
        .collect()
}

pub struct ChatScreen {
    messages: Rc<RefCell<Vec<ChatMessage>>>,
    scroller: Entity<MessageScrollerState>,
    composer: Entity<TextareaState>,
    attachments: Vec<(&'static str, AttachmentStatus)>,
    _subscriptions: Vec<Subscription>,
}

impl ChatScreen {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let messages = Rc::new(RefCell::new(seed()));
        let count = messages.borrow().len();
        let scroller = cx.new(|cx| MessageScrollerState::new(count, cx));
        let composer = cx.new(|cx| {
            TextareaState::new(window, cx)
                .auto_grow(1, 4)
                .placeholder("Message…")
        });
        let subscription = cx.subscribe_in(
            &composer,
            window,
            |this, _, event: &InputEvent, window, cx| {
                if let InputEvent::PressEnter {
                    secondary: true, ..
                } = event
                {
                    this.send(window, cx);
                }
            },
        );
        Self {
            messages,
            scroller,
            composer,
            attachments: vec![
                ("report.pdf", AttachmentStatus::Complete),
                ("photo.png", AttachmentStatus::Uploading),
            ],
            _subscriptions: vec![subscription],
        }
    }

    fn send(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let text = self.composer.read(cx).value().trim().to_string();
        if text.is_empty() {
            return;
        }
        log::info!("event: chat send {} chars", text.chars().count());
        self.messages.borrow_mut().push(ChatMessage {
            mine: true,
            text,
            variant: BubbleVariant::Filled,
        });
        self.messages.borrow_mut().push(ChatMessage {
            mine: false,
            text: format!("Echo at {}: received.", ui::timestamp()),
            variant: BubbleVariant::Secondary,
        });
        self.scroller.update(cx, |state, cx| {
            state.append(2, cx);
            state.scroll_to_end(cx);
        });
        self.composer
            .update(cx, |state, cx| state.set_value("", window, cx));
        cx.notify();
    }
}

impl Render for ChatScreen {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let messages = self.messages.clone();
        let scroller =
            MessageScroller::new("chat-scroller", self.scroller.clone(), move |ix, _, _| {
                let message = messages.borrow().get(ix).cloned();
                let Some(message) = message else {
                    return div().into_any_element();
                };
                let alignment = if message.mine {
                    MessageAlignment::End
                } else {
                    MessageAlignment::Start
                };
                let name = if message.mine { "You" } else { "Ada" };
                div()
                    .py_1()
                    .child(
                        Message::new()
                            .alignment(alignment)
                            .avatar(Avatar::new().name(name).size_7())
                            .header(MessageHeader::new().child(name))
                            .content(
                                MessageContent::new().bubble(
                                    Bubble::new().with_variant(message.variant.clone()).content(
                                        BubbleContent::new().child(
                                            TextView::markdown(("chat", ix), message.text.clone())
                                                .selectable(true)
                                                .w_full()
                                                .min_w_0(),
                                        ),
                                    ),
                                ),
                            )
                            .footer(MessageFooter::new().child(format!("#{ix}"))),
                    )
                    .into_any_element()
            })
            .with_bottom_fade(cx.theme().background);

        let attachments = self
            .attachments
            .iter()
            .enumerate()
            .map(|(ix, (name, status))| {
                Attachment::new()
                    .id(("attachment", ix))
                    .small()
                    .axis(Axis::Horizontal)
                    .status(status.clone())
                    .progress(if matches!(status, AttachmentStatus::Uploading) {
                        45.
                    } else {
                        100.
                    })
                    .media(AttachmentMedia::new().child(Icon::new(IconName::FileText)))
                    .content(
                        AttachmentContent::new()
                            .title(AttachmentTitle::new(*name))
                            .description(AttachmentDescription::new(format!("{status:?}"))),
                    )
                    .on_remove(cx.listener(move |this, _, _, cx| {
                        if ix < this.attachments.len() {
                            let (name, _) = this.attachments.remove(ix);
                            log::info!("event: attachment removed {name}");
                        }
                        cx.notify();
                    }))
            });

        v_flex()
            .size_full()
            .child(div().flex_1().min_h_0().child(scroller))
            .child(
                v_flex()
                    .flex_none()
                    .p_2()
                    .gap_2()
                    .border_t_1()
                    .border_color(cx.theme().border)
                    .bg(cx.theme().background)
                    .when(!self.attachments.is_empty(), |this| {
                        this.child(h_flex().gap_2().children(attachments))
                    })
                    .child(
                        h_flex()
                            .gap_2()
                            .items_end()
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .child(Textarea::new(&self.composer)),
                            )
                            .child(
                                Button::new("send")
                                    .primary()
                                    .icon(IconName::ArrowUp)
                                    .on_click(
                                        cx.listener(|this, _, window, cx| this.send(window, cx)),
                                    ),
                            ),
                    )
                    .child(ui::hint(
                        "The composer and Send button must stay above the keyboard. \
                         Scroll up to reveal the 'jump to latest' button.",
                        cx,
                    )),
            )
    }
}
