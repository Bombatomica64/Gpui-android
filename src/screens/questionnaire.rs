use gpui::{
    AppContext as _, Context, Entity, IntoElement, ParentElement, Render, Styled, Subscription,
    Window,
};
use gpui_kit::component::{
    button::Button,
    input::InputState,
    questionnaire::{
        Questionnaire, QuestionnaireActions, QuestionnaireChoice, QuestionnaireChoiceDefinition,
        QuestionnaireChoices, QuestionnaireDescription, QuestionnaireError, QuestionnaireEvent,
        QuestionnaireInput, QuestionnaireInputDefinition, QuestionnaireItem,
        QuestionnaireItemDefinition, QuestionnaireNext, QuestionnairePrevious,
        QuestionnaireProgress, QuestionnaireSkip, QuestionnaireState, QuestionnaireSubmission,
        QuestionnaireSubmit, QuestionnaireTitle,
    },
    v_flex,
};

use crate::ui::{self, EventLog, prelude::*};

const ITEMS: &[(&str, &[&str], bool)] = &[
    ("direction", &["delegation", "questions", "both"], true),
    ("tools", &["editor", "terminal", "browser"], true),
    ("handle", &[], true),
    ("summary", &["short", "detailed"], false),
];

pub struct QuestionnaireScreen {
    state: Entity<QuestionnaireState>,
    current: String,
    submission: Option<String>,
    log: EventLog,
    _subscriptions: Vec<Subscription>,
}

fn summarize(submission: &QuestionnaireSubmission) -> String {
    submission
        .items()
        .iter()
        .map(|item| format!("{}={:?}", item.name(), item.answer()))
        .collect::<Vec<_>>()
        .join("; ")
}

impl QuestionnaireScreen {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let direction_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("Something else…"));
        let tools_input = cx.new(|cx| InputState::new(window, cx).placeholder("Other tool"));
        let handle_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("@handle (3+ chars)"));
        let items = vec![
            QuestionnaireItemDefinition::new("direction", "What should we prototype next?")
                .with_required(true)
                .with_description("Choose one direction or write your own.")
                .with_choices([
                    QuestionnaireChoiceDefinition::new("delegation", "Delegation")
                        .with_description("Show how work moves to a specialist.")
                        .with_default_selected(true),
                    QuestionnaireChoiceDefinition::new("questions", "Question prompts"),
                    QuestionnaireChoiceDefinition::new("both", "Both together"),
                ])
                .with_input(QuestionnaireInputDefinition::new(
                    direction_input,
                    "Custom direction",
                )),
            QuestionnaireItemDefinition::new("tools", "Which tools do you use?")
                .with_multiple(true)
                .with_description("Multiple choice; Browser is disabled.")
                .with_choices([
                    QuestionnaireChoiceDefinition::new("editor", "Editor"),
                    QuestionnaireChoiceDefinition::new("terminal", "Terminal"),
                    QuestionnaireChoiceDefinition::new("browser", "Browser").with_disabled(true),
                ])
                .with_input(QuestionnaireInputDefinition::new(tools_input, "Other tool")),
            QuestionnaireItemDefinition::new("handle", "Choose a public handle")
                .with_required(true)
                .with_description("The validator rejects handles shorter than 3 characters.")
                .with_input(QuestionnaireInputDefinition::new(
                    handle_input,
                    "Public handle",
                ))
                .with_validator(|context| {
                    if context
                        .answer()
                        .freeform()
                        .is_some_and(|value| value.as_ref().len() >= 3)
                    {
                        Ok(())
                    } else {
                        Err("Use at least three characters.".into())
                    }
                }),
            QuestionnaireItemDefinition::new("summary", "How should we summarize it?")
                .with_description("Optional: can be skipped.")
                .with_choices([
                    QuestionnaireChoiceDefinition::new("short", "Short"),
                    QuestionnaireChoiceDefinition::new("detailed", "Detailed"),
                ]),
        ];
        let state =
            cx.new(|cx| QuestionnaireState::new(items, cx).expect("valid questionnaire schema"));
        let subscription = cx.subscribe(&state, |this, _, event: &QuestionnaireEvent, cx| {
            match event {
                QuestionnaireEvent::CurrentItemChanged { current, .. } => {
                    this.current = format!("{current:?}");
                    this.log.push(format!("current -> {current:?}"));
                }
                QuestionnaireEvent::AnswerChanged(change) => {
                    this.log.push(format!("answer changed: {}", change.item()));
                }
                QuestionnaireEvent::Completed(submission) => {
                    this.log.push("completed");
                    this.submission = Some(summarize(submission));
                }
                QuestionnaireEvent::Submit(submission) => {
                    this.log.push("submit");
                    this.submission = Some(summarize(submission));
                }
                _ => this.log.push("other questionnaire event"),
            }
            cx.notify();
        });
        Self {
            state,
            current: "Some(\"direction\")".into(),
            submission: None,
            log: EventLog::default(),
            _subscriptions: vec![subscription],
        }
    }
}

#[gpui_hot::hot]
impl Render for QuestionnaireScreen {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = &self.state;
        let mut questionnaire = Questionnaire::new(state).child(QuestionnaireProgress::new(state));
        for (name, choices, has_input) in ITEMS {
            let mut parts = QuestionnaireChoices::new(state, *name);
            for value in *choices {
                parts = parts.child(QuestionnaireChoice::new(state, *name, *value));
            }
            if *has_input {
                parts = parts.child(QuestionnaireInput::new(state, *name));
            }
            questionnaire = questionnaire.child(
                QuestionnaireItem::new(state, *name)
                    .child(QuestionnaireTitle::new(state, *name))
                    .child(QuestionnaireDescription::new(state, *name))
                    .child(parts)
                    .child(QuestionnaireError::new(state, *name)),
            );
        }
        let questionnaire = questionnaire.child(
            QuestionnaireActions::new(state)
                .child(QuestionnairePrevious::new(state))
                .child(QuestionnaireSkip::new(state))
                .child(QuestionnaireNext::new(state))
                .child(QuestionnaireSubmit::new(state)),
        );

        v_flex()
            .gap_3()
            .child(ui::section("Questionnaire (new in 0.7.0)", cx).child(questionnaire))
            .child(
                ui::section("State", cx)
                    .child(ui::value_row("Current item", self.current.clone(), cx))
                    .child(ui::value_row(
                        "Submission",
                        self.submission.clone().unwrap_or_else(|| "none yet".into()),
                        cx,
                    ))
                    .child(
                        Button::new("reset-questionnaire")
                            .small()
                            .outline()
                            .label("Reset")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.state.update(cx, |state, cx| state.reset(window, cx));
                                this.submission = None;
                                this.log.push("reset");
                                cx.notify();
                            })),
                    )
                    .child(ui::hint(
                        "Next is blocked until required items are answered; the handle \
                         validator shows an error below the field.",
                        cx,
                    )),
            )
            .child(self.log.render(cx))
    }
}
