use leptos::js_sys::Reflect;
use leptos::web_sys::MouseEvent;
use leptos::JsCast;
use web_sys::PointerEvent;

use crate::is_android;

/// Keyboards, Assistive Technologies, and element.click() all produce a
/// "virtual" click event. This is a method of inferring such clicks. Every
/// browser except IE 11 only sets a zero value of "detail" for click events
/// that are "virtual". However, IE 11 uses a zero value for all click events.
/// For IE 11 we rely on the quirk that it produces click events that are of
/// type PointerEvent, and where only the "virtual" click lacks a pointerType
/// field.
pub fn is_virtual_click(event: impl AsRef<MouseEvent>) -> bool {
  let event = event.as_ref();

  let mozilla_input_source = Reflect::get(event, &"mozInputSource".into())
    .ok()
    .and_then(|s| s.as_f64());

  // JAWS/NVDA with Firefox.
  if mozilla_input_source == Some(0f64) && event.is_trusted() {
    return true;
  }

  // Android TalkBack's detail value varies depending on the event listener
  // providing the event so we have specific logic here instead If pointerType
  // is defined, event is from a click listener. For events from mousedown
  // listener, detail === 0 is a sufficient check to detect TalkBack virtual
  // clicks.
  if is_android() && event.is_instance_of::<PointerEvent>() {
    return event.type_() == "click" && event.buttons() == 1;
  }

  event.detail() == 0 && !event.is_instance_of::<PointerEvent>()
}

pub fn is_virtual_pointer_event(event: impl AsRef<PointerEvent>) -> bool {
  let event = event.as_ref();
  // If the pointer size is zero, then we assume it's from a screen reader.
  // Android TalkBack double tap will sometimes return a event with width and
  // height of 1 and pointerType === 'mouse' so we need to check for a
  // specific combination of event attributes. Cannot use "event.pressure ===
  // 0" as the sole check due to Safari pointer events always returning pressure
  // === 0 instead of .5, see https://bugs.webkit.org/show_bug.cgi?id=206216. event.pointerType === 'mouse' is to distingush
  // Talkback double tap from Windows Firefox touch screen press
  (event.width() == 0 && event.height() == 0)
    || (event.width() == 1
      && event.height() == 1
      && event.pressure() == 0f32
      && event.detail() == 0
      && event.pointer_type() == "mouse")
}
