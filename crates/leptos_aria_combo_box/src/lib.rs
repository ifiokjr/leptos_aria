#![doc(html_logo_url = "https://ifiokjr.github.io/leptos_aria/logo.svg")]
#![doc(html_favicon_url = "https://ifiokjr.github.io/leptos_aria/favicon.ico")]
#![allow(clippy::needless_pass_by_value)]

use std::sync::Arc;

use leptos::prelude::*;
use leptos_aria_list_box::ListBox;
use leptos_aria_list_box::ListBoxOption;
use leptos_aria_selection::Selection;
use leptos_aria_selection::SelectionMode;
use leptos_aria_selection::SelectionState;
use leptos_aria_selection::UseSelectionStateProps;
use leptos_aria_selection::use_selection_state;
use leptos_aria_utils::merge_class;
use wasm_bindgen::JsCast;

/// One option of a combo box: a value with its text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComboBoxItem {
	pub value: String,
	pub label: String,
}

/// A selection signal, the reactive read of the shared selection state.
pub type SelectionSignal = Signal<Selection>;

/// A callback invoked when the combo box value changes.
pub type BoxedComboChangeCallback = Box<dyn Fn(&str) + Send + Sync>;

/// The shared state of one combo box, split from the component so both the
/// list and the input handlers can hold a clone.
type SharedComboBoxState = Arc<ComboBoxState>;

/// The text of the input: owned by the caller, or by the combo box itself.
#[derive(Clone, Copy)]
enum InputValue {
	Controlled(Signal<String>),
	Uncontrolled(RwSignal<String>),
}

impl InputValue {
	/// The signal every read of the input text goes through.
	fn signal(self) -> Signal<String> {
		match self {
			InputValue::Controlled(signal) => signal,
			InputValue::Uncontrolled(signal) => Signal::from(signal),
		}
	}
}

/// The shared wiring of a combo box: the open state, the input text, the
/// focused and selected options, and the options filtered by the input text.
struct ComboBoxState {
	is_open: RwSignal<bool>,
	input_value: InputValue,
	focused_key: RwSignal<Option<String>>,
	selection: SelectionState,
	filtered: Memo<Vec<ComboBoxItem>>,
	selected_label: Memo<Option<String>>,
	on_change: Option<BoxedComboChangeCallback>,
}

impl ComboBoxState {
	fn new(
		options: Vec<ComboBoxItem>,
		input_value: InputValue,
		on_change: Option<BoxedComboChangeCallback>,
	) -> Self {
		let options = Arc::new(options);
		let selection = use_selection_state(UseSelectionStateProps {
			mode: SelectionMode::Single,
			..Default::default()
		});

		Self {
			is_open: RwSignal::new(false),
			input_value,
			focused_key: RwSignal::new(None),
			filtered: filter_options(&options, input_value.signal()),
			selected_label: resolve_selected_label(&options, selection.selection_signal()),
			selection,
			on_change,
		}
	}

	/// Update the input text.
	///
	/// A controlled value reports upward and lets the owner decide; an
	/// uncontrolled one updates itself.
	fn set_input_value(&self, value: String) {
		if self
			.input_value
			.signal()
			.with_untracked(|current| *current == value)
		{
			return;
		}

		if let InputValue::Uncontrolled(signal) = self.input_value {
			signal.set(value.clone());
		}

		if let Some(on_change) = &self.on_change {
			on_change(&value);
		}
	}

	fn handle_input(&self, event: web_sys::Event) {
		let Some(typed) = typed_value(event) else {
			return;
		};

		self.set_input_value(typed);
		self.focused_key.set(None);
		self.is_open.set(true);
	}

	fn handle_key_down(&self, event: web_sys::KeyboardEvent) {
		match event.key().as_str() {
			"ArrowDown" => self.focus_adjacent(FocusDirection::Next),
			"ArrowUp" => self.focus_adjacent(FocusDirection::Previous),
			"Enter" => self.commit_focused(),
			"Escape" => self.close(),
			_ => return,
		}

		event.prevent_default();
	}

