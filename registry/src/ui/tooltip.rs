//! A short label that appears above an element on hover or keyboard focus.
//!
//! ```rust ignore
//! use crate::components::ui::tooltip::Tooltip;
//!
//! view! {
//!     <Tooltip label="Add to library">
//!         <button class="rounded-md border px-4 py-2">"Add"</button>
//!     </Tooltip>
//! }
//! ```

use leptos::prelude::*;
use leptos_aria::Tooltip as AriaTooltip;
use leptos_aria::TooltipTrigger;

use super::utils::cn;

// `ml-[50%]` and `-translate-x-1/2` centre the label over the trigger: the
// headless tooltip pins it to the wrapper's left edge.
const CONTENT: &str = "z-50 mb-1.5 ml-[50%] w-max max-w-xs -translate-x-1/2 rounded-md \
                       bg-primary px-3 py-1.5 text-xs text-balance text-primary-foreground \
                       transition-opacity starting:opacity-0";

/// Wraps an element and describes it with a tooltip after a short delay.
#[component]
pub fn Tooltip(
	/// The tooltip text. It also becomes the trigger's accessible
	/// description.
	#[prop(into)]
	label: String,
	/// Whether pointer hover or focus opens the tooltip. Keyboard focus
	/// inside the trigger opens it either way.
	#[prop(default = TooltipTrigger::Hover.into(), into)]
	trigger: Signal<TooltipTrigger>,
	/// Classes merged over the tooltip label's defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let class = Signal::derive(move || class.with(|class| cn(&[CONTENT, class])));

	view! {
		<span
			data-slot="tooltip"
			class="relative inline-flex w-fit"
		>
			<AriaTooltip label=label trigger=trigger class=class>
				{children()}
			</AriaTooltip>
		</span>
	}
}
