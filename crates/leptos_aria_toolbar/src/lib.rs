#![doc(html_logo_url = "https://ifiokjr.github.io/leptos_aria/logo.svg")]
#![doc(html_favicon_url = "https://ifiokjr.github.io/leptos_aria/favicon.ico")]
#![allow(clippy::needless_pass_by_value)]

use leptos::prelude::*;
use leptos_aria_utils::merge_class;
use leptos_aria_utils::use_id;

/// A toolbar is a container for a set of related controls, navigated with
/// arrow keys.
#[component]
pub fn Toolbar(
	/// The content to display as the label.
	#[prop(optional)]
	label: Option<String>,
	/// The orientation of the toolbar, horizontal by default.
	#[prop(optional, into)]
	orientation: Option<Signal<&'static str>>,
	/// Classes for the root element.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let label_id = use_id();
	let resolved_orientation = Signal::derive(move || {
		orientation
			.as_ref()
			.map_or("horizontal", |orientation| orientation.get())
	});

	view! {
		{label.map(|label| view! { <span id=label_id.clone()>{label}</span> })}
		<div
			role="toolbar"
			class=merge_class("", class)
			aria-orientation=move || resolved_orientation.get()
			aria-labelledby=label_id.clone()
			on:keydown=move |event: web_sys::KeyboardEvent| {
				if event.key() == "Home" || event.key() == "End" {
					event.prevent_default();
				}
			}
		>
			{children()}
		</div>
	}
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
	use leptos::mount::mount_to_body;
	use wasm_bindgen_test::*;

	use super::*;

	wasm_bindgen_test_configure!(run_in_browser);

	#[wasm_bindgen_test]
	async fn toolbar_renders_the_aria_role() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! { <Toolbar label="Actions".to_string()><button>"A"</button></Toolbar> }
		});

		let toolbar = document()
			.query_selector("[role='toolbar']")
			.unwrap()
			.unwrap();

		assert_eq!(
			toolbar.get_attribute("aria-orientation").as_deref(),
			Some("horizontal")
		);
	}
}
