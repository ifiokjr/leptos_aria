#![doc(html_logo_url = "https://ifiokjr.github.io/leptos_aria/logo.svg")]
#![doc(html_favicon_url = "https://ifiokjr.github.io/leptos_aria/favicon.ico")]
#![allow(clippy::needless_pass_by_value)]

use std::collections::HashMap;
use std::sync::Arc;

use leptos::prelude::*;
use leptos_aria_list_box::ListBox;
use leptos_aria_list_box::ListBoxOption;
use leptos_aria_list_box::ListBoxRole;
use leptos_aria_overlay::use_overlay_dismiss;
use leptos_aria_overlay::use_popup_focus;
use leptos_aria_selection::Selection;
use leptos_aria_utils::merge_class;

/// A callback invoked with an item's value when the item is chosen.
pub type BoxedMenuActionCallback = Box<dyn Fn(String) + Send + Sync>;

/// The open state a [`MenuTrigger`] shares with the [`Menu`] inside it, so
/// choosing an item can close the trigger.
#[derive(Clone, Copy)]
pub struct MenuTriggerContext {
	/// Whether the trigger's menu is open.
	pub is_open: RwSignal<bool>,
}

/// The actions of one menu's items, by value, so the menu can run the
/// chosen one. The list box never reports a disabled item as chosen.
#[derive(Clone, Copy)]
struct MenuItems(StoredValue<HashMap<String, Callback<()>>>);

impl MenuItems {
	fn register(self, value: String, on_action: Callback<()>) {
		self.0.update_value(|items| {
			items.insert(value, on_action);
		});
	}

	/// Run the item's action, if it has one.
	fn activate(self, value: &str) {
		if let Some(on_action) = self.0.with_value(|items| items.get(value).copied()) {
			on_action.run(());
		}
	}
}

/// A menu presents a list of actions or options the user can choose.
///
/// The menu is an overlay list box: the same keyboard interaction, with a
/// `menu` role. Choosing an item, by pointer or with enter or space on the
/// highlighted one, runs its action and closes the menu; escape closes it
/// too. Inside a [`MenuTrigger`] the menu follows the trigger's open state.
#[component]
pub fn Menu(
	/// Whether the menu is open. Defaults to the enclosing
	/// [`MenuTrigger`]'s state, or open when there is none.
	#[prop(optional)]
	is_open: Option<RwSignal<bool>>,
	/// The content to display as the label.
	#[prop(optional)]
	label: Option<String>,
	/// Handler that is called with an item's value when it is chosen.
	#[prop(optional)]
	on_action: Option<BoxedMenuActionCallback>,
	/// Classes for the root element.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let trigger = use_context::<MenuTriggerContext>();
	let is_open = is_open
		.or(trigger.map(|trigger| trigger.is_open))
		.unwrap_or_else(|| RwSignal::new(true));
	let items = MenuItems(StoredValue::new(HashMap::new()));
	let close = move || {
		is_open.set(false);

		if let Some(trigger) = trigger {
			trigger.is_open.set(false);
		}
	};
	let on_action = on_action.map(Arc::<dyn Fn(String) + Send + Sync>::from);

	provide_context(items);

	// The list box never holds a selection, so choosing the same item twice
	// reports twice.
	let choose = Box::new(move |selection: &Selection| {
		let Selection::Keys(keys) = selection else {
			return;
		};
		let Some(value) = keys.iter().next() else {
			return;
		};

		items.activate(value);

		if let Some(on_action) = &on_action {
			on_action(value.clone());
		}

		close();
	});

	view! {
		<div
			class=merge_class("", class)
			hidden=move || !is_open.get()
			on:keydown=move |event: web_sys::KeyboardEvent| {
				if event.key() == "Escape" {
					close();
				}
			}
		>
			<ListBox
				role=ListBoxRole::Menu
				nostrip:label=label.filter(|label| !label.is_empty())
				selection=Signal::from(Selection::None)
				on_change=choose
			>
				{children()}
			</ListBox>
		</div>
	}
}

