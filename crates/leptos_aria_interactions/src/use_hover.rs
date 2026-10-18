use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use leptos::prelude::*;
use leptos_aria_utils::GlobalListeners;
use typed_builder::TypedBuilder;
use wasm_bindgen::JsCast;
use web_sys::Element;
use web_sys::PointerEvent;

use crate::use_press::PointerType;

/// The event passed to the hover handlers.
#[derive(Clone, Debug)]
pub struct HoverEvent {
	/// The element that was hovered.
	pub target: Element,

	/// The pointer type that triggered the hover event.
	pub pointer_type: PointerType,
}

/// A callback invoked for a [`HoverEvent`].
pub type BoxedHoverEventCallback = Box<dyn Fn(&HoverEvent)>;

/// A callback invoked when the hover state changes.
pub type BoxedHoverChangeCallback = Box<dyn Fn(bool)>;

type HoverEventCallback = Rc<dyn Fn(&HoverEvent)>;
type HoverChangeCallback = Rc<dyn Fn(bool)>;

// iOS fires onPointerEnter twice: once with pointerType="touch" and again
// with pointerType="mouse". These emulated events are ignored so they do not
// trigger hover behavior. See <https://bugs.webkit.org/show_bug.cgi?id=214609>.
thread_local! {
	static IGNORE_EMULATED_MOUSE_EVENTS: Cell<bool> = const { Cell::new(false) };
	static HOVER_COUNT: Cell<u32> = const { Cell::new(0) };
	static TOUCH_LISTENERS: RefCell<GlobalListeners> = RefCell::default();
}

/// How long emulated mouse events are ignored after a touch.
const EMULATED_MOUSE_TIMEOUT: Duration = Duration::from_millis(500);

/// The props accepted by [`use_hover`].
#[derive(TypedBuilder, Default)]
pub struct UseHoverProps {
	/// Handler that is called when a hover interaction starts.
	#[builder(default, setter(strip_option))]
	pub on_hover_start: Option<BoxedHoverEventCallback>,

	/// Handler that is called when a hover interaction ends.
	#[builder(default, setter(strip_option))]
	pub on_hover_end: Option<BoxedHoverEventCallback>,

	/// Handler that is called when the hover state changes.
	#[builder(default, setter(strip_option))]
	pub on_hover_change: Option<BoxedHoverChangeCallback>,

	/// Whether the hover events should be disabled.
	#[builder(default, setter(strip_option, into))]
	pub is_disabled: Option<Signal<bool>>,
}

/// The props returned by [`use_hover`] which should be spread onto the target
/// element.
#[derive(Clone)]
pub struct HoverResult {
	/// Whether the target is currently hovered.
	pub is_hovered: Signal<bool>,

	pub on_pointer_enter: Rc<dyn Fn(&PointerEvent)>,
	pub on_pointer_leave: Rc<dyn Fn(&PointerEvent)>,
}

/// Handles pointer hover interactions for an element.
///
/// Normalizes behavior across browsers and platforms, and ignores emulated
/// mouse events on touch devices.
pub fn use_hover(props: UseHoverProps) -> HoverResult {
	let state = Rc::new(HoverState::new(props));
	let hovered = state.is_hovered;
	let is_hovered = Signal::derive(move || hovered.get());

	setup_global_touch_events();

	HoverResult {
		is_hovered,
		on_pointer_enter: hover_handler(&state, HoverState::on_pointer_enter),
		on_pointer_leave: hover_handler(&state, HoverState::on_pointer_leave),
	}
}

/// Wrap a [`HoverState`] handler method as a cloneable event callback.
fn hover_handler(
	state: &Rc<HoverState>,
	method: fn(&Rc<HoverState>, &PointerEvent),
) -> Rc<dyn Fn(&PointerEvent)> {
	let state = Rc::clone(state);

	Rc::new(move |event| method(&state, event))
}

/// The state shared by the hover handlers of one hook instance.
struct HoverState {
	listeners: Rc<RefCell<GlobalListeners>>,
	target: RwSignal<Option<Element>>,
	is_hovered: RwSignal<bool>,
	is_disabled: Signal<bool>,
	on_hover_start: Option<HoverEventCallback>,
	on_hover_end: Option<HoverEventCallback>,
	on_hover_change: Option<HoverChangeCallback>,
}

impl HoverState {
	fn new(props: UseHoverProps) -> Self {
		Self {
			listeners: Rc::new(RefCell::new(GlobalListeners::default())),
			target: RwSignal::new(None),
			is_hovered: RwSignal::new(false),
			is_disabled: props.is_disabled.unwrap_or_else(|| Signal::from(false)),
			on_hover_start: props.on_hover_start.map(wrap_hover_callback),
			on_hover_end: props.on_hover_end.map(wrap_hover_callback),
			on_hover_change: props.on_hover_change.map(wrap_change_callback),
		}
	}

	fn on_pointer_enter(state: &Rc<Self>, event: &PointerEvent) {
		// Ignore the emulated mouse events a touch produces shortly after a
		// touch interaction.
		let is_emulated_mouse = IGNORE_EMULATED_MOUSE_EVENTS.get()
			&& PointerType::from(event.pointer_type()) == PointerType::Mouse;

		if !is_emulated_mouse {
			Self::trigger_hover_start(state, event);
		}
	}

	fn on_pointer_leave(state: &Rc<Self>, event: &PointerEvent) {
		let pointer_type = PointerType::from(event.pointer_type());

		if !state.is_disabled.get_untracked() && contains_target(event) {
			Self::trigger_hover_end(state, event, pointer_type);
		}
	}

