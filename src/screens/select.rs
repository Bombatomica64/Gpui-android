use gpui::{
    AppContext as _, Context, Entity, Hsla, IntoElement, ParentElement, Styled, Subscription,
    Window, div, prelude::*,
};
use gpui_kit::component::{
    IndexPath,
    color_picker::{ColorPicker, ColorPickerEvent, ColorPickerState},
    combobox::{Combobox, ComboboxEvent, ComboboxState},
    h_flex,
    select::{SearchableVec, Select, SelectEvent, SelectState},
    v_flex,
};

use crate::ui::{self, EventLog, prelude::*};

const FRUITS: &[&str] = &[
    "Apple",
    "Banana",
    "Cherry",
    "Durian",
    "Elderberry",
    "Fig",
    "Grape",
];

const COUNTRIES: &[&str] = &[
    "Argentina",
    "Australia",
    "Austria",
    "Belgium",
    "Brazil",
    "Canada",
    "Chile",
    "China",
    "Colombia",
    "Czechia",
    "Denmark",
    "Egypt",
    "Finland",
    "France",
    "Germany",
    "Greece",
    "Hungary",
    "Iceland",
    "India",
    "Indonesia",
    "Ireland",
    "Italy",
    "Japan",
    "Kenya",
    "Korea",
    "Mexico",
    "Morocco",
    "Netherlands",
    "New Zealand",
    "Nigeria",
    "Norway",
    "Peru",
    "Philippines",
    "Poland",
    "Portugal",
    "Romania",
    "Spain",
    "Sweden",
    "Switzerland",
    "Thailand",
    "Turkey",
    "Ukraine",
    "United Kingdom",
    "United States",
    "Vietnam",
];

type Vec_ = SearchableVec<&'static str>;

pub struct SelectScreen {
    fruit: Entity<SelectState<Vec_>>,
    /// Popups anchor to the last-rendered element of a state, so a disabled
    /// demo needs its own state.
    fruit_disabled: Entity<SelectState<Vec_>>,
    country: Entity<SelectState<Vec_>>,
    single: Entity<ComboboxState<Vec_>>,
    multi: Entity<ComboboxState<Vec_>>,
    color: Entity<ColorPickerState>,
    selected_fruit: Option<&'static str>,
    selected_country: Option<&'static str>,
    single_value: Vec<&'static str>,
    multi_value: Vec<&'static str>,
    color_value: Option<Hsla>,
    log: EventLog,
    _subscriptions: Vec<Subscription>,
}

impl SelectScreen {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let fruit = cx.new(|cx| {
            SelectState::new(
                SearchableVec::new(FRUITS.to_vec()),
                Some(IndexPath::new(1)),
                window,
                cx,
            )
        });
        let fruit_disabled = cx.new(|cx| {
            SelectState::new(
                SearchableVec::new(FRUITS.to_vec()),
                Some(IndexPath::new(1)),
                window,
                cx,
            )
        });
        let country = cx.new(|cx| {
            SelectState::new(SearchableVec::new(COUNTRIES.to_vec()), None, window, cx)
                .searchable(true)
        });
        let single = cx.new(|cx| {
            ComboboxState::new(SearchableVec::new(COUNTRIES.to_vec()), vec![], window, cx)
                .searchable(true)
        });
        let multi = cx.new(|cx| {
            ComboboxState::new(
                SearchableVec::new(FRUITS.to_vec()),
                vec![IndexPath::new(0)],
                window,
                cx,
            )
            .multiple(true)
        });
        let color = cx.new(|cx| ColorPickerState::new(window, cx));

