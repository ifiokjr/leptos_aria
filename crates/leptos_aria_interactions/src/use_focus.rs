use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;
use leptos_aria_utils::FocusableElement;
use leptos_aria_utils::GlobalListeners;
use leptos_aria_utils::ToFocusableElement;
use typed_builder::TypedBuilder;
use wasm_bindgen::JsCast;
use web_sys::Element;
use web_sys::HtmlButtonElement;
use web_sys::HtmlInputElement;
use web_sys::HtmlSelectElement;
use web_sys::HtmlTextAreaElement;

/// A focus event carrying an explicit [`FocusableElement`] target, so handlers
/// see the tracked element even for synthesized events.
#[derive(Clone)]
pub struct FocusEvent {
	/// The element receiving or losing focus.
	pub target: FocusableElement,

	/// The element focus is moving to, for blur events.
	pub related_target: Option<FocusableElement>,
}

impl FocusEvent {
	/// Resolve the tracked elements of a native focus event.
	pub(crate) fn from_native(event: &web_sys::FocusEvent) -> Self {
		let target = focus_target(event)
			.expect("focus events carry a target")
			.to_focusable_element();

		Self {
			target,
			related_target: event
				.related_target()
				.and_then(|target| target.dyn_into::<Element>().ok())
				.map(|element| element.to_focusable_element()),
		}
	}
}

/// A callback invoked for a [`FocusEvent`].
pub type BoxedFocusEventCallback = Box<dyn Fn(&FocusEvent)>;

/// A callback invoked when the focus state changes.
pub type BoxedFocusChangeCallback = Box<dyn Fn(bool)>;

pub(crate) type FocusEventCallback = Rc<dyn Fn(&FocusEvent)>;
pub(crate) type FocusChangeCallback = Rc<dyn Fn(bool)>;

/// The props accepted by [`use_focus`].
#[derive(TypedBuilder, Default)]
pub struct UseFocusProps {
	/// Handler that is called when the target receives focus.
	#[builder(default, setter(strip_option))]
	pub on_focus: Option<BoxedFocusEventCallback>,

	/// Handler that is called when the target loses focus.
	#[builder(default, setter(strip_option))]
	pub on_blur: Option<BoxedFocusEventCallback>,

	/// Handler that is called when the focus state changes.
	#[builder(default, setter(strip_option))]
	pub on_focus_change: Option<BoxedFocusChangeCallback>,

	/// Whether the focus events should be disabled.
	#[builder(default, setter(strip_option, into))]
	pub is_disabled: Option<Signal<bool>>,
}

/// The props returned by [`use_focus`] which should be spread onto the target
/// element.
#[derive(Clone)]
pub struct FocusResult {
	pub on_focus: Rc<dyn Fn(&web_sys::FocusEvent)>,
	pub on_blur: Rc<dyn Fn(&web_sys::FocusEvent)>,
}

/// Handles focus events for the immediate target.
///
/// Focus events on child elements are ignored, and a blur is synthesized when
/// a focused form control becomes disabled — the browser fires no blur event
/// in that case in some browsers, so the handler would otherwise never run.
pub fn use_focus(props: UseFocusProps) -> FocusResult {
	let state = Rc::new(FocusState::new(props));

	FocusResult {
		on_focus: focus_handler(&state, FocusState::on_focus),
		on_blur: focus_handler(&state, FocusState::on_blur),
	}
}

/// Wrap a [`FocusState`] handler method as a cloneable event callback.
fn focus_handler(
	state: &Rc<FocusState>,
	method: fn(&Rc<FocusState>, &web_sys::FocusEvent),
) -> Rc<dyn Fn(&web_sys::FocusEvent)> {
	let state = Rc::clone(state);

	Rc::new(move |event| method(&state, event))
}

/// The state shared by the focus handlers of one hook instance.
struct FocusState {
	listeners: Rc<RefCell<GlobalListeners>>,
	is_disabled: Signal<bool>,
	on_focus: Option<FocusEventCallback>,
	on_blur: Option<FocusEventCallback>,
	on_focus_change: Option<FocusChangeCallback>,
}

impl FocusState {
	fn new(props: UseFocusProps) -> Self {
		Self {
			listeners: Rc::new(RefCell::new(GlobalListeners::default())),
			is_disabled: props.is_disabled.unwrap_or_else(|| Signal::from(false)),
			on_focus: props.on_focus.map(wrap_focus_callback),
			on_blur: props.on_blur.map(wrap_focus_callback),
			on_focus_change: props.on_focus_change.map(wrap_change_callback),
		}
	}

