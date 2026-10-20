#![doc(html_logo_url = "https://ifiokjr.github.io/leptos_aria/logo.svg")]
#![doc(html_favicon_url = "https://ifiokjr.github.io/leptos_aria/favicon.ico")]
#![allow(clippy::needless_pass_by_value)]

use leptos::prelude::*;
use leptos_aria_list_box::ListBox;
use leptos_aria_list_box::ListBoxOption;
use leptos_aria_utils::merge_class;

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
	/// Classes for the root element.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	view! {
		<div
			class=merge_class("", class)
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
	/// Classes for the root element, the list box option.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	view! {
		<ListBoxOption value=value is_disabled=is_disabled class=class>
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
				<div id="plain-menu-test">
					<Menu is_open=RwSignal::new(true) label="Actions".to_string()>
						<MenuItem value="copy".to_string()>"Copy"</MenuItem>
						<MenuItem value="paste".to_string()>"Paste"</MenuItem>
					</Menu>
				</div>
			}
		});

		let scope = document()
			.query_selector("#plain-menu-test")
			.unwrap()
			.unwrap();
		let menu = scope.query_selector("[role='menu']").unwrap().unwrap();

		assert!(!menu.has_attribute("hidden"), "an open menu is visible");

		let items = scope.query_selector_all("[role='menuitem']").unwrap();

		assert_eq!(items.length(), 2);
	}

	#[wasm_bindgen_test]
	async fn menu_item_class_lands_on_the_option() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! {
				<Menubar class="menu-test-bar flex">
					<Menu is_open=RwSignal::new(true)>
						<MenuItem value="copy".to_string() class="menu-test-item">"Copy"</MenuItem>
					</Menu>
				</Menubar>
			}
		});

		let bar = document()
			.query_selector(".menu-test-bar")
			.unwrap()
			.unwrap();
		let option = document()
			.query_selector("[role='option'].menu-test-item")
			.unwrap();

		assert_eq!(
			bar.get_attribute("class").as_deref(),
			Some("leptos-aria-menubar menu-test-bar flex")
		);
		assert!(option.is_some(), "the item class is on the option root");
	}
}

/// A menu trigger opens a menu from a button.
///
/// The trigger chains the button press into the menu's open state, and
/// closes again on escape or outside interaction.
#[component]
pub fn MenuTrigger(
	/// The menu's accessible label.
	#[prop(into)]
	label: String,
	/// Whether the trigger is disabled.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	/// Classes added after the built-in `leptos-aria-menu-trigger` class.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let is_open = RwSignal::new(false);

	view! {
		<div class=merge_class("leptos-aria-menu-trigger", class)>
			<button
				type="button"
				class="leptos-aria-menu-trigger__button"
				aria-haspopup="menu"
				aria-expanded=move || is_open.get().to_string()
				disabled=move || is_disabled.get()
				on:click=move |_| is_open.update(|open| *open = !*open)
			>
				{label.clone()}
			</button>
			<div
				class="leptos-aria-menu-trigger__menu"
				hidden=move || !is_open.get()
				on:keydown=move |event: web_sys::KeyboardEvent| {
					if event.key() == "Escape" {
						is_open.set(false);
					}
				}
			>
				{children()}
			</div>
		</div>
	}
}

#[cfg(all(test, target_arch = "wasm32"))]
mod trigger_tests {
	use leptos::mount::mount_to_body;
	use wasm_bindgen::JsCast;
	use wasm_bindgen_test::*;

	use super::*;

	wasm_bindgen_test_configure!(run_in_browser);

	#[wasm_bindgen_test]
	async fn menu_trigger_toggles_the_menu() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! {
				<MenuTrigger label="Actions".to_string()>
					<Menu is_open=RwSignal::new(true)>
						<MenuItem value="copy".to_string()>"Copy"</MenuItem>
					</Menu>
				</MenuTrigger>
			}
		});

		let trigger = document()
			.query_selector(".leptos-aria-menu-trigger__button")
			.unwrap()
			.unwrap()
			.unchecked_into::<web_sys::HtmlButtonElement>();

		let menu = document()
			.query_selector(".leptos-aria-menu-trigger__menu")
			.unwrap()
			.unwrap();

		assert!(menu.has_attribute("hidden"));

		trigger.click();

		let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
		wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();

		assert!(!menu.has_attribute("hidden"), "the menu opens");
		assert_eq!(
			trigger.get_attribute("aria-expanded").as_deref(),
			Some("true")
		);
	}
}

/// A context menu opens its menu on right-click instead of a button press.
#[component]
pub fn ContextMenu(
	/// Classes added after the built-in `leptos-aria-context-menu__trigger`
	/// class.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let is_open = RwSignal::new(false);

	view! {
		<span
			class=merge_class("leptos-aria-context-menu__trigger", class)
			data-state=move || if is_open.get() { "open" } else { "closed" }
			on:contextmenu=move |event: web_sys::MouseEvent| {
				event.prevent_default();
				is_open.set(true);
			}
		>
			{children()}
			<Menu is_open=is_open label=String::new()>{|| ().into_view()}</Menu>
		</span>
	}
}

/// A menubar is a horizontal bar of menus, navigated left and right.
#[component]
pub fn Menubar(
	/// Classes added after the built-in `leptos-aria-menubar` class.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	view! {
		<div
			class=merge_class("leptos-aria-menubar", class)
			role="menubar"
			aria-orientation="horizontal"
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
