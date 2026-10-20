//! The path to the current page through a hierarchy of links.
//!
//! ```rust ignore
//! use crate::components::ui::breadcrumb::*;
//!
//! view! {
//!     <Breadcrumb>
//!         <BreadcrumbItem href="/">"Home"</BreadcrumbItem>
//!         <BreadcrumbSeparator />
//!         <BreadcrumbItem href="/docs">"Docs"</BreadcrumbItem>
//!         <BreadcrumbSeparator />
//!         <BreadcrumbItem is_current=true>"Breadcrumb"</BreadcrumbItem>
//!     </Breadcrumb>
//! }
//! ```

use leptos::prelude::*;
use leptos_aria::BreadcrumbItem as AriaBreadcrumbItem;
use leptos_aria::Breadcrumbs;

use super::utils::cn;

// The headless trail is a `nav` around an unstyled `ol`, so the list is
// styled from the `nav`.
const TRAIL: &str = "text-sm break-words text-muted-foreground [&>ol]:flex [&>ol]:flex-wrap \
                     [&>ol]:items-center [&>ol]:gap-1.5 sm:[&>ol]:gap-2.5";

const ITEM: &str = "inline-flex items-center gap-1.5 [&>a]:transition-colors \
                    [&>a]:hover:text-foreground [&>[aria-current=page]]:font-normal \
                    [&>[aria-current=page]]:text-foreground";

/// The breadcrumb trail: a labelled `nav` around an ordered list.
#[component]
pub fn Breadcrumb(
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let class = Signal::derive(move || class.with(|class| cn(&[TRAIL, class])));

	view! {
		<Breadcrumbs attr:data-slot="breadcrumb" class=class>
			{children()}
		</Breadcrumbs>
	}
}

/// One step in the trail: a link, or the current page.
#[component]
pub fn BreadcrumbItem(
	/// Where the step links to. Without it the step renders as text.
	#[prop(optional, into)]
	href: Option<String>,
	/// Marks the step as the current page.
	#[prop(optional, into)]
	is_current: Signal<bool>,
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let class = Signal::derive(move || class.with(|class| cn(&[ITEM, class])));

	match href {
		Some(href) => view! {
			<AriaBreadcrumbItem attr:data-slot="breadcrumb-item" href=href is_current=is_current class=class>
				{children()}
			</AriaBreadcrumbItem>
		}
		.into_any(),
		None => {
			view! {
				<AriaBreadcrumbItem attr:data-slot="breadcrumb-item" is_current=is_current class=class>
					{children()}
				</AriaBreadcrumbItem>
			}
			.into_any()
		}
	}
}

/// The divider between two steps. Hidden from assistive technology.
#[component]
pub fn BreadcrumbSeparator(
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
) -> impl IntoView {
	let class = move || class.with(|class| cn(&["[&>svg]:size-3.5", class]));

	view! {
		<li data-slot="breadcrumb-separator" role="presentation" aria-hidden="true" class=class>
			<svg
				xmlns="http://www.w3.org/2000/svg"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<path d="m9 18 6-6-6-6" />
			</svg>
		</li>
	}
}
