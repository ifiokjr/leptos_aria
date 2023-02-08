use leptos::js_sys::Boolean;
use leptos::js_sys::Function;
use leptos::js_sys::JsString;
use leptos::web_sys::EventTarget;

use crate::Map;
use crate::Tuple3;

type GlobalListenersMap = Map<Function, Tuple3<EventTarget, JsString, Boolean>>;

#[derive(Default)]
pub struct GlobalListeners(GlobalListenersMap);

impl GlobalListeners {
  /// Add a closure as an event listener.
  pub fn add_listener(
    &self,
    target: impl AsRef<EventTarget>,
    type_: impl Into<String>,
    function: Function,
    capture: bool,
  ) {
    let event_target = target.as_ref().clone();
    let event_type: String = type_.into();
    // let function = closure.as_ref().unchecked_ref::<Function>();
    event_target
      .add_event_listener_with_callback_and_bool(event_type.as_str(), &function, capture)
      .unwrap();

    let tuple = Tuple3::new(event_target, event_type.into(), capture.into());
    self.0.set(&function, &tuple);
  }

  /// Remove all the generated listeners.
  pub fn remove_all_listeners(&self) {
    self.0.for_each(
      &mut |function: Function, tuple: Tuple3<EventTarget, JsString, Boolean>| {
        let (event_target, event_type, capture) = tuple.get();
        event_target
          .remove_event_listener_with_callback_and_bool(
            event_type.as_string().unwrap().as_str(),
            &function,
            capture.as_bool().unwrap(),
          )
          .unwrap();
      },
    );
    // for (event_target, event_type, closure, capture) in self.0.iter() {
    //   event_target
    //     .remove_event_listener_with_callback_and_bool(
    //       event_type.as_str(),
    //       closure.as_ref().unchecked_ref(),
    //       *capture,
    //     )
    //     .unwrap();
    // }

    self.0.clear();
  }
}
