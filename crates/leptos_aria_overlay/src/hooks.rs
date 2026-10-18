use std::cell::Cell;
use std::sync::Arc;
use std::sync::Mutex;

use leptos::prelude::*;
use leptos_aria_utils::FocusableElement;
use leptos_aria_utils::ToFocusableElement;
use leptos_aria_utils::focus_safely;
use leptos_aria_utils::is_focusable;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::Element;
use web_sys::HtmlElement;

thread_local! {
	static SCROLL_LOCK_COUNT: Cell<u32> = const { Cell::new(0) };
	static SAVED_SCROLL_STYLES: Cell<Option<(String, String)>> = const { Cell::new(None) };
}

pub(crate) fn document() -> web_sys::Document {
	web_sys::window()
		.expect("the `window` global is available in the browser")
		.document()
		.expect("the `document` global is available in the browser")
}

fn body() -> HtmlElement {
	document().body().expect("the document body")
}

/// A callback invoked when an overlay should close.
pub type BoxedOverlayCloseCallback = Box<dyn Fn() + Send + Sync + 'static>;

/// Lock page scrolling while an overlay is open.
///
/// The body gets `overflow: hidden` with scrollbar-width compensation so the
/// page does not shift. Nested overlays stack their locks.
pub fn use_prevent_scroll(is_open: Signal<bool>) {
	let open = Cell::new(false);

	Effect::new(move |_| {
		let is_open = is_open.get();

		if is_open == open.get() {
			return;
		}

		open.set(is_open);

		if is_open {
			lock_scroll();
		} else {
			unlock_scroll();
		}
	});
}

fn lock_scroll() {
	if SCROLL_LOCK_COUNT.get() == 0 {
		let style = body().style();
		let saved = SAVED_SCROLL_STYLES.take().unwrap_or_else(|| {
			(
				style.get_property_value("overflow").unwrap_or_default(),
				style
					.get_property_value("padding-right")
					.unwrap_or_default(),
			)
		});

		// Compensate for the scrollbar the page loses while locked.
		let scrollbar_width = window_inner_width() - document_client_width();
		let padding = if scrollbar_width > 0.0 {
			format!("calc({} + {scrollbar_width}px)", saved.1)
		} else {
			saved.1.clone()
		};

		style.set_property("overflow", "hidden").ok();
		style.set_property("padding-right", &padding).ok();

		SAVED_SCROLL_STYLES.set(Some(saved));
	}

	SCROLL_LOCK_COUNT.set(SCROLL_LOCK_COUNT.get() + 1);
}

fn unlock_scroll() {
	let count = SCROLL_LOCK_COUNT.get().saturating_sub(1);
	SCROLL_LOCK_COUNT.set(count);

	if count > 0 {
		return;
	}

	if let Some((overflow, padding)) = SAVED_SCROLL_STYLES.take() {
		let style = body().style();

		style.set_property("overflow", &overflow).ok();
		style.set_property("padding-right", &padding).ok();
	}
}

fn window_inner_width() -> f64 {
	web_sys::window()
		.and_then(|window| window.inner_width().ok())
		.and_then(|width| width.as_f64())
		.unwrap_or(0.0)
}

fn document_client_width() -> f64 {
	document()
		.document_element()
		.map_or(0.0, |element| f64::from(element.client_width()))
}

/// The props accepted by [`use_modal`].
pub struct UseModalProps {
	/// Whether the overlay is open.
	pub is_open: Signal<bool>,
}

/// Contain focus inside an overlay while it is open.
///
/// Tabbing past the last focusable element wraps to the first, shift-tabbing
/// past the first wraps to the last, focus escaping to the document is pulled
/// back, and the previously focused element is restored on close.
pub fn use_modal(overlay: NodeRef<leptos::html::Div>, props: UseModalProps) {
	let restore_target = Arc::new(Mutex::new(None::<FocusableElement>));

	Effect::new(move |_| {
		if !props.is_open.get() {
			return;
		}

		let Some(element) = overlay.get().map(|element| web_sys_ref(&element)) else {
			return;
		};

		*restore_target.lock().unwrap() = document()
			.active_element()
			.map(|active| active.to_focusable_element());

		let focus_target =
			first_focusable(&element).unwrap_or_else(|| FocusableElement::Html(element.clone()));

		focus_safely(focus_target.as_element());

		let guard = send_wrapper::SendWrapper::new(install_focus_guard(&element));
		let restore = Arc::clone(&restore_target);

		on_cleanup(move || {
			drop(guard);

			if let Some(target) = restore.lock().unwrap().take() {
				focus_safely(target.as_element());
			}
		});
	});
}

/// Listeners keeping focus inside the overlay until dropped.
struct FocusGuard {
	key_handler: Closure<dyn FnMut(web_sys::KeyboardEvent)>,
	focusin_handler: Closure<dyn FnMut(web_sys::Event)>,
}

impl Drop for FocusGuard {
	fn drop(&mut self) {
		document()
			.remove_event_listener_with_callback(
				"keydown",
				self.key_handler
					.as_ref()
					.unchecked_ref::<js_sys::Function>(),
			)
			.ok();

		document()
			.remove_event_listener_with_callback(
				"focusin",
				self.focusin_handler
					.as_ref()
					.unchecked_ref::<js_sys::Function>(),
			)
			.ok();
	}
}

fn install_focus_guard(overlay: &HtmlElement) -> FocusGuard {
	FocusGuard {
		key_handler: guard_key_handler(overlay.clone()),
		focusin_handler: guard_focusin_handler(overlay.clone()),
	}
}

/// The element behind a typed node ref, as a plain `HtmlElement`.
fn web_sys_ref(element: &impl JsCast) -> HtmlElement {
	element.unchecked_ref::<HtmlElement>().clone()
}

