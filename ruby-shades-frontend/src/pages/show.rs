use std::rc::Rc;

use yew::{
    Callback, Html, Properties, function_component, html, use_context, use_state,
    virtual_dom::VNode,
};
use yew_router::hooks::use_navigator;

use crate::{
    MetadataContext, Route,
    backend_handler::{Metadata, MetadataResponse, PathObject, TvShowMetadata},
    components::loading_indicator::LoadingIndicator,
    pages::tmdb_image_url,
};

#[derive(Properties, PartialEq)]
pub struct TvShowProps {
    pub metadata: TvShowMetadata,
    pub path_object: PathObject,
}
#[derive(Clone)]
struct Episode {
    season: u16,
    episode: u16,
    name: String,
    poster: String,
    description: String,
    path: String,
}

#[derive(Clone)]
struct Season {
    number: u16,
    episodes: Vec<Episode>,
}

#[function_component(ShowPage)]
pub fn show(props: &TvShowProps) -> Html {
    let metadata_context =
        use_context::<MetadataContext>().expect("metadata context should exist at this stage");
    let navigator = use_navigator().unwrap();
    if metadata_context.0.is_none() {
        return html!(
            <>
            <div style="display: flex; justify-content: center; align-items: center; height: 100vh;">
                <LoadingIndicator />
            </div>
        </>);
    }
    let data: Rc<MetadataResponse> = metadata_context.0.as_ref().cloned().unwrap();
    let path_clone = props.path_object.clone();
    let metadata_clone = props.metadata.clone();
    let metadata_response = metadata_context.0.as_ref().unwrap();
    let seasons = use_state(|| {
        let episodes = traverse_path_for_episodes(path_clone, &metadata_response, &metadata_clone);

        let mut seasons: Vec<Season> = Vec::new();
        for ep in episodes {
            match seasons.iter_mut().find(|s| s.number == ep.season) {
                Some(season) => season.episodes.push(ep),
                None => seasons.push(Season {
                    number: ep.season,
                    episodes: vec![ep],
                }),
            };
        }
        seasons.sort_by(|a, b| a.number.cmp(&b.number));
        seasons
    });

    let current_season = use_state(|| seasons.get(0).cloned().unwrap());
    let nodes: VNode = (seasons).iter().map(|season| {
        let cloned_season = season.clone();
        let is_selected = cloned_season.number == current_season.number;
        let border_style = if is_selected {
            "2px solid var(--primary)"
        } else {
            "2px solid transparent"
        };
        let background_color = if is_selected {
            "var(--primary)"
        } else {
            "transparent"
        };
        let onclick = {
            let current_season = current_season.clone();
            Callback::from(move |_| {
                current_season.set(cloned_season.clone());
            })
        };

        let cloned_season = season.clone();
        html! {
            <li
                {onclick}
                style={format!(
                    "margin-bottom: 0.5rem; cursor: pointer; padding: 0.5rem; border-radius: 5px; border: {}; background-color: {}; color: var(--text); transition: background 0.3s ease;",
                    border_style,
                    background_color
                )}>
                { format!("Season {}", cloned_season.number) }
            </li>
        }
    }).collect();

    html! {
        <div style="display: flex; flex-direction: column; width: 100%; background-color: var(--background); color: var(--text);">

            // Backdrop Image
            <div style={format!(
                "width: 100%; height: 1000px; background-size: cover; background-position: center; background-image: url('{}'); position: relative; z-index: 0;",
                tmdb_image_url(&props.metadata.backdrop)
            )}>
                <div style="position: absolute; bottom: 0; width: 100%; height: 100%; background: linear-gradient(to top, rgba(0,0,0,0.9), rgba(0,0,0,0.3));">
                </div>
            </div>

            // Poster + Title + Description
            <div style="display: flex; flex-direction: row; padding: 2rem; gap: 2rem; margin-top: 0; align-items: flex-start; z-index: 1; position: relative;">
                // Poster
                <img
                    src={tmdb_image_url(&props.metadata.poster)}
                    alt="poster"
                    style="width: 300px; border-radius: 12px; box-shadow: 0 4px 20px rgba(0,0,0,0.6); z-index: 2; margin-top: -300px;"
                />

                // Title + Description
                <div style="flex: 1; z-index: 2;">
                    <h1 style="font-size: 3rem; margin-bottom: 1rem; color: var(--text);">{ &props.metadata.name }</h1>
                    <p style="font-size: 1.2rem; line-height: 1.6; color: var(--text-muted);">
                        { &props.metadata.description }
                    </p>
                </div>
            </div>

            // Seasons + Episodes
            <div style="display: flex; flex-direction: row; padding: 2rem; gap: 3rem;">

                // Seasons list
                <div style="width: 35%;">
                    <h3 style="color: var(--accent); margin-bottom: 1rem;">{ "Seasons" }</h3>
                    <ul style="list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 0.5rem;">
                        { nodes }
                    </ul>
                </div>

                // Episodes grid
                <div style="flex-grow: 1; display: grid; grid-template-columns: repeat(3, 1fr); gap: 2rem;">
                    {

                        for current_season.episodes.iter().map(|ep| {
                            let on_click_episode = {
                                let episode = ep.clone();
                                let navigator = navigator.clone();

                                Callback::from(move |_| {
                                    navigator.push_with_query(&Route::Watch, &[("resource", episode.path.clone())]).unwrap();
                                })
                            };

                            html! {
                                <div onclick={on_click_episode} style="border-radius: 10px; overflow: hidden; background-color: var(--surface); box-shadow: 0 4px 12px rgba(0,0,0,0.2); transition: transform 0.2s ease-in-out; cursor: pointer; min-height: 400px; display: flex; flex-direction: column;">
                                    <img src={tmdb_image_url(&ep.poster)} alt={ep.name.clone()} style="width: 100%; height: auto;" />
                                    <div style="padding: 1rem; flex: 1; display: flex; flex-direction: column;">
                                        <h4 style="margin: 0 0 0.5rem 0; font-size: 1.2rem; color: var(--text);">{ &ep.name }</h4>
                                        <div style="flex: 1; overflow-y: auto; margin-bottom: 0.5rem;">
                                            <p style="font-size: 1rem; color: var(--text-muted);">{ &ep.description }</p>
                                        </div>
                                        <span style="font-size: 0.85rem; color: var(--accent);">{ format!("Episode {}", ep.episode) }</span>
                                    </div>
                                </div>
                            }
                        })
                    }
                </div>
            </div>

            <div style="height: 300px;"></div>
        </div>
    }
}

