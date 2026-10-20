#![doc(html_logo_url = "https://ifiokjr.github.io/leptos_aria/logo.svg")]
#![doc(html_favicon_url = "https://ifiokjr.github.io/leptos_aria/favicon.ico")]
#![allow(clippy::needless_pass_by_value)]

use std::sync::Arc;

use leptos::prelude::*;
use leptos_aria_utils::merge_class;
use leptos_aria_utils::use_id;
use wasm_bindgen::JsCast;

/// A callback invoked with the dropped files.
pub type BoxedDropCallback = Box<dyn Fn(Vec<web_sys::File>) + Send + Sync + 'static>;

/// A drop zone is an area onto which files can be dragged and dropped.
///
/// The zone highlights while a drag hovers it, and dropping files or
/// activating it through the keyboard both deliver the files to the handler.
#[component]
pub fn DropZone(
	/// Handler that is called when files are dropped or picked.
	on_drop: BoxedDropCallback,
	/// The content to display as the label.
	#[prop(optional)]
	label: Option<String>,
	/// Classes added after the built-in `leptos-aria-drop-zone` class.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let zone_id = use_id();
	let is_dragging = RwSignal::new(false);
	let shared_on_drop = Arc::new(on_drop);

	let enter_state = is_dragging;
	let leave_state = is_dragging;
	let drop_state = is_dragging;
	let drop_handler = Arc::clone(&shared_on_drop);
	let click_handler = Arc::clone(&shared_on_drop);
	let input_id = use_id();
	let input_selector = format!("input#{input_id}");
	let key_selector = input_selector.clone();
	let click_selector = input_selector.clone();

	view! {
		{label.map(|label| view! { <span id=zone_id.clone()>{label}</span> })}
		<div
			class=merge_class("leptos-aria-drop-zone", class)
			role="button"
			tabindex="0"
			aria-labelledby=zone_id.clone()
			data-dragging=move || if is_dragging.get() { "true" } else { "false" }
			on:dragenter=move |_| enter_state.set(true)
			on:dragover=move |event: web_sys::DragEvent| {
				// A drag over must be canceled for the zone to accept the drop.
				event.prevent_default();
			}
			on:dragleave=move |_| leave_state.set(false)
			on:drop=move |event: web_sys::DragEvent| {
				event.prevent_default();
				drop_state.set(false);
				deliver_dragged(&event, &drop_handler);
			}
			on:keydown=move |event: web_sys::KeyboardEvent| {
				if event.key() == "Enter" || event.key() == " " {
					event.prevent_default();
					open_picker(&key_selector);
				}
			}
			on:click=move |_| open_picker(&click_selector)
		>
			{children()}
		</div>
		<input
			id=input_id
			type="file"
			class="leptos-aria-drop-zone__input"
			style="display: none;"
			on:change=move |event: web_sys::Event| {
				if let Some(files) = changed_files(&event) {
					(*click_handler)(files);
				}
			}
		/>
	}
}

/// Hand the files of a drop event to the handler.
fn deliver_dragged(event: &web_sys::DragEvent, on_drop: &Arc<BoxedDropCallback>) {
	if let Some(files) = dragged_files(event) {
		(*on_drop)(files);
	}
}

/// Open the platform file picker behind the hidden input.
fn open_picker(selector: &str) {
	let found = document()
		.query_selector(selector)
		.ok()
		.flatten()
		.and_then(|input| input.dyn_into::<web_sys::HtmlInputElement>().ok());

	if let Some(input) = found {
		input.click();
	}
}

/// The files carried by a drop event.
fn dragged_files(event: &web_sys::DragEvent) -> Option<Vec<web_sys::File>> {
	event
		.data_transfer()
		.and_then(|transfer| transfer.files())
		.map(|files| {
			(0..files.length())
				.filter_map(|index| files.item(index))
				.collect()
		})
}

/// The files chosen through the hidden input.
fn changed_files(event: &web_sys::Event) -> Option<Vec<web_sys::File>> {
	event
		.target()
		.and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
		.and_then(|input| input.files())
		.map(|files| {
			(0..files.length())
				.filter_map(|index| files.item(index))
				.collect()
		})
}

fn document() -> web_sys::Document {
	web_sys::window()
		.expect("the  global is available in the browser")
		.document()
		.expect("the  global is available in the browser")
}

