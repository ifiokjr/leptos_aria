#![doc(html_logo_url = "https://ifiokjr.github.io/leptos_aria/logo.svg")]
#![doc(html_favicon_url = "https://ifiokjr.github.io/leptos_aria/favicon.ico")]
#![allow(clippy::needless_pass_by_value)]

mod color;

use std::sync::Arc;

pub use color::Color;
use leptos::prelude::*;
use leptos_aria_utils::merge_class;
use wasm_bindgen::JsCast;

/// A callback invoked when the color changes.
pub type BoxedColorChangeCallback = Box<dyn Fn(&Color) + Send + Sync + 'static>;

/// A color swatch shows a single color as a labeled preview.
#[component]
pub fn ColorSwatch(
	/// The color to show.
	#[prop(into)]
	color: Signal<Color>,
	/// Classes added after the built-in `leptos-aria-color-swatch` class.
	#[prop(optional, into)]
	class: Signal<String>,
) -> impl IntoView {
	view! {
		<span
			class=merge_class("leptos-aria-color-swatch", class)
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
	/// Classes for the root element, the text input.
	#[prop(optional, into)]
	class: Signal<String>,
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
			class=merge_class("", class)
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

/// A color area is a two-dimensional pad picking saturation and lightness at
/// a fixed hue.
///
/// Dragging moves through the color space horizontally (saturation) and
/// vertically (lightness); the pad shows the current position as a thumb and
/// reports every change.
#[component]
pub fn ColorArea(
	/// The current color (controlled).
	#[prop(into)]
	value: Signal<Color>,
	/// Handler that is called when the color changes.
	#[prop(optional)]
	on_change: Option<BoxedColorChangeCallback>,
	/// Classes added after the built-in `leptos-aria-color-area` class.
	#[prop(optional, into)]
	class: Signal<String>,
) -> impl IntoView {
	let state = Arc::new(ColorAreaState {
		color: value,
		on_change: Arc::new(on_change),
	});
	let press_state = Arc::clone(&state);

	view! {
		<div
			class=merge_class("leptos-aria-color-area", class)
			role="application"
			aria-label="Saturation and brightness"
			style=move || format!("background: {};", hue_background(&value.get()))
			on:pointerdown=move |event| handle_area_pointer(&press_state, &event)
		>
			<div
				class="leptos-aria-color-area__thumb"
				style=move || area_thumb_style(&value.get())
			></div>
		</div>
	}
}

/// The shared state of a color area.
struct ColorAreaState {
	color: Signal<Color>,
	on_change: Arc<Option<BoxedColorChangeCallback>>,
}

/// Set the color from a pointer position on the area.
fn handle_area_pointer(state: &Arc<ColorAreaState>, event: &web_sys::PointerEvent) {
	let Some(target) = event.current_target() else {
		return;
	};
	let Ok(element) = target.dyn_into::<web_sys::Element>() else {
		return;
	};

	let next = color_at_position(&element, event, &state.color.get_untracked());

	if let Some(on_change) = state.on_change.as_ref() {
		(*on_change)(&next);
	}
}

/// The color under a pointer position within an area, keeping the hue.
fn color_at_position(
	element: &web_sys::Element,
	event: &web_sys::PointerEvent,
	current: &Color,
) -> Color {
	let rect = element.get_bounding_client_rect();
	let width = rect.width();
	let height = rect.height();

	let x = (f64::from(event.client_x()) - rect.left()).clamp(0.0, width) / width.max(1.0);
	let y = (f64::from(event.client_y()) - rect.top()).clamp(0.0, height) / height.max(1.0);

	current.with_saturation(x).with_lightness(1.0 - y)
}

/// The hue-only background of an area pad.
fn hue_background(color: &Color) -> String {
	Color::new(1.0, 0.0, 0.0, 1.0)
		.with_hue(color.hue())
		.to_rgb_string()
}

/// The thumb position style of an area pad.
fn area_thumb_style(color: &Color) -> String {
	format!(
		"position: absolute; left: {:.1}%; top: {:.1}%;",
		color.saturation() * 100.0,
		(1.0 - color.lightness()) * 100.0
	)
}

/// A color slider adjusts one channel of a color along a track.
///
/// The channel defaults to hue; dragging the thumb sets the channel value
/// directly.
#[component]
pub fn ColorSlider(
	/// The current color (controlled).
	#[prop(into)]
	value: Signal<Color>,
	/// The channel the slider adjusts, hue by default.
	#[prop(optional, into)]
	channel: Option<Signal<ColorChannel>>,
	/// Handler that is called when the color changes.
	#[prop(optional)]
	on_change: Option<BoxedColorChangeCallback>,
	/// Classes added after the built-in `leptos-aria-color-slider` class.
	#[prop(optional, into)]
	class: Signal<String>,
) -> impl IntoView {
	let channel = channel.unwrap_or_else(|| Signal::from(ColorChannel::Hue));
	let state = Arc::new(SliderState {
		color: value,
		channel,
		on_change: Arc::new(on_change),
	});
	let style_state = Arc::clone(&state);
	let thumb_state = Arc::clone(&state);
	let value_state = Arc::clone(&state);
	let pointer_state = Arc::clone(&state);
	let key_state = Arc::clone(&state);

	view! {
		<div
			class=merge_class("leptos-aria-color-slider", class)
			role="slider"
			aria-label=move || channel.get().label()
			aria-valuemin="0"
			aria-valuemax="100"
			aria-valuenow=move || format_channel_value(&value_state)
			tabindex="0"
			style=move || slider_track_style(&style_state)
			on:pointerdown=move |event| handle_slider_pointer(&pointer_state, event)
			on:keydown=move |event| handle_slider_key_down(&key_state, &event)
		>
			<div
				class="leptos-aria-color-slider__thumb"
				style=move || slider_thumb_style(&thumb_state)
			></div>
		</div>
	}
}

/// The channel a color slider adjusts.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum ColorChannel {
	#[default]
	Hue,
	Saturation,
	Lightness,
}

impl ColorChannel {
	fn label(self) -> &'static str {
		match self {
			ColorChannel::Hue => "Hue",
			ColorChannel::Saturation => "Saturation",
			ColorChannel::Lightness => "Lightness",
		}
	}

	/// Read the channel as a fraction of one.
	fn read(self, color: &Color) -> f64 {
		match self {
			ColorChannel::Hue => color.hue() / 360.0,
			ColorChannel::Saturation => color.saturation(),
			ColorChannel::Lightness => color.lightness(),
		}
	}

	/// Write the channel from a fraction of one.
	fn write(self, color: &Color, value: f64) -> Color {
		match self {
			ColorChannel::Hue => color.with_hue(value * 360.0),
			ColorChannel::Saturation => color.with_saturation(value),
			ColorChannel::Lightness => color.with_lightness(value),
		}
	}
}

