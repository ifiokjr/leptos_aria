use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;
use leptos_aria_utils::FocusableElement;
use leptos_aria_utils::GlobalListeners;
use leptos_aria_utils::ToFocusableElement;
use leptos_aria_utils::focus_without_scrolling;
use leptos_aria_utils::is_virtual_click;
use leptos_aria_utils::is_virtual_pointer_event;
use typed_builder::TypedBuilder;
use wasm_bindgen::JsCast;
use web_sys::DomRect;
use web_sys::DragEvent;
use web_sys::Element;
use web_sys::HtmlAnchorElement;
use web_sys::HtmlButtonElement;
use web_sys::HtmlElement;
use web_sys::HtmlInputElement;
use web_sys::HtmlTextAreaElement;
use web_sys::KeyboardEvent;
use web_sys::MouseEvent;
use web_sys::Node;
use web_sys::PointerEvent;
use web_sys::WheelEvent;

use crate::text_selection::disable_text_selection;
use crate::text_selection::restore_text_selection;

fn document() -> web_sys::Document {
	web_sys::window()
		.expect("the `window` global is available in the browser")
		.document()
		.expect("the `document` global is available in the browser")
}

/// Handles press interactions across mouse, touch, keyboard, and screen
/// readers.
///
/// A press interaction starts when a user presses down with a mouse or their
/// finger on the target, and ends when they move the pointer off the target.
/// It may start again if the pointer re-enters the target. `use_press`
/// returns the current press state, which can be used to adjust the visual
/// appearance of the target. If the pointer is released over the target, then
/// an `on_press` event is fired.
///
/// * Handles mouse and touch events
/// * Handles <kbd>Enter</kbd> or <kbd>Space</kbd> key presses
/// * Handles screen reader virtual clicks
/// * Uses pointer events where available, with fallbacks to mouse and touch
///   events
/// * Normalizes focus behavior on mouse and touch interactions across browsers
/// * Handles disabling text selection on mobile while the press interaction is
///   active
/// * Handles canceling press interactions on scroll
/// * Normalizes many cross browser inconsistencies
///
/// Read the [`react-aria` blog
/// post](https://react-spectrum.adobe.com/blog/building-a-button-part-1.html)
/// about the complexities of press event handling to learn more.
///
/// ## Usage
///
/// `use_press` returns props that should be added the target component (spread
/// is not yet supported in `leptos`):
///
/// ```
/// // let press = use_press(UsePressProps::builder().build());
/// ```
pub fn use_press(props: UsePressProps) -> PressResult {
	// Internal state.
	let listeners = Rc::new(RefCell::new(GlobalListeners::default()));
	let ignore_emulated_mouse_events = RwSignal::new(false);
	let ignore_click_after_press = RwSignal::new(false);
	let did_fire_press_start = RwSignal::new(false);
	let active_pointer_id: RwSignal<Option<i32>> = RwSignal::new(None);
	let target: RwSignal<Option<Element>> = RwSignal::new(None);
	let is_over_target = RwSignal::new(false);
	let pointer_type = RwSignal::new(PointerType::default());

	let is_disabled = props.is_disabled.unwrap_or_else(|| Signal::from(false));
	let prevent_focus_on_press = props
		.prevent_focus_on_press
		.unwrap_or_else(|| Signal::from(false));
	let should_cancel_on_pointer_exit = props
		.should_cancel_on_pointer_exit
		.unwrap_or_else(|| Signal::from(false));
	let allow_text_selection_on_press = props
		.allow_text_selection_on_press
		.unwrap_or_else(|| Signal::from(false));

	let on_press = props.on_press.map(wrap_press_event_callback);
	let on_press_start = props.on_press_start.map(wrap_press_event_callback);
	let on_press_end = props.on_press_end.map(wrap_press_event_callback);
	let on_press_change = props.on_press_change.map(wrap_change_callback);
	let on_press_up = props.on_press_up.map(wrap_press_event_callback);

	let local_is_pressed = RwSignal::new(false);
	let is_pressed = Signal::derive({
		let controlled = props.is_pressed.unwrap_or_else(|| Signal::from(false));

		move || controlled.get() || local_is_pressed.get()
	});

	// Trigger the beginning of a custom press event.
	let trigger_press_start = {
		let on_press_start = on_press_start.clone();
		let on_press_change = on_press_change.clone();

		move |focusable_event: &FocusableEvent, pointer: PointerType| {
			if is_disabled.get_untracked() || did_fire_press_start.get_untracked() {
				return;
			}

			let event = PressEvent::create(&pointer, PressEventType::PressStart, focusable_event);
			call_event(on_press_start.as_ref(), &event);
			call_change_event(on_press_change.as_ref(), true);

			did_fire_press_start.set(true);
			local_is_pressed.set(true);
		}
	};

	let trigger_press_end = {
		let on_press_end = on_press_end.clone();
		let on_press_change = on_press_change.clone();
		let on_press = on_press.clone();

		let callback =
			move |focusable_event: &FocusableEvent, pointer: PointerType, was_pressed: bool| {
				if !did_fire_press_start.get_untracked() {
					return;
				}

				ignore_click_after_press.set(true);
				did_fire_press_start.set(false);

				let event = PressEvent::create(&pointer, PressEventType::PressEnd, focusable_event);
				call_event(on_press_end.as_ref(), &event);
				call_change_event(on_press_change.as_ref(), false);

				local_is_pressed.set(false);

				if !was_pressed || is_disabled.get_untracked() {
					return;
				}

				let event = PressEvent::create(&pointer, PressEventType::Press, focusable_event);
				call_event(on_press.as_ref(), &event);
			};

		Rc::new(callback)
	};

	let trigger_press_up = {
		let on_press_up = on_press_up.clone();

		let callback = move |focusable_event: &FocusableEvent, pointer: PointerType| {
			if is_disabled.get_untracked() {
				return;
			}

			let event = PressEvent::create(&pointer, PressEventType::PressUp, focusable_event);
			call_event(on_press_up.as_ref(), &event);
		};

		Rc::new(callback)
	};

	let cancel = {
		let trigger_press_end = trigger_press_end.clone();
		let listeners = listeners.clone();

		let callback = move |focusable_event: &FocusableEvent| {
			if !local_is_pressed.get_untracked() {
				return;
			}

			if is_over_target.get_untracked() {
				trigger_press_end(focusable_event, pointer_type.get_untracked(), false);
			}

			local_is_pressed.set(false);
			is_over_target.set(false);
			active_pointer_id.set(None);
			pointer_type.set(PointerType::default());

			listeners.borrow_mut().remove_all_listeners();

			if !allow_text_selection_on_press.get_untracked() {
				if let Some(element) = target.get_untracked().as_ref() {
					restore_text_selection(element);
				}
			}
		};

		Rc::new(callback)
	};

	let on_key_up: PressCallback<KeyboardEvent> = {
		let trigger_press_up = trigger_press_up.clone();

		let handler = move |event: KeyboardEvent| {
			let Some(event_current_target) = current_target_element(&event) else {
				return;
			};

			let event_target = event_target_node(&event);

			if !is_valid_keyboard_event(&event, &event_current_target)
				|| event.repeat()
				|| !event_current_target.contains(event_target.as_ref())
			{
				return;
			}

			let focusable_event = FocusableEvent::Keyboard(
				event,
				target
					.get_untracked()
					.map(|target| target.to_focusable_element()),
			);

			trigger_press_up(&focusable_event, PointerType::Keyboard);
		};

		Rc::new(handler)
	};

	let global_on_key_up: PressCallback<KeyboardEvent> = {
		let trigger_press_end = trigger_press_end.clone();
		let listeners = listeners.clone();

		let handler = move |event: KeyboardEvent| {
			let event_target = event_target_node(&event);

			if !local_is_pressed.get_untracked() {
				return;
			}

			let Some(element) = target.get_untracked() else {
				return;
			};

			if !is_valid_keyboard_event(&event, &element) {
				return;
			}

			if should_prevent_default_keyboard(
				event_target_element(&event).as_ref().unwrap_or(&element),
				&event.key(),
			) {
				event.prevent_default();
			}

			event.stop_propagation();
			local_is_pressed.set(false);

			let focusable_event =
				FocusableEvent::Keyboard(event, Some(element.to_focusable_element()));

			let contains_target = element.contains(event_target.as_ref());

			trigger_press_end(&focusable_event, PointerType::Keyboard, contains_target);
			listeners.borrow_mut().remove_all_listeners();

			// If the target is a link, trigger the click method to open the
			// URL, but defer triggering press end until the click event
			// handler.
			if element.is_instance_of::<HtmlElement>()
				&& element.contains(event_target.as_ref())
				&& has_link_role(&element)
			{
				element.unchecked_ref::<HtmlElement>().click();
			}
		};

		Rc::new(handler)
	};

	let on_key_down: PressCallback<KeyboardEvent> = {
		let global_on_key_up = global_on_key_up.clone();
		let trigger_press_start = trigger_press_start.clone();
		let listeners = listeners.clone();

		let handler = move |event: KeyboardEvent| {
			let Some(event_current_target) = current_target_element(&event) else {
				return;
			};

			let event_target = event_target_node(&event);

			if is_valid_keyboard_event(&event, &event_current_target)
				&& event_current_target.contains(event_target.as_ref())
			{
				if should_prevent_default_keyboard(
					event_target_element(&event)
						.as_ref()
						.unwrap_or(&event_current_target),
					&event.key(),
				) {
					event.prevent_default();
				}

				event.stop_propagation();

				// If the event is repeating, it may have started on a different
				// element after which focus moved to the current element. Ignore
				// these events and only handle the first key down event.
				if !local_is_pressed.get_untracked() && !event.repeat() {
					target.set(Some(event_current_target.clone()));
					local_is_pressed.set(true);

					let focusable_event = FocusableEvent::Keyboard(event, None);
					trigger_press_start(&focusable_event, PointerType::Keyboard);

					let key_up = global_on_key_up.clone();

					// Focus may move before the key up event, so register the
					// event on the document instead of the same element where
					// the key down event occurred.
					listeners.borrow_mut().add::<KeyboardEvent, _>(
						&document(),
						"keyup",
						move |event| key_up(event),
						false,
					);
				}
			} else if event.key() == "Enter" && is_html_anchor_link(&event_current_target) {
				// If the target is a link, we won't have handled this above
				// because we want the default browser behavior to open the link
				// when pressing Enter. But we still need to prevent default so
				// that elements above do not also handle it (e.g. table row).
				event.stop_propagation();
			}
		};

		Rc::new(handler)
	};

	let on_click: PressCallback<MouseEvent> = {
		let trigger_press_start = trigger_press_start.clone();
		let trigger_press_up = trigger_press_up.clone();
		let trigger_press_end = trigger_press_end.clone();

		let callback = move |event: MouseEvent| {
			let Some(event_current_target) = current_target_element(&event) else {
				return;
			};

			let event_target = event_target_node(&event);

			if !event_current_target.contains(event_target.as_ref()) {
				return;
			}

			// Ensure it was the main mouse button that was clicked.
			if event.button() != 0 {
				return;
			}

			event.stop_propagation();

			if is_disabled.get_untracked() {
				event.prevent_default();
			}

			// If triggered from a screen reader or by using element.click(),
			// trigger as if it were a keyboard click.
			if !ignore_click_after_press.get_untracked()
				&& !ignore_emulated_mouse_events.get_untracked()
				&& (pointer_type.get_untracked() == PointerType::Virtual
					|| is_virtual_click(&event))
			{
				// Ensure the element receives focus (VoiceOver on iOS does not
				// do this).
				if !is_disabled.get_untracked() && !prevent_focus_on_press.get_untracked() {
					focus_without_scrolling(&event_current_target);
				}

				let focusable_event = FocusableEvent::Mouse(event, None);
				trigger_press_start(&focusable_event, PointerType::Virtual);
				trigger_press_up(&focusable_event, PointerType::Virtual);
				trigger_press_end(&focusable_event, PointerType::Virtual, true);
			}

			ignore_emulated_mouse_events.set(false);
			ignore_click_after_press.set(false);
		};

		Rc::new(callback)
	};

	let on_drag_start: PressCallback<DragEvent> = {
		let cancel = cancel.clone();

		let handler = move |event: DragEvent| {
			let Some(event_current_target) = current_target_element(&event) else {
				return;
			};

			if !event_current_target.contains(event_target_node(&event).as_ref()) {
				return;
			}

			// Safari does not call onPointerCancel when a drag starts, whereas
			// Chrome and Firefox do.
			let focusable_event = FocusableEvent::Drag(event, None);
			cancel(&focusable_event);
		};

		Rc::new(handler)
	};

	let on_mouse_down: PressCallback<MouseEvent> = {
		let handler = move |event: MouseEvent| {
			let Some(event_current_target) = current_target_element(&event) else {
				return;
			};

			if !event_current_target.contains(event_target_node(&event).as_ref()) {
				return;
			}

			if event.button() == 0 {
				// Chrome and Firefox on touch Windows devices require mouse
				// down events to be canceled in addition to pointer events, or
				// an extra asynchronous focus event will be fired.
				if should_prevent_default(&event_current_target) {
					event.prevent_default();
				}

				event.stop_propagation();
			}
		};

		Rc::new(handler)
	};

	let on_pointer_cancel: PressCallback<PointerEvent> = {
		let cancel = cancel.clone();

		let handler = move |event: PointerEvent| {
			let focusable_event = FocusableEvent::Pointer(event, None);
			cancel(&focusable_event);
		};

		Rc::new(handler)
	};

	// Safari on iOS < 13.2 does not implement pointerenter/pointerleave events
	// correctly. Use pointer move events instead to implement our own hit
	// testing. See <https://bugs.webkit.org/show_bug.cgi?id=199803>.
	let on_pointer_move: PressCallback<PointerEvent> = {
		let trigger_press_start = trigger_press_start.clone();
		let trigger_press_end = trigger_press_end.clone();
		let cancel = cancel.clone();

		let handler = move |event: PointerEvent| {
			if Some(event.pointer_id()) != active_pointer_id.get_untracked() {
				return;
			}

			let Some(element) = target.get_untracked() else {
				return;
			};

			let focusable_event =
				FocusableEvent::Pointer(event.clone(), Some(element.to_focusable_element()));

			if is_above_target(&event, &element) && !is_over_target.get_untracked() {
				is_over_target.set(true);
				trigger_press_start(&focusable_event, pointer_type.get_untracked());
			} else if is_over_target.get_untracked() {
				is_over_target.set(false);
				trigger_press_end(&focusable_event, pointer_type.get_untracked(), false);

				if should_cancel_on_pointer_exit.get_untracked() {
					cancel(&focusable_event);
				}
			}
		};

		Rc::new(handler)
	};

	let on_pointer_up: PressCallback<PointerEvent> = {
		let trigger_press_up = trigger_press_up.clone();

		let handler = move |event: PointerEvent| {
			let Some(event_current_target) = current_target_element(&event) else {
				return;
			};

			// iOS fires pointerup with zero width and height, so check the
			// pointer type recorded during pointerdown.
			if !event_current_target.contains(event_target_node(&event).as_ref())
				|| pointer_type.get_untracked() == PointerType::Virtual
			{
				return;
			}

			// Only handle left clicks. Safari on iOS sometimes fires pointerup
			// events, even when the touch isn't over the target, so double
			// check.
			if event.button() == 0 && is_above_target(&event, &event_current_target) {
				let focusable_event = FocusableEvent::Pointer(event, None);
				trigger_press_up(&focusable_event, pointer_type.get_untracked());
			}
		};

		Rc::new(handler)
	};

	let global_on_pointer_up: PressCallback<PointerEvent> = {
		let trigger_press_end = trigger_press_end.clone();
		let listeners = listeners.clone();

		let handler = move |event: PointerEvent| {
			if Some(event.pointer_id()) != active_pointer_id.get_untracked()
				|| !local_is_pressed.get_untracked()
				|| event.button() != 0
			{
				return;
			}

			let Some(element) = target.get_untracked() else {
				return;
			};

			let focusable_event =
				FocusableEvent::Pointer(event.clone(), Some(element.to_focusable_element()));

			if is_above_target(&event, &element) {
				trigger_press_end(&focusable_event, pointer_type.get_untracked(), true);
			} else if is_over_target.get_untracked() {
				trigger_press_end(&focusable_event, pointer_type.get_untracked(), false);
			}

			local_is_pressed.set(false);
			is_over_target.set(false);
			active_pointer_id.set(None);
			pointer_type.set(PointerType::default());
			listeners.borrow_mut().remove_all_listeners();

			if !allow_text_selection_on_press.get_untracked() {
				restore_text_selection(&element);
			}
		};

		Rc::new(handler)
	};

	let on_pointer_down: PressCallback<PointerEvent> = {
		let trigger_press_start = trigger_press_start.clone();
		let on_pointer_move = on_pointer_move.clone();
		let on_pointer_cancel = on_pointer_cancel.clone();
		let listeners = listeners.clone();

		let handler = move |event: PointerEvent| {
			let Some(event_current_target) = current_target_element(&event) else {
				return;
			};

			// Only handle left clicks, and ignore events that bubbled through
			// portals.
			if event.button() != 0
				|| !event_current_target.contains(event_target_node(&event).as_ref())
			{
				return;
			}

			// iOS safari fires pointer events from VoiceOver with incorrect
			// coordinates/target. Ignore and let the onClick handler take care
			// of it instead.
			// <https://bugs.webkit.org/show_bug.cgi?id=222627>
			// <https://bugs.webkit.org/show_bug.cgi?id=223202>
			if is_virtual_pointer_event(&event) {
				pointer_type.set(PointerType::Virtual);
				return;
			}

			// Due to browser inconsistencies, especially on mobile browsers, we
			// prevent default on pointer down and handle focusing the pressable
			// element ourselves.
			if should_prevent_default(&event_current_target) {
				event.prevent_default();
			}

			pointer_type.set(event.pointer_type().into());
			event.stop_propagation();

			if local_is_pressed.get_untracked() {
				return;
			}

			local_is_pressed.set(true);
			is_over_target.set(true);
			active_pointer_id.set(Some(event.pointer_id()));
			target.set(Some(event_current_target.clone()));

			if !is_disabled.get_untracked() && !prevent_focus_on_press.get_untracked() {
				focus_without_scrolling(&event_current_target);
			}

			if !allow_text_selection_on_press.get_untracked() {
				disable_text_selection(target.get_untracked().as_ref());
			}

			let focusable_event = FocusableEvent::Pointer(event, None);
			trigger_press_start(&focusable_event, pointer_type.get_untracked());

			let pointer_move = on_pointer_move.clone();
			let pointer_up = global_on_pointer_up.clone();
			let pointer_cancel = on_pointer_cancel.clone();

			let mut global_listeners = listeners.borrow_mut();
			global_listeners.add::<PointerEvent, _>(
				&document(),
				"pointermove",
				move |event| pointer_move(event),
				false,
			);
			global_listeners.add::<PointerEvent, _>(
				&document(),
				"pointerup",
				move |event| pointer_up(event),
				false,
			);
			global_listeners.add::<PointerEvent, _>(
				&document(),
				"pointercancel",
				move |event| pointer_cancel(event),
				false,
			);
		};

		Rc::new(handler)
	};

	PressResult {
		is_pressed,
		is_disabled,
		on_click,
		on_drag_start,
		on_key_down,
		on_key_up,
		on_mouse_down,
		on_pointer_down,
		on_pointer_up,
	}
}

