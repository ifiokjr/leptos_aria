//! A small status label.
//!
//! ```rust ignore
//! use crate::components::ui::badge::Badge;
//! use crate::components::ui::badge::BadgeVariant;
//!
//! view! {
//!     <Badge>"New"</Badge>
//!     <Badge variant=BadgeVariant::Secondary>"Draft"</Badge>
//!     <Badge variant=BadgeVariant::Destructive>"Failed"</Badge>
//!     <Badge variant=BadgeVariant::Outline>"v0.1.0"</Badge>
//! }
//! ```

use leptos::prelude::*;

use super::utils::cn;

const BASE: &str = "inline-flex w-fit shrink-0 items-center justify-center gap-1 overflow-hidden \
                    rounded-md border px-2 py-0.5 text-xs font-medium whitespace-nowrap \
                    transition-[color,box-shadow] focus-visible:border-ring \
                    focus-visible:ring-3 focus-visible:ring-ring/50 [&>svg]:pointer-events-none \
                    [&>svg]:size-3";

/// The look of a [`Badge`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgeVariant {
	/// Solid primary colour.
	#[default]
	Default,
	/// Quiet secondary colour.
	Secondary,
	/// Signals an error or a destructive state.
	Destructive,
	/// Border only.
	Outline,
}

impl BadgeVariant {
	/// The classes this variant adds.
	pub const fn class(self) -> &'static str {
		match self {
			Self::Default => "border-transparent bg-primary text-primary-foreground",
			Self::Secondary => "border-transparent bg-secondary text-secondary-foreground",
			Self::Destructive => {
				"border-transparent bg-destructive text-white focus-visible:ring-destructive/20 \
				 dark:bg-destructive/60 dark:focus-visible:ring-destructive/40"
			}
			Self::Outline => "text-foreground",
		}
	}
}

/// A small status label.
#[component]
pub fn Badge(
	/// The look of the badge.
	#[prop(optional, into)]
	variant: Signal<BadgeVariant>,
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let class = move || class.with(|class| cn(&[BASE, variant.get().class(), class]));

	view! { <span data-slot="badge" class=class>{children()}</span> }
}
