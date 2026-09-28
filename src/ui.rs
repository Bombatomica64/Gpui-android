//! Small layout helpers shared by every screen.

use std::collections::VecDeque;

use gpui::{
    AnyElement, App, Div, Hsla, IntoElement, ParentElement, SharedString, Styled, div, prelude::*,
};
use gpui_kit::component::{ActiveTheme as _, StyledExt as _, h_flex, tag::Tag, v_flex};

use crate::matrix::Status;

/// A titled card grouping one scenario.
pub fn section(title: impl Into<SharedString>, cx: &App) -> Div {
    v_flex()
        .w_full()
        .gap_3()
        .p_3()
        .rounded(cx.theme().radius_lg)
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().background)
        .child(div().text_sm().font_semibold().child(title.into()))
}

/// Muted helper text explaining what to try.
pub fn hint(text: impl Into<SharedString>, cx: &App) -> Div {
    div()
        .text_xs()
        .text_color(cx.theme().muted_foreground)
        .child(text.into())
}

/// `label: value` line used under interactive controls.
pub fn value_row(label: impl Into<SharedString>, value: impl Into<SharedString>, cx: &App) -> Div {
    h_flex()
        .gap_2()
        .text_xs()
        .child(
            div()
                .text_color(cx.theme().muted_foreground)
                .child(label.into()),
        )
        .child(div().font_family(cx.theme().mono_font_family.clone()).child(value.into()))
}

pub fn status_tag(status: Status) -> Tag {
    let tag = match status {
        Status::Working => Tag::success(),
        Status::Partial => Tag::warning(),
        Status::Broken => Tag::danger(),
        Status::NotApplicable => Tag::secondary(),
        Status::Untested => Tag::info(),
    };
    tag.outline().child(status.label())
}

pub fn status_color(status: Status, cx: &App) -> Hsla {
    match status {
        Status::Working => cx.theme().success,
        Status::Partial => cx.theme().warning,
        Status::Broken => cx.theme().danger,
        Status::NotApplicable | Status::Untested => cx.theme().muted_foreground,
    }
}

/// Wall-clock `HH:MM:SS` in the device's local time zone.
pub fn timestamp() -> String {
    chrono::Local::now().format("%H:%M:%S").to_string()
}

/// A short, newest-first log of events for one screen, so it is visible that
/// handlers actually fired.
#[derive(Default)]
pub struct EventLog {
    entries: VecDeque<String>,
    total: usize,
}

impl EventLog {
    const CAPACITY: usize = 6;

    pub fn push(&mut self, message: impl Into<String>) {
        let message = message.into();
        log::info!("event: {message}");
        self.total += 1;
        self.entries.push_front(format!("{} {}", timestamp(), message));
        self.entries.truncate(Self::CAPACITY);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.total = 0;
    }

    pub fn render(&self, cx: &App) -> AnyElement {
        v_flex()
            .w_full()
            .gap_0p5()
            .p_2()
            .rounded(cx.theme().radius)
            .bg(cx.theme().muted)
            .text_xs()
            .font_family(cx.theme().mono_font_family.clone())
            .child(
                div()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!("Events ({} total):", self.total)),
            )
            .when(self.entries.is_empty(), |this| {
                this.child(div().text_color(cx.theme().muted_foreground).child("— none yet —"))
            })
            .children(self.entries.iter().map(|entry| div().child(entry.clone())))
            .into_any_element()
    }
}

/// Kit extension traits every screen needs in scope.
pub mod prelude {
    pub use gpui_kit::component::button::{ButtonVariants as _, ToggleVariants as _};
    pub use gpui_kit::component::{
        ActiveTheme as _, Colorize as _, Disableable as _, Selectable as _, Sizable as _, StyledExt as _,
    };
}
