use std::rc::Rc;
use std::sync::Arc;
use std::sync::RwLock;

use leptos::create_rw_signal;
use leptos::document;
use leptos::js_sys::Function;
use leptos::typed_builder::TypedBuilder;
use leptos::wasm_bindgen::prelude::Closure;
use leptos::web_sys::DragEvent;
use leptos::web_sys::Element;
use leptos::web_sys::HtmlAnchorElement;
use leptos::web_sys::HtmlElement;
use leptos::web_sys::HtmlInputElement;
use leptos::web_sys::HtmlTextAreaElement;
use leptos::web_sys::KeyboardEvent;
use leptos::web_sys::MouseEvent;
use leptos::web_sys::PointerEvent;
use leptos::web_sys::TouchEvent;
use leptos::web_sys::WheelEvent;
use leptos::AnyElement;
use leptos::IntoSignal;
use leptos::JsCast;
use leptos::MaybeSignal;
use leptos::NodeRef;
use leptos::Scope;
use leptos::Signal;
use leptos::UntrackedGettableSignal;
use leptos::UntrackedSettableSignal;
use leptos::*;
use leptos_aria_utils::focus_without_scrolling;
use leptos_aria_utils::is_virtual_click;
use leptos_aria_utils::is_virtual_pointer_event;
use leptos_aria_utils::FocusableElement;
use leptos_aria_utils::GlobalListeners;
use leptos_aria_utils::ToFocusableElement;
use web_sys::DomRect;
use web_sys::HtmlButtonElement;
use web_sys::Node;

use crate::text_selection::disable_text_selection;
use crate::text_selection::restore_text_selection;

