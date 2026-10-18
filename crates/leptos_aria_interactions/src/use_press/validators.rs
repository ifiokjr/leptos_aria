use wasm_bindgen::JsCast;
use web_sys::Element;
use web_sys::HtmlAnchorElement;
use web_sys::HtmlButtonElement;
use web_sys::HtmlElement;
use web_sys::HtmlInputElement;
use web_sys::HtmlTextAreaElement;
use web_sys::KeyboardEvent;
use web_sys::Node;

/// We cannot prevent default if the target is not an `HTMLElement` or if it is
/// a draggable `HTMLElement`.
pub(crate) fn should_prevent_default(target: impl AsRef<Element>) -> bool {
	let element = target.as_ref();

	!element.is_instance_of::<HtmlElement>() || !element.unchecked_ref::<HtmlElement>().draggable()
}

pub(crate) fn should_prevent_default_keyboard(target: impl AsRef<Element>, key: &str) -> bool {
	let element = target.as_ref();

	if element.is_instance_of::<HtmlInputElement>() {
		!is_valid_input_key(element.unchecked_ref(), key)
	} else if element.is_instance_of::<HtmlButtonElement>() {
		element.unchecked_ref::<HtmlButtonElement>().type_() == "submit"
	} else {
		true
	}
}

pub(crate) fn is_valid_keyboard_event(event: &KeyboardEvent, current_target: &Element) -> bool {
	let key = event.key();
	let code = event.code();
	let element: &HtmlElement = current_target.unchecked_ref();
	let role = element.get_attribute("role");

	// Accessibility for keyboards. Space and Enter only.
	is_activating_key(&key, &code)
		&& accepts_keyboard_activation(element, &key)
		// A link with a valid href should be handled natively, unless it also
		// has `role="button"` and was triggered using Space.
		&& handles_link_activation(element, role.as_deref(), &key)
}

fn is_activating_key(key: &str, code: &str) -> bool {
	key == "Enter" || key == " " || code == "Space"
}

/// Text-editing elements and inputs that type text must keep their native key
/// handling.
fn accepts_keyboard_activation(element: &HtmlElement, key: &str) -> bool {
	let is_text_input = element.is_instance_of::<HtmlInputElement>()
		&& !is_valid_input_key(element.unchecked_ref(), key);

	!is_text_input
		&& !element.is_instance_of::<HtmlTextAreaElement>()
		&& !element.is_content_editable()
}

fn handles_link_activation(element: &HtmlElement, role: Option<&str>, key: &str) -> bool {
	if is_html_anchor_link(element) {
		// A `role="button"` link still activates on Space; Enter is left to
		// the browser.
		role == Some("button") && key != "Enter"
	} else {
		role != Some("link") || key != "Enter"
	}
}

pub(crate) fn is_html_anchor_link(target: impl AsRef<Element>) -> bool {
	let element = target.as_ref();

	element.is_instance_of::<HtmlAnchorElement>()
		|| (element.tag_name() == "A" && element.has_attribute("href"))
}

pub(crate) fn has_link_role(target: impl AsRef<Element>) -> bool {
	let element = target.as_ref();

	is_html_anchor_link(element) || element.get_attribute("role").as_deref() == Some("link")
}

fn is_valid_input_key(target: &HtmlInputElement, key: &str) -> bool {
	// Only space should toggle checkboxes and radios, not enter.
	if target.type_() == "checkbox" || target.type_() == "radio" {
		key == " "
	} else {
		NON_TEXT_INPUT_TYPES.contains(&target.type_().as_str())
	}
}

const NON_TEXT_INPUT_TYPES: &[&str; 9] = &[
	"checkbox", "radio", "range", "color", "file", "image", "button", "submit", "reset",
];

pub(crate) fn current_target_element(event: &impl AsRef<web_sys::Event>) -> Option<Element> {
	event
		.as_ref()
		.current_target()
		.and_then(|target| target.dyn_into::<Element>().ok())
}

pub(crate) fn event_target_node(event: &impl AsRef<web_sys::Event>) -> Option<Node> {
	event
		.as_ref()
		.target()
		.and_then(|target| target.dyn_into::<Node>().ok())
}

pub(crate) fn event_target_element(event: &impl AsRef<web_sys::Event>) -> Option<Element> {
	event
		.as_ref()
		.target()
		.and_then(|target| target.dyn_into::<Element>().ok())
}
