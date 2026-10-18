use std::cell::Cell;

use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;

/// How the user is currently interacting with the page.
///
/// Focus rings are only shown for keyboard users, and screen reader users
/// (a "virtual" modality) need focus moved after transitions rather than
/// during them. The modality is detected from global input events, exactly
/// like the upstream implementation.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum InteractionModality {
	#[default]
	/// No interaction has been recorded yet.
	None,
	/// Keyboard navigation: show focus rings.
	Keyboard,
	/// Mouse, touch, or pen interaction: hide focus rings.
	Pointer,
	/// Screen reader or other assistive technology focus.
	Virtual,
}

thread_local! {
	static MODALITY: Cell<InteractionModality> = const { Cell::new(InteractionModality::None) };
	static LISTENERS_SETUP: Cell<bool> = const { Cell::new(false) };
}

/// The current interaction modality, as detected from global input events.
pub fn interaction_modality() -> InteractionModality {
	setup_listeners();
	MODALITY.get()
}

/// Whether focus should be visible for the current interaction modality.
///
/// Keyboard users get a focus ring; pointer users do not. A fresh page with
/// no recorded modality (a screen reader moving focus directly) counts as
/// keyboard-like so the ring is not hidden from assistive technology.
pub fn is_focus_visible() -> bool {
	matches!(
		interaction_modality(),
		InteractionModality::Keyboard | InteractionModality::Virtual
	)
}

fn set_modality(modality: InteractionModality) {
	MODALITY.set(modality);
}

/// Install the global listeners which detect the interaction modality.
///
/// The listeners live for the page and are shared by every consumer, so they
/// are attached once on first use.
fn setup_listeners() {
	if LISTENERS_SETUP.get() {
		return;
	}

	LISTENERS_SETUP.set(true);

	let window = window();

	// A plain key press with no modifiers held signals keyboard navigation.
	let key_handler =
		Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(|event: web_sys::KeyboardEvent| {
			let plain_key = !event.ctrl_key() && !event.meta_key() && !event.alt_key();
			let is_tab = event.key() == "Tab";
			let is_character = event.key().chars().count() == 1;

			if plain_key && (is_tab || is_character) {
				set_modality(InteractionModality::Keyboard);
			}
		});

	window
		.add_event_listener_with_callback_and_bool(
			"keydown",
			key_handler.as_ref().unchecked_ref::<js_sys::Function>(),
			true,
		)
		.ok();

	let pointer_handler =
		Closure::<dyn FnMut(web_sys::PointerEvent)>::new(|_: web_sys::PointerEvent| {
			set_modality(InteractionModality::Pointer);
		});

	window
		.add_event_listener_with_callback(
			"pointerdown",
			pointer_handler.as_ref().unchecked_ref::<js_sys::Function>(),
		)
		.ok();

	// Focus arriving with no prior modality means an assistive technology (or
	// autofocus) moved it, which behaves like a virtual cursor.
	let focus_handler = Closure::<dyn FnMut(web_sys::Event)>::new(|_: web_sys::Event| {
		if MODALITY.get() == InteractionModality::None {
			set_modality(InteractionModality::Virtual);
		}
	});

	window
		.add_event_listener_with_callback_and_bool(
			"focusin",
			focus_handler.as_ref().unchecked_ref::<js_sys::Function>(),
			true,
		)
		.ok();

	// The closures must outlive this call: they back listeners which are never
	// removed, so a deliberate leak of the three wrappers is the page-lifetime
	// ownership model.
	std::mem::forget(key_handler);
	std::mem::forget(pointer_handler);
	std::mem::forget(focus_handler);
}

fn window() -> web_sys::Window {
	web_sys::window().expect("the `window` global is available in the browser")
}
