//! Browser tests for the registry components that wrap headless crates.
//!
//! Every test mounts into its own container and queries inside it, so tests
//! never see each other's elements, and unmounts when it finishes.

#![cfg(target_arch = "wasm32")]

use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use leptos::mount::mount_to;
use leptos::prelude::*;
use leptos_aria::SelectBoxItem;
use leptos_aria_registry::ui::accordion::*;
use leptos_aria_registry::ui::alert_dialog::AlertDialog;
use leptos_aria_registry::ui::dialog::*;
use leptos_aria_registry::ui::dropdown_menu::*;
use leptos_aria_registry::ui::pagination::Pagination;
use leptos_aria_registry::ui::popover::*;
use leptos_aria_registry::ui::select::Select;
use leptos_aria_registry::ui::tabs::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// A fresh element in the body for one test to mount into.
fn container() -> web_sys::HtmlElement {
	let container = document()
		.create_element("div")
		.unwrap()
		.unchecked_into::<web_sys::HtmlElement>();

	document().body().unwrap().append_child(&container).unwrap();

	container
}

fn find(scope: &web_sys::Element, selector: &str) -> web_sys::HtmlElement {
	scope
		.query_selector(selector)
		.unwrap()
		.unwrap_or_else(|| panic!("nothing matches {selector}"))
		.unchecked_into()
}

fn class_of(element: &web_sys::Element) -> String {
	element.get_attribute("class").unwrap_or_default()
}

/// Dispatch a bubbling `pointerdown`, as a real press does before its click.
fn press(element: &web_sys::Element) {
	let event = document().create_event("Event").unwrap();

	event.init_event_with_bubbles("pointerdown", true);
	element.dispatch_event(&event).unwrap();
}

/// Let effects scheduled by the last interaction run.
async fn tick() {
	let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
	wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
}

#[wasm_bindgen_test]
async fn accordion_trigger_opens_its_content() {
	console_error_panic_hook::set_once();

	let root = container();
	let handle = mount_to(root.clone(), || {
		view! {
			<Accordion>
				<AccordionItem key="shipping">
					<AccordionTrigger>"Shipping"</AccordionTrigger>
					<AccordionContent>"Two working days."</AccordionContent>
				</AccordionItem>
			</Accordion>
		}
	});
	let trigger = find(&root, "[data-slot=accordion-trigger]");
	let content = find(&root, "[data-slot=accordion-content]");

	assert!(content.has_attribute("hidden"), "the section starts closed");
	assert_eq!(
		trigger.get_attribute("aria-expanded").as_deref(),
		Some("false")
	);

	trigger.click();
	tick().await;

	assert!(!content.has_attribute("hidden"), "the trigger opens it");
	assert_eq!(
		trigger.get_attribute("aria-expanded").as_deref(),
		Some("true")
	);
	assert_eq!(content.get_attribute("data-state").as_deref(), Some("open"));
	assert!(
		root.query_selector("h3[data-slot=accordion-header] > [data-slot=accordion-trigger]")
			.unwrap()
			.is_some(),
		"the trigger sits in a heading"
	);

	drop(handle);
	root.remove();
}

#[wasm_bindgen_test]
async fn tab_trigger_switches_the_visible_panel() {
	console_error_panic_hook::set_once();

	let root = container();
	let handle = mount_to(root.clone(), || {
		view! {
			<Tabs default_selected_key="overlays-account">
				<TabsList>
					<TabsTrigger key="overlays-account">"Account"</TabsTrigger>
					<TabsTrigger key="overlays-password">"Password"</TabsTrigger>
				</TabsList>
				<TabsContent key="overlays-account">"Account panel"</TabsContent>
				<TabsContent key="overlays-password">"Password panel"</TabsContent>
			</Tabs>
		}
	});
	let list = find(&root, "[data-slot=tabs-list]");
	let account = find(&root, "#leptos-aria-tab-overlays-account");
	let password = find(&root, "#leptos-aria-tab-overlays-password");
	let account_panel = find(&root, "#leptos-aria-tab-panel-overlays-account");
	let password_panel = find(&root, "#leptos-aria-tab-panel-overlays-password");

	assert_eq!(list.get_attribute("role").as_deref(), Some("tablist"));
	assert_eq!(
		account.get_attribute("aria-selected").as_deref(),
		Some("true")
	);
	assert!(!account_panel.has_attribute("hidden"));
	assert!(password_panel.has_attribute("hidden"));

	password.click();
	tick().await;

	assert_eq!(
		password.get_attribute("aria-selected").as_deref(),
		Some("true")
	);
	assert_eq!(
		account.get_attribute("aria-selected").as_deref(),
		Some("false")
	);
	assert!(account_panel.has_attribute("hidden"), "the old panel hides");
	assert!(
		!password_panel.has_attribute("hidden"),
		"the new panel shows"
	);
	assert!(
		root.query_selector("[data-slot=tabs] > [data-slot=tabs-content]")
			.unwrap()
			.is_some(),
		"panels are siblings of the list, not inside it"
	);

	drop(handle);
	root.remove();
}

