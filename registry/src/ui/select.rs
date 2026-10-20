//! A button that opens a list of options and shows the one chosen.
//!
//! ```rust ignore
//! use leptos_aria::SelectBoxItem;
//!
//! use crate::components::ui::select::Select;
//!
//! let fruit = RwSignal::new(None::<String>);
//!
//! view! {
//!     <Select
//!         label="Fruit"
//!         placeholder="Select a fruit"
//!         selected_key=fruit
//!         options=vec![
//!             SelectBoxItem { value: "apple".into(), label: "Apple".into() },
//!             SelectBoxItem { value: "banana".into(), label: "Banana".into() },
//!         ]
//!     />
//! }
//! ```

use leptos::prelude::*;
use leptos_aria::BoxedSelectBoxChangeCallback;
use leptos_aria::SelectBox;
use leptos_aria::SelectBoxItem;

use super::utils::cn;

/// The trigger. `leptos_aria` renders the chosen label, or the placeholder,
/// in its first `span` and sets `data-placeholder` while nothing is chosen.
const TRIGGER: &str = "flex h-9 w-full items-center justify-between gap-2 rounded-md border \
                       border-input bg-transparent px-3 py-2 text-sm whitespace-nowrap shadow-xs \
                       transition-[color,box-shadow] outline-none focus-visible:border-ring \
                       focus-visible:ring-ring/50 focus-visible:ring-3 \
                       disabled:cursor-not-allowed disabled:opacity-50 \
                       aria-invalid:border-destructive aria-invalid:ring-destructive/20 \
                       data-[placeholder]:text-muted-foreground dark:bg-input/30 \
                       dark:hover:bg-input/50 dark:aria-invalid:ring-destructive/40 \
                       [&>span]:line-clamp-1 [&_svg]:pointer-events-none [&_svg]:size-4 \
                       [&_svg]:shrink-0 [&_svg]:opacity-50";

// The listbox renders its label in a `span` beside it; the trigger already
// shows the value, so the label is only announced.
const CONTENT: &str = "absolute top-full left-0 z-50 mt-1 max-h-72 w-max min-w-full \
                       overflow-y-auto rounded-md border bg-popover p-1 \
                       text-popover-foreground shadow-md transition-opacity starting:opacity-0 \
                       [&>span]:sr-only [&_[role=listbox]]:outline-none";

const ITEM: &str = "group relative flex w-full cursor-default items-center gap-2 rounded-sm py-1.5 \
                    pr-8 pl-2 text-sm outline-hidden select-none hover:bg-accent \
                    hover:text-accent-foreground aria-disabled:pointer-events-none \
                    aria-disabled:opacity-50 data-[focused=true]:bg-accent \
                    data-[focused=true]:text-accent-foreground";

/// A single-choice select. Opening, closing, focus, and the chosen value
/// come from `leptos_aria::SelectBox`.
#[component]
pub fn Select(
	/// Names the select for assistive technology.
	#[prop(into)]
	label: String,
	/// The choices, in order.
	#[prop(into)]
	options: Vec<SelectBoxItem>,
	/// The chosen option's value. Pass a signal to read or set it; otherwise
	/// the select keeps its own, empty at first.
	#[prop(optional)]
	selected_key: Option<RwSignal<Option<String>>>,
	/// Shown on the trigger until an option is chosen. Defaults to "Select…".
	#[prop(optional, into)]
	placeholder: Option<String>,
	/// Stops the select from opening.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
) -> impl IntoView {
	let value: Option<Signal<Option<String>>> = selected_key.map(Into::into);
	let on_change = selected_key.map(|selected_key| {
		let on_change: BoxedSelectBoxChangeCallback =
			Box::new(move |value| selected_key.set(value));

		on_change
	});
	let class = move || class.with(|class| cn(&["relative inline-flex w-fit", class]));
	let check = ViewFn::from(|| {
		view! {
			<span class="absolute right-2 flex size-3.5 items-center justify-center invisible group-aria-selected:visible">
				<svg
					class="size-4"
					xmlns="http://www.w3.org/2000/svg"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
					aria-hidden="true"
				>
					<path d="M20 6 9 17l-5-5" />
				</svg>
			</span>
		}
	});

	view! {
		<SelectBox
			attr:data-slot="select"
			label=label
			options=options
			nostrip:value=value
			nostrip:on_change=on_change
			nostrip:placeholder=placeholder
			is_disabled=is_disabled
			class=class
			trigger_class=TRIGGER
			popup_class=CONTENT
			option_class=ITEM
			indicator=check
		>
			<svg
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
		</SelectBox>
	}
}
