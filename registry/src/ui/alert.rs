//! A prominent message that draws attention without interrupting.
//!
//! ```rust ignore
//! use crate::components::ui::alert::*;
//!
//! view! {
//!     <Alert variant=AlertVariant::Destructive>
//!         <AlertTitle>"Payment failed"</AlertTitle>
//!         <AlertDescription>"Check your card details and try again."</AlertDescription>
//!     </Alert>
//! }
//! ```

use leptos::prelude::*;

use super::utils::cn;

const BASE: &str = "relative grid w-full grid-cols-[0_1fr] items-start gap-y-0.5 rounded-lg border \
                    px-4 py-3 text-sm has-[>svg]:grid-cols-[calc(var(--spacing)*4)_1fr] \
                    has-[>svg]:gap-x-3 [&>svg]:size-4 [&>svg]:translate-y-0.5 \
                    [&>svg]:text-current";

/// The look of an [`Alert`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AlertVariant {
	/// Neutral information.
	#[default]
	Default,
	/// An error or warning.
	Destructive,
}

impl AlertVariant {
	/// The classes this variant adds.
	pub const fn class(self) -> &'static str {
		match self {
			Self::Default => "bg-card text-card-foreground",
			Self::Destructive => {
				"bg-card text-destructive *:data-[slot=alert-description]:text-destructive/90 \
				 [&>svg]:text-current"
			}
		}
	}
}

/// A prominent message. Announced to screen readers with `role="alert"`.
#[component]
pub fn Alert(
	/// The look of the alert.
	#[prop(optional, into)]
	variant: Signal<AlertVariant>,
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let class = move || class.with(|class| cn(&[BASE, variant.get().class(), class]));

	view! {
		<div data-slot="alert" role="alert" class=class>
			{children()}
		</div>
	}
}

/// The alert's heading.
#[component]
pub fn AlertTitle(
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let class = move || {
		class.with(|class| {
			cn(&[
				"col-start-2 line-clamp-1 min-h-4 font-medium tracking-tight",
				class,
			])
		})
	};

	view! { <div data-slot="alert-title" class=class>{children()}</div> }
}

/// The alert's body text.
#[component]
pub fn AlertDescription(
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let class = move || {
		class.with(|class| {
			cn(&[
				"col-start-2 grid justify-items-start gap-1 text-sm text-muted-foreground \
				 [&_p]:leading-relaxed",
				class,
			])
		})
	};

	view! { <div data-slot="alert-description" class=class>{children()}</div> }
}
