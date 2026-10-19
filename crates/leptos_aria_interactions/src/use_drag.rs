use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;
use leptos_aria_utils::GlobalListeners;
use typed_builder::TypedBuilder;
use wasm_bindgen::JsCast;

use crate::use_press::PointerType;

#[derive(Clone, Copy, Debug)]
pub struct DragEvent {
	/// The pointer type driving the drag.
	pub pointer_type: PointerType,

	/// The horizontal pointer position.
	pub x: f64,

	/// The vertical pointer position.
	pub y: f64,

	/// Whether the shift key was held.
	pub shift_key: bool,
}

/// A callback invoked for a [`DragEvent`].
pub type BoxedDragCallback = Box<dyn Fn(&DragEvent)>;

type DragCallback = Rc<dyn Fn(&DragEvent)>;

use std::cell::Cell;

/// A registered drop target: movement dispatch and release dispatch.
struct DropTarget {
	on_move: Rc<dyn Fn(&DragEvent)>,
	on_end: Rc<dyn Fn(&DragEvent)>,
}

thread_local! {
	static DRAG_ACTIVE: Cell<bool> = const { Cell::new(false) };
	static DROP_TARGETS: RefCell<Vec<DropTarget>> = const { RefCell::new(Vec::new()) };
}

/// Whether a drag gesture is currently in flight.
pub fn is_drag_active() -> bool {
	DRAG_ACTIVE.get()
}

/// The props accepted by [`use_drag`].
#[derive(TypedBuilder, Default)]
pub struct UseDragProps {
	/// Handler that is called when a drag interaction starts.
	#[builder(default, setter(strip_option))]
	pub on_drag_start: Option<BoxedDragCallback>,

	/// Handler that is called for every drag movement.
	#[builder(default, setter(strip_option))]
	pub on_drag_move: Option<BoxedDragCallback>,

	/// Handler that is called when a drag interaction ends.
	#[builder(default, setter(strip_option))]
	pub on_drag_end: Option<BoxedDragCallback>,

	/// How far the pointer must move before the drag starts, eight pixels by
	/// default.
	#[builder(default, setter(strip_option))]
	pub threshold: Option<f64>,
}

/// The props returned by [`use_drag`] which should be spread onto the target
/// element.
#[derive(Clone)]
pub struct DragResult {
	pub on_pointer_down: Rc<dyn Fn(&web_sys::PointerEvent)>,
}

/// The state of one in-flight drag gesture.
#[derive(Default)]
struct DragState {
	started: bool,
	start_x: f64,
	start_y: f64,
	last_x: f64,
	last_y: f64,
}

