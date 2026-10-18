#![allow(clippy::needless_pass_by_value)]

use leptos::prelude::*;
use leptos_aria_utils::use_id;

/// The state a tab list shares with its tabs and panels through context.
#[derive(Clone)]
pub struct TabsContext {
	selected_key: RwSignal<Option<String>>,
	keys: RwSignal<Vec<String>>,
}

impl TabsContext {
	/// Register a tab's key in document order.
	pub fn register_key(&self, key: &str) {
		self.keys.update(|keys| keys.push(key.to_string()));
	}

	/// The currently selected tab key.
	///
	/// Reactive: reading inside an effect or view tracks the selection.
	pub fn selected_key(&self) -> Option<String> {
		self.selected_key.get()
	}

	/// Whether the tab with this key is selected.
	///
	/// Reactive: reading inside an effect or view tracks the selection.
	pub fn is_selected(&self, key: &str) -> bool {
		self.selected_key.get().as_deref() == Some(key)
	}

	/// Whether the tab with this key is selected, without tracking.
	pub fn is_selected_untracked(&self, key: &str) -> bool {
		self.selected_key.get_untracked().as_deref() == Some(key)
	}

	/// The roving tab index of a tab: the selected tab participates in tab
	/// order, the others do not.
	pub fn tab_index_for(&self, key: &str) -> i32 {
		if self.is_selected(key) { 0 } else { -1 }
	}

	/// Select the tab with this key.
	pub fn select(&self, key: &str) {
		if self.keys.get_untracked().iter().any(|k| k == key) {
			self.selected_key.set(Some(key.to_string()));
		}
	}

	/// Move the selection to the tab after the selected one, wrapping.
	pub fn select_next(&self) {
		self.move_selection(1);
	}

	/// Move the selection to the tab before the selected one, wrapping.
	pub fn select_previous(&self) {
		self.move_selection(-1);
	}

	/// Select the first tab.
	pub fn select_first(&self) {
		let first = self.keys.get_untracked().first().cloned();
		self.selected_key.set(first);
	}

	/// Select the last tab.
	pub fn select_last(&self) {
		let last = self.keys.get_untracked().last().cloned();
		self.selected_key.set(last);
	}

	fn move_selection(&self, offset: isize) {
		let keys = self.keys.get_untracked();
		let current = self.selected_key.get_untracked();

		let index = current
			.as_deref()
			.and_then(|key| keys.iter().position(|k| k == key))
			.unwrap_or(0);

		let length = keys.len();

		if length == 0 {
			return;
		}

		let next = (index as isize + offset).rem_euclid(length as isize) as usize;
		self.selected_key.set(keys.get(next).cloned());
	}
}

/// A tab list lets the user switch between related panels of content.
///
/// Arrow keys move the selection between tabs, home and end select the first
/// and last tab, and only the selected tab participates in the tab order.
#[component]
pub fn Tabs(
	/// The key of the initially selected tab (uncontrolled).
	#[prop(optional, into)]
	default_selected_key: Option<String>,
	/// The content to display as the label.
	#[prop(optional)]
	label: Option<String>,
	children: Children,
) -> impl IntoView {
	let context = TabsContext {
		selected_key: RwSignal::new(default_selected_key),
		keys: RwSignal::new(Vec::new()),
	};

	provide_context(context.clone());

	let label_id = use_id();

	view! {
		{label.map(|label| view! { <span id=label_id.clone()>{label}</span> })}
		<div role="tablist" aria-labelledby=label_id.clone() on:keydown=move |event: web_sys::KeyboardEvent| {
			handle_key_down(&context, &event);
		}>
			{children()}
		</div>
	}
}

/// The keyboard interaction of the tab list.
fn handle_key_down(context: &TabsContext, event: &web_sys::KeyboardEvent) {
	match event.key().as_str() {
		"ArrowRight" | "ArrowDown" => context.select_next(),
		"ArrowLeft" | "ArrowUp" => context.select_previous(),
		"Home" => context.select_first(),
		"End" => context.select_last(),
		_ => return,
	}

	event.prevent_default();
}

