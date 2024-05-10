use konnektoren_core::prelude::*;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct GameMapComponentProps {
    pub game_path: GamePath,
    pub current_challenge: usize,
    #[prop_or_default]
    pub on_select_challenge: Option<Callback<usize>>,
}

const SCALE: i32 = 10;

fn draw_circle(x: i32, y: i32, color: &str, on_click: Callback<MouseEvent>) -> Html {
    html! {
        <circle
            cx={(x * SCALE).to_string()}
            cy={(-y * SCALE).to_string()}
            r="3"
            fill={color.to_string()}
            onclick={on_click}
        />
    }
}

fn draw_text(x: i32, y: i32, name: &str, on_click: Callback<MouseEvent>) -> Html {
    html! {
        <text
            x={(x * SCALE).to_string()}
            y={(-y * SCALE).to_string()}
            font-size="3"
            text-anchor="middle"
            alignment-baseline="middle"
            onclick={on_click}
        >
            {name}
        </text>
    }
}

fn draw_line(x1: i32, y1: i32, x2: i32, y2: i32) -> Html {
    html! {
        <line
            x1={(x1 * SCALE).to_string()}
            y1={(-y1 * SCALE).to_string()}
            x2={(x2* SCALE).to_string()}
            y2={(-y2* SCALE).to_string()}
            stroke="black"
            stroke-width="2"
        />
    }
}

fn calculate_bounds(challenges: &[(String, i32, i32)]) -> ([i32; 2], [i32; 2]) {
    let x_min = challenges
        .iter()
        .map(|(_, x, _)| *x)
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(0);
    let x_max = challenges
        .iter()
        .map(|(_, x, _)| *x)
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(0);
    let y_min = challenges
        .iter()
        .map(|(_, _, y)| *y)
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(0);
    let y_max = challenges
        .iter()
        .map(|(_, _, y)| *y)
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(0);

    (
        [x_min - SCALE, x_max + 2 * SCALE],
        [2 * y_min - SCALE, y_max + 4 * SCALE],
    )
}

#[function_component(GameMapComponent)]
pub fn game_map_component(props: &GameMapComponentProps) -> Html {
    let bounds = calculate_bounds(
        &props
            .game_path
            .challenges
            .iter()
            .map(|challenge| {
                let (x, y) = challenge.position.unwrap_or((0, 0));
                (challenge.name.clone(), x * SCALE, y * SCALE)
            })
            .collect::<Vec<_>>(),
    );

    html! {
        <div class="game-map">
            <h1>{&props.game_path.name}</h1>
            <svg
                xmlns="http://www.w3.org/2000/svg"
                viewBox={format!("{} {} {} {}", bounds.0[0], bounds.1[0], bounds.0[1], bounds.1[1])}
                class="game-map-svg"
            >
                {for props.game_path.challenges.iter().enumerate().map(|(index, challenge)| {
                    let (x, y) = challenge.position.unwrap_or((0, 0));
                    let next_challenge = props.game_path.challenges.get(index + 1);
                    let color = if props.current_challenge == index { "red" } else { "yellow" };

                    let on_click = {
                        let on_select_challenge = props.on_select_challenge.clone();
                        Callback::from(move |_| {
                            if let Some(ref callback) = on_select_challenge {
                                callback.emit(index);
                            }
                        })
                    };

                    html! {
                        <>
                            {if let Some(next) = next_challenge {
                                let (next_x, next_y) = next.position.unwrap_or((0, 0));
                                draw_line(x, y, next_x, next_y)
                            } else {
                                html!(<></>)
                            }}
                            {draw_circle(x, y, color, on_click.clone())}
                            {draw_text(x, y, &challenge.name, on_click)}
                        </>
                    }
                })}
            </svg>
        </div>
    }
}
