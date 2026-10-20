#![doc(html_logo_url = "https://ifiokjr.github.io/leptos_aria/logo.svg")]
#![doc(html_favicon_url = "https://ifiokjr.github.io/leptos_aria/favicon.ico")]
#![allow(clippy::needless_pass_by_value)]

use std::collections::HashMap;

use leptos::prelude::*;
use leptos_aria_selection::CollectionKeys;
use leptos_aria_selection::SelectionMode;
use leptos_aria_selection::SelectionState;
use leptos_aria_selection::UseSelectionStateProps;
use leptos_aria_selection::use_selection_state;
use leptos_aria_utils::merge_class;
use leptos_aria_utils::use_id;

/// A callback invoked when the focused option changes.
pub type BoxedFocusChangeCallback = Box<dyn Fn(Option<String>) + 'static>;

/// The state a list box shares with its options through context.
#[derive(Clone)]
pub struct ListBoxContext {
	pub state: SelectionState,
	pub keys: RwSignal<CollectionKeys>,
	pub focused_key: RwSignal<Option<String>>,
	/// The id of the list box element; option ids derive from it.
	pub id: String,
	/// Each registered option's disabled flag, by key. A key without a flag
	/// is enabled.
	pub disabled_options: StoredValue<HashMap<String, Signal<bool>>>,
}

impl ListBoxContext {
	/// Register an option's key in document order, as an enabled option.
	pub fn register_key(&self, key: &str) {
		self.keys.update(|keys| {
			keys.0.push(key.to_string());
		});
	}

	/// Register an option's key in document order with its disabled flag.
	pub fn register_option(&self, key: &str, is_disabled: Signal<bool>) {
		self.register_key(key);
		self.disabled_options.update_value(|options| {
			options.insert(key.to_string(), is_disabled);
		});
	}

	/// Whether the option with this key is disabled, read untracked.
	///
	/// Keys outlive their options, so an option that has unmounted, whose
	/// flag is disposed, counts as disabled: it can be neither focused nor
	/// selected.
	pub fn is_key_disabled(&self, key: &str) -> bool {
		self.disabled_options
			.with_value(|options| options.get(key).copied())
			.is_some_and(|is_disabled| is_disabled.try_get_untracked().unwrap_or(true))
	}

	/// The id of the option with this key, unique within the document.
	///
	/// Reactive: reading inside an effect or view tracks the registered keys.
	pub fn option_id(&self, key: &str) -> Option<String> {
		let index = self
			.keys
			.with(|keys| keys.0.iter().position(|k| k == key))?;

		Some(format!("{}-option-{index}", self.id))
	}

	/// The id of the focused option, for `aria-activedescendant`.
	///
	/// Reactive: reading inside an effect or view tracks the focus.
	pub fn active_descendant(&self) -> Option<String> {
		self.focused_key.get().and_then(|key| self.option_id(&key))
	}

	/// Move focus to the enabled option after the currently focused one.
	pub fn focus_next(&self) {
		let current = self.focused_key.get_untracked();
		self.focus(self.find_enabled_key(current.as_deref(), true, |_| true));
	}

	/// Move focus to the enabled option before the currently focused one.
	pub fn focus_previous(&self) {
		let current = self.focused_key.get_untracked();
		self.focus(self.find_enabled_key(current.as_deref(), false, |_| true));
	}

	/// Move focus to the first enabled option.
	pub fn focus_first(&self) {
		self.focus(self.find_enabled_key(None, true, |_| true));
	}

	/// Move focus to the last enabled option.
	pub fn focus_last(&self) {
		self.focus(self.find_enabled_key(None, false, |_| true));
	}

	/// Move focus to the enabled option matching a typed prefix.
	pub fn search_keys(&self, prefix: &str) {
		let current = self.focused_key.get_untracked();
		let prefix = prefix.to_lowercase();

		self.focus(self.find_enabled_key(current.as_deref(), true, |key| {
			key.to_lowercase().starts_with(&prefix)
		}));
	}