/// A callback invoked for a [`PressEvent`].
pub type BoxedPressEventCallback = Box<dyn Fn(&PressEvent)>;

/// A callback invoked when the press state changes.
pub type BoxedPressChangeCallback = Box<dyn Fn(bool)>;

type PressEventCallback = Rc<dyn Fn(&PressEvent)>;
type PressChangeCallback = Rc<dyn Fn(bool)>;
type PressCallback<E> = Rc<dyn Fn(E)>;

fn wrap_press_event_callback(callback: BoxedPressEventCallback) -> PressEventCallback {
	Rc::new(move |event: &PressEvent| callback(event))
}

fn wrap_change_callback(callback: BoxedPressChangeCallback) -> PressChangeCallback {
	Rc::new(move |pressed: bool| callback(pressed))
}

fn call_event(callback: Option<&PressEventCallback>, event: &PressEvent) {
	if let Some(callback) = callback {
		callback(event);
	}
}

fn call_change_event(callback: Option<&PressChangeCallback>, pressed: bool) {
	if let Some(callback) = callback {
		callback(pressed);
	}
}

fn current_target_element(event: &impl AsRef<web_sys::Event>) -> Option<Element> {
	event
		.as_ref()
		.current_target()
		.and_then(|target| target.dyn_into::<Element>().ok())
}

