use leptos::js_sys::Boolean;
use leptos::js_sys::Function;
use leptos::js_sys::JsString;
use leptos::web_sys::EventTarget;
use slotmap::DefaultKey;
// use slotmap::Key;
// use slotmap::SecondaryMap;
use slotmap::SlotMap;

use crate::Map;
use crate::Tuple3;

// type GlobalListenersMap = Map<Function, Tuple3<EventTarget, JsString,
// Boolean>>;

#[derive(Default)]
pub struct GlobalListeners(SlotMap<DefaultKey, (Function, EventTarget, String, bool)>);

impl GlobalListeners {
  /// Add a closure as an event listener.
  pub fn add_listener(
    &mut self,
    target: impl AsRef<EventTarget>,
    type_: impl Into<String>,
    function: Function,
    capture: bool,
  ) -> DefaultKey {
    let event_target = target.as_ref().clone();
    let event_type: String = type_.into();
    // let function = closure.as_ref().unchecked_ref::<Function>();
    event_target
      .add_event_listener_with_callback_and_bool(event_type.as_str(), &function, capture)
      .unwrap();

    self.0.insert((function, event_target, event_type, capture))
  }

  pub fn remove_listener(&mut self, key: DefaultKey) {
    if let Some((function, event_target, event_type, capture)) = self.0.get(key) {
      event_target
        .remove_event_listener_with_callback_and_bool(event_type.as_str(), function, *capture)
        .unwrap();
    };

    self.0.remove(key);
  }

  /// Remove all the generated listeners.
  pub fn remove_all_listeners(&mut self) {
    self
      .0
      .values()
      .for_each(|(function, event_target, event_type, capture)| {
        event_target
          .remove_event_listener_with_callback_and_bool(event_type.as_str(), function, *capture)
          .unwrap();
      });

    self.0.clear();
  }
}

impl Drop for GlobalListeners {
  fn drop(&mut self) {
    self.remove_all_listeners();
  }
}
