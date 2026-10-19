#![doc(html_logo_url = "https://ifiokjr.github.io/leptos_aria/logo.svg")]
#![doc(html_favicon_url = "https://ifiokjr.github.io/leptos_aria/favicon.ico")]
#![allow(clippy::needless_pass_by_value)]

use leptos::prelude::*;

/// State management for a disclosure: expanded content toggled by a button.
#[derive(Clone)]
pub struct DisclosureState {
	is_open: RwSignal<bool>,
	is_disabled: bool,
}

impl DisclosureState {
	/// Whether the content is expanded.
	///
	/// Reactive: reading inside an effect or view tracks the state.
	pub fn is_open(&self) -> bool {
		self.is_open.get()
	}

	/// Expand the content.
	pub fn open(&self) {
		self.set_open(true);
	}

	/// Collapse the content.
	pub fn close(&self) {
		self.set_open(false);
	}

	/// Expand or collapse the content, unless disabled.
	pub fn toggle(&self) {
		self.set_open(!self.is_open.get_untracked());
	}

	/// Set the expanded state directly, unless disabled.
	pub fn set_open(&self, is_open: bool) {
		if !self.is_disabled {
			self.is_open.set(is_open);
		}
	}
}

/// The options accepted by [`use_disclosure_state`].
#[derive(Default)]
pub struct UseDisclosureStateProps {
	/// Whether the content is initially expanded.
	pub default_open: bool,

	/// Whether the disclosure is disabled.
	pub is_disabled: bool,
}

/// Provides state management for a disclosure.
pub fn use_disclosure_state(props: UseDisclosureStateProps) -> DisclosureState {
	DisclosureState {
		is_open: RwSignal::new(props.default_open),
		is_disabled: props.is_disabled,
	}
}

/// A disclosure is a toggleable section of content.
#[component]
pub fn Disclosure(
	/// Whether the content is initially expanded (uncontrolled).
	#[prop(optional, into)]
	default_open: Signal<bool>,
	children: Children,
) -> impl IntoView {
	let state = use_disclosure_state(UseDisclosureStateProps {
		default_open: default_open.get_untracked(),
		is_disabled: false,
	});

	provide_context(state.clone());

	let panel_id = leptos_aria_utils::use_id();
	let button_id = leptos_aria_utils::use_id();
	let expanded_state = state.clone();
	let panel_state = state.clone();
	let toggle_state = state;

	view! {
		<div class="leptos-aria-disclosure">
			<button
				id=button_id.clone()
				type="button"
				class="leptos-aria-disclosure__trigger"
				aria-expanded=move || expanded_state.is_open().to_string()
				aria-controls=panel_id.clone()
				on:click=move |_| toggle_state.toggle()
			>
				{children()}
			</button>
			<div
				id=panel_id.clone()
				class="leptos-aria-disclosure__panel"
				role="region"
				aria-labelledby=button_id.clone()
				hidden=move || !panel_state.is_open()
			>
			</div>
		</div>
	}
}

/// The panel content of a disclosure; rendered inside the hidden region.
#[component]
pub fn DisclosurePanel(children: Children) -> impl IntoView {
	view! { <div class="leptos-aria-disclosure__content">{children()}</div> }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
	use leptos::mount::mount_to_body;
	use wasm_bindgen::JsCast;
	use wasm_bindgen_test::*;

	use super::*;

	wasm_bindgen_test_configure!(run_in_browser);

	#[wasm_bindgen_test]
	async fn disclosure_toggles_the_panel() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! { <Disclosure>"More info"</Disclosure> }
		});

		let trigger = document()
			.query_selector(".leptos-aria-disclosure__trigger")
			.unwrap()
			.unwrap()
			.unchecked_into::<web_sys::HtmlButtonElement>();

		let panel = document()
			.query_selector(".leptos-aria-disclosure__panel")
			.unwrap()
			.unwrap();

		assert!(panel.has_attribute("hidden"), "the panel starts hidden");

		trigger.click();

		let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
		wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();

		assert!(!panel.has_attribute("hidden"), "the panel expands");
		assert_eq!(
			trigger.get_attribute("aria-expanded").as_deref(),
			Some("true")
		);
	}
}
