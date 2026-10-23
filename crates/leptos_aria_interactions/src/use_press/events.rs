use std::rc::Rc;

use leptos_aria_utils::FocusableElement;
use leptos_aria_utils::ToFocusableElement;
use typed_builder::TypedBuilder;
use wasm_bindgen::JsCast;
use web_sys::DragEvent;
use web_sys::Element;
use web_sys::KeyboardEvent;
use web_sys::MouseEvent;
use web_sys::PointerEvent;
use web_sys::WheelEvent;

/// A callback invoked for a [`PressEvent`].
pub type BoxedPressEventCallback = Box<dyn Fn(&PressEvent)>;

/// A callback invoked when the press state changes.
pub type BoxedPressChangeCallback = Box<dyn Fn(bool)>;

pub(crate) type PressEventCallback = Rc<dyn Fn(&PressEvent)>;
pub(crate) type PressChangeCallback = Rc<dyn Fn(bool)>;
pub(crate) type PressCallback<E> = Rc<dyn Fn(E)>;

pub(crate) fn wrap_press_event_callback(callback: BoxedPressEventCallback) -> PressEventCallback {
	Rc::new(move |event: &PressEvent| callback(event))
}

pub(crate) fn wrap_change_callback(callback: BoxedPressChangeCallback) -> PressChangeCallback {
	Rc::new(move |pressed: bool| callback(pressed))
}

pub(crate) fn call_event(callback: Option<&PressEventCallback>, event: &PressEvent) {
	if let Some(callback) = callback {
		callback(event);
	}
}

pub(crate) fn call_change_event(callback: Option<&PressChangeCallback>, pressed: bool) {
	if let Some(callback) = callback {
		callback(pressed);
	}
}

/// Any event that can be pressed.
#[derive(Clone)]
pub enum FocusableEvent {
	Mouse(MouseEvent, Option<FocusableElement>),
	Keyboard(KeyboardEvent, Option<FocusableElement>),
	Touch(web_sys::TouchEvent, Option<FocusableElement>),
	Drag(DragEvent, Option<FocusableElement>),
	Pointer(PointerEvent, Option<FocusableElement>),
	Wheel(WheelEvent, Option<FocusableElement>),
}

/// Match the wrapped event of a [`FocusableEvent`] and run the same expression
/// against each variant.
macro_rules! on_wrapped_event {
	($self:expr, $event:ident => $body:expr) => {
		match $self {
			FocusableEvent::Mouse($event, _) => $body,
			FocusableEvent::Keyboard($event, _) => $body,
			FocusableEvent::Touch($event, _) => $body,
			FocusableEvent::Drag($event, _) => $body,
			FocusableEvent::Pointer($event, _) => $body,
			FocusableEvent::Wheel($event, _) => $body,
		}
	};
}

impl FocusableEvent {
	pub fn focusable_target(&self) -> FocusableElement {
		match self {
			FocusableEvent::Mouse(_, element)
			| FocusableEvent::Keyboard(_, element)
			| FocusableEvent::Touch(_, element)
			| FocusableEvent::Drag(_, element)
			| FocusableEvent::Pointer(_, element)
			| FocusableEvent::Wheel(_, element) => {
				element
					.clone()
					.unwrap_or_else(|| self.current_target().to_focusable_element())
			}
		}
	}

	pub fn current_target(&self) -> Element {
		let explicit = match self {
			Self::Mouse(_, element)
			| Self::Keyboard(_, element)
			| Self::Touch(_, element)
			| Self::Drag(_, element)
			| Self::Pointer(_, element)
			| Self::Wheel(_, element) => element,
		};

		// Global release listeners run on Document. Their explicit target is
		// the element that started the press, including when focus has moved.
		if let Some(element) = explicit {
			return element.as_element().clone();
		}

		on_wrapped_event!(self, event => event
			.current_target()
			.and_then(|target| target.dyn_into::<Element>().ok())
			.expect("the current target is available during event dispatch"))
	}

	pub fn shift_key(&self) -> bool {
		on_wrapped_event!(self, event => event.shift_key())
	}

	pub fn ctrl_key(&self) -> bool {
		on_wrapped_event!(self, event => event.ctrl_key())
	}

	pub fn alt_key(&self) -> bool {
		on_wrapped_event!(self, event => event.alt_key())
	}

	pub fn meta_key(&self) -> bool {
		on_wrapped_event!(self, event => event.meta_key())
	}
}

/// The four keyboard-modifier flags mirror the upstream `react-aria` event
/// shape, so the struct intentionally carries more than three booleans.
#[allow(clippy::struct_excessive_bools)]
#[derive(TypedBuilder, Clone, Debug)]
pub struct PressEvent {
	/// The type of press event being fired.
	pub event_type: PressEventType,

	/// The pointer type that triggered the press event.
	pub pointer_type: PointerType,

	/// The target element of the press event.
	///
	/// Get the element from the target as shown here:
	/// <https://users.rust-lang.org/t/get-element-from-web-sys-eventtarget/44925>
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

impl AsRef<PressEvent> for PressEvent {
	fn as_ref(&self) -> &PressEvent {
		self
	}
}

impl PressEvent {
	pub fn create(
		pointer_type: &PointerType,
		event_type: PressEventType,
		focusable_event: &FocusableEvent,
	) -> Self {
		Self::builder()
			.event_type(event_type)
			.pointer_type(*pointer_type)
			.target(focusable_event.current_target())
			.shift_key(focusable_event.shift_key())
			.meta_key(focusable_event.meta_key())
			.ctrl_key(focusable_event.ctrl_key())
			.alt_key(focusable_event.alt_key())
			.build()
	}
}

#[derive(Copy, Clone, Debug)]
pub enum PressEventType {
	PressStart,
	PressEnd,
	PressUp,
	Press,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PointerType {
	#[default]
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