#[wasm_bindgen_test]
async fn dialog_opens_as_a_labelled_dialog() {
	console_error_panic_hook::set_once();

	let is_open = RwSignal::new(false);
	let root = container();
	let handle = mount_to(root.clone(), move || {
		view! {
			<Dialog is_open=is_open>
				<DialogTrigger>"Edit profile"</DialogTrigger>
				<DialogContent>
					<DialogHeader>
						<DialogTitle>"Edit profile"</DialogTitle>
						<DialogDescription>"Make changes here."</DialogDescription>
					</DialogHeader>
					<DialogFooter>
						<DialogClose class="overlays-dialog-cancel">"Cancel"</DialogClose>
					</DialogFooter>
				</DialogContent>
			</Dialog>
		}
	});
	let dialogs = root.query_selector_all("[role=dialog]").unwrap();

	assert_eq!(dialogs.length(), 1, "only the panel is a dialog");

	let dialog = find(&root, "[role=dialog]");
	let layer = dialog.parent_element().unwrap();

	assert!(layer.has_attribute("hidden"), "the dialog starts closed");

	find(&root, "[data-slot=dialog-trigger]").click();
	tick().await;

	assert!(is_open.get_untracked());
	assert!(!layer.has_attribute("hidden"), "the trigger opens it");
	assert_eq!(dialog.get_attribute("aria-modal").as_deref(), Some("true"));

	let title = find(&root, "[data-slot=dialog-title]");
	let description = find(&root, "[data-slot=dialog-description]");

	assert_eq!(title.text_content().as_deref(), Some("Edit profile"));
	assert_eq!(dialog.get_attribute("aria-labelledby"), Some(title.id()));
	assert_eq!(
		dialog.get_attribute("aria-describedby"),
		Some(description.id())
	);
	assert!(
		root.query_selector("[data-slot=dialog-content] > button[data-slot=dialog-close]")
			.unwrap()
			.is_some(),
		"the close button renders by default"
	);

	find(&root, ".overlays-dialog-cancel").click();
	tick().await;

	assert!(!is_open.get_untracked(), "the close button closes it");
	assert!(layer.has_attribute("hidden"));

	drop(handle);
	root.remove();
}

#[wasm_bindgen_test]
async fn pagination_next_and_previous_change_the_page() {
	console_error_panic_hook::set_once();

	let page = RwSignal::new(2_usize);
	let root = container();
	let handle = mount_to(root.clone(), move || {
		view! { <Pagination page=page count=3_usize on_change=Box::new(move |next| page.set(next)) /> }
	});
	let nav = find(&root, "[data-slot=pagination]");

	find(&root, "[aria-label='Next page']").click();
	tick().await;

	assert_eq!(page.get_untracked(), 3, "next moves forward");
	assert_eq!(nav.get_attribute("data-page").as_deref(), Some("3"));

	find(&root, "[aria-label='Previous page']").click();
	tick().await;

	assert_eq!(page.get_untracked(), 2, "previous moves back");
	assert_eq!(nav.get_attribute("data-page").as_deref(), Some("2"));

	drop(handle);
	root.remove();
}

#[wasm_bindgen_test]
fn caller_classes_win_over_defaults() {
	console_error_panic_hook::set_once();

	let root = container();
	let handle = mount_to(root.clone(), || {
		view! {
			<Pagination
				page=1_usize
				count=2_usize
				on_change=Box::new(|_| {})
				class="gap-4"
			/>
			<AlertDialog
				is_open=RwSignal::new(false)
				title="Delete?"
				description="This cannot be undone."
				on_confirm=Arc::new(|| {})
			/>
		}
	});
	let pagination = class_of(&find(&root, "[data-slot=pagination]"));

	assert!(pagination.contains("gap-4"), "{pagination}");
	assert!(
		!pagination.contains("gap-1"),
		"the default gap is merged away: {pagination}"
	);
	assert!(
		pagination.starts_with("leptos-aria-pagination "),
		"the headless class stays: {pagination}"
	);

	// The alert dialog is styled entirely through arbitrary variants, which
	// must survive the merge intact.
	let alert = class_of(&find(&root, "[data-slot=alert-dialog]"));

	for class in [
		"[&>h2]:text-lg",
		"sm:[&>div]:flex-row",
		"[&_[data-action=confirm]]:bg-primary",
		"[&_[data-action=cancel]]:hover:bg-accent",
		// The focus ring's width and colour are separate utilities; both stay.
		"[&_button]:focus-visible:ring-3",
		"[&_button]:focus-visible:ring-ring/50",
	] {
		assert!(alert.split(' ').any(|c| c == class), "{class} in {alert}");
	}

	drop(handle);
	root.remove();
}