pub fn use_press(cx: Scope, props: UsePressProps) -> PressResult {
  let listeners = Arc::new(RwLock::new(GlobalListeners::default()));
  let ignore_emulated_mouse_events = create_rw_signal(cx, false);
  let ignore_click_after_press = create_rw_signal(cx, false);
  let did_fire_press_start = create_rw_signal(cx, false);
  let active_pointer_id = create_rw_signal::<Option<i32>>(cx, None);
  let target = create_rw_signal::<Option<Element>>(cx, None);
  let is_over_target = create_rw_signal(cx, false);
  let pointer_type = create_rw_signal(cx, PointerType::Unsupported);
  let user_select = create_rw_signal::<Option<String>>(cx, None);

  let original_is_disabled = props.is_disabled.unwrap_or(false.into());
  let is_disabled = (move || original_is_disabled.get()).derive_signal(cx);
  let original_prevent_focus_on_press = props.prevent_focus_on_press.unwrap_or(false.into());
  let prevent_focus_on_press = (move || original_prevent_focus_on_press.get()).derive_signal(cx);
  let original_should_cancel_on_pointer_exit =
    props.should_cancel_on_pointer_exit.unwrap_or(false.into());
  let should_cancel_on_pointer_exit =
    (move || original_should_cancel_on_pointer_exit.get()).derive_signal(cx);
  let original_allow_text_selection_on_press =
    props.allow_text_selection_on_press.unwrap_or(false.into());
  let allow_text_selection_on_press =
    (move || original_allow_text_selection_on_press.get()).derive_signal(cx);

  let wrapped_on_press: Option<OnPressCallback> = props.on_press.map(Rc::new);
  let wrapped_on_press_start: Option<OnPressCallback> = props.on_press_start.map(Rc::new);
  let wrapped_on_press_end: Option<OnPressCallback> = props.on_press_end.map(Rc::new);
  let wrapped_on_press_change: Option<OnPressChangeCallback> = props.on_press_change.map(Rc::new);
  let wrapped_on_press_up: Option<OnPressCallback> = props.on_press_up.map(Rc::new);

  let is_pressed = create_rw_signal(cx, false);
  let original_is_pressed = props.is_pressed.unwrap_or(false.into());
  let derived_is_pressed =
    (move || original_is_pressed.get() || is_pressed.get()).derive_signal(cx);

  // Trigger the beginning of a custom press event.
  let trigger_press_start = {
    let wrapped_on_press_start = wrapped_on_press_start.clone();
    let wrapped_on_press_change = wrapped_on_press_change.clone();

    move |focusable_event: &FocusableEvent, pointer: PointerType| {
      if is_disabled.get() || did_fire_press_start.get_untracked() {
        return;
      }

      let event = PressEvent::create(&pointer, PressEventType::PressStart, focusable_event);
      call_event(&wrapped_on_press_start, &event);
      call_event(&wrapped_on_press_change, true);

      did_fire_press_start.set_untracked(true);
      is_pressed.set(true);
    }
  };

  let trigger_press_end = {
    let wrapped_on_press_end = wrapped_on_press_end.clone();
    let wrapped_on_press_change = wrapped_on_press_change.clone();

    let callback =
      move |focusable_event: &FocusableEvent, pointer: PointerType, was_pressed: bool| {
        if !did_fire_press_start.get_untracked() {
          return;
        }

        ignore_click_after_press.set_untracked(true);
        did_fire_press_start.set_untracked(false);

        let event = PressEvent::create(&pointer, PressEventType::PressEnd, focusable_event);
        call_event(&wrapped_on_press_end.clone(), &event);
        call_event(&wrapped_on_press_change.clone(), false);

        is_pressed.set(false);

        if !was_pressed || is_disabled.get() {
          return;
        }

        let event = PressEvent::create(&pointer, PressEventType::Press, focusable_event);
        call_event(&wrapped_on_press, &event);
      };

    Rc::new(Box::new(callback))
  };

  let trigger_press_up = {
    let callback = move |focusable_event: &FocusableEvent, pointer: PointerType| {
      if is_disabled.get() {
        return;
      }

      let event = PressEvent::create(&pointer, PressEventType::PressUp, focusable_event);
      call_event(&wrapped_on_press_up, &event);
    };

    Rc::new(Box::new(callback))
  };

  let cancel = {
    let trigger_press_end = trigger_press_end.clone();
    let listeners = listeners.clone();

    let callback = move |focusable_event: &FocusableEvent| {
      if !is_pressed.get_untracked() {
        return;
      }

      if is_over_target.get_untracked() {
        trigger_press_end(focusable_event, pointer_type.get_untracked(), false);
      }

      is_pressed.set_untracked(false);
      is_over_target.set_untracked(false);
      active_pointer_id.set_untracked(None);
      pointer_type.set_untracked(PointerType::Unsupported);

      listeners.write().unwrap().remove_all_listeners();

      if !allow_text_selection_on_press.get() {
        if let Some(ref element) = target.get_untracked() {
          restore_text_selection(cx, element);
        }
      }
    };

    Rc::new(Box::new(callback))
  };

  let on_key_up: PressCallback<KeyboardEvent> = {
    let trigger_press_up = trigger_press_up.clone();
    let handler = move |event: KeyboardEvent| {
      let event_current_target: Element = event.current_target().unwrap().unchecked_into();
      let event_target: Option<Node> = event.target().map(|target| target.unchecked_into());

      if !is_valid_keyboard_event(&event, &event_current_target)
        || event.repeat()
        || !event_current_target.contains(event_target.as_ref())
      {
        return;
      }

      let focusable_event = FocusableEvent::Keyboard(
        event,
        target
          .get_untracked()
          .map(|target| target.to_focusable_element()),
      );

      trigger_press_up(&focusable_event, PointerType::Keyboard);
    };

    Rc::new(Box::new(handler))
  };

  let global_on_key_up: PressCallback<KeyboardEvent> = {
    let trigger_press_end = trigger_press_end.clone();
    let listeners = listeners.clone();

    let handler = move |event: KeyboardEvent| {
      let event_current_target: Element = event.current_target().unwrap().unchecked_into();
      let event_target: Option<Node> = event.target().map(|target| target.unchecked_into());

      if !is_pressed.get_untracked() || !is_valid_keyboard_event(&event, &event_current_target) {
        return;
      }

      if should_prevent_default(&event_current_target) {
        event.prevent_default();
      }

      event.stop_propagation();
      is_pressed.set_untracked(false);
      let focusable_event = FocusableEvent::Keyboard(
        event,
        target
          .get_untracked()
          .map(|target| target.to_focusable_element()),
      );

      let contains_target = target
        .get_untracked()
        .as_ref()
        .map(|element| element.contains(event_target.as_ref()))
        .unwrap_or(false);

      trigger_press_end(&focusable_event, PointerType::Keyboard, contains_target);
      listeners.write().unwrap().remove_all_listeners();

      let Some(ref element) = target.get_untracked() else {
        return;
      };

      if !element.is_instance_of::<HtmlElement>()
        || !element.contains(event_target.as_ref())
        || !has_link_role(element)
      {
        return;
      }

      element.unchecked_ref::<HtmlElement>().click();
    };

    Rc::new(Box::new(handler))
  };

  let on_key_down: PressCallback<KeyboardEvent> = {
    let global_on_key_up = global_on_key_up.clone();
    let trigger_press_start = trigger_press_start.clone();
    let listeners = listeners.clone();

    let handler = move |event: KeyboardEvent| {
      let event_current_target: Element = event.current_target().unwrap().unchecked_into();
      let event_target: Option<Node> = event.target().map(|target| target.unchecked_into());

      if is_valid_keyboard_event(&event, &event_current_target)
        && event_current_target.contains(event_target.as_ref())
      {
        if should_prevent_default(&event_current_target) {
          event.prevent_default();
        }

        event.stop_propagation();

        // If the event is repeating, it may have started on a different element
        // after which focus moved to the current element. Ignore these events and
        // only handle the first key down event.
        if !is_pressed.get_untracked() && !event.repeat() {
          target.set_untracked(Some(event_current_target.clone()));
          is_pressed.set_untracked(true);
          let focusable_event = FocusableEvent::Keyboard(event, None);
          trigger_press_start(&focusable_event, PointerType::Keyboard);
          let callback = {
            let global_on_key_up = global_on_key_up.clone();
            move |event: KeyboardEvent| {
              global_on_key_up(event);
            }
          };
          let closure = Closure::wrap(Box::new(callback) as Box<dyn Fn(KeyboardEvent)>);
          let function = closure.as_ref().unchecked_ref::<Function>().clone();

          // Focus may move before the key up event, so register the event on the document
          // instead of the same element where the key down event occurred.
          listeners
            .write()
            .unwrap()
            .add_listener(document(), "keyup", function, false);
        }
      } else if event.key() == "Enter" && is_html_anchor_link(&event_current_target) {
        // If the target is a link, we won't have handled this above because we want the
        // default browser behavior to open the link when pressing Enter. But we
        // still need to prevent default so that elements above do not also handle
        // it (e.g. table row).
        event.stop_propagation();
      }
    };

    Rc::new(Box::new(handler))
  };

  let on_click: PressCallback<MouseEvent> = {
    let trigger_press_start = trigger_press_start.clone();
    let trigger_press_up = trigger_press_up.clone();
    let trigger_press_end = trigger_press_end.clone();

    let callback = move |event: MouseEvent| {
      let event_current_target: Element = event.current_target().unwrap().unchecked_into();
      let event_target: Option<Node> = event.target().map(|target| target.unchecked_into());

      if !event_current_target.contains(event_target.as_ref()) {
        return;
      }

      // Ensure it was the main mouse button that was clicked.
      if event.button() != 0 {
        return;
      }

      event.stop_propagation();

      if is_disabled.get_untracked() {
        event.prevent_default();
      }

      // If triggered from a screen reader or by using element.click(),
      // trigger as if it were a keyboard click.
      if !ignore_click_after_press.get_untracked()
        && !ignore_emulated_mouse_events.get_untracked()
        && (pointer_type.get_untracked() == PointerType::Virtual || is_virtual_click(&event))
      {
        if !is_disabled.get_untracked() || !prevent_focus_on_press.get_untracked() {
          focus_without_scrolling(&event_current_target);
        }

        let focusable_event = FocusableEvent::Mouse(event, None);
        trigger_press_start(&focusable_event, PointerType::Virtual);
        trigger_press_up(&focusable_event, PointerType::Virtual);
        trigger_press_end(&focusable_event, PointerType::Virtual, true);
      }

      ignore_emulated_mouse_events.set_untracked(false);
      ignore_click_after_press.set_untracked(false);
    };

    Rc::new(Box::new(callback))
  };

  let on_drag_start: PressCallback<DragEvent> = {
    let cancel = cancel.clone();

    let handler = move |event: DragEvent| {
      let event_current_target: Element = event.current_target().unwrap().unchecked_into();
      let event_target: Option<Node> = event.target().map(|target| target.unchecked_into());

      if !event_current_target.contains(event_target.as_ref()) {
        return;
      }

      // Safari does not call onPointerCancel when a drag starts, whereas Chrome
      // and Firefox do.
      let focusable_event = FocusableEvent::Drag(event, None);
      cancel(&focusable_event);
    };

    Rc::new(Box::new(handler))
  };

  let on_mouse_down: PressCallback<MouseEvent> = {
    let handler = move |event: MouseEvent| {
      let event_current_target: Element = event.current_target().unwrap().unchecked_into();
      let event_target: Option<Node> = event.target().map(|target| target.unchecked_into());

      if event_current_target.contains(event_target.as_ref()) {
        return;
      }

      if event.button() != 0 {
        return;
      }

      // Chrome and Firefox on touch Windows devices require mouse down events
      // to be canceled in addition to pointer events, or an extra asynchronous
      // focus event will be fired.
      if should_prevent_default(event_current_target) {
        event.prevent_default();
      }

      event.stop_propagation();
    };

    Rc::new(Box::new(handler))
  };

  let on_pointer_cancel: PressCallback<PointerEvent> = {
    let cancel = cancel.clone();

    let handler = move |event: PointerEvent| {
      let focusable_event = FocusableEvent::Pointer(event, None);
      cancel(&focusable_event);
    };

    Rc::new(Box::new(handler))
  };

  let on_pointer_enter: PressCallback<PointerEvent> = {
    let handler = move |_| {};
    Rc::new(Box::new(handler))
  };

  let on_pointer_leave: PressCallback<PointerEvent> = {
    let handler = move |_| {};
    Rc::new(Box::new(handler))
  };

  // Safari on iOS < 13.2 does not implement pointerenter/pointerleave events
  // correctly. Use pointer move events instead to implement our own hit
  // testing. See https://bugs.webkit.org/show_bug.cgi?id=199803
  let on_pointer_move: PressCallback<PointerEvent> = {
    let trigger_press_start = trigger_press_start.clone();
    let trigger_press_end = trigger_press_end.clone();
    let cancel = cancel.clone();

    let handler = move |event: PointerEvent| {
      if Some(event.pointer_id()) != active_pointer_id.get_untracked() {
        return;
      }

      let Some(ref element) = target.get_untracked() else {
        return;
      };

      let focusable_event =
        FocusableEvent::Pointer(event.clone(), Some(element.to_focusable_element()));

      if is_above_target(&event, element) && !is_over_target.get_untracked() {
        is_over_target.set_untracked(true);
        trigger_press_start(&focusable_event, pointer_type.get_untracked());
      } else if is_over_target.get_untracked() {
        is_over_target.set_untracked(false);
        trigger_press_end(&focusable_event, pointer_type.get_untracked(), false);

        if should_cancel_on_pointer_exit.get_untracked() {
          cancel(&focusable_event);
        }
      }
    };

    Rc::new(Box::new(handler))
  };

  let on_pointer_up: PressCallback<PointerEvent> = {
    let handler = move |event: PointerEvent| {
      // iOS fires pointerup with zero width and height, so check the pointerType
      // recorded during pointerdown.
      let event_current_target: Element = event.current_target().unwrap().unchecked_into();
      let event_target: Option<Node> = event.target().map(|target| target.unchecked_into());

      if !event_current_target.contains(event_target.as_ref())
        || pointer_type.get_untracked() == PointerType::Virtual
      {
        return;
      }

      // Only handle left clicks
      // Safari on iOS sometimes fires pointerup events, even
      // when the touch isn't over the target, so double check.
      if event.button() != 0 || !is_above_target(&event, &event_current_target) {
        return;
      }

      let focusable_event = FocusableEvent::Pointer(event, None);
      trigger_press_up(&focusable_event, PointerType::Mouse);
    };

    Rc::new(Box::new(handler))
  };

  let global_on_pointer_up: PressCallback<PointerEvent> = {
    let trigger_press_end = trigger_press_end.clone();
    let listeners = listeners.clone();

    let handler = move |event: PointerEvent| {
      if Some(event.pointer_id()) != active_pointer_id.get_untracked()
        || !is_pressed.get_untracked()
        || event.button() != 0
      {
        return;
      }

      let Some(ref element) = target.get_untracked() else {
        return;
      };

      let focusable_event =
        FocusableEvent::Pointer(event.clone(), Some(element.to_focusable_element()));

      if is_above_target(&event, element) {
        trigger_press_end(&focusable_event, pointer_type.get_untracked(), true);
      } else if is_over_target.get_untracked() {
        trigger_press_end(&focusable_event, pointer_type.get_untracked(), false);
      }

      is_pressed.set_untracked(false);
      is_over_target.set_untracked(false);
      active_pointer_id.set_untracked(None);
      pointer_type.set_untracked(PointerType::Unsupported);
      listeners.write().unwrap().remove_all_listeners();

      if !allow_text_selection_on_press.get_untracked() {
        restore_text_selection(cx, element);
      }
    };

    Rc::new(Box::new(handler))
  };

  let on_pointer_down: PressCallback<PointerEvent> = {
    let trigger_press_start = trigger_press_start.clone();
    let on_pointer_move = on_pointer_move.clone();
    let on_pointer_cancel = on_pointer_cancel.clone();
    // let listeners = listeners.clone();

    let handler = move |event: PointerEvent| {
      let event_current_target: Element = event.current_target().unwrap().unchecked_into();
      let event_target: Option<Node> = event.target().map(|target| target.unchecked_into());

      // Only handle left clicks, and ignore events that bubbled through portals.
      if event.button() == 0 || !event_current_target.contains(event_target.as_ref()) {
        return;
      }

      // iOS safari fires pointer events from VoiceOver with incorrect
      // coordinates/target. Ignore and let the onClick handler take care of it
      // instead. https://bugs.webkit.org/show_bug.cgi?id=222627
      // https://bugs.webkit.org/show_bug.cgi?id=223202

      if is_virtual_pointer_event(&event) {
        pointer_type.set_untracked(PointerType::Virtual);
        return;
      }

      // Due to browser inconsistencies, especially on mobile browsers, we
      // prevent default on pointer down and handle focusing the pressable
      // element ourselves.
      if should_prevent_default(&event_current_target) {
        event.prevent_default();
      }

      pointer_type.set_untracked(event.pointer_type().into());
      event.stop_propagation();

      if is_pressed.get_untracked() {
        return;
      }

      is_pressed.set_untracked(true);
      is_over_target.set_untracked(true);
      active_pointer_id.set_untracked(Some(event.pointer_id()));
      target.set_untracked(Some(event_current_target.clone()));

      if !is_disabled.get_untracked() || !prevent_focus_on_press.get_untracked() {
        focus_without_scrolling(&event_current_target);
      }

      if allow_text_selection_on_press.get_untracked() {
        disable_text_selection(cx, &target.get_untracked());
      }

      let focusable_event = FocusableEvent::Pointer(event, None);
      trigger_press_start(&focusable_event, pointer_type.get_untracked());

      let pointer_move_function = {
        let on_pointer_move = on_pointer_move.clone();
        let callback = move |event: PointerEvent| {
          on_pointer_move(event);
        };
        Closure::wrap(Box::new(callback) as Box<dyn Fn(PointerEvent)>)
          .as_ref()
          .unchecked_ref::<Function>()
          .clone()
      };

      let pointer_up_function = {
        let on_pointer_up = global_on_pointer_up.clone();
        let callback = move |event: PointerEvent| {
          on_pointer_up(event);
        };
        Closure::wrap(Box::new(callback) as Box<dyn Fn(PointerEvent)>)
          .as_ref()
          .unchecked_ref::<Function>()
          .clone()
      };

      let pointer_cancel_function = {
        let on_pointer_cancel = on_pointer_cancel.clone();
        let callback = move |event: PointerEvent| {
          on_pointer_cancel(event);
        };
        Closure::wrap(Box::new(callback) as Box<dyn Fn(PointerEvent)>)
          .as_ref()
          .unchecked_ref::<Function>()
          .clone()
      };

      let mut global_listener = listeners.write().unwrap();
      global_listener.add_listener(document(), "pointermove", pointer_move_function, false);
      global_listener.add_listener(document(), "pointerup", pointer_up_function, false);
      global_listener.add_listener(document(), "pointercancel", pointer_cancel_function, false);
    };

    Rc::new(Box::new(handler))
  };

  PressResult {
    is_pressed: derived_is_pressed,
    is_disabled,
    prevent_focus_on_press,
    should_cancel_on_pointer_exit,
    allow_text_selection_on_press,
    on_click,
    on_drag_start,
    on_key_down,
    on_key_up,
    on_mouse_down,
    on_pointer_down,
    on_pointer_enter,
    on_pointer_leave,
    on_pointer_up,
  }
}

