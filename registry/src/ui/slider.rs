//! A thumb on a track for choosing a value in a range.
//!
//! ```rust ignore
//! use crate::components::ui::slider::Slider;
//!
//! view! {
//!     <Slider
//!         label="Volume"
//!         default_value=40.0
//!         step=5.0
//!         on_change=Box::new(|volume| logging::log!("{volume}"))
//!         class="w-60"
//!     />
//! }
//! ```

use leptos::prelude::*;
use leptos_aria::BoxedSliderChangeCallback;
use leptos_aria::Slider as AriaSlider;

use super::utils::cn;

/// The root. Its native range input carries the value, keyboard, and
/// `role="slider"` semantics, so it is stretched invisibly over the whole
/// control to take pointer input as well.
const BASE: &str = "relative flex h-4 w-full touch-none items-center select-none \
                    has-disabled:opacity-50 [&>input]:absolute [&>input]:inset-0 \
                    [&>input]:z-10 [&>input]:m-0 [&>input]:size-full [&>input]:cursor-pointer \
                    [&>input]:opacity-0 [&>input:disabled]:cursor-not-allowed";

// The track, fill, and thumb are rendered by `leptos_aria` with the classes
// `leptos-aria-slider__track`, `__fill`, and `__thumb`. Tailwind reads `_` in
// an arbitrary variant as a space, so they are matched by class suffix.

const TRACK: &str = "[&_[class$=track]]:relative [&_[class$=track]]:h-1.5 \
                     [&_[class$=track]]:w-full [&_[class$=track]]:grow \
                     [&_[class$=track]]:rounded-full [&_[class$=track]]:bg-muted";

const RANGE: &str = "[&_[class$=fill]]:absolute [&_[class$=fill]]:inset-y-0 \
                     [&_[class$=fill]]:left-0 [&_[class$=fill]]:rounded-full \
                     [&_[class$=fill]]:bg-primary";

const THUMB: &str = "[&_[class$=thumb]]:absolute [&_[class$=thumb]]:top-1/2 \
                     [&_[class$=thumb]]:block [&_[class$=thumb]]:size-4 \
                     [&_[class$=thumb]]:-translate-x-1/2 [&_[class$=thumb]]:-translate-y-1/2 \
                     [&_[class$=thumb]]:rounded-full [&_[class$=thumb]]:border \
                     [&_[class$=thumb]]:border-primary [&_[class$=thumb]]:bg-white \
                     [&_[class$=thumb]]:shadow-sm [&_[class$=thumb]]:ring-ring/50 \
                     [&_[class$=thumb]]:transition-[color,box-shadow] \
                     hover:[&_[class$=thumb]]:ring-4 \
                     has-[input:focus-visible]:[&_[class$=thumb]]:ring-4";

/// A single-thumb slider. Value, stepping, clamping, and the range input come
/// from `leptos_aria::Slider`.
#[component]
pub fn Slider(
	/// The value (controlled).
	#[prop(optional, into)]
	value: Option<Signal<f64>>,
	/// The value at first (uncontrolled), `min` by default.
	#[prop(optional)]
	default_value: Option<f64>,
	/// Called with the new value as the thumb moves.
	#[prop(optional)]
	on_change: Option<BoxedSliderChangeCallback>,
	/// The smallest value, `0` by default.
	#[prop(optional)]
	min: Option<f64>,
	/// The largest value, `100` by default.
	#[prop(optional)]
	max: Option<f64>,
	/// The step between values, `1` by default.
	#[prop(optional)]
	step: Option<f64>,
	/// Whether the slider is disabled.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	/// The accessible name, set as the input's `aria-label`.
	#[prop(optional, into)]
	label: Option<String>,
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
) -> impl IntoView {
	let class = move || class.with(|class| cn(&[BASE, TRACK, RANGE, THUMB, class]));

	view! {
		<AriaSlider
			nostrip:value=value
			nostrip:default_value=default_value
			nostrip:on_change=on_change
			nostrip:min=min
			nostrip:max=max
			nostrip:step=step
			is_disabled=is_disabled
			nostrip:label=label
			class=class
			attr:data-slot="slider"
		/>
	}
}
