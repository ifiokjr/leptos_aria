//! Browser tests for the registry components that wrap headless controls.

#![cfg(target_arch = "wasm32")]

use leptos::mount::UnmountHandle;
use leptos::mount::mount_to;
use leptos::prelude::*;
use leptos::tachys::view::Mountable;
use leptos_aria::PressEvent;
use leptos_aria_registry::ui::avatar::Avatar;
use leptos_aria_registry::ui::button::Button;
use leptos_aria_registry::ui::button::ButtonSize;
use leptos_aria_registry::ui::button::ButtonVariant;
use leptos_aria_registry::ui::checkbox::Checkbox;
use leptos_aria_registry::ui::input::Input;
use leptos_aria_registry::ui::progress::Progress;
use leptos_aria_registry::ui::radio_group::RadioGroup;
use leptos_aria_registry::ui::radio_group::RadioGroupItem;
use leptos_aria_registry::ui::separator::Separator;
use leptos_aria_registry::ui::separator::SeparatorOrientation;
use leptos_aria_registry::ui::slider::Slider;
use leptos_aria_registry::ui::switch::Switch;
use leptos_aria_registry::ui::toggle::Toggle;
use leptos_aria_registry::ui::toggle::ToggleVariant;
use leptos_aria_registry::ui::toggle_group::ToggleGroup;
use leptos_aria_registry::ui::toggle_group::ToggleGroupItem;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// A view mounted into its own container, removed again when dropped so
/// tests never see each other's elements.
struct Fixture<M: Mountable> {
	root: web_sys::HtmlElement,
	_handle: UnmountHandle<M>,
}

impl<M: Mountable> Fixture<M> {
	fn find(&self, selector: &str) -> web_sys::HtmlElement {
		self.root
			.query_selector(selector)
			.unwrap()
			.unwrap_or_else(|| panic!("no element matches {selector}"))
			.unchecked_into()
	}

	fn input(&self, selector: &str) -> web_sys::HtmlInputElement {
		self.find(selector).unchecked_into()
	}
}

impl<M: Mountable> Drop for Fixture<M> {
	fn drop(&mut self) {
		self.root.remove();
	}
}

fn mount<N: IntoView + 'static>(view: impl FnOnce() -> N + 'static) -> Fixture<N::State> {
	console_error_panic_hook::set_once();

	let root: web_sys::HtmlElement = document().create_element("div").unwrap().unchecked_into();
	document().body().unwrap().append_child(&root).unwrap();
	let handle = mount_to(root.clone(), view);

	Fixture {
		root,
		_handle: handle,
	}
}

/// Wait for queued reactive updates to apply.
async fn flush_microtasks() {
	let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);

	wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
}

/// Whether the element has every class in `classes` and none of `absent`.
fn has_classes(element: &web_sys::Element, classes: &[&str], absent: &[&str]) -> bool {
	let list = element.class_list();

	classes.iter().all(|class| list.contains(class))
		&& !absent.iter().any(|class| list.contains(class))
}

#[wasm_bindgen_test]
fn button_variants_render_their_classes_and_caller_classes_win() {
	let fixture = mount(|| {
		view! {
			<Button>"Save"</Button>
			<Button variant=ButtonVariant::Outline size=ButtonSize::Sm class="px-6">
				"Cancel"
			</Button>
		}
	});

	let save = fixture.find("[data-slot=button]:first-child");
	assert!(has_classes(&save, &["bg-primary", "h-9", "px-4"], &[]));

	let cancel = fixture.find("[data-slot=button]:last-child");
	assert!(
		has_classes(
			&cancel,
			&["border", "bg-background", "h-8"],
			&["bg-primary"]
		),
		"only the outline variant and the small size apply"
	);
	assert!(
		has_classes(&cancel, &["px-6"], &["px-3"]),
		"the caller's padding replaces the size's"
	);
}

#[wasm_bindgen_test]
async fn button_on_press_fires_on_click_unless_disabled() {
	let presses = RwSignal::new(0);
	let fixture = mount(move || {
		view! {
			<Button on_press=Box::new(move |_: &PressEvent| presses.update(|count| *count += 1))>
				"Press"
			</Button>
			<Button
				is_disabled=true
				on_press=Box::new(move |_: &PressEvent| presses.update(|count| *count += 100))
			>
				"Disabled"
			</Button>
		}
	});

	fixture.find("[data-slot=button]:first-child").click();
	fixture.find("[data-slot=button]:last-child").click();
	flush_microtasks().await;

	assert_eq!(presses.get_untracked(), 1);
	assert!(
		fixture
			.find("[data-slot=button]:last-child")
			.has_attribute("disabled")
	);
}

