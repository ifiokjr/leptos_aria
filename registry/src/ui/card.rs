//! A bordered surface that groups related content.
//!
//! ```rust ignore
//! use crate::components::ui::card::*;
//!
//! view! {
//!     <Card>
//!         <CardHeader>
//!             <CardTitle>"Notifications"</CardTitle>
//!             <CardDescription>"You have 3 unread messages."</CardDescription>
//!         </CardHeader>
//!         <CardContent>"..."</CardContent>
//!         <CardFooter>"..."</CardFooter>
//!     </Card>
//! }
//! ```

use leptos::prelude::*;

use super::utils::cn;

/// Render a `div` with a `data-slot` and merged classes.
macro_rules! slot {
	($(#[$doc:meta])* $name:ident, $slot:literal, $base:expr) => {
		$(#[$doc])*
		#[component]
		pub fn $name(
			/// Classes merged over the defaults; yours win conflicts.
			#[prop(optional, into)]
			class: Signal<String>,
			children: Children,
		) -> impl IntoView {
			let class = move || class.with(|class| cn(&[$base, class]));

			view! { <div data-slot=$slot class=class>{children()}</div> }
		}
	};
}

slot!(
	/// The card surface.
	Card,
	"card",
	"flex flex-col gap-6 rounded-xl border bg-card py-6 text-card-foreground shadow-sm"
);
slot!(
	/// The title, description, and optional action at the top of a card.
	CardHeader,
	"card-header",
	"@container/card-header grid auto-rows-min grid-rows-[auto_auto] items-start gap-1.5 px-6 \
	 has-data-[slot=card-action]:grid-cols-[1fr_auto] [.border-b]:pb-6"
);
slot!(
	/// The card's heading text.
	CardTitle,
	"card-title",
	"leading-none font-semibold"
);
slot!(
	/// Supporting text under the title.
	CardDescription,
	"card-description",
	"text-sm text-muted-foreground"
);
slot!(
	/// A control pinned to the header's top right.
	CardAction,
	"card-action",
	"col-start-2 row-span-2 row-start-1 self-start justify-self-end"
);
slot!(
	/// The card body.
	CardContent,
	"card-content",
	"px-6"
);
slot!(
	/// Actions at the bottom of a card.
	CardFooter,
	"card-footer",
	"flex items-center px-6 [.border-t]:pt-6"
);