/// One dropped item in a drop zone listing.
#[component]
pub fn DropZoneItem(
	/// The name of the file.
	#[prop(into)]
	name: String,
	/// The size of the file in bytes.
	#[prop(optional, into)]
	size: Option<u64>,
	/// Classes added after the built-in `leptos-aria-drop-zone__item` class.
	#[prop(optional, into)]
	class: Signal<String>,
) -> impl IntoView {
	view! {
		<li class=merge_class("leptos-aria-drop-zone__item", class)>
			<span>{name}</span>
			{size.map(|size| view! { <span class="leptos-aria-drop-zone__size">{size}</span> })}
		</li>
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
	async fn drop_zone_renders_an_accessible_target() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! {
				<DropZone
					label="Drop files".to_string()
					on_drop=Box::new(|_| {})
				>
					"Drag files here"
				</DropZone>
			}
		});

		let zone = document()
			.query_selector(".leptos-aria-drop-zone")
			.unwrap()
			.unwrap()
			.unchecked_into::<web_sys::Element>();

		assert_eq!(zone.get_attribute("tabindex").as_deref(), Some("0"));
		assert_eq!(
			zone.get_attribute("data-dragging").as_deref(),
			Some("false")
		);
	}

	#[wasm_bindgen_test]
	async fn drop_zone_appends_the_caller_class() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! {
				<DropZone class="custom-zone" on_drop=Box::new(|_| {})>
					"Drag files here"
				</DropZone>
			}
		});

		let zone = document()
			.query_selector(".leptos-aria-drop-zone.custom-zone")
			.unwrap()
			.unwrap()
			.unchecked_into::<web_sys::Element>();

		assert_eq!(
			zone.get_attribute("class").as_deref(),
			Some("leptos-aria-drop-zone custom-zone")
		);
	}
}

/// A callback invoked when a drag starts or ends.
pub type BoxedDragStateCallback = Box<dyn Fn() + Send + Sync + 'static>;

/// A draggable item participates in HTML5 drag and drop.
///
/// The port of the upstream `useDraggableItem`: the element becomes
/// draggable, its payload is written to the data transfer on drag start, and
/// an `is-dragging` attribute tracks the state for styling.
#[component]
pub fn DraggableItem(
	/// The data transfer type the payload is written under.
	#[prop(into)]
	data_type: String,
	/// The payload dropped targets read back.
	#[prop(into)]
	value: String,
	/// Handler that is called when the drag starts.
	#[prop(optional)]
	on_drag_start: Option<BoxedDragStateCallback>,
	/// Handler that is called when the drag ends.
	#[prop(optional)]
	on_drag_end: Option<BoxedDragStateCallback>,
	/// Classes added after the built-in `leptos-aria-draggable` class.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let is_dragging = RwSignal::new(false);

	view! {
		<div
			class=merge_class("leptos-aria-draggable", class)
			draggable="true"
			data-is-dragging=move || if is_dragging.get() { "true" } else { "false" }
			on:dragstart=move |event: web_sys::DragEvent| {
				let Some(transfer) = event.data_transfer() else {
					return;
				};

				transfer.set_data(&data_type, &value).ok();
				is_dragging.set(true);

				if let Some(on_drag_start) = &on_drag_start {
					(*on_drag_start)();
				}
			}
			on:dragend=move |_| {
				is_dragging.set(false);

				if let Some(on_drag_end) = &on_drag_end {
					(*on_drag_end)();
				}
			}
		>
			{children()}
		</div>
	}
}

#[cfg(all(test, target_arch = "wasm32"))]
mod draggable_tests {
	use leptos::mount::mount_to_body;
	use wasm_bindgen::JsCast;
	use wasm_bindgen_test::*;

	use super::*;

	wasm_bindgen_test_configure!(run_in_browser);

	#[wasm_bindgen_test]
	async fn draggable_item_renders_draggable_with_state() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! {
				<DraggableItem data_type="text/plain".to_string() value="card".to_string()>
					"Card"
				</DraggableItem>
			}
		});

		let item = document()
			.query_selector(".leptos-aria-draggable")
			.unwrap()
			.unwrap()
			.unchecked_into::<web_sys::Element>();

		assert_eq!(
			item.get_attribute("draggable").as_deref(),
			Some("true"),
			"the item is draggable"
		);
		assert_eq!(
			item.get_attribute("data-is-dragging").as_deref(),
			Some("false"),
			"the drag state starts idle"
		);
	}
}
