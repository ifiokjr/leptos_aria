#![allow(clippy::needless_pass_by_value)]

use leptos::prelude::*;
use leptos_aria_calendar::Calendar;
use leptos_aria_date::DateValue;

/// A date field lets the user type a date in `YYYY-MM-DD` form.
#[component]
pub fn DateField(
	/// The current date (controlled).
	#[prop(optional, into)]
	value: Option<Signal<DateValue>>,
	/// The initial date (uncontrolled).
	#[prop(optional)]
	default_value: Option<DateValue>,
	/// The content to display as the label.
	#[prop(optional)]
	label: Option<String>,
) -> impl IntoView {
	let initial = default_value.unwrap_or(DateValue::new(1970, 1, 1));
	let local = RwSignal::new(initial);
	let read_value = value;

	let text = Signal::derive(move || {
		let date = read_value.map_or_else(|| local.get(), |value| value.get());

		format!("{}-{:02}-{:02}", date.year, date.month, date.day)
	});

	view! {
		<input type="text" value=move || text.get() aria-label=label.unwrap_or_default() />
	}
}

/// A date picker combines a date field with a calendar popover.
#[component]
pub fn DatePicker(
	/// The initially selected date (uncontrolled).
	#[prop(optional)]
	default_value: Option<DateValue>,
	/// The content to display as the label.
	#[prop(optional)]
	label: Option<String>,
) -> impl IntoView {
	let is_open = RwSignal::new(false);

	view! {
		<div class="leptos-aria-date-picker">
			<button
				type="button"
				class="leptos-aria-date-picker__trigger"
				aria-haspopup="dialog"
				aria-expanded=move || is_open.get().to_string()
				on:click=move |_| is_open.update(|open| *open = !*open)
			>
				{label.unwrap_or_else(|| "Select date".to_string())}
			</button>
			<div
				class="leptos-aria-date-picker__calendar"
				hidden=move || !is_open.get()
				role="dialog"
			>
				<Calendar default_value=default_value.unwrap() />
			</div>
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
	async fn date_picker_toggles_the_calendar() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! { <DatePicker default_value=DateValue::new(2026, 10, 18) /> }
		});

		let trigger = document()
			.query_selector(".leptos-aria-date-picker__trigger")
			.unwrap()
			.unwrap()
			.unchecked_into::<web_sys::HtmlButtonElement>();

		let calendar = document()
			.query_selector(".leptos-aria-date-picker__calendar")
			.unwrap()
			.unwrap();

		assert!(calendar.has_attribute("hidden"));

		trigger.click();

		let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
		wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();

		assert!(!calendar.has_attribute("hidden"), "the calendar opens");
	}
}
