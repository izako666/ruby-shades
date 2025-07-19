use yew::prelude::*;

#[function_component(LoadingIndicator)]
pub fn loading_indicator() -> Html {
    html! {
        <>
            <style>
                {r#"
                    .loading-spinner {
                        border: 8px solid var(--secondary); /* Light grey background */
                        border-top: 8px solid var(--primary); /* Red color for the spinner */
                        border-radius: 50%;
                        width: 50px;
                        height: 50px;
                        animation: spin 2s linear infinite;
                    }

                    /* Spin animation */
                    @keyframes spin {
                        0% { transform: rotate(0deg); }
                        100% { transform: rotate(360deg); }
                    }

                    .loading-container {
                        display: flex;
                        justify-content: center;
                        align-items: center;
                        height: 100vh;
                    }
                "#}
            </style>

            <div class="loading-container">
                <div class="loading-spinner"></div>
            </div>
        </>
    }
}
