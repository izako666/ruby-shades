use gloo::utils::document;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlBaseElement, wasm_bindgen};

#[wasm_bindgen(inline_js = "
export function setup_hls(video_id, src) {
    const video = document.getElementById(video_id);
    if (Hls.isSupported()) {
        const hls = new Hls();
        hls.loadSource(src);
        hls.attachMedia(video);
        hls.on(Hls.Events.MANIFEST_PARSED, function () {
            video.play();
        });
    } else if (video.canPlayType('application/vnd.apple.mpegurl')) {
        video.src = src;
        video.addEventListener('loadedmetadata', function () {
            video.play();
        });
    }
}
")]
extern "C" {
    fn setup_hls(video_id: &str, src: &str);
}

pub fn init_hls_player(video_id: &str, src: &str) {
    setup_hls(video_id, src);
}
