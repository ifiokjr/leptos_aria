use std::cell::Cell;

use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::Element;
use web_sys::FocusOptions;
use web_sys::HtmlElement;
use web_sys::Node;
use web_sys::SvgElement;

fn document() -> web_sys::Document {
	web_sys::window()
		.expect("the `window` global is available in the browser")
		.document()
		.expect("the `document` global is available in the browser")
}

pub trait ToFocusableElement {
	/// Converts the type to a [`FocusableElement`] enum.
	fn to_focusable_element(&self) -> FocusableElement;
}

/// An element that can receive focus: either an `HTMLElement` or an
/// `SVGElement`.
#[derive(Clone)]
pub enum FocusableElement {
	Svg(SvgElement),
	Html(HtmlElement),
}

impl<E> ToFocusableElement for E
where
	E: AsRef<Element>,
{
	fn to_focusable_element(&self) -> FocusableElement {
		FocusableElement::from(self.as_ref().clone())
	}
}

impl From<Element> for FocusableElement {
	fn from(value: Element) -> Self {
		if value.is_instance_of::<SvgElement>() {
			FocusableElement::Svg(value.unchecked_into())
		} else {
			FocusableElement::Html(value.unchecked_into())
		}
	}
}

impl From<&Element> for FocusableElement {
	fn from(value: &Element) -> Self {
		Self::from(value.clone())
	}
}

impl FocusableElement {
	pub fn parent_node(&self) -> Option<Node> {
		match self {
			FocusableElement::Svg(element) => element.parent_node(),
			FocusableElement::Html(element) => element.parent_node(),
		}
	}

	pub fn as_element(&self) -> &Element {
		match self {
			FocusableElement::Svg(element) => element.as_ref(),
			FocusableElement::Html(element) => element.as_ref(),
		}
	}

	pub fn focus(&self) {
		match self {
			FocusableElement::Svg(element) => element.focus().unwrap(),
			FocusableElement::Html(element) => element.focus().unwrap(),
		}
	}

	pub fn focus_with_options(&self, options: &FocusOptions) {
		match self {
			FocusableElement::Svg(element) => element.focus_with_options(options).unwrap(),
			FocusableElement::Html(element) => element.focus_with_options(options).unwrap(),
		}
	}
}

thread_local! {
	static SUPPORTS_PREVENT_SCROLL: Cell<Option<bool>> = const { Cell::new(None) };
}

/// A polyfill for `element.focus({preventScroll: true})`.
///
/// Currently necessary for Safari and old Edge:
/// <https://caniuse.com/#feat=mdn-api_htmlelement_focus_preventscroll_option>,
/// see <https://bugs.webkit.org/show_bug.cgi?id=178583>.
///
/// Original licensing for the following methods can be found in the `license`
/// file in the root directory of this source tree, via the
/// [focus-options-polyfill](https://github.com/calvellido/focus-options-polyfill)
/// project.
pub fn focus_without_scrolling(element: impl AsRef<Element>) {
	let element = element.to_focusable_element();

	if supports_prevent_scroll() {
		let options = FocusOptions::new();
		options.set_prevent_scroll(true);
		element.focus_with_options(&options);
	} else {
		let scrollable_elements = get_scrollable_elements(&element);
		element.focus();
		restore_scroll_position(scrollable_elements);
	}
}

#[wasm_bindgen(inline_js = r#"
  export function leptos_aria_supports_prevent_scroll() {
    let supports = false;
    document.createElement("div").focus({
      get preventScroll() {
        supports = true;
        return true;
      },
    });
    return supports;
  }"#)]
extern "C" {
	/// Check for support of the `preventScroll` option in `focus()`.
	///
	/// Taken from <https://stackoverflow.com/a/59518678>.
	#[wasm_bindgen(catch)]
	fn leptos_aria_supports_prevent_scroll() -> Result<bool, JsValue>;
}

fn supports_prevent_scroll() -> bool {
	let cached = SUPPORTS_PREVENT_SCROLL.get();

	if let Some(supports) = cached {
		supports
	} else {
		let supports = leptos_aria_supports_prevent_scroll().unwrap_or(false);
		SUPPORTS_PREVENT_SCROLL.set(Some(supports));
		supports
	}
}

fn get_scrollable_elements(element: &FocusableElement) -> Vec<ScrollableElement> {
	let mut parent = element.parent_node();
	let mut scrollable_elements: Vec<ScrollableElement> = vec![];
	let root_scrolling_element = document()
		.scrolling_element()
		.unwrap_or_else(|| document().document_element().unwrap());

	while parent
		.as_ref()
		.is_some_and(JsCast::is_instance_of::<HtmlElement>)
	{
		let node = parent.as_ref().unwrap();

		if node.unchecked_ref::<Element>() != &root_scrolling_element {
			let element = node.unchecked_ref::<HtmlElement>();

			if element.offset_height() < element.scroll_height()
				|| element.offset_width() < element.scroll_width()
			{
				scrollable_elements.push(ScrollableElement {
					element: element.clone(),
					scroll_top: element.scroll_top(),
					scroll_left: element.scroll_left(),
				});
			}
		}

		parent = node.parent_node();
	}

	if root_scrolling_element.is_instance_of::<HtmlElement>() {
		let element = root_scrolling_element.unchecked_ref::<HtmlElement>();
		scrollable_elements.push(ScrollableElement {
			element: element.clone(),
			scroll_top: element.scroll_top(),
			scroll_left: element.scroll_left(),
		});
	}

	scrollable_elements
}

struct ScrollableElement {
	element: HtmlElement,
	scroll_top: i32,
	scroll_left: i32,
}

fn restore_scroll_position(scrollable_elements: Vec<ScrollableElement>) {
	for scrollable_element in scrollable_elements {
		scrollable_element
			.element
			.set_scroll_top(scrollable_element.scroll_top);
		scrollable_element
			.element
			.set_scroll_left(scrollable_element.scroll_left);
	}
}
