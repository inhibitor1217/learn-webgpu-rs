#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub mod app;
pub mod graphics;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn run() {
    app::App::run();
}