type OnPressCallback = Rc<Box<dyn Fn(&PressEvent)>>;
type OnPressChangeCallback = Rc<Box<dyn Fn(bool)>>;
type PressCallback<E> = Rc<Box<dyn Fn(E)>>;

#[derive(Clone, TypedBuilder)]
pub struct PressResult {
  pub allow_text_selection_on_press: Signal<bool>,
  pub is_disabled: Signal<bool>,
  pub is_pressed: Signal<bool>,
  pub prevent_focus_on_press: Signal<bool>,
  pub should_cancel_on_pointer_exit: Signal<bool>,
  pub on_click: PressCallback<MouseEvent>,
  pub on_drag_start: PressCallback<DragEvent>,
  pub on_key_down: PressCallback<KeyboardEvent>,
  pub on_key_up: PressCallback<KeyboardEvent>,
  pub on_mouse_down: PressCallback<MouseEvent>,
  pub on_pointer_down: PressCallback<PointerEvent>,
  pub on_pointer_enter: PressCallback<PointerEvent>,
  pub on_pointer_leave: PressCallback<PointerEvent>,
  pub on_pointer_up: PressCallback<PointerEvent>,
}

fn call_event<E>(callback: &Option<PressCallback<E>>, event: E) {
  if let Some(ref callback) = callback {
    let cb = callback.clone();
    cb(event);
  }
}

