mod events;
mod hit_target;
mod validators;

use std::cell::RefCell;
use std::rc::Rc;

pub use events::BoxedPressChangeCallback;
pub use events::BoxedPressEventCallback;
pub use events::FocusableEvent;
pub use events::PointerType;
use events::PressCallback;
use events::PressChangeCallback;
pub use events::PressEvent;
use events::PressEventCallback;
pub use events::PressEventType;
use events::call_change_event;
use events::call_event;
use events::wrap_change_callback;
use events::wrap_press_event_callback;
pub use hit_target::GetRects;
pub use hit_target::Rect;
use hit_target::is_above_target;
use leptos::prelude::*;
use leptos_aria_utils::GlobalListeners;
use leptos_aria_utils::ToFocusableElement;
use leptos_aria_utils::focus_without_scrolling;
use leptos_aria_utils::is_virtual_click;
use leptos_aria_utils::is_virtual_pointer_event;
use typed_builder::TypedBuilder;
use validators::current_target_element;
use validators::event_target_element;
use validators::event_target_node;
use validators::has_link_role;
use validators::is_html_anchor_link;
use validators::is_valid_keyboard_event;
use validators::should_prevent_default;
use validators::should_prevent_default_keyboard;
use wasm_bindgen::JsCast;
use web_sys::DragEvent;
use web_sys::Element;
use web_sys::HtmlElement;
use web_sys::KeyboardEvent;
use web_sys::MouseEvent;
use web_sys::PointerEvent;

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
	let state = Rc::new(PressState::new(props));

	let controlled = state.controlled_is_pressed;
	let local = state.local_is_pressed;

	PressResult {
		is_pressed: Signal::derive(move || controlled.get() || local.get()),
		is_disabled: state.is_disabled,
		on_click: press_handler(&state, PressState::on_click),
		on_drag_start: press_handler(&state, PressState::on_drag_start),
		on_key_down: press_handler(&state, PressState::on_key_down),
		on_key_up: press_handler(&state, PressState::on_key_up),
		on_mouse_down: press_handler(&state, PressState::on_mouse_down),
		on_pointer_down: press_handler(&state, PressState::on_pointer_down),
		on_pointer_up: press_handler(&state, PressState::on_pointer_up),
	}
}

