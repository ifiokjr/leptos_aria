#![allow(clippy::needless_pass_by_value)]

mod color;

use std::sync::Arc;

pub use color::Color;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

/// A callback invoked when the color changes.
pub type BoxedColorChangeCallback = Box<dyn Fn(&Color) + Send + Sync + 'static>;

/// A color swatch shows a single color as a labeled preview.
#[component]
pub fn ColorSwatch(
	/// The color to show.
	#[prop(into)]
	color: Signal<Color>,
) -> impl IntoView {
	view! {
		<span
			class="leptos-aria-color-swatch"
			role="img"
			aria-label=move || color.get().to_hex()
			style=move || format!("background-color: {};", color.get().to_rgb_string())
		></span>
	}
}

/// A color field lets the user type a color value with a live preview.
///
/// The input parses every change through the [`Color`] value type; an
/// invalid value keeps the previous color and marks the field invalid.
#[component]
pub fn ColorField(
	/// The content to display as the label.
	#[prop(optional)]
	label: Option<String>,
	/// The current color (controlled).
	#[prop(optional, into)]
	value: Option<Signal<Color>>,
	/// The initial color (uncontrolled).
	#[prop(optional)]
	default_value: Option<Color>,
	/// Handler that is called when the color changes.
	#[prop(optional)]
	on_change: Option<BoxedColorChangeCallback>,
) -> impl IntoView {
	let initial = default_value.unwrap_or_else(|| Color::new(0.0, 0.0, 0.0, 1.0));
	let local = RwSignal::new(initial.clone());
	let input_text = RwSignal::new(initial.to_hex());

	let state = Arc::new(ColorFieldState {
		local,
		input_text,
		on_change: Arc::new(on_change),
	});

	if let Some(controlled) = value {
		let local = state.local;
		let text = state.input_text;

		Effect::new(move |_| {
			local.set(controlled.get());
			text.set(controlled.get().to_hex());
		});
	}

	let _ = &state;

	let input_state = Arc::clone(&state);
	let swatch_state = Arc::clone(&state);
	let handlers = ColorInputHandlers { state: input_state };
	let key_handlers = handlers.clone();

	let aria = leptos_aria_text_field::field_aria(label.as_deref(), None);

	view! {
		<input
			id=aria.field_id.clone()
			type="text"
			aria-labelledby=aria.aria_labelledby.clone()
			prop:value=move || state.input_text.get()
			on:input=move |event: web_sys::Event| handlers.handle_input(event)
			on:keydown=move |event: web_sys::KeyboardEvent| ColorInputHandlers::handle_key_down(&key_handlers, event)
		/>
		<ColorSwatch color=Signal::derive(move || swatch_state.local.get()) />
		{label.map(|label| {
			view! {
				<label id=aria.label_id.clone() for=aria.field_id.clone()>
					{label}
				</label>
			}
		})}
	}
}

/// The shared state of a color field.
struct ColorFieldState {
	local: RwSignal<Color>,
	input_text: RwSignal<String>,
	on_change: Arc<Option<BoxedColorChangeCallback>>,
}

#[derive(Clone)]
struct ColorInputHandlers {
	state: Arc<ColorFieldState>,
}

impl ColorInputHandlers {
	fn handle_input(&self, event: web_sys::Event) {
		let Some(typed) = typed_value(event) else {
			return;
		};

		self.state.input_text.set(typed.clone());

		if let Some(color) = Color::parse(&typed) {
			self.state.local.set(color.clone());

			if let Some(on_change) = self.state.on_change.as_ref() {
				(*on_change)(&color);
			}
		}
	}

	fn handle_key_down(_: &Self, event: web_sys::KeyboardEvent) {
		if event.key() == "Enter" {
			event.prevent_default();
		}
	}
}

/// The text typed into an input event.
fn typed_value(event: web_sys::Event) -> Option<String> {
	event
		.target()
		.and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
		.map(|input| input.value())
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
	use leptos::mount::mount_to_body;
	use wasm_bindgen::JsCast;
	use wasm_bindgen_test::*;

	use super::*;

	wasm_bindgen_test_configure!(run_in_browser);

	#[wasm_bindgen_test]
	async fn color_field_previews_the_initial_color() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! {
				<ColorField
					label="Accent".to_string()
					default_value=Color::parse("#ff0000").unwrap()
				/>
			}
		});

		let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
		wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();

		let input = document()
			.query_selector("input[type='text']")
			.unwrap()
			.unwrap()
			.unchecked_into::<web_sys::HtmlInputElement>();

		let swatch = document()
			.query_selector(".leptos-aria-color-swatch")
			.unwrap()
			.unwrap();

		assert_eq!(input.value(), "#ff0000");
		assert!(swatch.get_attribute("style").unwrap().contains("255, 0, 0"));
	}
}
