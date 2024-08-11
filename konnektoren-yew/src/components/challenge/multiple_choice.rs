use super::{ChallengeActions, ChallengeActionsComponent, OptionsComponent, QuestionComponent};
use crate::components::challenge::ChallengeEvent;
use crate::components::challenge::MultipleChoiceResultComponent;
use crate::components::ProgressBar;
use crate::prelude::ReadText;
use konnektoren_core::challenges::{
    ChallengeInput, ChallengeResult, MultipleChoice, MultipleChoiceOption,
};
use yew::prelude::*;

#[derive(Properties, PartialEq, Default)]
pub struct MultipleChoiceComponentProps {
    pub challenge: MultipleChoice,
    #[prop_or_default]
    pub on_finish: Option<Callback<ChallengeResult>>,
    #[prop_or_default]
    pub on_event: Option<Callback<ChallengeEvent>>,
}

pub fn is_correct(
    challenge: &MultipleChoice,
    challenge_result: &ChallengeResult,
    index: usize,
) -> bool {
    let question = challenge.questions.get(index);
    let result = match challenge_result {
        ChallengeResult::MultipleChoice(ref mc) => mc.get(index),
        _ => None,
    };
    match (question, result) {
        (Some(question), Some(result)) => question.option == result.id,
        _ => false,
    }
}

pub fn create_handle_action(
    task_index: UseStateHandle<usize>,
    show_help: UseStateHandle<bool>,
    total_tasks: usize,
    on_event: Option<Callback<ChallengeEvent>>,
) -> Callback<ChallengeActions> {
    Callback::from(move |action: ChallengeActions| match action {
        ChallengeActions::Next => {
            if *task_index < total_tasks - 1 {
                let next_task_index = *task_index + 1;
                task_index.set(next_task_index);

                if let Some(on_event) = on_event.as_ref() {
                    on_event.emit(ChallengeEvent::NextTask(next_task_index));
                }
            }
        }
        ChallengeActions::Previous => {
            if *task_index > 0 {
                let previous_task_index = *task_index - 1;
                task_index.set(previous_task_index);
                if let Some(on_event) = on_event.as_ref() {
                    on_event.emit(ChallengeEvent::PreviousTask(previous_task_index));
                }
            }
        }
        ChallengeActions::Help => {
            show_help.set(!*show_help);
        }
    })
}

pub fn create_handle_option_selection(
    task_index: UseStateHandle<usize>,
    challenge: MultipleChoice,
    challenge_result: UseStateHandle<ChallengeResult>,
    total_tasks: usize,
    on_finish: Option<Callback<ChallengeResult>>,
    on_event: Option<Callback<ChallengeEvent>>,
) -> Callback<MultipleChoiceOption> {
    Callback::from(move |option: MultipleChoiceOption| {
        let mut challenge_result_update = (*challenge_result).clone();
        challenge_result_update
            .add_input(ChallengeInput::MultipleChoice(option.clone()))
            .unwrap();
        challenge_result.set(challenge_result_update.clone());

        if let Some(on_event) = on_event.as_ref() {
            if is_correct(&challenge, &challenge_result_update, *task_index) {
                on_event.emit(ChallengeEvent::SolvedCorrect(*task_index));
            } else {
                on_event.emit(ChallengeEvent::SolvedIncorrect(*task_index));
            }
        }

        if *task_index < total_tasks - 1 {
            let next_task_index = *task_index + 1;
            task_index.set(next_task_index);

            if let Some(on_event) = on_event.as_ref() {
                on_event.emit(ChallengeEvent::NextTask(next_task_index));
            }
        } else {
            if let Some(on_finish) = on_finish.as_ref() {
                on_finish.emit(challenge_result_update.clone());
            }
            if let Some(on_event) = on_event.as_ref() {
                on_event.emit(ChallengeEvent::Finish(challenge_result_update.clone()));
            }
        }
    })
}

#[function_component(MultipleChoiceComponent)]
pub fn multiple_choice_component(props: &MultipleChoiceComponentProps) -> Html {
    let task_index = use_state(|| 0);
    let challenge_result = use_state(ChallengeResult::default);
    let show_help = use_state(|| false);

    if *task_index >= props.challenge.questions.len() {
        return html! {};
    }

    let handle_action = create_handle_action(
        task_index.clone(),
        show_help.clone(),
        props.challenge.questions.len(),
        props.on_event.clone(),
    );

    let handle_option_selection = create_handle_option_selection(
        task_index.clone(),
        props.challenge.clone(),
        challenge_result.clone(),
        props.challenge.questions.len(),
        props.on_finish.clone(),
        props.on_event.clone(),
    );

    html! {
        <div class="multiple-choice">
            <ProgressBar
                value={*task_index}
                max={props.challenge.questions.len()}
                label={format!("Question {} of {}", *task_index + 1, props.challenge.questions.len())}
            />
            <QuestionComponent
                question={props.challenge.questions[*task_index].clone()}
                help={*show_help}
            />
            <OptionsComponent
                options={props.challenge.options.clone()}
                on_select={handle_option_selection}
            />
            <ChallengeActionsComponent on_action={handle_action} />
            <MultipleChoiceResultComponent
                challenge={props.challenge.clone()}
                challenge_result={(*challenge_result).clone()}
            />
            <ReadText text={props.challenge.questions[*task_index].help.clone()} lang="de-DE" />
        </div>
    }
}
