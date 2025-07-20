use yew::{Callback, Html, Properties, function_component, html, use_context, use_state};

use crate::{
    MetadataContext,
    backend_handler::{MovieMetadata, PathObject},
    components::loading_indicator::LoadingIndicator,
    pages::tmdb_image_url,
};

#[derive(Properties, PartialEq)]
pub struct MovieProps {
    pub metadata: MovieMetadata,
    pub path_object: PathObject,
}

#[function_component(MoviePage)]
pub fn movie(props: &MovieProps) -> Html {
    let metadata_context =
        use_context::<MetadataContext>().expect("metadata context should exist at this stage");

    if metadata_context.0.is_none() {
        return html!(
            <>
            <div style="display: flex; justify-content: center; align-items: center; height: 100vh;">
                <LoadingIndicator />
            </div>
        </>);
    }

    let is_hovered = use_state(|| false);

    let onmouseover = {
        let is_hovered = is_hovered.clone();
        Callback::from(move |_| is_hovered.set(true))
    };

    let onmouseout = {
        let is_hovered = is_hovered.clone();
        Callback::from(move |_| is_hovered.set(false))
    };

    let button_style = format!(
        "background-color: {}; color: var(--text); border: none; padding: 1rem 2rem; \
     border-radius: 10px; font-size: 1rem; font-weight: bold; cursor: pointer; \
     transition: background-color 0.3s ease;",
        if *is_hovered {
            "var(--secondary)"
        } else {
            "var(--primary)"
        }
    );
    let backdrop_url = tmdb_image_url(&props.metadata.backdrop);
    html! {
        <div style="width: 100%; overflow-x: hidden; background-color: var(--background); color: var(--text);">

            // Backdrop image container
            <div style="position: relative; width: 100%; max-height: 1000px; overflow: hidden;">
                <img
                    src={backdrop_url}
                    alt="Backdrop"
                    style="width: 100%; height: auto; display: block; object-fit: cover;"
                />
            </div>

            // Main content wrapper
            <div style="position: relative; display: flex; flex-direction: row;  margin: -150px auto 0 auto; padding: 2rem; gap: 2rem; z-index: 2;">

                // Poster on the left
                <img
                    src={tmdb_image_url(&props.metadata.poster)}
                    alt="Poster"
                    style="width: 250px; height: auto; border-radius: 12px; box-shadow: 0 8px 24px rgba(0, 0, 0, 0.6);"
                />

                // Middle content with title and description
                <div style="flex-grow: 1; display: flex; flex-direction: column; justify-content: center;">
                    <h1 style="font-size: 2.5rem; margin-bottom: 1rem;">{ &props.metadata.name }</h1>
                    <p style="font-size: 1.1rem; line-height: 1.6; color: var(--accent);">{ &props.metadata.description }</p>
                </div>

                // Play button on the right
                <div style="display: flex; align-items: center;">
                    <button
                        style={button_style.clone()}
                        onmouseover={onmouseover}
                        onmouseout={onmouseout}
                    >
                        { "► Play" }
                    </button>
                </div>
            </div>
        </div>
    }
}
