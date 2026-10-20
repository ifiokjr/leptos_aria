#![doc(html_logo_url = "https://ifiokjr.github.io/leptos_aria/logo.svg")]
#![doc(html_favicon_url = "https://ifiokjr.github.io/leptos_aria/favicon.ico")]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::derived_hash_with_manual_eq)]

use std::collections::HashSet;
use std::sync::Arc;

use leptos::prelude::*;
use leptos_aria_toggle::ToggleState;
use leptos_aria_utils::merge_class;
use leptos_aria_utils::use_id;
use wasm_bindgen::JsCast;

/// A callback invoked when the selection changes.
pub type BoxedSelectionChangeCallback = leptos_aria_toggle::BoxedSelectionChangeCallback;

/// A callback invoked when the selected values change.
pub type BoxedValuesChangeCallback = Box<dyn Fn(SelectedValues) + Send + Sync>;

/// The selected values of a checkbox group.
pub type SelectedValues = HashSet<String>;

/// State management for a checkbox group.
///
/// Manages the set of selected values, guarding disabled and read-only
/// groups, the same shape as the upstream `useCheckboxGroupState`.
#[derive(Clone)]
pub struct CheckboxGroupState {
	values: SelectedValue,
	on_change: Option<Arc<dyn Fn(SelectedValues) + Send + Sync>>,
	is_disabled: bool,
	is_read_only: bool,
}

#[derive(Clone, Copy)]
enum SelectedValue {
	Controlled(Signal<HashSet<String>>),
	Uncontrolled(RwSignal<HashSet<String>>),
}

/// The options accepted by [`use_checkbox_group_state`].
#[derive(Default)]
pub struct UseCheckboxGroupStateProps {
	/// The initial values (uncontrolled).
	pub default_value: Option<SelectedValues>,

	/// The current values (controlled).
	pub value: Option<Signal<SelectedValues>>,

	/// Handler that is called when the values change.
	pub on_change: Option<BoxedValuesChangeCallback>,

	/// Whether the whole group is disabled.
	pub is_disabled: bool,

	/// Whether the whole group is read only.
	pub is_read_only: bool,
}

/// Provides state management for a checkbox group.
#[allow(clippy::needless_pass_by_value)]
pub fn use_checkbox_group_state(props: UseCheckboxGroupStateProps) -> CheckboxGroupState {
	let values = match props.value {
		Some(controlled) => SelectedValue::Controlled(controlled),
		None => SelectedValue::Uncontrolled(RwSignal::new(props.default_value.unwrap_or_default())),
	};

	let on_change = props.on_change.map(|callback| {
		let callback: Arc<dyn Fn(HashSet<String>) + Send + Sync> = Arc::new(callback);

		callback
	});

	CheckboxGroupState {
		values,
		on_change,
		is_disabled: props.is_disabled,
		is_read_only: props.is_read_only,
	}
}

impl CheckboxGroupState {
	/// The currently selected values.
	///
	/// Reactive: reading inside an effect or view tracks the selection.
	pub fn value(&self) -> HashSet<String> {
		self.values_signal().get()
	}

	/// The currently selected values, without tracking.
	pub fn value_untracked(&self) -> HashSet<String> {
		self.values_signal().get_untracked()
	}

	/// The signal backing the values, for reactive bindings.
	pub fn values_signal(&self) -> Signal<SelectedValues> {
		match &self.values {
			SelectedValue::Controlled(signal) => *signal,
			SelectedValue::Uncontrolled(signal) => Signal::from(*signal),
		}
	}

	/// Whether the group is disabled.
	pub fn is_disabled(&self) -> bool {
		self.is_disabled
	}

	/// Whether the group is read only.
	pub fn is_read_only(&self) -> bool {
		self.is_read_only
	}

	/// Replace the selected values.
	pub fn set_value(&self, values: SelectedValues) {
		if self.is_disabled
			|| self.is_read_only
			|| self
				.values_signal()
				.with_untracked(|current| *current == values)
		{
			return;
		}

		if let SelectedValue::Uncontrolled(signal) = &self.values {
			signal.set(values.clone());
		}

		if let Some(on_change) = &self.on_change {
			on_change(values);
		}
	}

