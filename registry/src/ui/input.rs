//! A single-line text input.
//!
//! ```rust ignore
//! use crate::components::ui::input::Input;
//!
//! let email = RwSignal::new(String::new());
//!
//! view! {
//!     <Input
//!         label="Email"
//!         input_type="email"
//!         placeholder="you@example.com"
//!         value=email
//!         on_change=Box::new(move |value: &str| email.set(value.to_string()))
//!     />
//! }
//! ```

use leptos::prelude::*;
use leptos_aria::BoxedTextChangeCallback;
use leptos_aria::TextField;

use super::utils::cn;

const BASE: &str = "h-9 w-full min-w-0 rounded-md border border-input bg-transparent px-3 py-1 \
                    text-base shadow-xs transition-[color,box-shadow] outline-none \
                    selection:bg-primary selection:text-primary-foreground file:inline-flex \
                    file:h-7 file:border-0 file:bg-transparent file:text-sm file:font-medium \
                    file:text-foreground placeholder:text-muted-foreground \
                    focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 \
                    disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 \
                    aria-invalid:border-destructive aria-invalid:ring-destructive/20 md:text-sm \
                    dark:bg-input/30 dark:aria-invalid:ring-destructive/40";

/// A text input. Value state, labelling, and the native `input` come from
/// `leptos_aria::TextField`, which renders its `label` before the input and
/// spreads `class` and `data-slot` onto the input alone.
#[component]
pub fn Input(
	/// A visible label, linked to the input with `for`.
	#[prop(optional, into)]
	label: Option<String>,
	/// The value at first (uncontrolled).
	#[prop(optional, into)]
	default_value: Option<String>,
	/// The value (controlled).
	#[prop(optional, into)]
	value: Option<Signal<String>>,
	/// Called with the new value on every keystroke.
	#[prop(optional)]
	on_change: Option<BoxedTextChangeCallback>,
	/// Text shown while the input is empty.
	#[prop(optional, into)]
	placeholder: Option<String>,
	/// The input name used when submitting a form.
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
	/// The `id` of an element describing the input.
	#[prop(optional, into)]
	description_id: Option<String>,
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
) -> impl IntoView {
	let class = Signal::derive(move || class.with(|class| cn(&[BASE, class])));

	view! {
		<TextField
			nostrip:label=label
			nostrip:default_value=default_value
			nostrip:value=value
			nostrip:on_change=on_change
			nostrip:placeholder=placeholder
			nostrip:name=name
			nostrip:input_type=input_type
			is_disabled=is_disabled
			is_read_only=is_read_only
			is_required=is_required
			nostrip:description_id=description_id
			class=class
			attr:data-slot="input"
		/>
	}
}
