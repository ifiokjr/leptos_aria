use std::rc::Rc;

use leptos::create_rw_signal;
use leptos::typed_builder::TypedBuilder;
use leptos::web_sys::DragEvent;
use leptos::web_sys::Element;
use leptos::web_sys::HtmlAnchorElement;
use leptos::web_sys::HtmlElement;
use leptos::web_sys::HtmlInputElement;
use leptos::web_sys::HtmlTextAreaElement;
use leptos::web_sys::KeyboardEvent;
use leptos::web_sys::MouseEvent;
use leptos::web_sys::PointerEvent;
use leptos::web_sys::SvgElement;
use leptos::web_sys::TouchEvent;
use leptos::web_sys::WheelEvent;
use leptos::IntoSignal;
use leptos::JsCast;
use leptos::MaybeSignal;
use leptos::NodeRef;
use leptos::RwSignal;
use leptos::Scope;
use leptos::Signal;
use leptos::UntrackedGettableSignal;
use leptos::UntrackedSettableSignal;
use leptos_aria_utils::GlobalListeners;

pub fn use_press(cx: Scope, props: UsePressProps) -> PressResult {
  PressResult::new(cx, props)
}

type OnPressCallback = Rc<Box<dyn Fn(&PressEvent)>>;
type OnPressChangeCallback = Rc<Box<dyn Fn(bool)>>;

pub struct PressResult {
  listeners: GlobalListeners,
  ignore_emulated_mouse_event: RwSignal<bool>,
  ignore_click_after_press: RwSignal<bool>,
  did_fire_press_start: RwSignal<bool>,
  active_pointer_id: RwSignal<Option<u32>>,
  target: RwSignal<Option<Element>>,
  is_over_target: RwSignal<bool>,
  pointer_type: RwSignal<PointerType>,
  user_select: RwSignal<Option<String>>,
  wrapped_is_pressed: RwSignal<bool>,
  pub is_pressed: Signal<bool>,
  pub is_disabled: Signal<bool>,
  pub prevent_focus_on_press: Signal<bool>,
  pub should_cancel_on_pointer_exit: Signal<bool>,
  pub allow_text_selection_on_press: Signal<bool>,
  wrapped_on_press: Option<OnPressCallback>,
  wrapped_on_press_start: Option<OnPressCallback>,
  wrapped_on_press_end: Option<OnPressCallback>,
  wrapped_on_press_change: Option<OnPressChangeCallback>,
  wrapped_on_press_up: Option<OnPressCallback>,
}

impl PressResult {
  fn new(cx: Scope, props: UsePressProps) -> Self {
    let listeners = Default::default();
    let ignore_emulated_mouse_event = create_rw_signal(cx, false);
    let ignore_click_after_press = create_rw_signal(cx, false);
    let did_fire_press_start = create_rw_signal(cx, false);
    let active_pointer_id = create_rw_signal(cx, None);
    let target = create_rw_signal(cx, None);
    let is_over_target = create_rw_signal(cx, false);
    let pointer_type = create_rw_signal(cx, PointerType::Unsupported);
    let user_select = create_rw_signal(cx, None);

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

    let wrapped_on_press = props.on_press.map(Rc::new);
    let wrapped_on_press_start = props.on_press_start.map(Rc::new);
    let wrapped_on_press_end = props.on_press_end.map(Rc::new);
    let wrapped_on_press_change = props.on_press_change.map(Rc::new);
    let wrapped_on_press_up = props.on_press_up.map(Rc::new);

    let wrapped_is_pressed = create_rw_signal(cx, false);
    let original_is_pressed = props.is_pressed.unwrap_or(false.into());
    let is_pressed =
      (move || original_is_pressed.get() || wrapped_is_pressed.get()).derive_signal(cx);

    Self {
      listeners,
      ignore_emulated_mouse_event,
      ignore_click_after_press,
      did_fire_press_start,
      active_pointer_id,
      target,
      is_over_target,
      pointer_type,
      user_select,
      is_pressed,
      is_disabled,
      prevent_focus_on_press,
      should_cancel_on_pointer_exit,
      allow_text_selection_on_press,
      wrapped_is_pressed,
      wrapped_on_press,
      wrapped_on_press_start,
      wrapped_on_press_end,
      wrapped_on_press_change,
      wrapped_on_press_up,
    }
  }

  /// Trigger the beginning of a custom press event.
  fn trigger_press_start(&self, focusable_event: FocusableEvent, pointer: PointerType) {
    if self.is_disabled.get() || self.did_fire_press_start.get_untracked() {
      return;
    }

    let event = PressEvent::create(&pointer, PressEventType::PressStart, &focusable_event);
    call_event(&self.wrapped_on_press_start, &event);
    call_event(&self.wrapped_on_press_change, true);

    self.did_fire_press_start.set_untracked(true);
    self.wrapped_is_pressed.set(true);
  }

  fn trigger_press_end(
    &self,
    focusable_event: FocusableEvent,
    pointer: PointerType,
    was_pressed: bool,
  ) {
    if !self.did_fire_press_start.get_untracked() {
      return;
    }

    self.ignore_click_after_press.set_untracked(true);
    self.did_fire_press_start.set_untracked(false);

    let event = PressEvent::create(&pointer, PressEventType::PressEnd, &focusable_event);
    call_event(&self.wrapped_on_press_end, &event);
    call_event(&self.wrapped_on_press_change, false);

    self.wrapped_is_pressed.set(false);

    if !was_pressed || self.is_disabled.get() {
      return;
    }

    let event = PressEvent::create(&pointer, PressEventType::Press, &focusable_event);
    call_event(&self.wrapped_on_press, &event);
  }

  fn trigger_press_up(&self, focusable_event: FocusableEvent, pointer: PointerType) {
    if self.is_disabled.get() {
      return;
    }

    let event = PressEvent::create(&pointer, PressEventType::PressUp, &focusable_event);
    call_event(&self.wrapped_on_press_up, &event);
  }

  fn cancel(&mut self, focusable_event: FocusableEvent) {
    if !self.is_pressed.get_untracked() {
      return;
    }

    if self.is_over_target.get_untracked() {
      self.trigger_press_end(focusable_event, self.pointer_type.get_untracked(), false);
    }

    self.wrapped_is_pressed.set_untracked(false);
    self.is_over_target.set_untracked(false);
    self.active_pointer_id.set_untracked(None);
    self.pointer_type.set_untracked(PointerType::Unsupported);

    self.listeners.remove_all_listeners();

    if !self.allow_text_selection_on_press.get() {}
    // if !self.
  }

  // pub fn on_press_start(&self, event: PressEvent) {}
}

fn call_event<T>(callback: &Option<Rc<Box<dyn Fn(T)>>>, event: T) {
  if let Some(callback) = callback.as_ref() {
    let cb = callback.clone();
    (cb)(event);
  }
}

fn is_valid_keyboard_event(event: &KeyboardEvent, current_target: &Element) -> bool {
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

fn is_html_anchor_link(target: &HtmlElement) -> bool {
  target.is_instance_of::<HtmlAnchorElement>()
    || (target.tag_name() == "A" && target.has_attribute("href"))
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

pub trait ToFocusableElement {
  /// Converts the type to a FocusableElement enum.
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
  pub _ref: NodeRef<leptos::HtmlElement<leptos::AnyElement>>,
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

#[derive(Clone, Debug)]
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