/// Wrap a [`PressState`] handler method as a cloneable event callback.
fn press_handler<E>(state: &Rc<PressState>, method: fn(&Rc<PressState>, &E)) -> PressCallback<E>
where
	E: 'static,
{
	let state = Rc::clone(state);

	Rc::new(move |event| method(&state, &event))
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

/// The mutable state shared by every handler of one press interaction,
/// mirroring the ref-held state of the upstream implementation.
struct PressState {
	listeners: Rc<RefCell<GlobalListeners>>,
	ignore_emulated_mouse_events: RwSignal<bool>,
	ignore_click_after_press: RwSignal<bool>,
	did_fire_press_start: RwSignal<bool>,
	active_pointer_id: RwSignal<Option<i32>>,
	target: RwSignal<Option<Element>>,
	is_over_target: RwSignal<bool>,
	pointer_type: RwSignal<PointerType>,
	local_is_pressed: RwSignal<bool>,
	controlled_is_pressed: Signal<bool>,
	is_disabled: Signal<bool>,
	prevent_focus_on_press: Signal<bool>,
	should_cancel_on_pointer_exit: Signal<bool>,
	allow_text_selection_on_press: Signal<bool>,
	callbacks: PressCallbacks,
}

/// The user-supplied callbacks for the press lifecycle. The `on_press` prefix
/// mirrors the upstream prop names.
#[allow(clippy::struct_field_names)]
struct PressCallbacks {
	on_press: Option<PressEventCallback>,
	on_press_start: Option<PressEventCallback>,
	on_press_end: Option<PressEventCallback>,
	on_press_change: Option<PressChangeCallback>,
	on_press_up: Option<PressEventCallback>,
}

impl PressState {
	fn new(props: UsePressProps) -> Self {
		Self {
			listeners: Rc::new(RefCell::new(GlobalListeners::default())),
			ignore_emulated_mouse_events: RwSignal::new(false),
			ignore_click_after_press: RwSignal::new(false),
			did_fire_press_start: RwSignal::new(false),
			active_pointer_id: RwSignal::new(None),
			target: RwSignal::new(None),
			is_over_target: RwSignal::new(false),
			pointer_type: RwSignal::new(PointerType::default()),
			local_is_pressed: RwSignal::new(false),
			controlled_is_pressed: props.is_pressed.unwrap_or_else(|| Signal::from(false)),
			is_disabled: props.is_disabled.unwrap_or_else(|| Signal::from(false)),
			prevent_focus_on_press: props
				.prevent_focus_on_press
				.unwrap_or_else(|| Signal::from(false)),
			should_cancel_on_pointer_exit: props
				.should_cancel_on_pointer_exit
				.unwrap_or_else(|| Signal::from(false)),
			allow_text_selection_on_press: props
				.allow_text_selection_on_press
				.unwrap_or_else(|| Signal::from(false)),
			callbacks: PressCallbacks {
				on_press: props.on_press.map(wrap_press_event_callback),
				on_press_start: props.on_press_start.map(wrap_press_event_callback),
				on_press_end: props.on_press_end.map(wrap_press_event_callback),
				on_press_change: props.on_press_change.map(wrap_change_callback),
				on_press_up: props.on_press_up.map(wrap_press_event_callback),
			},
		}
	}

	/// Fire the start of a press interaction, unless it already started or the
	/// target is disabled.
	fn trigger_press_start(&self, focusable_event: &FocusableEvent, pointer: PointerType) {
		if self.is_disabled.get_untracked() || self.did_fire_press_start.get_untracked() {
			return;
		}

		let event = PressEvent::create(&pointer, PressEventType::PressStart, focusable_event);

		call_event(self.callbacks.on_press_start.as_ref(), &event);
		call_change_event(self.callbacks.on_press_change.as_ref(), true);

		self.did_fire_press_start.set(true);
		self.local_is_pressed.set(true);
	}

	/// Fire the end of a press interaction, and the press itself when it ended
	/// over the target of a still-enabled target.
	fn trigger_press_end(
		&self,
		focusable_event: &FocusableEvent,
		pointer: PointerType,
		was_pressed: bool,
	) {
		if !self.did_fire_press_start.get_untracked() {
			return;
		}

		self.ignore_click_after_press.set(true);
		self.did_fire_press_start.set(false);

		let event = PressEvent::create(&pointer, PressEventType::PressEnd, focusable_event);

		call_event(self.callbacks.on_press_end.as_ref(), &event);
		call_change_event(self.callbacks.on_press_change.as_ref(), false);

		self.local_is_pressed.set(false);

		if was_pressed && !self.is_disabled.get_untracked() {
			let event = PressEvent::create(&pointer, PressEventType::Press, focusable_event);

			call_event(self.callbacks.on_press.as_ref(), &event);
		}
	}

	/// Fire a press released over the target, regardless of where it started.
	fn trigger_press_up(&self, focusable_event: &FocusableEvent, pointer: PointerType) {
		if self.is_disabled.get_untracked() {
			return;
		}

		let event = PressEvent::create(&pointer, PressEventType::PressUp, focusable_event);

		call_event(self.callbacks.on_press_up.as_ref(), &event);
	}

	/// Cancel an in-flight press interaction and clean up after it.
	fn cancel(&self, focusable_event: &FocusableEvent) {
		if !self.local_is_pressed.get_untracked() {
			return;
		}

		if self.is_over_target.get_untracked() {
			let pointer = self.pointer_type.get_untracked();
			self.trigger_press_end(focusable_event, pointer, false);
		}

		self.reset_press_state();
		self.restore_target_text_selection();
	}

	/// Clear the interaction state and drop every global listener.
	fn reset_press_state(&self) {
		self.local_is_pressed.set(false);
		self.is_over_target.set(false);
		self.active_pointer_id.set(None);
		self.pointer_type.set(PointerType::default());

		self.listeners.borrow_mut().remove_all_listeners();
	}

	/// Restore text selection disabled at press start, unless the caller opted
	/// out of disabling it.
	fn restore_target_text_selection(&self) {
		if self.allow_text_selection_on_press.get_untracked() {
			return;
		}

		if let Some(element) = self.target.get_untracked().as_ref() {
			restore_text_selection(element);
		}
	}

	/// Focus the pressed element without scrolling, unless the target is
	/// disabled and focus is not forced.
	fn focus_target(&self, element: impl AsRef<Element>) {
		let focusable =
			!self.is_disabled.get_untracked() && !self.prevent_focus_on_press.get_untracked();

		if focusable {
			focus_without_scrolling(element);
		}
	}

	/// The keyboard events which should activate the target.
	fn on_key_down(state: &Rc<Self>, event: &KeyboardEvent) {
		let Some(current_target) = current_target_element(event) else {
			return;
		};

		let target = event_target_node(event);

		if is_valid_keyboard_event(event, &current_target)
			&& current_target.contains(target.as_ref())
		{
			Self::start_keyboard_press(state, event, &current_target);
		} else if event.key() == "Enter" && is_html_anchor_link(&current_target) {
			// If the target is a link, we won't have handled this above
			// because we want the default browser behavior to open the link
			// when pressing Enter. But we still need to prevent default so
			// that elements above do not also handle it (e.g. table row).
			event.stop_propagation();
		}
	}

	/// Register the press start for a valid keyboard activation.
	fn start_keyboard_press(state: &Rc<Self>, event: &KeyboardEvent, current_target: &Element) {
		if should_prevent_default_keyboard(
			event_target_element(event)
				.as_ref()
				.unwrap_or(current_target),
			&event.key(),
		) {
			event.prevent_default();
		}

		event.stop_propagation();

		// If the event is repeating, it may have started on a different
		// element after which focus moved to the current element. Ignore these
		// events and only handle the first key down event.
		if state.local_is_pressed.get_untracked() || event.repeat() {
			return;
		}

		state.target.set(Some(current_target.clone()));
		state.local_is_pressed.set(true);

		let focusable_event = FocusableEvent::Keyboard(event.clone(), None);
		state.trigger_press_start(&focusable_event, PointerType::Keyboard);

		// Focus may move before the key up event, so register the event on the
		// document instead of the same element where the key down event
		// occurred.
		Self::add_global_listener(state, "keyup", Self::global_on_key_up, false);
	}

	fn on_key_up(state: &Rc<Self>, event: &KeyboardEvent) {
		let Some(current_target) = current_target_element(event) else {
			return;
		};

		if !is_valid_keyboard_event(event, &current_target)
			|| event.repeat()
			|| !current_target.contains(event_target_node(event).as_ref())
		{
			return;
		}

		let focusable_event = FocusableEvent::Keyboard(
			event.clone(),
			state
				.target
				.get_untracked()
				.map(|target| target.to_focusable_element()),
		);

		state.trigger_press_up(&focusable_event, PointerType::Keyboard);
	}

	/// The document-level key up listener registered at keyboard press start.
	fn global_on_key_up(state: &Rc<Self>, event: &KeyboardEvent) {
		if !state.local_is_pressed.get_untracked() {
			return;
		}

		let Some(element) = state.target.get_untracked() else {
			return;
		};

		if !is_valid_keyboard_event(event, &element) {
			return;
		}

		if should_prevent_default_keyboard(
			event_target_element(event).as_ref().unwrap_or(&element),
			&event.key(),
		) {
			event.prevent_default();
		}

		event.stop_propagation();
		state.local_is_pressed.set(false);

		let focusable_event =
			FocusableEvent::Keyboard(event.clone(), Some(element.to_focusable_element()));
		let contains_target = element.contains(event_target_node(event).as_ref());

		state.trigger_press_end(&focusable_event, PointerType::Keyboard, contains_target);
		state.reset_press_state();

		// If the target is a link, trigger the click method to open the URL,
		// but defer triggering press end until the click event handler.
		if element.is_instance_of::<HtmlElement>() && has_link_role(&element) && contains_target {
			element.unchecked_ref::<HtmlElement>().click();
		}
	}

	fn on_click(state: &Rc<Self>, event: &MouseEvent) {
		let Some(current_target) = current_target_element(event) else {
			return;
		};

		if !current_target.contains(event_target_node(event).as_ref()) {
			return;
		}

		// Ensure it was the main mouse button that was clicked.
		if event.button() != 0 {
			return;
		}

		event.stop_propagation();

		if state.is_disabled.get_untracked() {
			event.prevent_default();
		}

		// If triggered from a screen reader or by using element.click(),
		// trigger as if it were a keyboard click.
		let is_virtual = !state.ignore_click_after_press.get_untracked()
			&& !state.ignore_emulated_mouse_events.get_untracked()
			&& (state.pointer_type.get_untracked() == PointerType::Virtual
				|| is_virtual_click(event));

		if is_virtual {
			state.focus_target(&current_target);

			let focusable_event = FocusableEvent::Mouse(event.clone(), None);
			let pointer = PointerType::Virtual;

			state.trigger_press_start(&focusable_event, pointer);
			state.trigger_press_up(&focusable_event, pointer);
			state.trigger_press_end(&focusable_event, pointer, true);
		}

		state.ignore_emulated_mouse_events.set(false);
		state.ignore_click_after_press.set(false);
	}

	fn on_drag_start(state: &Rc<Self>, event: &DragEvent) {
		let Some(current_target) = current_target_element(event) else {
			return;
		};

		if !current_target.contains(event_target_node(event).as_ref()) {
			return;
		}

		// Safari does not call onPointerCancel when a drag starts, whereas
		// Chrome and Firefox do.
		let focusable_event = FocusableEvent::Drag(event.clone(), None);
		state.cancel(&focusable_event);
	}

	fn on_mouse_down(_state: &Rc<Self>, event: &MouseEvent) {
		let Some(current_target) = current_target_element(event) else {
			return;
		};

		if !current_target.contains(event_target_node(event).as_ref()) || event.button() != 0 {
			return;
		}

		// Chrome and Firefox on touch Windows devices require mouse down
		// events to be canceled in addition to pointer events, or an extra
		// asynchronous focus event will be fired.
		if should_prevent_default(&current_target) {
			event.prevent_default();
		}

		event.stop_propagation();
	}

	/// The pointer events which start a press interaction.
	fn on_pointer_down(state: &Rc<Self>, event: &PointerEvent) {
		let Some(current_target) = current_target_element(event) else {
			return;
		};

		// Only handle left clicks, and ignore events that bubbled through
		// portals.
		if event.button() != 0 || !current_target.contains(event_target_node(event).as_ref()) {
			return;
		}

		// iOS safari fires pointer events from VoiceOver with incorrect
		// coordinates/target. Ignore and let the onClick handler take care of
		// it instead.
		// <https://bugs.webkit.org/show_bug.cgi?id=222627>
		// <https://bugs.webkit.org/show_bug.cgi?id=223202>
		if is_virtual_pointer_event(event) {
			state.pointer_type.set(PointerType::Virtual);
			return;
		}

		// Due to browser inconsistencies, especially on mobile browsers, we
		// prevent default on pointer down and handle focusing the pressable
		// element ourselves.
		if should_prevent_default(&current_target) {
			event.prevent_default();
		}

		state.pointer_type.set(event.pointer_type().into());
		event.stop_propagation();

		if !state.local_is_pressed.get_untracked() {
			Self::start_pointer_press(state, event, &current_target);
		}
	}

	/// Record the press start and register the global pointer listeners which
	/// follow the pointer across the page.
	fn start_pointer_press(state: &Rc<Self>, event: &PointerEvent, current_target: &Element) {
		state.local_is_pressed.set(true);
		state.is_over_target.set(true);
		state.active_pointer_id.set(Some(event.pointer_id()));
		state.target.set(Some(current_target.clone()));

		state.focus_target(current_target);

		if !state.allow_text_selection_on_press.get_untracked() {
			disable_text_selection(state.target.get_untracked().as_ref());
		}

		let focusable_event = FocusableEvent::Pointer(event.clone(), None);
		let pointer = state.pointer_type.get_untracked();

		state.trigger_press_start(&focusable_event, pointer);

		Self::add_global_listener(state, "pointermove", Self::on_pointer_move, false);
		Self::add_global_listener(state, "pointerup", Self::global_on_pointer_up, false);
		Self::add_global_listener(state, "pointercancel", Self::on_pointer_cancel, false);
	}

	/// Safari on iOS < 13.2 does not implement pointerenter/pointerleave
	/// events correctly, so pointer move events implement our own hit testing.
	/// See <https://bugs.webkit.org/show_bug.cgi?id=199803>.
	fn on_pointer_move(state: &Rc<Self>, event: &PointerEvent) {
		if Some(event.pointer_id()) != state.active_pointer_id.get_untracked() {
			return;
		}

		let Some(element) = state.target.get_untracked() else {
			return;
		};

		let focusable_event =
			FocusableEvent::Pointer(event.clone(), Some(element.to_focusable_element()));

		if is_above_target(event, &element) && !state.is_over_target.get_untracked() {
			state.is_over_target.set(true);
			let pointer = state.pointer_type.get_untracked();
			state.trigger_press_start(&focusable_event, pointer);
		} else if state.is_over_target.get_untracked() {
			state.is_over_target.set(false);
			let pointer = state.pointer_type.get_untracked();
			state.trigger_press_end(&focusable_event, pointer, false);

			if state.should_cancel_on_pointer_exit.get_untracked() {
				state.cancel(&focusable_event);
			}
		}
	}

	fn on_pointer_up(state: &Rc<Self>, event: &PointerEvent) {
		let Some(current_target) = current_target_element(event) else {
			return;
		};

		// iOS fires pointerup with zero width and height, so check the
		// pointer type recorded during pointerdown.
		if !current_target.contains(event_target_node(event).as_ref())
			|| state.pointer_type.get_untracked() == PointerType::Virtual
		{
			return;
		}

		// Only handle left clicks. Safari on iOS sometimes fires pointerup
		// events, even when the touch isn't over the target, so double check.
		if event.button() == 0 && is_above_target(event, &current_target) {
			let focusable_event = FocusableEvent::Pointer(event.clone(), None);
			let pointer = state.pointer_type.get_untracked();

			state.trigger_press_up(&focusable_event, pointer);
		}
	}

	/// The document-level pointer up listener registered at press start.
	fn global_on_pointer_up(state: &Rc<Self>, event: &PointerEvent) {
		if Some(event.pointer_id()) != state.active_pointer_id.get_untracked()
			|| !state.local_is_pressed.get_untracked()
			|| event.button() != 0
		{
			return;
		}

		let Some(element) = state.target.get_untracked() else {
			return;
		};

		let focusable_event =
			FocusableEvent::Pointer(event.clone(), Some(element.to_focusable_element()));
		let pointer = state.pointer_type.get_untracked();

		if is_above_target(event, &element) {
			state.trigger_press_end(&focusable_event, pointer, true);
		} else if state.is_over_target.get_untracked() {
			state.trigger_press_end(&focusable_event, pointer, false);
		}

		state.reset_press_state();
		state.restore_target_text_selection();
	}

	/// The document-level pointer cancel listener registered at press start.
	fn on_pointer_cancel(state: &Rc<Self>, event: &PointerEvent) {
		let focusable_event = FocusableEvent::Pointer(event.clone(), None);
		state.cancel(&focusable_event);
	}

	/// Register one of the handler methods as a document-level listener.
	fn add_global_listener<E>(
		state: &Rc<Self>,
		event_type: &'static str,
		method: fn(&Rc<Self>, &E),
		capture: bool,
	) where
		E: JsCast + 'static,
	{
		let listener = Rc::clone(state);

		state.listeners.borrow_mut().add::<E, _>(
			&document(),
			event_type,
			move |event| method(&listener, &event),
			capture,
		);
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

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
	use std::cell::RefCell;

	use leptos::mount::mount_to_body;
	use wasm_bindgen::JsCast;
	use wasm_bindgen_test::*;
	use web_sys::HtmlButtonElement;

	use super::*;

	wasm_bindgen_test_configure!(run_in_browser);

	thread_local! {
		static PRESS_COUNT: RefCell<Option<RwSignal<u32>>> = const { RefCell::new(None) };
	}

	#[component]
	fn Example() -> impl IntoView {
		let disabled = RwSignal::new(false);
		let press_count = RwSignal::new(0);
		let on_press_count = press_count;
		PRESS_COUNT.with_borrow_mut(|slot| *slot = Some(on_press_count));
		let press = use_press(
			UsePressProps::builder()
				.on_press(Box::new(|_: &PressEvent| {
					PRESS_COUNT.with_borrow(|slot| {
						if let Some(count) = slot {
							count.set(count.get_untracked() + 1);
						}
					});
				}))
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

		let count = PRESS_COUNT.with_borrow(|slot| slot.map(|count| count.get_untracked()));

		assert_eq!(count, Some(1), "a synthetic click fires on_press");
	}
}