	/// Add or remove a value from the selection.
	pub fn toggle_value(&self, value: &str) {
		let mut values = self.value_untracked();

		if values.contains(value) {
			values.remove(value);
		} else {
			values.insert(value.to_string());
		}

		self.set_value(values);
	}
}

/// A checkbox allows users to select multiple items from a list of
/// individual items, or to mark one individual item as selected.
#[component]
pub fn Checkbox(
	/// The content to display as the label.
	children: Children,
	/// The value of the checkbox inside a group.
	#[prop(optional, into)]
	value: Option<String>,
	/// Whether the checkbox should be selected (uncontrolled).
	#[prop(optional, into)]
	default_selected: Signal<bool>,
	/// Whether the checkbox should be selected (controlled).
	#[prop(optional, into)]
	is_selected: Option<Signal<bool>>,
	/// Handler that is called when the selection state changes.
	#[prop(optional)]
	on_change: Option<BoxedSelectionChangeCallback>,
	/// Whether the checkbox is disabled.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	/// Whether the checkbox is read only.
	#[prop(optional, into)]
	is_read_only: Signal<bool>,
	/// Whether the checkbox is indeterminate.
	///
	/// Indeterminism is presentational only: the visual representation remains
	/// regardless of user interaction.
	#[prop(optional, into)]
	is_indeterminate: Signal<bool>,
	/// Classes for the root element.
	#[prop(optional, into)]
	class: Signal<String>,
) -> impl IntoView {
	let input_id = use_id();
	let group = use_context::<CheckboxGroupContext>();

	let state = toggle_state(
		group.as_ref(),
		value.as_deref(),
		default_selected,
		is_selected,
		on_change,
	);
	let is_selected = state.selected_signal();

	let group_disabled = group
		.as_ref()
		.is_some_and(|group| group.state.is_disabled());
	let group_read_only = group
		.as_ref()
		.is_some_and(|group| group.state.is_read_only());

	let (name, checked) = group_props(group.as_ref(), value.as_deref(), &is_selected);

	view! {
		<label id=input_id.clone() class=merge_class("", class) on:mousedown=|event: web_sys::MouseEvent| {
			// Prevent the label from taking focus when pressed; the input
			// still receives focus on click.
			event.prevent_default();
		}>
			<input
				type="checkbox"
				name=move || name.clone()
				checked=move || checked.get()
				disabled=move || is_disabled.get() || group_disabled
				readonly=move || is_read_only.get() || group_read_only
				prop:indeterminate=move || is_indeterminate.get()
				on:click=move |event: web_sys::MouseEvent| {
					// The input already toggled itself; stop the group-level
					// handler from double processing.
					event.stop_propagation();
				}
				on:change=move |event: web_sys::Event| {
					let Some(input) = event
						.target()
						.and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
					else {
						return;
					};

					if let Some(group) = &group {
						if let Some(value) = &value {
							group.state.toggle_value(value);
							return;
						}
					}

					state.set_selected(input.checked());
				}
			/>
			{children()}
		</label>
	}
}

/// The checkbox group state shared with its checkboxes through context.
#[derive(Clone)]
pub struct CheckboxGroupContext {
	state: CheckboxGroupState,
}

