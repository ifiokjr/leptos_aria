#![doc(html_logo_url = "https://ifiokjr.github.io/leptos_aria/logo.svg")]
#![doc(html_favicon_url = "https://ifiokjr.github.io/leptos_aria/favicon.ico")]
#![allow(clippy::needless_pass_by_value)]

use leptos::prelude::*;
use leptos_aria_selection::SelectionMode;
use leptos_aria_selection::UseSelectionStateProps;
use leptos_aria_selection::use_selection_state;
use leptos_aria_utils::use_id;

/// The state a tag group shares with its tags.
#[derive(Clone)]
pub struct TagGroupContext {
	pub state: leptos_aria_selection::SelectionState,
}

/// A tag group presents a set of tags the user can select and remove.
#[component]
pub fn TagGroup(
	/// Whether more than one tag can be selected.
	#[prop(optional, into)]
	is_multiple: Signal<bool>,
	/// Whether the whole group is disabled.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	/// The content to display as the label.
	#[prop(optional)]
	label: Option<String>,
	/// The current selection (controlled).
	#[prop(optional, into)]
	selection: Option<Signal<leptos_aria_selection::Selection>>,
	children: Children,
) -> impl IntoView {
	let state = use_selection_state(UseSelectionStateProps {
		selection,
		mode: Signal::derive(move || {
			if is_multiple.get() {
				SelectionMode::Multiple
			} else {
				SelectionMode::Single
			}
		})
		.get_untracked(),
		is_disabled: is_disabled.get_untracked(),
		..Default::default()
	});

	provide_context(TagGroupContext {
		state: state.clone(),
	});

	let label_id = use_id();

	view! {
		{label.map(|label| view! { <span id=label_id.clone()>{label}</span> })}
		<div role="group" aria-labelledby=label_id.clone()>
			{children()}
		</div>
	}
}

/// An individual tag inside a tag group.
#[component]
pub fn Tag(
	/// The value of the tag, reported in the selection.
	#[prop(into)]
	value: String,
	children: Children,
) -> impl IntoView {
	let context = expect_context::<TagGroupContext>();
	let tag_id = use_id();
	let is_selected = Signal::derive({
		let selection = context.state.selection_signal();
		let value = value.clone();

		move || selection.with(|selection| selection.contains(&value))
	});
	let tag_value = value.clone();

	view! {
		<div
			id=tag_id.clone()
			role="row"
			aria-selected=move || is_selected.get().to_string()
			data-selected=move || is_selected.get().to_string()
			on:click={
				let context = context.clone();
				let value = tag_value.clone();

				move |_| context.state.select_key(&value, false)
			}
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

	#[wasm_bindgen_test]
	async fn tags_select_on_click() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! {
				<TagGroup label="Filters".to_string()>
					<Tag value="science".to_string()>"Science"</Tag>
					<Tag value="art".to_string()>"Art"</Tag>
				</TagGroup>
			}
		});

		let first = document()
			.query_selector("[role='group'] [data-selected]")
			.unwrap()
			.unwrap()
			.unchecked_into::<web_sys::Element>();

		first
			.dispatch_event(&web_sys::Event::new("click").unwrap())
			.unwrap();

		let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
		wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();

		assert_eq!(
			first.get_attribute("data-selected").as_deref(),
			Some("true")
		);
	}
}
