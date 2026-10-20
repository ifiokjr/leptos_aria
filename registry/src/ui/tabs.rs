//! Layered panels of content, one visible at a time, switched by a row of
//! tabs.
//!
//! ```rust ignore
//! use crate::components::ui::tabs::*;
//!
//! view! {
//!     <Tabs default_selected_key="account">
//!         <TabsList attr:aria-label="Settings">
//!             <TabsTrigger key="account">"Account"</TabsTrigger>
//!             <TabsTrigger key="password">"Password"</TabsTrigger>
//!         </TabsList>
//!         <TabsContent key="account">"Change your account details."</TabsContent>
//!         <TabsContent key="password">"Change your password."</TabsContent>
//!     </Tabs>
//! }
//! ```
//!
//! Tab and panel ids derive from their keys, so keys must be unique across
//! every set of tabs on the page.

use leptos::context::Provider;
use leptos::prelude::*;
use leptos_aria::Tab;
use leptos_aria::TabPanel;
use leptos_aria::Tabs as AriaTabs;

use super::utils::cn;

const LIST: &str = "inline-flex h-9 w-fit items-center justify-center rounded-lg bg-muted p-[3px] \
                    text-muted-foreground";

const TRIGGER: &str = "inline-flex h-[calc(100%-1px)] flex-1 items-center justify-center gap-1.5 \
                       rounded-md border border-transparent px-2 py-1 text-sm font-medium \
                       whitespace-nowrap text-foreground transition-[color,box-shadow] \
                       focus-visible:border-ring focus-visible:ring-ring/50 \
                       focus-visible:ring-3 focus-visible:outline-1 \
                       focus-visible:outline-ring aria-disabled:pointer-events-none \
                       aria-disabled:opacity-50 aria-selected:bg-background aria-selected:shadow-sm \
                       dark:text-muted-foreground dark:aria-selected:border-input \
                       dark:aria-selected:bg-input/30 dark:aria-selected:text-foreground \
                       [&_svg]:pointer-events-none [&_svg]:shrink-0 \
                       [&_svg:not([class*='size-'])]:size-4";

/// The tab selected on first render, shared from [`Tabs`] with its
/// [`TabsList`].
#[derive(Clone)]
struct DefaultSelectedKey(String);

/// The tabs root. Holds the list of tabs and their panels.
#[component]
pub fn Tabs(
	/// The key of the tab selected on first render.
	#[prop(into)]
	default_selected_key: String,
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let class = move || class.with(|class| cn(&["flex flex-col gap-2", class]));

	// The provider scopes the headless tab state to this set of tabs, so
	// sibling `Tabs` never read each other's selection.
	view! {
		<div data-slot="tabs" class=class>
			<Provider value=DefaultSelectedKey(default_selected_key)>{children()}</Provider>
		</div>
	}
}

/// The row of tabs, rendered as the `role="tablist"`. Arrow keys, Home, and
/// End move the selection.
#[component]
pub fn TabsList(
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let DefaultSelectedKey(default_selected_key) = expect_context::<DefaultSelectedKey>();
	let class = Signal::derive(move || class.with(|class| cn(&[LIST, class])));

	view! {
		<AriaTabs
			attr:data-slot="tabs-list"
			default_selected_key=default_selected_key
			class=class
		>
			{children()}
		</AriaTabs>
	}
}

/// A tab that shows the panel with the same key.
#[component]
pub fn TabsTrigger(
	/// Matches the key of the [`TabsContent`] this tab shows.
	#[prop(into)]
	key: String,
	/// Marks the tab as unavailable.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let class = Signal::derive(move || class.with(|class| cn(&[TRIGGER, class])));

	view! {
		<Tab attr:data-slot="tabs-trigger" key=key is_disabled=is_disabled class=class>
			{children()}
		</Tab>
	}
}

/// The panel for one tab. Hidden while another tab is selected.
#[component]
pub fn TabsContent(
	/// Matches the key of the [`TabsTrigger`] that shows this panel.
	#[prop(into)]
	key: String,
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let class = Signal::derive(move || class.with(|class| cn(&["flex-1 outline-none", class])));

	view! {
		<TabPanel attr:data-slot="tabs-content" key=key class=class>
			{children()}
		</TabPanel>
	}
}
