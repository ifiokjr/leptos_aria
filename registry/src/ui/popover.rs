//! Rich content in a floating panel, anchored to the button that opens it.
//! Escape or a click outside closes it.
//!
//! ```rust ignore
//! use crate::components::ui::popover::*;
//!
//! view! {
//!     <Popover>
//!         <PopoverTrigger class="rounded-md border px-4 py-2">"Dimensions"</PopoverTrigger>
//!         <PopoverContent>
//!             <p class="text-sm text-muted-foreground">"Set the dimensions for the layer."</p>
//!         </PopoverContent>
//!     </Popover>
//! }
//! ```

use std::sync::Arc;

use leptos::context::Provider;
use leptos::prelude::*;
use leptos_aria::Popover as AriaPopover;
use leptos_aria::PopoverPlacement;

use super::utils::cn;

const CONTENT: &str = "z-50 w-72 rounded-md border bg-popover p-4 text-popover-foreground \
                       shadow-md outline-hidden transition-opacity starting:opacity-0";

/// State shared by every part of one [`Popover`].
#[derive(Clone, Copy)]
struct PopoverState {
	is_open: RwSignal<bool>,
	trigger: NodeRef<leptos::html::Button>,
	/// Set when a press on the trigger also dismissed the open popover, so
	/// the click that follows does not reopen it.
	dismissed_by_trigger: StoredValue<bool>,
}

/// The popover root. Renders no element; it shares the open state with its
/// trigger and content.
#[component]
pub fn Popover(
	/// Whether the popover is open. Pass a signal to control it yourself;
	/// otherwise the popover keeps its own state, closed at first.
	#[prop(optional)]
	is_open: Option<RwSignal<bool>>,
	children: Children,
) -> impl IntoView {
	let state = PopoverState {
		is_open: is_open.unwrap_or_else(|| RwSignal::new(false)),
		trigger: NodeRef::new(),
		dismissed_by_trigger: StoredValue::new(false),
	};

	view! { <Provider value=state>{children()}</Provider> }
}

/// The button that toggles the popover and anchors its position. Unstyled,
/// so pass button classes.
#[component]
pub fn PopoverTrigger(
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let PopoverState {
		is_open,
		trigger,
		dismissed_by_trigger,
	} = expect_context();

	// An open popover closes on any press outside it, and the trigger is
	// outside it. This capture listener sits on the trigger itself, so it runs
	// before the document-level dismissal, while the popover is still open.
	view! {
		<button
			type="button"
			data-slot="popover-trigger"
			aria-haspopup="dialog"
			aria-expanded=move || is_open.get().to_string()
			class=move || class.get()
			node_ref=trigger
			on:pointerdown:capture=move |_| {
				dismissed_by_trigger.set_value(is_open.get_untracked());
			}
			on:click=move |_| {
				if dismissed_by_trigger.get_value() {
					dismissed_by_trigger.set_value(false);
				} else {
					is_open.update(|open| *open = !*open);
				}
			}
		>
			{children()}
		</button>
	}
}

/// The floating panel. It opens below the trigger and flips above it when
/// there is no room.
#[component]
pub fn PopoverContent(
	/// The preferred side of the trigger.
	#[prop(optional, into)]
	placement: Signal<PopoverPlacement>,
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let PopoverState {
		is_open, trigger, ..
	} = expect_context();
	let class = Signal::derive(move || class.with(|class| cn(&[CONTENT, class])));
	let anchor = Arc::new(move || trigger.get_untracked().map(Into::into));

	view! {
		<AriaPopover
			attr:data-slot="popover-content"
			is_open=is_open
			anchor=anchor
			placement=placement
			class=class
		>
			{children()}
		</AriaPopover>
	}
}