fn event_target_node(event: &impl AsRef<web_sys::Event>) -> Option<Node> {
	event
		.as_ref()
		.target()
		.and_then(|target| target.dyn_into::<Node>().ok())
}

fn event_target_element(event: &impl AsRef<web_sys::Event>) -> Option<Element> {
	event
		.as_ref()
		.target()
		.and_then(|target| target.dyn_into::<Element>().ok())
}

/// The props returned by [`use_press`] which should be spread onto the target
/// element.
#[derive(Clone)]
pub struct PressResult {
	/// Whether the target is currently pressed.
	pub is_pressed: Signal<bool>,

	/// Whether the press events are disabled.
	pub is_disabled: Signal<bool>,

	pub on_click: PressCallback<MouseEvent>,
	pub on_drag_start: PressCallback<DragEvent>,
	pub on_key_down: PressCallback<KeyboardEvent>,
	pub on_key_up: PressCallback<KeyboardEvent>,
	pub on_mouse_down: PressCallback<MouseEvent>,
	pub on_pointer_down: PressCallback<PointerEvent>,
	pub on_pointer_up: PressCallback<PointerEvent>,
}

fn are_rectangles_overlapping(dom_rect: &DomRect, rects: &[Rect]) -> bool {
	rects.iter().any(|rect| {
		// Check if they cannot overlap on the x axis.
		if dom_rect.left() > rect.right || dom_rect.right() < rect.left {
			return false;
		}

		// Check if they cannot overlap on the y axis.
		dom_rect.top() <= rect.bottom && dom_rect.bottom() >= rect.top
	})
}