fn are_rectangles_overlapping(dom_rect: &DomRect, rects: &Vec<Rect>) -> bool {
  let mut is_overlapping = false;

  for rect in rects {
    // check if they cannot overlap on x axis
    if dom_rect.left() > rect.right || dom_rect.right() < rect.left {
      continue;
    }

    // check if they cannot overlap on y axis
    if dom_rect.top() > rect.bottom || dom_rect.bottom() < rect.top {
      continue;
    }

    is_overlapping = true;
    break;
  }

  is_overlapping
}

fn is_above_target(point: &impl GetRects, target: &Element) -> bool {
  let rect = target.get_bounding_client_rect();
  let point_rects = point.get_rects();
  are_rectangles_overlapping(&rect, &point_rects)
}

/// We cannot prevent default if the target is not an HTMLElement or if it is
/// a draggable HTMLElement.
fn should_prevent_default(target: impl AsRef<Element>) -> bool {
  let element = target.as_ref();
  !element.is_instance_of::<HtmlElement>() || !element.unchecked_ref::<HtmlElement>().draggable()
}

fn should_prevent_default_keyboard(target: impl AsRef<Element>, key: String) -> bool {
  let element = target.as_ref();

  if element.is_instance_of::<HtmlInputElement>() {
    !is_valid_input_key(element.unchecked_ref(), key)
  } else if element.is_instance_of::<HtmlButtonElement>() {
    element.unchecked_ref::<HtmlButtonElement>().type_() == "submit"
  } else {
    true
  }
}

