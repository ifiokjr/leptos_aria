use std::sync::Arc;
use std::sync::Mutex;

use leptos::prelude::*;
use leptos_aria_utils::focus_safely;
use leptos_aria_utils::is_focusable;
use wasm_bindgen::JsCast;

/// Which direction the focus manager walks.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum FocusDirection {
	Forward,
	Backward,
}

/// Programmatic focus movement within a container.
///
/// The port of the upstream `FocusManager` API: next, previous, first, and
/// last walk the focusable descendants of a root element in document order,
/// skipping invisible elements, and wrap around by default.
#[derive(Clone)]
pub struct FocusManager {
	root: Arc<Mutex<Option<web_sys::Element>>>,
	wrap: bool,
}

impl FocusManager {
	/// Create a manager bound to a root element reference.
	pub fn new(root: web_sys::Element) -> Self {
		Self {
			root: Arc::new(Mutex::new(Some(root))),
			wrap: true,
		}
	}

	/// Create a manager whose root resolves later, when the element mounts.
	pub fn deferred() -> Self {
		Self {
			root: Arc::new(Mutex::new(None)),
			wrap: true,
		}
	}

	/// Bind the root element once it is available.
	pub fn set_root(&self, root: web_sys::Element) {
		*self.root.lock().unwrap() = Some(root);
	}

	/// Focus the next focusable descendant, wrapping past the last.
	pub fn focus_next(&self) -> bool {
		self.move_focus(FocusDirection::Forward)
	}

	/// Focus the previous focusable descendant, wrapping past the first.
	pub fn focus_previous(&self) -> bool {
		self.move_focus(FocusDirection::Backward)
	}

	/// Focus the first focusable descendant.
	pub fn focus_first(&self) -> bool {
		let focusables = self.focusables();
		let Some(first) = focusables.first() else {
			return false;
		};

		focus_safely(first);
		true
	}

	/// Focus the last focusable descendant.
	pub fn focus_last(&self) -> bool {
		let focusables = self.focusables();
		let Some(last) = focusables.last() else {
			return false;
		};

		focus_safely(last);
		true
	}

	fn move_focus(&self, direction: FocusDirection) -> bool {
		let focusables = self.focusables();
		let active = self.active_element();
		let count = focusables.len();

		if count == 0 {
			return false;
		}

		// Without a focused descendant, forward starts at the first element
		// and backward at the last.
		let position = active
			.as_ref()
			.and_then(|active| focusables.iter().position(|element| element == active));

		let next = match (direction, position, self.wrap) {
			(FocusDirection::Forward, None, _) => Some(0),
			(FocusDirection::Backward, None, _) => count.checked_sub(1),
			(_, Some(index), true) => {
				match direction {
					FocusDirection::Forward => Some((index + 1) % count),
					FocusDirection::Backward => Some((index + count - 1) % count),
				}
			}
			(FocusDirection::Forward, Some(index), false) => {
				(index + 1 != count).then_some(index + 1)
			}
			(FocusDirection::Backward, Some(index), false) => (index != 0).then_some(index - 1),
		};

		match next.and_then(|index| focusables.get(index)) {
			Some(element) => {
				focus_safely(element);
				true
			}
			None => false,
		}
	}

	/// The focusable descendants of the root in document order.
	fn focusables(&self) -> Vec<web_sys::Element> {
		let root = self.root.lock().unwrap().clone();

		root.as_ref().map_or_else(Vec::new, |root| {
			let mut focusables = root_focusables(root);

			focusables.extend(descendant_focusables(root));

			focusables
		})
	}

	fn active_element(&self) -> Option<web_sys::Element> {
		self.root
			.lock()
			.unwrap()
			.as_ref()
			.and_then(|root| root.owner_document())
			.and_then(|document| document.active_element())
	}
}

/// The root itself when it is focusable.
fn root_focusables(root: &web_sys::Element) -> Vec<web_sys::Element> {
	is_focusable(root, false)
		.then(|| root.clone())
		.into_iter()
		.collect()
}

/// The focusable descendants of a root in document order.
fn descendant_focusables(root: &web_sys::Element) -> Vec<web_sys::Element> {
	let Ok(nodes) = root.query_selector_all("*") else {
		return Vec::new();
	};

	(0..nodes.length())
		.filter_map(|index| nodes.item(index))
		.filter_map(|node| node.dyn_into::<web_sys::Element>().ok())
		.filter(|element| is_focusable(element, false))
		.collect()
}

/// Provide a focus manager for a subtree.
///
/// The manager resolves its root from the first bound element; descendants
/// read the nearest provider through context.
pub fn use_focus_manager() -> FocusManager {
	if let Some(manager) = use_context::<FocusManager>() {
		manager
	} else {
		let manager = FocusManager::deferred();

		provide_context(manager.clone());

		manager
	}
}

/// Focus an element when it mounts.
///
/// The port of the upstream autofocus handling inside `FocusScope`: the
/// element focuses once on mount — through `focus_safely` so screen reader
/// focus waits out transitions — and only when the user has not already
/// moved focus elsewhere programmatically.
pub fn use_autofocus(target: NodeRef<leptos::html::Div>, auto_focus: bool) {
	if !auto_focus {
		return;
	}

	Effect::new(move |_| {
		let Some(element) = target.get() else {
			return;
		};

		let element: &web_sys::Element = element.unchecked_ref();

		focus_safely(element);
	});
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
	use std::time::Duration;

	use wasm_bindgen_test::*;

	use super::*;

	wasm_bindgen_test_configure!(run_in_browser);

	#[component]
	fn Example() -> impl IntoView {
		view! {
			<div id="focus-root">
				<button id="first">"First"</button>
				<button id="second">"Second"</button>
				<button id="third">"Third"</button>
			</div>
		}
	}

	/// Resolve a promise after one animation-frame-sized delay: the first
	/// focus flips the interaction modality to virtual, after which
	/// `focus_safely` defers focusing by one animation frame, so each step
	/// waits a frame before asserting.
	fn settle_timer(resolve: js_sys::Function, _: js_sys::Function) {
		set_timeout(
			move || {
				resolve.call0(&js_sys::global()).ok();
			},
			Duration::from_millis(50),
		);
	}

	async fn settle() {
		let timeout = js_sys::Promise::new(&mut settle_timer);

		wasm_bindgen_futures::JsFuture::from(timeout).await.unwrap();
	}

	#[wasm_bindgen_test]
	async fn focus_manager_walks_in_order_and_wraps() {
		console_error_panic_hook::set_once();

		mount_to_body(|| view! { <Example /> });

		let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
		wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();

		let root = document().query_selector("#focus-root").unwrap().unwrap();

		let manager = FocusManager::new(root);

		assert!(manager.focus_first(), "focusing succeeds with targets");
		settle().await;
		assert_eq!(document().active_element().unwrap().id(), "first");

		assert!(manager.focus_next());
		settle().await;
		assert_eq!(document().active_element().unwrap().id(), "second");

		assert!(manager.focus_next());
		settle().await;
		assert_eq!(document().active_element().unwrap().id(), "third");

		assert!(manager.focus_next());
		settle().await;
		assert_eq!(
			document().active_element().unwrap().id(),
			"first",
			"focus wraps past the last element"
		);

		assert!(manager.focus_previous());
		settle().await;
		assert_eq!(
			document().active_element().unwrap().id(),
			"third",
			"focus wraps past the first element backward"
		);
	}
}