	/// Select the focused option, or a given one. Disabled options are
	/// ignored.
	pub fn select(&self, key: Option<&str>) {
		let key = key
			.map(str::to_string)
			.or_else(|| self.focused_key.get_untracked());

		if let Some(key) = key.filter(|key| !self.is_key_disabled(key)) {
			self.state.select_key(&key, false);
		}
	}

	/// Focus `key` when there is one, keeping the current focus otherwise.
	fn focus(&self, key: Option<String>) {
		if key.is_some() {
			self.focused_key.set(key);
		}
	}

	/// Walk the keys once from `from` in one direction, wrapping, and return
	/// the first enabled key that `accepts`. Without `from`, the walk starts
	/// at the first key going forward and the last going backward.
	fn find_enabled_key(
		&self,
		from: Option<&str>,
		forward: bool,
		accepts: impl Fn(&str) -> bool,
	) -> Option<String> {
		self.keys.with_untracked(|keys| {
			let keys = &keys.0;
			let length = keys.len();
			let start = from.and_then(|from| keys.iter().position(|key| key == from));

			(1..=length)
				.map(|offset| {
					match (start, forward) {
						(Some(start), true) => (start + offset) % length,
						(Some(start), false) => (start + length - offset) % length,
						(None, true) => offset - 1,
						(None, false) => length - offset,
					}
				})
				.filter_map(|index| keys.get(index))
				.find(|key| accepts(key) && !self.is_key_disabled(key))
				.cloned()
		})
	}
}

/// A list box presents a list of options and lets the user select one or
/// more of them.
///
/// Arrow keys move focus between options, home and end jump to the first and
/// last option, typing moves focus by prefix, and enter or space selects the
/// focused option.
#[component]
pub fn ListBox(
	/// Whether the user can select more than one option.
	#[prop(optional, into)]
	is_multiple: Signal<bool>,
	/// Whether selection is disabled.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	/// The content to display as the label.
	#[prop(optional)]
	label: Option<String>,
	/// The initial selection (uncontrolled).
	#[prop(optional)]
	default_selection: Option<leptos_aria_selection::Selection>,
	/// The current selection (controlled).
	#[prop(optional, into)]
	selection: Option<Signal<leptos_aria_selection::Selection>>,
	/// Handler that is called when the selection changes.
	#[prop(optional)]
	on_change: Option<leptos_aria_selection::BoxedSelectionCallback>,
	/// The focused option (controlled), for an owner such as a combo box
	/// whose input keeps DOM focus while the arrow keys move through the
	/// options.
	#[prop(optional)]
	focused_key: Option<RwSignal<Option<String>>>,
	/// Classes for the root element.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let state = use_selection_state(UseSelectionStateProps {
		default_selection,
		selection,
		mode: if is_multiple.get_untracked() {
			SelectionMode::Multiple
		} else {
			SelectionMode::Single
		},
		is_disabled: is_disabled.get_untracked(),
		on_change,
	});

	let context = ListBoxContext {
		state,
		keys: RwSignal::new(CollectionKeys::default()),
		focused_key: focused_key.unwrap_or_else(|| RwSignal::new(None)),
		id: use_id(),
		disabled_options: StoredValue::new(HashMap::new()),
	};

	provide_context(context.clone());

	let label_id = label.is_some().then(use_id);
	let listbox_id = context.id.clone();
	let active_context = context.clone();

	view! {
		{label.map(|label| view! { <span id=label_id.clone()>{label}</span> })}
		<div
			class=merge_class("", class)
			id=listbox_id.clone()
			role="listbox"
			aria-labelledby=label_id.clone()
			aria-activedescendant=move || active_context.active_descendant()
			aria-multiselectable=move || is_multiple.get().to_string()
			tabindex=0
			on:keydown=move |event: web_sys::KeyboardEvent| handle_key_down(&context, &event)
		>
			{children()}
		</div>
	}
}

