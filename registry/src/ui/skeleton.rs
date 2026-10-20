//! A pulsing placeholder shown while content loads.
//!
//! ```rust ignore
//! use crate::components::ui::skeleton::Skeleton;
//!
//! view! { <Skeleton class="h-4 w-48" /> }
//! ```

use leptos::prelude::*;

use super::utils::cn;

/// A pulsing placeholder shown while content loads. Size it with `class`.
#[component]
pub fn Skeleton(
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
) -> impl IntoView {
	let class = move || class.with(|class| cn(&["animate-pulse rounded-md bg-accent", class]));

	view! { <div data-slot="skeleton" aria-hidden="true" class=class></div> }
}