	/// Open the list and move focus to the visible option next to the focused
	/// one, wrapping at either end.
	fn focus_adjacent(&self, direction: FocusDirection) {
		let next = self.filtered.with_untracked(|items| {
			self.focused_key.with_untracked(|focused| {
				let focused = focused.as_deref();

				match direction {
					FocusDirection::Next => adjacent_key(items.iter(), focused),
					FocusDirection::Previous => adjacent_key(items.iter().rev(), focused),
				}
			})
		});

		self.is_open.set(true);
		self.focused_key.set(next);
	}

	/// Select the focused option when it is still visible, then commit the
	/// selection into the input.
	fn commit_focused(&self) {
		let focused = self.focused_key.get_untracked().filter(|key| {
			self.filtered
				.with_untracked(|items| items.iter().any(|item| &item.value == key))
		});

		if let Some(key) = focused {
			self.selection.select_key(&key, false);
		}

		self.commit_selection();
	}

	/// Move the selected option's label into the input and close the list.
	fn commit_selection(&self) {
		if let Some(label) = self.selected_label.get_untracked() {
			self.set_input_value(label);
		}

		self.close();
	}

	fn close(&self) {
		self.focused_key.set(None);
		self.is_open.set(false);
	}
}

/// Which way the arrow keys move focus through the options.
#[derive(Clone, Copy)]
enum FocusDirection {
	Next,
	Previous,
}

/// The key after `focused` in `items`, or the first key when nothing is
/// focused, the focused key is hidden, or focus is on the last key.
fn adjacent_key<'a>(
	items: impl Iterator<Item = &'a ComboBoxItem> + Clone,
	focused: Option<&str>,
) -> Option<String> {
	let first = items.clone().next();

	items
		.skip_while(|item| Some(item.value.as_str()) != focused)
		.nth(1)
		.or(first)
		.map(|item| item.value.clone())
}

/// The options whose labels contain the current input value.
fn filter_options(
	options: &Arc<Vec<ComboBoxItem>>,
	value: Signal<String>,
) -> Memo<Vec<ComboBoxItem>> {
	let options = Arc::clone(options);

	Memo::new(move |_| {
		let query = value.with(|value| value.to_lowercase());

		options
			.iter()
			.filter(|item| item.label.to_lowercase().contains(&query))
			.cloned()
			.collect()
	})
}

/// The label of the selected option, if any.
fn resolve_selected_label(
	options: &Arc<Vec<ComboBoxItem>>,
	selection: Signal<Selection>,
) -> Memo<Option<String>> {
	let options = Arc::clone(options);

	Memo::new(move |_| {
		selection.with(|selection| {
			match selection {
				Selection::Keys(keys) => {
					keys.iter()
						.next()
						.and_then(|key| options.iter().find(|item| &item.value == key))
						.map(|item| item.label.clone())
				}
				_ => None,
			}
		})
	})
}

