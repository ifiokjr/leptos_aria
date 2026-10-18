use wasm_bindgen::JsCast;
use web_sys::MouseEvent;
use web_sys::PointerEvent;

use crate::is_android;

/// Keyboards, Assistive Technologies, and `element.click()` all produce a
/// "virtual" click event. This is a method of inferring such clicks: every
/// browser except IE 11 only sets a zero value of "detail" for click events
/// that are "virtual", and only a genuine pointer-driven click carries a
/// `pointerType`.
pub fn is_virtual_click(event: impl AsRef<MouseEvent>) -> bool {
	let event = event.as_ref();
	let pointer_event = event.dyn_ref::<PointerEvent>();

	// JAWS/NVDA with Firefox.
	if event.is_trusted() && pointer_event.is_none_or(|event| event.pointer_type().is_empty()) {
		return true;
	}

	// Android TalkBack's detail value varies depending on the event listener
	// providing the event, so there is specific logic here instead. If
	// `pointerType` is set the event is from a click listener.
	if is_android() && pointer_event.is_some_and(|event| !event.pointer_type().is_empty()) {
		return event.type_() == "click" && event.buttons() == 1;
	}

	let has_pointer_type = pointer_event.is_some_and(|event| !event.pointer_type().is_empty());

	event.detail() == 0 && !has_pointer_type
}

pub fn is_virtual_pointer_event(event: impl AsRef<PointerEvent>) -> bool {
	let event = event.as_ref();

	// If the pointer size is zero, then it is assumed to be from a screen
	// reader. Android TalkBack double tap will sometimes return an event with
	// width and height of 1 and `pointerType == "mouse"`, so it needs the more
	// specific combination. `pressure` cannot be used as the sole check due to
	// Safari pointer events always returning `pressure == 0` instead of `.5`;
	// see <https://bugs.webkit.org/show_bug.cgi?id=206216>.
	(!is_android() && event.width() == 0 && event.height() == 0)
		|| (is_android()
			&& event.width() == 1
			&& event.height() == 1
			&& event.pressure() == 0_f32
			&& event.detail() == 0
			&& event.pointer_type() == "mouse")
}
