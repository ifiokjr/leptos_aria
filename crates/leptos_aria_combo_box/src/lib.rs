#![doc(html_logo_url = "https://ifiokjr.github.io/leptos_aria/logo.svg")]
#![doc(html_favicon_url = "https://ifiokjr.github.io/leptos_aria/favicon.ico")]
#![allow(clippy::needless_pass_by_value)]

use std::sync::Arc;

use leptos::prelude::*;
use leptos_aria_list_box::ListBox;
use leptos_aria_list_box::ListBoxOption;
use leptos_aria_selection::Selection;
use leptos_aria_selection::SelectionMode;
use leptos_aria_selection::UseSelectionStateProps;
use leptos_aria_selection::use_selection_state;
use wasm_bindgen::JsCast;

/// One option of a combo box: a value with its text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComboBoxItem {
	pub value: String,
	pub label: String,
}

/// A selection signal, the reactive read of the shared selection state.
pub type SelectionSignal = Signal<Selection>;

/// The shared state of one combo box, split from the component so both the
/// list and the input handlers can hold a clone.
type SharedComboBoxState = Arc<ComboBoxState>;

/// The shared wiring of a combo box: the open state, the selection, and the
/// filtered options derived from the input value.
struct ComboBoxState {
	is_open: RwSignal<bool>,
	local_value: RwSignal<String>,
	#[allow(dead_code)]
	selection: leptos_aria_selection::SelectionState,
	filtered: Memo<Vec<ComboBoxItem>>,
	selected_label: Memo<Option<String>>,
}

impl ComboBoxState {
	fn new(
		options: Arc<Vec<ComboBoxItem>>,
		initial_value: String,
		input_value: Signal<String>,
	) -> Self {
		let is_open = RwSignal::new(false);
		let local_value = RwSignal::new(initial_value);
		let selection = use_selection_state(UseSelectionStateProps {
			mode: SelectionMode::Single,
			..Default::default()
		});

		let filtered = filter_options(&options, input_value);
		let selected_label = resolve_selected_label(&options, selection.selection_signal());

		Self {
			is_open,
			local_value,
			selection,
			filtered,
			selected_label,
		}
	}
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
) -> impl IntoView {
	let initial_value = default_value.unwrap_or_default();
	let input_value = value.unwrap_or_else(|| Signal::from(RwSignal::new(initial_value.clone())));
	let input_value: Signal<String> = input_value;

	let state = ComboBoxState::new(Arc::new(options), initial_value, input_value);
	let listbox_id = leptos_aria_utils::use_id();

	let state: SharedComboBoxState = Arc::new(state);
	let list_state = Arc::clone(&state);
	let list = move || render_options(&list_state);

	let handlers = InputHandlers {
		state: Arc::clone(&state),
		on_change: Arc::new(on_change),
	};
	let input_handlers = handlers;
	let key_handlers = input_handlers.clone();

	view! {
		<input
			type="text"
			role="combobox"
			aria-expanded=move || state.is_open.get().to_string()
			aria-controls=listbox_id.clone()
			aria-autocomplete="list"
			aria-haspopup="listbox"
			disabled=move || is_disabled.get()
			prop:value=move || input_value.get()
			on:input=move |event: web_sys::Event| input_handlers.handle_input(event)
			on:keydown=move |event: web_sys::KeyboardEvent| key_handlers.handle_key_down(event)
		/>
		<ListBox label=label.unwrap_or_default()>{list}</ListBox>
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

/// The input event handlers, split from the component so the wiring reads at
/// a glance.
#[derive(Clone)]
struct InputHandlers {
	state: SharedComboBoxState,
	on_change: SharedChangeCallback,
}

/// The change callback shared by the input handlers.
type SharedChangeCallback = Arc<Option<BoxedComboChangeCallback>>;

/// A callback invoked when the combo box value changes.
pub type BoxedComboChangeCallback = Box<dyn Fn(&str) + Send + Sync>;

impl InputHandlers {
	fn handle_input(&self, event: web_sys::Event) {
		let Some(typed) = typed_value(event) else {
			return;
		};

		self.state.local_value.set(typed);
		self.state.is_open.set(true);

		if let Some(on_change) = self.on_change.as_ref() {
			on_change(&self.state.local_value.get_untracked());
		}
	}

	fn handle_key_down(&self, event: web_sys::KeyboardEvent) {
		match event.key().as_str() {
			"ArrowDown" => self.state.is_open.set(true),
			"Escape" => self.state.is_open.set(false),
			"Enter" => self.commit_selection(),
			_ => {}
		}
	}

	/// Move the selected option's label into the input and close the list.
	fn commit_selection(&self) {
		if let Some(label) = self.state.selected_label.get_untracked() {
			self.state.local_value.set(label);
		}

		self.state.is_open.set(false);
	}
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
	use leptos::mount::mount_to_body;
	use wasm_bindgen::JsCast;
	use wasm_bindgen_test::*;

	use super::*;

	wasm_bindgen_test_configure!(run_in_browser);

	#[wasm_bindgen_test]
	async fn combo_box_renders_the_combobox_role() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! {
				<ComboBox
					label="Fruit".to_string()
					options=vec![
						ComboBoxItem { value: "apple".into(), label: "Apple".into() },
						ComboBoxItem { value: "banana".into(), label: "Banana".into() },
					]
				/>
			}
		});

		let input = document()
			.query_selector("[role='combobox']")
			.unwrap()
			.unwrap()
			.unchecked_into::<web_sys::HtmlInputElement>();

		assert_eq!(
			input.get_attribute("aria-autocomplete").as_deref(),
			Some("list")
		);
	}
}
