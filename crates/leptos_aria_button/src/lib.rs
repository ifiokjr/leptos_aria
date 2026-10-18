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
	use leptos::mount::mount_to_body;
	use leptos::prelude::document;
	use leptos_aria_interactions::PressEvent;
	use wasm_bindgen::JsCast;
	use wasm_bindgen_test::*;

	use super::*;

	wasm_bindgen_test_configure!(run_in_browser);

	#[component]
	fn Example() -> impl IntoView {
		let pressed = RwSignal::new(false);

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
	}
}
