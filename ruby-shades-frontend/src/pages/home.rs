use std::thread::current;

use crate::backend_handler::get_metadata;
use crate::components::glass_box::GlassBox;
use crate::components::loading_indicator::LoadingIndicator;
use crate::{DirectoryContext, backend_handler::PathObject};
use yew::{Callback, Html, function_component, html, use_context, use_state};

/*
let directory;
let current_directory;
if current_directory.isNone -> loading
current_directory.nested_paths -> loop through -> check if each has metadata -> if it does check show, movie, -> <ShowBlock/> -> opens <ShowPage/>, <MovieBlock/> -> opens <MoviePage/>, if not <Block/>





*/
#[function_component(Home)]
pub fn home() -> Html {
    let directory = use_context::<DirectoryContext>().expect("no ctx found");
    let current_directory = use_state(|| directory.0.clone());
    if current_directory.is_none() {
        return html! {
            <>
                <div style="display: flex; justify-content: center; align-items: center; height: 100vh;">
                    <LoadingIndicator />
                </div>
            </>
        };
    }
    let metadata = get_metadata(&current_directory.unwrap().path);
    let handle_click = Callback::from(|text: String| {
        web_sys::console::log_1(&format!("GlassBox clicked: {}", text).into());
    });
    let content = match &*directory.0 {
        Some(path_obj) => {
            let boxes = path_obj.nested_paths.iter().map(|path| {
                html! {
                    <GlassBox text={path.name.clone()} path={path.path.clone()} on_click={handle_click.clone()} />
                }
            });

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
                            { for boxes }
                        </div>
                    </div>
                </>
            }
        }
        None => html! { <div style="color: white;">{ "Loading..." }</div> },
    };

    html! { content }
}
