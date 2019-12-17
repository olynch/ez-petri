#![recursion_limit = "512"]
mod math;
mod graph;
mod utils;

use wasm_bindgen::prelude::*;
use stdweb::web::{document, IParentNode};
use yew::app::App;
use stdweb::console;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// This is the entry point for the web app
#[wasm_bindgen]
pub fn run_app(mount_id: &str, js_props: &str) -> Result<(), JsValue> {
    let props: graph::Props = serde_json::from_str(js_props)
        .map_err(|e| JsValue::from_str(&format!("json parse error: {}",e)))?;
    utils::set_panic_hook();
    let document = document();
    let control_mount = document.query_selector(&format!("#{}",&mount_id)).unwrap().unwrap();
    App::<graph::App>::new().mount_with_props(control_mount,props);
    Ok(())
}
