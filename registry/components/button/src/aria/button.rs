//! Copy of `leptos_aria_button::Button` with styling hooks: every prop is
//! forwarded so your Tailwind classes own the look. Edit freely — this file
//! is yours.

use leptos::prelude::*;
use leptos_aria::PressEvent;
use leptos_aria::UsePressProps;
use leptos_aria::use_press;

/// A callback invoked for a [`PressEvent`].
pub type BoxedPressEventCallback = Box<dyn Fn(&PressEvent)>;

/// Your button. Style it with Tailwind:
///
/// ```rust ignore
/// <Button class="rounded-md px-4 py-2 data-[disabled]:opacity-50">
///     "Save"
/// </Button>
/// ```
#[component]
pub fn Button(
	/// Your classes, spread onto the element.
	#[prop(optional, into)]
	class: Option<String>,
	/// Whether the button is disabled.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	/// Handler for the press release.
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
		<button
			type="button"
			class=class.unwrap_or_default()
			disabled=move || is_disabled.get()
			aria-disabled=move || is_disabled.get().to_string()
			data-disabled=move || is_disabled.get().to_string()
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
