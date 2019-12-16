#![recursion_limit = "512"]
mod app;
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
    let props: app::Props = serde_json::from_str(js_props).unwrap();
    // console!(log, "deserialized");
    utils::set_panic_hook();
    let document = document();
    let control_mount = document.query_selector(&format!("#{}",&mount_id)).unwrap().unwrap();
    // console!(log, "mount point aquired");
    App::<app::App>::new().mount_with_props(control_mount,props);
    Ok(())
}
