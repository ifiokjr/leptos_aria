use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;
use leptos_aria_utils::GlobalListeners;
use typed_builder::TypedBuilder;
use wasm_bindgen::JsCast;
use web_sys::Element;
use web_sys::FocusEvent as WebFocusEvent;

use crate::use_focus::BoxedFocusChangeCallback;
use crate::use_focus::BoxedFocusEventCallback;
use crate::use_focus::FocusChangeCallback;
use crate::use_focus::FocusEvent;
use crate::use_focus::FocusEventCallback;
use crate::use_focus::call_change_event;
use crate::use_focus::call_focus_event;
use crate::use_focus::wrap_change_callback;
use crate::use_focus::wrap_focus_callback;

/// The props accepted by [`use_focus_within`].
#[derive(TypedBuilder, Default)]
pub struct UseFocusWithinProps {
	/// Handler that is called when the target element or a descendant receives
	/// focus.
	#[builder(default, setter(strip_option))]
	pub on_focus_within: Option<BoxedFocusEventCallback>,

	/// Handler that is called when the target element and all descendants lose
	/// focus.
	#[builder(default, setter(strip_option))]
	pub on_blur_within: Option<BoxedFocusEventCallback>,

	/// Handler that is called when the focus within state changes.
	#[builder(default, setter(strip_option))]
	pub on_focus_within_change: Option<BoxedFocusChangeCallback>,

	/// Whether the focus within events should be disabled.
	#[builder(default, setter(strip_option, into))]
	pub is_disabled: Option<Signal<bool>>,
}

/// The props returned by [`use_focus_within`] which should be spread onto the
/// target element.
#[derive(Clone)]
pub struct FocusWithinResult {
	/// Whether an element inside the target is currently focused.
	pub is_focus_within: Signal<bool>,

	pub on_focus: Rc<dyn Fn(&WebFocusEvent)>,
	pub on_blur: Rc<dyn Fn(&WebFocusEvent)>,
}

/// Handles focus events for the target and its descendants.
///
/// A blur only counts when focus moves outside the target subtree, and focus
/// moving to a removed element is detected through a capture-phase listener
/// since browsers fire no blur events for elements removed from the DOM.
pub fn use_focus_within(props: UseFocusWithinProps) -> FocusWithinResult {
	let state = Rc::new(FocusWithinState::new(props));
	let focus_within = state.is_focus_within;
	let is_focus_within = Signal::derive(move || focus_within.get());

	FocusWithinResult {
		is_focus_within,
		on_focus: focus_within_handler(&state, FocusWithinState::on_focus),
		on_blur: focus_within_handler(&state, FocusWithinState::on_blur),
	}
}

/// Wrap a [`FocusWithinState`] handler method as a cloneable event callback.
fn focus_within_handler(
	state: &Rc<FocusWithinState>,
	method: fn(&Rc<FocusWithinState>, &WebFocusEvent),
) -> Rc<dyn Fn(&WebFocusEvent)> {
	let state = Rc::clone(state);

	Rc::new(move |event| method(&state, event))
}

/// The state shared by the focus within handlers of one hook instance.
struct FocusWithinState {
	listeners: Rc<RefCell<GlobalListeners>>,
	is_focus_within: RwSignal<bool>,
	on_focus_within: Option<FocusEventCallback>,
	on_blur_within: Option<FocusEventCallback>,
	on_focus_within_change: Option<FocusChangeCallback>,
}

impl FocusWithinState {
	fn new(props: UseFocusWithinProps) -> Self {
		Self {
			listeners: Rc::new(RefCell::new(GlobalListeners::default())),
			is_focus_within: RwSignal::new(false),
			on_focus_within: props.on_focus_within.map(wrap_focus_callback),
			on_blur_within: props.on_blur_within.map(wrap_focus_callback),
			on_focus_within_change: props.on_focus_within_change.map(wrap_change_callback),
		}
	}

	/// The focus handler: start focus-within when the active element moves
	/// inside the target.
	fn on_focus(state: &Rc<Self>, event: &WebFocusEvent) {
		// Ignore events bubbling through portals.
		let Some(current_target) = current_target(event) else {
			return;
		};

		if !contains_target(event, &current_target) || is_already_focus_within(state) {
			return;
		}

		call_focus_event(state.on_focus_within.as_ref(), &event_to_focus_event(event));
		call_change_event(state.on_focus_within_change.as_ref(), true);

		state.is_focus_within.set(true);

		Self::watch_for_focus_escape(state, &current_target);
	}

	/// The blur handler: end focus-within only when focus leaves the subtree.
	fn on_blur(state: &Rc<Self>, event: &WebFocusEvent) {
		// Ignore events bubbling through portals.
		let Some(current_target) = current_target(event) else {
			return;
		};

		// Moving focus inside the element must not end the focus-within, so
		// only trigger when the current target does not contain the related
		// target focus is moving to.
		let related_inside = event
			.related_target()
			.and_then(|related| related.dyn_into::<Element>().ok())
			.is_some_and(|related| current_target.contains(Some(&related)));

		if !contains_target(event, &current_target) || related_inside {
			return;
		}

		Self::trigger_blur_within(state, event);
	}

	fn trigger_blur_within(&self, event: &WebFocusEvent) {
		if !self.is_focus_within.get_untracked() {
			return;
		}

		self.is_focus_within.set(false);
		self.listeners.borrow_mut().remove_all_listeners();

		call_focus_event(self.on_blur_within.as_ref(), &event_to_focus_event(event));
		call_change_event(self.on_focus_within_change.as_ref(), false);
	}

	/// Browsers do not fire blur events when elements are removed from the
	/// DOM, so a capture-phase focus listener ends the focus-within once focus
	/// lands outside the tracked subtree.
	fn watch_for_focus_escape(state: &Rc<Self>, current_target: &Element) {
		let watcher = Rc::clone(state);
		let tracked_target = current_target.clone();

		state.listeners.borrow_mut().add::<WebFocusEvent, _>(
			&document(),
			"focus",
			move |event| {
				let focus_inside = event
					.target()
					.and_then(|target| target.dyn_into::<Element>().ok())
					.is_some_and(|target| tracked_target.contains(Some(&target)));

				if !focus_inside {
					watcher.trigger_blur_within(&event);
				}
			},
			true,
		);
	}
}

/// The focus handler double-checks the active element in case a chained focus
/// handler already moved focus somewhere else.
fn is_already_focus_within(state: &Rc<FocusWithinState>) -> bool {
	state.is_focus_within.get_untracked()
}

fn current_target(event: &WebFocusEvent) -> Option<Element> {
	event
		.current_target()
		.and_then(|target| target.dyn_into::<Element>().ok())
}

fn contains_target(event: &WebFocusEvent, current_target: &Element) -> bool {
	event
		.target()
		.and_then(|target| target.dyn_into::<Element>().ok())
		.is_some_and(|target| current_target.contains(Some(&target)))
}

fn event_to_focus_event(event: &WebFocusEvent) -> FocusEvent {
	FocusEvent::from_native(event)
}

fn document() -> web_sys::Document {
	web_sys::window()
		.expect("the `window` global is available in the browser")
		.document()
		.expect("the `document` global is available in the browser")
}