/// The first focusable descendant, or the overlay itself.
fn first_focusable(element: &HtmlElement) -> Option<FocusableElement> {
	focusable_descendants(element).into_iter().next()
}

/// Every focusable descendant in document order.
fn focusable_descendants(element: &HtmlElement) -> Vec<FocusableElement> {
	let Some(nodes) = element.query_selector_all("*").ok() else {
		return Vec::new();
	};

	(0..nodes.length())
		.filter_map(|index| nodes.item(index))
		.filter_map(|node| node.dyn_into::<Element>().ok())
		.filter(|descendant| is_focusable(descendant, true))
		.map(|descendant| descendant.to_focusable_element())
		.collect()
}

#[allow(clippy::needless_pass_by_value)]
fn guard_key_handler(overlay: HtmlElement) -> Closure<dyn FnMut(web_sys::KeyboardEvent)> {
	let handler = Closure::new(move |event: web_sys::KeyboardEvent| {
		if event.key() != "Tab" {
			return;
		}

		let focusables = focusable_descendants(&overlay);

		let Some(first) = focusables.first() else {
			return;
		};

		let Some(last) = focusables.last() else {
			return;
		};

		let active = document().active_element();

		if event.shift_key() && active_ref_matches(active.as_ref(), first) {
			event.prevent_default();
			focus_safely(last.as_element());
		} else if !event.shift_key() && active_ref_matches(active.as_ref(), last) {
			event.prevent_default();
			focus_safely(first.as_element());
		}
	});

	document()
		.add_event_listener_with_callback(
			"keydown",
			handler.as_ref().unchecked_ref::<js_sys::Function>(),
		)
		.ok();

	handler
}

fn active_ref_matches(active: Option<&Element>, target: &FocusableElement) -> bool {
	active.is_some_and(|active| active == target.as_element())
}

fn guard_focusin_handler(overlay: HtmlElement) -> Closure<dyn FnMut(web_sys::Event)> {
	let handler = Closure::new(move |event: web_sys::Event| {
		let Some(target) = event.target() else {
			return;
		};

		let inside_overlay = overlay.contains(target.dyn_ref::<web_sys::Node>());

		if !inside_overlay && !target_is_root(&target) {
			let overlay_element: &Element = overlay.as_ref();

			focus_safely(overlay_element);
		}
	});

	document()
		.add_event_listener_with_callback(
			"focusin",
			handler.as_ref().unchecked_ref::<js_sys::Function>(),
		)
		.ok();

	handler
}

fn target_is_root(target: &web_sys::EventTarget) -> bool {
	let document = document();

	if target == document.as_ref() {
		return true;
	}

	web_sys::window().is_some_and(|window| {
		let window_target: &web_sys::EventTarget = window.as_ref();

		window_target == target
	})
}

/// Listeners dismissing the overlay until dropped.
struct DismissGuard {
	pointer_handler: Closure<dyn FnMut(web_sys::Event)>,
	key_handler: Closure<dyn FnMut(web_sys::KeyboardEvent)>,
}

impl Drop for DismissGuard {
	fn drop(&mut self) {
		document()
			.remove_event_listener_with_callback(
				"pointerdown",
				self.pointer_handler
					.as_ref()
					.unchecked_ref::<js_sys::Function>(),
			)
			.ok();

		document()
			.remove_event_listener_with_callback(
				"keydown",
				self.key_handler
					.as_ref()
					.unchecked_ref::<js_sys::Function>(),
			)
			.ok();
	}
}

/// Dismiss an overlay on outside pointer interaction or the escape key.
pub fn use_overlay_dismiss(
	overlay: NodeRef<leptos::html::Div>,
	is_open: Signal<bool>,
	on_close: Box<dyn Fn() + Send + Sync + 'static>,
) {
	let on_close: Arc<dyn Fn() + Send + Sync> = Arc::from(on_close);

	Effect::new(move |_| {
		if !is_open.get() {
			return;
		}

		let Some(element) = overlay.get().map(|element| web_sys_ref(&element)) else {
			return;
		};

		let guard =
			send_wrapper::SendWrapper::new(install_dismiss_guard(&element, Arc::clone(&on_close)));

		on_cleanup(move || drop(guard));
	});
}

fn install_dismiss_guard(
	overlay: &HtmlElement,
	on_close: Arc<dyn Fn() + Send + Sync>,
) -> DismissGuard {
	DismissGuard {
		pointer_handler: outside_pointer_handler(overlay.clone(), Arc::clone(&on_close)),
		key_handler: escape_handler(on_close),
	}
}

fn outside_pointer_handler(
	overlay: HtmlElement,
	on_close: Arc<dyn Fn() + Send + Sync>,
) -> Closure<dyn FnMut(web_sys::Event)> {
	let handler = Closure::new(move |event: web_sys::Event| {
		let Some(target) = event.target() else {
			return;
		};

		if !overlay.contains(target.dyn_ref::<web_sys::Node>()) {
			on_close();
		}
	});

	document()
		.add_event_listener_with_callback(
			"pointerdown",
			handler.as_ref().unchecked_ref::<js_sys::Function>(),
		)
		.ok();

	handler
}

fn escape_handler(
	on_close: Arc<dyn Fn() + Send + Sync>,
) -> Closure<dyn FnMut(web_sys::KeyboardEvent)> {
	let handler = Closure::new(move |event: web_sys::KeyboardEvent| {
		if event.key() == "Escape" {
			on_close();
		}
	});

	document()
		.add_event_listener_with_callback(
			"keydown",
			handler.as_ref().unchecked_ref::<js_sys::Function>(),
		)
		.ok();

	handler
}
