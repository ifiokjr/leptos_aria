#![allow(clippy::needless_pass_by_value)]

use leptos::prelude::*;
use leptos_aria_text_field::BoxedTextChangeCallback;
use leptos_aria_text_field::TextField;
use wasm_bindgen::JsCast;

/// A callback invoked when the search is submitted.
pub type BoxedSearchSubmitCallback = Box<dyn Fn(&str)>;

/// A callback invoked when the field is cleared.
pub type BoxedSearchClearCallback = Box<dyn Fn()>;

/// A search field allows a user to enter and clear a search query.
#[component]
pub fn SearchField(
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
	/// Handler that is called when the search is submitted.
	#[prop(optional)]
	on_submit: Option<BoxedSearchSubmitCallback>,
	/// Handler that is called when the clear button is pressed.
	#[prop(optional)]
	on_clear: Option<BoxedSearchClearCallback>,
	/// The placeholder text when the field is empty.
	#[prop(optional, into)]
	placeholder: Option<String>,
	/// The name of the input, used when submitting an HTML form.
	#[prop(optional, into)]
	name: Option<String>,
	/// Whether the input is disabled.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	/// Whether the search is still loading, which disables submission.
	#[prop(optional, into)]
	is_loading: Signal<bool>,
) -> impl IntoView {
	let state = leptos_aria_text_field::use_text_field_state(
		leptos_aria_text_field::UseTextFieldStateProps {
			default_value,
			value,
			on_change,
		},
	);

	let field_value = state.value_signal();
	let submit_state = state.clone();
	let controlled_value: Signal<String> = Signal::derive(move || field_value.get());

	view! {
		<TextField
			label=label.clone().unwrap_or_default()
			value=controlled_value
			placeholder=placeholder.clone().unwrap_or_default()
			name=name.clone().unwrap_or_default()
			is_disabled=is_disabled
			input_type="search".to_string()
		/>
		<button
			type="button"
			aria-label="Clear search"
			disabled=move || is_disabled.get() || field_value.get().is_empty()
			on:mousedown=|event: web_sys::MouseEvent| {
				// Keep focus in the field when clicking the clear button.
				event.prevent_default();
			}
			on:click=move |_| {
				state.set_value("");
				if let Some(on_clear) = &on_clear {
					on_clear();
				}
				if let Some(input) = document().query_selector("input[type='search']").unwrap() {
					let input = input.unchecked_into::<web_sys::HtmlInputElement>();
					let _ = input.focus();
				}
			}
		>
			"✕"
		</button>
		<input
			class="leptos-aria-search-field__submit"
			type="submit"
			value="Search"
			disabled=move || is_loading.get()
			on:click=move |event: web_sys::MouseEvent| {
				event.prevent_default();

				if let Some(on_submit) = &on_submit {
					on_submit(&submit_state.value());
				}
			}
		/>
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
	async fn clear_button_empties_the_field() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! { <SearchField label="Search".to_string() default_value="query".to_string() /> }
		});

		let input = document()
			.query_selector("input[type='search']")
			.unwrap()
			.unwrap()
			.unchecked_into::<web_sys::HtmlInputElement>();

		let clear = document()
			.query_selector("button[aria-label='Clear search']")
			.unwrap()
			.unwrap()
			.unchecked_into::<web_sys::HtmlButtonElement>();

		assert_eq!(input.value(), "query");

		clear.click();

		let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
		wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();

		assert_eq!(input.value(), "", "the clear button empties the field");
	}
}