fn is_above_target(point: &impl GetRects, target: &Element) -> bool {
	let rect = target.get_bounding_client_rect();
	let point_rects = point.get_rects();

	are_rectangles_overlapping(&rect, &point_rects)
}

/// We cannot prevent default if the target is not an `HTMLElement` or if it is
/// a draggable `HTMLElement`.
fn should_prevent_default(target: impl AsRef<Element>) -> bool {
	let element = target.as_ref();

	!element.is_instance_of::<HtmlElement>() || !element.unchecked_ref::<HtmlElement>().draggable()
}

fn should_prevent_default_keyboard(target: impl AsRef<Element>, key: &str) -> bool {
	let element = target.as_ref();

	if element.is_instance_of::<HtmlInputElement>() {
		!is_valid_input_key(element.unchecked_ref(), key)
	} else if element.is_instance_of::<HtmlButtonElement>() {
		element.unchecked_ref::<HtmlButtonElement>().type_() == "submit"
	} else {
		true
	}
}

fn is_valid_keyboard_event(event: &KeyboardEvent, current_target: &Element) -> bool {
	let key = event.key();
	let code = event.code();
	let element: &HtmlElement = current_target.unchecked_ref();
	let role = element.get_attribute("role");

	// Accessibility for keyboards. Space and Enter only.
	(key == "Enter" || key == " " || code == "Space")
		&& !((element.is_instance_of::<HtmlInputElement>()
			&& !is_valid_input_key(element.unchecked_ref(), &key))
			|| element.is_instance_of::<HtmlTextAreaElement>()
			|| element.is_content_editable())
		// A link with a valid href should be handled natively, unless it also
		// has `role="button"` and was triggered using Space.
		&& (!is_html_anchor_link(element)
			|| (role.as_deref() == Some("button") && key != "Enter"))
		&& !(role.as_deref() == Some("link") && key != "Enter")
}

