#![allow(clippy::needless_pass_by_value)]

use std::sync::Arc;

use leptos::prelude::*;

/// An accordion is a vertically stacked set of expandable sections.
///
/// Sections toggle by trigger; in `single` mode opening one closes the
/// others, in `multiple` mode any set stays open. Each section's state is
/// exposed as `data-state="open"` or `"closed"` for styling.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum AccordionMode {
	/// One section open at a time.
	#[default]
	Single,
	/// Any number of sections open.
	Multiple,
}

/// The state an accordion shares with its items.
#[derive(Clone)]
pub struct AccordionContext {
	mode: AccordionMode,
	open_keys: RwSignal<Vec<String>>,
}

impl AccordionContext {
	fn toggle(&self, key: &str) {
		self.open_keys.update(|keys| {
			if keys.iter().any(|k| k == key) {
				keys.retain(|k| k != key);
			} else if self.mode == AccordionMode::Single {
				keys.clear();
				keys.push(key.to_string());
			} else {
				keys.push(key.to_string());
			}
		});
	}

	fn is_open(&self, key: &str) -> bool {
		self.open_keys.read().iter().any(|k| k == key)
	}
}

/// The per-item state handed to the trigger and panel.
struct AccordionItemState {
	context: AccordionContext,
	key: String,
	is_disabled: Signal<bool>,
}

impl AccordionItemState {
	fn open_state_label(&self) -> &'static str {
		if self.context.is_open(&self.key) {
			"open"
		} else {
			"closed"
		}
	}
}

/// The root of an accordion.
#[component]
pub fn Accordion(
	/// Whether one or many sections can be open, single by default.
	#[prop(optional, into)]
	mode: Signal<AccordionMode>,
	/// The keys of the initially open sections.
	#[prop(optional, into)]
	default_open: Vec<String>,
	children: Children,
) -> impl IntoView {
	let resolved_mode = mode.get_untracked();
	let context = AccordionContext {
		mode: resolved_mode,
		open_keys: RwSignal::new(default_open),
	};

	provide_context(context);

	view! {
		<div class="leptos-aria-accordion" data-mode=mode_label(resolved_mode)>
			{children()}
		</div>
	}
}

fn mode_label(mode: AccordionMode) -> &'static str {
	match mode {
		AccordionMode::Single => "single",
		AccordionMode::Multiple => "multiple",
	}
}

/// One expandable section of an accordion.
#[component]
pub fn AccordionItem(
	/// The section's key, unique within the accordion.
	#[prop(into)]
	key: String,
	/// Whether this section is disabled.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	children: Children,
) -> impl IntoView {
	let context = expect_context::<AccordionContext>();
	let item = Arc::new(AccordionItemState {
		context,
		key,
		is_disabled,
	});
	let item_state = Arc::clone(&item);
	let disabled_state = Arc::clone(&item);

	provide_context(Arc::clone(&item));

	view! {
		<div
			class="leptos-aria-accordion__item"
			data-state=move || item_state.open_state_label()
			data-disabled=move || disabled_state.is_disabled.get().to_string()
		>
			{children()}
		</div>
	}
}

/// The trigger toggling an [`AccordionItem`].
#[component]
pub fn AccordionTrigger(children: Children) -> impl IntoView {
	let item = expect_context::<Arc<AccordionItemState>>();
	let toggle_item = Arc::clone(&item);
	let label_item = Arc::clone(&item);
	let state_item = Arc::clone(&item);

	view! {
		<button
			type="button"
			class="leptos-aria-accordion__trigger"
			aria-expanded=move || {
				if label_item
					.context
					.is_open(&label_item.key) { "true" } else { "false" }
			}
			data-state=move || state_item.open_state_label()
			disabled=move || item.is_disabled.get()
			on:click=move |_| {
				if !toggle_item.is_disabled.get_untracked() {
					toggle_item.context.toggle(&toggle_item.key);
				}
			}
		>
			{children()}
		</button>
	}
}

/// The expandable content of an [`AccordionItem`].
#[component]
pub fn AccordionPanel(children: Children) -> impl IntoView {
	let item = expect_context::<Arc<AccordionItemState>>();
	let panel = Arc::clone(&item);
	let panel_state = Arc::clone(&item);

	view! {
		<div
			class="leptos-aria-accordion__panel"
			role="region"
			data-state=move || panel_state.open_state_label()
			hidden=move || !panel.context.is_open(&panel.key)
		>
			{children()}
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

	#[component]
	fn Example() -> impl IntoView {
		view! {
			<Accordion mode=Signal::from(AccordionMode::Single)>
				<AccordionItem key="one".to_string()>
					<AccordionTrigger>"One"</AccordionTrigger>
					<AccordionPanel>"First"</AccordionPanel>
				</AccordionItem>
				<AccordionItem key="two".to_string()>
					<AccordionTrigger>"Two"</AccordionTrigger>
					<AccordionPanel>"Second"</AccordionPanel>
				</AccordionItem>
			</Accordion>
		}
	}

	#[wasm_bindgen_test]
	async fn single_mode_opens_one_section_at_a_time() {
		console_error_panic_hook::set_once();

		mount_to_body(|| view! { <Example /> });

		let triggers = document()
			.query_selector_all(".leptos-aria-accordion__trigger")
			.unwrap();
		let first = triggers
			.get(0)
			.unwrap()
			.unchecked_into::<web_sys::Element>();
		let second = triggers
			.get(1)
			.unwrap()
			.unchecked_into::<web_sys::Element>();

		assert_eq!(
			first.get_attribute("aria-expanded").as_deref(),
			Some("false")
		);

		first
			.dispatch_event(&web_sys::Event::new("click").unwrap())
			.unwrap();

		let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
		wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();

		assert_eq!(
			first.get_attribute("aria-expanded").as_deref(),
			Some("true")
		);

		second
			.dispatch_event(&web_sys::Event::new("click").unwrap())
			.unwrap();

		let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
		wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();

		assert_eq!(
			second.get_attribute("aria-expanded").as_deref(),
			Some("true")
		);
		assert_eq!(
			first.get_attribute("aria-expanded").as_deref(),
			Some("false"),
			"single mode closes the first section"
		);
	}
}