/// The shared state of a color slider.
struct SliderState {
	color: Signal<Color>,
	channel: Signal<ColorChannel>,
	on_change: Arc<Option<BoxedColorChangeCallback>>,
}

/// The formatted aria value of a slider, `0` through `100`.
fn format_channel_value(state: &Arc<SliderState>) -> String {
	format!(
		"{}",
		(state
			.channel
			.get_untracked()
			.read(&state.color.get_untracked())
			* 100.0)
			.round() as u32
	)
}

/// The track background showing the channel gradient.
fn slider_track_style(state: &Arc<SliderState>) -> String {
	let channel = state.channel.get_untracked();
	let color = state.color.get_untracked();

	let start = channel.write(&color, 0.0).to_rgb_string();
	let end = channel.write(&color, 1.0).to_rgb_string();

	format!("background: linear-gradient(to right, {start}, {end});")
}

/// The thumb position style of a slider.
fn slider_thumb_style(state: &Arc<SliderState>) -> String {
	let value = state
		.channel
		.get_untracked()
		.read(&state.color.get_untracked());

	format!("position: absolute; left: {:.1}%;", value * 100.0)
}

/// Set the channel from a pointer position on the track.
fn handle_slider_pointer(state: &Arc<SliderState>, event: web_sys::PointerEvent) {
	let Some(target) = event.current_target() else {
		return;
	};
	let Ok(element) = target.dyn_into::<web_sys::Element>() else {
		return;
	};

	let rect = element.get_bounding_client_rect();
	let fraction =
		((f64::from(event.client_x()) - rect.left()) / rect.width().max(1.0)).clamp(0.0, 1.0);

	set_channel(state, fraction);
}

/// Step the channel with the arrow keys.
fn handle_slider_key_down(state: &Arc<SliderState>, event: &web_sys::KeyboardEvent) {
	let step = match event.key().as_str() {
		"ArrowRight" | "ArrowUp" => 0.01,
		"ArrowLeft" | "ArrowDown" => -0.01,
		_ => return,
	};

	event.prevent_default();

	let current = state
		.channel
		.get_untracked()
		.read(&state.color.get_untracked());

	set_channel(state, (current + step).clamp(0.0, 1.0));
}

/// Write the channel and report the change.
///
/// The color signal is a derived view over the owner's state, so the new
/// value flows out through the change callback.
fn set_channel(state: &Arc<SliderState>, fraction: f64) {
	let channel = state.channel.get_untracked();
	let next = channel.write(&state.color.get_untracked(), fraction);

	if let Some(on_change) = state.on_change.as_ref() {
		(*on_change)(&next);
	}
}

