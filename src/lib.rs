#![recursion_limit = "1024"]
mod math;
mod petri;
mod plot;
#[cfg(target_arch = "wasm32")]
mod editor;
mod utils;
mod ssa;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use stdweb::web::{document, IParentNode};

#[cfg(target_arch = "wasm32")]
use yew::app::App;

#[cfg(target_arch = "wasm32")]
use crate::editor::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// This is the entry point for the web app
#[wasm_bindgen]
#[cfg(target_arch = "wasm32")]
pub fn run_app(mount_id: &str) -> Result<(), JsValue> {
    utils::set_panic_hook();
    let document = document();
    let control_mount = document.query_selector(&format!("#{}",&mount_id)).unwrap().unwrap();
    let initial_state = document.location()
        .and_then(|l| { l.hash().ok() })
        .and_then(|h| { GE::from_url_hash(if h.len() > 0 { &h[1..] } else { &h }) })
        .unwrap_or_default();
    App::<Editor>::new().mount_with_props(control_mount,EditorProps {
        initial_state: initial_state
    });
    Ok(())
}
