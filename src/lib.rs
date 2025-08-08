mod sketch;
mod easing;

use async_std::task::block_on;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::{Element, HtmlCanvasElement};

use crate::sketch::{run_app, run_app_canvas};

// #[wasm_bindgen]
// pub async fn start_with_canvas(canvas: HtmlCanvasElement) -> Result<(), JsValue> {
//     #[cfg(debug_assertions)]
//     console_error_panic_hook::set_once();
//
//     // hand the canvas into your app
//     run_app_canvas(canvas).await;
//     Ok(())
// }

#[wasm_bindgen]
pub async fn start(width: u32, height: u32) -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    // hand the canvas into your app
    run_app(width, height).await;
    Ok(())
}