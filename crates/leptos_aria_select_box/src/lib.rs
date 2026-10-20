#![doc(html_logo_url = "https://ifiokjr.github.io/leptos_aria/logo.svg")]
#![doc(html_favicon_url = "https://ifiokjr.github.io/leptos_aria/favicon.ico")]
#![allow(clippy::needless_pass_by_value)]

use std::collections::HashSet;
use std::sync::Arc;

use leptos::prelude::*;
use leptos_aria_list_box::ListBox;
use leptos_aria_list_box::ListBoxOption;
use leptos_aria_overlay::use_overlay_dismiss;
use leptos_aria_overlay::use_popup_focus;
use leptos_aria_selection::Selection;
use leptos_aria_utils::merge_class;

/// One option of a select box: a value with its text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectBoxItem {
	pub value: String,
	pub label: String,
}

/// A callback invoked with the chosen value when the selection changes.
pub type BoxedSelectBoxChangeCallback = Box<dyn Fn(Option<String>) + Send + Sync>;

/// The chosen value of a select box, owned by the caller or the select box.
#[derive(Clone, Copy)]
enum SelectedValue {
	Controlled(Signal<Option<String>>),
	Uncontrolled(RwSignal<Option<String>>),
}

impl SelectedValue {
	fn signal(self) -> Signal<Option<String>> {
		match self {
			SelectedValue::Controlled(signal) => signal,
			SelectedValue::Uncontrolled(signal) => signal.into(),
		}
	}
}