fn is_valid_keyboard_event(
  event: impl AsRef<KeyboardEvent>,
  current_target: impl AsRef<Element>,
) -> bool {
  let event = event.as_ref();
  let current_target = current_target.as_ref();
  let key = event.key();
  let code = event.code();
  let element = current_target.unchecked_ref::<HtmlElement>();

  let role = element.get_attribute("role");

  // Accessibility for keyboards. Space and Enter only.
  (key == "Enter" || key == " " || code == "Space")
    && !((element.is_instance_of::<HtmlInputElement>()
      && !{
        is_valid_input_key(element.unchecked_ref(), &key)
      })
      || element.is_instance_of::<HtmlTextAreaElement>()
      || element.is_content_editable())
  // A link with a valid href should be handled natively,
  // unless is also has `role="button"` and was triggered using `Space`.
  && (!is_html_anchor_link(element) || (role.as_ref().map_or(false, |role| role == "button" )&& key != "Enter"))
  && !(role.as_ref().map_or(false, |role| role == "link") && key != "Enter")
}

fn is_html_anchor_link(target: impl AsRef<Element>) -> bool {
  let element = target.as_ref();
  element.is_instance_of::<HtmlAnchorElement>()
    || (element.tag_name() == "A" && element.has_attribute("href"))
}

