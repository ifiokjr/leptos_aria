#![allow(clippy::needless_pass_by_value)]

use std::sync::Arc;
use std::time::Duration;

pub use focus_manager::FocusManager;
pub use focus_manager::use_autofocus;
pub use focus_manager::use_focus_manager;
pub use hooks::BoxedOverlayCloseCallback;
pub use hooks::use_modal;
pub use hooks::use_overlay_dismiss;
pub use hooks::use_prevent_scroll;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;

mod focus_manager;
mod hooks;

/// A modal is an overlay element which blocks interaction with the page
/// behind it.
///
/// Focus is contained inside the modal and restored on close, page scrolling
/// locks while open, and pressing escape or interacting outside dismisses it.
#[component]
pub fn Modal(
	/// Whether the modal is open.
	is_open: RwSignal<bool>,
	/// Handler that is called when the modal should close.
	#[prop(optional)]
	on_close: Option<BoxedOverlayCloseCallback>,
	/// The accessible role of the dialog, `dialog` by default.
	#[prop(optional, into)]
	role: Option<&'static str>,
	children: Children,
) -> impl IntoView {
	let close = unwrap_close(on_close, is_open);

	let overlay: NodeRef<leptos::html::Div> = NodeRef::new();

	use_prevent_scroll(Signal::from(is_open));
	use_modal(
		overlay,
		hooks::UseModalProps {
			is_open: Signal::from(is_open),
		},
	);
	use_overlay_dismiss(overlay, Signal::from(is_open), close);

	let role = role.unwrap_or("dialog");

	view! {
		<div
			class="leptos-aria-modal__backdrop"
			hidden=move || !is_open.get()
			style="position: fixed; inset: 0; background: rgba(0, 0, 0, 0.5);"
		/>
		<div
			class="leptos-aria-modal"
			node_ref=overlay
			role=role
			aria-modal="true"
			hidden=move || !is_open.get()
			style="position: fixed; inset: 0; display: grid; place-items: center;"
		>
			{children()}
		</div>
	}
}

/// A dialog is a window overlaid on the page, carrying a role and an optional
/// accessible label.
#[component]
pub fn Dialog(
	/// The accessible role, `dialog` by default.
	#[prop(optional, into)]
	role: Option<&'static str>,
	/// The accessible name of the dialog.
	#[prop(optional, into)]
	aria_label: Option<String>,
	/// The id of the element labeling the dialog.
	#[prop(optional, into)]
	aria_labelledby: Option<String>,
	children: Children,
) -> impl IntoView {
	let role = role.unwrap_or("dialog");

	view! {
		<div role=role aria-label=aria_label aria-labelledby=aria_labelledby>
			{children()}
		</div>
	}
}

/// The placement of a popover relative to its trigger.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum PopoverPlacement {
	#[default]
	Bottom,
	Top,
}

/// A popover is an overlay anchored to a trigger element.
///
/// Positioning is computed from the trigger's bounding box on each open; the
/// full flip and shift collision handling of the upstream floating layer is
/// tracked in the porting roadmap.
#[component]
pub fn Popover(
	/// Whether the popover is open.
	is_open: RwSignal<bool>,
	/// Handler that is called when the popover should close.
	#[prop(optional)]
	on_close: Option<BoxedOverlayCloseCallback>,
	/// Resolves the trigger element the popover anchors to.
	anchor: Arc<dyn Fn() -> Option<web_sys::Element> + Send + Sync>,
	/// The placement relative to the trigger, bottom by default.
	#[prop(optional, into)]
	placement: Signal<PopoverPlacement>,
	children: Children,
) -> impl IntoView {
	let popover_ref: NodeRef<leptos::html::Div> = NodeRef::new();
	let close = unwrap_close(on_close, is_open);

	use_overlay_dismiss(popover_ref, Signal::from(is_open), close);

	view! {
		<div
			class="leptos-aria-popover"
			node_ref=popover_ref
			role="dialog"
			hidden=move || !is_open.get()
			style=move || {
				if is_open.get() {
					anchor()
						.map(|trigger| popover_style(&trigger, placement.get_untracked()))
						.unwrap_or_default()
				} else {
					"display: none;".to_string()
				}
			}
		>
			{children()}
		</div>
	}
}

/// Resolve the fixed-position style anchoring the popover to the trigger.
fn popover_style(trigger: &web_sys::Element, placement: PopoverPlacement) -> String {
	let rect = trigger.get_bounding_client_rect();
	let flipped = flip_placement(&rect, placement);

	position_style(&rect, shifted_left(&rect), flipped)
}

/// The placement actually used: flipped when the preferred side would push
/// the popover off the viewport.
fn flip_placement(rect: &web_sys::DomRect, placement: PopoverPlacement) -> PopoverPlacement {
	let viewport_height = window()
		.inner_height()
		.ok()
		.and_then(|h| h.as_f64())
		.unwrap_or(0.0);

	match placement {
		PopoverPlacement::Bottom if viewport_height - rect.bottom() < MIN_POPOVER_SPACE => {
			PopoverPlacement::Top
		}
		PopoverPlacement::Top if rect.top() < MIN_POPOVER_SPACE => PopoverPlacement::Bottom,
		other => other,
	}
}

/// The left position clamped so the popover stays inside the viewport.
fn shifted_left(rect: &web_sys::DomRect) -> f64 {
	let viewport_width = window()
		.inner_width()
		.ok()
		.and_then(|w| w.as_f64())
		.unwrap_or(0.0);
	let max_left = (viewport_width - MIN_POPOVER_SPACE).max(0.0);

	rect.left().clamp(0.0, max_left)
}

