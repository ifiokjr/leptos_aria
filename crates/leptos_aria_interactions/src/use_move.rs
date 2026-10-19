use std::cell::RefCell;
use std::rc::Rc;

use leptos_aria_utils::GlobalListeners;
use typed_builder::TypedBuilder;

use crate::use_press::PointerType;

/// The event passed to the move handlers. The modifier flags mirror the
/// upstream event shape, so the struct carries more than three booleans.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Copy, Debug)]
pub struct MoveEvent {
	/// The pointer type driving the move.
	pub pointer_type: PointerType,

	/// The horizontal delta since the last move.
	pub delta_x: f64,

	/// The vertical delta since the last move.
	pub delta_y: f64,

	/// Whether the shift key was held.
	pub shift_key: bool,

	/// Whether the ctrl key was held.
	pub ctrl_key: bool,

	/// Whether the meta key was held.
	pub meta_key: bool,

	/// Whether the alt key was held.
	pub alt_key: bool,
}

/// A callback invoked for a [`MoveEvent`].
pub type BoxedMoveCallback = Box<dyn Fn(&MoveEvent)>;

/// The props accepted by [`use_move`].
#[derive(TypedBuilder, Default)]
pub struct UseMoveProps {
	/// Handler that is called when a move interaction starts.
	#[builder(default, setter(strip_option))]
	pub on_move_start: Option<BoxedMoveCallback>,

	/// Handler that is called for every movement delta.
	#[builder(default, setter(strip_option))]
	pub on_move: Option<BoxedMoveCallback>,

	/// Handler that is called when a move interaction ends.
	#[builder(default, setter(strip_option))]
	pub on_move_end: Option<BoxedMoveCallback>,
}

/// The props returned by [`use_move`] which should be spread onto the target
/// element.
#[derive(Clone)]
pub struct MoveResult {
	pub on_pointer_down: Rc<dyn Fn(&web_sys::PointerEvent)>,
}

/// The end event of a move gesture: zero deltas, the modifiers preserved.
fn end_event(event: &web_sys::PointerEvent) -> MoveEvent {
	MoveEvent {
		pointer_type: PointerType::from(event.pointer_type()),
		delta_x: 0.0,
		delta_y: 0.0,
		shift_key: event.shift_key(),
		ctrl_key: event.ctrl_key(),
		meta_key: event.meta_key(),
		alt_key: event.alt_key(),
	}
}

/// The mutable state of one move gesture.
#[derive(Default)]
struct MoveState {
	did_move: bool,
	last_x: f64,
	last_y: f64,
	has_position: bool,
}

/// Handles move interactions across pointer and touch devices.
///
/// A move gesture starts on pointer or touch down, reports the delta of every
/// movement through `on_move`, and ends on pointer up or cancel. It powers the
/// draggable surfaces: color areas, sliders, and wheels.
pub fn use_move(props: UseMoveProps) -> MoveResult {
	let state = Rc::new(RefCell::new(MoveState::default()));
	let wrap = |callback: BoxedMoveCallback| {
		let callback: Rc<dyn Fn(&MoveEvent)> = Rc::new(move |event: &MoveEvent| callback(event));

		callback
	};

	let callbacks = MoveCallbacks {
		on_move_start: props.on_move_start.map(wrap),
		on_move: props.on_move.map(wrap),
		on_move_end: props.on_move_end.map(wrap),
	};

	MoveResult {
		on_pointer_down: Rc::new(move |event| begin_move(&state, &callbacks, event)),
	}
}

/// The user callbacks of one move gesture; the `on_move` prefix mirrors the
/// upstream prop names.
#[allow(clippy::struct_field_names)]
struct MoveCallbacks {
	on_move_start: Option<MoveEventCallback>,
	on_move: Option<MoveEventCallback>,
	on_move_end: Option<MoveEventCallback>,
}

/// Start tracking a move gesture and register its global listeners.
fn begin_move(
	state: &Rc<RefCell<MoveState>>,
	callbacks: &MoveCallbacks,
	event: &web_sys::PointerEvent,
) {
	if event.button() != 0 {
		return;
	}

	event.prevent_default();
	event.stop_propagation();
	reset_move_state(state, event);

	let listeners = Rc::new(RefCell::new(GlobalListeners::default()));

	bind_move_listeners(state, callbacks, &listeners);
}

