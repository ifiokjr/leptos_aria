use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use leptos::prelude::*;
use leptos_aria_utils::GlobalListeners;
use leptos_aria_utils::focus_without_scrolling;
use typed_builder::TypedBuilder;
use web_sys::Element;

use crate::use_press::PointerType;
use crate::use_press::PressEvent;
use crate::use_press::PressResult;
use crate::use_press::UsePressProps;
use crate::use_press::use_press;

/// How long a press must be held before it counts as a long press.
const DEFAULT_THRESHOLD: Duration = Duration::from_millis(500);

/// The pointer types a long press listens for.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum LongPressPointerType {
	/// Both mouse and touch, the default.
	#[default]
	Any,
	Mouse,
	Touch,
}

impl LongPressPointerType {
	fn accepts(self, pointer_type: PointerType) -> bool {
		match self {
			LongPressPointerType::Any => {
				matches!(pointer_type, PointerType::Mouse | PointerType::Touch)
			}
			LongPressPointerType::Mouse => pointer_type == PointerType::Mouse,
			LongPressPointerType::Touch => pointer_type == PointerType::Touch,
		}
	}
}

/// The modifier flags mirror the upstream event shape, so the struct
/// intentionally carries more than three booleans.
#[allow(clippy::struct_excessive_bools)]
/// The event passed to the long press handlers.
#[derive(Clone, Debug)]
pub struct LongPressEvent {
	/// The type of long press event being fired.
	pub event_type: LongPressEventType,

	/// The element the long press interaction targets.
	pub target: Element,

	/// The pointer type that triggered the event.
	pub pointer_type: PointerType,

	/// Whether the shift keyboard modifier was held.
	pub shift_key: bool,

	/// Whether the ctrl keyboard modifier was held.
	pub ctrl_key: bool,

	/// Whether the meta keyboard modifier was held.
	pub meta_key: bool,

	/// Whether the alt keyboard modifier was held.
	pub alt_key: bool,
}

impl LongPressEvent {
	fn from_press_event(event: &PressEvent, event_type: LongPressEventType) -> Self {
		Self {
			event_type,
			target: event.target.clone(),
			pointer_type: event.pointer_type,
			shift_key: event.shift_key,
			ctrl_key: event.ctrl_key,
			meta_key: event.meta_key,
			alt_key: event.alt_key,
		}
	}
}

/// The type of long press event being fired.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LongPressEventType {
	LongPressStart,
	LongPressEnd,
	LongPress,
}

/// A callback invoked for a [`LongPressEvent`].
pub type BoxedLongPressEventCallback = Box<dyn Fn(&LongPressEvent)>;

type LongPressEventCallback = Rc<dyn Fn(&LongPressEvent)>;

/// The props accepted by [`use_long_press`].
#[derive(TypedBuilder, Default)]
pub struct UseLongPressProps {
	/// Whether long press events should be disabled.
	#[builder(default, setter(strip_option, into))]
	pub is_disabled: Option<Signal<bool>>,

	/// Which pointer type to listen for. Both mouse and touch by default.
	#[builder(default, setter(strip_option))]
	pub pointer_type: Option<LongPressPointerType>,

	/// Handler that is called when a long press interaction starts.
	#[builder(default, setter(strip_option))]
	pub on_long_press_start: Option<BoxedLongPressEventCallback>,

	/// Handler that is called when a long press interaction ends, either over
	/// the target or when the pointer leaves the target.
	#[builder(default, setter(strip_option))]
	pub on_long_press_end: Option<BoxedLongPressEventCallback>,

	/// Handler that is called when the threshold time is met while the press
	/// is over the target.
	#[builder(default, setter(strip_option))]
	pub on_long_press: Option<BoxedLongPressEventCallback>,

	/// The amount of time to wait before triggering a long press. Half a
	/// second by default.
	#[builder(default, setter(strip_option))]
	pub threshold: Option<Duration>,
}

/// Handles long press interactions across mouse and touch devices.
///
/// Supports a customizable time threshold and normalizes behavior across
/// browsers and devices.
pub fn use_long_press(props: UseLongPressProps) -> PressResult {
	let state = Rc::new(LongPressState::new(props));
	let is_disabled = state.is_disabled;

	use_press(UsePressProps {
		is_disabled: Some(is_disabled),
		on_press_start: Some(Box::new({
			let state = Rc::clone(&state);

			move |event: &PressEvent| LongPressState::start(&state, event)
		})),
		on_press_end: Some(Box::new({
			let state = Rc::clone(&state);

			move |event: &PressEvent| LongPressState::end(&state, event)
		})),
		..Default::default()
	})
}

