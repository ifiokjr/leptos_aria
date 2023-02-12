//! Extend the `web_sys` crate with methods that are not yet available in the
//! `web_sys` crate.

use leptos::js_sys::Object;
use leptos::js_sys::Reflect;
use leptos::JsCast;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;
use web_sys::EventTarget;
use web_sys::Node;

#[wasm_bindgen]
extern "C" {
  #[wasm_bindgen(extends = web_sys::HtmlElement, extends = web_sys::Element, extends = Node, extends = EventTarget, extends = Object, js_name = HTMLElement, typescript_type = "HTMLElement")]
  #[derive(Debug, Clone, PartialEq, Eq)]
  #[doc = "The `HtmlElement` class."]
  #[doc = ""]
  #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement)"]
  #[doc = ""]
  #[doc = "*This API requires the following crate features to be activated: `HtmlElement`*"]
  pub type HtmlElement;

  #[wasm_bindgen(catch, method, structural, js_class = "HTMLElement", js_name = focus)]
  #[doc = "The `focus()` method."]
  pub fn focus_with_options(this: &HtmlElement, options: &FocusOptions) -> Result<(), JsValue>;
}

#[wasm_bindgen]
extern "C" {
  #[wasm_bindgen(extends = web_sys::SvgElement, extends = web_sys::Element, extends = Node, extends = EventTarget, extends = Object, js_name = SVGElement, typescript_type = "SVGElement")]
  #[derive(Debug, Clone, PartialEq, Eq)]
  #[doc = "The `SvgElement` class."]
  #[doc = ""]
  #[doc = "[MDN Documentation](https://developer.mozilla.org/en-US/docs/Web/API/SVGElement)"]
  #[doc = ""]
  #[doc = "*This API requires the following crate features to be activated: `SvgElement`*"]
  pub type SvgElement;

  #[wasm_bindgen(catch, method, structural, js_class = "SVGElement", js_name = focus)]
  #[doc = "The `focus()` method."]
  pub fn focus_with_options(this: &SvgElement, options: &FocusOptions) -> Result<(), JsValue>;
}

#[wasm_bindgen]
extern "C" {
  # [wasm_bindgen (extends = Object , js_name = FocusOptions)]
  #[derive(Debug, Clone, PartialEq, Eq)]
  #[doc = "The `FocusOptions` dictionary."]
  #[doc = ""]
  #[doc = "*This API requires the following crate features to be activated: `FocusOptions`*"]
  pub type FocusOptions;
}

impl FocusOptions {
  pub fn new() -> Self {
    #[allow(unused_mut)]
    let mut ret: Self = JsCast::unchecked_into(Object::new());
    ret
  }

  /// A boolean value indicating whether or not the browser should scroll the
  /// document to bring the newly-focused element into view. A value of false
  /// for preventScroll (the default) means that the browser will scroll the
  /// element into view after focusing it. If preventScroll is set to true, no
  /// scrolling will occur.
  pub fn prevent_scroll(&mut self, val: bool) -> &mut Self {
    let result = Reflect::set(
      self.as_ref(),
      &JsValue::from("preventScroll"),
      &JsValue::from(val),
    );
    debug_assert!(
      result.is_ok(),
      "setting properties should never fail on our dictionary objects"
    );
    let _ = result;
    self
  }

  // A boolean value that should be set to true to force visible indication
  /// that the element is focused. By default, or if the property is not true, a
  /// browser may still provide visible indication if it determines that this
  /// would improve accessibility for users.
  pub fn focus_visible(&mut self, val: bool) -> &mut Self {
    let result = Reflect::set(
      self.as_ref(),
      &JsValue::from("focusVisible"),
      &JsValue::from(val),
    );
    debug_assert!(
      result.is_ok(),
      "setting properties should never fail on our dictionary objects"
    );
    let _ = result;
    self
  }
}

impl Default for FocusOptions {
  fn default() -> Self {
    Self::new()
  }
}
