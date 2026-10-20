#![doc(html_logo_url = "https://ifiokjr.github.io/leptos_aria/logo.svg")]
#![doc(html_favicon_url = "https://ifiokjr.github.io/leptos_aria/favicon.ico")]
#![allow(clippy::needless_pass_by_value)]

use std::sync::Arc;
use std::time::Duration;

use leptos::prelude::*;
use leptos_aria_utils::merge_class;

/// How long a toast stays on screen before expiring, five seconds by
/// default.
const DEFAULT_EXPIRY: Duration = Duration::from_millis(5000);

/// One toast in a queue.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Toast {
	/// The programmatic key of the toast.
	pub key: String,

	/// The text announced and displayed.
	pub message: String,

	/// How the toast is announced: polite waits for the current speech,
	/// assertive interrupts it.
	pub politeness: ToastPoliteness,

	/// How long the toast stays before expuring; the default of five seconds
	/// when `None`.
	pub expiry: Option<Duration>,
}

/// How a toast is announced to assistive technology.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum ToastPoliteness {
	/// The announcement waits for the current speech to finish.
	#[default]
	Polite,
	/// The announcement interrupts the current speech.
	Assertive,
}

impl ToastPoliteness {
	fn as_str(self) -> &'static str {
		match self {
			ToastPoliteness::Polite => "polite",
			ToastPoliteness::Assertive => "assertive",
		}
	}
}

/// The toast queue shared between the region and the toasts' close buttons.
#[derive(Clone, Default)]
pub struct ToastQueue {
	toasts: RwSignal<Vec<Toast>>,
}

impl ToastQueue {
	/// Create an empty queue.
	pub fn new() -> Self {
		Self::default()
	}

	/// Add a toast, expiring it after its delay.
	pub fn push(&self, mut toast: Toast) {
		if toast.key.is_empty() {
			toast.key = leptos_aria_utils::use_id();
		}

		let expiry = toast.expiry.unwrap_or(DEFAULT_EXPIRY);
		let key = toast.key.clone();

		self.toasts.update(|toasts| toasts.push(toast));

		let queue = Arc::new(self.clone());

		set_timeout(move || queue.remove(&key), expiry);
	}

	/// Add a toast from just its message.
	pub fn push_message(&self, message: impl Into<String>) {
		self.push(Toast {
			key: String::new(),
			message: message.into(),
			politeness: ToastPoliteness::Polite,
			expiry: None,
		});
	}

	/// Remove the toast with this key.
	pub fn remove(&self, key: &str) {
		self.toasts
			.update(|toasts| toasts.retain(|toast| toast.key != key));
	}

	/// Remove every toast.
	pub fn clear(&self) {
		self.toasts.set(Vec::new());
	}

	/// The queued toasts.
	///
	/// Reactive: reading inside an effect or view tracks the queue.
	pub fn toasts(&self) -> Vec<Toast> {
		self.toasts.get()
	}

	/// The queued toasts, without tracking.
	pub fn toasts_untracked(&self) -> Vec<Toast> {
		self.toasts.get_untracked()
	}

	/// How many toasts are queued.
	pub fn len(&self) -> usize {
		self.toasts_untracked().len()
	}

	/// Whether no toasts are queued.
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}
}

/// The toast queue shared through context by a [`ToastRegion`].
#[derive(Clone)]
pub struct ToastRegionContext {
	pub queue: ToastQueue,
}

/// Create the toast queue for a region and provide it to descendants.
#[component]
pub fn ToastRegion(
	/// Classes added after the built-in `leptos-aria-toast-region` class.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let queue = ToastQueue::new();

	provide_context(ToastRegionContext {
		queue: queue.clone(),
	});

	let toasts = queue.toasts;

	view! {
		{children()}
		<div
			class=merge_class("leptos-aria-toast-region", class)
			role="region"
			aria-label="Notifications"
			aria-live="polite"
		>
			<ol class="leptos-aria-toast-region__list">
				{move || {
					toasts
						.get()
						.into_iter()
						.map(|toast| {
							let close_queue = queue.clone();
							let close_key = toast.key.clone();
							let politeness = toast.politeness.as_str();
							let message = toast.message.clone();

							view! {
								<li class="leptos-aria-toast" aria-live={politeness}>
									{message}
									<button
										type="button"
										class="leptos-aria-toast__close"
										aria-label="Dismiss"
										on:click=move |_| close_queue.remove(&close_key)
									>
										"✕"
									</button>
								</li>
							}
						})
						.collect::<Vec<_>>()
				}}
			</ol>
		</div>
	}
}

/// Push a toast onto the nearest [`ToastRegion`] queue.
///
/// Panics when no region is above the caller, matching `expect_context`.
pub fn use_toast_queue() -> ToastQueue {
	expect_context::<ToastRegionContext>().queue
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
		let queue = use_toast_queue();

		view! {
			<button on:click=move |_| queue.push_message("Saved")>"Save"</button>
		}
	}

	#[wasm_bindgen_test]
	async fn toasts_appear_and_dismiss() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! {
				<ToastRegion>
					<Example />
				</ToastRegion>
			}
		});

		let trigger = document()
			.query_selector("button:not([aria-label])")
			.unwrap()
			.unwrap()
			.unchecked_into::<web_sys::HtmlButtonElement>();

		assert!(
			document()
				.query_selector(".leptos-aria-toast")
				.unwrap()
				.is_none(),
			"no toast is shown initially"
		);

		trigger.click();

		let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
		wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();

		let toast = document().query_selector(".leptos-aria-toast").unwrap();

		assert!(toast.is_some(), "the toast appears");

		let dismiss = document()
			.query_selector(".leptos-aria-toast__close")
			.unwrap()
			.unwrap()
			.unchecked_into::<web_sys::HtmlButtonElement>();

		dismiss.click();

		let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
		wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();

		assert!(
			document()
				.query_selector(".leptos-aria-toast")
				.unwrap()
				.is_none(),
			"the toast dismisses"
		);
	}
}
