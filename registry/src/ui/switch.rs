//! An on/off switch with its label.
//!
//! ```rust ignore
//! use crate::components::ui::switch::Switch;
//!
//! let airplane = RwSignal::new(false);
//!
//! view! {
//!     <Switch is_checked=airplane on_change=Box::new(move |on| airplane.set(on))>
//!         "Airplane mode"
//!     </Switch>
//! }
//! ```

use leptos::prelude::*;
use leptos_aria::BoxedSelectionChangeCallback;
use leptos_aria::Switch as AriaSwitch;

use super::utils::cn;

/// The label root. The native `role="switch"` checkbox stays in the tree for
/// keyboard and assistive technology but is visually hidden; the control
/// below reads its state through `has-*` variants.
const BASE: &str = "group/switch relative inline-flex items-center gap-2 text-sm leading-none \
                    font-medium select-none has-disabled:cursor-not-allowed \
                    has-disabled:opacity-50 [&>input]:sr-only";

const CONTROL: &str = "inline-flex h-[1.15rem] w-8 shrink-0 items-center rounded-full border \
                       border-transparent shadow-xs transition-all \
                       group-has-checked/switch:bg-primary \
                       group-has-[input:not(:checked)]/switch:bg-input \
                       dark:group-has-[input:not(:checked)]/switch:bg-input/80 \
                       group-has-focus-visible/switch:border-ring \
                       group-has-focus-visible/switch:ring-3 \
                       group-has-focus-visible/switch:ring-ring/50";

const THUMB: &str = "pointer-events-none block size-4 translate-x-0 rounded-full bg-background \
                     ring-0 transition-transform group-has-checked/switch:translate-x-[calc(100%-2px)] \
                     dark:group-has-[input:not(:checked)]/switch:bg-foreground \
                     dark:group-has-checked/switch:bg-primary-foreground";

/// An on/off switch. The children are its label, so clicking them toggles it.
/// State, `role="switch"`, and the `data-state` attribute on the input come
/// from `leptos_aria::Switch`.
#[component]
pub fn Switch(
	/// Whether the switch starts on (uncontrolled).
	#[prop(optional, into)]
	default_checked: Signal<bool>,
	/// Whether the switch is on (controlled).
	#[prop(optional, into)]
	is_checked: Option<Signal<bool>>,
	/// Called with the new state when the switch is toggled.
	#[prop(optional)]
	on_change: Option<BoxedSelectionChangeCallback>,
	/// Whether the switch is disabled.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let class = Signal::derive(move || class.with(|class| cn(&[BASE, class])));

	view! {
		<AriaSwitch
			default_checked=default_checked
			nostrip:is_checked=is_checked
			nostrip:on_change=on_change
			is_disabled=is_disabled
			class=class
			attr:data-slot="switch"
		>
			<span data-slot="switch-control" class=CONTROL>
				<span data-slot="switch-thumb" class=THUMB></span>
			</span>
			{children()}
		</AriaSwitch>
	}
}
