use std::ptr::eq;
use std::time::Duration;

use leptos::*;
use leptos_aria_utils::is_ios;
use leptos_aria_utils::ContextProvider;

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

type ElementList = Vec<(web_sys::Element, String)>;

#[derive(Copy, Clone)]
pub(crate) struct ElementListContext(RwSignal<ElementList>);

impl ContextProvider for ElementListContext {
  type Value = ElementList;

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

pub(crate) fn disable_text_selection(cx: Scope, element: Option<impl AsRef<web_sys::Element>>) {
  if is_ios() {
    let selection = SelectionContext::provide(cx);
    let user_select = UserSelectContext::provide(cx);

    if selection.get() == Selection::Default {
      let style = document()
        .document_element()
        .unwrap()
        .unchecked_ref::<web_sys::HtmlElement>()
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

  if !target.is_instance_of::<web_sys::HtmlElement>()
    && !target.is_instance_of::<web_sys::HtmlElement>()
  {
    return;
  }

  let mut should_append = true;
  let element_list = ElementListContext::provide(cx);
  let style = target.unchecked_ref::<web_sys::HtmlElement>().style();
  let list = element_list.get();
  let cloned_target = target.clone();
  let user_select = style.get_property_value("user-select").unwrap_or("".into());

  let mut list = list
    .iter()
    .map(|item| {
      if item.0 == cloned_target {
        should_append = false;

        (item.0.clone(), user_select.clone())
      } else {
        item.clone()
      }
    })
    .collect::<Vec<_>>();

  if should_append {
    list.push((cloned_target, user_select));
  }

  element_list.set(list);
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
pub(crate) fn restore_text_selection(cx: Scope, element: impl AsRef<web_sys::Element>) {
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

    let cb = move || {
      if selection.get() != Selection::Default {
        return;
      }

      let document_element: web_sys::HtmlElement =
        document().document_element().unwrap().unchecked_into();

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

    set_timeout(cb, Duration::from_millis(300));

    return;
  }

  let target = element.as_ref();

  if !target.is_instance_of::<web_sys::HtmlElement>()
    && !target.is_instance_of::<web_sys::SvgElement>()
  {
    return;
  }

  let element_list = ElementListContext::provide(cx);
  let list = element_list.get();

  let Some((_, found_selection)) = list.iter().find(|value| value.0 == target.clone()) else {
    return;
  };

  let style = target.unchecked_ref::<web_sys::HtmlElement>().style();
  if style.get_property_value("user-select").ok().as_deref() == Some("none") {
    style.set_property("user-select", found_selection).ok();
  }

  if target
    .get_attribute("style")
    .as_ref()
    .filter(|value| !value.is_empty())
    .is_none()
  {
    target.remove_attribute("style").ok();
  }

  let new_list = list
    .iter()
    .filter(|&item| item.0 != target.clone())
    .map(|item| (item.0.clone(), item.1.clone()))
    .collect();

  element_list.set(new_list);

  // let targetOldc

  // let value = element.to_leptos_element(cx);
  // let hashable_target = target
  //   .unchecked_ref::<web_sys::HtmlElement>()
  //   .to_leptos_element();
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
