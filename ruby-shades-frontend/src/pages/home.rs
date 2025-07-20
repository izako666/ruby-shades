use std::thread::current;

use crate::backend_handler::{Metadata, get_metadata};
use crate::components::glass_box::GlassBox;
use crate::components::loading_indicator::LoadingIndicator;
use crate::components::movie_block::MovieBlock;
use crate::components::show_block::ShowBlock;
use crate::pages::movie::MoviePage;
use crate::pages::show::ShowPage;
use crate::{DirectoryContext, backend_handler::PathObject};
use crate::{MetadataContext, Route};
use web_sys::console;
use web_sys::wasm_bindgen::JsValue;
use yew::{Callback, Html, Properties, function_component, html, use_context, use_state};
use yew_router::hooks::use_navigator;

#[derive(Properties, PartialEq)]
pub struct HomeProps {
    pub path: String,
}
#[function_component(Home)]
pub fn home(props: &HomeProps) -> Html {
    console::log_1(&JsValue::from_str("reached home page"));
    let directory = use_context::<DirectoryContext>().expect("no ctx found");
    let metadata_context = use_context::<MetadataContext>().expect("no metadata found");
    let navigator = use_navigator().unwrap();

    if let None = &*directory.0 {
        return html!(
            <>
            <div style="display: flex; justify-content: center; align-items: center; height: 100vh;">
                <LoadingIndicator />
            </div>
        </>);
    }
    if let None = &*metadata_context.0 {
        return html!(
            <>
            <div style="display: flex; justify-content: center; align-items: center; height: 100vh;">
                <LoadingIndicator />
            </div>
        </>);
    }
    let dir_obj = directory.0.as_ref().unwrap().clone();
    let metadata_obj = metadata_context.0.as_ref().unwrap().clone();
    let current_path_obj = if props.path.is_empty() {
        Some((*dir_obj).clone())
    } else {
        find_nested_path_object(&*dir_obj.clone(), &props.path)
    };

    if current_path_obj.is_none() {
        return html! { <div>{"Not Found"}</div> };
    }

    let current = current_path_obj.unwrap();
    console::log_1(&JsValue::from_str(&format!(
        "current path: {}",
        &current.path
    )));
    if let Some(metadata) = metadata_obj.metadata.get(&current.path) {
        let node = match metadata {
            Metadata::Show(show_metadata) => html! {
                <ShowPage metadata={show_metadata.clone()} path_object={current.clone()} />
            },
            Metadata::Movie(movie_metadata) => html! {
                <MoviePage metadata={movie_metadata.clone()} path_object={current.clone()} />
            },
            _ => html! {
                <h1>{ "404" }</h1>
            },
        };
        return html! {node};
    }
    let base_path = dir_obj.path.clone();
    let handle_click = {
        let navigator = navigator.clone();
        let base_path = base_path.clone();
        Callback::from(move |path: String| {
            let relative = path
                .strip_prefix(&base_path)
                .unwrap_or(&path)
                .trim_start_matches('/');

            navigator.push(&Route::Home {
                path: relative.to_string(),
            });
        })
    };
    let nested_paths: Vec<PathObject> = current.clone().nested_paths.clone();
    console::log_1(&JsValue::from_str(&format!(
        "nested_paths: {:?}",
        &nested_paths
    )));

    let nodes = nested_paths.iter().filter_map(|nested_path| {
            if let Some(metadata) = metadata_obj.metadata.get(&nested_path.path) {
                Some(match metadata {
                    Metadata::Show(show_metadata) => html! {
                        <ShowBlock show_metadata={show_metadata.clone()} path={nested_path.path.clone()} />
                    },
                    Metadata::Movie(movie_metadata) => html! {
                        <MovieBlock movie_metadata={movie_metadata.clone()} path={nested_path.path.clone()} />
                    },
                    _ => html! {
                        <GlassBox text={nested_path.name.clone()} path={nested_path.path.clone()} on_click={handle_click.clone()} />
                    },
                })
            } else {

                if nested_path.name.rsplit_once('.').is_some() {
                    return None;
                }
    Some(html! {
        <GlassBox text={nested_path.name.clone()} path={nested_path.path.clone()} on_click={handle_click.clone()} />
    })
            }
        }).collect::<Vec<Html>>();

    html! {
        <>
            <style>
                {r#"
                            .container {
                                display: flex;
                                flex-direction: column;
                                min-height: 100vh;
                                padding: 2rem;
                            }

                            .app-bar {
                                display: flex;
                                align-items: center;
                                margin-bottom: 2rem;
                            }

                            .app-bar img {
                                height: 40px;
                                margin-right: 1rem;
                            }

                            .app-bar h1 {
                                font-size: 2rem;
                            }

                            .grid {
                                display: grid;
                                gap: 2rem;
                                grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
                            }

                            @media (min-width: 900px) {
                                .grid {
                                    grid-template-columns: repeat(3, 1fr);
                                }
                            }
                        "#}
            </style>

            <div class="container">
                <div class="app-bar">
                    <img src="/logo.png" alt="Logo" />
                    <h1>{ "Ruby Shades" }</h1>
                </div>

                <div class="grid">
                    { for nodes }
                </div>
            </div>
        </>
    }
}

fn find_nested_path_object<'a>(main: &'a PathObject, relative: &str) -> Option<PathObject> {
    let segments: Vec<&str> = relative.split('/').collect();
    let mut current = main;

    for segment in segments {
        if let Some(found) = current.nested_paths.iter().find(|p| p.name == segment) {
            current = found;
        } else {
            return None;
        }
    }

    Some(current.clone())
}
