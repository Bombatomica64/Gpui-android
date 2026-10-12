use gpui::{
    App, AppContext as _, Context, Entity, Global, IntoElement, ParentElement, SharedString,
    Styled, Window, div, prelude::*, px,
};
use gpui_kit::component::{
    button::Button,
    checkbox::Checkbox,
    form::{field, v_form},
    group_box::{GroupBox, GroupBoxVariant, GroupBoxVariants as _},
    h_flex,
    input::{Input, InputState},
    setting::{SettingField, SettingGroup, SettingItem, SettingPage, Settings},
    switch::Switch,
    v_flex,
};

use crate::ui::{self, EventLog, prelude::*};

#[derive(Clone)]
struct Prefs {
    notifications: bool,
    autosave: bool,
    name: SharedString,
    language: SharedString,
}

impl Global for Prefs {}

fn prefs(cx: &App) -> &Prefs {
    cx.global::<Prefs>()
}

fn update(cx: &mut App, f: impl FnOnce(&mut Prefs)) {
    let mut prefs = cx.global::<Prefs>().clone();
    f(&mut prefs);
    log::info!(
        "settings: notifications={} autosave={} name={:?} language={:?}",
        prefs.notifications,
        prefs.autosave,
        prefs.name,
        prefs.language
    );
    cx.set_global(prefs);
    cx.refresh_windows();
}

pub struct SettingsFormScreen {
    name: Entity<InputState>,
    email: Entity<InputState>,
    bio: Entity<InputState>,
    agree: bool,
    compact: bool,
    submitted: Option<String>,
    log: EventLog,
}

impl SettingsFormScreen {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        if !cx.has_global::<Prefs>() {
            cx.set_global(Prefs {
                notifications: true,
                autosave: false,
                name: "Lab user".into(),
                language: "en".into(),
            });
        }
        Self {
            name: cx.new(|cx| InputState::new(window, cx).placeholder("Jane Doe")),
            email: cx.new(|cx| InputState::new(window, cx).placeholder("jane@example.com")),
            bio: cx.new(|cx| InputState::new(window, cx).placeholder("Short bio")),
            agree: false,
            compact: false,
            submitted: None,
            log: EventLog::default(),
        }
    }
}

crate::hot_render!(SettingsFormScreen);

impl SettingsFormScreen {
    fn render_view(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let settings = Settings::new("lab-settings").pages(vec![
            SettingPage::new("General").default_open(true).groups(vec![
                SettingGroup::new().title("Behaviour").items(vec![
                    SettingItem::new(
                        "Notifications",
                        SettingField::switch(
                            |cx: &App| prefs(cx).notifications,
                            |value: bool, cx: &mut App| update(cx, |p| p.notifications = value),
                        ),
                    )
                    .description("Show a toast when something finishes."),
                    SettingItem::new(
                        "Autosave",
                        SettingField::checkbox(
                            |cx: &App| prefs(cx).autosave,
                            |value: bool, cx: &mut App| update(cx, |p| p.autosave = value),
                        ),
                    ),
                ]),
                SettingGroup::new()
                    .title("Profile")
                    .variant(GroupBoxVariant::Outline)
                    .items(vec![
                        SettingItem::new(
                            "Display name",
                            SettingField::input(
                                |cx: &App| prefs(cx).name.clone(),
                                |value: SharedString, cx: &mut App| update(cx, |p| p.name = value),
                            ),
                        ),
                        SettingItem::new(
                            "Language",
                            SettingField::dropdown(
                                vec![
                                    ("en".into(), "English".into()),
                                    ("it".into(), "Italiano".into()),
                                    ("ja".into(), "日本語".into()),
                                ],
                                |cx: &App| prefs(cx).language.clone(),
                                |value: SharedString, cx: &mut App| {
                                    update(cx, |p| p.language = value)
                                },
                            ),
                        ),
                    ]),
            ]),
            SettingPage::new("About").group(SettingGroup::new().title("Build").item(
                SettingItem::render(|_, _, _| {
                    div().child(format!("GPUI Kit {}", crate::GPUI_KIT_VERSION))
                }),
            )),
        ]);

        let p = prefs(cx).clone();
        let form = v_form()
            .child(
                field()
                    .label("Name")
                    .required(true)
                    .child(Input::new(&self.name)),
            )
            .child(
                field()
                    .label("Email")
                    .description("We never share it.")
                    .child(Input::new(&self.email)),
            )
            .child(field().label("Bio").child(Input::new(&self.bio)))
            .child(
                field().child(
                    Checkbox::new("agree")
                        .label("I agree to the terms")
                        .checked(self.agree)
                        .on_change(cx.listener(|this, checked: &bool, _, cx| {
                            this.agree = *checked;
                            cx.notify();
                        })),
                ),
            )
            .footer(
                h_flex().gap_2().child(
                    Button::new("submit")
                        .primary()
                        .label("Submit")
                        .disabled(!self.agree)
                        .on_click(cx.listener(|this, _, _, cx| {
                            let name = this.name.read(cx).value();
                            let email = this.email.read(cx).value();
                            this.submitted = Some(format!("name={name:?} email={email:?}"));
                            this.log.push(format!("form submitted: {name:?}"));
                            cx.notify();
                        })),
                ),
            );

        v_flex()
            .gap_3()
            .child(
                ui::section("Settings (desktop sidebar layout)", cx)
                    .child(
                        div()
                            .h(px(460.))
                            .border_1()
                            .border_color(cx.theme().border)
                            .child(settings),
                    )
                    .child(ui::value_row(
                        "Stored values",
                        format!(
                            "notifications={} autosave={} name={:?} language={:?}",
                            p.notifications, p.autosave, p.name, p.language
                        ),
                        cx,
                    )),
            )
            .child(ui::section("Form", cx).child(form).child(ui::value_row(
                "Submitted",
                self.submitted.clone().unwrap_or_else(|| "—".into()),
                cx,
            )))
            .child(
                ui::section("GroupBox variants", cx)
                    .child(
                        Button::new("gb-compact")
                            .small()
                            .outline()
                            .selected(self.compact)
                            .label("Toggle switch")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.compact = !this.compact;
                                cx.notify();
                            })),
                    )
                    .children(
                        [
                            ("Normal", GroupBoxVariant::Normal),
                            ("Fill", GroupBoxVariant::Fill),
                            ("Outline", GroupBoxVariant::Outline),
                        ]
                        .map(|(label, variant)| {
                            let group = GroupBox::new()
                                .title(label)
                                .footer("Footer text outside the card.")
                                .child(
                                    h_flex()
                                        .justify_between()
                                        .child("Make profile private")
                                        .child(Switch::new(label).checked(self.compact)),
                                );
                            match variant {
                                GroupBoxVariant::Fill => group.fill(),
                                GroupBoxVariant::Outline => group.outline(),
                                _ => group.normal(),
                            }
                        }),
                    ),
            )
            .child(self.log.render(cx))
    }
}