	/// Fire the hover start, unless disabled, touch-driven, or already
	/// hovering.
	fn trigger_hover_start(state: &Rc<Self>, event: &PointerEvent) {
		let pointer_type = PointerType::from(event.pointer_type());

		let Some(target) = enter_target(event) else {
			return;
		};

		let inactive = state.is_disabled.get_untracked()
			|| pointer_type == PointerType::Touch
			|| state.is_hovered.get_untracked();

		if inactive || !contains_target(event) {
			return;
		}

		state.is_hovered.set(true);
		state.target.set(Some(target.clone()));

		Self::watch_for_hover_target_removal(state);

		let hover_event = HoverEvent {
			target,
			pointer_type,
		};

		call_hover_event(state.on_hover_start.as_ref(), &hover_event);
		call_change_event(state.on_hover_change.as_ref(), true);
	}

	/// Fire the hover end, unless touch-driven or not hovering.
	fn trigger_hover_end(state: &Rc<Self>, _event: &PointerEvent, pointer_type: PointerType) {
		let Some(target) = state.target.get_untracked() else {
			return;
		};

		if pointer_type == PointerType::Touch || !state.is_hovered.get_untracked() {
			return;
		}

		state.is_hovered.set(false);
		state.target.set(None);
		state.listeners.borrow_mut().remove_all_listeners();

		let hover_event = HoverEvent {
			target,
			pointer_type,
		};

		call_hover_event(state.on_hover_end.as_ref(), &hover_event);
		call_change_event(state.on_hover_change.as_ref(), false);
	}

	/// When an element that is hovered over is removed, no `pointerleave`
	/// event is fired by the browser. However, a `pointerover` event is fired
	/// on the new target the pointer is over, so a capture-phase listener ends
	/// the hover when the pointer moves outside the tracked target.
	fn watch_for_hover_target_removal(state: &Rc<Self>) {
		let watcher = Rc::clone(state);

		state.listeners.borrow_mut().add::<PointerEvent, _>(
			&document(),
			"pointerover",
			move |event| {
				let still_over_target = watcher
					.target
					.get_untracked()
					.is_some_and(|target| target.contains(event_target(&event).as_ref()));

				if watcher.is_hovered.get_untracked() && !still_over_target {
					let pointer_type = PointerType::from(event.pointer_type());

					HoverState::trigger_hover_end(&watcher, &event, pointer_type);
				}
			},
			true,
		);
	}
}

fn wrap_hover_callback(callback: BoxedHoverEventCallback) -> HoverEventCallback {
	Rc::new(move |event: &HoverEvent| callback(event))
}

fn wrap_change_callback(callback: BoxedHoverChangeCallback) -> HoverChangeCallback {
	Rc::new(move |hovered: bool| callback(hovered))
}

fn call_hover_event(callback: Option<&HoverEventCallback>, event: &HoverEvent) {
	if let Some(callback) = callback {
		callback(event);
	}
}

fn call_change_event(callback: Option<&HoverChangeCallback>, hovered: bool) {
	if let Some(callback) = callback {
		callback(hovered);
	}
}

fn contains_target(event: &PointerEvent) -> bool {
	match (
		event.current_target().and_then(dyn_element),
		event_target(event),
	) {
		(Some(current_target), Some(target)) => current_target.contains(Some(target.as_ref())),
		_ => false,
	}
}

fn enter_target(event: &PointerEvent) -> Option<Element> {
	event
		.current_target()
		.and_then(dyn_element)
		.filter(|target| target.contains(event_target(event).as_ref()))
}

fn event_target(event: &PointerEvent) -> Option<web_sys::Node> {
	event
		.target()
		.and_then(|target| target.dyn_into::<web_sys::Node>().ok())
}

fn dyn_element(target: web_sys::EventTarget) -> Option<Element> {
	target.dyn_into::<Element>().ok()
}

fn document() -> web_sys::Document {
	web_sys::window()
		.expect("the `window` global is available in the browser")
		.document()
		.expect("the `document` global is available in the browser")
}

/// Register the global touch handling the first hook needs, and drop it when
/// the last hook goes away.
fn setup_global_touch_events() {
	if HOVER_COUNT.get() == 0 {
		TOUCH_LISTENERS.with_borrow_mut(|listeners| {
			listeners.add::<PointerEvent, _>(
				&document(),
				"pointerup",
				|event: PointerEvent| handle_global_pointer_up(&event),
				false,
			);
		});
	}

	HOVER_COUNT.set(HOVER_COUNT.get() + 1);

	on_cleanup(move || {
		TOUCH_LISTENERS.with_borrow_mut(|listeners| {
			if HOVER_COUNT.get() == 1 {
				listeners.remove_all_listeners();
			}

			HOVER_COUNT.set(HOVER_COUNT.get() - 1);
		});
	});
}

fn handle_global_pointer_up(event: &PointerEvent) {
	if event.pointer_type() == "touch" {
		ignore_emulated_mouse_events();
	}
}

/// Ignore emulated mouse events for a short while after a touch.
///
/// On other devices without the iOS quirk a mouse hover in the distant future
/// must not stay ignored because of an earlier touch.
fn ignore_emulated_mouse_events() {
	IGNORE_EMULATED_MOUSE_EVENTS.set(true);

	set_timeout(
		|| IGNORE_EMULATED_MOUSE_EVENTS.set(false),
		EMULATED_MOUSE_TIMEOUT,
	);
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn pointer_types_map_from_strings() {
		assert_eq!(PointerType::from("mouse"), PointerType::Mouse);
		assert_eq!(PointerType::from("touch"), PointerType::Touch);
		assert_eq!(PointerType::from("pen"), PointerType::Pen);
	}
}
