#![doc(html_logo_url = "https://ifiokjr.github.io/leptos_aria/logo.svg")]
#![doc(html_favicon_url = "https://ifiokjr.github.io/leptos_aria/favicon.ico")]
#![allow(clippy::needless_pass_by_value)]

use leptos::prelude::*;
use leptos_aria_utils::merge_class;
use leptos_aria_utils::use_id;
use wasm_bindgen::JsCast;

/// The id of the tab with this key, which its panel is labelled by.
fn tab_id(key: &str) -> String {
	format!("leptos-aria-tab-{key}")
}

/// The id of the panel of the tab with this key.
fn panel_id(key: &str) -> String {
	format!("leptos-aria-tab-panel-{key}")
}

/// A registered tab: its key and whether it is disabled.
#[derive(Clone)]
struct TabEntry {
	key: String,
	is_disabled: Signal<bool>,
}

/// The state a tab list shares with its tabs and panels through context.
#[derive(Clone)]
pub struct TabsContext {
	selected_key: RwSignal<Option<String>>,
	tabs: RwSignal<Vec<TabEntry>>,
}

impl TabsContext {
	/// Register an enabled tab's key in document order.
	pub fn register_key(&self, key: &str) {
		self.register_tab(key, Signal::from(false));
	}

	/// Register a tab in document order with its disabled state, which
	/// keyboard navigation and selection respect.
	pub fn register_tab(&self, key: &str, is_disabled: Signal<bool>) {
		self.tabs.update(|tabs| {
			tabs.push(TabEntry {
				key: key.to_string(),
				is_disabled,
			});
		});
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
	/// order, the others do not. While nothing is selected, the first
	/// enabled tab takes part so keyboard users can reach the list.
	///
	/// Reactive: reading inside an effect or view tracks the selection and
	/// the registered tabs.
	pub fn tab_index_for(&self, key: &str) -> i32 {
		let focusable = match self.selected_key.get() {
			Some(selected) => selected == key,
			None => {
				self.tabs.with(|tabs| {
					tabs.iter()
						.find(|tab| !tab.is_disabled.get())
						.is_some_and(|tab| tab.key == key)
				})
			}
		};

		if focusable { 0 } else { -1 }
	}

	/// Select the tab with this key, unless it is missing or disabled.
	pub fn select(&self, key: &str) {
		let enabled = self.tabs.with_untracked(|tabs| {
			tabs.iter()
				.any(|tab| tab.key == key && !tab.is_disabled.get_untracked())
		});

		if enabled {
			self.selected_key.set(Some(key.to_string()));
		}
	}

	/// Move the selection to the next enabled tab after the selected one,
	/// wrapping.
	pub fn select_next(&self) {
		self.move_selection(1);
	}

	/// Move the selection to the next enabled tab before the selected one,
	/// wrapping.
	pub fn select_previous(&self) {
		self.move_selection(-1);
	}

	/// Move keyboard focus to the tab with this key, if it is mounted.
	///
	/// Tabs rove: only the selected tab is in the tab order, so focus has to
	/// travel with the selection for the next key press to reach the right
	/// tab.
	pub fn focus_tab(&self, key: &str) {
		let tab = document()
			.get_element_by_id(&tab_id(key))
			.and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok());

		if let Some(tab) = tab {
			// Focusing an element that cannot take focus is a no-op, not an error.
			let _ = tab.focus();
		}
	}

	/// Move keyboard focus to the selected tab.
	pub fn focus_selected(&self) {
		if let Some(key) = self.selected_key.get_untracked() {
			self.focus_tab(&key);
		}
	}

	/// Select the first enabled tab.
	pub fn select_first(&self) {
		let first = self.tabs.with_untracked(|tabs| {
			tabs.iter()
				.find(|tab| !tab.is_disabled.get_untracked())
				.map(|tab| tab.key.clone())
		});

		if first.is_some() {
			self.selected_key.set(first);
		}
	}

	/// Select the last enabled tab.
	pub fn select_last(&self) {
		let last = self.tabs.with_untracked(|tabs| {
			tabs.iter()
				.rev()
				.find(|tab| !tab.is_disabled.get_untracked())
				.map(|tab| tab.key.clone())
		});

		if last.is_some() {
			self.selected_key.set(last);
		}
	}

	/// Step `offset` tabs at a time from the selected one, wrapping and
	/// passing over disabled tabs. Without a selection, stepping forward
	/// starts before the first tab and stepping back after the last.
	fn move_selection(&self, offset: isize) {
		let selected = self.selected_key.get_untracked();
		let next = self.tabs.with_untracked(|tabs| {
			let length = tabs.len() as isize;
			let start = selected
				.as_deref()
				.and_then(|key| tabs.iter().position(|tab| tab.key == key))
				.map_or(if offset > 0 { -1 } else { length }, |index| index as isize);

			(1..=length)
				.filter_map(|step| tabs.get((start + offset * step).rem_euclid(length) as usize))
				.find(|tab| !tab.is_disabled.get_untracked())
				.map(|tab| tab.key.clone())
		});

		if next.is_some() {
			self.selected_key.set(next);
		}
	}
}

