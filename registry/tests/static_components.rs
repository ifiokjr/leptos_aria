//! Browser tests for the registry components that need no headless crate.

#![cfg(target_arch = "wasm32")]

use leptos::mount::mount_to_body;
use leptos::prelude::*;
use leptos_aria_registry::ui::badge::Badge;
use leptos_aria_registry::ui::badge::BadgeVariant;
use leptos_aria_registry::ui::card::Card;
use leptos_aria_registry::ui::card::CardTitle;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

fn slot(name: &str) -> web_sys::Element {
	document()
		.query_selector(&format!("[data-slot=\"{name}\"]"))
		.unwrap()
		.unwrap_or_else(|| panic!("no element with data-slot={name}"))
}

#[wasm_bindgen_test]
fn caller_classes_win_conflicts() {
	console_error_panic_hook::set_once();

	mount_to_body(|| {
		view! {
			<Badge variant=BadgeVariant::Outline class="px-4">"New"</Badge>
			<Card class="rounded-none">
				<CardTitle>"Title"</CardTitle>
			</Card>
		}
	});

	let badge = slot("badge").get_attribute("class").unwrap();
	assert!(badge.contains("px-4"), "{badge}");
	assert!(
		!badge.contains("px-2"),
		"the default padding is merged away: {badge}"
	);
	assert!(
		badge.contains("text-foreground"),
		"the variant applies: {badge}"
	);

	let card = slot("card").get_attribute("class").unwrap();
	assert!(
		card.contains("rounded-none") && !card.contains("rounded-xl"),
		"{card}"
	);
	assert_eq!(slot("card-title").text_content().as_deref(), Some("Title"));
}
