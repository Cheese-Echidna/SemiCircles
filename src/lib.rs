mod sketch;
mod easing;

use async_std::task::block_on;
use wasm_bindgen::prelude::wasm_bindgen;

use sketch::{run_app};

// web app entry_point
#[wasm_bindgen(start)]
pub async fn main_web() {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    println!("Rust app started");

    block_on(async move {
        run_app(1920, 1080).await;
    });
}