/// Reset the gesture state at pointer down.
fn reset_move_state(state: &Rc<RefCell<MoveState>>, event: &web_sys::PointerEvent) {
	let mut state = state.borrow_mut();

	state.did_move = false;
	state.last_x = f64::from(event.client_x());
	state.last_y = f64::from(event.client_y());
	state.has_position = true;
}

/// Register the document-level move, up, and cancel listeners of a gesture.
fn bind_move_listeners(
	state: &Rc<RefCell<MoveState>>,
	callbacks: &MoveCallbacks,
	listeners: &Rc<RefCell<GlobalListeners>>,
) {
	let move_listener = move_listener(state, callbacks);
	let end_listener = end_listener(state, callbacks, listeners);

	let mut bound = listeners.borrow_mut();

	for event_type in ["pointermove", "pointerup", "pointercancel"] {
		let listener = Rc::clone(&move_listener_for(
			event_type,
			&move_listener,
			&end_listener,
		));

		bound.add::<web_sys::PointerEvent, _>(
			&document(),
			event_type,
			move |event| listener(event),
			false,
		);
	}
}

/// Choose the listener for a document event type.
fn move_listener_for(
	event_type: &str,
	move_listener: &Rc<dyn Fn(web_sys::PointerEvent)>,
	end_listener: &Rc<dyn Fn(web_sys::PointerEvent)>,
) -> Rc<dyn Fn(web_sys::PointerEvent)> {
	match event_type {
		"pointermove" => Rc::clone(move_listener),
		_ => Rc::clone(end_listener),
	}
}

/// The pointermove listener: report deltas and the move-start once.
fn move_listener(
	state: &Rc<RefCell<MoveState>>,
	callbacks: &MoveCallbacks,
) -> Rc<dyn Fn(web_sys::PointerEvent)> {
	let state = Rc::clone(state);
	let on_move_start = callbacks.on_move_start.clone();
	let on_move = callbacks.on_move.clone();

	Rc::new(move |event: web_sys::PointerEvent| {
		let (delta_x, delta_y) = record_delta(&state, &event);
		let move_event = delta_event(&event, delta_x, delta_y);

		let mut state = state.borrow_mut();

		if !state.did_move {
			state.did_move = true;

			if let Some(on_move_start) = on_move_start.as_ref() {
				(*on_move_start)(&move_event);
			}
		}

		if let Some(on_move) = on_move.as_ref() {
			(*on_move)(&move_event);
		}
	})
}

/// The pointerup and pointercancel listener: end the gesture.
fn end_listener(
	state: &Rc<RefCell<MoveState>>,
	callbacks: &MoveCallbacks,
	listeners: &Rc<RefCell<GlobalListeners>>,
) -> Rc<dyn Fn(web_sys::PointerEvent)> {
	let state = Rc::clone(state);
	let listeners = Rc::clone(listeners);
	let on_move_end = callbacks.on_move_end.clone();

	Rc::new(move |event: web_sys::PointerEvent| {
		listeners.borrow_mut().remove_all_listeners();

		if !state.borrow().did_move {
			return;
		}

		if let Some(on_move_end) = on_move_end.as_ref() {
			(*on_move_end)(&end_event(&event));
		}
	})
}

/// Record the position delta of one movement.
fn record_delta(state: &Rc<RefCell<MoveState>>, event: &web_sys::PointerEvent) -> (f64, f64) {
	let mut state = state.borrow_mut();
	let delta_x = f64::from(event.client_x()) - state.last_x;
	let delta_y = f64::from(event.client_y()) - state.last_y;

	state.last_x = f64::from(event.client_x());
	state.last_y = f64::from(event.client_y());

	(delta_x, delta_y)
}

type MoveEventCallback = Rc<dyn Fn(&MoveEvent)>;

/// The move event of a pointer movement.
fn delta_event(event: &web_sys::PointerEvent, delta_x: f64, delta_y: f64) -> MoveEvent {
	MoveEvent {
		pointer_type: PointerType::from(event.pointer_type()),
		delta_x,
		delta_y,
		shift_key: event.shift_key(),
		ctrl_key: event.ctrl_key(),
		meta_key: event.meta_key(),
		alt_key: event.alt_key(),
	}
}

fn document() -> web_sys::Document {
	web_sys::window()
		.expect("the `window` global is available in the browser")
		.document()
		.expect("the `document` global is available in the browser")
}
