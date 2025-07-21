use crate::{
    Route,
    backend_handler::{listen_for_status_updates, start_watch},
    components::loading_indicator::LoadingIndicator,
    config::read_config,
    video_player::init_hls_player,
};
use wasm_bindgen_futures::spawn_local;
use web_sys::{UrlSearchParams, console, wasm_bindgen::JsValue};
use yew::prelude::*;
use yew_router::hooks::{use_location, use_navigator};

#[function_component(VideoPlayer)]
pub fn video_player() -> Html {
    let location = use_location().expect("no location");
    let navigator = use_navigator().unwrap();
    let raw_query = location.query_str().to_string();
    let query = raw_query.clone();
    let resource = use_memo((), move |_| {
        UrlSearchParams::new_with_str(&query)
            .ok()
            .and_then(|params| params.get("resource"))
    });

    let backend_url = read_config().backend_url;
    let selected_quality = use_state(|| "1080p".to_string());
    let hls_url = use_state(|| None as Option<String>);
    let uuid = use_state(|| None as Option<String>);
    let start_watching = use_state(|| false);
    let raw_query = raw_query.clone();

    {
        let resource = resource.clone();
        let selected_quality = selected_quality.clone();
        let start_watching = start_watching.clone();
        let uuid = uuid.clone();
        use_effect_with(
            (resource.clone(), selected_quality.clone()),
            move |(resource, selected_quality)| {
                if let Some(resource) = &**resource {
                    let resource = resource.clone();
                    let selected_quality = (*selected_quality).clone();
                    let start_watching = start_watching.clone();
                    let uuid = uuid.clone();
                    console::log_1(&JsValue::from_str("starting watch"));
                    spawn_local(async move {
                        if let Ok(result) = start_watch(&resource, Some(&selected_quality)).await {
                            let uuid_str = result.uuid.clone();
                            uuid.set(Some(result.uuid.clone()));
                            match listen_for_status_updates(&result.uuid, move |status| {
                                if status.uuid == uuid_str {
                                    start_watching.set(
                                        !(status.status == "PROCESSING"
                                            || status.status == "FAILURE"),
                                    );
                                }
                            })
                            .await
                            {
                                Ok(_) => {
                                    console::log_1(&JsValue::from_str("connected to websocket"))
                                }
                                Err(_) => console::log_1(&JsValue::from_str(
                                    "failed to connect to websocket",
                                )),
                            }
                        }
                    });
                } else {
                    console::log_1(&JsValue::from_str("resource not found??"));
                    console::log_1(&JsValue::from_str(&raw_query));
                }

                || ()
            },
        );
    }

    {
        let start_watching = *start_watching;
        let uuid = (*uuid).clone();
        let hls_url = hls_url.clone();
        let backend_url = backend_url.clone();

        use_effect_with((start_watching, uuid.clone()), move |(watching, uuid)| {
            if *watching {
                if let Some(uuid) = uuid {
                    let hls = format!("{}/videos/{}/index.m3u8", backend_url, uuid);
                    init_hls_player("video-player", &hls);
                    hls_url.set(Some(hls));
                }
            }
            || ()
        });
    }
    {
        let selected_quality = selected_quality.clone();
        use_effect_with(selected_quality.clone(), move |_| {
            console::log_1(&JsValue::from_str(&format!(
                "Selected quality: {}",
                *selected_quality
            )));
            || ()
        });
    }
    let on_quality_change = {
        let selected_quality = selected_quality.clone();
        let uuid = uuid.clone();
        let start_watching = start_watching.clone();
        Callback::from(move |e: Event| {
            if let Some(input) = e.target_dyn_into::<web_sys::HtmlSelectElement>() {
                selected_quality.set(input.value());
                start_watching.set(false); // reset watching
                uuid.set(None); // clear UUID to show loading state again
            }
        })
    };

    // Loading state before UUID is assigned
    if uuid.is_none() {
        return html!(
            <div style="display: flex; justify-content: center; align-items: center; height: 100vh;">
                <LoadingIndicator />
            </div>
        );
    }
    console::log_1(&JsValue::from_str(&format!(
        "selected quality is: {}",
        *selected_quality
    )));
    html! {
        <div>
            <select onchange={on_quality_change} value={"1080p"}>
                <option value="1080p">{ "1080p" }</option>
                <option value="720p">{ "720p" }</option>
                <option value="480p">{ "480p" }</option>
                <option value="360p">{ "360p" }</option>
            </select>

            <video
                id="video-player"
                controls=true
                style="width: 100%; height: auto; margin-top: 1rem;"
            />
        </div>
    }
}
