use yew::prelude::*;
use yew_router::hooks::use_navigator;

use crate::{
    DirectoryContext, Route, backend_handler::MovieMetadata,
    components::loading_indicator::LoadingIndicator, pages::tmdb_image_url,
};

#[derive(Properties, PartialEq)]
pub struct MovieBlockProps {
    pub movie_metadata: MovieMetadata,
    pub path: String,
}

#[function_component(MovieBlock)]
pub fn movie_block(props: &MovieBlockProps) -> Html {
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
    let poster_url = tmdb_image_url(&props.movie_metadata.poster);
    let dir_obj = directory.0.as_ref().unwrap().clone();

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

    let name = props.movie_metadata.name.clone();

    html! {
        <>
            <div class="movie-block" onclick={handle_click.clone()}>
                <img class="movie-poster" src={poster_url} alt={name.clone()} />
                <h3 class="movie-name">{ name.clone() }</h3>
            </div>

            <style>
                {r#"
                    .movie-block {
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

                    .movie-block:hover {
                        transform: scale(1.05);
                        box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
                    }

                    .movie-poster {
                        width: 100%;
                        max-width: 300px;
                        border-radius: 8px;
                        margin-bottom: 1rem;
                    }

                    .movie-name {
                        font-size: 1.25rem;
                        font-weight: bold;
                        color: var(--text);
                    }
                "#}
            </style>
        </>
    }
}