fn is_html_anchor_link(target: impl AsRef<Element>) -> bool {
	let element = target.as_ref();

	element.is_instance_of::<HtmlAnchorElement>()
		|| (element.tag_name() == "A" && element.has_attribute("href"))
}

fn has_link_role(target: impl AsRef<Element>) -> bool {
	let element = target.as_ref();

	is_html_anchor_link(element) || element.get_attribute("role").as_deref() == Some("link")
}

fn is_valid_input_key(target: &HtmlInputElement, key: &str) -> bool {
	// Only space should toggle checkboxes and radios, not enter.
	if target.type_() == "checkbox" || target.type_() == "radio" {
		key == " "
	} else {
		NON_TEXT_INPUT_TYPES.contains(&target.type_().as_str())
	}
}

const NON_TEXT_INPUT_TYPES: &[&str; 9] = &[
	"checkbox", "radio", "range", "color", "file", "image", "button", "submit", "reset",
];

#[derive(Debug, Clone, Copy, Default)]
pub struct Rect {
	pub top: f64,
	pub right: f64,
	pub bottom: f64,
	pub left: f64,
}

pub trait GetRects {
	fn get_rects(&self) -> Vec<Rect>;
}

impl GetRects for MouseEvent {
	fn get_rects(&self) -> Vec<Rect> {
		vec![point_rect(self.client_x(), self.client_y())]
	}
}

