use std::{ops::Deref, rc::Rc};

use yew::{Callback, Html, Properties, function_component, html, use_context, use_memo, use_state};

use crate::{
    MetadataContext,
    backend_handler::{Metadata, MetadataResponse, PathObject, TvShowMetadata},
    components::loading_indicator::LoadingIndicator,
};

// pass TvShowMetadata, get context metadata, pass path object
// traverse path object, check each path for episode, if so store episode_convenience_struct (u16 season, u16 episode, name, poster, description)
// create page Backdrop, under it is title, under it is overview, under that to the left is list block of seasons (only available ones based on the episodes we found (if we found an episode from s01, s03, s05 then those are the only seasons we show))
// to the right of the seasons block is a list of episode_tiles, each episode tile shows poster -> name and description over each other -> episode number, and should be hoverable and clickable
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

    if metadata_context.0.is_none() {
        return html!(
            <>
            <div style="display: flex; justify-content: center; align-items: center; height: 100vh;">
                <LoadingIndicator />
            </div>
        </>);
    }
    let data: Rc<MetadataResponse> = metadata_context.0.as_ref().cloned().unwrap();
    let seasons = use_memo(
        (
            props.path_object.clone(),
            props.metadata.clone(),
            data.clone(),
        ),
        |tup| {
            let episodes = traverse_path_for_episodes(tup.0.clone(), &*tup.2, &tup.1);

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
        },
        // Dependency tuple
    );

    let current_season = use_state(|| seasons.get(0).cloned().unwrap());
    html! {
        <div style="display: flex; flex-direction: column; width: 100%; background-color: var(--background); color: var(--text);">
            // Backdrop image
            <div style={format!(
                "width: 100%; background-size: cover; background-position: center; background-image: url('{}');",
                tmdb_image_url(&props.metadata.backdrop)
            )}>
                <div style="padding: 2rem; background: linear-gradient(to bottom, rgba(0,0,0,0.6), rgba(0,0,0,0.9));">
                    <h1 style="font-size: 2.5rem; margin: 0;">{ &props.metadata.name }</h1>
                </div>
            </div>

            // Overview section
            <div style="padding: 1rem 2rem; font-size: 1.1rem; line-height: 1.6;">
                { &props.metadata.description }
            </div>

            // Main content: Seasons on left, episodes on right
            <div style="display: flex; flex-direction: row; padding: 2rem; gap: 2rem;">
                // Seasons list
                <div style="min-width: 150px;">
                    <h3 style="color: var(--accent);">{ "Seasons" }</h3>
                    <ul style="list-style: none; padding: 0; margin: 0;">
                        {
                            for (*seasons).clone().iter().map(|season| {
                                let is_selected = season.number == current_season.number;
                                let border_style = if is_selected {
                                    "2px solid var(--primary)"
                                } else {
                                    "2px solid transparent"
                                };
                                let season_number = season.number;
                                let onclick = {
                                    let current_season = current_season.clone();
                                    Callback::from(move |_| {
                                        current_season.set(season.clone());
                                    })
                                };
                                html! {
                                    <li
                                        {onclick}
                                        style={format!(
                                            "margin-bottom: 0.5rem; cursor: pointer; padding: 0.5rem; border-radius: 5px; border: {}; background-color: var(--secondary); color: var(--text);",
                                            border_style
                                        )}>
                                        { format!("Season {}", season.number) }
                                    </li>
                                }
                            })
                        }
                    </ul>
                </div>

                // Episodes grid
                <div style="flex-grow: 1; display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 1.5rem;">
                    {
                        for current_season.episodes.iter().map(|ep| {
                            html! {
                                <div style="border: 1px solid var(--accent); border-radius: 8px; overflow: hidden; box-shadow: 0 2px 5px rgba(0,0,0,0.1); cursor: pointer; transition: transform 0.2s ease-in-out;">
                                    <img src={tmdb_image_url(&ep.poster)} alt={ep.name.clone()} style="width: 100%; height: auto;" />
                                    <div style="padding: 1rem;">
                                        <h4 style="margin: 0 0 0.5rem 0; color: var(--text);">{ &ep.name }</h4>
                                        <p style="font-size: 0.9rem; color: var(--accent);">{ &ep.description }</p>
                                        <span style="font-size: 0.8rem; color: var(--accent);">{ format!("Episode {}", ep.episode) }</span>
                                    </div>
                                </div>
                            }
                        })
                    }
                </div>
            </div>
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

    for item in path_stack.pop() {
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
                    };
                    episodes.push(new_episode);
                }
            }
        }
    }
    return episodes;
}

fn tmdb_image_url(path: &str) -> String {
    format!("https://image.tmdb.org/t/p/original{}", path)
}
