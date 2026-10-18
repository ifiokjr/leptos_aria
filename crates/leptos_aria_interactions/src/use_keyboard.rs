use std::rc::Rc;

/// A callback invoked for a keyboard event.
pub type BoxedKeyboardEventCallback = Box<dyn Fn(&web_sys::KeyboardEvent)>;

/// The props accepted by [`use_keyboard`].
#[derive(Default)]
pub struct UseKeyboardProps {
	/// Handler that is called when a key is pressed down.
	pub on_key_down: Option<BoxedKeyboardEventCallback>,

	/// Handler that is called when a key is released.
	pub on_key_up: Option<BoxedKeyboardEventCallback>,
}

/// The props returned by [`use_keyboard`] which should be spread onto the
/// target element.
#[derive(Clone)]
pub struct KeyboardResult {
	pub on_key_down: Rc<dyn Fn(&web_sys::KeyboardEvent)>,
	pub on_key_up: Rc<dyn Fn(&web_sys::KeyboardEvent)>,
}

/// Handles keyboard events for the target.
///
/// A thin passthrough matching the upstream hook: consumers composing hooks
/// take keyboard events from here rather than binding their own.
pub fn use_keyboard(props: UseKeyboardProps) -> KeyboardResult {
	let on_key_down = wrap_key_callback(props.on_key_down);
	let on_key_up = wrap_key_callback(props.on_key_up);

	KeyboardResult {
		on_key_down: Rc::new(move |event| call_key_event(&on_key_down, event)),
		on_key_up: Rc::new(move |event| call_key_event(&on_key_up, event)),
	}
}

type KeyboardCallback = Rc<dyn Fn(&web_sys::KeyboardEvent)>;

#[allow(clippy::type_complexity)]
fn wrap_key_callback(callback: Option<BoxedKeyboardEventCallback>) -> KeyboardCallback {
	Rc::new(move |event: &web_sys::KeyboardEvent| {
		if let Some(callback) = &callback {
			callback(event);
		}
	})
}

fn call_key_event(callback: &KeyboardCallback, event: &web_sys::KeyboardEvent) {
	callback(event);
}