impl GetRects for PointerEvent {
	fn get_rects(&self) -> Vec<Rect> {
		let offset_x = self.width();
		let offset_y = self.height();

		vec![Rect {
			top: (self.client_y() - offset_y).into(),
			right: (self.client_x() + offset_x).into(),
			bottom: (self.client_y() + offset_y).into(),
			left: (self.client_x() - offset_x).into(),
		}]
	}
}

impl GetRects for DragEvent {
	fn get_rects(&self) -> Vec<Rect> {
		vec![point_rect(self.client_x(), self.client_y())]
	}
}

impl GetRects for web_sys::TouchEvent {
	fn get_rects(&self) -> Vec<Rect> {
		let touches = self.touches();
		let mut rects = Vec::with_capacity(touches.length() as usize);

		for index in 0..touches.length() {
			let Some(touch) = touches.item(index) else {
				continue;
			};

			let offset_x = touch.radius_x();
			let offset_y = touch.radius_y();

			rects.push(Rect {
				top: (touch.client_y() - offset_y).into(),
				right: (touch.client_x() + offset_x).into(),
				bottom: (touch.client_y() + offset_y).into(),
				left: (touch.client_x() - offset_x).into(),
			});
		}

		rects
	}
}

fn point_rect(x: i32, y: i32) -> Rect {
	Rect {
		top: f64::from(y),
		right: f64::from(x),
		bottom: f64::from(y),
		left: f64::from(x),
	}
}

