use web_sys::wasm_bindgen::prelude::Closure;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct GlassBoxProps {
    pub text: String,
    pub path: String,
    pub on_click: Callback<String>,
}

#[function_component(GlassBox)]
pub fn glass_box(props: &GlassBoxProps) -> Html {
    let onclick = {
        let text = props.path.clone();
        let on_click = props.on_click.clone();
        Callback::from(move |_| on_click.emit(text.clone()))
    };
    html! {
        <>
            <style>
                {r#"
                    .glass-box {
                        aspect-ratio: 3 / 2;
                        display: flex;
                        justify-content: center;
                        align-items: center;
                        position: relative;
                        padding: 1rem;
                        border-radius: 20px;
                        overflow: hidden;

                        background: linear-gradient(
                            135deg,
                            rgba(155, 17, 30, 0.08),
                            rgba(255, 255, 255, 0.06)
                        );
                        backdrop-filter: blur(16px) saturate(160%);
                        -webkit-backdrop-filter: blur(16px) saturate(160%);
                        box-shadow: 0 4px 16px rgba(0, 0, 0, 0.25);

                        color: var(--text);
                        font-size: 1.5rem;
                        text-align: center;
                        transition: transform 0.3s ease, box-shadow 0.3s ease;
                    }

                    .glass-box:hover {
                        transform: scale(1.02);
                        box-shadow: 0 6px 16px rgba(155, 17, 30, 0.15);
                    }

                    .glass-noise {
                        position: absolute;
                        inset: 0;
                        background-image: url("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABAQMAAAAl21bKAAAABlBMVEUAAAD///+l2Z/dAAAACklEQVR4nGMAAQAABQABDQottAAAAABJRU5ErkJggg==");
                        background-repeat: repeat;
                        opacity: 0.05;
                        pointer-events: none;
                        z-index: 0;
                    }

                    .glass-content {
                        z-index: 1;
                    }
                "#}
            </style>

            <div class="glass-box" onclick={onclick}>
                <div class="glass-noise" />
                <span class="glass-content">{ &props.text }</span>
            </div>
        </>
    }
}
