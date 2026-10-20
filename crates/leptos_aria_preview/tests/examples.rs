//! Every registry component's usage example renders in a browser.

#![cfg(target_arch = "wasm32")]

use leptos::mount::mount_to;
use leptos::prelude::*;
use leptos_aria_preview::EXAMPLES;
use leptos_aria_preview::example;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn every_example_mounts_content() {
	console_error_panic_hook::set_once();

	for (name, render) in EXAMPLES {
		let root = document().create_element("div").unwrap();
		document().body().unwrap().append_child(&root).unwrap();

		mount_to(root.clone().unchecked_into(), *render).forget();

		assert!(
			root.child_element_count() > 0,
			"the `{name}` example rendered nothing"
		);
	}
}

#[wasm_bindgen_test]
fn looks_up_examples_by_registry_name() {
	assert_eq!(EXAMPLES.len(), 29);
	assert!(example("alert-dialog").is_some());
	assert!(example("toggle-group").is_some());
	assert!(example("missing").is_none());
}
