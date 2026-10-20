#![doc(html_logo_url = "https://ifiokjr.github.io/leptos_aria/logo.svg")]
#![doc(html_favicon_url = "https://ifiokjr.github.io/leptos_aria/favicon.ico")]
#![allow(clippy::needless_pass_by_value)]

use std::sync::Arc;

use leptos::prelude::*;
use leptos_aria_utils::merge_class;

/// A callback invoked when the page changes.
pub type BoxedPageChangeCallback = Box<dyn Fn(usize) + Send + Sync + 'static>;

/// Pagination navigates a paginated list by page number.
///
/// The nav exposes previous, next, and numbered page buttons with the
/// current page marked `aria-current="page"`.
#[component]
pub fn Pagination(
	/// The one-based current page (controlled).
	#[prop(into)]
	page: Signal<usize>,
	/// The total number of pages.
	#[prop(into)]
	count: Signal<usize>,
	/// Handler that is called when the page changes.
	#[prop(optional)]
	on_change: Option<BoxedPageChangeCallback>,
	/// The content to display as the label.
	#[prop(optional)]
	label: Option<String>,
	/// Classes added after the built-in `leptos-aria-pagination` class.
	#[prop(optional, into)]
	class: Signal<String>,
) -> impl IntoView {
	let state = Arc::new(PaginationState {
		page,
		count,
		on_change: Arc::new(on_change),
	});
	let prev_disabled = Arc::clone(&state);
	let prev_click = Arc::clone(&state);
	let next_disabled = Arc::clone(&state);
	let next_click = Arc::clone(&state);
	let pages = Arc::clone(&state);
	let nav_state = Arc::clone(&state);
	let pages_render = pages;
	let label_id = leptos_aria_utils::use_id();

	view! {
		<nav
			class=merge_class("leptos-aria-pagination", class)
			aria-labelledby=label_id.clone()
			data-page=move || nav_state.page.get().to_string()
		>
			<span id=label_id.clone() style="display: none;">{label.unwrap_or_else(|| "Pagination".to_string())}</span>
			<button
				type="button"
				class="leptos-aria-pagination__previous"
				aria-label="Previous page"
				disabled=move || { prev_disabled.page.get() <= 1 }
				on:click=move |_| prev_click.go_to(prev_click.page.get_untracked().saturating_sub(1))
			>
				"‹"
			</button>
			{move || render_pages(&pages_render)}
			<button
				type="button"
				class="leptos-aria-pagination__next"
				aria-label="Next page"
				disabled=move || { next_disabled.page.get() >= next_disabled.count.get() }
				on:click=move |_| next_click.go_to(next_click.page.get_untracked() + 1)
			>
				"›"
			</button>
		</nav>
	}
}

/// The shared state of one pagination nav.
struct PaginationState {
	page: Signal<usize>,
	count: Signal<usize>,
	on_change: Arc<Option<BoxedPageChangeCallback>>,
}

impl PaginationState {
	/// Move to a page when it is in range.
	fn go_to(&self, page: usize) {
		if page >= 1 && page <= self.count.get_untracked() {
			if let Some(on_change) = self.on_change.as_ref() {
				(*on_change)(page);
			}
		}
	}
}

/// The numbered page buttons.
fn render_pages(state: &Arc<PaginationState>) -> Vec<AnyView> {
	let current = state.page.get_untracked();
	let count = state.count.get_untracked();
	let click_state = Arc::clone(state);

	(1..=count)
		.map(|number| {
			let click_state = Arc::clone(&click_state);
			let is_current = number == current;

			view! {
				<button
					type="button"
					class="leptos-aria-pagination__page"
					aria-current=is_current.then_some("page")
					aria-label=format!("Page {number}")
					data-current=is_current.then_some("true")
					on:click=move |_| click_state.go_to(number)
				>
					{number.to_string()}
				</button>
			}
			.into_any()
		})
		.collect()
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
	use leptos::mount::mount_to_body;
	use wasm_bindgen::JsCast;
	use wasm_bindgen_test::*;

	use super::*;

	wasm_bindgen_test_configure!(run_in_browser);

	fn button(selector: &str) -> web_sys::HtmlButtonElement {
		document()
			.query_selector(selector)
			.unwrap()
			.unwrap()
			.unchecked_into::<web_sys::HtmlButtonElement>()
	}

	async fn tick() {
		let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
		wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
	}

	#[wasm_bindgen_test]
	async fn previous_and_next_move_one_page() {
		console_error_panic_hook::set_once();

		let page = RwSignal::new(2_usize);

		mount_to_body(move || {
			view! {
				<Pagination
					page=page
					count=3_usize
					on_change=Box::new(move |next| page.set(next))
					class="custom-pagination"
				/>
			}
		});

		let nav = document()
			.query_selector(".leptos-aria-pagination")
			.unwrap()
			.unwrap();
		assert_eq!(
			nav.get_attribute("class").as_deref(),
			Some("leptos-aria-pagination custom-pagination")
		);

		button(".leptos-aria-pagination__next").click();
		tick().await;
		assert_eq!(page.get_untracked(), 3, "next moves forward");

		button(".leptos-aria-pagination__previous").click();
		tick().await;
		assert_eq!(page.get_untracked(), 2, "previous moves back");
	}
}