#[wasm_bindgen_test]
async fn popover_trigger_toggles_and_an_outside_press_closes() {
	console_error_panic_hook::set_once();

	let is_open = RwSignal::new(false);
	let root = container();
	let handle = mount_to(root.clone(), move || {
		view! {
			<Popover is_open=is_open>
				<PopoverTrigger>"Dimensions"</PopoverTrigger>
				<PopoverContent>"Set the dimensions."</PopoverContent>
			</Popover>
		}
	});
	let trigger = find(&root, "[data-slot=popover-trigger]");

	press(&trigger);
	trigger.click();
	tick().await;
	assert!(is_open.get_untracked(), "the trigger opens it");

	// The press dismisses the open popover; the click must not reopen it.
	press(&trigger);
	trigger.click();
	tick().await;
	assert!(!is_open.get_untracked(), "the trigger closes it again");

	press(&trigger);
	trigger.click();
	tick().await;
	press(&document().body().unwrap());
	tick().await;
	assert!(!is_open.get_untracked(), "a press outside closes it");

	drop(handle);
	root.remove();
}

#[wasm_bindgen_test]
async fn select_shows_the_chosen_option_and_closes() {
	console_error_panic_hook::set_once();

	let fruit = RwSignal::new(None::<String>);
	let root = container();
	let handle = mount_to(root.clone(), move || {
		view! {
			<Select
				label="Fruit"
				placeholder="Pick one"
				selected_key=fruit
				options=vec![
					SelectBoxItem { value: "apple".into(), label: "Apple".into() },
					SelectBoxItem { value: "banana".into(), label: "Banana".into() },
				]
			/>
		}
	});
	let trigger = find(&root, "[data-slot=select] [role=combobox]");
	let content = find(&root, ".leptos-aria-select-box__popup");

	assert_eq!(trigger.text_content().as_deref(), Some("Pick one"));
	assert!(trigger.has_attribute("data-placeholder"));
	assert!(content.has_attribute("hidden"));

	trigger.click();
	tick().await;
	assert!(
		!content.has_attribute("hidden"),
		"the trigger opens the list"
	);

	let options = root.query_selector_all("[role=option]").unwrap();
	options
		.get(1)
		.unwrap()
		.unchecked_into::<web_sys::HtmlElement>()
		.click();
	tick().await;

	assert_eq!(fruit.get_untracked().as_deref(), Some("banana"));
	assert_eq!(trigger.text_content().as_deref(), Some("Banana"));
	assert!(!trigger.has_attribute("data-placeholder"));
	assert!(content.has_attribute("hidden"), "choosing closes the list");

	drop(handle);
	root.remove();
}

#[wasm_bindgen_test]
async fn dropdown_menu_item_runs_its_action_and_closes() {
	console_error_panic_hook::set_once();

	let chosen = Arc::new(AtomicUsize::new(0));
	let root = container();
	let handle = mount_to(root.clone(), {
		let chosen = Arc::clone(&chosen);

		move || {
			view! {
				<DropdownMenu>
					<DropdownMenuTrigger>"Open"</DropdownMenuTrigger>
					<DropdownMenuContent label="Account">
						<DropdownMenuItem
							value="profile"
							on_select=move || {
								chosen.fetch_add(1, Ordering::Relaxed);
							}
						>
							"Profile"
						</DropdownMenuItem>
					</DropdownMenuContent>
				</DropdownMenu>
			}
		}
	});
	let trigger = find(&root, "[data-slot=dropdown-menu-trigger]");
	let content = find(&root, "[data-slot=dropdown-menu-content]");

	assert!(content.has_attribute("hidden"));

	trigger.click();
	tick().await;
	assert!(
		!content.has_attribute("hidden"),
		"the trigger opens the menu"
	);

	find(&root, "[data-slot=dropdown-menu-item]").click();
	tick().await;

	assert_eq!(chosen.load(Ordering::Relaxed), 1, "the item's action runs");
	assert!(content.has_attribute("hidden"), "choosing closes the menu");

	drop(handle);
	root.remove();
}
