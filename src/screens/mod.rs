//! Demo screens. `title` must match the "Demo screen" column of COMPONENT_MATRIX.md.

use gpui::{AnyView, App, Window};

mod buttons;
mod charts;
mod chat;
mod choice;
mod command;
mod date_time;
mod diagnostics;
mod dialogs;
mod disclosure;
mod display;
mod dock;
mod feedback;
mod layout;
mod lists;
mod navigation;
mod number_otp;
mod platform_apis;
mod popovers;
mod questionnaire;
mod rich_text;
mod scroll;
mod select;
mod settings_form;
mod stress;
mod tables;
pub mod text_input;
mod textarea;
pub mod themes;
mod toolbar;
mod touch;

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
    screen!("Select & Combobox", select::SelectScreen),
    screen!("Date & Time", date_time::DateTimeScreen),
    screen!("Dialogs & Sheets", dialogs::DialogsScreen),
    screen!("Popovers & Menus", popovers::PopoversScreen),
    screen!("Navigation", navigation::NavigationScreen),
    screen!("Command", command::CommandScreen),
    screen!("Dock", dock::DockScreen, true),
    screen!("Tables", tables::TablesScreen, true),
    screen!("Lists & Trees", lists::ListsScreen, true),
    screen!("Display", display::DisplayScreen),
    screen!("Disclosure", disclosure::DisclosureScreen),
    screen!("Feedback", feedback::FeedbackScreen),
    screen!("Layout", layout::LayoutScreen),
    screen!("Settings & Form", settings_form::SettingsFormScreen),
    screen!("Questionnaire", questionnaire::QuestionnaireScreen),
    screen!("Charts", charts::ChartsScreen),
    screen!("Rich Text", rich_text::RichTextScreen),
    screen!("Chat", chat::ChatScreen, true),
    screen!("Touch Lab", touch::TouchScreen),
    screen!("Scroll Stress", scroll::ScrollScreen, true),
    screen!("Diagnostics", diagnostics::DiagnosticsScreen),
    screen!("Platform APIs", platform_apis::PlatformApisScreen),
    screen!("Stress Test", stress::StressScreen),
    screen!("Themes", themes::ThemesScreen),
];

/// Lab tools listed above the component catalog.
pub static TOOLS: &[&str] = &[
    "Touch Lab",
    "Scroll Stress",
    "Diagnostics",
    "Platform APIs",
    "Stress Test",
    "Themes",
];

pub fn find(title: &str) -> Option<&'static ScreenDef> {
    SCREENS.iter().find(|screen| screen.title == title)
}

pub fn init(cx: &mut App) {
    themes::init(cx);
}
