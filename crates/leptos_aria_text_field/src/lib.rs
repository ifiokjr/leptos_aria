#![doc(html_logo_url = "https://ifiokjr.github.io/leptos_aria/logo.svg")]
#![doc(html_favicon_url = "https://ifiokjr.github.io/leptos_aria/favicon.ico")]
#![allow(clippy::needless_pass_by_value)]

use std::rc::Rc;

use leptos::prelude::*;
use leptos_aria_utils::UseLabelsProps;
use leptos_aria_utils::use_labels;
use wasm_bindgen::JsCast;

/// A callback invoked when the value changes.
pub type BoxedTextChangeCallback = Box<dyn Fn(&str)>;

/// State management for a text field's value.
///
/// The value can be controlled (a signal driven by the owner) or uncontrolled
/// (an internal signal), the same shape as the upstream
/// `useControlledState`.
/// The change callback storage of a text field state.
type TextFieldChangeCallback = Rc<dyn Fn(&str)>;

#[derive(Clone)]
pub struct TextFieldState {
	value: TextValue,
	default_value: String,
	on_change: Option<TextFieldChangeCallback>,
}

#[derive(Clone)]
enum TextValue {
	Controlled(Signal<String>),
	Uncontrolled(RwSignal<String>),
}

/// The options accepted by [`use_text_field_state`].
#[derive(Default)]
pub struct UseTextFieldStateProps {
	/// The initial value (uncontrolled).
	pub default_value: Option<String>,

	/// The current value (controlled).
	pub value: Option<Signal<String>>,

	/// Handler that is called when the value changes.
	pub on_change: Option<BoxedTextChangeCallback>,
}

/// Provides state management for a text field's value.
#[allow(clippy::needless_pass_by_value)]
pub fn use_text_field_state(props: UseTextFieldStateProps) -> TextFieldState {
	let default_value = props.default_value.unwrap_or_default();

	let value = match props.value {
		Some(controlled) => TextValue::Controlled(controlled),
		None => TextValue::Uncontrolled(RwSignal::new(default_value.clone())),
	};

	let on_change = props.on_change.map(|callback| {
		let callback: Rc<dyn Fn(&str)> = Rc::new(move |value: &str| callback(value));

		callback
	});

	TextFieldState {
		value,
		default_value,
		on_change,
	}
}

impl TextFieldState {
	/// The current value.
	pub fn value(&self) -> String {
		self.value_signal().get_untracked()
	}

	/// The signal backing the value, for reactive bindings.
	pub fn value_signal(&self) -> Signal<String> {
		match &self.value {
			TextValue::Controlled(signal) => *signal,
			TextValue::Uncontrolled(signal) => Signal::from(*signal),
		}
	}

	/// The value the field resets to.
	pub fn default_value(&self) -> String {
		self.default_value.clone()
	}

	/// Update the value.
	///
	/// A controlled state reports upward and lets the owner decide; an
	/// uncontrolled one updates itself.
	pub fn set_value(&self, value: impl Into<String>) {
		let value = value.into();

		if self.value() == value {
			return;
		}

		if let TextValue::Uncontrolled(signal) = &self.value {
			signal.set(value.clone());
		}

		if let Some(on_change) = &self.on_change {
			on_change(&value);
		}
	}
}

/// The shared labeling and description wiring of the text fields.
pub struct FieldAria {
	pub label_id: String,
	pub field_id: String,
	pub aria_labelledby: Option<String>,
}

/// Resolve the label and field wiring for a labeled field.
#[allow(clippy::option_option)]
pub fn field_aria(label: Option<&str>, aria_label: Option<&str>) -> FieldAria {
	let labels = use_labels(UseLabelsProps {
		label: label.map(str::to_string).filter(|label| !label.is_empty()),
		aria_label: aria_label.map(str::to_string),
		..Default::default()
	});

	FieldAria {
		label_id: labels.label_props.id,
		field_id: labels.field_props.id.clone(),
		aria_labelledby: labels.field_props.aria_labelledby,
	}
}

/// The props shared by [`TextField`] and [`TextArea`].
#[doc(hidden)]
pub struct TextInputBase {
	pub label: Option<String>,
	pub placeholder: Option<String>,
	pub name: Option<String>,
	pub is_disabled: bool,
	pub is_read_only: bool,
	pub is_required: bool,
	pub description_id: Option<String>,
}

