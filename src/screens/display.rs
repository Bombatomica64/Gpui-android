use std::sync::Arc;

use gpui::{
    Context, Image, ImageFormat, IntoElement, ObjectFit, ParentElement, Render, Styled, Window,
    div, img, prelude::*, px,
};
use gpui_kit::component::{
    Icon, IconName, Size,
    avatar::{Avatar, AvatarGroup},
    badge::Badge,
    button::Button,
    description_list::DescriptionList,
    empty::{
        Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyMediaVariant,
        EmptyTitle,
    },
    h_flex,
    tag::Tag,
    v_flex,
};

use crate::ui::{self, EventLog, prelude::*};

const SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 120 80">
<rect width="120" height="80" rx="10" fill="#1e293b"/>
<circle cx="30" cy="40" r="18" fill="#f59e0b"/>
<path d="M55 62 L78 26 L101 62 Z" fill="#22c55e"/>
</svg>"##;

pub struct DisplayScreen {
    count: usize,
    size: Size,
    log: EventLog,
}

impl DisplayScreen {
    pub fn new(_: &mut Window, _: &mut Context<Self>) -> Self {
        Self {
            count: 3,
            size: Size::Medium,
            log: EventLog::default(),
        }
    }
}

impl Render for DisplayScreen {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let names = [
            "Ada Lovelace",
            "Bruno Rossi",
            "Chiara Bianchi",
            "Dmitri Ivanov",
            "Emi Tanaka",
            "Farah Khan",
            "김민준",
        ];
        let icons = [
            IconName::Bell,
            IconName::Calendar,
            IconName::Check,
            IconName::CircleAlert,
            IconName::Copy,
            IconName::Eye,
            IconName::Folder,
            IconName::Github,
            IconName::Globe,
            IconName::Heart,
            IconName::Inbox,
            IconName::Info,
            IconName::Loader,
            IconName::Map,
            IconName::Menu,
            IconName::Moon,
            IconName::Palette,
            IconName::Search,
            IconName::Settings,
            IconName::Star,
            IconName::Sun,
            IconName::ThumbsUp,
            IconName::TriangleAlert,
            IconName::User,
        ];
        let size = self.size;
        v_flex()
            .gap_3()
            .child(
                h_flex().gap_1().children([("Small", Size::Small), ("Medium", Size::Medium), ("Large", Size::Large)].map(
                    |(label, s)| {
                        Button::new(label).small().outline().selected(size == s).label(label).on_click(cx.listener(
                            move |this, _, _, cx| {
                                this.size = s;
                                this.log.push(format!("size -> {label}"));
                                cx.notify();
                            },
                        ))
                    },
                )),
            )
            .child(
                ui::section("DescriptionList", cx)
                    .child(
                        DescriptionList::new()
                            .columns(2)
                            .with_size(size)
                            .item("Name", "Ada Lovelace", 1)
                            .item("Role", "Engineer", 1)
                            .item("Email", "ada@example.com", 2)
                            .separator()
                            .item("Address", "Via Roma 1, 20121 Milano, Italia — a long value that must wrap", 2),
                    )
                    .child(
                        DescriptionList::vertical()
                            .columns(1)
                            .with_size(size)
                            .item("Vertical layout", "Label above value", 1)
                            .item("Unicode", "日本語 中文 한국어 😀", 1),
                    ),
            )
            .child(
                ui::section("Avatar / AvatarGroup", cx)
                    .child(h_flex().gap_2().children(names.iter().take(5).map(|name| Avatar::new().name(*name).with_size(size))))
                    .child(h_flex().gap_2().child(Avatar::new().with_size(size)).child(Avatar::new().name("김민준").with_size(size)))
                    .child(AvatarGroup::new().with_size(size).limit(4).ellipsis().children(names.iter().map(|name| Avatar::new().name(*name)))),
            )
            .child(
                ui::section("Badge", cx)
                    .child(
                        h_flex()
                            .gap_6()
                            .child(Badge::new().count(self.count).with_size(size).child(Icon::new(IconName::Bell).large()))
                            .child(Badge::new().count(120).with_size(size).child(Icon::new(IconName::Inbox).large()))
                            .child(Badge::new().dot().with_size(size).child(Icon::new(IconName::User).large()))
                            .child(Badge::new().icon(IconName::Check).with_size(size).child(Avatar::new().name("Ada"))),
                    )
                    .child(
                        h_flex()
                            .gap_1()
                            .child(Button::new("badge-inc").small().outline().label("+1").on_click(cx.listener(|this, _, _, cx| {
                                this.count += 1;
                                this.log.push(format!("badge count -> {}", this.count));
                                cx.notify();
                            })))
                            .child(Button::new("badge-reset").small().outline().label("0 (hides)").on_click(cx.listener(|this, _, _, cx| {
                                this.count = 0;
                                this.log.push("badge count -> 0");
                                cx.notify();
                            }))),
                    )
                    .child(ui::value_row("Count", self.count.to_string(), cx)),
            )
            .child(
                ui::section("Tag", cx)
                    .child(
                        h_flex()
                            .flex_wrap()
                            .gap_1()
                            .child(Tag::primary().with_size(size).child("Primary"))
                            .child(Tag::secondary().with_size(size).child("Secondary"))
                            .child(Tag::success().with_size(size).child("Success"))
                            .child(Tag::warning().with_size(size).child("Warning"))
                            .child(Tag::danger().with_size(size).child("Danger"))
                            .child(Tag::info().with_size(size).child("Info"))
                            .child(Tag::primary().outline().with_size(size).child("Outline"))
                            .child(Tag::success().rounded_full().with_size(size).child("Rounded")),
                    ),
            )
            .child(
                ui::section("Icon", cx)
                    .child(h_flex().flex_wrap().gap_3().children(icons.into_iter().map(|icon| Icon::new(icon).with_size(size))))
                    .child(
                        h_flex()
                            .gap_3()
                            .child(Icon::new(gpui_kit::assets::IconName::Rocket).large())
                            .child(Icon::new(gpui_kit::assets::IconName::Bike).large())
                            .child(Icon::new(gpui_kit::assets::IconName::Croissant).large())
                            .child(Icon::new(IconName::Heart).large().text_color(cx.theme().danger)),
                    )
                    .child(ui::hint("Last row uses the full Lucide catalog (AllAssets) and a custom color.", cx)),
            )
            .child(
                ui::section("Image (img)", cx)
                    .child(
                        h_flex()
                            .gap_3()
                            .child(
                                img(Arc::new(Image::from_bytes(ImageFormat::Svg, SVG.as_bytes().to_vec())))
                                    .w(px(120.))
                                    .h(px(80.)),
                            )
                            .child(img("icons/github.svg").size(px(48.)).object_fit(ObjectFit::Contain))
                            .child(
                                img("icons/does-not-exist.svg")
                                    .size(px(48.))
                                    .with_fallback(|| div().text_xs().child("fallback").into_any_element()),
                            ),
                    )
                    .child(ui::hint(
                        "In-memory SVG bytes, an asset-source SVG, and a missing asset showing its fallback. \
                         Network images are not tested (the app sets no HTTP client).",
                        cx,
                    )),
            )
            .child(
                ui::section("Empty", cx).child(
                    Empty::new()
                        .header(
                            EmptyHeader::new()
                                .media(EmptyMedia::new().with_variant(EmptyMediaVariant::Icon).child(Icon::new(IconName::Folder)))
                                .title(EmptyTitle::new().child("No projects yet"))
                                .description(EmptyDescription::new().child("Create your first project to get started.")),
                        )
                        .content(EmptyContent::new().child(Button::new("empty-create").primary().label("Create project").on_click(
                            cx.listener(|this, _, _, cx| {
                                this.log.push("empty state action");
                                cx.notify();
                            }),
                        ))),
                ),
            )
            .child(self.log.render(cx))
    }
}