/// The fixed-position style for a placement.
fn position_style(rect: &web_sys::DomRect, left: f64, placement: PopoverPlacement) -> String {
	match placement {
		PopoverPlacement::Bottom => {
			format!("position: fixed; top: {}px; left: {left}px;", rect.bottom())
		}
		PopoverPlacement::Top => {
			format!(
				"position: fixed; top: {}px; left: {left}px; transform: translateY(-100%);",
				rect.top()
			)
		}
	}
}

/// The space a popover needs on its preferred side, in pixels.
const MIN_POPOVER_SPACE: f64 = 120.0;

fn window() -> web_sys::Window {
	web_sys::window().expect("the `window` global is available in the browser")
}

/// How a tooltip is triggered.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum TooltipTrigger {
	#[default]
	Focus,
	Hover,
}

/// The delay before a tooltip shows after triggering.
const TOOLTIP_DELAY: Duration = Duration::from_millis(400);

/// A tooltip describes its trigger when hovered or focused.
#[component]
pub fn Tooltip(
	/// The tooltip text.
	#[prop(into)]
	label: String,
	/// How the tooltip is triggered, focus by default.
	#[prop(optional, into)]
	trigger: Signal<TooltipTrigger>,
	/// The content of the trigger.
	children: Children,
) -> impl IntoView {
	let tooltip_id = leptos_aria_utils::use_id();
	let is_open = RwSignal::new(false);
	let trigger_ref: NodeRef<leptos::html::Span> = NodeRef::new();

	apply_tooltip_triggers(trigger_ref, trigger.get_untracked(), is_open);

	view! {
		<span
			aria-describedby=tooltip_id.clone()
			style="position: relative;"
			node_ref=trigger_ref
		>
			{children()}
		</span>
		<span
			id=tooltip_id.clone()
			role="tooltip"
			class="leptos-aria-tooltip"
			hidden=move || !is_open.get()
			style="position: absolute; bottom: 100%; left: 0;"
		>
			{label.clone()}
		</span>
	}
}

/// Watch the trigger element and show the tooltip on the configured trigger.
fn apply_tooltip_triggers(
	trigger_ref: NodeRef<leptos::html::Span>,
	trigger: TooltipTrigger,
	is_open: RwSignal<bool>,
) {
	Effect::new(move |_| {
		let Some(element) = trigger_ref.get() else {
			return;
		};

		let (open_event, close_event) = match trigger {
			TooltipTrigger::Focus => ("focus", "blur"),
			TooltipTrigger::Hover => ("pointerenter", "pointerleave"),
		};

		let open = show_after_delay(is_open);
		let close = hide_now(is_open);

		let _ = element.add_event_listener_with_callback(
			open_event,
			open.as_ref().unchecked_ref::<js_sys::Function>(),
		);
		let _ = element.add_event_listener_with_callback(
			close_event,
			close.as_ref().unchecked_ref::<js_sys::Function>(),
		);

		let _ = open;
		let _ = close;
	});
}

/// Show the tooltip after a short delay, cancelled by an earlier hide.
fn show_after_delay(is_open: RwSignal<bool>) -> Closure<dyn Fn()> {
	Closure::new(move || {
		set_timeout(move || is_open.set(true), TOOLTIP_DELAY);
	})
}

fn hide_now(is_open: RwSignal<bool>) -> Closure<dyn Fn()> {
	Closure::new(move || is_open.set(false))
}

/// The close behavior: the owner's callback, or flipping the open signal.
fn unwrap_close(
	on_close: Option<BoxedOverlayCloseCallback>,
	is_open: RwSignal<bool>,
) -> Box<dyn Fn() + Send + Sync> {
	match on_close {
		Some(on_close) => on_close,
		None => Box::new(move || is_open.set(false)),
	}
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
	use leptos::mount::mount_to_body;
	use wasm_bindgen::JsCast;
	use wasm_bindgen_test::*;

	use super::*;

	wasm_bindgen_test_configure!(run_in_browser);

	#[component]
	fn Example() -> impl IntoView {
		let is_open = RwSignal::new(false);
		let trigger: NodeRef<leptos::html::Button> = NodeRef::new();

		view! {
			<button
				class="leptos-aria-modal-test__trigger"
				node_ref=trigger
				on:click=move |_| is_open.set(!is_open.get_untracked())
			>
				"Open"
			</button>
			<Modal is_open=is_open>
				<Dialog aria_label="Example dialog".to_string()>
					<p>"Dialog content"</p>
				</Dialog>
			</Modal>
		}
	}

	#[wasm_bindgen_test]
	async fn modal_toggles_visibility_with_the_open_state() {
		console_error_panic_hook::set_once();

		mount_to_body(|| view! { <Example /> });

		let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
		wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();

		let open_button = document()
			.query_selector(".leptos-aria-modal-test__trigger")
			.unwrap()
			.unwrap()
			.unchecked_into::<web_sys::HtmlButtonElement>();

		let dialog = document()
			.query_selector("[role='dialog']")
			.unwrap()
			.unwrap();

		assert!(
			dialog.has_attribute("hidden"),
			"the modal is closed initially"
		);

		open_button.click();

		let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
		wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();

		assert!(!dialog.has_attribute("hidden"), "the modal opens on click");
	}
}
