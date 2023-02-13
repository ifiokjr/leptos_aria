// This is a polyfill for element.focus({preventScroll: true});
// Currently necessary for Safari and old Edge:
// https://caniuse.com/#feat=mdn-api_htmlelement_focus_preventscroll_option
// See https://bugs.webkit.org/show_bug.cgi?id=178583
//

// Original licensing for the following methods can be found in the
// NOTICE file in the root directory of this source tree.
// See https://github.com/calvellido/focus-options-polyfill

use leptos::create_rw_signal;
use leptos::document;
use leptos::wasm_bindgen::prelude::wasm_bindgen;
use leptos::wasm_bindgen::JsValue;
use leptos::web_sys::Element;
use leptos::web_sys::Node;
use leptos::JsCast;
use leptos::RwSignal;
use leptos::Scope;
use leptos::UntrackedGettableSignal;
use leptos::UntrackedSettableSignal;

use crate::ContextProvider;
use crate::FocusOptions;
use crate::HtmlElement;
use crate::SvgElement;

pub trait ToFocusableElement {
  /// Converts the type to a `FocusableElement` enum.
  fn to_focusable_element(&self) -> FocusableElement;
}

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
    if value.is_instance_of::<SvgElement>() {
      FocusableElement::Svg(value.clone().unchecked_into())
    } else {
      FocusableElement::Html(value.clone().unchecked_into())
    }
  }
}

impl FocusableElement {
  pub fn parent_node(&self) -> Option<Node> {
    match self {
      FocusableElement::Svg(element) => element.parent_node(),
      FocusableElement::Html(element) => element.parent_node(),
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

/// This is a polyfill for element.focus({preventScroll: true});
/// Currently necessary for Safari and old Edge:
/// https://caniuse.com/#feat=mdn-api_htmlelement_focus_preventscroll_option
/// See https://bugs.webkit.org/show_bug.cgi?id=178583
///
/// Original licensing for the following methods can be found in the
/// NOTICE file in the root directory of this source tree.
/// See https://github.com/calvellido/focus-options-polyfill
///
/// TODO: check if supported like in `react-aria`
pub fn focus_without_scrolling(cx: Scope, element: impl AsRef<Element>) {
  let element = element.to_focusable_element();

  if supports_prevent_scroll(cx) {
    let mut options = FocusOptions::new();
    options.prevent_scroll(true);
    element.focus_with_options(&options);
  } else {
    let scrollable_elements = get_scrollable_elements(&element);
    element.focus();
    restore_scroll_position(scrollable_elements);
  }
}

#[derive(Copy, Clone)]
pub(crate) struct SupportsPreventScrollContext(RwSignal<Option<bool>>);

impl ContextProvider for SupportsPreventScrollContext {
  type Value = Option<bool>;

  fn from_leptos_scope(cx: Scope) -> Self {
    Self(create_rw_signal(cx, Self::Value::default()))
  }

  fn get(&self) -> Self::Value {
    self.0.get_untracked()
  }

  fn set(&self, value: Self::Value) {
    self.0.set_untracked(value);
  }
}

#[wasm_bindgen(
  inline_js = "export function leptos_aria_supports_prevent_scroll() { let \
               supports=false;document.createElement('div').focus({ get preventScroll(){supports \
               = true; return true;}}); return supports;}"
)]
extern "C" {
  /// Check to see support for `preventScroll` option in `focus()`.
  /// Taken from https://stackoverflow.com/a/59518678
  #[wasm_bindgen(catch)]
  fn leptos_aria_supports_prevent_scroll() -> Result<bool, JsValue>;
}

fn supports_prevent_scroll(cx: Scope) -> bool {
  let supports_prevent_scroll_context = SupportsPreventScrollContext::provide(cx);
  let supports_prevent_scroll = supports_prevent_scroll_context.get();

  match supports_prevent_scroll {
    Some(supports_prevent_scroll) => supports_prevent_scroll,
    None => {
      match leptos_aria_supports_prevent_scroll() {
        Ok(value) => {
          supports_prevent_scroll_context.set(Some(value));
          value
        }
        Err(_) => {
          supports_prevent_scroll_context.set(Some(false));
          false
        }
      }
    }
  }
}

fn get_scrollable_elements(element: &FocusableElement) -> Vec<ScrollableElement> {
  let mut parent = element.parent_node();
  let mut scrollable_elements: Vec<ScrollableElement> = vec![];
  let root_scrolling_element = document()
    .scrolling_element()
    .unwrap_or(document().document_element().unwrap());

  while parent.as_ref().map_or(false, |node| {
    node.is_instance_of::<HtmlElement>()
      && node.unchecked_ref::<Element>() != &root_scrolling_element
  }) {
    if let Some(ref node) = parent {
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

    parent = parent.and_then(|node| node.parent_node());
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