/// The state shared by the long press handlers of one hook instance.
struct LongPressState {
	listeners: Rc<RefCell<GlobalListeners>>,
	is_disabled: Signal<bool>,
	pointer_type: LongPressPointerType,
	threshold: Duration,
	on_long_press_start: Option<LongPressEventCallback>,
	on_long_press_end: Option<LongPressEventCallback>,
	on_long_press: Option<LongPressEventCallback>,
}

impl LongPressState {
	fn new(props: UseLongPressProps) -> Self {
		Self {
			listeners: Rc::new(RefCell::new(GlobalListeners::default())),
			is_disabled: props.is_disabled.unwrap_or_else(|| Signal::from(false)),
			pointer_type: props.pointer_type.unwrap_or_default(),
			threshold: props.threshold.unwrap_or(DEFAULT_THRESHOLD),
			on_long_press_start: props.on_long_press_start.map(wrap_long_press_callback),
			on_long_press_end: props.on_long_press_end.map(wrap_long_press_callback),
			on_long_press: props.on_long_press.map(wrap_long_press_callback),
		}
	}

	/// The press start: begin the threshold timer and guard against the
	/// context menu touch devices open on long press.
	fn start(state: &Rc<Self>, event: &PressEvent) {
		if !state.pointer_type.accepts(event.pointer_type) {
			return;
		}

		call_long_press_event(
			state.on_long_press_start.as_ref(),
			&LongPressEvent::from_press_event(event, LongPressEventType::LongPressStart),
		);

		let timer_event = LongPressEvent::from_press_event(event, LongPressEventType::LongPress);
		let threshold = state.threshold;

		set_timeout(
			{
				let state = Rc::clone(state);

				move || LongPressState::fire(state, timer_event)
			},
			threshold,
		);

		// Prevent the context menu touch devices open on long press.
		if event.pointer_type == PointerType::Touch {
			state.listeners.borrow_mut().add::<web_sys::Event, _>(
				&event.target,
				"contextmenu",
				|event: web_sys::Event| event.prevent_default(),
				false,
			);
		}
	}

	/// The threshold timer: fire the long press and prevent the default click
	/// action which follows it.
	#[allow(clippy::needless_pass_by_value)]
	fn fire(state: Rc<Self>, event: LongPressEvent) {
		// Prevent other press handlers from also handling this interaction.
		if let Ok(pointercancel) = web_sys::PointerEvent::new("pointercancel") {
			event.target.dispatch_event(&pointercancel).ok();
		}

		// Prevent the default click action (e.g. opening a link) after a long
		// press.
		state.listeners.borrow_mut().add::<web_sys::Event, _>(
			&event.target,
			"click",
			|event: web_sys::Event| event.prevent_default(),
			false,
		);

		// Ensure the target is focused; on touch devices browsers typically
		// focus on pointer up.
		let document = owner_document(&event.target);
		let needs_focus = document
			.as_ref()
			.and_then(web_sys::Document::active_element)
			.is_none_or(|active| active != event.target);

		if needs_focus {
			focus_without_scrolling(&event.target);
		}

		call_long_press_event(state.on_long_press.as_ref(), &event);
	}

	/// The press end: fire the end event for accepted pointer types.
	#[allow(clippy::needless_pass_by_value)]
	fn end(state: &Rc<Self>, event: &PressEvent) {
		if state.pointer_type.accepts(event.pointer_type) {
			call_long_press_event(
				state.on_long_press_end.as_ref(),
				&LongPressEvent::from_press_event(event, LongPressEventType::LongPressEnd),
			);
		}

		// If no contextmenu or click event follows the pointer up quickly, the
		// guards are removed so future events outside a long press are not
		// prevented.
		let listeners = Rc::clone(&state.listeners);

		set_timeout(
			move || listeners.borrow_mut().remove_all_listeners(),
			Duration::from_millis(100),
		);
	}
}

fn wrap_long_press_callback(callback: BoxedLongPressEventCallback) -> LongPressEventCallback {
	Rc::new(move |event: &LongPressEvent| callback(event))
}

fn call_long_press_event(callback: Option<&LongPressEventCallback>, event: &LongPressEvent) {
	if let Some(callback) = callback {
		callback(event);
	}
}

fn owner_document(element: &Element) -> Option<web_sys::Document> {
	element.owner_document()
}
