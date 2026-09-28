use gpui::{
    AppContext as _, Context, Entity, IntoElement, ParentElement, Render, Styled, Subscription,
    Window, div, px,
};
use gpui_kit::component::{
    IconName, Size,
    checkbox::Checkbox,
    h_flex,
    radio::{Radio, RadioGroup},
    rating::Rating,
    slider::{Slider, SliderEvent, SliderState},
    stepper::{Stepper, StepperItem},
    switch::Switch,
    v_flex,
};

use crate::ui::{self, EventLog, prelude::*};

pub struct ChoiceScreen {
    checks: [bool; 3],
    radio: Option<usize>,
    radio_h: Option<usize>,
    switch: bool,
    switch_small: bool,
    slider: Entity<SliderState>,
    range: Entity<SliderState>,
    vertical: Entity<SliderState>,
    slider_value: String,
    range_value: String,
    vertical_value: String,
    rating: usize,
    step: usize,
    log: EventLog,
    _subscriptions: Vec<Subscription>,
}

impl ChoiceScreen {
    pub fn new(_: &mut Window, cx: &mut Context<Self>) -> Self {
        let slider = cx.new(|_| SliderState::new().min(0.).max(100.).default_value(40.).step(1.));
        let range = cx.new(|_| SliderState::new().min(0.).max(1000.).default_value(200.0..700.0).step(10.));
        let vertical = cx.new(|_| SliderState::new().min(0.).max(10.).default_value(3.).step(1.));
        let subscriptions = vec![
            cx.subscribe(&slider, |this, _, event: &SliderEvent, cx| {
                this.on_slider("slider", event, cx)
            }),
            cx.subscribe(&range, |this, _, event: &SliderEvent, cx| this.on_slider("range", event, cx)),
            cx.subscribe(&vertical, |this, _, event: &SliderEvent, cx| {
                this.on_slider("vertical", event, cx)
            }),
        ];
        Self {
            checks: [true, false, false],
            radio: Some(0),
            radio_h: None,
            switch: true,
            switch_small: false,
            slider,
            range,
            vertical,
            slider_value: "40".into(),
            range_value: "200..700".into(),
            vertical_value: "3".into(),
            rating: 3,
            step: 1,
            log: EventLog::default(),
            _subscriptions: subscriptions,
        }
    }

    fn on_slider(&mut self, name: &str, event: &SliderEvent, cx: &mut Context<Self>) {
        let (kind, value) = match event {
            SliderEvent::Change(value) => ("change", value),
            SliderEvent::Release(value) => ("release", value),
        };
        let text = if value.is_single() {
            format!("{:.0}", value.start())
        } else {
            format!("{:.0}..{:.0}", value.start(), value.end())
        };
        match name {
            "slider" => self.slider_value = text.clone(),
            "range" => self.range_value = text.clone(),
            _ => self.vertical_value = text.clone(),
        }
        // Change fires continuously while dragging; log only releases to keep the log readable.
        if kind == "release" {
            self.log.push(format!("{name} released at {text}"));
        }
        cx.notify();
    }
}