/// A combo box combines a text input with a list box of suggestions.
///
/// Typing filters the options, the arrow keys move through them, enter
/// chooses the focused option into the input, and escape closes the list.
#[component]
pub fn ComboBox(
	/// The content to display as the label.
	#[prop(optional)]
	label: Option<String>,
	/// The available options.
	#[prop(into)]
	options: Vec<ComboBoxItem>,
	/// The current input value (controlled).
	#[prop(optional, into)]
	value: Option<Signal<String>>,
	/// The initial input value (uncontrolled).
	#[prop(optional, into)]
	default_value: Option<String>,
	/// Handler that is called when the input value changes.
	#[prop(optional)]
	on_change: Option<BoxedComboChangeCallback>,
	/// Whether the combo box is disabled.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	/// Classes added after the built-in `leptos-aria-combo-box` class.
	#[prop(optional, into)]
	class: Signal<String>,
) -> impl IntoView {
	let input_value = match value {
		Some(controlled) => InputValue::Controlled(controlled),
		None => InputValue::Uncontrolled(RwSignal::new(default_value.unwrap_or_default())),
	};
	let state: SharedComboBoxState = Arc::new(ComboBoxState::new(options, input_value, on_change));
	let listbox_id = leptos_aria_utils::use_id();

	let list_state = Arc::clone(&state);
	let list = move || render_options(&list_state);
	let on_selection_change: leptos_aria_selection::BoxedSelectionCallback = Box::new({
		let state = Arc::clone(&state);

		move |selection: &Selection| {
			state.selection.set_selection(selection.clone());
			state.commit_selection();
		}
	});
	let is_open = state.is_open;
	let selection = state.selection.selection_signal();
	let focused_key = state.focused_key;
	let input_state = Arc::clone(&state);
	let key_state = state;

	view! {
		<div class=merge_class("leptos-aria-combo-box", class)>
			<input
				type="text"
				role="combobox"
				aria-expanded=move || is_open.get().to_string()
				aria-controls=listbox_id.clone()
				aria-autocomplete="list"
				aria-haspopup="listbox"
				disabled=move || is_disabled.get()
				prop:value=move || input_value.signal().get()
				on:input=move |event: web_sys::Event| input_state.handle_input(event)
				on:keydown=move |event: web_sys::KeyboardEvent| key_state.handle_key_down(event)
			/>
			<ListBox
				label=label.unwrap_or_default()
				selection=selection
				on_change=on_selection_change
				focused_key=focused_key
			>
				{list}
			</ListBox>
		</div>
	}
}

/// Render the filtered options as list box options.
fn render_options(state: &SharedComboBoxState) -> Vec<AnyView> {
	state
		.filtered
		.get()
		.into_iter()
		.map(|item| {
			view! {
				<ListBoxOption value=item.value>{item.label}</ListBoxOption>
			}
			.into_any()
		})
		.collect()
}

