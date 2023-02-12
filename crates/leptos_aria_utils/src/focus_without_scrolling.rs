// This is a polyfill for element.focus({preventScroll: true});
// Currently necessary for Safari and old Edge:
// https://caniuse.com/#feat=mdn-api_htmlelement_focus_preventscroll_option
// See https://bugs.webkit.org/show_bug.cgi?id=178583
//

// Original licensing for the following methods can be found in the
// NOTICE file in the root directory of this source tree.
// See https://github.com/calvellido/focus-options-polyfill

use leptos::JsCast;
use web_sys::Element;

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
pub fn focus_without_scrolling(element: impl AsRef<Element>) {
  let element = element.to_focusable_element();
  let mut options = FocusOptions::new();

  options.prevent_scroll(true);
  element.focus_with_options(&options);
}