fn has_link_role(target: impl AsRef<Element>) -> bool {
  let element = target.as_ref();
  is_html_anchor_link(element)
    || element
      .get_attribute("role")
      .as_ref()
      .map_or(false, |role| role == "link")
}

fn is_valid_input_key(target: &HtmlInputElement, key: impl AsRef<str>) -> bool {
  // Only space should toggle checkboxes and radios, not enter.
  if target.type_() == "checkbox" || target.type_() == "radio" {
    key.as_ref() == " "
  } else {
    NON_TEXT_INPUT_TYPES.contains(&target.type_().as_str())
  }
}

const NON_TEXT_INPUT_TYPES: &[&str; 9] = &[
  "checkbox", "radio", "range", "color", "file", "image", "button", "submit", "reset",
];

#[derive(Debug, Clone, Copy, Default)]
pub struct Rect {
  pub top: f64,
  pub right: f64,
  pub bottom: f64,
  pub left: f64,
}

pub trait GetRects {
  fn get_rects(&self) -> Vec<Rect>;
}

impl GetRects for MouseEvent {
  fn get_rects(&self) -> Vec<Rect> {
    vec![Rect {
      top: self.client_y().into(),
      right: self.client_x().into(),
      bottom: self.client_y().into(),
      left: self.client_x().into(),
    }]
  }
}

impl GetRects for PointerEvent {
  fn get_rects(&self) -> Vec<Rect> {
    let offset_x = self.width();
    let offset_y = self.height();

    let top = (self.client_y() - offset_y).into();
    let right = (self.client_x() + offset_x).into();
    let bottom = (self.client_y() + offset_y).into();
    let left = (self.client_x() - offset_x).into();

    vec![Rect {
      top,
      right,
      bottom,
      left,
    }]
  }
}

impl GetRects for DragEvent {
  fn get_rects(&self) -> Vec<Rect> {
    vec![Rect {
      top: self.client_y().into(),
      right: self.client_x().into(),
      bottom: self.client_y().into(),
      left: self.client_x().into(),
    }]
  }
}

impl GetRects for TouchEvent {
  fn get_rects(&self) -> Vec<Rect> {
    let mut rects = vec![];
    self.touches(); // TODO disable this on safari
    let touches = self.touches();

    for index in 0..touches.length() {
      let touch = touches.item(index).unwrap();
      let offset_x = touch.radius_x();
      let offset_y = touch.radius_y();
      let top = (touch.client_y() - offset_y).into();
      let right = (touch.client_x() + offset_x).into();
      let bottom = (touch.client_y() + offset_y).into();
      let left = (touch.client_x() - offset_x).into();

      rects.push(Rect {
        top,
        right,
        bottom,
        left,
      });
    }

    // self.radius
    rects
  }
}

/// Any event that can be pressed.
#[derive(Clone)]
pub enum FocusableEvent {
  Mouse(MouseEvent, Option<FocusableElement>),
  Keyboard(KeyboardEvent, Option<FocusableElement>),
  Touch(TouchEvent, Option<FocusableElement>),
  Drag(DragEvent, Option<FocusableElement>),
  Pointer(PointerEvent, Option<FocusableElement>),
  Wheel(WheelEvent, Option<FocusableElement>),
}