#[wasm_bindgen_test]
async fn toggle_flips_aria_pressed_on_click() {
	let fixture = mount(|| {
		view! { <Toggle variant=ToggleVariant::Outline>"B"</Toggle> }
	});
	let toggle = fixture.find("[data-slot=toggle]");

	assert!(has_classes(&toggle, &["border-input"], &[]));
	assert_eq!(
		toggle.get_attribute("aria-pressed").as_deref(),
		Some("false")
	);

	toggle.click();
	flush_microtasks().await;

	assert_eq!(
		toggle.get_attribute("aria-pressed").as_deref(),
		Some("true")
	);
}

#[wasm_bindgen_test]
async fn toggle_group_presses_one_item_at_a_time() {
	let fixture = mount(|| {
		view! {
			<ToggleGroup variant=ToggleVariant::Outline default_value=vec!["left".to_string()]>
				<ToggleGroupItem value="left">"L"</ToggleGroupItem>
				<ToggleGroupItem value="right">"R"</ToggleGroupItem>
			</ToggleGroup>
		}
	});
	let left = fixture.find("[data-slot=toggle-group-item]:first-child");
	let right = fixture.find("[data-slot=toggle-group-item]:last-child");

	assert!(
		has_classes(&right, &["border-l-0"], &[]),
		"items take the group's variant"
	);
	assert_eq!(left.get_attribute("aria-pressed").as_deref(), Some("true"));

	right.click();
	flush_microtasks().await;

	assert_eq!(left.get_attribute("aria-pressed").as_deref(), Some("false"));
	assert_eq!(right.get_attribute("aria-pressed").as_deref(), Some("true"));
}

#[wasm_bindgen_test]
async fn switch_toggles_on_click() {
	let changes = RwSignal::new(Vec::new());
	let fixture = mount(move || {
		view! {
			<Switch on_change=Box::new(move |on| changes.update(|changes| changes.push(on)))>
				"Wi-Fi"
			</Switch>
		}
	});
	let switch = fixture.find("[data-slot=switch]");
	let input = fixture.input("[data-slot=switch] input[role=switch]");

	assert!(has_classes(&switch, &["group/switch"], &[]));
	assert_eq!(
		input.get_attribute("data-state").as_deref(),
		Some("unchecked")
	);

	switch.click();
	flush_microtasks().await;

	assert!(input.checked());
	assert_eq!(
		input.get_attribute("data-state").as_deref(),
		Some("checked")
	);

	switch.click();
	flush_microtasks().await;

	assert!(!input.checked());
	assert_eq!(changes.get_untracked(), vec![true, false]);
}

#[wasm_bindgen_test]
async fn checkbox_toggles_on_click() {
	let changes = RwSignal::new(Vec::new());
	let fixture = mount(move || {
		view! {
			<Checkbox on_change=Box::new(move |on| changes.update(|changes| changes.push(on)))>
				"Accept"
			</Checkbox>
		}
	});
	let checkbox = fixture.find("[data-slot=checkbox]");
	let input = fixture.input("[data-slot=checkbox] input[type=checkbox]");

	assert!(!input.checked());
	assert!(
		fixture
			.root
			.query_selector("[data-slot=checkbox-control]")
			.unwrap()
			.is_some()
	);

	checkbox.click();
	flush_microtasks().await;

	assert!(input.checked());
	assert_eq!(changes.get_untracked(), vec![true]);
}

#[wasm_bindgen_test]
async fn radio_group_selects_the_clicked_item() {
	let fixture = mount(|| {
		view! {
			<RadioGroup label="Density" default_value="default">
				<RadioGroupItem value="default">"Default"</RadioGroupItem>
				<RadioGroupItem value="compact">"Compact"</RadioGroupItem>
			</RadioGroup>
		}
	});
	let first = fixture.input("[data-slot=radio-group-item] input[value=default]");
	let second = fixture.input("[data-slot=radio-group-item] input[value=compact]");

	assert_eq!(
		fixture
			.find("[data-slot=radio-group]")
			.get_attribute("role")
			.as_deref(),
		Some("radiogroup")
	);
	assert!(first.checked());

	fixture
		.find("[data-slot=radio-group-item]:last-child")
		.click();
	flush_microtasks().await;

	assert!(second.checked());
	assert!(!first.checked());
}

