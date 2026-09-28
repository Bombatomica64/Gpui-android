use chrono::{Days, NaiveTime};
use gpui::{
    AppContext as _, Context, Entity, IntoElement, ParentElement, Render, Styled, Subscription,
    Window,
};
use gpui_kit::component::{
    button::Button,
    calendar::{Calendar, CalendarEvent, CalendarState, Date},
    date_picker::{DatePicker, DatePickerEvent, DatePickerState, DateRangePreset},
    h_flex,
    time_field::{HourCycle, TimeField, TimeFieldEvent, TimeFieldState, TimePrecision},
    v_flex,
};

use crate::ui::{self, EventLog, prelude::*};

pub struct DateTimeScreen {
    calendar: Entity<CalendarState>,
    single: Entity<DatePickerState>,
    range: Entity<DatePickerState>,
    date_time: Entity<DatePickerState>,
    presets: Entity<DatePickerState>,
    time_24: Entity<TimeFieldState>,
    time_12: Entity<TimeFieldState>,
    time_seconds: Entity<TimeFieldState>,
    /// Separate states for the invalid/disabled demos: popups and focus follow the
    /// last-rendered element of a state.
    disabled_date: Entity<DatePickerState>,
    time_invalid: Entity<TimeFieldState>,
    time_disabled: Entity<TimeFieldState>,
    values: [String; 8],
    log: EventLog,
    _subscriptions: Vec<Subscription>,
}

fn describe_date(date: &Date) -> String {
    match date {
        Date::Single(Some(d)) => d.to_string(),
        Date::Range(Some(start), Some(end)) => format!("{start} → {end}"),
        Date::Range(Some(start), None) => format!("{start} → …"),
        _ => "none".into(),
    }
}

impl DateTimeScreen {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let today = chrono::Local::now().date_naive();
        let calendar = cx.new(|cx| CalendarState::new(window, cx));
        let single = cx.new(|cx| {
            // Weekends disabled to exercise the matcher.
            let mut picker = DatePickerState::new(window, cx).disabled_matcher(vec![0, 6]);
            picker.set_date(today, window, cx);
            picker
        });
        let range = cx.new(|cx| DatePickerState::range(window, cx));
        let date_time = cx.new(|cx| {
            DatePickerState::new(window, cx)
                .time_precision(TimePrecision::Minute)
                .default_time(NaiveTime::from_hms_opt(9, 0, 0).expect("valid time"))
        });
        let presets = cx.new(|cx| DatePickerState::range(window, cx));
        let time_24 = cx.new(|cx| TimeFieldState::new(window, cx));
        let time_12 = cx.new(|cx| TimeFieldState::new(window, cx).hour_cycle(HourCycle::H12));
        let time_seconds = cx.new(|cx| {
            TimeFieldState::new(window, cx)
                .precision(TimePrecision::Second)
                .hour_cycle(HourCycle::H23)
        });
        time_12.update(cx, |state, cx| {
            state.set_time(
                NaiveTime::from_hms_opt(14, 30, 0).expect("valid time"),
                window,
                cx,
            )
        });

        let mut subscriptions =
            vec![
                cx.subscribe(&calendar, |this, _, event: &CalendarEvent, cx| {
                    let CalendarEvent::Selected(date) = event;
                    this.values[0] = describe_date(date);
                    this.log
                        .push(format!("calendar selected {}", this.values[0]));
                    cx.notify();
                }),
            ];
        for (ix, name, state) in [
            (1, "date", &single),
            (2, "range", &range),
            (3, "date+time", &date_time),
            (4, "presets", &presets),
        ] {
            subscriptions.push(
                cx.subscribe(state, move |this, _, event: &DatePickerEvent, cx| {
                    let DatePickerEvent::Change(value) = event;
                    this.values[ix] = if ix == 3 {
                        value
                            .format("%Y-%m-%d %H:%M")
                            .map(|text| text.to_string())
                            .unwrap_or_else(|| "none".into())
                    } else {
                        describe_date(&value.date())
                    };
                    this.log
                        .push(format!("{name} picker -> {}", this.values[ix]));
                    cx.notify();
                }),
            );
        }
        for (ix, name, state) in [
            (5, "24h", &time_24),
            (6, "12h", &time_12),
            (7, "seconds", &time_seconds),
        ] {
            subscriptions.push(
                cx.subscribe(state, move |this, _, event: &TimeFieldEvent, cx| {
                    let TimeFieldEvent::Change(time) = event;
                    this.values[ix] = time.to_string();
                    this.log.push(format!("time {name} -> {time}"));
                    cx.notify();
                }),
            );
        }