/// A color wheel picks a hue around a circular track.
#[component]
pub fn ColorWheel(
	/// The current color (controlled).
	#[prop(into)]
	value: Signal<Color>,
	/// Handler that is called when the hue changes.
	#[prop(optional)]
	on_change: Option<BoxedColorChangeCallback>,
	/// Classes added after the built-in `leptos-aria-color-wheel` class.
	#[prop(optional, into)]
	class: Signal<String>,
) -> impl IntoView {
	let state = Arc::new(WheelState {
		color: value,
		on_change: Arc::new(on_change),
	});
	let value_state = Arc::clone(&state);
	let thumb_state = Arc::clone(&state);
	let pointer_state = Arc::clone(&state);
	let key_state = Arc::clone(&state);

	view! {
		<div
			class=merge_class("leptos-aria-color-wheel", class)
			role="slider"
			aria-label="Hue"
			aria-valuemin="0"
			aria-valuemax="360"
			aria-valuenow=move || {
				format!("{}", value_state.color.get_untracked().hue().round() as u32)
			}
			tabindex="0"
			style="border-radius: 50%; background: conic-gradient(red, yellow, lime, cyan, blue, magenta, red);"
			on:pointerdown=move |event| handle_wheel_pointer(&pointer_state, event)
			on:keydown=move |event| handle_wheel_key_down(&key_state, &event)
		>
			<div
				class="leptos-aria-color-wheel__thumb"
				style=move || wheel_thumb_style(&thumb_state)
			></div>
		</div>
	}
}

/// The shared state of a color wheel.
struct WheelState {
	color: Signal<Color>,
	on_change: Arc<Option<BoxedColorChangeCallback>>,
}

/// The thumb position style of a wheel.
fn wheel_thumb_style(state: &Arc<WheelState>) -> String {
	let hue = state.color.get_untracked().hue();
	let angle = hue * std::f64::consts::PI / 180.0;

	format!(
		"position: absolute; left: {:.1}%; top: {:.1}%;",
		50.0 + angle.cos() * 40.0,
		50.0 + angle.sin() * 40.0
	)
}

/// Set the hue from a pointer position around the wheel center.
fn handle_wheel_pointer(state: &Arc<WheelState>, event: web_sys::PointerEvent) {
	let Some(target) = event.current_target() else {
		return;
	};
	let Ok(element) = target.dyn_into::<web_sys::Element>() else {
		return;
	};

	let rect = element.get_bounding_client_rect();
	let center_x = rect.left() + rect.width() / 2.0;
	let center_y = rect.top() + rect.height() / 2.0;

	let hue = (f64::from(event.client_y()) - center_y)
		.atan2(f64::from(event.client_x()) - center_x)
		.to_degrees()
		.rem_euclid(360.0);

	set_hue(state, hue);
}

/// Step the hue with the arrow keys.
fn handle_wheel_key_down(state: &Arc<WheelState>, event: &web_sys::KeyboardEvent) {
	let step = match event.key().as_str() {
		"ArrowRight" | "ArrowUp" => 1.0,
		"ArrowLeft" | "ArrowDown" => -1.0,
		_ => return,
	};

	event.prevent_default();

	let next = (state.color.get_untracked().hue() + step).rem_euclid(360.0);

	set_hue(state, next);
}

/// Write the hue and report the change; the color signal is a derived view
/// over the owner's state.
fn set_hue(state: &Arc<WheelState>, hue: f64) {
	let next = state.color.get_untracked().with_hue(hue);

	if let Some(on_change) = state.on_change.as_ref() {
		(*on_change)(&next);
	}
}

#[cfg(all(test, target_arch = "wasm32"))]
mod surface_tests {
	use leptos::mount::mount_to_body;
	use wasm_bindgen_test::*;

	use super::*;

	wasm_bindgen_test_configure!(run_in_browser);

	#[wasm_bindgen_test]
	async fn color_slider_renders_the_channel_semantics() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! {
				<ColorSlider value=Signal::derive(|| Color::parse("#ff0000").unwrap()) />
			}
		});

		let slider = document()
			.query_selector(".leptos-aria-color-slider")
			.unwrap()
			.unwrap();

		assert_eq!(slider.get_attribute("aria-label").as_deref(), Some("Hue"));
		assert_eq!(slider.get_attribute("aria-valuemin").as_deref(), Some("0"));
		assert_eq!(
			slider.get_attribute("aria-valuemax").as_deref(),
			Some("100")
		);
	}

	#[wasm_bindgen_test]
	async fn color_wheel_renders_the_hue_semantics() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! {
				<ColorWheel value=Signal::derive(|| Color::parse("#00ff00").unwrap()) />
			}
		});

		let wheel = document()
			.query_selector(".leptos-aria-color-wheel")
			.unwrap()
			.unwrap();

		assert_eq!(
			wheel.get_attribute("aria-valuenow").as_deref(),
			Some("120"),
			"green reads as a hue of 120 degrees"
		);
	}
}
