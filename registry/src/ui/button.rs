//! A button with the press, keyboard, and focus behaviour of `leptos_aria`.
//!
//! ```rust ignore
//! use crate::components::ui::button::Button;
//! use crate::components::ui::button::ButtonSize;
//! use crate::components::ui::button::ButtonVariant;
//! use leptos_aria::PressEvent;
//!
//! view! {
//!     <Button on_press=Box::new(|_: &PressEvent| logging::log!("saved"))>"Save"</Button>
//!     <Button variant=ButtonVariant::Outline size=ButtonSize::Sm>"Cancel"</Button>
//!     <Button variant=ButtonVariant::Destructive is_disabled=true>"Delete"</Button>
//! }
//! ```

use leptos::prelude::*;
use leptos_aria::BoxedPressEventCallback;
use leptos_aria::Button as AriaButton;
use leptos_aria::ButtonType;

use super::utils::cn;

const BASE: &str = "inline-flex shrink-0 items-center justify-center gap-2 rounded-md text-sm \
                    font-medium whitespace-nowrap transition-all outline-none \
                    focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 \
                    disabled:pointer-events-none disabled:opacity-50 \
                    aria-invalid:border-destructive aria-invalid:ring-destructive/20 \
                    dark:aria-invalid:ring-destructive/40 [&_svg]:pointer-events-none \
                    [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4";

/// The look of a [`Button`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonVariant {
	/// Solid primary colour, for the main action.
	#[default]
	Default,
	/// Signals a destructive action.
	Destructive,
	/// Border only.
	Outline,
	/// Quiet secondary colour.
	Secondary,
	/// No background until hovered.
	Ghost,
	/// Looks like a link.
	Link,
}

impl ButtonVariant {
	/// The classes this variant adds.
	pub const fn class(self) -> &'static str {
		match self {
			Self::Default => "bg-primary text-primary-foreground shadow-xs hover:bg-primary/90",
			Self::Destructive => {
				"bg-destructive text-white shadow-xs hover:bg-destructive/90 \
				 focus-visible:ring-destructive/20 dark:bg-destructive/60 \
				 dark:focus-visible:ring-destructive/40"
			}
			Self::Outline => {
				"border bg-background shadow-xs hover:bg-accent hover:text-accent-foreground \
				 dark:border-input dark:bg-input/30 dark:hover:bg-input/50"
			}
			Self::Secondary => {
				"bg-secondary text-secondary-foreground shadow-xs hover:bg-secondary/80"
			}
			Self::Ghost => "hover:bg-accent hover:text-accent-foreground dark:hover:bg-accent/50",
			Self::Link => "text-primary underline-offset-4 hover:underline",
		}
	}
}

/// The size of a [`Button`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonSize {
	/// The standard height.
	#[default]
	Default,
	/// Compact.
	Sm,
	/// Roomy.
	Lg,
	/// A square button for a single icon.
	Icon,
}

impl ButtonSize {
	/// The classes this size adds.
	pub const fn class(self) -> &'static str {
		match self {
			Self::Default => "h-9 px-4 py-2 has-[>svg]:px-3",
			Self::Sm => "h-8 gap-1.5 rounded-md px-3 has-[>svg]:px-2.5",
			Self::Lg => "h-10 rounded-md px-6 has-[>svg]:px-4",
			Self::Icon => "size-9",
		}
	}
}

/// A button. Presses, keyboard activation, and focus come from
/// `leptos_aria::Button`.
#[component]
pub fn Button(
	/// The look of the button.
	#[prop(optional, into)]
	variant: Signal<ButtonVariant>,
	/// The size of the button.
	#[prop(optional, into)]
	size: Signal<ButtonSize>,
	/// Whether the button is disabled.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	/// Called when a press is released over the button.
	#[prop(optional)]
	on_press: Option<BoxedPressEventCallback>,
	/// Called when a press starts.
	#[prop(optional)]
	on_press_start: Option<BoxedPressEventCallback>,
	/// Called when a press ends, over the button or not.
	#[prop(optional)]
	on_press_end: Option<BoxedPressEventCallback>,
	/// The behaviour of the button inside an HTML form, `button` by default.
	#[prop(optional, into)]
	r#type: Signal<ButtonType>,
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let class =
		move || class.with(|class| cn(&[BASE, variant.get().class(), size.get().class(), class]));

	view! {
		<AriaButton
			is_disabled=is_disabled
			nostrip:on_press=on_press
			nostrip:on_press_start=on_press_start
			nostrip:on_press_end=on_press_end
			r#type=r#type
			class=class
			attr:data-slot="button"
		>
			{children()}
		</AriaButton>
	}
}