impl FocusableEvent {
  pub fn focusable_target(&self) -> FocusableElement {
    use FocusableEvent::*;

    match self {
      Mouse(_, element) => {
        element
          .as_ref()
          .unwrap_or(&self.current_target().into())
          .clone()
      }
      Keyboard(_, element) => {
        element
          .as_ref()
          .unwrap_or(&self.current_target().into())
          .clone()
      }
      Touch(_, element) => {
        element
          .as_ref()
          .unwrap_or(&self.current_target().into())
          .clone()
      }
      Drag(_, element) => {
        element
          .as_ref()
          .unwrap_or(&self.current_target().into())
          .clone()
      }
      Pointer(_, element) => {
        element
          .as_ref()
          .unwrap_or(&self.current_target().into())
          .clone()
      }
      Wheel(_, element) => {
        element
          .as_ref()
          .unwrap_or(&self.current_target().into())
          .clone()
      }
    }
  }

  pub fn current_target(&self) -> Element {
    use FocusableEvent::*;

    let target = match self {
      Mouse(event, _) => event.current_target(),
      Keyboard(event, _) => event.current_target(),
      Touch(event, _) => event.current_target(),
      Drag(event, _) => event.current_target(),
      Pointer(event, _) => event.current_target(),
      Wheel(event, _) => event.current_target(),
    };

    target.unwrap().unchecked_into()
  }

  pub fn shift_key(&self) -> bool {
    use FocusableEvent::*;

    match self {
      Mouse(event, _) => event.shift_key(),
      Keyboard(event, _) => event.shift_key(),
      Touch(event, _) => event.shift_key(),
      Drag(event, _) => event.shift_key(),
      Pointer(event, _) => event.shift_key(),
      Wheel(event, _) => event.shift_key(),
    }
  }

  pub fn ctrl_key(&self) -> bool {
    use FocusableEvent::*;

    match self {
      Mouse(event, _) => event.ctrl_key(),
      Keyboard(event, _) => event.ctrl_key(),
      Touch(event, _) => event.ctrl_key(),
      Drag(event, _) => event.ctrl_key(),
      Pointer(event, _) => event.ctrl_key(),
      Wheel(event, _) => event.ctrl_key(),
    }
  }

  pub fn alt_key(&self) -> bool {
    use FocusableEvent::*;

    match self {
      Mouse(event, _) => event.alt_key(),
      Keyboard(event, _) => event.alt_key(),
      Touch(event, _) => event.alt_key(),
      Drag(event, _) => event.alt_key(),
      Pointer(event, _) => event.alt_key(),
      Wheel(event, _) => event.alt_key(),
    }
  }

  pub fn meta_key(&self) -> bool {
    use FocusableEvent::*;

    match self {
      Mouse(event, _) => event.meta_key(),
      Keyboard(event, _) => event.meta_key(),
      Touch(event, _) => event.meta_key(),
      Drag(event, _) => event.meta_key(),
      Pointer(event, _) => event.meta_key(),
      Wheel(event, _) => event.meta_key(),
    }
  }
}

#[derive(TypedBuilder, Default)]
pub struct PressProps {
  /// Handler that is called when the press is released over the target.
  #[builder(default, setter(into, strip_option))]
  pub on_press: Option<Box<dyn Fn(&PressEvent)>>,

  /// Handler that is called when a press interaction starts.
  #[builder(default, setter(into, strip_option))]
  pub on_press_start: Option<Box<dyn Fn(&PressEvent)>>,

  /// Handler that is called when a press interaction ends, either over the
  /// target or when the pointer leaves the target.
  #[builder(default, setter(into, strip_option))]
  pub on_press_end: Option<Box<dyn Fn(&PressEvent)>>,

  /// Handler that is called when the press state changes.
  #[builder(default, setter(into, strip_option))]
  pub on_press_change: Option<Box<dyn Fn(bool)>>,

  /// Handler that is called when a press is released over the target,
  /// regardless of whether it started on the target or not.
  #[builder(default, setter(into, strip_option))]
  pub on_press_up: Option<Box<dyn Fn(&PressEvent)>>,

  /// Whether the target is in a controlled press state (e.g. an overlay it
  /// triggers is open).
  #[builder(default, setter(strip_option))]
  pub is_pressed: Option<bool>,

  /// Whether the press events should be disabled.
  #[builder(default, setter(strip_option))]
  pub is_disabled: Option<bool>,

  /// Whether the target should not receive focus on press.
  #[builder(default, setter(strip_option))]
  pub prevent_focus_on_press: Option<bool>,

  /// Whether press events should be canceled when the pointer leaves the target
  /// while pressed. By default, this is `false`, which means if the pointer
  /// returns back over the target while still pressed, onPressStart will be
  /// fired again. If set to `true`, the press is canceled when the pointer
  /// leaves the target and onPressStart will not be fired if the pointer
  /// returns.
  #[builder(default, setter(strip_option))]
  pub should_cancel_on_pointer_exit: Option<bool>,