/// An action or option of a menu.
#[component]
pub fn MenuItem(
	/// The value of the item, reported to the menu's `on_action`.
	#[prop(into)]
	value: String,
	/// Whether this item is disabled.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	/// Handler that is called when this item is chosen.
	#[prop(optional, into)]
	on_action: Option<Callback<()>>,
	/// Classes for the root element, the list box option.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	if let (Some(items), Some(on_action)) = (use_context::<MenuItems>(), on_action) {
		items.register(value.clone(), on_action);
	}

	view! {
		<ListBoxOption role=ListBoxRole::Menu value=value is_disabled=is_disabled class=class>
			{children()}
		</ListBoxOption>
	}
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
	use std::sync::atomic::AtomicUsize;
	use std::sync::atomic::Ordering;

	use leptos::mount::mount_to_body;
	use wasm_bindgen::JsCast;
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
			.query_selector("[role='menuitem'].menu-test-item")
			.unwrap();

		assert_eq!(
			bar.get_attribute("class").as_deref(),
			Some("leptos-aria-menubar menu-test-bar flex")
		);
		assert!(option.is_some(), "the item class is on the option root");
	}

	pub(super) fn find(selector: &str) -> web_sys::HtmlElement {
		document()
			.query_selector(selector)
			.unwrap()
			.unwrap_or_else(|| panic!("nothing matches {selector}"))
			.unchecked_into::<web_sys::HtmlElement>()
	}

	pub(super) async fn tick() {
		let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
		wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
	}

	/// Dispatch a bubbling `keydown` for `key`.
	pub(super) fn key_down(target: &web_sys::Element, key: &str) {
		let init = web_sys::KeyboardEventInit::new();
		init.set_key(key);
		init.set_bubbles(true);

		let event =
			web_sys::KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init).unwrap();

		target.dispatch_event(&event).unwrap();
	}

	#[wasm_bindgen_test]
	async fn enter_runs_the_highlighted_item_and_closes() {
		console_error_panic_hook::set_once();

		let is_open = RwSignal::new(true);
		let chosen = RwSignal::new(Vec::<String>::new());
		let pastes = Arc::new(AtomicUsize::new(0));
		let item_pastes = Arc::clone(&pastes);

		mount_to_body(move || {
			view! {
				<Menu
					is_open=is_open
					on_action=Box::new(move |value| chosen.update(|chosen| chosen.push(value)))
					class="menu-test-enter"
				>
					<MenuItem value="copy">"Copy"</MenuItem>
					<MenuItem
						value="paste"
						on_action=move || {
							item_pastes.fetch_add(1, Ordering::Relaxed);
						}
					>
						"Paste"
					</MenuItem>
				</Menu>
			}
		});

		let listbox = find(".menu-test-enter [role='menu']");

		key_down(&listbox, "ArrowDown");
		key_down(&listbox, "ArrowDown");
		tick().await;

		let active = listbox
			.get_attribute("aria-activedescendant")
			.expect("the highlighted item is announced");

		assert_eq!(
			document()
				.get_element_by_id(&active)
				.and_then(|item| item.text_content())
				.as_deref(),
			Some("Paste")
		);

		key_down(&listbox, "Enter");
		tick().await;

		assert_eq!(chosen.get_untracked(), vec!["paste".to_string()]);
		assert_eq!(pastes.load(Ordering::Relaxed), 1, "the item's action runs");
		assert!(!is_open.get_untracked(), "choosing closes the menu");

		is_open.set(true);
		key_down(&listbox, " ");
		tick().await;

		assert_eq!(
			chosen.get_untracked().len(),
			2,
			"the same item can be chosen again"
		);
	}

	#[wasm_bindgen_test]
	async fn a_disabled_item_cannot_be_chosen() {
		console_error_panic_hook::set_once();

		let is_open = RwSignal::new(true);
		let chosen = RwSignal::new(Vec::<String>::new());
		let deletes = Arc::new(AtomicUsize::new(0));
		let item_deletes = Arc::clone(&deletes);

		mount_to_body(move || {
			view! {
				<Menu
					is_open=is_open
					on_action=Box::new(move |value| chosen.update(|chosen| chosen.push(value)))
					class="menu-test-disabled"
				>
					<MenuItem
						value="delete"
						is_disabled=true
						on_action=move || {
							item_deletes.fetch_add(1, Ordering::Relaxed);
						}
					>
						"Delete"
					</MenuItem>
				</Menu>
			}
		});

		find(".menu-test-disabled [role='menuitem']").click();
		tick().await;

		assert_eq!(chosen.get_untracked(), Vec::<String>::new());
		assert_eq!(
			deletes.load(Ordering::Relaxed),
			0,
			"the item's action does not run"
		);
		assert!(is_open.get_untracked(), "the menu stays open");
	}
}

