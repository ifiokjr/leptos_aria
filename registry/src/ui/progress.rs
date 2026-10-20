//! A bar showing how far a task has come.
//!
//! ```rust ignore
//! use crate::components::ui::progress::Progress;
//!
//! view! {
//!     <Progress value=Signal::from(66.0) attr:aria-label="Upload" class="w-60" />
//!     <Progress is_indeterminate=true attr:aria-label="Loading" />
//! }
//! ```

use leptos::prelude::*;
use leptos_aria::ProgressBar;

use super::utils::cn;

/// The root, which `leptos_aria` renders with `role="progressbar"` and the
/// `aria-value*` attributes. Its fill has the class
/// `leptos-aria-progress-bar__fill`, matched by suffix because Tailwind reads
/// `_` in an arbitrary variant as a space. While indeterminate the root has
/// no `aria-valuenow` and the fill pulses across the whole track.
const BASE: &str = "relative h-2 w-full overflow-hidden rounded-full bg-primary/20 \
                    [&>span]:block [&>span]:size-full [&_[class$=fill]]:block \
                    [&_[class$=fill]]:h-full [&_[class$=fill]]:bg-primary \
                    [&_[class$=fill]]:transition-all \
                    [&:not([aria-valuenow])_[class$=fill]]:w-full \
                    [&:not([aria-valuenow])_[class$=fill]]:animate-pulse";

/// A progress bar. The `progressbar` role and its ARIA values come from
/// `leptos_aria::ProgressBar`. Name it with `attr:aria-label`.
#[component]
pub fn Progress(
	/// How far the task has come, `0` by default.
	#[prop(optional, into)]
	value: Option<Signal<f64>>,
	/// The value at the start of the bar, `0` by default.
	#[prop(optional)]
	min_value: Option<f64>,
	/// The value at the end of the bar, `100` by default.
	#[prop(optional)]
	max_value: Option<f64>,
	/// Whether progress is unknown, so the bar pulses instead of filling.
	#[prop(optional, into)]
	is_indeterminate: Signal<bool>,
	/// The value as read aloud, such as "3 of 4 files". A percentage by
	/// default.
	#[prop(optional, into)]
	value_label: Option<String>,
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
) -> impl IntoView {
	let class = move || class.with(|class| cn(&[BASE, class]));

	view! {
		<ProgressBar
			nostrip:value=value
			nostrip:min_value=min_value
			nostrip:max_value=max_value
			is_indeterminate=is_indeterminate
			nostrip:value_label=value_label
			class=class
			attr:data-slot="progress"
		/>
	}
}
