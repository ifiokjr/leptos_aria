#![doc(html_logo_url = "https://ifiokjr.github.io/leptos_aria/logo.svg")]
#![doc(html_favicon_url = "https://ifiokjr.github.io/leptos_aria/favicon.ico")]

use std::sync::Arc;
use std::sync::Mutex;

use leptos::prelude::*;
use leptos_aria_utils::merge_class;

/// The form context provided by the [`Form`] component.
///
/// Controls inside the form register a reset callback keyed by name, and the
/// form's `reset` event replays their initial values — the DOM resets native
/// controls by itself; this extends the behavior to signal-driven state.
#[derive(Clone, Default)]
pub struct FormContext {
	reset_handlers: Arc<Mutex<Vec<FormResetHandler>>>,
}

struct FormResetHandler {
	name: String,
	handler: Handler,
}

#[derive(Clone)]
pub struct Handler(pub Arc<dyn Fn() + Send + Sync>);

impl std::ops::Deref for Handler {
	type Target = Arc<dyn Fn() + Send + Sync>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

/// Clone the handlers without holding the lock during the callbacks.
trait CloneHandlers {
	fn clone_shallow(&self) -> Vec<FormResetHandler>;
}

impl CloneHandlers for Vec<FormResetHandler> {
	fn clone_shallow(&self) -> Vec<FormResetHandler> {
		self.iter()
			.map(|handler| {
				FormResetHandler {
					name: handler.name.clone(),
					handler: handler.handler.clone(),
				}
			})
			.collect()
	}
}

/// Register a reset callback for a named control, active until the calling
/// component is disposed.
///
/// A port of the `react-aria` `useFormReset` hook, generalized from form
/// controls to any named state.
pub fn use_form_reset(
	context: &FormContext,
	name: &str,
	on_reset: impl Fn() + Send + Sync + 'static,
) {
	context.register(name, Handler(Arc::new(on_reset)));

	let unregister_context = context.clone();
	let unregister_name = name.to_string();

	on_cleanup(move || unregister_context.unregister(&unregister_name));
}

impl FormContext {
	/// Register a reset handler for a named control.
	pub fn register(&self, name: &str, handler: Handler) {
		self.reset_handlers.lock().unwrap().push(FormResetHandler {
			name: name.to_string(),
			handler,
		});
	}

	/// Remove the reset handlers registered under a name.
	pub fn unregister(&self, name: &str) {
		self.reset_handlers
			.lock()
			.unwrap()
			.retain(|handler| handler.name != name);
	}

	/// Replay every registered reset handler.
	pub fn reset(&self) {
		let handlers = self.reset_handlers.lock().unwrap().clone_shallow();

		for handler in &handlers {
			(handler.handler)();
		}
	}
}

/// An HTML form element which resets signal-driven control state alongside
/// the native control reset.
#[component]
pub fn Form(
	/// The id of the form element.
	#[prop(optional, into)]
	id: Option<String>,
	/// Classes for the root element.
	#[prop(optional, into)]
	class: Signal<String>,
	/// The content of the form.
	children: Children,
) -> impl IntoView {
	let context = FormContext::default();
	let reset_context = context.clone();

	provide_context(context);

	view! {
		<form
			id=move || id.clone().unwrap_or_default()
			class=merge_class("", class)
			on:reset=move |_| reset_context.reset()
		>
			{children()}
		</form>
	}
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
		let context = expect_context::<FormContext>();
		let value = RwSignal::new("initial".to_string());

		use_form_reset(&context, "example", move || {
			value.set("initial".to_string());
		});

		view! {
			<input
				name="example"
				prop:value=move || value.get()
				on:input=move |event: web_sys::Event| {
					let target = event.target().unwrap();
					let input = JsCast::dyn_ref::<web_sys::HtmlInputElement>(&target)
						.unwrap();
					value.set(input.value());
				}
			/>
		}
	}

	#[wasm_bindgen_test]
	async fn reset_replays_registered_handlers() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! {
				<Form>
					<Example />
				</Form>
			}
		});

		let input = document()
			.query_selector("input[name='example']")
			.unwrap()
			.unwrap()
			.unchecked_into::<web_sys::HtmlInputElement>();

		let form = document()
			.query_selector("form")
			.unwrap()
			.unwrap()
			.unchecked_into::<web_sys::HtmlFormElement>();

		flush_microtasks().await;

		assert_eq!(input.value(), "initial");

		input.set_value("changed");
		input
			.dispatch_event(&web_sys::Event::new("input").unwrap())
			.unwrap();
		flush_microtasks().await;

		assert_eq!(input.value(), "changed");

		form.reset();

		// Let the reactive update bound to the input flush.
		flush_microtasks().await;

		assert_eq!(
			input.value(),
			"initial",
			"the registered handler restored the signal state"
		);
	}
}