/// Handles drag interactions driven by pointer movement.
///
/// A drag starts after the pointer moves past the threshold and follows the
/// pointer across the page; the matching [`use_drop`] targets receive the
/// movement while the drag is active.
pub fn use_drag(props: UseDragProps) -> DragResult {
	let state = Rc::new(RefCell::new(DragState::default()));
	let listeners = Rc::new(RefCell::new(GlobalListeners::default()));
	let on_drag_start = props
		.on_drag_start
		.map(|callback| Rc::new(move |event: &DragEvent| callback(event)) as DragCallback);
	let on_drag_move = props
		.on_drag_move
		.map(|callback| Rc::new(move |event: &DragEvent| callback(event)) as DragCallback);
	let on_drag_end = props
		.on_drag_end
		.map(|callback| Rc::new(move |event: &DragEvent| callback(event)) as DragCallback);
	let threshold = props.threshold.unwrap_or(DEFAULT_DRAG_THRESHOLD);

	let on_pointer_down = {
		let state = Rc::clone(&state);
		let listeners = Rc::clone(&listeners);
		let on_drag_start = on_drag_start.clone();
		let on_drag_move = on_drag_move.clone();
		let on_drag_end = on_drag_end.clone();

		move |event: &web_sys::PointerEvent| {
			if event.button() != 0 {
				return;
			}

			let drag_event = |event: &web_sys::PointerEvent| {
				DragEvent {
					pointer_type: PointerType::from(event.pointer_type()),
					x: f64::from(event.client_x()),
					y: f64::from(event.client_y()),
					shift_key: event.shift_key(),
				}
			};

			{
				let mut state = state.borrow_mut();
				state.started = false;
				state.start_x = f64::from(event.client_x());
				state.start_y = f64::from(event.client_y());
				state.last_x = state.start_x;
				state.last_y = state.start_y;
			}

			let move_listener = {
				let state = Rc::clone(&state);
				let on_drag_start = on_drag_start.clone();
				let on_drag_move = on_drag_move.clone();

				move |event: web_sys::PointerEvent| {
					let x = f64::from(event.client_x());
					let y = f64::from(event.client_y());
					let mut state = state.borrow_mut();

					if !state.started && !crossed_threshold(&state, x, y, threshold) {
						return;
					}

					if !state.started {
						state.started = true;
						DRAG_ACTIVE.set(true);
						fire_drag_callback(on_drag_start.as_ref(), &drag_event(&event));
					}

					state.last_x = x;
					state.last_y = y;
					fire_drag_callback(on_drag_move.as_ref(), &drag_event(&event));

					let drag_event = drag_event(&event);

					dispatch_to_targets(&drag_event, &|target| &target.on_move);
				}
			};

			let end_listener = {
				let state = Rc::clone(&state);
				let listeners = Rc::clone(&listeners);
				let on_drag_end = on_drag_end.clone();

				move |event: web_sys::PointerEvent| {
					listeners.borrow_mut().remove_all_listeners();
					DRAG_ACTIVE.set(false);

					if !state.borrow().started {
						return;
					}

					fire_drag_callback(on_drag_end.as_ref(), &drag_event(&event));

					// The release position dispatches to every drop target;
					// each checks its own bounds before firing `on_drop`.
					let release = drag_event(&event);

					dispatch_to_targets(&release, &|target| &target.on_end);
				}
			};

			let move_listener: Rc<dyn Fn(web_sys::PointerEvent)> = Rc::new(move_listener);
			let end_listener: Rc<dyn Fn(web_sys::PointerEvent)> = Rc::new(end_listener);

			let mut bound = listeners.borrow_mut();

			bound.add::<web_sys::PointerEvent, _>(
				&document(),
				"pointermove",
				{
					let listener = Rc::clone(&move_listener);

					move |event| listener(event)
				},
				false,
			);

			bound.add::<web_sys::PointerEvent, _>(
				&document(),
				"pointerup",
				{
					let listener = Rc::clone(&end_listener);

					move |event| listener(event)
				},
				false,
			);
		}
	};

	DragResult {
		on_pointer_down: Rc::new(on_pointer_down),
	}
}

/// The props accepted by [`use_drop`].
#[derive(TypedBuilder, Default)]
pub struct UseDropProps {
	/// Handler that is called when the dragged pointer enters the target.
	#[builder(default, setter(strip_option))]
	pub on_drop_enter: Option<BoxedDragCallback>,

	/// Handler that is called while the dragged pointer is over the target.
	#[builder(default, setter(strip_option))]
	pub on_drop_move: Option<BoxedDragCallback>,

	/// Handler that is called when the dragged pointer leaves the target.
	#[builder(default, setter(strip_option))]
	pub on_drop_exit: Option<BoxedDragCallback>,

	/// Handler that is called when the pointer is released over the target.
	#[builder(default, setter(strip_option))]
	pub on_drop: Option<BoxedDragCallback>,
}

/// Receive drops from an active [`use_drag`] gesture.
///
/// The target watches the global drag position against its own bounding box:
/// enter and exit fire as the pointer crosses the edge, and `on_drop` fires
/// when the pointer is released inside.
pub fn use_drop(target: NodeRef<leptos::html::Div>, props: UseDropProps) {
	let handlers = Rc::new(DropHandlers {
		target,
		is_inside: Cell::new(false),
		on_drop_enter: props
			.on_drop_enter
			.map(|callback| Rc::new(move |event: &DragEvent| callback(event)) as DragCallback),
		on_drop_move: props
			.on_drop_move
			.map(|callback| Rc::new(move |event: &DragEvent| callback(event)) as DragCallback),
		on_drop_exit: props
			.on_drop_exit
			.map(|callback| Rc::new(move |event: &DragEvent| callback(event)) as DragCallback),
		on_drop: props
			.on_drop
			.map(|callback| Rc::new(move |event: &DragEvent| callback(event)) as DragCallback),
	});

	let move_handlers = Rc::clone(&handlers);
	let end_handlers = handlers;

	register_drop_target(
		move |event| move_handlers.handle_move(event),
		move |event| end_handlers.handle_release(event),
	);
}