/// An individual tab inside a tab list.
#[component]
pub fn Tab(
	/// The key of the tab, matching its panel.
	#[prop(into)]
	key: String,
	/// Whether the tab is disabled.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	children: Children,
) -> impl IntoView {
	let context = expect_context::<TabsContext>();
	context.register_key(&key);

	let tab_id = format!("leptos-aria-tab-{key}");
	let panel_id = format!("leptos-aria-tab-panel-{key}");
	let is_selected = Signal::derive({
		let selected = context.selected_key;
		let key = key.clone();

		move || selected.get().as_deref() == Some(key.as_str())
	});
	let tab_key = key.clone();
	let tabindex_context = context.clone();
	let tabindex_key = tab_key.clone();
	let tabindex = Signal::derive(move || tabindex_context.tab_index_for(&tabindex_key));

	view! {
		<button
			id=tab_id.clone()
			type="button"
			role="tab"
			aria-selected=move || is_selected.get().to_string()
			aria-disabled=move || is_disabled.get().to_string()
			aria-controls=panel_id.clone()
			tabindex=move || tabindex.get()
			data-focused=move || is_selected.get().to_string()
			on:click={
				let context = context.clone();
				let key = tab_key.clone();

				move |_| context.select(&key)
			}
		>
			{children()}
		</button>
	}
}

/// The content panel of a tab.
#[component]
pub fn TabPanel(
	/// The key of the tab this panel belongs to.
	#[prop(into)]
	key: String,
	/// The id of the element labeling the panel; the matching tab by default.
	#[prop(optional, into)]
	aria_labelledby: Option<String>,
	children: Children,
) -> impl IntoView {
	let context = expect_context::<TabsContext>();
	let is_selected = Signal::derive({
		let selected = context.selected_key;
		let key = key.clone();

		move || selected.get().as_deref() == Some(key.as_str())
	});
	let labelledby = aria_labelledby.unwrap_or_else(|| format!("leptos-aria-tab-{key}"));

	view! {
		<div
			id=format!("leptos-aria-tab-panel-{key}")
			role="tabpanel"
			aria-labelledby=labelledby
			hidden=move || !is_selected.get()
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
			<Tabs>
				<Tab key="one".to_string()>"One"</Tab>
				<Tab key="two".to_string()>"Two"</Tab>
			</Tabs>
			<TabPanel key="one".to_string()>"First panel"</TabPanel>
			<TabPanel key="two".to_string()>"Second panel"</TabPanel>
		}
	}

	#[wasm_bindgen_test]
	async fn tabs_switch_panels_and_rove_the_tab_index() {
		console_error_panic_hook::set_once();

		mount_to_body(|| view! { <Example /> });

		let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
		wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();

		let tabs = document().query_selector_all("[role='tab']").unwrap();
		let first = tabs.get(0).unwrap().unchecked_into::<web_sys::Element>();
		let second = tabs.get(1).unwrap().unchecked_into::<web_sys::Element>();

		let panels = document().query_selector_all("[role='tabpanel']").unwrap();
		let first_panel = panels.get(0).unwrap().unchecked_into::<web_sys::Element>();
		let second_panel = panels.get(1).unwrap().unchecked_into::<web_sys::Element>();

		second
			.dispatch_event(&web_sys::Event::new("click").unwrap())
			.unwrap();

		let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
		wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();

		assert_eq!(
			second.get_attribute("aria-selected").as_deref(),
			Some("true")
		);
		assert_eq!(
			first.get_attribute("aria-selected").as_deref(),
			Some("false")
		);
		assert!(first_panel.has_attribute("hidden"), "the first panel hides");
		assert!(
			!second_panel.has_attribute("hidden"),
			"the second panel shows"
		);

		web_sys::console::log_1(&format!("DBG second tab html={}", second.outer_html()).into());

		assert_eq!(second.get_attribute("tabindex"), Some("0".to_string()));
		assert_eq!(
			first.get_attribute("tabindex").as_deref(),
			Some("-1"),
			"the unselected tab leaves the tab order"
		);
	}
}