	/// The focus handler: only the immediate target counts, and only while it
	/// is still the active element.
	fn on_focus(state: &Rc<Self>, event: &web_sys::FocusEvent) {
		if state.is_disabled.get_untracked() {
			return;
		}

		if event.target() != event.current_target() || !is_active_element(event) {
			return;
		}

		call_focus_event(state.on_focus.as_ref(), &FocusEvent::from_native(event));
		call_change_event(state.on_focus_change.as_ref(), true);

		Self::watch_for_disabled_blur(state, event);
	}

	/// The blur handler: only the immediate target counts.
	fn on_blur(state: &Rc<Self>, event: &web_sys::FocusEvent) {
		if state.is_disabled.get_untracked() {
			return;
		}

		if event.target() != event.current_target() {
			return;
		}

		state.trigger_blur(event);
	}

	fn trigger_blur(&self, event: &web_sys::FocusEvent) {
		call_focus_event(self.on_blur.as_ref(), &FocusEvent::from_native(event));
		call_change_event(self.on_focus_change.as_ref(), false);
	}

	/// Watch for the target becoming disabled while focused.
	///
	/// When a focused form control becomes disabled the browser fires no blur
	/// event in some browsers, so a `focusout` listener synthesizes one and
	/// removes itself again.
	fn watch_for_disabled_blur(state: &Rc<Self>, event: &web_sys::FocusEvent) {
		let Some(target) = focus_target(event) else {
			return;
		};

		if !tracks_disable_blur(&target) {
			return;
		}

		let listener_state = Rc::clone(state);
		let tracked_target = target.clone();

		state.listeners.borrow_mut().add::<web_sys::FocusEvent, _>(
			&target,
			"focusout",
			move |focus_out| {
				listener_state.listeners.borrow_mut().remove_all_listeners();

				if is_disabled_element(&tracked_target) {
					listener_state.trigger_blur(&focus_out);
				}
			},
			false,
		);
	}
}

pub(crate) fn wrap_focus_callback(callback: BoxedFocusEventCallback) -> FocusEventCallback {
	Rc::new(move |event: &FocusEvent| callback(event))
}

pub(crate) fn wrap_change_callback(callback: BoxedFocusChangeCallback) -> FocusChangeCallback {
	Rc::new(move |focused: bool| callback(focused))
}

pub(crate) fn call_focus_event(callback: Option<&FocusEventCallback>, event: &FocusEvent) {
	if let Some(callback) = callback {
		callback(event);
	}
}

pub(crate) fn call_change_event(callback: Option<&FocusChangeCallback>, focused: bool) {
	if let Some(callback) = callback {
		callback(focused);
	}
}

fn focus_target(event: &web_sys::FocusEvent) -> Option<Element> {
	event
		.target()
		.and_then(|target| target.dyn_into::<Element>().ok())
}

/// Whether the event target is the document's active element.
fn is_active_element(event: &web_sys::FocusEvent) -> bool {
	focus_target(event).is_some_and(|target| {
		document()
			.active_element()
			.is_some_and(|active| active == target)
	})
}

fn document() -> web_sys::Document {
	web_sys::window()
		.expect("the `window` global is available in the browser")
		.document()
		.expect("the `document` global is available in the browser")
}

/// Only form controls lose focus when disabled.
fn tracks_disable_blur(target: &Element) -> bool {
	target.is_instance_of::<HtmlButtonElement>()
		|| target.is_instance_of::<HtmlInputElement>()
		|| target.is_instance_of::<HtmlTextAreaElement>()
		|| target.is_instance_of::<HtmlSelectElement>()
}

fn is_disabled_element(element: &Element) -> bool {
	is_disabled_as::<HtmlButtonElement>(element)
		|| is_disabled_as::<HtmlInputElement>(element)
		|| is_disabled_as::<HtmlTextAreaElement>(element)
		|| is_disabled_as::<HtmlSelectElement>(element)
}

fn is_disabled_as<T>(element: &Element) -> bool
where
	T: JsCast + Disabled,
{
	element.dyn_ref::<T>().is_some_and(T::disabled)
}

/// The `disabled` property shared by the form controls.
trait Disabled {
	fn disabled(&self) -> bool;
}

impl Disabled for HtmlButtonElement {
	fn disabled(&self) -> bool {
		HtmlButtonElement::disabled(self)
	}
}

impl Disabled for HtmlInputElement {
	fn disabled(&self) -> bool {
		HtmlInputElement::disabled(self)
	}
}

impl Disabled for HtmlTextAreaElement {
	fn disabled(&self) -> bool {
		HtmlTextAreaElement::disabled(self)
	}
}

impl Disabled for HtmlSelectElement {
	fn disabled(&self) -> bool {
		HtmlSelectElement::disabled(self)
	}
}
