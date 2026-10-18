#![allow(clippy::needless_pass_by_value)]

use leptos::prelude::*;
use leptos_aria_selection::SelectionMode;
use leptos_aria_selection::UseSelectionStateProps;
use leptos_aria_selection::use_selection_state;

/// The state a grid shares with its cells through context.
#[derive(Clone)]
pub struct GridContext {
	pub selection: leptos_aria_selection::SelectionState,
}

/// A grid presents tabular data navigated by rows and columns with the
/// arrow keys.
#[component]
pub fn Grid(
	/// The content to display as the label.
	#[prop(optional)]
	label: Option<String>,
	/// Whether more than one row can be selected.
	#[prop(optional, into)]
	is_multiple: Signal<bool>,
	children: Children,
) -> impl IntoView {
	let selection = use_selection_state(UseSelectionStateProps {
		mode: Signal::derive(move || {
			if is_multiple.get() {
				SelectionMode::Multiple
			} else {
				SelectionMode::Single
			}
		})
		.get_untracked(),
		..Default::default()
	});

	provide_context(GridContext {
		selection: selection.clone(),
	});

	let label_id = leptos_aria_utils::use_id();

	view! {
		{label.map(|label| view! { <span id=label_id.clone()>{label}</span> })}
		<div
			role="grid"
			aria-labelledby=label_id.clone()
			on:keydown=move |event: web_sys::KeyboardEvent| {
				// Arrow navigation between cells is native to the browser in
				// grid layout; the grid only claims the roving keys.
				if event.key() == "Home" || event.key() == "End" {
					event.prevent_default();
				}
			}
		>
			{children()}
		</div>
	}
}

/// A row of a grid.
#[component]
pub fn GridRow(
	/// The row's key, reported in the selection.
	#[prop(into)]
	key: String,
	children: Children,
) -> impl IntoView {
	view! {
		<div role="row" data-key=key>{children()}</div>
	}
}

/// A cell of a grid row.
#[component]
pub fn GridCell(
	/// Whether the cell is a column header.
	#[prop(optional, into)]
	is_header: Signal<bool>,
	children: Children,
) -> impl IntoView {
	view! {
		<div
			role=move || if is_header.get() { "columnheader" } else { "gridcell" }
			tabindex="-1"
		>
			{children()}
		</div>
	}
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
	use leptos::mount::mount_to_body;
	use wasm_bindgen_test::*;

	use super::*;

	wasm_bindgen_test_configure!(run_in_browser);

	#[wasm_bindgen_test]
	async fn grid_renders_rows_and_cells() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! {
				<Grid label="Prices".to_string()>
					<GridRow key="row-1".to_string()>
						<GridCell is_header=Signal::from(true)>"Item"</GridCell>
						<GridCell>"Apple"</GridCell>
					</GridRow>
				</Grid>
			}
		});

		let grid = document().query_selector("[role='grid']").unwrap();
		let header = document().query_selector("[role='columnheader']").unwrap();
		let cell = document().query_selector("[role='gridcell']").unwrap();

		assert!(grid.is_some());
		assert!(header.is_some());
		assert!(cell.is_some());
	}
}
