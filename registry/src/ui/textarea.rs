//! A multi-line text input that grows with its content.
//!
//! ```rust ignore
//! use crate::components::ui::textarea::Textarea;
//!
//! view! {
//!     <Textarea
//!         label="Message"
//!         placeholder="Type your message here."
//!         on_change=Box::new(|value: &str| logging::log!("{value}"))
//!     />
//! }
//! ```

use leptos::prelude::*;
use leptos_aria::BoxedTextChangeCallback;
use leptos_aria::TextArea;

use super::utils::cn;

const BASE: &str = "flex field-sizing-content min-h-16 w-full rounded-md border border-input \
                    bg-transparent px-3 py-2 text-base shadow-xs transition-[color,box-shadow] \
                    outline-none placeholder:text-muted-foreground focus-visible:border-ring \
                    focus-visible:ring-3 focus-visible:ring-ring/50 \
                    disabled:cursor-not-allowed disabled:opacity-50 \
                    aria-invalid:border-destructive aria-invalid:ring-destructive/20 md:text-sm \
                    dark:bg-input/30 dark:aria-invalid:ring-destructive/40";

/// A text area. Value state, labelling, and the native `textarea` come from
/// `leptos_aria::TextArea`, which renders its `label` after the field.
#[component]
pub fn Textarea(
	/// A visible label, linked to the field with `aria-labelledby`.
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
	/// Text shown while the field is empty.
	#[prop(optional, into)]
	placeholder: Option<String>,
	/// The field name used when submitting a form.
	#[prop(optional, into)]
	name: Option<String>,
	/// Whether the field is disabled.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	/// Whether the field is read only.
	#[prop(optional, into)]
	is_read_only: Signal<bool>,
	/// Whether the field is required.
	#[prop(optional, into)]
	is_required: Signal<bool>,
	/// The `id` of an element describing the field.
	#[prop(optional, into)]
	description_id: Option<String>,
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
) -> impl IntoView {
	let class = move || class.with(|class| cn(&[BASE, class]));

	view! {
		<TextArea
			nostrip:label=label
			nostrip:default_value=default_value
			nostrip:value=value
			nostrip:on_change=on_change
			nostrip:placeholder=placeholder
			nostrip:name=name
			is_disabled=is_disabled
			is_read_only=is_read_only
			is_required=is_required
			nostrip:description_id=description_id
			class=class
			attr:data-slot="textarea"
		/>
	}
}
