use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;

use js_sys::JsString;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::Element;
use web_sys::Event;
use web_sys::TransitionEvent;

use crate::silly_map::Map;
use crate::silly_map::Set;

type TransitionCallback = Rc<dyn Fn()>;

/// A list of callbacks to run once no elements are transitioning.
type TransitionCallbacks = Vec<TransitionCallback>;

/// A `transitionrun` or `transitionend` listener attached to the document or
/// a single element.
type TransitionHandler = Closure<dyn FnMut(TransitionEvent)>;

/// The listener that sets up the global events once the document has loaded.
type DomContentLoadedHandler = Closure<dyn FnMut(Event)>;

/// A `transitioncancel` listener attached to one element, alongside the element
/// so it can be found again for removal.
type CancelHandler = (Element, TransitionHandler);

/// The elements currently transitioning, mapped to the set of CSS properties
/// that are transitioning for that element.
///
/// This is tracked rather than a simple count of transitions because of
/// browser bugs: Chrome sometimes fires both `transitionend` and
/// `transitioncancel` rather than one or the other, so the actual transitioning
/// properties are tracked in order to ignore these duplicate events.
type ElementTransitions = Map<Element, Set<JsString>>;

thread_local! {
	static ELEMENT_TRANSITIONS: RefCell<ElementTransitions> = RefCell::default();
	static TRANSITION_CALLBACKS: RefCell<TransitionCallbacks> = const { RefCell::new(Vec::new()) };
	static TRANSITION_CANCEL_HANDLERS: RefCell<Vec<CancelHandler>> =
		const { RefCell::new(Vec::new()) };
	static DOCUMENT_HANDLERS: RefCell<Vec<TransitionHandler>> =
		const { RefCell::new(Vec::new()) };
	static DOM_CONTENT_LOADED_HANDLER: RefCell<Option<DomContentLoadedHandler>> =
		const { RefCell::new(None) };
	static LISTENERS_SETUP: Cell<bool> = const { Cell::new(false) };
}

fn document() -> web_sys::Document {
	web_sys::window()
		.expect("the `window` global is available in the browser")
		.document()
		.expect("the `document` global is available in the browser")
}

/// Perform an action after all CSS transitions have finished on the page.
///
/// When no transitions are running the callback runs immediately; otherwise it
/// is queued to run at the end of the last transition. A single animation
/// frame is waited first so that a transition starting on mount is noticed.
pub fn run_after_transition(callback: impl Fn() + 'static) {
	setup_transition_listener();

	let callback = Rc::new(callback);

	request_animation_frame(move || {
		let no_transitions_running = ELEMENT_TRANSITIONS.with_borrow(Map::is_empty);

		if no_transitions_running {
			callback();
		} else {
			TRANSITION_CALLBACKS.with_borrow_mut(|callbacks| callbacks.push(callback));
		}
	});
}

/// Set up the global listeners for transition events on the page.
///
/// The listeners are attached lazily on the first call and never removed: they
/// only record which elements and properties are transitioning.
fn setup_transition_listener() {
	if LISTENERS_SETUP.get() {
		return;
	}

	LISTENERS_SETUP.set(true);

	if document().ready_state() == "loading" {
		let handler: DomContentLoadedHandler = Closure::new(|_: Event| setup_global_events());

		document()
			.add_event_listener_with_callback(
				"DOMContentLoaded",
				handler.as_ref().unchecked_ref::<js_sys::Function>(),
			)
			.ok();

		DOM_CONTENT_LOADED_HANDLER.with_borrow_mut(|slot| *slot = Some(handler));
	} else {
		setup_global_events();
	}
}

fn setup_global_events() {
	let run_handler = Closure::new(|event: TransitionEvent| on_transition_run(&event));
	let end_handler = Closure::new(|event: TransitionEvent| on_transition_end(&event));

	let body: web_sys::EventTarget = document().body().expect("the document body").into();

	body.add_event_listener_with_callback(
		"transitionrun",
		run_handler.as_ref().unchecked_ref::<js_sys::Function>(),
	)
	.ok();

	body.add_event_listener_with_callback(
		"transitionend",
		end_handler.as_ref().unchecked_ref::<js_sys::Function>(),
	)
	.ok();

	DOCUMENT_HANDLERS.with_borrow_mut(|handlers| {
		handlers.push(run_handler);
		handlers.push(end_handler);
	});
}

/// Record the element and property for a transition that just started.
fn on_transition_run(event: &TransitionEvent) {
	let Some(element) = transition_element(event) else {
		return;
	};

	let property: JsString = event.property_name().into();

	ELEMENT_TRANSITIONS.with_borrow_mut(|transitions| {
		if let Some(properties) = transitions.get(&element) {
			properties.add(&property);
			return;
		}

		let properties: Set<JsString> = Set::new();
		properties.add(&property);

		// A transitioncancel handler on the element itself cleans up when a
		// transition is cancelled rather than completed.
		let cancel_handler = Closure::new(|event: TransitionEvent| on_transition_end(&event));

		element
			.add_event_listener_with_callback(
				"transitioncancel",
				cancel_handler.as_ref().unchecked_ref::<js_sys::Function>(),
			)
			.ok();

		TRANSITION_CANCEL_HANDLERS.with_borrow_mut(|handlers| {
			handlers.push((element.clone(), cancel_handler));
		});

		transitions.set(&element, &properties);
	});
}

/// Remove the property for a transition that finished or was cancelled, and
/// run the queued callbacks once nothing is transitioning any more.
fn on_transition_end(event: &TransitionEvent) {
	let Some(element) = transition_element(event) else {
		return;
	};

	let property: JsString = event.property_name().into();
	let mut is_empty_after = false;

	ELEMENT_TRANSITIONS.with_borrow_mut(|transitions| {
		let Some(properties) = transitions.get(&element) else {
			return;
		};

		properties.delete(&property);

		if properties.is_empty() {
			remove_cancel_handler(&element);
			transitions.delete(&element);
		}

		is_empty_after = transitions.is_empty();
	});

	if is_empty_after {
		let callbacks = TRANSITION_CALLBACKS.with_borrow_mut(std::mem::take);

		for callback in callbacks {
			callback();
		}
	}
}

fn transition_element(event: &TransitionEvent) -> Option<Element> {
	event
		.target()
		.and_then(|target| target.dyn_into::<Element>().ok())
}

fn remove_cancel_handler(element: &Element) {
	TRANSITION_CANCEL_HANDLERS.with_borrow_mut(|handlers| {
		if let Some(index) = handlers.iter().position(|(target, _)| target == element) {
			let (_, closure) = handlers.remove(index);

			element
				.remove_event_listener_with_callback(
					"transitioncancel",
					closure.as_ref().unchecked_ref::<js_sys::Function>(),
				)
				.ok();
		}
	});
}
