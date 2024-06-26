use konnektoren_core::prelude::*;
use yew::prelude::*;

use crate::components::challenge_config::ChallengeConfigComponent;

#[derive(Properties, PartialEq)]
pub struct GamePathComponentProps {
    pub game_path: GamePath,
    #[prop_or_default]
    pub on_challenge_config: Option<Callback<ChallengeConfig>>,
}

#[function_component(GamePathComponent)]
pub fn game_path_component(props: &GamePathComponentProps) -> Html {
    html! {
        <div class="game-path">
            <h1>{&props.game_path.name}</h1>
            <ul>
                {for props.game_path.challenges.iter().map(|challenge| html! {
                    <li>
                    <ChallengeConfigComponent
                        challenge_config={challenge.clone()}
                        on_new={props.on_challenge_config.clone()} />
                    </li>
                })}
            </ul>
        </div>
    }
}
