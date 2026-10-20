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

use std::collections::HashSet;

use leptos::prelude::*;
use leptos_aria::ListBox;
use leptos_aria::ListBoxOption;
use leptos_aria::SelectBoxItem;
use leptos_aria::Selection;
use leptos_aria::focus_safely;
use leptos_aria::use_overlay_dismiss;

use super::utils::cn;

const TRIGGER: &str = "flex h-9 w-full items-center justify-between gap-2 rounded-md border \
                       border-input bg-transparent px-3 py-2 text-sm whitespace-nowrap shadow-xs \
                       transition-[color,box-shadow] outline-none focus-visible:border-ring \
                       focus-visible:ring-ring/50 focus-visible:ring-3 \
                       disabled:cursor-not-allowed disabled:opacity-50 \
                       aria-invalid:border-destructive aria-invalid:ring-destructive/20 \
                       data-[placeholder]:text-muted-foreground dark:bg-input/30 \
                       dark:hover:bg-input/50 dark:aria-invalid:ring-destructive/40 \
                       *:data-[slot=select-value]:line-clamp-1 [&_svg]:pointer-events-none \
                       [&_svg]:size-4 [&_svg]:shrink-0 [&_svg]:opacity-50";

// The listbox renders its label in a `span` beside it; the trigger already
// shows the value, so the label is only announced.
const CONTENT: &str = "absolute top-full left-0 z-50 mt-1 max-h-72 w-max min-w-full \
                       overflow-y-auto rounded-md border bg-popover p-1 \
                       text-popover-foreground shadow-md transition-opacity starting:opacity-0 \
                       [&>span]:sr-only";

const ITEM: &str = "group relative flex w-full cursor-default items-center gap-2 rounded-sm py-1.5 \
                    pr-8 pl-2 text-sm outline-hidden select-none hover:bg-accent \
                    hover:text-accent-foreground aria-disabled:pointer-events-none \
                    aria-disabled:opacity-50 data-[focused=true]:bg-accent \
                    data-[focused=true]:text-accent-foreground";

/// A single-choice select. Opens a listbox of `options` below a trigger that
/// shows the chosen label.
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
	let selected_key = selected_key.unwrap_or_else(|| RwSignal::new(None));
	let placeholder = placeholder.unwrap_or_else(|| "Select…".to_string());
	let is_open = RwSignal::new(false);
	let root: NodeRef<leptos::html::Div> = NodeRef::new();
	let trigger: NodeRef<leptos::html::Button> = NodeRef::new();
	let options = StoredValue::new(options);

	// Closing returns focus to the trigger, unless it has already moved
	// somewhere outside the select.
	let close = move || {
		is_open.set(false);

		let focus_inside = root
			.get_untracked()
			.zip(document().active_element())
			.is_some_and(|(root, active)| root.contains(Some(active.as_ref())));

		if let Some(trigger) = trigger.get_untracked().filter(|_| focus_inside) {
			focus_safely(&trigger);
		}
	};

	use_overlay_dismiss(root, is_open.into(), Box::new(close));

	// Keyboard selection happens on the listbox, so it takes focus on open.
	Effect::new(move |_| {
		if !is_open.get() {
			return;
		}

		let listbox = root.get().and_then(|root| {
			root.query_selector("[role=listbox]")
				.expect("the selector is valid")
		});

		if let Some(listbox) = listbox {
			focus_safely(&listbox);
		}
	});

	let selected_label = move || {
		let key = selected_key.get()?;

		options.with_value(|options| {
			options
				.iter()
				.find(|option| option.value == key)
				.map(|option| option.label.clone())
		})
	};
	let selection = Signal::derive(move || {
		selected_key
			.get()
			.map_or(Selection::None, |key| Selection::Keys(HashSet::from([key])))
	});
	let on_change = Box::new(move |selection: &Selection| {
		let chosen = match selection {
			Selection::Keys(keys) => keys.iter().next().cloned(),
			Selection::None | Selection::All => None,
		};

		if chosen.is_some() {
			selected_key.set(chosen);
		}

		close();
	});
	let class = move || class.with(|class| cn(&["relative inline-flex w-fit", class]));

	view! {
		<div data-slot="select" class=class node_ref=root>
			<button
				type="button"
				data-slot="select-trigger"
				role="combobox"
				aria-label=label.clone()
				aria-haspopup="listbox"
				aria-expanded=move || is_open.get().to_string()
				data-placeholder=move || selected_label().is_none().then_some("")
				disabled=move || is_disabled.get()
				class=TRIGGER
				node_ref=trigger
				on:click=move |_| is_open.update(|open| *open = !*open)
			>
				<span data-slot="select-value">
					{move || selected_label().unwrap_or_else(|| placeholder.clone())}
				</span>
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
			</button>
			<div data-slot="select-content" class=CONTENT hidden=move || !is_open.get()>
				<ListBox label=label selection=selection on_change=on_change class="outline-none">
					{options
						.get_value()
						.into_iter()
						.map(|option| {
							// A click on the chosen option changes nothing, so
							// it closes here rather than in `on_change`.
							view! {
								<ListBoxOption
									attr:data-slot="select-item"
									value=option.value
									class=ITEM
									on:click:capture=move |_| close()
								>
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
									{option.label}
								</ListBoxOption>
							}
						})
						.collect_view()}
				</ListBox>
			</div>
		</div>
	}
}
