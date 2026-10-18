use web_sys::Element;

use crate::FocusableElement;
use crate::ToFocusableElement;
use crate::focus_without_scrolling;
use crate::interaction_modality::InteractionModality;
use crate::interaction_modality::interaction_modality;
use crate::run_after_transition;

/// Focus an element while avoiding undesired side effects such as page
/// scrolling and screen reader issues with CSS transitions.
///
/// When the user is interacting with a virtual cursor, e.g. a screen reader,
/// focus waits until any animated transitions currently occurring on the page
/// have finished before shifting. This avoids `VoiceOver` on iOS scrolling the
/// page when moving focus to an element transitioning in from off screen.
pub fn focus_safely(element: impl AsRef<Element>) {
	let element = element.as_ref();

	if !is_connected(element) {
		return;
	}

	if interaction_modality() == InteractionModality::Virtual {
		let target: FocusableElement = element.to_focusable_element();
		let last_focused_element = active_element();

		run_after_transition(move || {
			let active = active_element();
			let focus_was_unchanged = active == last_focused_element || is_body(active.as_ref());

			// If focus did not move or focus was lost to the body, and the
			// element is still in the document, focus it.
			if focus_was_unchanged && is_connected(target.as_element()) {
				focus_without_scrolling(target.as_element());
			}
		});
	} else {
		focus_without_scrolling(element);
	}
}

fn is_connected(element: &Element) -> bool {
	element.is_connected()
}

fn active_element() -> Option<Element> {
	document().active_element()
}

fn is_body(element: Option<&Element>) -> bool {
	element.is_some_and(|element| element.tag_name() == "BODY")
}

fn document() -> web_sys::Document {
	web_sys::window()
		.expect("the `window` global is available in the browser")
		.document()
		.expect("the `document` global is available in the browser")
}
