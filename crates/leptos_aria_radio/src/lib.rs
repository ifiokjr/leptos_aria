#![doc(html_logo_url = "https://ifiokjr.github.io/leptos_aria/logo.svg")]
#![doc(html_favicon_url = "https://ifiokjr.github.io/leptos_aria/favicon.ico")]

use std::sync::Arc;

use leptos::prelude::*;
use leptos_aria_utils::merge_class;
use leptos_aria_utils::use_id;

/// A callback invoked when the selected value changes.
pub type BoxedRadioChangeCallback = Box<dyn Fn(Option<String>) + Send + Sync>;

/// State management for a radio group.
///
/// Manages the selected value and the last focused radio, the same shape as
/// the upstream `useRadioGroupState`.
#[derive(Clone)]
pub struct RadioGroupState {
	selected: SelectedValue,
	last_focused_value: ArcStored<Option<String>>,
	on_change: Option<Arc<dyn Fn(Option<String>) + Send + Sync>>,
	is_disabled: bool,
	is_read_only: bool,
	name: String,
}

#[derive(Clone, Copy)]
enum SelectedValue {
	Controlled(Signal<Option<String>>),
	Uncontrolled(RwSignal<Option<String>>),
}

type ArcStored<T> = Arc<std::sync::Mutex<T>>;

/// The options accepted by [`use_radio_group_state`].
#[derive(Default)]
pub struct UseRadioGroupStateProps {
	/// The initial value (uncontrolled).
	pub default_value: Option<String>,

	/// The current value (controlled).
	pub value: Option<Signal<Option<String>>>,

	/// Handler that is called when the value changes.
	pub on_change: Option<BoxedRadioChangeCallback>,

	/// Whether the whole group is disabled.
	pub is_disabled: bool,

	/// Whether the whole group is read only.
	pub is_read_only: bool,

	/// The name used when submitting an HTML form; one is generated when
	/// missing.
	pub name: Option<String>,
}

/// Provides state management for a radio group.
pub fn use_radio_group_state(props: UseRadioGroupStateProps) -> RadioGroupState {
	let selected = match props.value {
		Some(controlled) => SelectedValue::Controlled(controlled),
		None => SelectedValue::Uncontrolled(RwSignal::new(props.default_value.clone())),
	};

	let on_change = props.on_change.map(|callback| {
		let callback: Arc<dyn Fn(Option<String>) + Send + Sync> = Arc::new(callback);

		callback
	});

	RadioGroupState {
		selected,
		last_focused_value: Arc::new(std::sync::Mutex::new(None)),
		on_change,
		is_disabled: props.is_disabled,
		is_read_only: props.is_read_only,
		name: props
			.name
			.unwrap_or_else(|| format!("radio-group-{}", use_id())),
	}
}

impl RadioGroupState {
	/// The currently selected value.
	///
	/// Reactive: reading inside an effect or view tracks the selection.
	pub fn selected_value(&self) -> Option<String> {
		self.selected_signal().get()
	}

	/// The currently selected value, without tracking.
	pub fn selected_value_untracked(&self) -> Option<String> {
		self.selected_signal().get_untracked()
	}

	/// The signal backing the selection, for reactive bindings.
	pub fn selected_signal(&self) -> Signal<Option<String>> {
		match self.selected {
			SelectedValue::Controlled(signal) => signal,
			SelectedValue::Uncontrolled(signal) => Signal::from(signal),
		}
	}

	/// The name shared by the group's inputs.
	pub fn name(&self) -> String {
		self.name.clone()
	}

	/// Whether the group is disabled.
	pub fn is_disabled(&self) -> bool {
		self.is_disabled
	}

	/// Whether the group is read only.
	pub fn is_read_only(&self) -> bool {
		self.is_read_only
	}

	/// The value of the last focused radio.
	pub fn last_focused_value(&self) -> Option<String> {
		self.last_focused_value.lock().unwrap().clone()
	}

	/// Record the value of the last focused radio.
	pub fn set_last_focused_value(&self, value: Option<String>) {
		*self.last_focused_value.lock().unwrap() = value;
	}

	/// Update the selected value.
	pub fn set_selected_value(&self, value: Option<String>) {
		if self.is_disabled || self.is_read_only || self.selected_value_untracked() == value {
			return;
		}

		if let SelectedValue::Uncontrolled(signal) = self.selected {
			signal.set(value.clone());
		}

		if let Some(on_change) = &self.on_change {
			on_change(value);
		}
	}
}

