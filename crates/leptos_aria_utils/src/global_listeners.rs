use js_sys::Function;
use slotmap::DefaultKey;
use slotmap::SlotMap;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::Event;
use web_sys::EventTarget;

/// A set of event listeners registered on targets outside the rendered tree,
/// alive for the lifetime of an interaction.
///
/// Each [`Closure`] is stored alongside its registration so the Rust callback
/// stays alive for exactly as long as JavaScript can invoke it; dropping the
/// set removes every listener it added.
#[derive(Default)]
pub struct GlobalListeners(SlotMap<DefaultKey, Listener>);

struct Listener {
	closure: Closure<dyn FnMut(Event)>,
	target: EventTarget,
	event_type: String,
	capture: bool,
}

impl GlobalListeners {
	/// Register `handler` as an `event_type` listener on `target`.
	pub fn add<E, F>(
		&mut self,
		target: impl AsRef<EventTarget>,
		event_type: impl Into<String>,
		handler: F,
		capture: bool,
	) -> DefaultKey
	where
		E: JsCast,
		F: Fn(E) + 'static,
	{
		let closure = Closure::new(move |event: Event| handler(event.unchecked_into::<E>()));
		let event_type = event_type.into();
		let target = target.as_ref().clone();

		target
			.add_event_listener_with_callback_and_bool(
				event_type.as_str(),
				closure.as_ref().unchecked_ref::<Function>(),
				capture,
			)
			.unwrap();

		self.0.insert(Listener {
			closure,
			target,
			event_type,
			capture,
		})
	}

	pub fn remove_listener(&mut self, key: DefaultKey) {
		if let Some(listener) = self.0.remove(key) {
			detach(&listener);
		}
	}

	/// Remove every listener registered through this set.
	pub fn remove_all_listeners(&mut self) {
		for (_, listener) in self.0.drain() {
			detach(&listener);
		}
	}
}

fn detach(listener: &Listener) {
	listener
		.target
		.remove_event_listener_with_callback_and_bool(
			listener.event_type.as_str(),
			listener.closure.as_ref().unchecked_ref::<Function>(),
			listener.capture,
		)
		.unwrap();
}

impl Drop for GlobalListeners {
	fn drop(&mut self) {
		self.remove_all_listeners();
	}
}
