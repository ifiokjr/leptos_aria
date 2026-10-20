#![doc(html_logo_url = "https://ifiokjr.github.io/leptos_aria/logo.svg")]
#![doc(html_favicon_url = "https://ifiokjr.github.io/leptos_aria/favicon.ico")]
#![allow(clippy::needless_pass_by_value)]

use leptos::prelude::*;
use leptos_aria_utils::merge_class;

/// A table presents tabular data in semantic table elements.
#[component]
pub fn Table(
	/// The content to display as the caption.
	#[prop(optional)]
	caption: Option<String>,
	/// Classes added after the built-in `leptos-aria-table` class.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	view! {
		<table class=merge_class("leptos-aria-table", class)>
			{caption.map(|caption| view! { <caption>{caption}</caption> })}
			{children()}
		</table>
	}
}

/// The header section of a table.
#[component]
pub fn TableHeader(
	/// Classes for the root element.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	view! { <thead class=merge_class("", class)>{children()}</thead> }
}

/// The body section of a table.
#[component]
pub fn TableBody(
	/// Classes for the root element.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	view! { <tbody class=merge_class("", class)>{children()}</tbody> }
}

/// A row of a table.
#[component]
pub fn TableRow(
	/// Classes for the root element.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	view! { <tr class=merge_class("", class)>{children()}</tr> }
}

/// A header cell of a table.
#[component]
pub fn TableColumnHeader(
	/// Classes for the root element.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	view! { <th class=merge_class("", class) scope="col">{children()}</th> }
}

/// A data cell of a table.
#[component]
pub fn TableCell(
	/// Classes for the root element.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	view! { <td class=merge_class("", class)>{children()}</td> }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
	use leptos::mount::mount_to_body;
	use wasm_bindgen_test::*;

	use super::*;

	wasm_bindgen_test_configure!(run_in_browser);

	#[wasm_bindgen_test]
	async fn table_renders_semantic_elements() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! {
				<Table caption="Prices".to_string()>
					<TableHeader>
						<TableRow>
							<TableColumnHeader>"Item"</TableColumnHeader>
						</TableRow>
					</TableHeader>
					<TableBody>
						<TableRow>
							<TableCell>"Apple"</TableCell>
						</TableRow>
					</TableBody>
				</Table>
			}
		});

		let caption = document().query_selector("caption").unwrap();
		let header_cell = document().query_selector("th[scope='col']").unwrap();
		let body_cell = document().query_selector("td").unwrap();

		assert!(caption.is_some());
		assert!(header_cell.is_some());
		assert!(body_cell.is_some());
	}

	#[wasm_bindgen_test]
	async fn table_appends_the_caller_class_after_the_built_in_class() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! {
				<Table class="table-test-root w-full">
					<TableBody>
						<TableRow>
							<TableCell class="table-test-cell">"Apple"</TableCell>
						</TableRow>
					</TableBody>
				</Table>
			}
		});

		let table = document()
			.query_selector(".table-test-root")
			.unwrap()
			.unwrap();
		let cell = document()
			.query_selector(".table-test-cell")
			.unwrap()
			.unwrap();

		assert_eq!(
			table.get_attribute("class").as_deref(),
			Some("leptos-aria-table table-test-root w-full")
		);
		assert_eq!(
			cell.get_attribute("class").as_deref(),
			Some("table-test-cell"),
			"a part without a built-in class carries only the caller class"
		);
	}
}