impl Render for ChoiceScreen {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let checkbox = |this: &Self, ix: usize, label: &'static str, cx: &mut Context<Self>| {
            Checkbox::new(("check", ix))
                .label(label)
                .checked(this.checks[ix])
                .on_change(cx.listener(move |this, checked: &bool, _, cx| {
                    this.checks[ix] = *checked;
                    this.log.push(format!("checkbox '{label}' -> {checked}"));
                    cx.notify();
                }))
        };

        v_flex()
            .gap_3()
            .child(
                ui::section("Checkbox", cx)
                    .child(checkbox(self, 0, "Receive product updates", cx))
                    .child(checkbox(self, 1, "Enable a much longer label that has to wrap onto a second line on a narrow phone screen", cx))
                    .child(checkbox(self, 2, "Small size", cx).small())
                    .child(Checkbox::new("check-disabled").label("Disabled (checked)").checked(true).disabled(true))
                    .child(ui::value_row("Values", format!("{:?}", self.checks), cx)),
            )
            .child(
                ui::section("RadioGroup", cx)
                    .child(
                        RadioGroup::vertical("radio-v")
                            .children(["Standard shipping", "Express", "Pick up in store"])
                            .selected_index(self.radio)
                            .on_change(cx.listener(|this, ix: &usize, _, cx| {
                                this.radio = Some(*ix);
                                this.log.push(format!("radio -> {ix}"));
                                cx.notify();
                            })),
                    )
                    .child(
                        RadioGroup::horizontal("radio-h")
                            .child(Radio::new("r-s").label("S"))
                            .child(Radio::new("r-m").label("M"))
                            .child(Radio::new("r-l").label("L"))
                            .child(Radio::new("r-xl").label("XL").disabled(true))
                            .selected_index(self.radio_h)
                            .on_change(cx.listener(|this, ix: &usize, _, cx| {
                                this.radio_h = Some(*ix);
                                this.log.push(format!("size radio -> {ix}"));
                                cx.notify();
                            })),
                    )
                    .child(ui::value_row(
                        "Selected",
                        format!("vertical={:?} horizontal={:?} (XL disabled)", self.radio, self.radio_h),
                        cx,
                    )),
            )
            .child(
                ui::section("Switch", cx)
                    .child(
                        Switch::new("switch")
                            .label("Notifications")
                            .checked(self.switch)
                            .on_change(cx.listener(|this, checked: &bool, _, cx| {
                                this.switch = *checked;
                                this.log.push(format!("switch -> {checked}"));
                                cx.notify();
                            })),
                    )
                    .child(
                        Switch::new("switch-small")
                            .small()
                            .label("Small switch")
                            .checked(self.switch_small)
                            .on_change(cx.listener(|this, checked: &bool, _, cx| {
                                this.switch_small = *checked;
                                this.log.push(format!("small switch -> {checked}"));
                                cx.notify();
                            })),
                    )
                    .child(Switch::new("switch-disabled").label("Disabled").checked(true).disabled(true))
                    .child(ui::value_row(
                        "Value",
                        format!("{} / small {}", self.switch, self.switch_small),
                        cx,
                    )),
            )
            .child(
                ui::section("Slider", cx)
                    .child(Slider::new(&self.slider))
                    .child(ui::value_row("Single (0–100, step 1)", self.slider_value.clone(), cx))
                    .child(Slider::new(&self.range))
                    .child(ui::value_row("Range (0–1000, step 10)", self.range_value.clone(), cx))
                    .child(
                        h_flex()
                            .gap_4()
                            .child(div().h(px(140.)).child(Slider::new(&self.vertical).vertical()))
                            .child(ui::value_row("Vertical (0–10)", self.vertical_value.clone(), cx)),
                    )
                    .child(Slider::new(&self.slider).disabled(true))
                    .child(ui::hint(
                        "Drag thumbs horizontally; a vertical page scroll starting on a slider \
                         should still scroll the page.",
                        cx,
                    )),
            )
            .child(
                ui::section("Rating", cx)
                    .child(
                        Rating::new("rating")
                            .value(self.rating)
                            .max(5)
                            .with_size(Size::Large)
                            .on_click(cx.listener(|this, value: &usize, _, cx| {
                                this.rating = *value;
                                this.log.push(format!("rating -> {value}"));
                                cx.notify();
                            })),
                    )
                    .child(Rating::new("rating-ro").value(4).disabled(true))
                    .child(ui::value_row("Value", format!("{} / 5", self.rating), cx)),
            )
            .child(
                ui::section("Stepper", cx)
                    .child(
                        Stepper::new("stepper")
                            .selected_index(self.step)
                            .items([
                                StepperItem::new().icon(IconName::User).child("Account"),
                                StepperItem::new().icon(IconName::Inbox).child("Shipping"),
                                StepperItem::new().icon(IconName::Check).child("Confirm"),
                            ])
                            .on_click(cx.listener(|this, step: &usize, _, cx| {
                                this.step = *step;
                                this.log.push(format!("stepper -> {step}"));
                                cx.notify();
                            })),
                    )
                    .child(
                        Stepper::new("stepper-v")
                            .vertical()
                            .selected_index(self.step)
                            .items([
                                StepperItem::new().child("Step one"),
                                StepperItem::new().child("Step two"),
                                StepperItem::new().child("Step three"),
                            ])
                            .on_click(cx.listener(|this, step: &usize, _, cx| {
                                this.step = *step;
                                this.log.push(format!("vertical stepper -> {step}"));
                                cx.notify();
                            })),
                    )
                    .child(ui::value_row("Step", self.step.to_string(), cx)),
            )
            .child(self.log.render(cx))
    }
}