/// The text typed into an input event.
fn typed_value(event: web_sys::Event) -> Option<String> {
	event
		.target()
		.and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
		.map(|input| input.value())
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
	use leptos::mount::mount_to;
	use wasm_bindgen::JsCast;
	use wasm_bindgen_test::*;

	use super::*;

	wasm_bindgen_test_configure!(run_in_browser);

	#[wasm_bindgen_test]
	async fn combo_box_renders_the_combobox_role() {
		let fixture = Fixture::mount();

		assert_eq!(
			fixture.input.get_attribute("aria-autocomplete").as_deref(),
			Some("list")
		);
	}

	#[wasm_bindgen_test]
	async fn class_is_added_after_the_base_class() {
		console_error_panic_hook::set_once();

		let container = document()
			.create_element("div")
			.unwrap()
			.unchecked_into::<web_sys::HtmlElement>();
		document().body().unwrap().append_child(&container).unwrap();

		let class = RwSignal::new(String::new());
		let _handle = mount_to(container.clone(), move || {
			view! { <ComboBox options=Vec::new() class=class /> }
		});
		let root = container.first_element_child().unwrap();
		assert_eq!(root.class_name(), "leptos-aria-combo-box");

		class.set("  fruit-picker ".to_string());
		tick().await;
		assert_eq!(root.class_name(), "leptos-aria-combo-box fruit-picker");

		container.remove();
	}

	#[wasm_bindgen_test]
	async fn typing_filters_the_options_when_uncontrolled() {
		let fixture = Fixture::mount();
		assert_eq!(fixture.option_labels(), ["Apple", "Banana", "Blueberry"]);

		fixture.type_text("b").await;
		assert_eq!(fixture.option_labels(), ["Banana", "Blueberry"]);
		assert_eq!(fixture.input.value(), "b");
		assert_eq!(
			fixture.input.get_attribute("aria-expanded").as_deref(),
			Some("true")
		);

		fixture.type_text("blue").await;
		assert_eq!(fixture.option_labels(), ["Blueberry"]);
	}

	#[wasm_bindgen_test]
	async fn arrow_and_enter_commit_the_focused_option() {
		let fixture = Fixture::mount();

		fixture.type_text("b").await;
		fixture.press("ArrowDown").await;
		fixture.press("ArrowDown").await;
		assert_eq!(fixture.focused_label().as_deref(), Some("Blueberry"));

		fixture.press("Enter").await;
		assert_eq!(fixture.input.value(), "Blueberry");
		assert_eq!(fixture.selected_labels(), ["Blueberry"]);
		assert_eq!(
			fixture.input.get_attribute("aria-expanded").as_deref(),
			Some("false")
		);
	}

	#[wasm_bindgen_test]
	async fn arrow_up_wraps_to_the_last_visible_option() {
		let fixture = Fixture::mount();

		fixture.press("ArrowUp").await;
		assert_eq!(fixture.focused_label().as_deref(), Some("Blueberry"));

		fixture.press("ArrowDown").await;
		assert_eq!(fixture.focused_label().as_deref(), Some("Apple"));
	}

	#[wasm_bindgen_test]
	async fn clicking_an_option_commits_it() {
		let fixture = Fixture::mount();

		fixture
			.options()
			.into_iter()
			.find(|option| option.text_content().as_deref() == Some("Banana"))
			.unwrap()
			.dispatch_event(&web_sys::Event::new("click").unwrap())
			.unwrap();
		tick().await;

		assert_eq!(fixture.input.value(), "Banana");
		assert_eq!(fixture.selected_labels(), ["Banana"]);
	}

	/// An uncontrolled combo box mounted into its own container, so tests in
	/// the same page never query each other's elements.
	struct Fixture {
		container: web_sys::HtmlElement,
		input: web_sys::HtmlInputElement,
		_handle: Box<dyn std::any::Any>,
	}

	impl Fixture {
		fn mount() -> Self {
			console_error_panic_hook::set_once();

			let container = document()
				.create_element("div")
				.unwrap()
				.unchecked_into::<web_sys::HtmlElement>();
			document().body().unwrap().append_child(&container).unwrap();

			let handle = mount_to(container.clone(), || {
				view! {
					<ComboBox
						label="Fruit".to_string()
						options=vec![
							ComboBoxItem { value: "apple".into(), label: "Apple".into() },
							ComboBoxItem { value: "banana".into(), label: "Banana".into() },
							ComboBoxItem { value: "blueberry".into(), label: "Blueberry".into() },
						]
					/>
				}
			});

			let input = container
				.query_selector("[role='combobox']")
				.unwrap()
				.unwrap()
				.unchecked_into::<web_sys::HtmlInputElement>();

			Self {
				container,
				input,
				_handle: Box::new(handle),
			}
		}

		async fn type_text(&self, text: &str) {
			self.input.set_value(text);
			self.input
				.dispatch_event(&web_sys::Event::new("input").unwrap())
				.unwrap();
			tick().await;
		}

		async fn press(&self, key: &str) {
			let init = web_sys::KeyboardEventInit::new();
			init.set_key(key);
			init.set_bubbles(true);
			init.set_cancelable(true);

			let event = web_sys::KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init)
				.unwrap();
			self.input.dispatch_event(&event).unwrap();
			tick().await;
		}

		fn options(&self) -> Vec<web_sys::Element> {
			let options = self
				.container
				.query_selector_all("[role='option']")
				.unwrap();

			(0..options.length())
				.filter_map(|index| options.get(index))
				.map(JsCast::unchecked_into::<web_sys::Element>)
				.collect()
		}

		fn labels_where(&self, attribute: &str) -> Vec<String> {
			self.options()
				.into_iter()
				.filter(|option| option.get_attribute(attribute).as_deref() == Some("true"))
				.filter_map(|option| option.text_content())
				.collect()
		}

		fn option_labels(&self) -> Vec<String> {
			self.options()
				.into_iter()
				.filter_map(|option| option.text_content())
				.collect()
		}

		fn focused_label(&self) -> Option<String> {
			self.labels_where("data-focused").into_iter().next()
		}

		fn selected_labels(&self) -> Vec<String> {
			self.labels_where("aria-selected")
		}
	}

	impl Drop for Fixture {
		fn drop(&mut self) {
			self.container.remove();
		}
	}

	/// Let the reactive system flush queued effects into the DOM.
	async fn tick() {
		let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
		wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
	}
}
