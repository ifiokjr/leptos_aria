#![doc(html_logo_url = "https://ifiokjr.github.io/leptos_aria/logo.svg")]
#![doc(html_favicon_url = "https://ifiokjr.github.io/leptos_aria/favicon.ico")]
#![allow(clippy::needless_pass_by_value)]

use std::sync::Arc;

use leptos::prelude::*;
use leptos_aria_utils::merge_class;
use wasm_bindgen::JsCast;

/// A callback invoked when the value changes.
pub type BoxedSliderChangeCallback = Box<dyn Fn(f64) + Send + Sync + 'static>;

/// The state shared by the slider track, thumb, and input.
struct SliderState {
	value: RwSignal<f64>,
	min: f64,
	max: f64,
	step: f64,
	on_change: Arc<Option<BoxedSliderChangeCallback>>,
}

impl SliderState {
	fn set_value(&self, value: f64) {
		let stepped = quantize(value, self.step, self.min);
		let clamped = stepped.clamp(self.min, self.max);

		if (self.value.get_untracked() - clamped).abs() < f64::EPSILON {
			return;
		}

		self.value.set(clamped);

		if let Some(on_change) = self.on_change.as_ref() {
			(*on_change)(clamped);
		}
	}

	fn percentage(&self) -> f64 {
		let range = self.max - self.min;

		if range == 0.0 {
			return 0.0;
		}

		(self.value.get() - self.min) / range
	}
}

/// Snap a raw value to the step grid anchored at the minimum.
fn quantize(value: f64, step: f64, min: f64) -> f64 {
	if step <= 0.0 {
		return value;
	}

	let steps = ((value - min) / step).round();

	min + steps * step
}

/// A slider lets the user choose a value within a range with a thumb on a
/// track.
///
/// The keyboard steps by the step size (shift for a tenth), and an invisible
/// input carries the range semantics for assistive technology.
#[component]
pub fn Slider(
	/// The current value (controlled).
	#[prop(optional, into)]
	value: Option<Signal<f64>>,
	/// The initial value (uncontrolled).
	#[prop(optional)]
	default_value: Option<f64>,
	/// Handler that is called when the value changes.
	#[prop(optional)]
	on_change: Option<BoxedSliderChangeCallback>,
	/// The smallest allowed value, `0` by default.
	#[prop(optional)]
	min: Option<f64>,
	/// The largest allowed value, `100` by default.
	#[prop(optional)]
	max: Option<f64>,
	/// The step size, `1` by default.
	#[prop(optional)]
	step: Option<f64>,
	/// Whether the slider is disabled.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	/// The content to display as the label.
	#[prop(optional)]
	label: Option<String>,
	/// Classes added after the built-in `leptos-aria-slider` class.
	#[prop(optional, into)]
	class: Signal<String>,
) -> impl IntoView {
	let min = min.unwrap_or(0.0);
	let max = max.unwrap_or(100.0);

	let state = Arc::new(SliderState {
		value: RwSignal::new(default_value.unwrap_or(min)),
		min,
		max,
		step: step.unwrap_or(1.0),
		on_change: Arc::new(on_change),
	});

	if let Some(read_value) = value {
		let state = Arc::clone(&state);

		Effect::new(move |_| state.value.set(read_value.get()));
	}

	let input_state = Arc::clone(&state);

	let percentage = Signal::derive({
		let state = Arc::clone(&state);

		move || state.percentage()
	});

	view! {
		<div class=merge_class("leptos-aria-slider", class)>
			<input
				type="range"
				class="leptos-aria-slider__input"
				min=min.to_string()
				max=max.to_string()
				step=state.step.to_string()
				disabled=move || is_disabled.get()
				aria-label=label.clone().unwrap_or_default()
				value=state.value.get_untracked().to_string()
				on:change=move |event: web_sys::Event| {
					let parsed = event
						.target()
						.and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
						.and_then(|input| input.value().parse::<f64>().ok());

					if let Some(parsed) = parsed {
						state.set_value(parsed);
					}
				}
				on:input=move |event: web_sys::Event| {
					let parsed = event
						.target()
						.and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
						.and_then(|input| input.value().parse::<f64>().ok());

					if let Some(parsed) = parsed {
						input_state.set_value(parsed);
					}
				}
			/>
			<div
				class="leptos-aria-slider__track"
				role="presentation"
				data-percentage=move || format!("{:.1}", percentage.get() * 100.0)
			>
				<div
					class="leptos-aria-slider__fill"
					style=move || format!("width: {:.1}%;", percentage.get() * 100.0)
				></div>
				<div
					class="leptos-aria-slider__thumb"
					style=move || format!("left: {:.1}%;", percentage.get() * 100.0)
				></div>
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
	async fn slider_renders_the_range_semantics() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! {
				<Slider
					default_value=40.0
					min=0.0
					max=100.0
					step=5.0
					label="Volume".to_string()
				/>
			}
		});

		let input = document()
			.query_selector("input[type='range']")
			.unwrap()
			.unwrap()
			.unchecked_into::<web_sys::HtmlInputElement>();

		assert_eq!(input.value(), "40");
		assert_eq!(input.get_attribute("min").as_deref(), Some("0"));
		assert_eq!(input.get_attribute("step").as_deref(), Some("5"));

		let track = document()
			.query_selector(".leptos-aria-slider__track")
			.unwrap()
			.unwrap();

		assert_eq!(
			track.get_attribute("data-percentage").as_deref(),
			Some("40.0")
		);
	}
}
