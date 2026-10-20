//! Live previews of the `leptos_aria` registry components.
//!
//! The website's component pages frame `preview/index.html?example=<name>`
//! in an iframe. This crate is that page's app: it mounts the named
//! component's usage example, compiled by the build script from the
//! component's own module docs, so the preview runs exactly the code shown
//! beside it.
//!
//! ```bash
//! trunk build --release --config crates/leptos_aria_preview/Trunk.toml
//! ```

use leptos::prelude::*;
use wasm_bindgen::JsCast;

mod examples {
	use leptos::prelude::*;

	use super::Render;

	include!(concat!(env!("OUT_DIR"), "/examples.rs"));
}

pub use examples::EXAMPLES;

/// Renders one usage example.
pub type Render = fn() -> AnyView;

/// The id of the element the example mounts into, in `index.html`.
const MOUNT_ID: &str = "preview";

/// The usage example of the registry item called `name`.
pub fn example(name: &str) -> Option<Render> {
	EXAMPLES
		.iter()
		.find(|(example, _)| *example == name)
		.map(|(_, render)| *render)
}

/// Mount the example named by the page's `?example=` query, or a note that
/// there is none.
pub fn start() {
	console_error_panic_hook::set_once();

	let name = window()
		.location()
		.search()
		.ok()
		.and_then(|search| web_sys::UrlSearchParams::new_with_str(&search).ok())
		.and_then(|params| params.get("example"))
		.unwrap_or_default();
	let root = document()
		.get_element_by_id(MOUNT_ID)
		.expect("index.html has a mount element")
		.unchecked_into();

	mount_to(root, move || {
		match example(&name) {
			Some(render) => render(),
			None => {
				view! { <p class="text-sm text-muted-foreground">"No example named “" {name} "”."</p> }
					.into_any()
			}
		}
	})
	.forget();
}
