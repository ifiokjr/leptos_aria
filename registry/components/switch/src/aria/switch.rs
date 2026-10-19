//! Copy of `leptos_aria_toggle::Switch` with styling hooks.

use leptos::prelude::*;
use wasm_bindgen::JsCast;

/// Your switch. Style the two states with Tailwind:
///
/// ```rust ignore
/// <Switch class="peer" thumb_class="peer-data-[state=checked]:translate-x-4">
///     "Notifications"
/// </Switch>
/// ```
#[component]
pub fn Switch(
	/// Your classes for the track.
	#[prop(optional, into)]
	class: Option<String>,
	/// Your classes for the thumb.
	#[prop(optional, into)]
	thumb_class: Option<String>,
	/// Whether the switch is checked (uncontrolled).
	#[prop(optional, into)]
	default_checked: Signal<bool>,
	/// Whether the switch is disabled.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	children: Children,
) -> impl IntoView {
	let checked = RwSignal::new(default_checked.get_untracked());
	let is_checked = checked;

	view! {
		<label class=class.unwrap_or_default() on:mousedown=|event: web_sys::MouseEvent| {
			event.prevent_default();
		}>
			<input
				type="checkbox"
				role="switch"
				class="sr-only"
				checked=move || is_checked.get()
				disabled=move || is_disabled.get()
				data-state=move || if is_checked.get() { "checked" } else { "unchecked" }
				on:change=move |event: web_sys::Event| {
					event.stop_propagation();

					let next = event
						.target()
						.and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
						.is_some_and(|input| input.checked());

					checked.set(next);
				}
			/>
			<span class=thumb_class.unwrap_or_default()></span>
			{children()}
		</label>
	}
}