/// A select box lets the user pick one option from a collapsible list,
/// without a free text input.
///
/// The trigger shows the chosen option's label, or the placeholder, and
/// toggles a list box below it. Choosing an option, escape, or a press
/// outside closes the list; focus moves into the list on open and back to
/// the trigger on close.
#[component]
pub fn SelectBox(
	/// The content to display as the label.
	#[prop(optional)]
	label: Option<String>,
	/// The available options.
	#[prop(into)]
	options: Vec<SelectBoxItem>,
	/// The chosen option's value (controlled).
	#[prop(optional, into)]
	value: Option<Signal<Option<String>>>,
	/// The initially chosen option's value (uncontrolled).
	#[prop(optional, into)]
	default_value: Option<String>,
	/// Handler that is called with the chosen value when it changes.
	#[prop(optional)]
	on_change: Option<BoxedSelectBoxChangeCallback>,
	/// The trigger text while nothing is chosen, `Select…` by default.
	#[prop(optional, into)]
	placeholder: Option<String>,
	/// Whether the select box is disabled.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	/// Classes added after the built-in `leptos-aria-select-box` class on the
	/// root element.
	#[prop(optional, into)]
	class: Signal<String>,
	/// Classes for the trigger button.
	#[prop(optional, into)]
	trigger_class: Signal<String>,
	/// Classes for the popup holding the list.
	#[prop(optional, into)]
	popup_class: Signal<String>,
	/// Classes for every option.
	#[prop(optional, into)]
	option_class: Signal<String>,
	/// Content rendered at the start of every option, such as a check mark
	/// styled from the option's `aria-selected`.
	#[prop(optional, into)]
	indicator: Option<ViewFn>,
	/// Content rendered in the trigger after the value, such as a chevron.
	#[prop(optional)]
	children: Option<Children>,
) -> impl IntoView {
	let selected = match value {
		Some(controlled) => SelectedValue::Controlled(controlled),
		None => SelectedValue::Uncontrolled(RwSignal::new(default_value)),
	};
	let is_open = RwSignal::new(false);
	let root = NodeRef::<leptos::html::Div>::new();
	let trigger = NodeRef::<leptos::html::Button>::new();
	let options = Arc::new(options);
	let placeholder = placeholder.unwrap_or_else(|| "Select…".to_string());

	use_overlay_dismiss(root, is_open.into(), Box::new(move || is_open.set(false)));
	use_popup_focus(root, trigger, is_open.into(), "[role=listbox]");

	let selected_label = Memo::new({
		let options = Arc::clone(&options);

		move |_| {
			let value = selected.signal().get()?;

			options
				.iter()
				.find(|item| item.value == value)
				.map(|item| item.label.clone())
		}
	});
	let selection = Signal::derive(move || {
		selected.signal().get().map_or(Selection::None, |value| {
			Selection::Keys(HashSet::from([value]))
		})
	});
	let on_change = on_change.map(Arc::<dyn Fn(Option<String>) + Send + Sync>::from);
	let choose = Box::new(move |selection: &Selection| {
		let Selection::Keys(keys) = selection else {
			return;
		};
		let chosen = keys.iter().next().cloned();

		if let SelectedValue::Uncontrolled(signal) = selected {
			signal.set(chosen.clone());
		}

		if let Some(on_change) = &on_change {
			on_change(chosen);
		}
	});
	// Choosing closes the list even when the chosen option was already
	// selected, which the list box does not report as a change.
	let close_on_choice = move |event: web_sys::KeyboardEvent| {
		if matches!(event.key().as_str(), "Enter" | " ") {
			is_open.set(false);
		}
	};

	view! {
		<div
			class=merge_class("leptos-aria-select-box", class)
			data-open=move || is_open.get().to_string()
			node_ref=root
		>
			<button
				type="button"
				class=merge_class("leptos-aria-select-box__trigger", trigger_class)
				role="combobox"
				aria-label=label.clone()
				aria-haspopup="listbox"
				aria-expanded=move || is_open.get().to_string()
				data-placeholder=move || selected_label.get().is_none().then_some("")
				disabled=move || is_disabled.get()
				node_ref=trigger
				on:click=move |_| is_open.update(|open| *open = !*open)
			>
				<span class="leptos-aria-select-box__value">
					{move || selected_label.get().unwrap_or_else(|| placeholder.clone())}
				</span>
				{children.map(|children| children())}
			</button>
			<div
				class=merge_class("leptos-aria-select-box__popup", popup_class)
				hidden=move || !is_open.get()
				on:keydown=close_on_choice
			>
				<ListBox nostrip:label=label selection=selection on_change=choose>
					{options
						.iter()
						.cloned()
						.map(|SelectBoxItem { value, label: text }| {
							let indicator = indicator.clone();

							view! {
								<ListBoxOption
									value=value
									class=option_class
									on:click=move |_| is_open.set(false)
								>
									{indicator.map(|indicator| indicator.run())}
									{text}
								</ListBoxOption>
							}
						})
						.collect_view()}
				</ListBox>
			</div>
		</div>
	}
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
	use leptos::mount::mount_to_body;
	use wasm_bindgen::JsCast;
	use wasm_bindgen_test::*;

	use super::*;

	wasm_bindgen_test_configure!(run_in_browser);

	fn sizes() -> Vec<SelectBoxItem> {
		vec![
			SelectBoxItem {
				value: "s".into(),
				label: "Small".into(),
			},
			SelectBoxItem {
				value: "m".into(),
				label: "Medium".into(),
			},
		]
	}

	fn find(selector: &str) -> web_sys::HtmlElement {
		document()
			.query_selector(selector)
			.unwrap()
			.unwrap_or_else(|| panic!("nothing matches {selector}"))
			.unchecked_into::<web_sys::HtmlElement>()
	}

	async fn tick() {
		let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
		wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
	}

	#[wasm_bindgen_test]
	async fn select_box_toggles_its_expanded_state() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! { <SelectBox label="Size".to_string() options=sizes() class="select-test-toggle" /> }
		});

		let trigger = find(".select-test-toggle [role='combobox']");
		let list = find(".select-test-toggle [role='listbox']");

		assert_eq!(
			trigger.get_attribute("aria-expanded").as_deref(),
			Some("false")
		);
		assert!(list.offset_parent().is_none(), "the list starts hidden");

		trigger.click();
		tick().await;

		assert_eq!(
			trigger.get_attribute("aria-expanded").as_deref(),
			Some("true"),
			"the trigger toggles the list"
		);
		assert!(list.offset_parent().is_some(), "the open list shows");
	}

	#[wasm_bindgen_test]
	async fn choosing_an_option_shows_it_reports_it_and_closes() {
		console_error_panic_hook::set_once();

		let reported = RwSignal::new(None::<String>);

		mount_to_body(move || {
			view! {
				<SelectBox
					label="Size".to_string()
					options=sizes()
					placeholder="Pick a size"
					on_change=Box::new(move |value| reported.set(value))
					class="select-test-choose"
				/>
			}
		});

		let trigger = find(".select-test-choose [role='combobox']");

		assert_eq!(trigger.text_content().as_deref(), Some("Pick a size"));
		assert!(trigger.has_attribute("data-placeholder"));

		trigger.click();
		tick().await;
		find(".select-test-choose [role='option']:last-child").click();
		tick().await;

		assert_eq!(reported.get_untracked().as_deref(), Some("m"));
		assert_eq!(trigger.text_content().as_deref(), Some("Medium"));
		assert!(!trigger.has_attribute("data-placeholder"));
		assert_eq!(
			find(".select-test-choose [role='option']:last-child")
				.get_attribute("aria-selected")
				.as_deref(),
			Some("true"),
			"the list box shows the selection"
		);
		assert_eq!(
			trigger.get_attribute("aria-expanded").as_deref(),
			Some("false"),
			"choosing closes the list"
		);
	}

	#[wasm_bindgen_test]
	async fn a_controlled_value_drives_the_trigger() {
		console_error_panic_hook::set_once();

		let value = RwSignal::new(Some("s".to_string()));

		mount_to_body(move || {
			view! { <SelectBox options=sizes() value=value class="select-test-controlled" /> }
		});

		let trigger = find(".select-test-controlled [role='combobox']");

		assert_eq!(trigger.text_content().as_deref(), Some("Small"));

		value.set(Some("m".to_string()));
		tick().await;

		assert_eq!(trigger.text_content().as_deref(), Some("Medium"));
	}

	#[wasm_bindgen_test]
	async fn a_press_outside_closes_the_list() {
		console_error_panic_hook::set_once();

		mount_to_body(|| view! { <SelectBox options=sizes() class="select-test-outside" /> });

		let trigger = find(".select-test-outside [role='combobox']");

		trigger.click();
		tick().await;

		let event = web_sys::Event::new("pointerdown").unwrap();
		document().dispatch_event(&event).unwrap();
		tick().await;

		assert_eq!(
			trigger.get_attribute("aria-expanded").as_deref(),
			Some("false")
		);
	}

	/// Wait up to a second for `done` to hold, yielding to the browser
	/// between checks so deferred focus can move.
	async fn wait_for(done: impl Fn() -> bool) -> bool {
		for _ in 0..100 {
			if done() {
				return true;
			}

			let promise = js_sys::Promise::new(&mut |resolve, _| {
				window()
					.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 10)
					.unwrap();
			});
			wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
		}

		done()
	}

	fn is_focused(element: &web_sys::HtmlElement) -> bool {
		document()
			.active_element()
			.is_some_and(|active| active.is_same_node(Some(element)))
	}

	/// Dispatch a bubbling `keydown` for `key`.
	fn key_down(target: &web_sys::Element, key: &str) {
		let init = web_sys::KeyboardEventInit::new();
		init.set_key(key);
		init.set_bubbles(true);

		let event =
			web_sys::KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &init).unwrap();

		target.dispatch_event(&event).unwrap();
	}

	#[wasm_bindgen_test]
	async fn focus_moves_into_the_list_and_back_to_the_trigger() {
		console_error_panic_hook::set_once();

		let reported = RwSignal::new(None::<String>);

		mount_to_body(move || {
			view! {
				<SelectBox
					options=sizes()
					on_change=Box::new(move |value| reported.set(value))
					class="select-test-focus"
				/>
			}
		});

		let trigger = find(".select-test-focus [role='combobox']");
		let list = find(".select-test-focus [role='listbox']");

		trigger.focus().unwrap();
		trigger.click();

		assert!(
			wait_for(|| is_focused(&list)).await,
			"opening focuses the list"
		);

		key_down(&list, "ArrowDown");
		key_down(&list, "Enter");

		assert!(
			wait_for(|| is_focused(&trigger)).await,
			"choosing returns focus to the trigger"
		);
		assert_eq!(reported.get_untracked().as_deref(), Some("s"));
		assert_eq!(
			trigger.get_attribute("aria-expanded").as_deref(),
			Some("false")
		);
	}
}
