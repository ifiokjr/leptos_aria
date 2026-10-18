use wasm_bindgen::JsCast;
use web_sys::Element;
use web_sys::HtmlElement;

/// The selectors matching natively focusable elements.
const FOCUSABLE_SELECTORS: &[&str] = &[
	"input:not([disabled]):not([type=hidden])",
	"select:not([disabled])",
	"textarea:not([disabled])",
	"button:not([disabled])",
	"a[href]",
	"area[href]",
	"summary",
	"iframe",
	"object",
	"embed",
	"audio[controls]",
	"video[controls]",
	"[contenteditable]:not([contenteditable^=\"false\"])",
	"permission",
];

/// Whether an element can receive keyboard focus.
///
/// The element must match a focusable selector, sit inside no inert subtree,
/// and be visible — unless the visibility check is skipped.
pub fn is_focusable(element: &Element, skip_visibility_check: bool) -> bool {
	matches_focusable_selectors(element)
		&& !is_inert(element)
		&& (skip_visibility_check || is_element_visible(element))
}

/// Whether an element can be reached with sequential Tab navigation.
pub fn is_tabbable(element: &Element) -> bool {
	let has_negative_tabindex = element
		.get_attribute("tabindex")
		.as_deref()
		.is_some_and(|tabindex| tabindex == "-1");

	matches_focusable_selectors(element)
		&& !has_negative_tabindex
		&& !is_inert(element)
		&& is_element_visible(element)
}

fn matches_focusable_selectors(element: &Element) -> bool {
	let base_match = FOCUSABLE_SELECTORS
		.iter()
		.any(|selector| matches_visible(element, selector))
		|| (element.has_attribute("tabindex") && !element_has_attribute(element, "disabled"));

	let hidden = element_has_attribute(element, "hidden");

	base_match && !hidden
}

fn matches_visible(element: &Element, selector: &str) -> bool {
	element
		.matches(&format!("{selector}:not([hidden])"))
		.unwrap_or(false)
}

fn element_has_attribute(element: &Element, name: &str) -> bool {
	element.has_attribute(name)
}

fn is_inert(element: &Element) -> bool {
	let mut node = Some(element.clone());

	while let Some(element) = node {
		if let Some(html_element) = element.dyn_ref::<HtmlElement>() {
			if html_element.inert() {
				return true;
			}
		}

		node = element.parent_element();
	}

	false
}

/// Visibility by layout: an element is visible when it produces client
/// rectangles, or when a child sticks out of a clipped `overflow: hidden`
/// container so the element still occupies space.
fn is_element_visible(element: &Element) -> bool {
	if element.get_bounding_client_rect().width() > 0.0
		|| element.get_bounding_client_rect().height() > 0.0
	{
		return true;
	}

	element.first_element_child().is_some_and(|child| {
		let child_rect = child.get_bounding_client_rect();

		child_rect.width() > 0.0 || child_rect.height() > 0.0
	})
}
