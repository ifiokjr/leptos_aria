#![doc(html_logo_url = "https://ifiokjr.github.io/leptos_aria/logo.svg")]
#![doc(html_favicon_url = "https://ifiokjr.github.io/leptos_aria/favicon.ico")]
#![allow(clippy::needless_pass_by_value)]

use leptos::prelude::*;

/// Breadcrumbs show the hierarchy of the current page.
#[component]
pub fn Breadcrumbs(children: Children) -> impl IntoView {
	view! {
		<nav aria-label="Breadcrumbs">
			<ol class="leptos-aria-breadcrumbs">{children()}</ol>
		</nav>
	}
}

/// One crumb of the breadcrumb trail.
#[component]
pub fn BreadcrumbItem(
	/// The URL the crumb links to; the last crumb usually has none.
	#[prop(optional, into)]
	href: Option<String>,
	/// Whether this crumb is the current page.
	#[prop(optional, into)]
	is_current: Signal<bool>,
	children: Children,
) -> impl IntoView {
	let as_link = href.is_some();

	let crumb: AnyView = if as_link {
		let href = href.unwrap_or_default();

		view! {
			<a href=href aria-current=move || if is_current.get() { "page" } else { "" }>
				{children()}
			</a>
		}
		.into_any()
	} else {
		view! {
			<span aria-current=move || if is_current.get() { "page" } else { "" }>
				{children()}
			</span>
		}
		.into_any()
	};

	view! {
		<li>{crumb}</li>
	}
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
	use leptos::mount::mount_to_body;
	use wasm_bindgen_test::*;

	use super::*;

	wasm_bindgen_test_configure!(run_in_browser);

	#[wasm_bindgen_test]
	async fn breadcrumbs_render_a_nav_list() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! {
				<Breadcrumbs>
					<BreadcrumbItem href="https://example.com".to_string()>
						"Home"
					</BreadcrumbItem>
					<BreadcrumbItem is_current=Signal::from(true)>"Library"</BreadcrumbItem>
				</Breadcrumbs>
			}
		});

		let nav = document()
			.query_selector("nav[aria-label='Breadcrumbs']")
			.unwrap();

		assert!(nav.is_some(), "the breadcrumb trail is a labeled nav");

		let current = document().query_selector("[aria-current='page']").unwrap();

		assert!(current.is_some(), "the current crumb is marked");
	}
}
