use std::rc::Rc;

use leptos::prelude::*;
use leptos_aria_utils::InteractionModality;
use leptos_aria_utils::interaction_modality;
use typed_builder::TypedBuilder;
use web_sys::FocusEvent;

use crate::use_focus::UseFocusProps;
use crate::use_focus::use_focus;
use crate::use_focus_within::UseFocusWithinProps;
use crate::use_focus_within::use_focus_within;

/// The props accepted by [`use_focus_ring`].
#[derive(TypedBuilder, Default)]
pub struct UseFocusRingProps {
	/// Whether the element should be treated as focused on mount.
	#[builder(default, setter(strip_option))]
	pub auto_focus: Option<bool>,

	/// Whether the target is a text input, whose focus is not highlighted with
	/// a ring for a virtual (assistive technology) modality.
	#[builder(default, setter(strip_option))]
	pub is_text_input: Option<bool>,

	/// Whether the focus ring tracks focus on the target's subtree rather than
	/// the target itself.
	#[builder(default, setter(strip_option))]
	pub within: Option<bool>,
}

/// The props returned by [`use_focus_ring`].
#[derive(Clone)]
pub struct FocusRingResult {
	/// Whether the target is currently focused.
	pub is_focused: Signal<bool>,

	/// Whether the focus ring should be visible: focused through the keyboard
	/// or an assistive technology.
	pub is_focus_visible: Signal<bool>,

	pub on_focus: Rc<dyn Fn(&FocusEvent)>,
	pub on_blur: Rc<dyn Fn(&FocusEvent)>,
}

/// Determines whether a focus ring should be shown to indicate keyboard
/// focus.
///
/// Focus rings are visible only when the user is interacting with a keyboard
/// or assistive technology, not with a mouse, touch, or other pointer input.
#[allow(clippy::needless_pass_by_value)]
pub fn use_focus_ring(props: UseFocusRingProps) -> FocusRingResult {
	let is_text_input = props.is_text_input.unwrap_or_default();
	let within = props.within.unwrap_or_default();

	// Autofocus shows the ring until a pointer interaction says otherwise,
	// matching the upstream initial state.
	let auto_focus_visible =
		props.auto_focus.unwrap_or_default() || leptos_aria_utils::is_focus_visible();

	let is_focused = RwSignal::new(false);
	let focus_visible = RwSignal::new(false);
	focus_visible.set(auto_focus_visible);

	let on_focus_change = Box::new(move |focused: bool| {
		is_focused.set(focused);

		focus_visible.set(focused && visible_for_modality(interaction_modality(), is_text_input));
	});

	if within {
		let result = use_focus_within(UseFocusWithinProps {
			on_focus_within_change: Some(on_focus_change),
			..Default::default()
		});

		return FocusRingResult {
			is_focused: Signal::derive(move || is_focused.get()),
			is_focus_visible: Signal::derive(move || focus_visible.get()),
			on_focus: result.on_focus,
			on_blur: result.on_blur,
		};
	}

	let result = use_focus(UseFocusProps {
		on_focus_change: Some(on_focus_change),
		..Default::default()
	});

	FocusRingResult {
		is_focused: Signal::derive(move || is_focused.get()),
		is_focus_visible: Signal::derive(move || focus_visible.get()),
		on_focus: result.on_focus,
		on_blur: result.on_blur,
	}
}

/// Keyboard focus always rings; a virtual cursor (screen reader) rings for
/// everything except text inputs, whose caret is their own indicator.
fn visible_for_modality(modality: InteractionModality, is_text_input: bool) -> bool {
	match modality {
		InteractionModality::Keyboard => true,
		InteractionModality::Virtual => !is_text_input,
		InteractionModality::Pointer | InteractionModality::None => false,
	}
}
