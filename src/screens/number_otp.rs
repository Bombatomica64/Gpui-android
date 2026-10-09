use gpui::{
    AppContext as _, Context, Entity, IntoElement, ParentElement, Styled, Subscription, Window,
};
use gpui_kit::component::{
    input::{
        InputEvent, InputState, MaskPattern, NumberInput, NumberInputEvent, OtpEvent, OtpInput,
        OtpState, StepAction,
    },
    v_flex,
};

use crate::ui::{self, EventLog, prelude::*};

pub struct NumberOtpScreen {
    clamped: Entity<InputState>,
    manual: Entity<InputState>,
    decimal: Entity<InputState>,
    disabled: Entity<InputState>,
    otp: Entity<OtpState>,
    otp_masked: Entity<OtpState>,
    manual_value: i64,
    otp_value: String,
    otp_complete: bool,
    log: EventLog,
    _subscriptions: Vec<Subscription>,
}

impl NumberOtpScreen {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let clamped = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value("5")
                .min(0.)
                .max(10.)
                .step(1.)
        });
        let manual = cx.new(|cx| InputState::new(window, cx).default_value("100"));
        manual.update(cx, |state, cx| state.set_step(None, window, cx));
        let decimal = cx.new(|cx| {
            InputState::new(window, cx)
                .mask_pattern(MaskPattern::Number {
                    separator: Some(','),
                    fraction: Some(2),
                })
                .default_value("1234.5")
                .step(0.25)
        });
        let disabled = cx.new(|cx| InputState::new(window, cx).default_value("42"));
        let otp = cx.new(|cx| OtpState::new(6, window, cx));
        let otp_masked = cx.new(|cx| OtpState::new(4, window, cx).masked(true));

        let mut subscriptions = vec![cx.subscribe_in(
            &manual,
            window,
            |this, state, event: &NumberInputEvent, window, cx| {
                let NumberInputEvent::Step(action) = event;
                this.manual_value += match action {
                    StepAction::Increment => 10,
                    StepAction::Decrement => -10,
                };
                let value = this.manual_value.to_string();
                this.log.push(format!("manual step {action:?} -> {value}"));
                state.update(cx, |state, cx| state.set_value(value, window, cx));
                cx.notify();
            },
        )];
        for (name, state) in [
            ("clamped 0–10", &clamped),
            ("manual ±10", &manual),
            ("decimal", &decimal),
        ] {
            subscriptions.push(
                cx.subscribe(state, move |this, state, event: &InputEvent, cx| {
                    if let InputEvent::Change = event {
                        this.log
                            .push(format!("{name}: change -> {}", state.read(cx).value()));
                        cx.notify();
                    }
                }),
            );
        }
        subscriptions.push(cx.subscribe(&otp, |this, state, event: &OtpEvent, cx| {
            this.otp_value = state.read(cx).value().to_string();
            match event {
                OtpEvent::Change => this.otp_complete = false,
                OtpEvent::Complete => {
                    this.otp_complete = true;
                    this.log.push(format!("otp complete: {}", this.otp_value));
                }
                OtpEvent::Focus => this.log.push("otp focus"),
                OtpEvent::Blur => this.log.push("otp blur"),
            }
            cx.notify();
        }));
        subscriptions.push(cx.subscribe(&otp_masked, |this, _, event: &OtpEvent, cx| {
            if let OtpEvent::Complete = event {
                this.log.push("masked PIN complete");
                cx.notify();
            }
        }));

        Self {
            clamped,
            manual,
            decimal,
            disabled,
            otp,
            otp_masked,
            manual_value: 100,
            otp_value: String::new(),
            otp_complete: false,
            log: EventLog::default(),
            _subscriptions: subscriptions,
        }
    }
}

crate::hot_render!(NumberOtpScreen);

impl NumberOtpScreen {
    fn render_view(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_3()
            .child(
                ui::section("NumberInput", cx)
                    .child(NumberInput::new(&self.clamped))
                    .child(ui::value_row("Clamped 0–10, step 1", self.clamped.read(cx).value(), cx))
                    .child(NumberInput::new(&self.manual))
                    .child(ui::value_row(
                        "App-handled steps (±10)",
                        format!("{} (model {})", self.manual.read(cx).value(), self.manual_value),
                        cx,
                    ))
                    .child(NumberInput::new(&self.decimal))
                    .child(ui::value_row("Decimal mask, step 0.25", self.decimal.read(cx).value(), cx))
                    .child(NumberInput::new(&self.disabled).disabled(true))
                    .child(ui::hint(
                        "Tap − / + repeatedly and quickly; typing should open the numeric keyboard.",
                        cx,
                    )),
            )
            .child(
                ui::section("OtpInput", cx)
                    .child(OtpInput::new(&self.otp).groups(2))
                    .child(ui::value_row(
                        "Value / complete",
                        format!("{:?} / {}", self.otp_value, self.otp_complete),
                        cx,
                    ))
                    .child(OtpInput::new(&self.otp_masked).large())
                    .child(ui::hint(
                        "Paste a 6-digit code (e.g. copy 123456) to fill all cells at once.",
                        cx,
                    )),
            )
            .child(self.log.render(cx))
    }
}
