#![doc(html_logo_url = "https://ifiokjr.github.io/leptos_aria/logo.svg")]
#![doc(html_favicon_url = "https://ifiokjr.github.io/leptos_aria/favicon.ico")]

use leptos::prelude::*;
pub use leptos_aria_interactions::BoxedPressEventCallback;
use leptos_aria_interactions::UsePressProps;
use leptos_aria_interactions::use_press;

/// The accessible button, providing mouse, keyboard, and touch interactions
/// plus focus handling across browsers.
///
/// This is a port of the `react-aria` `useButton` hook for the native
/// `<button>` element; the custom element types of the original are tracked
/// in the porting roadmap.
#[component]
pub fn Button(
	/// Whether the button is disabled.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	/// Handler that is called when the press is released over the target.
	#[prop(optional)]
	on_press: Option<BoxedPressEventCallback>,
	/// Handler that is called when a press interaction starts.
	#[prop(optional)]
	on_press_start: Option<BoxedPressEventCallback>,
	/// Handler that is called when a press interaction ends, either over the
	/// target or when the pointer leaves the target.
	#[prop(optional)]
	on_press_end: Option<BoxedPressEventCallback>,
	/// The behavior of the button when used in an HTML form.
	#[prop(optional, into)]
	r#type: Signal<ButtonType>,
	/// Indicates the current "pressed" state of toggle buttons.
	#[prop(optional, into)]
	aria_pressed: Option<Signal<bool>>,
	/// Indicates whether the element, or another grouping element it controls,
	/// is currently expanded or collapsed.
	#[prop(optional, into)]
	aria_expanded: Option<Signal<bool>>,
	/// Indicates the availability and type of interactive popup element that
	/// can be triggered by this element.
	#[prop(optional, into)]
	aria_haspopup: Option<Signal<String>>,
	children: Children,
) -> impl IntoView {
	let press = use_press(UsePressProps {
		on_press,
		on_press_start,
		on_press_end,
		is_disabled: Some(is_disabled),
		..Default::default()
	});

	view! {
		<button
			type=move || r#type.get().to_string()
			disabled=move || is_disabled.get()
			aria-disabled=move || is_disabled.get().to_string()
			aria-pressed=move || aria_pressed.map(|pressed| pressed.get().to_string())
			aria-expanded=move || aria_expanded.map(|expanded| expanded.get().to_string())
			aria-haspopup=move || aria_haspopup
			on:click=move |event| (press.on_click)(event)
			on:dragstart=move |event| (press.on_drag_start)(event)
			on:keydown=move |event| (press.on_key_down)(event)
			on:keyup=move |event| (press.on_key_up)(event)
			on:mousedown=move |event| (press.on_mouse_down)(event)
			on:pointerdown=move |event| (press.on_pointer_down)(event)
			on:pointerup=move |event| (press.on_pointer_up)(event)
		>
			{children()}
		</button>
	}
}

/// The behavior of the button when used in an HTML form.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum ButtonType {
	/// The button has no default behavior.
	#[default]
	Button,
	/// The button resets the form.
	Reset,
	/// The button submits the form.
	Submit,
}

impl std::fmt::Display for ButtonType {
	fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		formatter.write_str(self.as_str())
	}
}

impl ButtonType {
	fn as_str(self) -> &'static str {
		match self {
			ButtonType::Button => "button",
			ButtonType::Reset => "reset",
			ButtonType::Submit => "submit",
		}
	}
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
	use std::cell::RefCell;

	use leptos::mount::mount_to_body;
	use leptos::prelude::document;
	use leptos_aria_interactions::PressEvent;
	use wasm_bindgen::JsCast;
	use wasm_bindgen_test::*;

	use super::*;

	thread_local! {
		static PRESSED: RefCell<Option<RwSignal<bool>>> = const { RefCell::new(None) };
	}

	wasm_bindgen_test_configure!(run_in_browser);

	#[component]
	fn Example() -> impl IntoView {
		let pressed = RwSignal::new(false);
		PRESSED.with_borrow_mut(|slot| *slot = Some(pressed));

		view! {
			<Button on_press=Box::new(move |_: &PressEvent| pressed.set(true))>
				"Press me"
			</Button>
		}
	}

	#[wasm_bindgen_test]
	fn renders_a_native_button() {
		console_error_panic_hook::set_once();

		mount_to_body(|| view! { <Example /> });
		let pressed = PRESSED.with_borrow(|slot| *slot).unwrap();

		let button = document()
			.query_selector("button")
			.unwrap()
			.unwrap()
			.unchecked_into::<web_sys::HtmlButtonElement>();

		assert_eq!(
			button.get_attribute("type").as_deref(),
			Some("button"),
			"the default type is button"
		);

		button.click();

		assert!(
			pressed.get_untracked(),
			"a synthetic click fires on_press through use_press"
		);
	}
}

/// An anchor styled and behaving as a button.
///
/// The custom-element path of the upstream `useButton`: the link keeps its
/// href semantics when enabled, becomes `aria-disabled` without a href when
/// disabled, and carries the press interaction wiring.
#[component]
pub fn AnchorButton(
	/// The URL to navigate to.
	#[prop(into)]
	href: String,
	/// Whether the button is disabled: the href drops out and
	/// `aria-disabled` takes over.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	/// Handler that is called when the press is released over the target.
	#[prop(optional)]
	on_press: Option<BoxedPressEventCallback>,
	children: Children,
) -> impl IntoView {
	let press = use_press(UsePressProps {
		on_press,
		is_disabled: Some(is_disabled),
		..Default::default()
	});

	view! {
		<a
			role="button"
			tabindex="0"
			href=move || if is_disabled.get() { String::new() } else { href.clone() }
			aria-disabled=move || is_disabled.get().to_string()
			on:click=move |event| (press.on_click)(event)
			on:dragstart=move |event| (press.on_drag_start)(event)
			on:keydown=move |event| (press.on_key_down)(event)
			on:keyup=move |event| (press.on_key_up)(event)
			on:mousedown=move |event| (press.on_mouse_down)(event)
			on:pointerdown=move |event| (press.on_pointer_down)(event)
			on:pointerup=move |event| (press.on_pointer_up)(event)
		>
			{children()}
		</a>
	}
}

/// A generic element behaving as a button when no native control fits.
///
/// The custom-element path of the upstream `useButton`: a `role="button"`
/// div with the press wiring and the disabled state expressed through
/// `aria-disabled` and the tab order.
#[component]
pub fn CustomButton(
	/// Whether the button is disabled.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	/// Handler that is called when the press is released over the target.
	#[prop(optional)]
	on_press: Option<BoxedPressEventCallback>,
	children: Children,
) -> impl IntoView {
	let press = use_press(UsePressProps {
		on_press,
		is_disabled: Some(is_disabled),
		..Default::default()
	});

	view! {
		<div
			role="button"
			tabindex=move || if is_disabled.get() { -1 } else { 0 }
			aria-disabled=move || is_disabled.get().to_string()
			on:click=move |event| (press.on_click)(event)
			on:dragstart=move |event| (press.on_drag_start)(event)
			on:keydown=move |event| (press.on_key_down)(event)
			on:keyup=move |event| (press.on_key_up)(event)
			on:mousedown=move |event| (press.on_mouse_down)(event)
			on:pointerdown=move |event| (press.on_pointer_down)(event)
			on:pointerup=move |event| (press.on_pointer_up)(event)
		>
			{children()}
		</div>
	}
}
