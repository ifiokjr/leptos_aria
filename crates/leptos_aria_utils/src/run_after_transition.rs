use std::cell::RefCell;
use std::collections::HashSet;
use std::ptr::eq;
use std::rc::Rc;

use leptos::create_rw_signal;
use leptos::document;
use leptos::js_sys::Function;
use leptos::js_sys::JsString;
use leptos::request_animation_frame;
use leptos::wasm_bindgen::prelude::Closure;
use leptos::wasm_bindgen::JsValue;
use leptos::web_sys::Element;
use leptos::web_sys::Event;
use leptos::web_sys::TransitionEvent;
use leptos::JsCast;
use leptos::RwSignal;
use leptos::Scope;
use leptos::UntrackedGettableSignal;
use leptos::UntrackedSettableSignal;

use crate::silly_map::Map;
use crate::silly_map::Set;
use crate::ContextProvider;

type T = Map<Element, Set<String>>;

/// We store a global list of elements that are currently transitioning,
/// mapped to a set of CSS properties that are transitioning for that element.
/// This is necessary rather than a simple count of transitions because of
/// browser bugs, e.g. Chrome sometimes fires both transitionend and
/// transitioncancel rather than one or the other. So we need to track what's
/// actually transitioning so that we can ignore these duplicate events.
#[derive(Copy, Clone)]
pub(crate) struct ElementTransitionsContext(RwSignal<Map<Element, Set<JsString>>>);

impl ContextProvider for ElementTransitionsContext {
  type Value = Map<Element, Set<JsString>>;

  fn from_leptos_scope(cx: Scope) -> Self {
    Self(create_rw_signal(cx, Default::default()))
  }

  fn get(&self) -> Self::Value {
    self.0.get_untracked()
  }

  fn set(&self, value: Self::Value) {
    let reference = &value;
    self.0.set_untracked(if eq(reference, &self.get()) {
      // this happens when the value was directly mutated.
      reference.clone()
    } else {
      value
    });
  }
}

type TransitionCallback = Rc<Box<dyn Fn()>>;

/// A list of callbacks to call once there are no transitioning elements.
#[derive(Copy, Clone)]
pub(crate) struct TransitionCallbacksContext(RwSignal<Vec<TransitionCallback>>);

impl ContextProvider for TransitionCallbacksContext {
  type Value = Vec<TransitionCallback>;

  fn from_leptos_scope(cx: Scope) -> Self {
    Self(create_rw_signal(cx, Default::default()))
  }

  fn get(&self) -> Self::Value {
    self.0.get_untracked()
  }

  fn set(&self, value: Self::Value) {
    let reference = &value;
    self.0.set_untracked(if eq(reference, &self.get()) {
      // this happens when the value was directly mutated.
      reference.clone()
    } else {
      value
    });
  }
}

struct TransitionCallbacks {
  cx: Scope,
  closures: Option<Rc<Closures>>,
}

struct Closures {
  // on_start: Closure<dyn Fn(TransitionEvent)>,
  on_end: Closure<dyn Fn(TransitionEvent)>,
}

fn setup_global_events(cx: Scope) {
  let closure: Rc<RefCell<Closure<dyn Fn(TransitionEvent)>>> =
    Rc::new(RefCell::new(Closure::new(|_| {})));
  let update_closure = closure.clone();
  let other_closure = closure.clone();
  let on_end_closure = closure.clone();

  let on_transition_end = move |event: TransitionEvent| {
    let element: Element = event.target().unwrap().unchecked_into();
    let transitions_context = ElementTransitionsContext::provide(cx);
    let js_map = transitions_context.get();

    // Remove property from list of transitioning properties.
    let Some(properties) = js_map.get(&element) else {
      return;
    };

    properties.delete(&event.property_name().into());

    // If empty, remove transitioncancel event, and remove the element from the list
    // of transitioning elements.
    if properties.is_empty() {
      element
        .remove_event_listener_with_callback(
          "transitioncancel",
          other_closure.borrow().as_ref().unchecked_ref(),
        )
        .ok();

      js_map.delete(&element);
    }

    if js_map.is_empty() {
      let callbacks_context = TransitionCallbacksContext::provide(cx);

      for callback in callbacks_context.get().iter() {
        callback.clone()();
      }

      callbacks_context.set(Vec::new());
    }
  };

  let on_transition_start = move |event: TransitionEvent| {
    let element: Element = event.target().unwrap().unchecked_into();
    let transitions_context = ElementTransitionsContext::provide(cx);
    let js_map = transitions_context.get();

    match js_map.get(&element) {
      Some(set) => {
        set.add(&event.property_name().into());
      }
      None => {
        let set: Set<JsString> = Default::default();
        set.add(&event.property_name().into());

        element
          .add_event_listener_with_callback(
            "transitioncancel",
            closure.borrow().as_ref().unchecked_ref(),
          )
          .ok();

        js_map.set(&element, &set);
      }
    }
  };

  let document_transition_end: Closure<dyn Fn(TransitionEvent)> = Closure::new(on_transition_end);
  let cloned = document_transition_end.as_ref().clone();
  update_closure.replace(document_transition_end);

  let on_start_closure =
    Closure::wrap(Box::new(on_transition_start) as Box<dyn Fn(TransitionEvent)>);
  document()
    .body()
    .unwrap()
    .add_event_listener_with_callback("transitionrun", on_start_closure.as_ref().unchecked_ref())
    .ok();

  document()
    .body()
    .unwrap()
    .add_event_listener_with_callback("transitionend", cloned.unchecked_ref())
    .ok();
}

/// Setup a listener for transition events on the page.
///
/// This should only be run in the browser.
pub(crate) fn setup_transition_listener(cx: Scope) {
  if document().ready_state() != "loading" {
    setup_global_events(cx);
  } else {
    let callback = move |_: Event| setup_global_events(cx);
    let closure = Closure::wrap(Box::new(callback) as Box<dyn Fn(Event)>);

    document()
      .add_event_listener_with_callback("DOMContentLoaded", closure.as_ref().unchecked_ref())
      .ok();
  }
}

/// Perform a certain action after all CSS transitions have finished on the
/// page.
///
/// This prevents
pub fn run_after_transition<F>(cx: Scope, callback: F)
where
  F: Fn() + 'static,
{
  let cb = move || {
    let transitions_context = ElementTransitionsContext::provide(cx);
    let transitions = transitions_context.get();

    if transitions.is_empty() {
      // When no transitions are running, call the function immediately.
      // Otherwise, add it to a list of callbacks to run at the end of the animation.
      callback();
    } else {
      let callbacks_context = TransitionCallbacksContext::provide(cx);
      let mut callbacks = callbacks_context.get();
      let callback = Rc::new(Box::new(callback) as Box<dyn Fn() + 'static>);
      callbacks.push(callback);
    }
  };

  // Wait one frame to see if an animation starts, e.g. a transition on mount.
  request_animation_frame(cb);
}