/// A checkbox group allows users to select multiple items from a list of
/// individual items.
#[component]
pub fn CheckboxGroup(
	/// Whether the whole group is disabled.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	/// Whether the whole group is read only.
	#[prop(optional, into)]
	is_read_only: Signal<bool>,
	/// The initial values (uncontrolled).
	#[prop(optional)]
	default_value: Option<SelectedValues>,
	/// The current values (controlled).
	#[prop(optional, into)]
	value: Option<Signal<SelectedValues>>,
	/// Handler that is called when the values change.
	#[prop(optional)]
	on_change: Option<BoxedValuesChangeCallback>,
	/// The content to display as the group label.
	#[prop(optional)]
	label: Option<String>,
	/// Classes for the root element.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let state = use_checkbox_group_state(UseCheckboxGroupStateProps {
		default_value,
		value,
		on_change,
		is_disabled: is_disabled.get_untracked(),
		is_read_only: is_read_only.get_untracked(),
	});

	let label_id = use_id();

	provide_context(CheckboxGroupContext {
		state: state.clone(),
	});

	let values = state.values_signal();

	view! {
		<div role="group" class=merge_class("", class) aria-labelledby=label_id.clone()>
			{label.map(|label| {
				view! { <span id=label_id.clone()>{label}</span> }
			})}
			{children()}
			<span class="leptos-aria-checkbox-group__values" aria-live="polite">
				{move || values.with(HashSet::len).to_string() + " selected" }
			</span>
		</div>
	}
}

/// Resolve the toggle state for a checkbox, standalone or inside a group.
///
/// A grouped checkbox is a read-only view of the group selection: its input
/// routes changes to the group state directly.
fn toggle_state(
	group: Option<&CheckboxGroupContext>,
	value: Option<&str>,
	default_selected: Signal<bool>,
	is_selected: Option<Signal<bool>>,
	on_change: Option<BoxedSelectionChangeCallback>,
) -> ToggleState {
	if let (Some(group), Some(value)) = (group, value) {
		let group_state = group.state.clone();
		let value = value.to_string();
		let selected = Signal::derive(move || {
			group_state
				.values_signal()
				.with(|values| values.contains(&value))
		});

		return leptos_aria_toggle::use_toggle_state(leptos_aria_toggle::UseToggleStateProps {
			is_selected: Some(selected),
			..Default::default()
		});
	}

	leptos_aria_toggle::use_toggle_state(leptos_aria_toggle::UseToggleStateProps {
		default_selected: Some(default_selected.get_untracked()),
		is_selected,
		on_change,
		..Default::default()
	})
}

/// The name and checked binding of a checkbox inside (or outside) a group.
#[allow(clippy::needless_pass_by_value)]
fn group_props(
	group: Option<&CheckboxGroupContext>,
	value: Option<&str>,
	is_selected: &Signal<bool>,
) -> (Option<String>, Signal<bool>) {
	match (group, value) {
		(Some(group), Some(value)) => {
			let group_state = group.state.clone();
			let value = value.to_string();
			let checked = {
				let group_state = group_state.clone();
				let value = value.clone();

				Signal::derive(move || {
					group_state
						.values_signal()
						.with(|values| values.contains(&value))
				})
			};

			(Some(value), checked)
		}
		_ => (None, *is_selected),
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
	async fn standalone_checkbox_toggles() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! {
				<Checkbox default_selected=Signal::from(false)>
					"Enable notifications"
				</Checkbox>
			}
		});

		let input = document()
			.query_selector("input[type='checkbox']")
			.unwrap()
			.unwrap()
			.unchecked_into::<web_sys::HtmlInputElement>();

		assert!(!input.checked());

		input.click();

		let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
		wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();

		assert!(input.checked(), "a click toggles the checkbox");
	}

	#[wasm_bindgen_test]
	async fn group_checkbox_updates_the_shared_state() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! {
				<CheckboxGroup>
					<Checkbox value="a".to_string() default_selected=Signal::from(false)>
						"Option A"
					</Checkbox>
					<Checkbox value="b".to_string() default_selected=Signal::from(false)>
						"Option B"
					</Checkbox>
				</CheckboxGroup>
			}
		});

		let checkboxes = document()
			.query_selector_all("[role='group'] input[type='checkbox']")
			.unwrap();
		let first = checkboxes
			.get(0)
			.unwrap()
			.unchecked_into::<web_sys::HtmlInputElement>();
		let second = checkboxes
			.get(1)
			.unwrap()
			.unchecked_into::<web_sys::HtmlInputElement>();

		first.click();

		let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
		wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();

		assert!(first.checked());
		assert!(!second.checked());
	}
}
