#![allow(clippy::needless_pass_by_value)]

use std::rc::Rc;

use leptos::prelude::*;
use wasm_bindgen::JsCast;

/// A callback invoked when the value changes.
pub type BoxedNumberChangeCallback = Box<dyn Fn(f64)>;

/// A number field lets a user enter a numeric value, with keyboard stepping
/// through the min and max range.
///
/// The reactive wiring is deliberately split: this component resolves the
/// value, range, and callbacks once, and the input element binds them —
/// arrows step by the step size while typing parses and clamps.
#[component]
pub fn NumberField(
	/// The content to display as the label.
	#[prop(optional)]
	label: Option<String>,
	/// The current value (controlled).
	#[prop(optional, into)]
	value: Option<Signal<f64>>,
	/// The initial value (uncontrolled).
	#[prop(optional)]
	default_value: Option<f64>,
	/// Handler that is called when the value changes.
	#[prop(optional)]
	on_change: Option<BoxedNumberChangeCallback>,
	/// The smallest allowed value.
	#[prop(optional)]
	min_value: Option<f64>,
	/// The largest allowed value.
	#[prop(optional)]
	max_value: Option<f64>,
	/// How much each step changes the value, `1.0` by default.
	#[prop(optional)]
	step: Option<f64>,
	/// Whether the field is disabled.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
) -> impl IntoView {
	let step = step.unwrap_or(1.0);
	let local = RwSignal::new(default_value.unwrap_or(0.0));
	let value_signal = value;
	let read_value = value_signal.unwrap_or_else(|| Signal::from(local));

	let range = Rc::new((min_value, max_value));
	let on_change = Rc::new(on_change);

	let input_range = Rc::clone(&range);
	let input_on_change = Rc::clone(&on_change);
	let input_local = local;

	let key_range = Rc::clone(&range);
	let key_on_change = Rc::clone(&on_change);
	let key_step = step;
	let key_local = local;
	let key_read = read_value;

	let aria = leptos_aria_text_field::field_aria(label.as_deref(), None);
	let label_element = label.map(|label| {
		view! {
			<label id=aria.label_id.clone() for=aria.field_id.clone()>
				{label}
			</label>
		}
	});

	view! {
		<input
			id=aria.field_id.clone()
			type="number"
			value=move || read_value.get().to_string()
			min=min_value.map(|min| min.to_string())
			max=max_value.map(|max| max.to_string())
			step=step.to_string()
			disabled=move || is_disabled.get()
			aria-labelledby=aria.aria_labelledby.clone()
			on:input=move |event: web_sys::Event| {
				let parsed = parse_input_value(event);

				if let Some(parsed) = parsed {
					input_local.set(clamp(parsed, &input_range));

					if let Some(on_change) = input_on_change.as_ref() {
						(*on_change)(parsed);
					}
				}
			}
			on:keydown=move |event: web_sys::KeyboardEvent| {
				handle_step_key(
					&event,
					key_step,
					&key_range,
					&key_local,
					&key_read,
					&key_on_change,
				);
			}
		/>
		{label_element}
	}
}

/// Apply one keyboard step to the value.
#[allow(clippy::too_many_arguments)]
fn handle_step_key(
	event: &web_sys::KeyboardEvent,
	step: f64,
	range: &Rc<NumberRange>,
	local: &RwSignal<f64>,
	read: &Signal<f64>,
	on_change: &Rc<Option<BoxedNumberChangeCallback>>,
) {
	let delta = match event.key().as_str() {
		"ArrowUp" => step,
		"ArrowDown" => -step,
		_ => return,
	};

	event.prevent_default();

	let next = clamp(read.get_untracked() + delta, range);
	local.set(next);

	if let Some(on_change) = on_change.as_ref() {
		(*on_change)(next);
	}
}

/// The numeric value typed into an input event, if it parses.
fn parse_input_value(event: web_sys::Event) -> Option<f64> {
	event
		.target()
		.and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
		.and_then(|input| input.value().parse::<f64>().ok())
}

type NumberRange = (Option<f64>, Option<f64>);

/// Keep a value inside the optional min and max range.
fn clamp(value: f64, (min, max): &NumberRange) -> f64 {
	let value = min.map_or(value, |min| value.max(min));

	max.map_or(value, |max| value.min(max))
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
	use leptos::mount::mount_to_body;
	use wasm_bindgen::JsCast;
	use wasm_bindgen_test::*;

	use super::*;

	wasm_bindgen_test_configure!(run_in_browser);

	#[wasm_bindgen_test]
	async fn number_field_renders_the_range() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! {
				<NumberField
					label="Age".to_string()
					default_value=30.0
					min_value=0.0
					max_value=100.0
				/>
			}
		});

		let input = document()
			.query_selector("input[type='number']")
			.unwrap()
			.unwrap()
			.unchecked_into::<web_sys::HtmlInputElement>();

		assert_eq!(input.value(), "30");
		assert_eq!(input.get_attribute("min").as_deref(), Some("0"));
		assert_eq!(input.get_attribute("max").as_deref(), Some("100"));
	}
}
