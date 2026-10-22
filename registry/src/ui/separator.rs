//! A thin line between groups of content.
//!
//! ```rust ignore
//! use crate::components::ui::separator::Separator;
//! use crate::components::ui::separator::SeparatorOrientation;
//!
//! view! {
//!     <div class="flex h-5 items-center gap-4 text-sm">
//!         "Blog"
//!         <Separator orientation=SeparatorOrientation::Vertical />
//!         "Docs"
//!     </div>
//!     <Separator class="my-4" />
//! }
//! ```

use leptos::prelude::*;
use leptos_aria::Separator as AriaSeparator;

use super::utils::cn;

const BASE: &str = "shrink-0 bg-border data-[orientation=horizontal]:h-px \
                    data-[orientation=horizontal]:w-full data-[orientation=vertical]:h-full \
                    data-[orientation=vertical]:w-px";

/// The direction a [`Separator`] runs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SeparatorOrientation {
	/// A full-width line between stacked content.
	#[default]
	Horizontal,
	/// A full-height line between content side by side.
	Vertical,
}

impl SeparatorOrientation {
	/// The `data-orientation` and `aria-orientation` value.
	pub const fn as_str(self) -> &'static str {
		match self {
			Self::Horizontal => "horizontal",
			Self::Vertical => "vertical",
		}
	}
}

/// A separator line. Decorative (`role="presentation"`) unless it has a
/// `label`, which makes it a labelled `role="separator"`. Roles and
/// `data-orientation` come from `leptos_aria::Separator`.
#[component]
pub fn Separator(
	/// The direction the line runs, horizontal by default.
	#[prop(optional, into)]
	orientation: Signal<SeparatorOrientation>,
	/// An accessible label, for a separator that divides meaningful groups.
	#[prop(optional, into)]
	label: Option<String>,
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
) -> impl IntoView {
	let class = Signal::derive(move || class.with(|class| cn(&[BASE, class])));
	let orientation = Signal::derive(move || orientation.get().as_str());

	view! {
		<AriaSeparator
			nostrip:label=label
			orientation=orientation
			class=class
			attr:data-slot="separator"
		/>
	}
}
