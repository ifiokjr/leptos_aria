#![allow(clippy::needless_pass_by_value)]

use std::sync::Arc;

use leptos::prelude::*;
use leptos_aria_list_box::ListBox;
use leptos_aria_list_box::ListBoxOption;
use leptos_aria_selection::SelectionMode;
use leptos_aria_selection::UseSelectionStateProps;
use leptos_aria_selection::use_selection_state;

/// One option of a select box: a value with its text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectBoxItem {
	pub value: String,
	pub label: String,
}

/// A select box lets the user pick one option from a collapsible list,
/// without a free text input.
#[component]
pub fn SelectBox(
	/// The content to display as the label.
	#[prop(optional)]
	label: Option<String>,
	/// The available options.
	#[prop(into)]
	options: Vec<SelectBoxItem>,
	/// Whether the select box is disabled.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
) -> impl IntoView {
	let is_open = RwSignal::new(false);
	let state = use_selection_state(UseSelectionStateProps {
		mode: SelectionMode::Single,
		..Default::default()
	});
	let options = Arc::new(options);

	let selected_label = Memo::new({
		let selection = state.selection_signal();
		let options = Arc::clone(&options);

		move |_| {
			match selection.get() {
				leptos_aria_selection::Selection::Keys(keys) => {
					keys.iter()
						.next()
						.and_then(|key| options.iter().find(|item| &item.value == key))
						.map(|item| item.label.clone())
				}
				_ => None,
			}
		}
	});

	view! {
		<button
			type="button"
			role="combobox"
			aria-expanded=move || is_open.get().to_string()
			aria-haspopup="listbox"
			disabled=move || is_disabled.get()
			on:click=move |_| is_open.update(|open| *open = !*open)
		>
			{move || selected_label.get().unwrap_or_else(|| "Select…".to_string())}
		</button>
		<ListBox label=label.unwrap_or_default()>
			{move || {
				options
					.iter()
					.map(|item| {
						let value = item.value.clone();
						let label = item.label.clone();

						view! { <ListBoxOption value=value>{label}</ListBoxOption> }
					})
					.collect::<Vec<_>>()
			}}
		</ListBox>
		<span
			class="leptos-aria-select-box__value"
			data-open=move || if is_open.get() { "true" } else { "false" }
		></span>
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
	async fn select_box_toggles_its_expanded_state() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! {
				<SelectBox
					label="Size".to_string()
					options=vec![
						SelectBoxItem { value: "s".into(), label: "Small".into() },
						SelectBoxItem { value: "m".into(), label: "Medium".into() },
					]
				/>
			}
		});

		let trigger = document()
			.query_selector("[role='combobox']")
			.unwrap()
			.unwrap()
			.unchecked_into::<web_sys::HtmlButtonElement>();

		assert_eq!(
			trigger.get_attribute("aria-expanded").as_deref(),
			Some("false")
		);

		trigger.click();

		let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
		wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();

		assert_eq!(
			trigger.get_attribute("aria-expanded").as_deref(),
			Some("true"),
			"the trigger toggles the list"
		);
	}
}
