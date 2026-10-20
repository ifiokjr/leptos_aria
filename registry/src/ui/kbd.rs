//! A keyboard key or shortcut.
//!
//! ```rust ignore
//! use crate::components::ui::kbd::Kbd;
//!
//! view! { <p>"Press " <Kbd>"⌘"</Kbd> <Kbd>"K"</Kbd> " to search."</p> }
//! ```

use leptos::prelude::*;

use super::utils::cn;

const BASE: &str = "pointer-events-none inline-flex h-5 w-fit min-w-5 items-center justify-center \
                    gap-1 rounded-sm bg-muted px-1 font-sans text-xs font-medium \
                    text-muted-foreground select-none [&_svg:not([class*='size-'])]:size-3";

/// A keyboard key or shortcut.
#[component]
pub fn Kbd(
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let class = move || class.with(|class| cn(&[BASE, class]));

	view! { <kbd data-slot="kbd" class=class>{children()}</kbd> }
}