        let subscriptions = vec![
            cx.subscribe(&fruit, |this, _, event: &SelectEvent<Vec_>, cx| {
                let SelectEvent::Confirm(value) = event;
                this.selected_fruit = *value;
                this.log.push(format!("fruit select -> {value:?}"));
                cx.notify();
            }),
            cx.subscribe(&country, |this, _, event: &SelectEvent<Vec_>, cx| {
                let SelectEvent::Confirm(value) = event;
                this.selected_country = *value;
                this.log.push(format!("country select -> {value:?}"));
                cx.notify();
            }),
            cx.subscribe(&single, |this, _, event: &ComboboxEvent<Vec_>, cx| {
                match event {
                    ComboboxEvent::Change(values) => {
                        this.single_value = values.clone();
                        this.log.push(format!("combobox change -> {values:?}"));
                    }
                    ComboboxEvent::Confirm(values) => {
                        this.log.push(format!("combobox confirm -> {values:?}"));
                    }
                }
                cx.notify();
            }),
            cx.subscribe(&multi, |this, _, event: &ComboboxEvent<Vec_>, cx| {
                match event {
                    ComboboxEvent::Change(values) => {
                        this.multi_value = values.clone();
                        this.log.push(format!("multi change -> {values:?}"));
                    }
                    ComboboxEvent::Confirm(values) => {
                        this.log
                            .push(format!("multi confirm (closed) -> {values:?}"));
                    }
                }
                cx.notify();
            }),
            cx.subscribe(&color, |this, _, event: &ColorPickerEvent, cx| {
                let ColorPickerEvent::Change(color) = event;
                this.color_value = *color;
                this.log
                    .push(format!("color -> {:?}", color.map(|c| c.to_hex())));
                cx.notify();
            }),
        ];

        Self {
            fruit,
            fruit_disabled,
            country,
            single,
            multi,
            color,
            selected_fruit: Some(FRUITS[1]),
            selected_country: None,
            single_value: vec![],
            multi_value: vec![FRUITS[0]],
            color_value: None,
            log: EventLog::default(),
            _subscriptions: subscriptions,
        }
    }
}

#[gpui_hot::hot]
impl Render for SelectScreen {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_3()
            .child(
                ui::section("Select", cx)
                    .child(Select::new(&self.fruit).placeholder("Pick a fruit"))
                    .child(ui::value_row(
                        "Fruit",
                        format!("{:?}", self.selected_fruit),
                        cx,
                    ))
                    .child(
                        Select::new(&self.country)
                            .placeholder("Country (searchable, 45 items)")
                            .search_placeholder("Search countries…")
                            .cleanable(true),
                    )
                    .child(ui::value_row(
                        "Country",
                        format!("{:?}", self.selected_country),
                        cx,
                    ))
                    .child(Select::new(&self.fruit_disabled).disabled(true))
                    .child(ui::hint(
                        "Open, scroll and fling the country list; typing in its search field \
                         should raise the keyboard without covering the list.",
                        cx,
                    )),
            )
            .child(
                ui::section("Combobox", cx)
                    .child(
                        Combobox::new(&self.single)
                            .placeholder("Search a country…")
                            .search_placeholder("Type to filter"),
                    )
                    .child(ui::value_row(
                        "Single",
                        format!("{:?}", self.single_value),
                        cx,
                    ))
                    .child(Combobox::new(&self.multi).placeholder("Fruits (multiple)"))
                    .child(ui::value_row(
                        "Multiple",
                        format!("{:?}", self.multi_value),
                        cx,
                    ))
                    .child(ui::hint(
                        "Multi-select stays open until you tap outside or press back.",
                        cx,
                    )),
            )
            .child(
                ui::section("ColorPicker", cx)
                    .child(
                        h_flex()
                            .gap_3()
                            .child(ColorPicker::new(&self.color).label("Accent"))
                            .child(
                                div()
                                    .size_8()
                                    .rounded_full()
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .when_some(self.color_value, |this, color| this.bg(color)),
                            ),
                    )
                    .child(ui::value_row(
                        "Color",
                        format!("{:?}", self.color_value.map(|c| c.to_hex())),
                        cx,
                    )),
            )
            .child(self.log.render(cx))
    }
}