/// Any event that can be pressed.
#[derive(Clone)]
pub enum FocusableEvent {
	Mouse(MouseEvent, Option<FocusableElement>),
	Keyboard(KeyboardEvent, Option<FocusableElement>),
	Touch(web_sys::TouchEvent, Option<FocusableElement>),
	Drag(DragEvent, Option<FocusableElement>),
	Pointer(PointerEvent, Option<FocusableElement>),
	Wheel(WheelEvent, Option<FocusableElement>),
}

/// Match the wrapped event of a [`FocusableEvent`] and run the same expression
/// against each variant.
macro_rules! on_wrapped_event {
	($self:expr, $event:ident => $body:expr) => {
		match $self {
			FocusableEvent::Mouse($event, _) => $body,
			FocusableEvent::Keyboard($event, _) => $body,
			FocusableEvent::Touch($event, _) => $body,
			FocusableEvent::Drag($event, _) => $body,
			FocusableEvent::Pointer($event, _) => $body,
			FocusableEvent::Wheel($event, _) => $body,
		}
	};
}

impl FocusableEvent {
	pub fn focusable_target(&self) -> FocusableElement {
		match self {
			FocusableEvent::Mouse(_, element)
			| FocusableEvent::Keyboard(_, element)
			| FocusableEvent::Touch(_, element)
			| FocusableEvent::Drag(_, element)
			| FocusableEvent::Pointer(_, element)
			| FocusableEvent::Wheel(_, element) => {
				element
					.clone()
					.unwrap_or_else(|| self.current_target().to_focusable_element())
			}
		}
	}

	pub fn current_target(&self) -> Element {
		on_wrapped_event!(self, event => event
			.current_target()
			.and_then(|target| target.dyn_into::<Element>().ok())
			.expect("the current target is available during event dispatch"))
	}

	pub fn shift_key(&self) -> bool {
		on_wrapped_event!(self, event => event.shift_key())
	}

	pub fn ctrl_key(&self) -> bool {
		on_wrapped_event!(self, event => event.ctrl_key())
	}

	pub fn alt_key(&self) -> bool {
		on_wrapped_event!(self, event => event.alt_key())
	}

	pub fn meta_key(&self) -> bool {
		on_wrapped_event!(self, event => event.meta_key())
	}
}

/// The props accepted by [`use_press`].
#[derive(TypedBuilder, Default)]
pub struct UsePressProps {
	/// Handler that is called when the press is released over the target.
	#[builder(default, setter(strip_option))]
	pub on_press: Option<BoxedPressEventCallback>,

	/// Handler that is called when a press interaction starts.
	#[builder(default, setter(strip_option))]
	pub on_press_start: Option<BoxedPressEventCallback>,

	/// Handler that is called when a press interaction ends, either over the
	/// target or when the pointer leaves the target.
	#[builder(default, setter(strip_option))]
	pub on_press_end: Option<BoxedPressEventCallback>,

	/// Handler that is called when the press state changes.
	#[builder(default, setter(strip_option))]
	pub on_press_change: Option<BoxedPressChangeCallback>,

	/// Handler that is called when a press is released over the target,
	/// regardless of whether it started on the target or not.
	#[builder(default, setter(strip_option))]
	pub on_press_up: Option<BoxedPressEventCallback>,

	/// Whether the target is in a controlled press state (e.g. an overlay it
	/// triggers is open).
	#[builder(default, setter(strip_option, into))]
	pub is_pressed: Option<Signal<bool>>,

	/// Whether the press events should be disabled.
	#[builder(default, setter(strip_option, into))]
	pub is_disabled: Option<Signal<bool>>,

