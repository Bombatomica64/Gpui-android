//! Theme switching at runtime: System / Light / Dark plus a few bundled Kit themes.

use std::rc::Rc;

use gpui::{
    App, Context, Global, IntoElement, ParentElement, SharedString, Styled, Window, div, prelude::*,
};
use gpui_kit::component::{
    Theme, ThemeConfig, ThemeMode, ThemeRegistry, button::Button, h_flex, v_flex,
};

use crate::ui::{self, EventLog, prelude::*};

const BUNDLED: &[&str] = &[
    include_str!("../../themes/catppuccin.json"),
    include_str!("../../themes/tokyonight.json"),
    include_str!("../../themes/gruvbox.json"),
    include_str!("../../themes/ayu.json"),
];

#[derive(Clone, PartialEq, Debug)]
pub enum Choice {
    System,
    Light,
    Dark,
    Named(SharedString),
}

struct CurrentChoice(Choice);
impl Global for CurrentChoice {}

pub fn init(cx: &mut App) {
    for source in BUNDLED {
        if let Err(err) = ThemeRegistry::global_mut(cx).load_themes_from_str(source) {
            log::error!("loading bundled theme failed: {err:#}");
        }
    }
    cx.set_global(CurrentChoice(Choice::System));
}

/// Re-applies the theme when the platform appearance changes and the user picked System.
pub fn follow_system(window: &mut Window, cx: &mut App) {
    if cx.global::<CurrentChoice>().0 == Choice::System {
        log::info!(
            "theme: following system appearance {:?}",
            window.appearance()
        );
        Theme::sync_system_appearance(Some(window), cx);
    }
}

/// Applies the current choice to a freshly opened window.
pub fn apply_saved(window: &mut Window, cx: &mut App) {
    let choice = cx.global::<CurrentChoice>().0.clone();
    apply(choice, window, cx);
}

fn reset_to_defaults(cx: &mut App) {
    let light = ThemeRegistry::global(cx).default_light_theme().clone();
    let dark = ThemeRegistry::global(cx).default_dark_theme().clone();
    Theme::update(cx, |theme| {
        theme.apply_config(&light);
        theme.apply_config(&dark);
    });
}

pub fn apply(choice: Choice, window: &mut Window, cx: &mut App) {
    log::info!("theme: {choice:?}");
    match &choice {
        Choice::System => {
            reset_to_defaults(cx);
            Theme::sync_system_appearance(Some(window), cx);
        }
        Choice::Light => {
            reset_to_defaults(cx);
            Theme::change(ThemeMode::Light, Some(window), cx);
        }
        Choice::Dark => {
            reset_to_defaults(cx);
            Theme::change(ThemeMode::Dark, Some(window), cx);
        }
        Choice::Named(name) => {
            let config: Option<Rc<ThemeConfig>> =
                ThemeRegistry::global(cx).themes().get(name).cloned();
            match config {
                Some(config) => {
                    Theme::update(cx, |theme| theme.apply_config(&config));
                    Theme::change(config.mode, Some(window), cx);
                }
                None => log::error!("theme {name} is not registered"),
            }
        }
    }
    cx.set_global(CurrentChoice(choice));
}

pub struct ThemesScreen {
    log: EventLog,
    _appearance: gpui::Subscription,
}

impl ThemesScreen {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let appearance = cx.observe_window_appearance(window, |this, window, cx| {
            // The app-level observer (LabApp) applies the theme; this only logs it.
            this.log
                .push(format!("system appearance -> {:?}", window.appearance()));
            cx.notify();
        });
        Self {
            log: EventLog::default(),
            _appearance: appearance,
        }
    }

    fn choose(&mut self, choice: Choice, window: &mut Window, cx: &mut Context<Self>) {
        self.log.push(format!("apply {choice:?}"));
        apply(choice, window, cx);
        cx.notify();
    }
}

#[gpui_hot::hot]
impl Render for ThemesScreen {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let current = cx.global::<CurrentChoice>().0.clone();
        let base = [
            ("System", Choice::System),
            ("Light", Choice::Light),
            ("Dark", Choice::Dark),
        ]
        .into_iter()
        .map(|(label, choice)| {
            let selected = current == choice;
            Button::new(label)
                .label(label)
                .when(selected, |b| b.primary())
                .when(!selected, |b| b.outline())
                .on_click(
                    cx.listener(move |this, _, window, cx| this.choose(choice.clone(), window, cx)),
                )
        });

        let mut named: Vec<SharedString> =
            ThemeRegistry::global(cx).themes().keys().cloned().collect();
        named.sort();
        let named = named.into_iter().map(|name| {
            let selected = current == Choice::Named(name.clone());
            Button::new(SharedString::from(format!("theme-{name}")))
                .small()
                .label(name.clone())
                .when(selected, |b| b.primary())
                .when(!selected, |b| b.outline())
                .on_click(cx.listener(move |this, _, window, cx| {
                    this.choose(Choice::Named(name.clone()), window, cx)
                }))
        });

        let swatches = [
            ("background", cx.theme().background),
            ("foreground", cx.theme().foreground),
            ("primary", cx.theme().primary),
            ("secondary", cx.theme().secondary),
            ("accent", cx.theme().accent),
            ("muted", cx.theme().muted),
            ("border", cx.theme().border),
            ("success", cx.theme().success),
            ("warning", cx.theme().warning),
            ("danger", cx.theme().danger),
        ]
        .into_iter()
        .map(|(name, color)| {
            v_flex()
                .items_center()
                .gap_1()
                .w_16()
                .child(
                    div()
                        .size_10()
                        .rounded(cx.theme().radius)
                        .border_1()
                        .border_color(cx.theme().border)
                        .bg(color),
                )
                .child(div().text_xs().child(name))
        });

        v_flex()
            .gap_3()
            .child(
                ui::section("Mode", cx)
                    .child(h_flex().gap_2().children(base))
                    .child(ui::value_row("Current", format!("{current:?}"), cx))
                    .child(ui::value_row(
                        "Theme name",
                        cx.theme().theme_name().clone(),
                        cx,
                    ))
                    .child(ui::value_row(
                        "Window appearance (from platform)",
                        format!("{:?}", window.appearance()),
                        cx,
                    ))
                    .child(ui::hint(
                        "System follows window.appearance() as reported by gpui-mobile; \
                         toggle Android dark mode while this screen is open to test.",
                        cx,
                    )),
            )
            .child(
                ui::section("Bundled Kit themes", cx)
                    .child(h_flex().flex_wrap().gap_1().children(named)),
            )
            .child(
                ui::section("Palette", cx).child(h_flex().flex_wrap().gap_2().children(swatches)),
            )
            .child(self.log.render(cx))
    }
}
