//! A menu of actions that drops down from a button.
//!
//! ```rust ignore
//! use crate::components::ui::dropdown_menu::*;
//!
//! view! {
//!     <DropdownMenu>
//!         <DropdownMenuTrigger class="rounded-md border px-4 py-2">"Open"</DropdownMenuTrigger>
//!         <DropdownMenuContent label="Account">
//!             <DropdownMenuLabel>"My account"</DropdownMenuLabel>
//!             <DropdownMenuSeparator />
//!             <DropdownMenuItem value="profile" on_select=|| leptos::logging::log!("profile")>
//!                 "Profile"
//!             </DropdownMenuItem>
//!             <DropdownMenuItem value="billing" is_disabled=true>"Billing"</DropdownMenuItem>
//!         </DropdownMenuContent>
//!     </DropdownMenu>
//! }
//! ```

use leptos::context::Provider;
use leptos::prelude::*;
use leptos_aria::Menu;
use leptos_aria::MenuItem;
use leptos_aria::use_overlay_dismiss;
use leptos_aria::use_popup_focus;

use super::utils::cn;

// The menu renders its listbox's label in a `span` before the items; it is
// only announced.
const CONTENT: &str = "absolute top-full left-0 z-50 mt-1 max-h-96 min-w-[8rem] \
                       overflow-y-auto rounded-md border bg-popover p-1 \
                       text-popover-foreground shadow-md transition-opacity starting:opacity-0 \
                       [&>span]:sr-only [&_[role=menu]]:outline-none";

const ITEM: &str = "relative flex cursor-default items-center gap-2 rounded-sm px-2 py-1.5 \
                    text-sm outline-hidden select-none hover:bg-accent \
                    hover:text-accent-foreground aria-disabled:pointer-events-none \
                    aria-disabled:opacity-50 data-[focused=true]:bg-accent \
                    data-[focused=true]:text-accent-foreground *:flex *:items-center *:gap-2 \
                    [&_svg]:pointer-events-none [&_svg]:shrink-0 \
                    [&_svg:not([class*='size-'])]:size-4 \
                    [&_svg:not([class*='text-'])]:text-muted-foreground";

/// State shared by every part of one [`DropdownMenu`].
#[derive(Clone, Copy)]
struct MenuState {
	is_open: RwSignal<bool>,
	trigger: NodeRef<leptos::html::Button>,
}

/// The menu root. Escape or a click outside closes the menu, and focus
/// returns to the trigger when it closes.
#[component]
pub fn DropdownMenu(
	/// Whether the menu is open. Pass a signal to control it yourself;
	/// otherwise the menu keeps its own state, closed at first.
	#[prop(optional)]
	is_open: Option<RwSignal<bool>>,
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let state = MenuState {
		is_open: is_open.unwrap_or_else(|| RwSignal::new(false)),
		trigger: NodeRef::new(),
	};
	let root = NodeRef::new();
	let class = move || class.with(|class| cn(&["relative inline-block w-fit", class]));

	use_overlay_dismiss(
		root,
		state.is_open.into(),
		Box::new(move || state.is_open.set(false)),
	);
	use_popup_focus(root, state.trigger, state.is_open.into(), "[role=menu]");

	view! {
		<div data-slot="dropdown-menu" class=class node_ref=root>
			<Provider value=state>{children()}</Provider>
		</div>
	}
}

/// The button that opens and closes the menu. Unstyled, so pass button
/// classes.
#[component]
pub fn DropdownMenuTrigger(
	/// Stops the menu from opening.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let MenuState { is_open, trigger } = expect_context();

	view! {
		<button
			type="button"
			data-slot="dropdown-menu-trigger"
			aria-haspopup="menu"
			aria-expanded=move || is_open.get().to_string()
			disabled=move || is_disabled.get()
			class=move || class.get()
			node_ref=trigger
			on:click=move |_| is_open.update(|open| *open = !*open)
		>
			{children()}
		</button>
	}
}

/// The floating list of items, below the trigger. `leptos_aria::Menu`
/// highlights items with the arrow keys, runs one on a click, Enter, or
/// Space, and then closes.
#[component]
pub fn DropdownMenuContent(
	/// Names the menu for assistive technology.
	#[prop(optional, into)]
	label: Option<String>,
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let MenuState { is_open, .. } = expect_context();
	let class = Signal::derive(move || class.with(|class| cn(&[CONTENT, class])));

	view! {
		<Menu
			attr:data-slot="dropdown-menu-content"
			is_open=is_open
			nostrip:label=label
			class=class
		>
			{children()}
		</Menu>
	}
}

/// An action in the menu. Choosing it runs `on_select` and closes the menu.
#[component]
pub fn DropdownMenuItem(
	/// Identifies the item among its siblings.
	#[prop(into)]
	value: String,
	/// Stops the item from being chosen.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	/// Runs when the item is chosen.
	#[prop(optional, into)]
	on_select: Option<Callback<()>>,
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let class = Signal::derive(move || class.with(|class| cn(&[ITEM, class])));

	view! {
		<MenuItem
			attr:data-slot="dropdown-menu-item"
			value=value
			is_disabled=is_disabled
			nostrip:on_action=on_select
			class=class
		>
			{children()}
		</MenuItem>
	}
}

/// A heading for a group of items.
#[component]
pub fn DropdownMenuLabel(
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let class = move || class.with(|class| cn(&["px-2 py-1.5 text-sm font-medium", class]));

	view! {
		<div data-slot="dropdown-menu-label" role="presentation" class=class>
			{children()}
		</div>
	}
}

/// A line between groups of items.
#[component]
pub fn DropdownMenuSeparator(
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
) -> impl IntoView {
	let class = move || class.with(|class| cn(&["-mx-1 my-1 h-px bg-border", class]));

	view! { <div data-slot="dropdown-menu-separator" role="separator" class=class></div> }
}
