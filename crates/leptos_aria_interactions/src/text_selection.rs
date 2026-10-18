use std::cell::Cell;
use std::cell::RefCell;
use std::time::Duration;

use js_sys::JsString;
use leptos::prelude::*;
use leptos_aria_utils::Map;
use leptos_aria_utils::is_ios;
use leptos_aria_utils::run_after_transition;
use wasm_bindgen::JsCast;
use web_sys::Element;
use web_sys::HtmlElement;
use web_sys::SvgElement;

/// Safari on iOS starts selecting text on long press. The only way to avoid
/// this, it seems, is to add `user-select: none` to the entire page. Adding it
/// to the pressable element prevents that element from being selected, but
/// nearby elements may still receive selection. `user-select: none` is added
/// on touch start, and removed again on touch end to prevent this. This must
/// be implemented using global state to avoid race conditions between multiple
/// elements.
///
/// There are three possible states due to the delay before removing
/// `user-select: none` after pointer up. The `Default` state always
/// transitions to the `Disabled` state, which transitions to `Restoring`. The
/// `Restoring` state can either transition back to `Disabled` or `Default`.
///
/// For non-iOS devices, `user-select: none` is applied to the pressed element
/// instead, to avoid possible performance issues that arise from applying and
/// removing `user-select: none` on the entire page; see
/// <https://github.com/adobe/react-spectrum/issues/1609>.
#[derive(Copy, Clone, PartialEq, Eq, Default)]
enum Selection {
	#[default]
	Default,
	Disabled,
	Restoring,
}

/// There appears to be a delay on iOS where selection still might occur after
/// pointer up, so restoring waits a bit before removing user-select.
const SELECTION_RESTORE_DELAY: Duration = Duration::from_millis(300);

thread_local! {
	static SELECTION: Cell<Selection> = const { Cell::new(Selection::Default) };
	static SAVED_USER_SELECT: RefCell<Option<String>> = const { RefCell::new(None) };
	static MODIFIED_ELEMENTS: RefCell<Map<Element, JsString>> = RefCell::default();
}

fn document() -> web_sys::Document {
	web_sys::window()
		.expect("the `window` global is available in the browser")
		.document()
		.expect("the `document` global is available in the browser")
}

/// Disable text selection for the duration of a press interaction.
///
/// On iOS this applies `user-select: none` to the entire document, and on
/// other platforms only to the pressed `target`.
pub(crate) fn disable_text_selection(target: Option<&Element>) {
	if is_ios() {
		disable_text_selection_on_ios();
	} else if let Some(target) = target.filter(|element| is_stylable_element(element)) {
		// If not iOS, store the target's original user-select and change to
		// `user-select: none`. The shared state does not apply outside iOS.
		let style = target.unchecked_ref::<HtmlElement>().style();
		let user_select: JsString = style
			.get_property_value("user-select")
			.unwrap_or_default()
			.into();

		MODIFIED_ELEMENTS.with_borrow_mut(|elements| {
			elements.set(target, &user_select);
		});

		style.set_property("user-select", "none").ok();
	}
}

/// Only `HTMLElement`s and `SVGElement`s accept styling.
fn is_stylable_element(element: &Element) -> bool {
	element.is_instance_of::<HtmlElement>() || element.is_instance_of::<SvgElement>()
}

/// Apply `user-select: none` to the whole document, remembering what it was.
fn disable_text_selection_on_ios() {
	if SELECTION.get() == Selection::Default {
		let style = document_element_style();
		let saved = style.get_property_value("-webkit-user-select").ok();

		SAVED_USER_SELECT.with_borrow_mut(|slot| *slot = saved);

		style.set_property("-webkit-user-select", "none").ok();
	}

	SELECTION.set(Selection::Disabled);
}

/// Restore text selection disabled by [`disable_text_selection`].
pub(crate) fn restore_text_selection(target: impl AsRef<Element>) {
	let target = target.as_ref();

	if is_ios() {
		restore_text_selection_on_ios();
	} else if is_stylable_element(target) {
		// If not iOS, restore the target's original user-select if any. The
		// shared state does not apply outside iOS.
		restore_element_text_selection(target);
	}
}

/// Restore the target's original `user-select` and drop the empty `style`
/// attribute it may have left behind.
fn restore_element_text_selection(target: &Element) {
	MODIFIED_ELEMENTS.with_borrow_mut(|elements| {
		let Some(found_selection) = elements.get(target) else {
			return;
		};

		let style = target.unchecked_ref::<HtmlElement>().style();

		if style.get_property_value("user-select").ok().as_deref() == Some("none") {
			style
				.set_property("user-select", &String::from(found_selection))
				.ok();
		}

		if target
			.get_attribute("style")
			.as_deref()
			.is_none_or(str::is_empty)
		{
			target.remove_attribute("style").ok();
		}

		elements.delete(target);
	});
}

/// Queue the page-wide restore on iOS.
///
/// If the state is already default there's nothing to do, and if it is
/// restoring there's no need to queue a second restore.
fn restore_text_selection_on_ios() {
	if SELECTION.get() != Selection::Disabled {
		return;
	}

	SELECTION.set(Selection::Restoring);

	set_timeout(
		|| {
			// Wait for any CSS transitions to complete so recomputing the
			// style for the whole page does not happen in the middle of an
			// animation and cause jank.
			run_after_transition(|| {
				// Avoid race conditions.
				if SELECTION.get() != Selection::Restoring {
					return;
				}

				restore_document_user_select();

				SELECTION.set(Selection::Default);
			});
		},
		SELECTION_RESTORE_DELAY,
	);
}

/// Put back the saved user-select value when the document still has ours.
fn restore_document_user_select() {
	let style = document_element_style();

	if style
		.get_property_value("-webkit-user-select")
		.ok()
		.as_deref()
		!= Some("none")
	{
		return;
	}

	let saved = SAVED_USER_SELECT
		.with_borrow_mut(std::mem::take)
		.unwrap_or_default();

	style.set_property("-webkit-user-select", &saved).ok();
}

fn document_element_style() -> web_sys::CssStyleDeclaration {
	document()
		.document_element()
		.unwrap()
		.unchecked_into::<HtmlElement>()
		.style()
}