/// A text field allows a user to enter a single line of text.
#[component]
pub fn TextField(
	/// The content to display as the label.
	#[prop(optional)]
	label: Option<String>,
	/// The initial value (uncontrolled).
	#[prop(optional, into)]
	default_value: Option<String>,
	/// The current value (controlled).
	#[prop(optional, into)]
	value: Option<Signal<String>>,
	/// Handler that is called when the value changes.
	#[prop(optional)]
	on_change: Option<BoxedTextChangeCallback>,
	/// The placeholder text when the field is empty.
	#[prop(optional, into)]
	placeholder: Option<String>,
	/// The name of the input, used when submitting an HTML form.
	#[prop(optional, into)]
	name: Option<String>,
	/// The HTML input type, `text` by default.
	#[prop(optional, into)]
	input_type: Option<String>,
	/// Whether the input is disabled.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	/// Whether the input is read only.
	#[prop(optional, into)]
	is_read_only: Signal<bool>,
	/// Whether the input is required.
	#[prop(optional, into)]
	is_required: Signal<bool>,
	/// The id of the element describing the field.
	#[prop(optional, into)]
	description_id: Option<String>,
) -> impl IntoView {
	let state = use_text_field_state(UseTextFieldStateProps {
		default_value,
		value,
		on_change,
	});

	let field_value = state.value_signal();
	let aria = field_aria(label.as_deref(), None);
	let input_type = input_type.unwrap_or_else(|| "text".to_string());

	view! {
		<input
			id=aria.field_id.clone()
			type=input_type
			value=move || field_value.get()
			placeholder=move || placeholder.clone()
			name=move || name.clone()
			disabled=move || is_disabled.get()
			readonly=move || is_read_only.get()
			required=move || is_required.get()
			aria-required=move || is_required.get().to_string()
			aria-labelledby=aria.aria_labelledby.clone()
			aria-describedby=description_id.clone()
			on:input=move |event: web_sys::Event| {
				let value = event
					.target()
					.and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
					.map(|input| input.value());
				if let Some(value) = value {
					state.set_value(value);
				}
			}
		/>
		<Label id=aria.label_id r#for=aria.field_id.clone()>
			{label.clone()}
		</Label>
	}
}

/// A text area allows a user to enter multiple lines of text.
#[component]
pub fn TextArea(
	/// The content to display as the label.
	#[prop(optional)]
	label: Option<String>,
	/// The initial value (uncontrolled).
	#[prop(optional, into)]
	default_value: Option<String>,
	/// The current value (controlled).
	#[prop(optional, into)]
	value: Option<Signal<String>>,
	/// Handler that is called when the value changes.
	#[prop(optional)]
	on_change: Option<BoxedTextChangeCallback>,
	/// The placeholder text when the field is empty.
	#[prop(optional, into)]
	placeholder: Option<String>,
	/// The name of the input, used when submitting an HTML form.
	#[prop(optional, into)]
	name: Option<String>,
	/// Whether the input is disabled.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	/// Whether the input is read only.
	#[prop(optional, into)]
	is_read_only: Signal<bool>,
	/// Whether the input is required.
	#[prop(optional, into)]
	is_required: Signal<bool>,
	/// The id of the element describing the field.
	#[prop(optional, into)]
	description_id: Option<String>,
) -> impl IntoView {
	let state = use_text_field_state(UseTextFieldStateProps {
		default_value,
		value,
		on_change,
	});

	let field_value = state.value_signal();
	let aria = field_aria(label.as_deref(), None);

	view! {
		<textarea
			id=aria.field_id.clone()
			prop:value=move || field_value.get()
			placeholder=move || placeholder.clone()
			name=move || name.clone()
			disabled=move || is_disabled.get()
			readonly=move || is_read_only.get()
			required=move || is_required.get()
			aria-required=move || is_required.get().to_string()
			aria-labelledby=aria.aria_labelledby.clone()
			aria-describedby=description_id.clone()
			on:input=move |event: web_sys::Event| {
				let value = event
					.target()
					.and_then(|target| target.dyn_into::<web_sys::HtmlTextAreaElement>().ok())
					.map(|input| input.value());
				if let Some(value) = value {
					state.set_value(value);
				}
			}
		></textarea>
		{label
			.clone()
			.filter(|label: &String| !label.is_empty())
			.map(|label| {
				view! {
					<Label id=aria.label_id.clone() r#for=aria.field_id.clone()>
						{label}
					</Label>
				}
			})}
	}
}

/// The visible label element for a field.
#[component]
fn Label(
	/// The id to set on the label element.
	id: String,
	/// The `for` attribute pointing at the field.
	r#for: String,
	children: Children,
) -> impl IntoView {
	view! { <label id=id.clone() for=r#for.clone()>{children()}</label> }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
	use leptos::mount::mount_to_body;
	use wasm_bindgen_test::*;

	use super::*;

	wasm_bindgen_test_configure!(run_in_browser);

	#[wasm_bindgen_test]
	async fn text_field_binds_value_and_label() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			let value: Signal<String> = Signal::derive(|| "Ada".to_string());

			view! { <TextField label="First name".to_string() value=value /> }
		});

		flush_microtasks().await;

		let input = document()
			.query_selector("input")
			.unwrap()
			.unwrap()
			.unchecked_into::<web_sys::HtmlInputElement>();

		let label = document().query_selector("label").unwrap().unwrap();

		assert_eq!(input.value(), "Ada");
		assert_eq!(
			label.get_attribute("for").as_deref(),
			input.get_attribute("id").as_deref(),
			"the label points at the input"
		);
	}

	/// Wait for queued reactive updates to apply.
	async fn flush_microtasks() {
		let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);

		wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
	}
}

mod otp;
pub use otp::OneTimePasswordField;