  /// Whether text selection should be enabled on the pressable element.
  #[builder(default, setter(strip_option))]
  pub allow_text_selection_on_press: Option<bool>,
}

#[derive(TypedBuilder)]
pub struct UsePressProps {
  /// Handler that is called when the press is released over the target.
  #[builder(default, setter(into, strip_option))]
  pub on_press: Option<Box<dyn Fn(&PressEvent)>>,

  /// Handler that is called when a press interaction starts.
  #[builder(default, setter(into, strip_option))]
  pub on_press_start: Option<Box<dyn Fn(&PressEvent)>>,

  /// Handler that is called when a press interaction ends, either over the
  /// target or when the pointer leaves the target.
  #[builder(default, setter(into, strip_option))]
  pub on_press_end: Option<Box<dyn Fn(&PressEvent)>>,

  /// Handler that is called when the press state changes.
  #[builder(default, setter(into, strip_option))]
  pub on_press_change: Option<Box<dyn Fn(bool)>>,

  /// Handler that is called when a press is released over the target,
  /// regardless of whether it started on the target or not.
  #[builder(default, setter(into, strip_option))]
  pub on_press_up: Option<Box<dyn Fn(&PressEvent)>>,

  /// Whether the target is in a controlled press state (e.g. an overlay it
  /// triggers is open).
  #[builder(default, setter(strip_option, into))]
  pub is_pressed: Option<MaybeSignal<bool>>,

  /// Whether the press events should be disabled.
  #[builder(default, setter(strip_option, into))]
  pub is_disabled: Option<MaybeSignal<bool>>,

  /// Whether the target should not receive focus on press.
  #[builder(default, setter(strip_option, into))]
  pub prevent_focus_on_press: Option<MaybeSignal<bool>>,

  /// Whether press events should be canceled when the pointer leaves the target
  /// while pressed. By default, this is `false`, which means if the pointer
  /// returns back over the target while still pressed, onPressStart will be
  /// fired again. If set to `true`, the press is canceled when the pointer
  /// leaves the target and onPressStart will not be fired if the pointer
  /// returns.
  #[builder(default, setter(strip_option, into))]
  pub should_cancel_on_pointer_exit: Option<MaybeSignal<bool>>,

  /// Whether text selection should be enabled on the pressable element.
  #[builder(default, setter(strip_option, into))]
  pub allow_text_selection_on_press: Option<MaybeSignal<bool>>,

  /// The children of this provider.
  // pub children: Box<dyn FnOnce(Scope) -> Fragment>,

  /// The ref.
  #[builder(setter(into))]
  pub _ref: NodeRef<AnyElement>,
}

#[derive(TypedBuilder, Clone, Debug)]
pub struct PressEvent {
  /// The type of press event being fired.
  pub event_type: PressEventType,

  /// The pointer type that triggered the press event.
  pub pointer_type: PointerType,

  /// The target element of the press event.
  ///
  /// Get the element from the target as shown here:
  /// https://users.rust-lang.org/t/get-element-from-web-sys-eventtarget/44925
  pub target: Element,

  /// Whether the shift keyboard modifier was held during the press event.
  pub shift_key: bool,

  /// Whether the ctrl keyboard modifier was held during the press event.
  pub ctrl_key: bool,

  /// Whether the meta keyboard modifier was held during the press event.
  pub meta_key: bool,

  /// Whether the alt keyboard modifier was held during the press event.
  pub alt_key: bool,
}

impl PressEvent {
  pub fn create(
    pointer_type: &PointerType,
    event_type: PressEventType,
    focusable_event: &FocusableEvent,
  ) -> Self {
    Self::builder()
      .event_type(event_type)
      .pointer_type(pointer_type.clone())
      .target(focusable_event.current_target())
      .shift_key(focusable_event.shift_key())
      .meta_key(focusable_event.meta_key())
      .ctrl_key(focusable_event.ctrl_key())
      .alt_key(focusable_event.alt_key())
      .build()
  }
}

#[derive(Clone, Debug)]
pub enum PressEventType {
  PressStart,
  PressEnd,
  PressUp,
  Press,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PointerType {
  Unsupported,
  Mouse,
  Pen,
  Touch,
  Keyboard,
  Virtual,
}

impl From<&str> for PointerType {
  fn from(value: &str) -> Self {
    match value {
      "mouse" => Self::Mouse,
      "pen" => Self::Pen,
      "touch" => Self::Touch,
      "keyboard" => Self::Keyboard,
      "virtual" => Self::Virtual,
      _ => Self::Unsupported,
    }
  }
}

impl From<String> for PointerType {
  fn from(value: String) -> Self {
    Self::from(value.as_str())
  }
}

impl From<PointerEvent> for PointerType {
  fn from(value: PointerEvent) -> Self {
    Self::from(value.pointer_type())
  }
}
