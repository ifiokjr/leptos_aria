//! A modal that interrupts with an important message and waits for an answer.
//! Unlike a dialog, a click outside does not close it.
//!
//! ```rust ignore
//! use std::sync::Arc;
//!
//! use crate::components::ui::alert_dialog::AlertDialog;
//!
//! let is_open = RwSignal::new(false);
//!
//! view! {
//!     <button
//!         class="rounded-md bg-destructive px-4 py-2 text-sm font-medium text-white"
//!         on:click=move |_| is_open.set(true)
//!     >
//!         "Delete account"
//!     </button>
//!     <AlertDialog
//!         is_open=is_open
//!         title="Are you absolutely sure?"
//!         description="This permanently deletes your account and its data."
//!         on_confirm=Arc::new(|| leptos::logging::log!("deleted"))
//!     />
//! }
//! ```

use std::sync::Arc;

use leptos::prelude::*;
use leptos_aria::AlertDialog as AriaAlertDialog;

use super::utils::cn;

// The headless alert dialog renders a full-screen layer holding a heading, a
// paragraph, and a row of buttons. The layer is the backdrop, and its three
// children stack into one centred panel.
const BASE: &str = "z-50 content-center bg-black/50 p-4 text-center sm:text-left *:w-full \
                    *:max-w-lg *:border-x *:bg-background *:px-6 [&>h2]:rounded-t-lg \
                    [&>h2]:border-t [&>h2]:pt-6 [&>h2]:pb-2 [&>h2]:text-lg \
                    [&>h2]:font-semibold [&>p]:pb-4 [&>p]:text-sm [&>p]:text-muted-foreground \
                    [&>div]:flex [&>div]:flex-col-reverse [&>div]:gap-2 [&>div]:rounded-b-lg \
                    [&>div]:border-b [&>div]:pt-2 [&>div]:pb-6 sm:[&>div]:flex-row \
                    sm:[&>div]:justify-end [&_button]:inline-flex [&_button]:h-9 \
                    [&_button]:items-center [&_button]:justify-center [&_button]:rounded-md \
                    [&_button]:px-4 [&_button]:py-2 [&_button]:text-sm [&_button]:font-medium \
                    [&_button]:whitespace-nowrap [&_button]:transition-all \
                    [&_button]:outline-none [&_button]:focus-visible:ring-ring/50 \
                    [&_button]:focus-visible:ring-3 \
                    [&_[data-action=cancel]]:border [&_[data-action=cancel]]:bg-background \
                    [&_[data-action=cancel]]:shadow-xs \
                    [&_[data-action=cancel]]:hover:bg-accent \
                    [&_[data-action=cancel]]:hover:text-accent-foreground \
                    [&_[data-action=confirm]]:bg-primary \
                    [&_[data-action=confirm]]:text-primary-foreground \
                    [&_[data-action=confirm]]:shadow-xs \
                    [&_[data-action=confirm]]:hover:bg-primary/90";

/// A modal that asks the user to confirm or cancel. Escape cancels.
#[component]
pub fn AlertDialog(
	/// Whether the dialog is open. Both buttons close it.
	is_open: RwSignal<bool>,
	/// The question the dialog asks.
	#[prop(into)]
	title: String,
	/// What happens if the user confirms.
	#[prop(into)]
	description: String,
	/// The cancel button's text. Defaults to "Cancel".
	#[prop(optional, into)]
	cancel_label: Option<String>,
	/// The confirm button's text. Defaults to "Continue".
	#[prop(optional, into)]
	confirm_label: Option<String>,
	/// Runs when the user confirms, before the dialog closes.
	#[prop(optional)]
	on_confirm: Option<Arc<dyn Fn() + Send + Sync>>,
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
) -> impl IntoView {
	let class = Signal::derive(move || class.with(|class| cn(&[BASE, class])));
	let on_confirm = on_confirm.unwrap_or_else(|| Arc::new(|| {}));

	view! {
		<AriaAlertDialog
			attr:data-slot="alert-dialog"
			is_open=is_open
			title=title
			description=description
			cancel_label=cancel_label.unwrap_or_else(|| "Cancel".to_string())
			confirm_label=confirm_label.unwrap_or_else(|| "Continue".to_string())
			on_confirm=on_confirm
			class=class
		/>
	}
}
