#![allow(clippy::needless_pass_by_value)]

use std::sync::Arc;

use leptos::prelude::*;
use leptos_aria_selection::SelectionMode;
use leptos_aria_selection::UseSelectionStateProps;
use leptos_aria_selection::use_selection_state;

/// The state a tree shares with its items through context.
#[derive(Clone)]
pub struct TreeContext {
	/// The shared selection, exposed for items choosing to select on click.
	pub selection: Arc<leptos_aria_selection::SelectionState>,
	expanded_keys: RwSignal<Vec<String>>,
}

impl TreeContext {
	/// Toggle the expansion of a tree item.
	pub fn toggle_expanded(&self, key: &str) {
		self.expanded_keys.update(|keys| {
			if keys.iter().any(|k| k == key) {
				keys.retain(|k| k != key);
			} else {
				keys.push(key.to_string());
			}
		});
	}

	/// Whether a tree item is expanded.
	///
	/// Reactive: reading inside an effect or view tracks the expansion.
	pub fn is_expanded(&self, key: &str) -> bool {
		self.expanded_keys.get().iter().any(|k| k == key)
	}

	/// Whether a tree item is expanded, without tracking.
	pub fn is_expanded_untracked(&self, key: &str) -> bool {
		self.expanded_keys.get_untracked().iter().any(|k| k == key)
	}
}

/// A tree presents a hierarchy of items expandable by branch.
#[component]
pub fn Tree(
	/// The content to display as the label.
	#[prop(optional)]
	label: Option<String>,
	children: Children,
) -> impl IntoView {
	let selection = Arc::new(use_selection_state(UseSelectionStateProps {
		mode: SelectionMode::Single,
		..Default::default()
	}));

	provide_context(TreeContext {
		selection,
		expanded_keys: RwSignal::new(Vec::new()),
	});

	let label_id = leptos_aria_utils::use_id();

	view! {
		{label.map(|label| view! { <span id=label_id.clone()>{label}</span> })}
		<div
			role="tree"
			aria-labelledby=label_id.clone()
			on:keydown=move |event: web_sys::KeyboardEvent| {
				if event.key() == "Home" || event.key() == "End" {
					event.prevent_default();
				}
			}
		>
			{children()}
		</div>
	}
}

/// A branch of a tree: an expandable item with children.
#[component]
pub fn TreeItem(
	/// The item's key, reported in the selection.
	#[prop(into)]
	key: String,
	children: Children,
) -> impl IntoView {
	let context = expect_context::<TreeContext>();
	let item_key = key.clone();
	let toggle_key = key.clone();
	let expand_context = context.clone();
	let expanded = Signal::derive(move || expand_context.is_expanded(&item_key));

	view! {
		<div
			role="treeitem"
			aria-expanded=move || expanded.get().to_string()
			data-key=key.clone()
			on:click=move |_| context.toggle_expanded(&toggle_key)
		>
			{children()}
		</div>
	}
}

/// The children of a tree branch, shown when expanded.
#[component]
pub fn TreeGroup(children: Children) -> impl IntoView {
	view! { <div role="group">{children()}</div> }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
	use leptos::mount::mount_to_body;
	use wasm_bindgen::JsCast;
	use wasm_bindgen_test::*;

	use super::*;

	wasm_bindgen_test_configure!(run_in_browser);

	#[wasm_bindgen_test]
	async fn tree_items_expand_on_click() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! {
				<Tree label="Files".to_string()>
					<TreeItem key="src".to_string()>"src"</TreeItem>
				</Tree>
			}
		});

		let item = document()
			.query_selector("[role='treeitem']")
			.unwrap()
			.unwrap()
			.unchecked_into::<web_sys::Element>();

		assert_eq!(
			item.get_attribute("aria-expanded").as_deref(),
			Some("false")
		);

		item.dispatch_event(&web_sys::Event::new("click").unwrap())
			.unwrap();

		let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
		wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();

		assert_eq!(
			item.get_attribute("aria-expanded").as_deref(),
			Some("true"),
			"clicking a branch toggles expansion"
		);
	}
}
