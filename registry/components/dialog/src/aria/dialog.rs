//! Copy of `leptos_aria_overlay::Modal` + `Dialog` with styling hooks.

use leptos::prelude::*;
use leptos_aria::BoxedOverlayCloseCallback;
use leptos_aria::use_modal;
use leptos_aria::use_overlay_dismiss;
use leptos_aria::use_prevent_scroll;

/// Your dialog. Style the surface with Tailwind:
///
/// ```rust ignore
/// <Dialog is_open class="rounded-lg bg-white p-6">"Are you sure?"</Dialog>
/// ```
#[component]
pub fn Dialog(
	/// Whether the dialog is open.
	is_open: RwSignal<bool>,
	/// Your classes for the dialog surface.
	#[prop(optional, into)]
	class: Option<String>,
	/// The accessible label.
	#[prop(into)]
	label: String,
	/// Handler when the dialog should close.
	#[prop(optional)]
	on_close: Option<BoxedOverlayCloseCallback>,
	children: Children,
) -> impl IntoView {
	let overlay: NodeRef<leptos::html::Div> = NodeRef::new();

	use_prevent_scroll(Signal::from(is_open));
	use_modal(
		overlay,
		leptos_aria::UseModalProps {
			is_open: Signal::from(is_open),
		},
	);
	use_overlay_dismiss(
		overlay,
		Signal::from(is_open),
		on_close.unwrap_or_else(|| Box::new(move || is_open.set(false))),
	);

	view! {
		<div
			class="fixed inset-0 grid place-items-center bg-black/50"
			node_ref=overlay
			role="dialog"
			aria-modal="true"
			aria-label=label.clone()
			data-state=move || if is_open.get() { "open" } else { "closed" }
		>
			<div class=class.unwrap_or_default()>{children()}</div>
		</div>
	}
}
