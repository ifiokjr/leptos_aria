//! Vertically stacked sections that each expand to reveal their content.
//!
//! ```rust ignore
//! use crate::components::ui::accordion::*;
//!
//! view! {
//!     <Accordion default_open=vec!["shipping".to_string()]>
//!         <AccordionItem key="shipping">
//!             <AccordionTrigger>"Shipping"</AccordionTrigger>
//!             <AccordionContent>"Orders ship within two working days."</AccordionContent>
//!         </AccordionItem>
//!         <AccordionItem key="returns">
//!             <AccordionTrigger>"Returns"</AccordionTrigger>
//!             <AccordionContent>"Return anything within 30 days."</AccordionContent>
//!         </AccordionItem>
//!     </Accordion>
//! }
//! ```

use leptos::prelude::*;
use leptos_aria::Accordion as AriaAccordion;
use leptos_aria::AccordionItem as AriaAccordionItem;
use leptos_aria::AccordionMode;
use leptos_aria::AccordionPanel;
use leptos_aria::AccordionTrigger as AriaAccordionTrigger;

use super::utils::cn;

const TRIGGER: &str = "flex flex-1 items-start justify-between gap-4 rounded-md py-4 text-left \
                       text-sm font-medium transition-all outline-none hover:underline \
                       focus-visible:border-ring focus-visible:ring-ring/50 \
                       focus-visible:ring-3 disabled:pointer-events-none disabled:opacity-50 \
                       [&[data-state=open]>svg]:rotate-180";

/// The accordion root. Tracks which sections are open.
#[component]
pub fn Accordion(
	/// Whether one or many sections can be open at once.
	#[prop(optional, into)]
	mode: Signal<AccordionMode>,
	/// The keys of the sections open on first render.
	#[prop(optional, into)]
	default_open: Vec<String>,
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	view! {
		<AriaAccordion attr:data-slot="accordion" mode=mode default_open=default_open class=class>
			{children()}
		</AriaAccordion>
	}
}

/// One section: a trigger and the content it reveals.
#[component]
pub fn AccordionItem(
	/// Identifies the section among its siblings.
	#[prop(into)]
	key: String,
	/// Stops the section from opening or closing.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let class =
		Signal::derive(move || class.with(|class| cn(&["border-b last:border-b-0", class])));

	view! {
		<AriaAccordionItem attr:data-slot="accordion-item" key=key is_disabled=is_disabled class=class>
			{children()}
		</AriaAccordionItem>
	}
}

/// The button that opens and closes its section, with a chevron that turns
/// when the section is open.
#[component]
pub fn AccordionTrigger(
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let class = Signal::derive(move || class.with(|class| cn(&[TRIGGER, class])));

	view! {
		<h3 data-slot="accordion-header" class="flex">
			<AriaAccordionTrigger attr:data-slot="accordion-trigger" class=class>
				{children()}
				<svg
					class="pointer-events-none size-4 shrink-0 translate-y-0.5 text-muted-foreground transition-transform duration-200"
					xmlns="http://www.w3.org/2000/svg"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
					aria-hidden="true"
				>
					<path d="m6 9 6 6 6-6" />
				</svg>
			</AriaAccordionTrigger>
		</h3>
	}
}

/// The content a section reveals. Hidden while the section is closed.
#[component]
pub fn AccordionContent(
	/// Classes merged over the inner wrapper's defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let class = move || class.with(|class| cn(&["pt-0 pb-4", class]));

	view! {
		<AccordionPanel attr:data-slot="accordion-content" class="overflow-hidden text-sm">
			<div class=class>{children()}</div>
		</AccordionPanel>
	}
}
