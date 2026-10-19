use std::sync::Arc;

use leptos::prelude::*;
use leptos_aria_button::BoxedPressEventCallback;
use leptos_aria_button::Button;
use leptos_aria_button::ButtonType;
use leptos_aria_interactions::PressEvent;
use wasm_bindgen::JsCast;

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

/// A switch is a toggle with on/off semantics rather than pressed/unpressed.
///
/// The same state machine as [`ToggleButton`]; rendered as a checkbox input
/// with `role="switch"` so the label toggles with it natively.
#[component]
pub fn Switch(
	/// Whether the switch is checked (uncontrolled).
	#[prop(optional, into)]
	default_checked: Signal<bool>,
	/// Whether the switch is checked (controlled).
	#[prop(optional, into)]
	is_checked: Option<Signal<bool>>,
	/// Handler that is called when the checked state changes.
	#[prop(optional)]
	on_change: Option<BoxedSelectionChangeCallback>,
	/// Whether the switch is disabled.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	children: Children,
) -> impl IntoView {
	let state = use_toggle_state(UseToggleStateProps {
		default_selected: Some(default_checked.get_untracked()),
		is_selected: is_checked,
		on_change,
		..Default::default()
	});
	let checked = state.selected_signal();

	view! {
		<label class="leptos-aria-switch" on:mousedown=|event: web_sys::MouseEvent| {
			// Prevent the label from taking focus when pressed.
			event.prevent_default();
		}>
			<input
				type="checkbox"
				role="switch"
				class="leptos-aria-switch__input"
				checked=move || checked.get()
				disabled=move || is_disabled.get()
				data-state=move || if checked.get() { "checked" } else { "unchecked" }
				on:change=move |event: web_sys::Event| {
					event.stop_propagation();

					let checked = event
						.target()
						.and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
						.is_some_and(|input| input.checked());

					state.set_selected(checked);
				}
			/>
			{children()}
		</label>
	}
}

/// The selected keys shared by a toggle group.
type ToggleGroupKeys = Vec<String>;

/// The state a toggle group shares with its buttons.
#[derive(Clone)]
pub struct ToggleGroupContext {
	keys: RwSignal<ToggleGroupKeys>,
	mode: AccordionModeAlias,
}

/// The selection mode of a toggle group, mirroring the accordion's.
type AccordionModeAlias = leptos_aria_accordion_shim::Mode;

/// A minimal mode alias so the toggle crate stays dependency-light.
mod leptos_aria_accordion_shim {
	#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
	pub enum Mode {
		#[default]
		Single,
		Multiple,
	}
}

/// A toggle group is a set of toggle buttons selecting one or many values.
#[component]
pub fn ToggleGroup(
	/// Whether one or many buttons can be selected, single by default.
	#[prop(optional, into)]
	mode: Signal<AccordionModeAlias>,
	/// The keys of the initially selected buttons.
	#[prop(optional, into)]
	default_value: Vec<String>,
	children: Children,
) -> impl IntoView {
	let context = ToggleGroupContext {
		keys: RwSignal::new(default_value),
		mode: mode.get_untracked(),
	};

	provide_context(context.clone());

	view! { <div class="leptos-aria-toggle-group" role="group">{children()}</div> }
}

/// One toggle button inside a [`ToggleGroup`].
#[allow(clippy::needless_pass_by_value)]
#[component]
pub fn ToggleGroupItem(
	/// The value reported in the group selection.
	#[prop(into)]
	value: String,
	/// Whether this button is disabled.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	children: Children,
) -> impl IntoView {
	let context = expect_context::<ToggleGroupContext>();
	let is_selected = Signal::derive({
		let keys = context.keys;
		let value = value.clone();

		move || keys.get().contains(&value)
	});
	let click_value = value.clone();
	let click_context = context.clone();

	view! {
		<button
			type="button"
			class="leptos-aria-toggle-group__item"
			aria-pressed=move || is_selected.get().to_string()
			data-state=move || if is_selected.get() { "on" } else { "off" }
			disabled=move || is_disabled.get()
			on:click=move |_| {
				if !is_disabled.get_untracked() {
					toggle_group_key(&click_context, &click_value);
				}
			}
		>
			{children()}
		</button>
	}
}

/// Toggle a key in its group according to the mode.
fn toggle_group_key(context: &ToggleGroupContext, key: &str) {
	context.keys.update(|keys| {
		if keys.iter().any(|k| k == key) {
			keys.retain(|k| k != key);
		} else if context.mode == AccordionModeAlias::Single {
			keys.clear();
			keys.push(key.to_string());
		} else {
			keys.push(key.to_string());
		}
	});
}
