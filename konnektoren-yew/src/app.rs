use crate::components::{
    challenge::ChallengeComponent, game_map::GameMapComponent, game_path::GamePathComponent,
    ChallengeConfigComponent, ChallengeInfoComponent, MusicComponent, ProgressBar,
    TranslateComponent,
};

#[cfg(feature = "storage")]
use crate::components::profile::{ProfileConfigComponent, ProfilePointsComponent};

use crate::components::challenge::multiple_choice::MultipleChoiceComponentProps;
use crate::components::challenge::{
    MultipleChoiceCircleComponent, MultipleChoiceComponent, SortTableComponent,
};
use crate::components::game_map::{ChallengeIndex, Coordinate};
#[cfg(feature = "certificates")]
use crate::components::CertificateComponent;
#[cfg(feature = "effects")]
use crate::effects::BlinkAnimation;
use crate::prelude::{ChallengeActionsComponent, InformativeComponent, InformativeMarkdownComponent, OptionsComponent, QuestionComponent, ReadText, SelectLanguage};

use crate::i18n::{I18nConfig, I18nProvider};
use konnektoren_core::prelude::*;
use log;
use yew::prelude::*;
#[cfg(feature = "yew-preview")]
use yew_preview::{create_component_item, prelude::*};

#[function_component]
pub fn Example() -> Html {
    let game = Game::default();
    let challenge: UseStateHandle<Option<Challenge>> = use_state(|| None);

    let new_challenge_cb = {
        let game = game.clone();
        let challenge = challenge.clone();
        Callback::from(move |challenge_config: ChallengeConfig| {
            match game.create_challenge(&challenge_config.id) {
                Ok(c) => challenge.set(Some(c)),
                Err(err) => log::error!("Error creating challenge: {:?}", err),
            }
        })
    };

    let on_map_challenge_cb = {
        let game = game.clone();
        let challenge = challenge.clone();
        Callback::from(
            move |(challenge_index, coords): (Option<ChallengeIndex>, Coordinate)| {
                let (x, y) = coords;
                if let Some(challenge_index) = challenge_index {
                    log::info!("Challenge index: {}, x: {}, y: {}", challenge_index, x, y);
                    if let Some(challenge_config) = game.game_path.challenges.get(challenge_index) {
                        match game.create_challenge(&challenge_config.id) {
                            Ok(c) => challenge.set(Some(c)),
                            Err(_) => log::error!("Challenge creation failed"),
                        }
                    }
                } else {
                    log::error!("Challenge not found");
                }
            },
        )
    };

    let profile_config_component = {
        #[cfg(feature = "storage")]
        html! {<ProfileConfigComponent />}
        #[cfg(not(feature = "storage"))]
        html! {<></>}
    };

    let profile_points_component = {
        #[cfg(feature = "storage")]
        html! {<ProfilePointsComponent />}
        #[cfg(not(feature = "storage"))]
        html! {<></>}
    };
    html! {
        <div>
            {profile_config_component}
            {profile_points_component}
            <GamePathComponent game_path={game.game_path.clone()} on_challenge_config={new_challenge_cb} />
            {
                if let Some(ref challenge) = *challenge {
                    html! { <ChallengeComponent challenge={challenge.clone()} /> }
                } else {
                    html! {}
                }
            }
            <GameMapComponent
                game_path={game.game_path.clone()}
                current_challenge={0}
                on_select_challenge={on_map_challenge_cb}
            />
        </div>
    }
}

#[function_component]
pub fn App() -> Html {
    let game = Game::default();
    let default_challenge = game.create_challenge("konnektoren-1").unwrap();
    let default_multiple_choice: MultipleChoice = match &default_challenge.challenge_type {
        ChallengeType::MultipleChoice(multiple_choice) => multiple_choice.clone(),
        _ => unreachable!(),
    };

    let i18n_config = I18nConfig::default();

    #[cfg(feature = "yew-preview")]
    let component_list: ComponentList = vec![
        BlinkAnimation::preview(),
        create_component_item!(
            "MultipleChoiceComponent",
            MultipleChoiceComponent,
            vec![(
                "default",
                MultipleChoiceComponentProps {
                    challenge: default_multiple_choice.clone(),
                    ..Default::default()
                }
            )]
        ),
        create_component_item!(
            "MultipleChoiceCircleComponent",
            MultipleChoiceCircleComponent,
            vec![(
                "default",
                MultipleChoiceComponentProps {
                    challenge: default_multiple_choice,
                    ..Default::default()
                }
            )]
        ),
        SortTableComponent::preview(),
        ChallengeComponent::preview(),
        ProfileConfigComponent::preview(),
        ProfilePointsComponent::preview(),
        ChallengeActionsComponent::preview(),
        ChallengeConfigComponent::preview(),
        ChallengeInfoComponent::preview(),
        InformativeComponent::preview(),
        InformativeMarkdownComponent::preview(),
        GameMapComponent::preview(),
        GamePathComponent::preview(),
        MusicComponent::preview(),
        OptionsComponent::preview(),
        ProgressBar::preview(),
        QuestionComponent::preview(),
        TranslateComponent::preview(),
        CertificateComponent::preview(),
        ReadText::preview(),
        SelectLanguage::preview(),
        create_component_item!("Example", Example, vec![("default", ())]),
    ];

    #[cfg(feature = "yew-preview")]
    html! {
        <I18nProvider config={i18n_config}>
            <PreviewPage components={component_list} />
        </I18nProvider>
    }
    #[cfg(not(feature = "yew-preview"))]
    html! {
        <Example />
    }
}
