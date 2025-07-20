use crate::{
    backend_handler::{listen_for_status_updates, start_watch},
    config::read_config,
    video_player::init_hls_player,
};
use wasm_bindgen_futures::spawn_local;
use web_sys::{
    console::{self, log},
    wasm_bindgen::JsValue,
};
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct VideoPlayerProps {
    pub resource: String,
}

#[function_component(VideoPlayer)]
pub fn video_player(props: &VideoPlayerProps) -> Html {
    let backend_url = read_config().backend_url;
    let selected_quality = use_state(|| "1080p".to_string());
    let hls_url = use_state(|| None as Option<String>);
    let uuid = use_state(|| None as Option<String>);
    let start_watching = use_state(|| false);
    {
        let resource = props.resource.clone();
        let selected_quality = selected_quality.clone();
        let start_watching = start_watching.clone();
        let uuid = uuid.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                if let Ok(result) = start_watch(&resource, Some(&selected_quality)).await {
                    let uuid_str = result.uuid.clone();
                    uuid.set(Some(result.uuid.clone()));
                    match listen_for_status_updates(&result.uuid, move |status| {
                        if status.uuid == uuid_str {
                            start_watching.set(
                                !(status.status == "PROCESSING" || status.status == "FAILURE"),
                            );
                        }
                    })
                    .await
                    {
                        Ok(_) => console::log_1(&JsValue::from_str("connected to websocket")),
                        Err(_) => {
                            console::log_1(&JsValue::from_str("failed to connect to websocket"))
                        }
                    }
                }
            });

            || ()
        });
    }
    let start_watching = start_watching.clone();
    let uuid = (*uuid).clone().unwrap();
    if *start_watching {
        let hls = format!("{}/{}/index.m3u8", backend_url, uuid);
        init_hls_player("video-player", &hls);
        hls_url.set(Some(hls));
    }
    let on_quality_change = {
        let selected_quality = selected_quality.clone();
        Callback::from(move |e: Event| {
            let input = e.target_dyn_into::<web_sys::HtmlSelectElement>().unwrap();
            selected_quality.set(input.value());
        })
    };

    html! {
        <div>
            <select onchange={on_quality_change}>
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