        Self {
            calendar,
            single,
            range,
            date_time,
            presets,
            time_24,
            time_12,
            time_seconds,
            disabled_date: cx.new(|cx| DatePickerState::new(window, cx)),
            time_invalid: cx.new(|cx| TimeFieldState::new(window, cx)),
            time_disabled: cx.new(|cx| TimeFieldState::new(window, cx)),
            values: [
                "none".into(),
                today.to_string(),
                "none".into(),
                "none".into(),
                "none".into(),
                "00:00:00".into(),
                "14:30:00".into(),
                "00:00:00".into(),
            ],
            log: EventLog::default(),
            _subscriptions: subscriptions,
        }
    }
}

impl Render for DateTimeScreen {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let today = chrono::Local::now().date_naive();
        let presets = vec![
            DateRangePreset::range("Last 7 days", today - Days::new(6), today),
            DateRangePreset::range("Last 30 days", today - Days::new(29), today),
            DateRangePreset::range("Next week", today + Days::new(1), today + Days::new(7)),
        ];
        v_flex()
            .gap_3()
            .child(
                ui::section("Calendar", cx)
                    .child(Calendar::new(&self.calendar))
                    .child(ui::value_row("Selected", self.values[0].clone(), cx)),
            )
            .child(
                ui::section("DatePicker", cx)
                    .child(
                        DatePicker::new(&self.single)
                            .placeholder("Pick a weekday")
                            .cleanable(true),
                    )
                    .child(ui::value_row(
                        "Date (weekends disabled)",
                        self.values[1].clone(),
                        cx,
                    ))
                    .child(DatePicker::new(&self.range).placeholder("Pick a range"))
                    .child(ui::value_row("Range", self.values[2].clone(), cx))
                    .child(DatePicker::new(&self.date_time).placeholder("Date and time"))
                    .child(ui::value_row(
                        "Date + time (minute)",
                        self.values[3].clone(),
                        cx,
                    ))
                    .child(
                        DatePicker::new(&self.presets)
                            .presets(presets)
                            .placeholder("With presets"),
                    )
                    .child(ui::value_row("Preset range", self.values[4].clone(), cx))
                    .child(DatePicker::new(&self.disabled_date).disabled(true))
                    .child(ui::hint(
                        "Popups must stay inside the 411 pt wide viewport; tap outside or press \
                         back to close.",
                        cx,
                    )),
            )
            .child(
                ui::section("TimeField (new in 0.7.0)", cx)
                    .child(TimeField::new(&self.time_24))
                    .child(ui::value_row("24-hour, minute", self.values[5].clone(), cx))
                    .child(TimeField::new(&self.time_12).large())
                    .child(ui::value_row("12-hour, minute", self.values[6].clone(), cx))
                    .child(TimeField::new(&self.time_seconds).small())
                    .child(ui::value_row("24-hour, second", self.values[7].clone(), cx))
                    .child(TimeField::new(&self.time_invalid).invalid(true))
                    .child(TimeField::new(&self.time_disabled).disabled(true))
                    .child(
                        h_flex().gap_1().child(
                            Button::new("set-time")
                                .small()
                                .outline()
                                .label("Set 09:30 (no event)")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    let time =
                                        NaiveTime::from_hms_opt(9, 30, 0).expect("valid time");
                                    for state in [&this.time_24, &this.time_12, &this.time_seconds]
                                    {
                                        state.update(cx, |state, cx| {
                                            state.set_time(time, window, cx)
                                        });
                                    }
                                    this.log.push("set_time(09:30) on all fields");
                                    cx.notify();
                                })),
                        ),
                    )
                    .child(ui::hint(
                        "TimeField is keyboard-driven: tap a segment, then type digits on the \
                         soft keyboard or use hardware arrows. 'a'/'p' switch AM/PM.",
                        cx,
                    )),
            )
            .child(self.log.render(cx))
    }
}