/// The keyboard interaction of the list box.
fn handle_key_down(context: &ListBoxContext, event: &web_sys::KeyboardEvent) {
	match event.key().as_str() {
		"ArrowDown" => context.focus_next(),
		"ArrowUp" => context.focus_previous(),
		"Home" => context.focus_first(),
		"End" => context.focus_last(),
		"Enter" | " " => context.select(None),
		key if key.chars().count() == 1 => context.search_keys(key),
		_ => return,
	}

	event.prevent_default();
}

/// An option of a list box.
#[component]
pub fn ListBoxOption(
	/// The value of the option, reported in the selection.
	#[prop(into)]
	value: String,
	/// Whether this option is disabled. Keyboard navigation skips a disabled
	/// option, and neither a click nor enter selects it.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	/// Classes for the root element.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let context = expect_context::<ListBoxContext>();
	context.register_option(&value, is_disabled);

	let option_id = untrack(|| context.option_id(&value));
	let is_focused = Signal::derive({
		let focused = context.focused_key;
		let value = value.clone();

		move || focused.with(|focused| focused.as_deref() == Some(value.as_str()))
	});
	let is_selected = Signal::derive({
		let selection = context.state.selection_signal();
		let value = value.clone();

		move || selection.with(|selection| selection.contains(&value))
	});
	let option_value = value.clone();

	view! {
		<div
			class=merge_class("", class)
			id=option_id.clone()
			role="option"
			aria-selected=move || is_selected.get().to_string()
			aria-disabled=move || is_disabled.get().to_string()
			data-focused=move || is_focused.get().to_string()
			on:click={
				let context = context.clone();
				let value = option_value.clone();

				move |_| context.select(Some(&value))
			}
		>
			{children()}
		</div>
	}
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
	use leptos::mount::mount_to_body;
	use wasm_bindgen::JsCast;
	use wasm_bindgen_test::*;

	use super::*;

	wasm_bindgen_test_configure!(run_in_browser);

	#[wasm_bindgen_test]
	async fn options_select_exclusively_by_default() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! {
				<ListBox label="Fruits".to_string() class="list-box-test-exclusive">
					<ListBoxOption value="apple".to_string()>"Apple"</ListBoxOption>
					<ListBoxOption value="banana".to_string()>"Banana"</ListBoxOption>
				</ListBox>
			}
		});

		let options = document()
			.query_selector_all(".list-box-test-exclusive [role='option']")
			.unwrap();

		let first = options.get(0).unwrap().unchecked_into::<web_sys::Element>();
		let second = options.get(1).unwrap().unchecked_into::<web_sys::Element>();

		first.dispatch_event(&click_event()).unwrap();

		let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
		wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();

		assert_eq!(
			first.get_attribute("aria-selected").as_deref(),
			Some("true")
		);

		second.dispatch_event(&click_event()).unwrap();

		let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
		wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();

		assert_eq!(
			first.get_attribute("aria-selected").as_deref(),
			Some("false"),
			"a second click deselects the first option"
		);
		assert_eq!(
			second.get_attribute("aria-selected").as_deref(),
			Some("true")
		);
	}

	fn click_event() -> web_sys::Event {
		web_sys::Event::new("click").unwrap()
	}

	/// Dispatch a bubbling `keydown` for `key`.
	fn key_down(target: &web_sys::Element, key: &str) {
		let init = web_sys::KeyboardEventInit::new();
		init.set_key(key);
		init.set_bubbles(true);

		let event =
			web_sys::KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init).unwrap();

		target.dispatch_event(&event).unwrap();
	}

	#[wasm_bindgen_test]
	async fn the_focused_option_is_the_active_descendant() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! {
				<ListBox class="list-box-test-active">
					<ListBoxOption value="apple".to_string()>"Apple"</ListBoxOption>
					<ListBoxOption value="banana".to_string()>"Banana"</ListBoxOption>
				</ListBox>
			}
		});

		let listbox = document()
			.query_selector(".list-box-test-active")
			.unwrap()
			.unwrap();

		assert!(!listbox.has_attribute("aria-activedescendant"));
		assert!(
			!listbox.has_attribute("aria-labelledby"),
			"an unlabelled list box points at no label"
		);

		key_down(&listbox, "ArrowDown");
		key_down(&listbox, "ArrowDown");

		let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
		wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();

		let active = listbox
			.get_attribute("aria-activedescendant")
			.expect("the focused option is announced");
		let option = document().get_element_by_id(&active).unwrap();

		assert_eq!(option.text_content().as_deref(), Some("Banana"));
		assert_eq!(
			option.get_attribute("data-focused").as_deref(),
			Some("true")
		);
	}

	async fn tick() {
		let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
		wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
	}

	/// The text of the option the list box announces as focused.
	fn active_text(listbox: &web_sys::Element) -> Option<String> {
		let active = listbox.get_attribute("aria-activedescendant")?;

		document()
			.get_element_by_id(&active)
			.and_then(|option| option.text_content())
	}

	#[wasm_bindgen_test]
	async fn navigation_skips_disabled_options() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! {
				<ListBox class="list-box-test-skip">
					<ListBoxOption value="apple" is_disabled=true>"Apple"</ListBoxOption>
					<ListBoxOption value="banana">"Banana"</ListBoxOption>
					<ListBoxOption value="cherry" is_disabled=true>"Cherry"</ListBoxOption>
					<ListBoxOption value="date">"Date"</ListBoxOption>
					<ListBoxOption value="elderberry" is_disabled=true>"Elderberry"</ListBoxOption>
				</ListBox>
			}
		});

		let listbox = document()
			.query_selector(".list-box-test-skip")
			.unwrap()
			.unwrap();
		let steps = [
			("Home", "Banana", "home skips a leading disabled option"),
			("ArrowDown", "Date", "arrow down skips a disabled option"),
			(
				"ArrowDown",
				"Banana",
				"arrow down wraps past disabled options",
			),
			("ArrowUp", "Date", "arrow up wraps past disabled options"),
			("End", "Date", "end skips a trailing disabled option"),
			("c", "Date", "type-ahead ignores a disabled match"),
			("b", "Banana", "type-ahead finds an enabled match"),
		];

		for (key, expected, reason) in steps {
			key_down(&listbox, key);
			tick().await;

			assert_eq!(active_text(&listbox).as_deref(), Some(expected), "{reason}");
		}
	}

	#[wasm_bindgen_test]
	async fn disabled_options_cannot_be_selected() {
		console_error_panic_hook::set_once();

		let is_banana_disabled = RwSignal::new(false);

		mount_to_body(move || {
			view! {
				<ListBox class="list-box-test-disabled">
					<ListBoxOption value="apple" is_disabled=true>"Apple"</ListBoxOption>
					<ListBoxOption value="banana" is_disabled=is_banana_disabled>"Banana"</ListBoxOption>
				</ListBox>
			}
		});

		let listbox = document()
			.query_selector(".list-box-test-disabled")
			.unwrap()
			.unwrap();
		let options = listbox.query_selector_all("[role='option']").unwrap();
		let apple = options.get(0).unwrap().unchecked_into::<web_sys::Element>();
		let banana = options.get(1).unwrap().unchecked_into::<web_sys::Element>();

		apple.dispatch_event(&click_event()).unwrap();
		tick().await;

		assert_eq!(
			apple.get_attribute("aria-selected").as_deref(),
			Some("false"),
			"a click does not select a disabled option"
		);

		key_down(&listbox, "ArrowDown");
		tick().await;
		is_banana_disabled.set(true);
		key_down(&listbox, "Enter");
		tick().await;

		assert_eq!(active_text(&listbox).as_deref(), Some("Banana"));
		assert_eq!(
			banana.get_attribute("aria-selected").as_deref(),
			Some("false"),
			"enter does not select an option disabled after it was focused"
		);

		is_banana_disabled.set(false);
		key_down(&listbox, "Enter");
		tick().await;

		assert_eq!(
			banana.get_attribute("aria-selected").as_deref(),
			Some("true"),
			"re-enabling the option makes it selectable"
		);
	}
}