/// A menu trigger opens a menu from a button.
///
/// The trigger chains the button press into the menu's open state, and
/// closes again on escape, outside interaction, or when the [`Menu`] inside
/// it runs an item. Focus moves into the menu on open and back to the
/// button on close.
#[component]
pub fn MenuTrigger(
	/// The menu's accessible label.
	#[prop(into)]
	label: String,
	/// Whether the menu is open. Pass a signal to read or control it;
	/// otherwise the trigger keeps its own state, closed at first.
	#[prop(optional)]
	is_open: Option<RwSignal<bool>>,
	/// Whether the trigger is disabled.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	/// Classes added after the built-in `leptos-aria-menu-trigger` class.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let is_open = is_open.unwrap_or_else(|| RwSignal::new(false));
	let root = NodeRef::<leptos::html::Div>::new();
	let button = NodeRef::<leptos::html::Button>::new();

	provide_context(MenuTriggerContext { is_open });
	use_overlay_dismiss(root, is_open.into(), Box::new(move || is_open.set(false)));
	use_popup_focus(root, button, is_open.into(), "[role=menu]");

	view! {
		<div class=merge_class("leptos-aria-menu-trigger", class) node_ref=root>
			<button
				type="button"
				class="leptos-aria-menu-trigger__button"
				aria-haspopup="menu"
				aria-expanded=move || is_open.get().to_string()
				disabled=move || is_disabled.get()
				node_ref=button
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

	use super::tests::find;
	use super::tests::tick;
	use super::*;

	wasm_bindgen_test_configure!(run_in_browser);

	#[wasm_bindgen_test]
	async fn menu_trigger_toggles_the_menu() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! {
				<MenuTrigger label="Actions".to_string() class="trigger-test-toggle">
					<Menu is_open=RwSignal::new(true)>
						<MenuItem value="copy".to_string()>"Copy"</MenuItem>
					</Menu>
				</MenuTrigger>
			}
		});

		let trigger = document()
			.query_selector(".trigger-test-toggle .leptos-aria-menu-trigger__button")
			.unwrap()
			.unwrap()
			.unchecked_into::<web_sys::HtmlButtonElement>();

		let menu = document()
			.query_selector(".trigger-test-toggle .leptos-aria-menu-trigger__menu")
			.unwrap()
			.unwrap();

		assert!(menu.has_attribute("hidden"));

		trigger.click();
		tick().await;

		assert!(!menu.has_attribute("hidden"), "the menu opens");
		assert_eq!(
			trigger.get_attribute("aria-expanded").as_deref(),
			Some("true")
		);
	}

	#[wasm_bindgen_test]
	async fn choosing_an_item_closes_the_trigger() {
		console_error_panic_hook::set_once();

		let is_open = RwSignal::new(false);

		mount_to_body(move || {
			view! {
				<MenuTrigger label="Edit" is_open=is_open class="trigger-test-choose">
					<Menu>
						<MenuItem value="copy">"Copy"</MenuItem>
					</Menu>
				</MenuTrigger>
			}
		});

		let button = find(".trigger-test-choose .leptos-aria-menu-trigger__button");

		button.click();
		tick().await;
		assert!(is_open.get_untracked(), "the owner sees the menu open");

		find(".trigger-test-choose [role='menuitem']").click();
		tick().await;

		assert!(!is_open.get_untracked(), "choosing closes the trigger");
		assert_eq!(
			button.get_attribute("aria-expanded").as_deref(),
			Some("false")
		);
	}

	#[wasm_bindgen_test]
	async fn a_press_outside_closes_the_trigger() {
		console_error_panic_hook::set_once();

		let is_open = RwSignal::new(true);

		mount_to_body(move || {
			view! {
				<MenuTrigger label="File" is_open=is_open class="trigger-test-outside">
					<Menu>
						<MenuItem value="new">"New"</MenuItem>
					</Menu>
				</MenuTrigger>
			}
		});
		tick().await;

		let event = web_sys::Event::new("pointerdown").unwrap();
		document().dispatch_event(&event).unwrap();
		tick().await;

		assert!(!is_open.get_untracked(), "an outside press closes the menu");
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