/// The radio group state shared with its radios through context.
#[derive(Clone)]
pub struct RadioGroupContext {
	state: RadioGroupState,
}

/// A radio group allows users to select a single item from a list of
/// mutually exclusive options.
#[component]
pub fn RadioGroup(
	/// The content to display as the label.
	#[prop(optional)]
	label: Option<String>,
	/// The initial value (uncontrolled).
	#[prop(optional, into)]
	default_value: Option<String>,
	/// The current value (controlled).
	#[prop(optional, into)]
	value: Option<Signal<Option<String>>>,
	/// Handler that is called when the value changes.
	#[prop(optional)]
	on_change: Option<BoxedRadioChangeCallback>,
	/// Whether the whole group is disabled.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	/// Whether the whole group is read only.
	#[prop(optional, into)]
	is_read_only: Signal<bool>,
	/// The name used when submitting an HTML form.
	#[prop(optional, into)]
	name: Option<String>,
	/// Classes for the root element.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let state = use_radio_group_state(UseRadioGroupStateProps {
		default_value,
		value,
		on_change,
		is_disabled: is_disabled.get_untracked(),
		is_read_only: is_read_only.get_untracked(),
		name,
	});

	let label_id = use_id();

	provide_context(RadioGroupContext {
		state: state.clone(),
	});

	view! {
		<div
			role="radiogroup"
			class=merge_class("", class)
			aria-labelledby=label_id.clone()
			on:focusout=move |_| {
				// Reset the roving focus memory when focus leaves the group
				// without a selection.
				if state.selected_value_untracked().is_none() {
					state.set_last_focused_value(None);
				}
			}
		>
			{label.map(|label| view! { <span id=label_id.clone()>{label}</span> })}
			{children()}
		</div>
	}
}

/// An individual radio button in a radio group.
#[allow(clippy::needless_pass_by_value)]
#[component]
pub fn Radio(
	/// The value of the radio, used when submitting an HTML form.
	#[prop(into)]
	value: String,
	/// Whether this radio is disabled.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	/// Classes for the root element.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let group = expect_context::<RadioGroupContext>();
	let state = group.state.clone();

	let input_id = use_id();
	let checked = {
		let state = state.clone();
		let radio_value = value.clone();

		Signal::derive(move || state.selected_value().as_deref() == Some(radio_value.as_str()))
	};

	let group_disabled = state.is_disabled();
	let name = state.name();
	let focus_state = state.clone();
	let change_state = state.clone();
	let radio_value = value.clone();
	let change_value = value.clone();
	let focus_value = value.clone();

	view! {
		<label id=input_id.clone() class=merge_class("", class) on:mousedown=|event: web_sys::MouseEvent| {
			// Prevent the label from taking focus when pressed.
			event.prevent_default();
		}>
			<input
				type="radio"
				id=input_id.clone()
				name=name.clone()
				value=radio_value.clone()
				checked=move || checked.get()
				disabled=move || is_disabled.get() || group_disabled
				on:focus=move |_| focus_state.set_last_focused_value(Some(focus_value.clone()))
				on:change=move |event: web_sys::Event| {
					event.stop_propagation();
					change_state.set_selected_value(Some(change_value.clone()));
				}
			/>
			{children()}
		</label>
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
	async fn radios_are_exclusive_and_share_a_name() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! {
				<RadioGroup label="Favorite fruit".to_string()>
					<Radio value="apple".to_string()>"Apple"</Radio>
					<Radio value="banana".to_string()>"Banana"</Radio>
				</RadioGroup>
			}
		});

		let radios = document()
			.query_selector_all("[role='radiogroup'] input[type='radio']")
			.unwrap();
		let first = radios
			.get(0)
			.unwrap()
			.unchecked_into::<web_sys::HtmlInputElement>();
		let second = radios
			.get(1)
			.unwrap()
			.unchecked_into::<web_sys::HtmlInputElement>();

		assert_eq!(
			first.get_attribute("name").as_deref(),
			second.get_attribute("name").as_deref(),
			"radios in a group share a name"
		);

		first.click();

		let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
		wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();

		assert!(first.checked());

		second.click();

		let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
		wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();

		assert!(second.checked());
		assert!(!first.checked(), "selecting one radio deselects the other");
	}
}
