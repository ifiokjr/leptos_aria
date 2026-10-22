//! A set of options where only one can be chosen.
//!
//! ```rust ignore
//! use crate::components::ui::radio_group::RadioGroup;
//! use crate::components::ui::radio_group::RadioGroupItem;
//!
//! view! {
//!     <RadioGroup label="Density" default_value="comfortable">
//!         <RadioGroupItem value="default">"Default"</RadioGroupItem>
//!         <RadioGroupItem value="comfortable">"Comfortable"</RadioGroupItem>
//!         <RadioGroupItem value="compact">"Compact"</RadioGroupItem>
//!     </RadioGroup>
//! }
//! ```

use leptos::prelude::*;
use leptos_aria::BoxedRadioChangeCallback;
use leptos_aria::Radio;
use leptos_aria::RadioGroup as AriaRadioGroup;

use super::utils::cn;

/// The group root. Its optional `label` renders as the first `span`.
const GROUP: &str = "grid gap-3 [&>span]:text-sm [&>span]:leading-none [&>span]:font-medium";

/// The item's label root. The native radio stays in the tree for keyboard,
/// forms, and assistive technology but is visually hidden; the circle below
/// reads its state through `has-*` variants.
const ITEM: &str = "group/radio relative inline-flex items-center gap-2 text-sm leading-none \
                    font-medium select-none has-disabled:cursor-not-allowed \
                    has-disabled:opacity-50 [&>input]:sr-only";

const CONTROL: &str = "relative aspect-square size-4 shrink-0 rounded-full border border-input \
                       shadow-xs transition-[color,box-shadow] dark:bg-input/30 \
                       group-has-focus-visible/radio:border-ring \
                       group-has-focus-visible/radio:ring-3 \
                       group-has-focus-visible/radio:ring-ring/50";

const INDICATOR: &str = "absolute top-1/2 left-1/2 hidden size-2 -translate-x-1/2 \
                         -translate-y-1/2 rounded-full bg-primary \
                         group-has-checked/radio:block";

/// A group of radio items. Selection and the shared input name come from
/// `leptos_aria::RadioGroup`; the native radios move with the arrow keys.
#[component]
pub fn RadioGroup(
	/// A visible label naming the group.
	#[prop(optional, into)]
	label: Option<String>,
	/// The value selected at first (uncontrolled).
	#[prop(optional, into)]
	default_value: Option<String>,
	/// The selected value (controlled).
	#[prop(optional, into)]
	value: Option<Signal<Option<String>>>,
	/// Called with the new value when the selection changes.
	#[prop(optional)]
	on_change: Option<BoxedRadioChangeCallback>,
	/// Whether the whole group is disabled.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	/// Whether the whole group is read only.
	#[prop(optional, into)]
	is_read_only: Signal<bool>,
	/// The input name used when submitting a form. One is generated otherwise.
	#[prop(optional, into)]
	name: Option<String>,
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let class = Signal::derive(move || class.with(|class| cn(&[GROUP, class])));

	view! {
		<AriaRadioGroup
			nostrip:label=label
			nostrip:default_value=default_value
			nostrip:value=value
			nostrip:on_change=on_change
			is_disabled=is_disabled
			is_read_only=is_read_only
			nostrip:name=name
			class=class
			attr:data-slot="radio-group"
		>
			{children()}
		</AriaRadioGroup>
	}
}

/// One option in a [`RadioGroup`]. The children are its label.
#[component]
pub fn RadioGroupItem(
	/// The value the group reports when this item is selected.
	#[prop(into)]
	value: String,
	/// Whether this item is disabled.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let class = Signal::derive(move || class.with(|class| cn(&[ITEM, class])));

	view! {
		<Radio value=value is_disabled=is_disabled class=class attr:data-slot="radio-group-item">
			<span data-slot="radio-group-control" class=CONTROL>
				<span data-slot="radio-group-indicator" class=INDICATOR></span>
			</span>
			{children()}
		</Radio>
	}
}
