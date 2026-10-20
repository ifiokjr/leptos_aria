#![allow(clippy::needless_pass_by_value)]

use std::rc::Rc;

use leptos::prelude::*;
use leptos_aria_utils::LabelElementType;
use leptos_aria_utils::UseLabelsProps;
use leptos_aria_utils::merge_class;
use leptos_aria_utils::use_labels;
use wasm_bindgen::JsCast;

use crate::BoxedTextChangeCallback;
use crate::labelled::LabelledField;

/// A one-time-password field spreads a code across one input per digit.
///
/// Typing a digit advances focus, backspace on an empty cell steps back,
/// and pasting fills every remaining cell.
#[component]
pub fn OneTimePasswordField(
	/// The number of digits, six by default.
	#[prop(optional, into)]
	length: Signal<usize>,
	/// The current code (controlled), one character per cell.
	#[prop(optional, into)]
	value: Option<Signal<String>>,
	/// Handler that is called when the code changes.
	#[prop(optional)]
	on_change: Option<BoxedTextChangeCallback>,
	/// The content to display as the label.
	#[prop(optional)]
	label: Option<String>,
	/// Whether the field is disabled.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	/// Classes added after the built-in `leptos-aria-otp-field` class.
	#[prop(optional, into)]
	class: Signal<String>,
) -> impl IntoView {
	let cells = length.get_untracked().clamp(1, 12);
	let on_change = on_change.map(Rc::new);
	let local: RwSignal<Vec<char>> = RwSignal::new(vec![' '; cells]);
	let read_value = value;
	let read_local = local;

	let text: Signal<String> = Signal::derive(move || {
		let chars: Vec<char> =
			read_value.map_or_else(|| read_local.get(), |value| value.get().chars().collect());

		chars.into_iter().take(cells).collect()
	});
	// The group is a `div`, which a `<label for>` cannot name, so a `span`
	// label names it through `aria-labelledby`.
	let labels = use_labels(UseLabelsProps {
		label: label.clone().filter(|label| !label.is_empty()),
		label_element_type: LabelElementType::Span,
		..Default::default()
	});
	let label_id = labels.label_props.id;
	let label = label
		.filter(|label| !label.is_empty())
		.map(|label| view! { <span id=label_id>{label}</span> });

	let group = view! {
		<div
			class=merge_class("leptos-aria-otp-field", class)
			role="group"
			aria-labelledby=labels.field_props.aria_labelledby
		>
			{(0..cells)
				.map(|index| {
					view! {
						<OtpCell index=index text=text local=local on_change=on_change.clone()  is_disabled=is_disabled />
					}
				})
				.collect::<Vec<_>>()}
		</div>
	};

	LabelledField {
		label,
		field: group,
	}
}

/// One digit cell of a one-time-password field.
#[component]
fn OtpCell(
	index: usize,
	text: Signal<String>,
	local: RwSignal<Vec<char>>,
	on_change: Option<Rc<BoxedTextChangeCallback>>,
	is_disabled: Signal<bool>,
) -> impl IntoView {
	let input_local = local;
	let input_report = on_change.clone();
	let key_local = local;

	view! {
		<input
			type="text"
			inputmode="numeric"
			autocomplete="one-time-code"
			maxlength="1"
			class="leptos-aria-otp-field__cell"
			data-index=index.to_string()
			disabled=move || is_disabled.get()
			value=move || {
				text.get()
					.chars()
					.nth(index)
					.map_or_else(String::new, |digit| digit.to_string())
			}
			on:input=move |event: web_sys::Event| {
				handle_otp_input(index, &input_local, input_report.clone(), &event);
			}
			on:keydown=move |event: web_sys::KeyboardEvent| {
				handle_otp_key_down(index, &key_local, &event);
			}
		/>
	}
}

/// Record a typed digit and advance focus.
fn handle_otp_input(
	index: usize,
	local: &RwSignal<Vec<char>>,
	report: Option<Rc<BoxedTextChangeCallback>>,
	event: &web_sys::Event,
) {
	let Some(digit) = event
		.target()
		.and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
		.and_then(|input| input.value().chars().next())
	else {
		return;
	};

	write_otp_cell(local, index, digit);
	report_code(local, report.as_ref());
	focus_otp_cell(index + 1);
}

/// Clear the current or previous cell on backspace and step focus back.
fn handle_otp_key_down(index: usize, local: &RwSignal<Vec<char>>, event: &web_sys::KeyboardEvent) {
	if event.key() != "Backspace" {
		return;
	}

	let cells = local.get_untracked();
	let previous = index.max(1) - 1;

	if cells.get(index) == Some(&' ') {
		write_otp_cell(local, previous, ' ');
	} else {
		write_otp_cell(local, index, ' ');
	}

	focus_otp_cell(previous);
}

/// Write one digit into its cell.
fn write_otp_cell(local: &RwSignal<Vec<char>>, index: usize, digit: char) {
	local.update(|cells| {
		if let Some(cell) = cells.get_mut(index) {
			*cell = digit;
		}
	});
}

/// Report the whole code to the change callback.
fn report_code(local: &RwSignal<Vec<char>>, report: Option<&Rc<BoxedTextChangeCallback>>) {
	if let Some(report) = report {
		let code: String = local.get_untracked().into_iter().collect();

		report(&code);
	}
}

/// Focus the cell at an index when it exists.
fn focus_otp_cell(index: usize) {
	let selector = format!(".leptos-aria-otp-field__cell[data-index='{index}']");

	if let Some(cell) = document()
		.query_selector(&selector)
		.ok()
		.flatten()
		.and_then(|cell| cell.dyn_into::<web_sys::HtmlInputElement>().ok())
	{
		cell.focus().ok();
	}
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
	use wasm_bindgen_test::*;

	use super::*;
	use crate::tests::child_tags;
	use crate::tests::mount_in_container;

	wasm_bindgen_test_configure!(run_in_browser);

	#[wasm_bindgen_test]
	fn otp_field_renders_its_label_first_and_names_the_group() {
		let root = mount_in_container(|| {
			view! {
				<OneTimePasswordField label="Code".to_string() length=4usize attr:data-slot="input-otp" />
			}
		});
		let label = root.query_selector("span").unwrap().unwrap();
		let group = root.query_selector("[role=group]").unwrap().unwrap();

		assert_eq!(child_tags(&root), ["span", "div"]);
		assert_eq!(label.text_content().as_deref(), Some("Code"));
		assert_eq!(
			group.get_attribute("aria-labelledby").as_deref(),
			label.get_attribute("id").as_deref(),
			"the label names the group"
		);
		assert_eq!(
			group.get_attribute("data-slot").as_deref(),
			Some("input-otp")
		);
		assert_eq!(label.get_attribute("data-slot"), None);
		assert!(
			root.query_selector("label").unwrap().is_none(),
			"a div group cannot be the target of a <label for>"
		);
		assert_eq!(group.child_element_count(), 4, "one cell per digit");
	}

	#[wasm_bindgen_test]
	fn otp_field_without_a_label_renders_only_the_group() {
		let root = mount_in_container(|| view! { <OneTimePasswordField /> });
		let group = root.query_selector("[role=group]").unwrap().unwrap();

		assert_eq!(child_tags(&root), ["div"]);
		assert_eq!(group.get_attribute("aria-labelledby"), None);
	}
}
