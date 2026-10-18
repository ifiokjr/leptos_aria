use std::rc::Rc;

use leptos::prelude::*;
use typed_builder::TypedBuilder;

use crate::use_focus::UseFocusProps;
use crate::use_focus::use_focus;

/// The props accepted by [`use_focusable`].
#[derive(TypedBuilder, Default)]
pub struct UseFocusableProps {
	/// The explicit tab index, overriding the computed one.
	#[builder(default, setter(strip_option))]
	pub tab_index: Option<i32>,

	/// Whether the target is disabled: it leaves the tab order.
	#[builder(default, setter(strip_option, into))]
	pub is_disabled: Option<Signal<bool>>,
}

/// The props returned by [`use_focusable`] which should be spread onto the
/// target element.
#[derive(Clone)]
pub struct FocusableResult {
	/// The tab index for the element: the explicit one when provided, `-1`
	/// while disabled, `0` otherwise.
	pub tab_index: Signal<i32>,

	pub on_focus: Rc<dyn Fn(&web_sys::FocusEvent)>,
	pub on_blur: Rc<dyn Fn(&web_sys::FocusEvent)>,
}

/// Provides focusability for custom element types.
///
/// A disabled custom element cannot use the `disabled` attribute the way a
/// form control does, so it is removed from the tab order instead, and focus
/// handling is normalized through [`use_focus`].
#[allow(clippy::needless_pass_by_value)]
pub fn use_focusable(props: UseFocusableProps) -> FocusableResult {
	let is_disabled = props.is_disabled.unwrap_or_else(|| Signal::from(false));
	let tab_index = compute_tab_index(props.tab_index, is_disabled);

	let focus = use_focus(UseFocusProps::default());

	FocusableResult {
		tab_index,
		on_focus: focus.on_focus,
		on_blur: focus.on_blur,
	}
}

fn compute_tab_index(explicit: Option<i32>, is_disabled: Signal<bool>) -> Signal<i32> {
	Signal::derive(move || {
		match explicit {
			Some(tab_index) => tab_index,
			None if is_disabled.get() => -1,
			None => 0,
		}
	})
}
