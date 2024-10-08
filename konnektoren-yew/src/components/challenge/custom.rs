use crate::components::ChallengeEvent;
use crate::i18n::{I18nLoader, I18nYmlLoader, SelectedLanguage};
use gloo::net::http::Request;
use konnektoren_core::challenges::{ChallengeResult, Custom, KonnektorenJs};
use wasm_bindgen::JsValue;
use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct CustomComponentProps {
    pub challenge: Custom,
    #[prop_or_default]
    pub on_finish: Option<Callback<ChallengeResult>>,
    #[prop_or_default]
    pub on_event: Option<Callback<ChallengeEvent>>,
}

#[function_component(CustomComponent)]
pub fn custom_component(props: &CustomComponentProps) -> Html {
    // State hooks for storing content and loading status
    let html_content = use_state(|| "".to_string());
    let css_content = use_state(|| "".to_string());
    let js_content = use_state(|| "".to_string());
    let i18n_content = use_state(|| "".to_string());
    let loading = use_state(|| true);

    // Initialize KonnektorenJs instance
    let konnektoren_js = use_state(|| KonnektorenJs::new());

    // Effect to fetch content when the challenge changes
    {
        let html_content = html_content.clone();
        let css_content = css_content.clone();
        let js_content = js_content.clone();
        let i18n_content = i18n_content.clone();
        let challenge = props.challenge.clone();
        let loading = loading.clone();

        use_effect_with(challenge.clone(), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                fetch_content(&challenge.html, html_content).await;
                fetch_content(&challenge.css, css_content).await;
                fetch_content(&challenge.js, js_content).await;
                if let Some(challenge_i18n) = &challenge.i18n {
                    fetch_content(challenge_i18n, i18n_content).await;
                }
                loading.set(false);
            });
            || ()
        });
    }

    // Effect to set up the sendEvent callback once on mount
    {
        let konnektoren_js = konnektoren_js.clone();
        let on_event = props.on_event.clone();

        use_effect(move || {
            let on_event = on_event.clone();
            konnektoren_js.expose_send_event(move |event: JsValue| {
                if let Some(on_event_callback) = &on_event {
                    let challenge_event: ChallengeEvent = event.into();
                    on_event_callback.emit(challenge_event);
                }
            });
            || ()
        });
    }

    // Effect to process the loaded content after loading is complete
    {
        let konnektoren_js = konnektoren_js.clone();
        let challenge = props.challenge.clone();
        let js_code = (*js_content).clone();
        let i18n_content = (*i18n_content).clone();
        let loading = *loading;

        use_effect_with(
            (loading, challenge, js_code, i18n_content),
            move |(loading, challenge, js_code, i18n_content)| {
                if !*loading {
                    // Set challenge data
                    konnektoren_js.set_challenge_data(challenge.clone());

                    // Set i18n data if available
                    if !(i18n_content).is_empty() {
                        let language = SelectedLanguage::default().get();
                        let loader = I18nYmlLoader::new(&i18n_content);
                        let translations = loader.get(&language).unwrap_or_default();
                        konnektoren_js.set_i18n_data(translations);
                    }

                    // Execute JS code
                    konnektoren_js.execute_js(&js_code);
                }
            },
        );
    }

    // Render the HTML content
    let parsed_html = Html::from_html_unchecked(AttrValue::from((*html_content).clone()));

    html! {
        <div class="custom-challenge">
            <style>
                {(*css_content).clone()}
            </style>
            {parsed_html}
        </div>
    }
}

// Function to fetch content and update the corresponding state
pub async fn fetch_content(path: &str, handle: UseStateHandle<String>) {
    match fetch_file(path).await {
        Ok(content) => handle.set(content),
        Err(err) => log::error!("Failed to fetch the file content of {}: {}", path, err),
    }
}

// Function to fetch a file's content
pub async fn fetch_file(path: &str) -> Result<String, String> {
    let header_value = match path.split('.').last() {
        Some("js") => "application/javascript",
        Some("css") => "text/css",
        Some("html") => "text/html",
        Some("yml") => "text/yaml",
        _ => "text/plain",
    };

    let response = Request::get(path)
        .header("Accept", header_value)
        .send()
        .await
        .map_err(|_| format!("Failed to fetch the file {}", path))?;
    if response.status() == 200 {
        response
            .text()
            .await
            .map_err(|_| format!("Failed to read the file content of {}", path))
    } else {
        Err(format!("File not found: {}", path))
    }
}
