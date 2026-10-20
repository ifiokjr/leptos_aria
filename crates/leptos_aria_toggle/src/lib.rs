#![doc(html_logo_url = "https://ifiokjr.github.io/leptos_aria/logo.svg")]
#![doc(html_favicon_url = "https://ifiokjr.github.io/leptos_aria/favicon.ico")]

use std::sync::Arc;

use leptos::prelude::*;
use leptos_aria_button::BoxedPressEventCallback;
use leptos_aria_button::Button;
use leptos_aria_button::ButtonType;
use leptos_aria_interactions::PressEvent;
use leptos_aria_utils::merge_class;
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
	/// Classes for the root element.
	#[prop(optional, into)]
	class: Signal<String>,
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
			class=class
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
			<ToggleButton default_selected=Signal::from(false) class="toggle-test-pressed">
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
			.query_selector("button.toggle-test-pressed")
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

	#[wasm_bindgen_test]
	async fn toggle_button_forwards_the_caller_class() {
		console_error_panic_hook::set_once();

		mount_to_body(|| view! { <ToggleButton class="custom-toggle">"Bold"</ToggleButton> });

		flush_microtasks().await;

		let button = document()
			.query_selector("button.custom-toggle")
			.unwrap()
			.expect("the caller class sits on the toggle button");

		assert_eq!(
			button.get_attribute("aria-pressed").as_deref(),
			Some("false")
		);
	}

	#[wasm_bindgen_test]
	async fn switch_appends_the_caller_class() {
		console_error_panic_hook::set_once();

		mount_to_body(|| view! { <Switch class="custom-switch">"Wi-Fi"</Switch> });

		let switch = document()
			.query_selector(".leptos-aria-switch.custom-switch")
			.unwrap()
			.expect("the caller class sits on the switch root");

		assert_eq!(
			switch.get_attribute("class").as_deref(),
			Some("leptos-aria-switch custom-switch")
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
	/// Classes added after the built-in `leptos-aria-switch` class.
	#[prop(optional, into)]
	class: Signal<String>,
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
		<label class=merge_class("leptos-aria-switch", class) on:mousedown=|event: web_sys::MouseEvent| {
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

/// A callback invoked with every pressed value when a toggle group changes.
pub type BoxedToggleGroupChangeCallback = Box<dyn Fn(Vec<String>) + Send + Sync>;

/// Whether a toggle group keeps one or many buttons pressed.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum ToggleGroupMode {
	/// Pressing a button releases the others.
	#[default]
	Single,
	/// Every button toggles on its own.
	Multiple,
}

/// The pressed values of a toggle group, owned by the caller or the group.
#[derive(Clone, Copy)]
enum GroupValue {
	Controlled(Signal<Vec<String>>),
	Uncontrolled(RwSignal<Vec<String>>),
}

/// The state a toggle group shares with its buttons.
#[derive(Clone)]
pub struct ToggleGroupContext {
	value: GroupValue,
	mode: Signal<ToggleGroupMode>,
	on_change: Option<Arc<dyn Fn(Vec<String>) + Send + Sync>>,
}

impl ToggleGroupContext {
	/// The signal holding the pressed values, for reactive bindings.
	pub fn value_signal(&self) -> Signal<Vec<String>> {
		match self.value {
			GroupValue::Controlled(signal) => signal,
			GroupValue::Uncontrolled(signal) => signal.into(),
		}
	}

	/// Press or release `key` according to the group's mode.
	///
	/// A controlled group reports the new values and lets the owner decide;
	/// an uncontrolled one updates itself.
	pub fn toggle(&self, key: &str) {
		let mut keys = self.value_signal().get_untracked();

		if keys.iter().any(|k| k == key) {
			keys.retain(|k| k != key);
		} else {
			if self.mode.get_untracked() == ToggleGroupMode::Single {
				keys.clear();
			}

			keys.push(key.to_string());
		}

		if let GroupValue::Uncontrolled(signal) = self.value {
			signal.set(keys.clone());
		}

		if let Some(on_change) = &self.on_change {
			on_change(keys);
		}
	}
}

/// A toggle group is a set of toggle buttons selecting one or many values.
#[component]
pub fn ToggleGroup(
	/// Whether one or many buttons can be pressed, single by default.
	#[prop(optional, into)]
	mode: Signal<ToggleGroupMode>,
	/// The values of the initially pressed buttons (uncontrolled).
	#[prop(optional, into)]
	default_value: Vec<String>,
	/// The values of the pressed buttons (controlled).
	#[prop(optional, into)]
	value: Option<Signal<Vec<String>>>,
	/// Handler that is called with every pressed value when one changes.
	#[prop(optional)]
	on_change: Option<BoxedToggleGroupChangeCallback>,
	/// Classes added after the built-in `leptos-aria-toggle-group` class.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let value = match value {
		Some(controlled) => GroupValue::Controlled(controlled),
		None => GroupValue::Uncontrolled(RwSignal::new(default_value)),
	};

	provide_context(ToggleGroupContext {
		value,
		mode,
		on_change: on_change.map(Arc::from),
	});

	view! { <div class=merge_class("leptos-aria-toggle-group", class) role="group">{children()}</div> }
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
	/// Classes added after the built-in `leptos-aria-toggle-group__item` class.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let context = expect_context::<ToggleGroupContext>();
	let is_selected = Signal::derive({
		let keys = context.value_signal();
		let value = value.clone();

		move || keys.with(|keys| keys.contains(&value))
	});

	view! {
		<button
			type="button"
			class=merge_class("leptos-aria-toggle-group__item", class)
			aria-pressed=move || is_selected.get().to_string()
			data-state=move || if is_selected.get() { "on" } else { "off" }
			disabled=move || is_disabled.get()
			on:click=move |_| {
				if !is_disabled.get_untracked() {
					context.toggle(&value);
				}
			}
		>
			{children()}
		</button>
	}
}

#[cfg(all(test, target_arch = "wasm32"))]
mod group_tests {
	use leptos::mount::mount_to_body;
	use wasm_bindgen::JsCast;
	use wasm_bindgen_test::*;

	use super::*;

	wasm_bindgen_test_configure!(run_in_browser);

	fn item(selector: &str) -> web_sys::HtmlButtonElement {
		document()
			.query_selector(selector)
			.unwrap()
			.unwrap()
			.unchecked_into::<web_sys::HtmlButtonElement>()
	}

	fn pressed(selector: &str) -> Option<String> {
		item(selector).get_attribute("aria-pressed")
	}

	async fn tick() {
		let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
		wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
	}

	#[wasm_bindgen_test]
	async fn multiple_mode_presses_buttons_independently() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! {
				<ToggleGroup mode=ToggleGroupMode::Multiple class="group-test-multiple">
					<ToggleGroupItem value="bold" class="bold">"B"</ToggleGroupItem>
					<ToggleGroupItem value="italic" class="italic">"I"</ToggleGroupItem>
				</ToggleGroup>
			}
		});

		item(".group-test-multiple .bold").click();
		item(".group-test-multiple .italic").click();
		tick().await;

		assert_eq!(
			pressed(".group-test-multiple .bold").as_deref(),
			Some("true")
		);
		assert_eq!(
			pressed(".group-test-multiple .italic").as_deref(),
			Some("true")
		);
	}

	#[wasm_bindgen_test]
	async fn a_controlled_group_reports_changes_and_follows_its_value() {
		console_error_panic_hook::set_once();

		let value = RwSignal::new(vec!["left".to_string()]);
		let reported = RwSignal::new(Vec::<String>::new());

		mount_to_body(move || {
			view! {
				<ToggleGroup
					value=value
					on_change=Box::new(move |keys| reported.set(keys))
					class="group-test-controlled"
				>
					<ToggleGroupItem value="left" class="left">"L"</ToggleGroupItem>
					<ToggleGroupItem value="right" class="right">"R"</ToggleGroupItem>
				</ToggleGroup>
			}
		});

		item(".group-test-controlled .right").click();
		tick().await;

		assert_eq!(reported.get_untracked(), vec!["right".to_string()]);
		assert_eq!(
			pressed(".group-test-controlled .right").as_deref(),
			Some("false"),
			"the owner has not accepted the change yet"
		);

		value.set(vec!["right".to_string()]);
		tick().await;

		assert_eq!(
			pressed(".group-test-controlled .left").as_deref(),
			Some("false")
		);
		assert_eq!(
			pressed(".group-test-controlled .right").as_deref(),
			Some("true")
		);
	}
}
