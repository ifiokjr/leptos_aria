#![doc(html_logo_url = "https://ifiokjr.github.io/leptos_aria/logo.svg")]
#![doc(html_favicon_url = "https://ifiokjr.github.io/leptos_aria/favicon.ico")]
#![allow(clippy::needless_pass_by_value)]

use leptos::prelude::*;
use leptos_aria_utils::merge_class;
use wasm_bindgen::JsCast;

/// A callback invoked with the selected files.
pub type BoxedFileSelectCallback = Box<dyn Fn(Vec<web_sys::File>) + 'static>;

/// A file trigger opens the platform file picker.
#[component]
pub fn FileTrigger(
	/// Handler that is called when files are selected.
	#[prop(optional)]
	on_select: Option<BoxedFileSelectCallback>,
	/// Whether multiple files can be selected.
	#[prop(optional, into)]
	allows_multiple: Signal<bool>,
	/// The accepted file types as extensions or MIME types.
	#[prop(optional, into)]
	accepted_file_types: Option<String>,
	/// Whether the trigger is disabled.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	/// Classes added after the built-in `leptos-aria-file-trigger` class.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let input_id = leptos_aria_utils::use_id();

	view! {
		<label for=input_id.clone() class=merge_class("leptos-aria-file-trigger", class)>
			{children()}
		</label>
		<input
			id=input_id.clone()
			type="file"
			class="leptos-aria-file-trigger__input"
			style="display: none;"
			multiple=move || allows_multiple.get()
			accept=move || accepted_file_types.clone().unwrap_or_default()
			disabled=move || is_disabled.get()
			on:change=move |event: web_sys::Event| {
				let Some(input) = event
					.target()
					.and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
				else {
					return;
				};

				if let Some(on_select) = &on_select {
					let files = input.files();

					if let Some(files) = files {
						let selected: Vec<web_sys::File> = (0..files.length())
							.filter_map(|index| files.item(index))
							.collect();

						on_select(selected);
					}
				}

				input.set_value("");
			}
		/>
	}
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
	use leptos::mount::mount_to_body;
	use wasm_bindgen_test::*;

	use super::*;

	wasm_bindgen_test_configure!(run_in_browser);

	#[wasm_bindgen_test]
	async fn file_trigger_renders_a_hidden_input() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! {
				<FileTrigger allows_multiple=Signal::from(true)>
					"Upload files"
				</FileTrigger>
			}
		});

		let input = document()
			.query_selector("input[type='file']")
			.unwrap()
			.unwrap()
			.unchecked_into::<web_sys::HtmlInputElement>();

		assert!(input.multiple(), "the multiple attribute is set");
	}
}
