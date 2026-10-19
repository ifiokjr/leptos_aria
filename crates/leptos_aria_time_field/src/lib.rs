#![doc(html_logo_url = "https://ifiokjr.github.io/leptos_aria/logo.svg")]
#![doc(html_favicon_url = "https://ifiokjr.github.io/leptos_aria/favicon.ico")]
#![allow(clippy::needless_pass_by_value)]

use leptos::prelude::*;

/// A time of day, `0:00` through `23:59`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TimeValue {
	/// The hour, `0` through `23`.
	pub hour: u32,

	/// The minute, `0` through `59`.
	pub minute: u32,
}

impl TimeValue {
	/// A time, clamping the fields into range.
	pub fn new(hour: u32, minute: u32) -> Self {
		Self {
			hour: hour.min(23),
			minute: minute.min(59),
		}
	}

	/// Parse an `HH:MM` string.
	pub fn parse(value: &str) -> Option<Self> {
		let (hour, minute) = value.split_once(':')?;

		Some(Self::new(
			hour.trim().parse().ok()?,
			minute.trim().parse().ok()?,
		))
	}

	/// The `HH:MM` form.
	pub fn format(&self) -> String {
		format!("{:02}:{:02}", self.hour, self.minute)
	}

	/// The time shifted by a number of minutes, wrapping at midnight.
	#[must_use]
	pub fn add_minutes(&self, minutes: i32) -> Self {
		let total = (self.hour as i32 * 60 + self.minute as i32 + minutes).rem_euclid(24 * 60);

		Self::new((total / 60) as u32, (total % 60) as u32)
	}
}

/// A time field lets the user type a time in `HH:MM` form, stepping the
/// fields with the arrow keys.
#[component]
pub fn TimeField(
	/// The current time (controlled).
	#[prop(optional, into)]
	value: Option<Signal<TimeValue>>,
	/// The initial time (uncontrolled).
	#[prop(optional)]
	default_value: Option<TimeValue>,
	/// The content to display as the label.
	#[prop(optional)]
	label: Option<String>,
) -> impl IntoView {
	let initial = default_value.unwrap_or_default();
	let local = RwSignal::new(initial);
	let read_value = value;
	let step_state = local;

	let text = Signal::derive(move || {
		let time = read_value.map_or_else(|| local.get(), |value| value.get());

		time.format()
	});

	view! {
		<input
			type="text"
			value=move || text.get()
			aria-label=label.unwrap_or_default()
			on:keydown=move |event: web_sys::KeyboardEvent| {
				let step = match event.key().as_str() {
					"ArrowUp" => 1,
					"ArrowDown" => -1,
					_ => return,
				};

				event.prevent_default();

				let current = step_state.get_untracked();

				step_state.set(current.add_minutes(step));
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
	async fn time_field_renders_the_initial_time() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! { <TimeField default_value=TimeValue::new(9, 30) /> }
		});

		let input = document()
			.query_selector("input[type='text']")
			.unwrap()
			.unwrap()
			.unchecked_into::<web_sys::HtmlInputElement>();

		assert_eq!(input.value(), "09:30");
	}
}