#[wasm_bindgen_test]
fn slider_renders_its_range_values() {
	let fixture = mount(|| {
		view! { <Slider label="Volume" default_value=40.0 max=200.0 step=5.0 class="w-60" /> }
	});
	let input = fixture.input("[data-slot=slider] input[type=range]");

	assert_eq!(input.value(), "40");
	assert_eq!(input.get_attribute("min").as_deref(), Some("0"));
	assert_eq!(input.get_attribute("max").as_deref(), Some("200"));
	assert_eq!(input.get_attribute("step").as_deref(), Some("5"));
	assert_eq!(input.get_attribute("aria-label").as_deref(), Some("Volume"));
	assert_eq!(
		fixture
			.find("[data-percentage]")
			.get_attribute("data-percentage")
			.as_deref(),
		Some("20.0")
	);

	assert!(has_classes(
		&fixture.find("[data-slot=slider]"),
		&["w-60"],
		&["w-full"]
	));
}

#[wasm_bindgen_test]
fn progress_renders_its_aria_values() {
	let fixture = mount(|| {
		view! {
			<Progress value=Signal::from(50.0) max_value=200.0 attr:aria-label="Upload" />
			<Progress is_indeterminate=true attr:aria-label="Loading" />
		}
	});
	let progress = fixture.find("[data-slot=progress]:first-child");

	assert_eq!(
		progress.get_attribute("role").as_deref(),
		Some("progressbar")
	);
	assert_eq!(
		progress.get_attribute("aria-valuenow").as_deref(),
		Some("50")
	);
	assert_eq!(
		progress.get_attribute("aria-valuemin").as_deref(),
		Some("0")
	);
	assert_eq!(
		progress.get_attribute("aria-valuemax").as_deref(),
		Some("200")
	);
	assert_eq!(
		progress.get_attribute("aria-valuetext").as_deref(),
		Some("25%")
	);
	assert_eq!(
		progress.get_attribute("aria-label").as_deref(),
		Some("Upload")
	);

	let loading = fixture.find("[data-slot=progress]:last-child");
	assert!(
		!loading.has_attribute("aria-valuenow"),
		"an indeterminate bar has no current value"
	);
}

#[wasm_bindgen_test]
fn separator_renders_its_orientation() {
	let fixture = mount(|| {
		view! {
			<Separator />
			<Separator orientation=SeparatorOrientation::Vertical label="Sections" />
		}
	});
	let horizontal = fixture.find("[data-slot=separator]:first-child");
	let vertical = fixture.find("[data-slot=separator]:last-child");

	assert_eq!(
		horizontal.get_attribute("data-orientation").as_deref(),
		Some("horizontal")
	);
	assert_eq!(
		horizontal.get_attribute("role").as_deref(),
		Some("presentation")
	);
	assert_eq!(
		vertical.get_attribute("data-orientation").as_deref(),
		Some("vertical")
	);
	assert_eq!(vertical.get_attribute("role").as_deref(), Some("separator"));
}

#[wasm_bindgen_test]
fn input_merges_classes_and_binds_its_value() {
	let fixture = mount(|| {
		view! { <Input default_value="Ada" placeholder="Name" class="h-12" /> }
	});
	let input = fixture.input("input[data-slot=input]");
	assert_eq!(input.value(), "Ada");
	assert!(has_classes(&input, &["h-12"], &["h-9"]));
}

#[wasm_bindgen_test]
fn avatar_shows_the_fallback_without_an_image() {
	let fixture = mount(|| view! { <Avatar fallback="IJ" class="size-10" /> });
	let avatar = fixture.find("[data-slot=avatar]");

	assert_eq!(
		avatar.get_attribute("data-state").as_deref(),
		Some("fallback")
	);
	assert_eq!(avatar.text_content().as_deref(), Some("IJ"));
	assert!(has_classes(&avatar, &["size-10"], &["size-8"]));
}
