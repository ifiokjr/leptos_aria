use std::sync::Arc;

use leptos::prelude::*;
use leptos_aria_button::BoxedPressEventCallback;
use leptos_aria_button::Button;
use leptos_aria_button::ButtonType;
use leptos_aria_interactions::PressEvent;

/// A callback invoked when the selection state changes.
pub type BoxedSelectionChangeCallback = Box<dyn Fn(bool) + Send + Sync>;

/// State management for toggle components like checkboxes and switches.
///
/// The state can be controlled (an `is_selected` signal driven by the owner)
/// or uncontrolled (an internal signal), and guards read-only toggles.
#[derive(Clone)]
pub struct ToggleState {
	selection: Selection,
	default_selected: bool,
	is_read_only: bool,
	on_change: Option<RcChangeCallback>,
}

#[derive(Clone, Copy)]
enum Selection {
	Controlled(Signal<bool>),
	Uncontrolled(RwSignal<bool>),
}

type RcChangeCallback = Arc<dyn Fn(bool) + Send + Sync>;

/// The options accepted by [`use_toggle_state`].
#[derive(Default)]
pub struct UseToggleStateProps {
	/// Whether the element should be selected (uncontrolled).
	pub default_selected: Option<bool>,

	/// Whether the element should be selected (controlled).
	pub is_selected: Option<Signal<bool>>,

	/// Whether the element's selection state is read only.
	pub is_read_only: Option<bool>,

	/// Handler that is called when the selection state changes.
	pub on_change: Option<BoxedSelectionChangeCallback>,
}

/// Provides state management for toggle components like checkboxes and
/// switches.
pub fn use_toggle_state(props: UseToggleStateProps) -> ToggleState {
	let default_selected = props.default_selected.unwrap_or_default();

	let selection = match props.is_selected {
		Some(controlled) => Selection::Controlled(controlled),
		None => Selection::Uncontrolled(RwSignal::new(default_selected)),
	};

	let on_change = props.on_change.map(|callback| {
		let callback: Arc<dyn Fn(bool) + Send + Sync> =
			Arc::new(move |selected: bool| callback(selected));

		callback
	});

	ToggleState {
		selection,
		default_selected,
		is_read_only: props.is_read_only.unwrap_or_default(),
		on_change,
	}
}

impl ToggleState {
	/// Whether the toggle is selected.
	///
	/// Reactive: reading inside an effect or view tracks the selection.
	pub fn is_selected(&self) -> bool {
		self.selected_signal().get()
	}

	/// Whether the toggle is selected, without tracking.
	pub fn is_selected_untracked(&self) -> bool {
		self.selected_signal().get_untracked()
	}

	/// A reactive read of the selection state.
	pub fn selected(&self) -> bool {
		self.selected_signal().get()
	}

	/// Whether the toggle is selected by default.
	pub fn default_selected(&self) -> bool {
		self.default_selected
	}

	/// The signal backing the selection state, for reactive bindings.
	pub fn selected_signal(&self) -> Signal<bool> {
		match &self.selection {
			Selection::Controlled(signal) => *signal,
			Selection::Uncontrolled(signal) => Signal::from(*signal),
		}
	}

	/// Update the selection state, unless read only.
	///
	/// A controlled state reports upward and lets the owner decide; an
	/// uncontrolled one updates itself.
	pub fn set_selected(&self, selected: bool) {
		if self.is_read_only || self.is_selected_untracked() == selected {
			return;
		}

		if let Selection::Uncontrolled(signal) = &self.selection {
			signal.set(selected);
		}

		if let Some(on_change) = &self.on_change {
			on_change(selected);
		}
	}

	/// Toggle the selection state, unless read only.
	pub fn toggle(&self) {
		let selected = self.is_selected_untracked();
		self.set_selected(!selected);
	}
}

/// A two-state button that can be toggled on or off.
#[component]
pub fn ToggleButton(
	/// Whether the button is disabled.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	/// Whether the button should be selected (uncontrolled).
	#[prop(optional, into)]
	default_selected: Signal<bool>,
	/// Whether the button should be selected (controlled).
	#[prop(optional, into)]
	is_selected: Option<Signal<bool>>,
	/// Handler that is called when the button's selection state changes.
	#[prop(optional)]
	on_change: Option<BoxedSelectionChangeCallback>,
	/// Handler that is called when the press is released over the target.
	#[prop(optional)]
	on_press: Option<BoxedPressEventCallback>,
	/// The behavior of the button when used in an HTML form.
	#[prop(optional, into)]
	r#type: Signal<ButtonType>,
	children: Children,
) -> impl IntoView {
	let state = use_toggle_state(UseToggleStateProps {
		default_selected: Some(default_selected.get_untracked()),
		is_selected,
		is_read_only: None,
		on_change,
	});

	let selected = state.selected_signal();
	let press = merge_press_with_toggle(on_press, &state);

	view! {
		<Button
			is_disabled=is_disabled
			on_press=press
			r#type=r#type
			aria_pressed=Signal::derive(move || selected.get())
		>
			{children()}
		</Button>
	}
}

/// Wrap the owner's press handler so a press toggles the state first.
fn merge_press_with_toggle(
	on_press: Option<BoxedPressEventCallback>,
	state: &ToggleState,
) -> BoxedPressEventCallback {
	let state = state.clone();

	Box::new(move |event: &PressEvent| {
		state.toggle();

		if let Some(on_press) = &on_press {
			on_press(event);
		}
	})
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
	use leptos::mount::mount_to_body;
	use wasm_bindgen::JsCast;
	use wasm_bindgen_test::*;

	/// Wait for queued reactive updates to apply.
	async fn flush_microtasks() {
		let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);

		wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
	}

	use super::*;

	wasm_bindgen_test_configure!(run_in_browser);

	#[component]
	fn Example() -> impl IntoView {
		view! {
			<ToggleButton default_selected=Signal::from(false)>
				"Pin"
			</ToggleButton>
		}
	}

	#[wasm_bindgen_test]
	async fn renders_an_aria_pressed_button() {
		console_error_panic_hook::set_once();

		mount_to_body(|| view! { <Example /> });

		flush_microtasks().await;

		web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(
			&document().body().unwrap().inner_html(),
		));

		let button = document()
			.query_selector("button")
			.unwrap()
			.unwrap()
			.unchecked_into::<web_sys::HtmlButtonElement>();

		assert_eq!(
			button.get_attribute("aria-pressed").as_deref(),
			Some("false")
		);

		button.click();

		flush_microtasks().await;

		assert_eq!(
			button.get_attribute("aria-pressed").as_deref(),
			Some("true"),
			"a click toggles the pressed state"
		);
	}
}
