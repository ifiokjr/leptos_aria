use std::rc::Rc;

use leptos::prelude::*;
use leptos_use::use_event_listener;

/// The event passed to the scroll handler.
#[derive(Clone, Copy, Debug)]
pub struct ScrollEvent {
	/// The horizontal scroll delta.
	pub delta_x: f64,

	/// The vertical scroll delta.
	pub delta_y: f64,
}

/// A callback invoked for a [`ScrollEvent`].
pub type BoxedScrollCallback = Box<dyn Fn(&ScrollEvent)>;

/// The props accepted by [`use_scroll`].
pub struct UseScrollProps {
	/// Handler that is called for every wheel scroll over the target.
	pub on_scroll: BoxedScrollCallback,

	/// Whether the wheel handling is disabled.
	pub is_disabled: bool,
}

/// A cancelable wheel listener on a scrollable target.
///
/// The wheel event must be registered as non-passive for `prevent_default` to
/// stop the page scrolling under the surface — this is the reason the hook
/// exists rather than a plain `on:wheel` binding. Zoom gestures (`ctrlKey`)
/// are left to the browser.
pub fn use_scroll(target: NodeRef<leptos::html::Div>, props: UseScrollProps) {
	let on_scroll = Rc::new(props.on_scroll);
	let is_disabled = props.is_disabled;

	if is_disabled {
		return;
	}

	let handler = move |event: web_sys::WheelEvent| {
		// A ctrl-modified wheel is a zoom gesture, not a scroll.
		if event.ctrl_key() {
			return;
		}

		event.prevent_default();
		event.stop_propagation();

		on_scroll(&ScrollEvent {
			delta_x: event.delta_x(),
			delta_y: event.delta_y(),
		});
	};

	// `use_event_listener` attaches the listener and detaches again when the
	// target signal changes or the owner is disposed.
	let target_signal = Signal::derive(move || target.get());

	let _ = use_event_listener(target_signal, leptos::ev::wheel, handler);
}
