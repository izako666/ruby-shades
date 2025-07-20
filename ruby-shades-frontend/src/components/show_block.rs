use yew::prelude::*;
use yew_router::hooks::use_navigator;

use crate::{
    DirectoryContext, Route, backend_handler::TvShowMetadata,
    components::loading_indicator::LoadingIndicator, pages::tmdb_image_url,
};

#[derive(Properties, PartialEq)]
pub struct ShowBlockProps {
    pub show_metadata: TvShowMetadata,
    pub path: String,
}

#[function_component(ShowBlock)]
pub fn show_block(props: &ShowBlockProps) -> Html {
    let directory = use_context::<DirectoryContext>().expect("no ctx found");
    let navigator = use_navigator().unwrap();
    if let None = &*directory.0 {
        return html!(
                    <>
                    <div style="display: flex; justify-content: center; align-items: center; height: 100vh;">
                        <LoadingIndicator />
                    </div>
                </>);
    }
    let dir_obj = directory.0.as_ref().unwrap().clone();

    let poster_url = tmdb_image_url(&props.show_metadata.poster);
    let base_path = dir_obj.path.clone();
    let current_path = props.path.clone();
    let handle_click = {
        let navigator = navigator.clone();
        let base_path = base_path.clone();
        let current_path = current_path.clone();
        Callback::from(move |e| {
            let relative = current_path
                .strip_prefix(&base_path)
                .unwrap_or(&current_path)
                .trim_start_matches('/');

            navigator.push(&Route::Home {
                path: relative.to_string(),
            });
        })
    };

    let name = props.show_metadata.name.clone();
    html! {
        <>
            <div class="show-block" onclick={handle_click}>
                <img class="show-poster" src={poster_url} alt={name.clone()} />
                <h3 class="show-name">{&name }</h3>
            </div>

            <style>
                {r#"
                    .show-block {
                        display: flex;
                        flex-direction: column;
                        justify-content: center;
                        align-items: center;
                        text-align: center;
                        cursor: pointer;
                        padding: 1rem;
                        border-radius: 8px;
                        transition: transform 0.3s ease, box-shadow 0.3s ease;
                    }

                    .show-block:hover {
                        transform: scale(1.05);
                        box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
                    }

                    .show-poster {
                        width: 100%;
                        max-width: 300px;
                        border-radius: 8px;
                        margin-bottom: 1rem;
                    }

                    .show-name {
                        font-size: 1.25rem;
                        font-weight: bold;
                        color: var(--text);
                    }
                "#}
            </style>
        </>
    }
}
