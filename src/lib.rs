#![recursion_limit = "1024"]
mod math;
mod petri;
mod plot;
mod onload;
mod editor;
mod utils;

use wasm_bindgen::prelude::*;
use stdweb::web::{document, IParentNode};
use yew::app::App;
use crate::editor::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// This is the entry point for the web app
#[wasm_bindgen]
pub fn run_app(mount_id: &str) -> Result<(), JsValue> {
    utils::set_panic_hook();
    let document = document();
    let control_mount = document.query_selector(&format!("#{}",&mount_id)).unwrap().unwrap();
    App::<Editor>::new().mount_with_props(control_mount,EditorProps {});
    Ok(())
}
