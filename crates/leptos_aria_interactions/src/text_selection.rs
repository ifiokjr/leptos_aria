use std::ptr::eq;
use std::time::Duration;

use leptos::create_rw_signal;
use leptos::document;
use leptos::js_sys::JsString;
use leptos::set_timeout;
use leptos::web_sys::Element;
use leptos::web_sys::HtmlElement;
use leptos::web_sys::SvgElement;
use leptos::JsCast;
use leptos::RwSignal;
use leptos::Scope;
use leptos::UntrackedGettableSignal;
use leptos::UntrackedSettableSignal;
use leptos_aria_utils::is_ios;
use leptos_aria_utils::run_after_transition;
use leptos_aria_utils::ContextProvider;
use leptos_aria_utils::Map;

#[derive(Copy, Clone)]
pub(crate) struct SelectionContext(RwSignal<Selection>);

impl ContextProvider for SelectionContext {
  type Value = Selection;

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

#[derive(Copy, Clone)]
pub(crate) struct UserSelectContext(RwSignal<Option<String>>);

impl ContextProvider for UserSelectContext {
  type Value = Option<String>;

  fn from_leptos_scope(cx: Scope) -> Self {
    Self(create_rw_signal(cx, None))
  }

  fn get(&self) -> Self::Value {
    self.0.get_untracked()
  }

  fn set(&self, value: Self::Value) {
    self.0.set(value)
  }
}

type ElementMap = Map<Element, JsString>;

#[derive(Copy, Clone)]
pub(crate) struct ElementMapContext(RwSignal<ElementMap>);

impl ContextProvider for ElementMapContext {
  type Value = ElementMap;

  fn from_leptos_scope(cx: Scope) -> Self {
    Self(create_rw_signal(cx, Default::default()))
  }

  fn get(&self) -> Self::Value {
    self.0.get_untracked()
  }

  fn set(&self, value: Self::Value) {
    let reference = &value;
    self.0.set_untracked(if eq(reference, &self.get()) {
      // this happens when the value was directly mutated.
      reference.clone()
    } else {
      value
    });
  }
}

pub(crate) fn disable_text_selection(cx: Scope, element: Option<impl AsRef<Element>>) {
  if is_ios() {
    let selection = SelectionContext::provide(cx);
    let user_select = UserSelectContext::provide(cx);

    if selection.get() == Selection::Default {
      let style = document()
        .document_element()
        .unwrap()
        .unchecked_ref::<HtmlElement>()
        .style();
      user_select.set(style.get_property_value("-webkit-user-select").ok());
      style.set_property("-webkit-user-select", "none").ok();
    }

    selection.set(Selection::Disabled);
    return;
  }

  let Some(target) = element.as_ref().map(|item| item.as_ref()) else {
      return;
    };

  if !target.is_instance_of::<HtmlElement>() && !target.is_instance_of::<HtmlElement>() {
    return;
  }

  let _should_append = true;
  let element_list = ElementMapContext::provide(cx);
  let style = target.unchecked_ref::<HtmlElement>().style();
  let map = element_list.get();
  let _cloned_target = target.clone();
  let user_select = style.get_property_value("user-select").unwrap_or("".into());
  map.set(target, &user_select.into());

  element_list.set(map);
}

/// Safari on iOS starts selecting text on long press. The only way to avoid
/// this, it seems, is to add user-select: none to the entire page. Adding it
/// to the pressable element prevents that element from being selected, but
/// nearby elements may still receive selection. We add user-select: none on
/// touch start, and remove it again on touch end to prevent this. This must
/// be implemented using global state to avoid race conditions between
/// multiple elements.
///
/// There are three possible states due to the delay before removing
/// user-select: none after pointer up. The 'default' state always transitions
/// to the 'disabled' state, which transitions to 'restoring'. The 'restoring'
/// state can either transition back to 'disabled' or 'default'.
///
/// For non-iOS devices, we apply user-select: none to the pressed element
/// instead to avoid possible performance issues that arise from applying and
/// removing user-select: none to the entire page (see https://github.com/adobe/react-spectrum/issues/1609).
pub(crate) fn restore_text_selection(cx: Scope, element: impl AsRef<Element>) {
  if is_ios() {
    let selection = SelectionContext::provide(cx);
    let user_select = UserSelectContext::provide(cx);

    // If the state is already the default, there's nothing to do.
    // If restoring, then there's no need to queue a second restore.
    // if state != "disable"
    if selection.get() != Selection::Default {
      return;
    }

    selection.set(Selection::Restoring);

    let timeout_callback = move || {
      if selection.get() != Selection::Default {
        return;
      }

      let document_element: HtmlElement = document().document_element().unwrap().unchecked_into();

      if document_element
        .style()
        .get_property_value("-webkit-user-select")
        .ok()
        .as_deref()
        == Some("none")
      {
        document_element
          .style()
          .set_property(
            "-webkit-user-select",
            user_select.get().as_deref().unwrap_or(""),
          )
          .ok();
      }

      selection.set(Selection::Default);
      user_select.set(None);
    };

    set_timeout(
      move || run_after_transition(cx, timeout_callback),
      Duration::from_millis(300),
    );

    return;
  }

  let target = element.as_ref();

  if !target.is_instance_of::<HtmlElement>() && !target.is_instance_of::<SvgElement>() {
    return;
  }

  let element_map = ElementMapContext::provide(cx);
  let map = element_map.get();

  let Some(found_selection) = map.get(target) else {
    return;
  };

  let style = target.unchecked_ref::<HtmlElement>().style();
  if style.get_property_value("user-select").ok().as_deref() == Some("none") {
    let found_selection: String = found_selection.into();
    style
      .set_property("user-select", found_selection.as_str())
      .ok();
  }

  if target
    .get_attribute("style")
    .as_ref()
    .filter(|value| !value.is_empty())
    .is_none()
  {
    target.remove_attribute("style").ok();
  }

  map.delete(target);
  element_map.set(map);
}

#[derive(Copy, Clone, PartialEq)]
pub(crate) enum Selection {
  Default,
  Disabled,
  Restoring,
}

impl Default for Selection {
  fn default() -> Self {
    Self::Default
  }
}