/// A tab list lets the user switch between related panels of content.
///
/// Arrow keys move the selection between enabled tabs, home and end select
/// the first and last enabled tab, and keyboard focus moves with the
/// selection. Only the selected tab participates in the tab order.
#[component]
pub fn Tabs(
	/// The key of the initially selected tab (uncontrolled).
	#[prop(optional, into)]
	default_selected_key: Option<String>,
	/// The content to display as the label.
	#[prop(optional)]
	label: Option<String>,
	/// Classes for the root element.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let context = TabsContext {
		selected_key: RwSignal::new(default_selected_key),
		tabs: RwSignal::new(Vec::new()),
	};

	provide_context(context.clone());

	let label_id = use_id();

	view! {
		{label.map(|label| view! { <span id=label_id.clone()>{label}</span> })}
		<div class=merge_class("", class) role="tablist" aria-labelledby=label_id.clone() on:keydown=move |event: web_sys::KeyboardEvent| {
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
	context.focus_selected();
}

/// An individual tab inside a tab list.
///
/// `data-focused` is `true` while the tab has keyboard focus, which is not
/// the same as being selected: style selection with `aria-selected`.
#[component]
pub fn Tab(
	/// The key of the tab, matching its panel.
	#[prop(into)]
	key: String,
	/// Whether the tab is disabled.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	/// Classes for the root element.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let context = expect_context::<TabsContext>();
	context.register_tab(&key, is_disabled);

	let tab_id = tab_id(&key);
	let panel_id = panel_id(&key);
	let is_selected = Signal::derive({
		let selected = context.selected_key;
		let key = key.clone();

		move || selected.with(|selected| selected.as_deref() == Some(key.as_str()))
	});
	let tab_key = key.clone();
	let tabindex_context = context.clone();
	let tabindex_key = tab_key.clone();
	let tabindex = Signal::derive(move || tabindex_context.tab_index_for(&tabindex_key));
	let is_focused = RwSignal::new(false);

	view! {
		<button
			class=merge_class("", class)
			id=tab_id.clone()
			type="button"
			role="tab"
			aria-selected=move || is_selected.get().to_string()
			aria-disabled=move || is_disabled.get().to_string()
			aria-controls=panel_id.clone()
			tabindex=move || tabindex.get()
			data-focused=move || is_focused.get().to_string()
			on:focus=move |_| is_focused.set(true)
			on:blur=move |_| is_focused.set(false)
			on:click={
				let context = context.clone();
				let key = tab_key.clone();

				move |_| {
					context.select(&key);

					// Safari does not focus a button on click, and a disabled tab
					// takes neither the selection nor focus.
					if context.is_selected_untracked(&key) {
						context.focus_tab(&key);
					}
				}
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
	/// Classes for the root element.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let context = expect_context::<TabsContext>();
	let is_selected = Signal::derive({
		let selected = context.selected_key;
		let key = key.clone();

		move || selected.with(|selected| selected.as_deref() == Some(key.as_str()))
	});
	let labelledby = aria_labelledby.unwrap_or_else(|| tab_id(&key));

	view! {
		<div
			class=merge_class("", class)
			id=panel_id(&key)
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

		mount_to_body(|| {
			view! {
				<div id="tabs-test-example">
					<Example />
				</div>
			}
		});

		let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
		wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();

		let tabs = document()
			.query_selector_all("#tabs-test-example [role='tab']")
			.unwrap();
		let first = tabs.get(0).unwrap().unchecked_into::<web_sys::Element>();
		let second = tabs.get(1).unwrap().unchecked_into::<web_sys::Element>();

		let panels = document()
			.query_selector_all("#tabs-test-example [role='tabpanel']")
			.unwrap();
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

		assert_eq!(second.get_attribute("tabindex"), Some("0".to_string()));
		assert_eq!(
			first.get_attribute("tabindex").as_deref(),
			Some("-1"),
			"the unselected tab leaves the tab order"
		);
	}

	fn tab(selector: &str) -> web_sys::HtmlElement {
		document()
			.query_selector(selector)
			.unwrap()
			.unwrap()
			.unchecked_into::<web_sys::HtmlElement>()
	}

	async fn tick() {
		let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
		wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
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
	async fn arrow_keys_skip_disabled_tabs() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! {
				<Tabs default_selected_key="skip-a" class="tabs-test-skip">
					<Tab key="skip-a">"A"</Tab>
					<Tab key="skip-b" is_disabled=true>"B"</Tab>
					<Tab key="skip-c">"C"</Tab>
				</Tabs>
			}
		});

		let list = tab(".tabs-test-skip");
		key_down(&list, "ArrowRight");
		tick().await;

		assert_eq!(
			tab("#leptos-aria-tab-skip-c")
				.get_attribute("aria-selected")
				.as_deref(),
			Some("true"),
			"the arrow passes over the disabled tab"
		);

		tab("#leptos-aria-tab-skip-b").click();
		tick().await;

		assert_eq!(
			tab("#leptos-aria-tab-skip-b")
				.get_attribute("aria-selected")
				.as_deref(),
			Some("false"),
			"a disabled tab ignores clicks"
		);
	}

	#[wasm_bindgen_test]
	fn without_a_selection_the_first_enabled_tab_is_focusable() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! {
				<Tabs>
					<Tab key="unselected-a" is_disabled=true>"A"</Tab>
					<Tab key="unselected-b">"B"</Tab>
					<Tab key="unselected-c">"C"</Tab>
				</Tabs>
			}
		});

		assert_eq!(
			tab("#leptos-aria-tab-unselected-a")
				.get_attribute("tabindex")
				.as_deref(),
			Some("-1")
		);
		assert_eq!(
			tab("#leptos-aria-tab-unselected-b")
				.get_attribute("tabindex")
				.as_deref(),
			Some("0"),
			"keyboard users can reach the tab list"
		);
		assert_eq!(
			tab("#leptos-aria-tab-unselected-c")
				.get_attribute("tabindex")
				.as_deref(),
			Some("-1")
		);
	}

	/// The id of the element that has focus.
	fn active_id() -> Option<String> {
		document().active_element().map(|element| element.id())
	}

	#[wasm_bindgen_test]
	async fn keyboard_navigation_moves_focus_with_the_selection() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! {
				<Tabs default_selected_key="roving-a">
					<Tab key="roving-a">"A"</Tab>
					<Tab key="roving-b" is_disabled=true>"B"</Tab>
					<Tab key="roving-c">"C"</Tab>
					<Tab key="roving-d">"D"</Tab>
				</Tabs>
			}
		});

		let id = |key: &str| format!("leptos-aria-tab-roving-{key}");
		tab("#leptos-aria-tab-roving-a").focus().unwrap();

		key_down(&tab("#leptos-aria-tab-roving-a"), "ArrowRight");
		tick().await;
		assert_eq!(
			active_id(),
			Some(id("c")),
			"focus follows the selection and passes over the disabled tab"
		);
		assert_eq!(
			tab("#leptos-aria-tab-roving-c")
				.get_attribute("aria-selected")
				.as_deref(),
			Some("true")
		);

		key_down(&tab("#leptos-aria-tab-roving-c"), "End");
		tick().await;
		assert_eq!(active_id(), Some(id("d")), "End focuses the last tab");

		key_down(&tab("#leptos-aria-tab-roving-d"), "ArrowRight");
		tick().await;
		assert_eq!(
			active_id(),
			Some(id("a")),
			"the last tab wraps to the first"
		);

		key_down(&tab("#leptos-aria-tab-roving-a"), "ArrowLeft");
		tick().await;
		assert_eq!(
			active_id(),
			Some(id("d")),
			"the first tab wraps to the last"
		);

		key_down(&tab("#leptos-aria-tab-roving-d"), "Home");
		tick().await;
		assert_eq!(active_id(), Some(id("a")), "Home focuses the first tab");
	}

	#[wasm_bindgen_test]
	async fn clicking_a_tab_focuses_it() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! {
				<Tabs default_selected_key="click-a">
					<Tab key="click-a">"A"</Tab>
					<Tab key="click-b">"B"</Tab>
					<Tab key="click-c" is_disabled=true>"C"</Tab>
				</Tabs>
			}
		});

		// A scripted click does not move focus by itself, and Safari does not
		// focus a button on click either, so the tab has to.
		tab("#leptos-aria-tab-click-b").click();
		tick().await;
		assert_eq!(active_id().as_deref(), Some("leptos-aria-tab-click-b"));

		tab("#leptos-aria-tab-click-c").click();
		tick().await;
		assert_eq!(
			active_id().as_deref(),
			Some("leptos-aria-tab-click-b"),
			"a disabled tab takes neither the selection nor focus"
		);
	}

	#[wasm_bindgen_test]
	async fn data_focused_reports_the_tab_that_has_focus() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! {
				<Tabs default_selected_key="reported-a">
					<Tab key="reported-a">"A"</Tab>
					<Tab key="reported-b">"B"</Tab>
				</Tabs>
			}
		});

		let data_focused = |key: &str| {
			tab(&format!("#leptos-aria-tab-reported-{key}")).get_attribute("data-focused")
		};

		assert_eq!(
			data_focused("a").as_deref(),
			Some("false"),
			"a selected tab is not focused until it receives focus"
		);

		tab("#leptos-aria-tab-reported-a").focus().unwrap();
		tick().await;
		assert_eq!(data_focused("a").as_deref(), Some("true"));
		assert_eq!(data_focused("b").as_deref(), Some("false"));

		key_down(&tab("#leptos-aria-tab-reported-a"), "ArrowRight");
		tick().await;
		assert_eq!(data_focused("a").as_deref(), Some("false"));
		assert_eq!(data_focused("b").as_deref(), Some("true"));

		tab("#leptos-aria-tab-reported-b").blur().unwrap();
		tick().await;
		assert_eq!(
			data_focused("b").as_deref(),
			Some("false"),
			"a selected tab stops reporting focus once it loses it"
		);
	}
}
