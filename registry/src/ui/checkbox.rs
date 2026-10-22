//! A checkbox with its label.
//!
//! ```rust ignore
//! use crate::components::ui::checkbox::Checkbox;
//!
//! let accepted = RwSignal::new(false);
//!
//! view! {
//!     <Checkbox is_selected=accepted on_change=Box::new(move |on| accepted.set(on))>
//!         "Accept terms and conditions"
//!     </Checkbox>
//! }
//! ```

use leptos::prelude::*;
use leptos_aria::BoxedSelectionChangeCallback;
use leptos_aria::Checkbox as AriaCheckbox;

use super::utils::cn;

/// The label root. The native checkbox stays in the tree for keyboard, forms,
/// and assistive technology but is visually hidden; the box below reads its
/// state through `has-*` variants.
const BASE: &str = "group/checkbox relative inline-flex items-center gap-2 text-sm leading-none \
                    font-medium select-none has-disabled:cursor-not-allowed \
                    has-disabled:opacity-50 [&>input]:sr-only";

const CONTROL: &str = "flex size-4 shrink-0 items-center justify-center rounded-[4px] border \
                       border-input text-primary-foreground shadow-xs transition-shadow \
                       dark:bg-input/30 group-has-checked/checkbox:border-primary \
                       group-has-checked/checkbox:bg-primary \
                       dark:group-has-checked/checkbox:bg-primary \
                       group-has-indeterminate/checkbox:border-primary \
                       group-has-indeterminate/checkbox:bg-primary \
                       dark:group-has-indeterminate/checkbox:bg-primary \
                       group-has-focus-visible/checkbox:border-ring \
                       group-has-focus-visible/checkbox:ring-3 \
                       group-has-focus-visible/checkbox:ring-ring/50";

const CHECK: &str = "hidden size-3.5 group-has-[input:checked:not(:indeterminate)]/checkbox:block";

const DASH: &str = "hidden size-3.5 group-has-indeterminate/checkbox:block";

/// A checkbox. The children are its label, so clicking them toggles it.
/// State, group membership, and the native input come from
/// `leptos_aria::Checkbox`.
#[component]
pub fn Checkbox(
	/// The value this checkbox adds to a surrounding `CheckboxGroup`.
	#[prop(optional, into)]
	value: Option<String>,
	/// Whether the checkbox starts checked (uncontrolled).
	#[prop(optional, into)]
	default_selected: Signal<bool>,
	/// Whether the checkbox is checked (controlled).
	#[prop(optional, into)]
	is_selected: Option<Signal<bool>>,
	/// Called with the new state when the checkbox is toggled.
	#[prop(optional)]
	on_change: Option<BoxedSelectionChangeCallback>,
	/// Whether the checkbox is disabled.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	/// Whether the checkbox is read only.
	#[prop(optional, into)]
	is_read_only: Signal<bool>,
	/// Whether the checkbox shows a dash instead of a check. Presentational
	/// only: it stays until you clear it.
	#[prop(optional, into)]
	is_indeterminate: Signal<bool>,
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let class = Signal::derive(move || class.with(|class| cn(&[BASE, class])));

	view! {
		<AriaCheckbox
			nostrip:value=value
			default_selected=default_selected
			nostrip:is_selected=is_selected
			nostrip:on_change=on_change
			is_disabled=is_disabled
			is_read_only=is_read_only
			is_indeterminate=is_indeterminate
			class=class
			attr:data-slot="checkbox"
		>
			<span data-slot="checkbox-control" class=CONTROL>
				<svg
					data-slot="checkbox-indicator"
					class=CHECK
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="3"
					stroke-linecap="round"
					stroke-linejoin="round"
					aria-hidden="true"
				>
					<path d="M20 6 9 17l-5-5" />
				</svg>
				<svg
					data-slot="checkbox-indicator"
					class=DASH
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="3"
					stroke-linecap="round"
					aria-hidden="true"
				>
					<path d="M5 12h14" />
				</svg>
			</span>
			{children()}
		</AriaCheckbox>
	}
}
