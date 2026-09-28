//! Demo screens. `title` must match the "Demo screen" column of COMPONENT_MATRIX.md.

use gpui::{AnyView, App, Window};

mod buttons;
mod choice;
mod number_otp;
pub mod text_input;
mod textarea;
pub mod themes;
mod toolbar;

pub struct ScreenDef {
    pub title: &'static str,
    pub build: fn(&mut Window, &mut App) -> AnyView,
    /// The screen contains its own (virtualized) scroll container.
    pub owns_scroll: bool,
}

macro_rules! screen {
    ($title:expr, $ty:path) => {
        screen!($title, $ty, false)
    };
    ($title:expr, $ty:path, $owns_scroll:expr) => {
        ScreenDef {
            title: $title,
            build: |window, cx| {
                use gpui::AppContext as _;
                cx.new(|cx| <$ty>::new(window, cx)).into()
            },
            owns_scroll: $owns_scroll,
        }
    };
}

pub static SCREENS: &[ScreenDef] = &[
    screen!("Buttons", buttons::ButtonsScreen),
    screen!("Toolbar", toolbar::ToolbarScreen),
    screen!("Choice Controls", choice::ChoiceScreen),
    screen!("Text Input", text_input::TextInputScreen),
    screen!("Textarea & Editor", textarea::TextareaScreen),
    screen!("Number & OTP", number_otp::NumberOtpScreen),
    screen!("Themes", themes::ThemesScreen),
];

/// Lab tools listed above the component catalog.
pub static TOOLS: &[&str] = &["Themes"];

pub fn find(title: &str) -> Option<&'static ScreenDef> {
    SCREENS.iter().find(|screen| screen.title == title)
}

pub fn init(cx: &mut App) {
    themes::init(cx);
}
