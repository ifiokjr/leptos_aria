//! A modal window over the page. Focus stays inside while it is open, and
//! Escape or a click on the backdrop closes it.
//!
//! ```rust ignore
//! use crate::components::ui::dialog::*;
//!
//! view! {
//!     <Dialog>
//!         <DialogTrigger class="rounded-md border px-4 py-2 text-sm">
//!             "Edit profile"
//!         </DialogTrigger>
//!         <DialogContent>
//!             <DialogHeader>
//!                 <DialogTitle>"Edit profile"</DialogTitle>
//!                 <DialogDescription>"Make changes to your profile here."</DialogDescription>
//!             </DialogHeader>
//!             <input
//!                 aria-label="Name"
//!                 value="Ifiok"
//!                 class="h-9 rounded-md border border-input px-3 text-sm"
//!             />
//!             <DialogFooter>
//!                 <DialogClose class="rounded-md border px-4 py-2 text-sm">"Cancel"</DialogClose>
//!                 <button
//!                     type="submit"
//!                     class="rounded-md bg-primary px-4 py-2 text-sm text-primary-foreground"
//!                 >
//!                     "Save changes"
//!                 </button>
//!             </DialogFooter>
//!         </DialogContent>
//!     </Dialog>
//! }
//! ```
//!
//! Pass `is_open` to control the dialog yourself.

use leptos::context::Provider;
use leptos::prelude::*;
use leptos_aria::Dialog as AriaDialog;
use leptos_aria::Modal;
use leptos_aria::use_id;

use super::utils::cn;

const CONTENT: &str = "pointer-events-auto relative grid w-full max-w-[calc(100%-2rem)] gap-4 \
                       rounded-lg border bg-background p-6 shadow-lg transition-[opacity,scale] \
                       duration-200 starting:scale-95 starting:opacity-0 sm:max-w-lg";

const CLOSE: &str = "absolute top-4 right-4 rounded-xs opacity-70 ring-offset-background \
                     transition-opacity hover:opacity-100 focus:ring-2 focus:ring-ring \
                     focus:ring-offset-2 focus:outline-hidden disabled:pointer-events-none \
                     [&_svg]:pointer-events-none [&_svg]:shrink-0 \
                     [&_svg:not([class*='size-'])]:size-4";

/// Whether the dialog is open, shared by every part of one [`Dialog`].
#[derive(Clone, Copy)]
struct DialogState {
	is_open: RwSignal<bool>,
}

/// The ids that tie the dialog to its title and description.
#[derive(Clone)]
struct DialogLabels {
	title_id: String,
	description_id: String,
	has_description: RwSignal<bool>,
}

/// The dialog root. Renders no element; it shares the open state with its
/// trigger and content.
#[component]
pub fn Dialog(
	/// Whether the dialog is open. Pass a signal to control it yourself;
	/// otherwise the dialog keeps its own state, closed at first.
	#[prop(optional)]
	is_open: Option<RwSignal<bool>>,
	children: Children,
) -> impl IntoView {
	let state = DialogState {
		is_open: is_open.unwrap_or_else(|| RwSignal::new(false)),
	};

	view! { <Provider value=state>{children()}</Provider> }
}

/// A button that opens the dialog. Unstyled, so pass button classes.
#[component]
pub fn DialogTrigger(
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let DialogState { is_open } = expect_context();
	let class = move || class.get();

	view! {
		<button
			type="button"
			data-slot="dialog-trigger"
			aria-haspopup="dialog"
			aria-expanded=move || is_open.get().to_string()
			class=class
			on:click=move |_| is_open.set(true)
		>
			{children()}
		</button>
	}
}

/// The dialog panel, centred over a dimmed backdrop. Labelled by its
/// [`DialogTitle`] and described by its [`DialogDescription`].
#[component]
pub fn DialogContent(
	/// Show the close button in the top right corner.
	#[prop(default = true)]
	show_close_button: bool,
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let DialogState { is_open } = expect_context();
	let labels = DialogLabels {
		title_id: use_id(),
		description_id: use_id(),
		has_description: RwSignal::new(false),
	};
	let title_id = labels.title_id.clone();
	let described_by = {
		let DialogLabels {
			description_id,
			has_description,
			..
		} = labels.clone();

		move || has_description.get().then(|| description_id.clone())
	};
	let class = Signal::derive(move || class.with(|class| cn(&[CONTENT, class])));

	// The modal is a transparent full-screen layer that only centres the
	// panel. It ignores the pointer, so a click beside the panel lands on the
	// backdrop, which counts as outside and closes the dialog.
	view! {
		<Modal is_open=is_open role="presentation" class="pointer-events-none z-50">
			<AriaDialog
				attr:data-slot="dialog-content"
				attr:aria-modal="true"
				attr:aria-describedby=described_by
				aria_labelledby=title_id
				class=class
			>
				<Provider value=labels>{children()}</Provider>
				{show_close_button
					.then(|| {
						view! {
							<button
								type="button"
								data-slot="dialog-close"
								class=CLOSE
								on:click=move |_| is_open.set(false)
							>
								<svg
									xmlns="http://www.w3.org/2000/svg"
									viewBox="0 0 24 24"
									fill="none"
									stroke="currentColor"
									stroke-width="2"
									stroke-linecap="round"
									stroke-linejoin="round"
									aria-hidden="true"
								>
									<path d="M18 6 6 18" />
									<path d="m6 6 12 12" />
								</svg>
								<span class="sr-only">"Close"</span>
							</button>
						}
					})}
			</AriaDialog>
		</Modal>
	}
}

/// A button that closes the dialog. Unstyled, so pass button classes.
#[component]
pub fn DialogClose(
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let DialogState { is_open } = expect_context();
	let class = move || class.get();

	view! {
		<button
			type="button"
			data-slot="dialog-close"
			class=class
			on:click=move |_| is_open.set(false)
		>
			{children()}
		</button>
	}
}

/// Groups the title and description at the top of the dialog.
#[component]
pub fn DialogHeader(
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let class =
		move || class.with(|class| cn(&["flex flex-col gap-2 text-center sm:text-left", class]));

	view! { <div data-slot="dialog-header" class=class>{children()}</div> }
}

/// Actions at the bottom of the dialog, stacked on small screens.
#[component]
pub fn DialogFooter(
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let class = move || {
		class.with(|class| {
			cn(&[
				"flex flex-col-reverse gap-2 sm:flex-row sm:justify-end",
				class,
			])
		})
	};

	view! { <div data-slot="dialog-footer" class=class>{children()}</div> }
}

/// The dialog's heading. It names the dialog for assistive technology.
#[component]
pub fn DialogTitle(
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let DialogLabels { title_id, .. } = expect_context();
	let class = move || class.with(|class| cn(&["text-lg leading-none font-semibold", class]));

	view! {
		<h2 data-slot="dialog-title" id=title_id class=class>
			{children()}
		</h2>
	}
}

/// Supporting text under the title. It describes the dialog for assistive
/// technology.
#[component]
pub fn DialogDescription(
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let DialogLabels {
		description_id,
		has_description,
		..
	} = expect_context();
	let class = move || class.with(|class| cn(&["text-sm text-muted-foreground", class]));

	// The content only points `aria-describedby` here while this exists.
	has_description.set(true);
	on_cleanup(move || {
		has_description.try_set(false);
	});

	view! {
		<p data-slot="dialog-description" id=description_id class=class>
			{children()}
		</p>
	}
}