	/// Whether the target should not receive focus on press.
	#[builder(default, setter(strip_option, into))]
	pub prevent_focus_on_press: Option<Signal<bool>>,

	/// Whether press events should be canceled when the pointer leaves the
	/// target while pressed. By default this is `false`, which means if the
	/// pointer returns back over the target while still pressed,
	/// `on_press_start` will be fired again. If set to `true`, the press is
	/// canceled when the pointer leaves the target and `on_press_start` will
	/// not be fired if the pointer returns.
	#[builder(default, setter(strip_option, into))]
	pub should_cancel_on_pointer_exit: Option<Signal<bool>>,

	/// Whether text selection should be enabled on the pressable element.
	#[builder(default, setter(strip_option, into))]
	pub allow_text_selection_on_press: Option<Signal<bool>>,
}

/// The four keyboard-modifier flags mirror the upstream `react-aria` event
/// shape, so the struct intentionally carries more than three booleans.
#[allow(clippy::struct_excessive_bools)]
#[derive(TypedBuilder, Clone, Debug)]
pub struct PressEvent {
	/// The type of press event being fired.
	pub event_type: PressEventType,

	/// The pointer type that triggered the press event.
	pub pointer_type: PointerType,

	/// The target element of the press event.
	///
	/// Get the element from the target as shown here:
	/// <https://users.rust-lang.org/t/get-element-from-web-sys-eventtarget/44925>
	pub target: Element,

	/// Whether the shift keyboard modifier was held during the press event.
	pub shift_key: bool,

	/// Whether the ctrl keyboard modifier was held during the press event.
	pub ctrl_key: bool,

	/// Whether the meta keyboard modifier was held during the press event.
	pub meta_key: bool,

	/// Whether the alt keyboard modifier was held during the press event.
	pub alt_key: bool,
}

impl AsRef<PressEvent> for PressEvent {
	fn as_ref(&self) -> &PressEvent {
		self
	}
}

impl PressEvent {
	pub fn create(
		pointer_type: &PointerType,
		event_type: PressEventType,
		focusable_event: &FocusableEvent,
	) -> Self {
		Self::builder()
			.event_type(event_type)
			.pointer_type(*pointer_type)
			.target(focusable_event.current_target())
			.shift_key(focusable_event.shift_key())
			.meta_key(focusable_event.meta_key())
			.ctrl_key(focusable_event.ctrl_key())
			.alt_key(focusable_event.alt_key())
			.build()
	}
}

#[derive(Copy, Clone, Debug)]
pub enum PressEventType {
	PressStart,
	PressEnd,
	PressUp,
	Press,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PointerType {
	#[default]
	Unsupported,
	Mouse,
	Pen,
	Touch,
	Keyboard,
	Virtual,
}

impl From<&str> for PointerType {
	fn from(value: &str) -> Self {
		match value {
			"mouse" => Self::Mouse,
			"pen" => Self::Pen,
			"touch" => Self::Touch,
			"keyboard" => Self::Keyboard,
			"virtual" => Self::Virtual,
			_ => Self::Unsupported,
		}
	}
}

impl From<String> for PointerType {
	fn from(value: String) -> Self {
		Self::from(value.as_str())
	}
}

impl From<PointerEvent> for PointerType {
	fn from(value: PointerEvent) -> Self {
		Self::from(value.pointer_type())
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
		let disabled = RwSignal::new(false);
		let press = use_press(
			UsePressProps::builder()
				.on_press_start(Box::new(|_: &PressEvent| {}))
				.is_disabled(Signal::derive(move || disabled.get()))
				.build(),
		);

		view! {
			<button
				on:click=move |event| (press.on_click)(event)
				on:dragstart=move |event| (press.on_drag_start)(event)
				on:keydown=move |event| (press.on_key_down)(event)
				on:keyup=move |event| (press.on_key_up)(event)
				on:mousedown=move |event| (press.on_mouse_down)(event)
				on:pointerdown=move |event| (press.on_pointer_down)(event)
				on:pointerup=move |event| (press.on_pointer_up)(event)
			>
				"Example"
			</button>
		}
	}

	#[wasm_bindgen_test]
	fn basic() {
		console_error_panic_hook::set_once();

		mount_to_body(|| view! { <Example /> });

		let button = document()
			.query_selector("button")
			.unwrap()
			.unwrap()
			.unchecked_into::<HtmlButtonElement>();

		assert_eq!(button.inner_html(), "Example");
		button.click();
	}
}
