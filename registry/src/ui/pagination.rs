//! Page navigation: previous, next, and a button per page.
//!
//! ```rust ignore
//! use crate::components::ui::pagination::Pagination;
//!
//! let page = RwSignal::new(1_usize);
//!
//! view! {
//!     <Pagination page=page count=5_usize on_change=Box::new(move |next| page.set(next)) />
//! }
//! ```

use leptos::prelude::*;
use leptos_aria::BoxedPageChangeCallback;
use leptos_aria::Pagination as AriaPagination;

use super::utils::cn;

// The headless pagination is a `nav` of buttons, so they are styled from it.
const BASE: &str = "mx-auto flex w-full items-center justify-center gap-1 [&>button]:inline-flex \
                    [&>button]:size-9 [&>button]:items-center [&>button]:justify-center \
                    [&>button]:rounded-md [&>button]:border [&>button]:border-transparent \
                    [&>button]:text-sm [&>button]:font-medium [&>button]:transition-all \
                    [&>button]:outline-none [&>button]:hover:bg-accent \
                    [&>button]:hover:text-accent-foreground \
                    [&>button]:focus-visible:border-ring [&>button]:focus-visible:ring-ring/50 \
                    [&>button]:focus-visible:ring-3 [&>button]:disabled:pointer-events-none \
                    [&>button]:disabled:opacity-50 [&>button]:aria-[current=page]:border-border \
                    [&>button]:aria-[current=page]:bg-background \
                    [&>button]:aria-[current=page]:shadow-xs \
                    dark:[&>button]:aria-[current=page]:border-input \
                    dark:[&>button]:aria-[current=page]:bg-input/30";

/// Previous and next buttons around one button per page.
#[component]
pub fn Pagination(
	/// The current page, starting at 1.
	#[prop(into)]
	page: Signal<usize>,
	/// How many pages there are.
	#[prop(into)]
	count: Signal<usize>,
	/// Called with the page the user asks for. Update `page` here.
	on_change: BoxedPageChangeCallback,
	/// The navigation landmark's name. Defaults to "Pagination".
	#[prop(optional, into)]
	label: Option<String>,
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
) -> impl IntoView {
	let class = Signal::derive(move || class.with(|class| cn(&[BASE, class])));

	view! {
		<AriaPagination
			attr:data-slot="pagination"
			page=page
			count=count
			on_change=on_change
			label=label.unwrap_or_else(|| "Pagination".to_string())
			class=class
		/>
	}
}
