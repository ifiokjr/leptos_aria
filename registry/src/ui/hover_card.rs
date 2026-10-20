//! A preview card that appears below an element while it is hovered or
//! focused, for sighted users to peek at what a link leads to.
//!
//! ```rust ignore
//! use crate::components::ui::hover_card::HoverCard;
//!
//! view! {
//!     <HoverCard
//!         label="@leptos"
//!         content=Box::new(|| {
//!             view! { <p class="text-sm">"The Rust framework for the web."</p> }.into_any()
//!         })
//!     >
//!         <a href="https://leptos.dev" class="text-sm font-medium underline underline-offset-4">
//!             "@leptos"
//!         </a>
//!     </HoverCard>
//! }
//! ```

use leptos::prelude::*;
use leptos_aria::HoverCard as AriaHoverCard;

use super::utils::cn;

const CONTENT: &str = "absolute top-full left-1/2 z-50 mt-2 flex w-64 -translate-x-1/2 flex-col \
                       gap-2 rounded-md border bg-popover p-4 text-sm font-semibold \
                       text-popover-foreground shadow-md outline-hidden transition-opacity \
                       starting:opacity-0 [&>*]:font-normal";

/// Wraps an element and shows a card below it on hover or focus.
#[component]
pub fn HoverCard(
	/// The card's first line, also announced as the trigger's description.
	#[prop(into)]
	label: String,
	/// Classes merged over the card's defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
	/// The element that opens the card.
	children: Children,
	/// The rest of the card, under the label.
	content: Children,
) -> impl IntoView {
	let class = Signal::derive(move || class.with(|class| cn(&[CONTENT, class])));

	view! {
		<span data-slot="hover-card" class="relative inline-block">
			<AriaHoverCard label=label class=class content=content>
				{children()}
			</AriaHoverCard>
		</span>
	}
}
