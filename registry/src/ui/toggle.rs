//! A two-state button that stays pressed until pressed again.
//!
//! ```rust ignore
//! use crate::components::ui::toggle::Toggle;
//! use crate::components::ui::toggle::ToggleVariant;
//!
//! let bold = RwSignal::new(false);
//!
//! view! {
//!     <Toggle
//!         variant=ToggleVariant::Outline
//!         is_selected=bold
//!         on_change=Box::new(move |selected| bold.set(selected))
//!         attr:aria-label="Bold"
//!     >
//!         "B"
//!     </Toggle>
//! }
//! ```

use leptos::prelude::*;
use leptos_aria::BoxedPressEventCallback;
use leptos_aria::BoxedSelectionChangeCallback;
use leptos_aria::ToggleButton;

use super::utils::cn;

/// The classes every toggle starts from, shared with the toggle group. The
/// pressed state comes from the `aria-pressed` attribute `leptos_aria`
/// renders.
pub const TOGGLE_BASE: &str = "inline-flex items-center justify-center gap-2 rounded-md text-sm \
                               font-medium whitespace-nowrap transition-[color,box-shadow] \
                               outline-none hover:bg-muted hover:text-muted-foreground \
                               focus-visible:border-ring focus-visible:ring-3 \
                               focus-visible:ring-ring/50 disabled:pointer-events-none \
                               disabled:opacity-50 aria-invalid:border-destructive \
                               aria-invalid:ring-destructive/20 \
                               dark:aria-invalid:ring-destructive/40 aria-pressed:bg-accent \
                               aria-pressed:text-accent-foreground [&_svg]:pointer-events-none \
                               [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4";

/// The look of a [`Toggle`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToggleVariant {
	/// No background until hovered or pressed.
	#[default]
	Default,
	/// A bordered toggle.
	Outline,
}

impl ToggleVariant {
	/// The classes this variant adds.
	pub const fn class(self) -> &'static str {
		match self {
			Self::Default => "bg-transparent",
			Self::Outline => {
				"border border-input bg-transparent shadow-xs hover:bg-accent \
				 hover:text-accent-foreground"
			}
		}
	}
}

/// The size of a [`Toggle`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToggleSize {
	/// The standard height.
	#[default]
	Default,
	/// Compact.
	Sm,
	/// Roomy.
	Lg,
}

impl ToggleSize {
	/// The classes this size adds.
	pub const fn class(self) -> &'static str {
		match self {
			Self::Default => "h-9 min-w-9 px-2",
			Self::Sm => "h-8 min-w-8 px-1.5",
			Self::Lg => "h-10 min-w-10 px-2.5",
		}
	}
}

/// A two-state button. The pressed state, `aria-pressed`, and press handling
/// come from `leptos_aria::ToggleButton`.
#[component]
pub fn Toggle(
	/// The look of the toggle.
	#[prop(optional, into)]
	variant: Signal<ToggleVariant>,
	/// The size of the toggle.
	#[prop(optional, into)]
	size: Signal<ToggleSize>,
	/// Whether the toggle is disabled.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	/// Whether the toggle starts pressed (uncontrolled).
	#[prop(optional, into)]
	default_selected: Signal<bool>,
	/// Whether the toggle is pressed (controlled).
	#[prop(optional, into)]
	is_selected: Option<Signal<bool>>,
	/// Called with the new state when the toggle is pressed.
	#[prop(optional)]
	on_change: Option<BoxedSelectionChangeCallback>,
	/// Called when a press is released over the toggle.
	#[prop(optional)]
	on_press: Option<BoxedPressEventCallback>,
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let class = move || {
		class.with(|class| {
			cn(&[
				TOGGLE_BASE,
				variant.get().class(),
				size.get().class(),
				class,
			])
		})
	};

	view! {
		<ToggleButton
			is_disabled=is_disabled
			default_selected=default_selected
			nostrip:is_selected=is_selected
			nostrip:on_change=on_change
			nostrip:on_press=on_press
			class=class
			attr:data-slot="toggle"
		>
			{children()}
		</ToggleButton>
	}
}