fn traverse_path_for_episodes(
    path: PathObject,
    data: &MetadataResponse,
    show_data: &TvShowMetadata,
) -> Vec<Episode> {
    let mut path_stack: Vec<PathObject> = Vec::new();
    let mut episodes: Vec<Episode> = Vec::new();
    path.nested_paths
        .iter()
        .for_each(|p| path_stack.push(p.clone()));

    while let Some(item) = path_stack.pop() {
        item.nested_paths
            .iter()
            .for_each(|p| path_stack.push(p.clone()));
        if data.metadata.contains_key(&item.path) {
            let ep_metadata = data.metadata.get(&item.path);
            if let Some(ep_metadata) = ep_metadata {
                if let Metadata::Episode(ep) = ep_metadata {
                    let mut season_number = 0;

                    for (i, season) in show_data.seasons.iter().enumerate() {
                        if season.episodes.iter().any(|e| e.name == ep.name) {
                            season_number = i as u16;
                            break;
                        }
                    }

                    let new_episode: Episode = Episode {
                        season: season_number + 1 as u16,
                        episode: ep.number,
                        name: ep.name.to_string(),
                        poster: ep.poster.to_string(),
                        description: ep.description.to_string(),
                        path: item.path,
                    };
                    episodes.push(new_episode);
                }
            }
        }
    }
    return episodes;
}
