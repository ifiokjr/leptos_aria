#![allow(clippy::needless_pass_by_value)]

use leptos::prelude::*;

/// A table presents tabular data in semantic table elements.
#[component]
pub fn Table(
	/// The content to display as the caption.
	#[prop(optional)]
	caption: Option<String>,
	children: Children,
) -> impl IntoView {
	view! {
		<table class="leptos-aria-table">
			{caption.map(|caption| view! { <caption>{caption}</caption> })}
			{children()}
		</table>
	}
}

/// The header section of a table.
#[component]
pub fn TableHeader(children: Children) -> impl IntoView {
	view! { <thead>{children()}</thead> }
}

/// The body section of a table.
#[component]
pub fn TableBody(children: Children) -> impl IntoView {
	view! { <tbody>{children()}</tbody> }
}

/// A row of a table.
#[component]
pub fn TableRow(children: Children) -> impl IntoView {
	view! { <tr>{children()}</tr> }
}

/// A header cell of a table.
#[component]
pub fn TableColumnHeader(children: Children) -> impl IntoView {
	view! { <th scope="col">{children()}</th> }
}

/// A data cell of a table.
#[component]
pub fn TableCell(children: Children) -> impl IntoView {
	view! { <td>{children()}</td> }
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
}
