use std::rc::Rc;

use web_sys::{console, wasm_bindgen::JsValue};
use yew::{
    ContextProvider, Html, UseStateHandle, function_component, html, use_effect, use_effect_with,
    use_state,
};
use yew_router::{BrowserRouter, Routable, Switch};

use crate::{
    backend_handler::{MetadataResponse, PathObject, get_all_metadata, get_directory},
    config::load_config,
    pages::home::Home,
};

mod backend_handler;
mod components;
mod config;
mod pages;
#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[not_found]
    #[at("/404")]
    NotFound,
}

#[derive(Clone, PartialEq)]
pub struct DirectoryContext(pub UseStateHandle<Option<Rc<PathObject>>>);

#[derive(Clone, PartialEq)]
pub struct MetadataContext(pub UseStateHandle<Option<Rc<MetadataResponse>>>);

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <Home/> },

        Route::NotFound => html! { <h1>{ "404" }</h1> },
    }
}

#[function_component]
fn App() -> Html {
    let config_loaded = use_state(|| false);
    let directory_state = use_state(|| None::<Rc<PathObject>>);
    let metadata_state = use_state(|| None::<Rc<MetadataResponse>>);

    {
        let config_loaded = config_loaded.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Err(e) = load_config().await {
                    gloo::console::error!(format!("Failed to load config: {e}"));
                } else {
                    config_loaded.set(true);
                }
            });
            || ()
        });
    }

    {
        let directory_state = directory_state.clone();
        let config_loaded = *config_loaded;
        use_effect_with(config_loaded, move |loaded| {
            if *loaded {
                wasm_bindgen_futures::spawn_local(async move {
                    match get_directory().await {
                        Ok(directory) => {
                            directory_state.set(Some(Rc::new(directory)));
                        }
                        Err(err) => {
                            console::log_1(&JsValue::from_str(&format!(
                                "Failed to get directory: {}",
                                err
                            )));
                        }
                    }
                });
            }
            || ()
        })
    }

    {
        let metadata_state = metadata_state.clone();
        let config_loaded = *config_loaded;
        use_effect_with(config_loaded, move |loaded| {
            if *loaded {
                wasm_bindgen_futures::spawn_local(async move {
                    match get_all_metadata().await {
                        Ok(metadata) => {
                            metadata_state.set(Some(Rc::new(metadata)));
                        }
                        Err(err) => {
                            console::log_1(&JsValue::from_str(&format!(
                                "Failed to get all metadata: {}",
                                err
                            )));
                        }
                    }
                });
            }
            || ()
        })
    }

    if !*config_loaded {
        return html! { <>{"Loading config..."}</> };
    }

    html! {
        <ContextProvider<DirectoryContext> context={DirectoryContext(directory_state.clone())}>
            <ContextProvider<MetadataContext> context={MetadataContext(metadata_state.clone())}>
                <BrowserRouter>
                    <Switch<Route> render={switch} />
                </BrowserRouter>
            </ContextProvider<MetadataContext>>
        </ContextProvider<DirectoryContext>>
    }
}
fn main() {
    yew::Renderer::<App>::new().render();
}
