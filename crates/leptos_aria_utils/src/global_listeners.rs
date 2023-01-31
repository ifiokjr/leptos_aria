use leptos::js_sys;
use leptos::wasm_bindgen;
use leptos::web_sys;
use leptos::JsCast;

#[derive(Default)]
pub struct GlobalListeners(Vec<(web_sys::EventTarget, String, js_sys::Function, bool)>);

impl GlobalListeners {
  /// Add a closure as an event listener.
  pub fn add_listener(
    &mut self,
    target: impl AsRef<web_sys::EventTarget>,
    type_: impl Into<String>,
    closure: wasm_bindgen::closure::Closure<dyn Fn(web_sys::Event)>,
    capture: bool,
  ) {
    let event_target = target.as_ref().clone();
    let event_type = type_.into();
    let function = closure.as_ref().unchecked_ref::<js_sys::Function>().clone();
    event_target
      .add_event_listener_with_callback_and_bool(event_type.as_str(), &function, capture)
      .unwrap();

    self.0.push((event_target, event_type, function, capture));
  }

  /// Remove all the generated listeners.
  pub fn remove_all_listeners(&mut self) {
    for (event_target, event_type, function, capture) in self.0.iter() {
      event_target
        .remove_event_listener_with_callback_and_bool(event_type.as_str(), function, *capture)
        .unwrap();
    }

    self.0.clear();
  }
}