/// The handlers of one drop target.
struct DropHandlers {
	target: NodeRef<leptos::html::Div>,
	is_inside: Cell<bool>,
	on_drop_enter: Option<DragCallback>,
	on_drop_move: Option<DragCallback>,
	on_drop_exit: Option<DragCallback>,
	on_drop: Option<DragCallback>,
}

impl DropHandlers {
	fn handle_move(&self, event: &DragEvent) {
		let Some(target) = self.target.get() else {
			return;
		};

		let now_inside = rect_contains(&target.get_bounding_client_rect(), event);

		match (self.is_inside.get(), now_inside) {
			(false, true) => {
				self.is_inside.set(true);
				fire_drag_callback(self.on_drop_enter.as_ref(), event);
			}
			(true, false) => {
				self.is_inside.set(false);
				fire_drag_callback(self.on_drop_exit.as_ref(), event);
			}
			(true, true) => fire_drag_callback(self.on_drop_move.as_ref(), event),
			(false, false) => {}
		}
	}

	/// The drag released: fire `on_drop` when the release landed inside.
	fn handle_release(&self, event: &DragEvent) {
		if self.is_inside.get() {
			fire_drag_callback(self.on_drop.as_ref(), event);
		}
	}
}

/// Invoke an optional drag callback.
fn fire_drag_callback(callback: Option<&DragCallback>, event: &DragEvent) {
	if let Some(callback) = callback {
		(*callback)(event);
	}
}

/// Whether a pointer position falls inside a rectangle.
fn rect_contains(rect: &web_sys::DomRect, event: &DragEvent) -> bool {
	event.x >= rect.left()
		&& event.x <= rect.right()
		&& event.y >= rect.top()
		&& event.y <= rect.bottom()
}

/// The default drag threshold in pixels.
const DEFAULT_DRAG_THRESHOLD: f64 = 8.0;

/// Whether the pointer moved past the drag threshold from the start.
fn crossed_threshold(state: &DragState, x: f64, y: f64, threshold: f64) -> bool {
	(x - state.start_x).hypot(y - state.start_y) >= threshold
}

/// The dispatch selector: which callback of a drop target receives the event.
type DispatchSelector = dyn Fn(&DropTarget) -> &Rc<dyn Fn(&DragEvent)>;

/// Dispatch a drag position to every registered drop target.
fn dispatch_to_targets(event: &DragEvent, select: &DispatchSelector) {
	let count = DROP_TARGETS.with_borrow(Vec::len);

	for index in 0..count {
		DROP_TARGETS.with_borrow(|targets| {
			if let Some(target) = targets.get(index) {
				(select(target))(event);
			}
		});
	}
}

/// Register a drop target with the page-wide drag dispatch.
///
/// Targets persist for the page: each registers once and checks its own
/// bounding box before firing, so registration order never matters.
fn register_drop_target(
	on_move: impl Fn(&DragEvent) + 'static,
	on_end: impl Fn(&DragEvent) + 'static,
) {
	DROP_TARGETS.with_borrow_mut(|targets| {
		targets.push(DropTarget {
			on_move: Rc::new(on_move),
			on_end: Rc::new(on_end),
		});
	});
}

fn document() -> web_sys::Document {
	web_sys::window()
		.expect("the `window` global is available in the browser")
		.document()
		.expect("the `document` global is available in the browser")
}

// Touch events are not part of the pointer path; the cast keeps the JsValue
// import honest for future touch support.
#[allow(dead_code)]
fn unused_cast(event: web_sys::Event) -> Option<web_sys::PointerEvent> {
	event.dyn_into::<web_sys::PointerEvent>().ok()
}
