#![allow(clippy::needless_pass_by_value)]

use leptos::prelude::*;
use leptos_aria_list_box::ListBox;
use leptos_aria_list_box::ListBoxOption;

/// A menu presents a list of actions or options the user can choose.
///
/// The menu is an overlay list box: the same keyboard interaction, with a
/// `menu` role and escape closing it again.
#[component]
pub fn Menu(
	/// Whether the menu is open.
	is_open: RwSignal<bool>,
	/// The content to display as the label.
	#[prop(optional)]
	label: Option<String>,
	children: Children,
) -> impl IntoView {
	view! {
		<div
			role="menu"
			hidden=move || !is_open.get()
			on:keydown=move |event: web_sys::KeyboardEvent| {
				if event.key() == "Escape" {
					is_open.set(false);
				}
			}
		>
			<ListBox label=label.unwrap_or_default()>{children()}</ListBox>
		</div>
	}
}

/// An action or option of a menu.
#[component]
pub fn MenuItem(
	/// The value of the item, reported to the selection.
	#[prop(into)]
	value: String,
	/// Whether this item is disabled.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	children: Children,
) -> impl IntoView {
	view! {
		<ListBoxOption value=value is_disabled=is_disabled>
			<span role="menuitem">{children()}</span>
		</ListBoxOption>
	}
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
	use leptos::mount::mount_to_body;
	use wasm_bindgen_test::*;

	use super::*;

	wasm_bindgen_test_configure!(run_in_browser);

	#[wasm_bindgen_test]
	async fn menu_renders_items_with_menuitem_roles() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! {
				<Menu is_open=RwSignal::new(true) label="Actions".to_string()>
					<MenuItem value="copy".to_string()>"Copy"</MenuItem>
					<MenuItem value="paste".to_string()>"Paste"</MenuItem>
				</Menu>
			}
		});

		let menu = document().query_selector("[role='menu']").unwrap().unwrap();

		assert!(!menu.has_attribute("hidden"), "an open menu is visible");

		let items = document().query_selector_all("[role='menuitem']").unwrap();

		assert_eq!(items.length(), 2);
	}
}
