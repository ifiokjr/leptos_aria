#![allow(clippy::needless_pass_by_value)]

use leptos::prelude::*;

/// Heading renders a semantic heading level.
#[component]
pub fn Heading(
	/// The heading level, 2 by default.
	#[prop(optional, into)]
	level: Signal<u8>,
	children: Children,
) -> impl IntoView {
	// The level is resolved once at mount: headings do not change level in
	// place, and re-calling the children per level change is not possible with
	// `FnOnce` children.
	let level = level.get_untracked().clamp(1, 6);

	match level {
		1 => view! { <h1 class="leptos-aria-heading">{children()}</h1> }.into_any(),
		2 => view! { <h2 class="leptos-aria-heading">{children()}</h2> }.into_any(),
		3 => view! { <h3 class="leptos-aria-heading">{children()}</h3> }.into_any(),
		4 => view! { <h4 class="leptos-aria-heading">{children()}</h4> }.into_any(),
		5 => view! { <h5 class="leptos-aria-heading">{children()}</h5> }.into_any(),
		_ => view! { <h6 class="leptos-aria-heading">{children()}</h6> }.into_any(),
	}
}

/// Text renders text with a size, mapped to the matching heading level for
/// semantics while remaining visually independent of it.
#[component]
pub fn Text(
	/// The text size, one of the semantic sizes.
	#[prop(optional, into)]
	size: Signal<TextSize>,
	children: Children,
) -> impl IntoView {
	view! {
		<span class=move || format!("leptos-aria-text leptos-aria-text--{}", size.get().class())>
			{children()}
		</span>
	}
}

/// The semantic text sizes of the design system.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum TextSize {
	#[default]
	Body,
	Caption,
	Heading,
	Title,
}

impl TextSize {
	fn class(self) -> &'static str {
		match self {
			TextSize::Body => "body",
			TextSize::Caption => "caption",
			TextSize::Heading => "heading",
			TextSize::Title => "title",
		}
	}
}

/// VisuallyHidden renders content that is available to screen readers but
/// invisible on screen.
#[component]
pub fn VisuallyHidden(children: Children) -> impl IntoView {
	view! {
		<span class="leptos-aria-visually-hidden">
			{children()}
		</span>
	}
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
	use leptos::mount::mount_to_body;
	use wasm_bindgen_test::*;

	use super::*;

	wasm_bindgen_test_configure!(run_in_browser);

	#[wasm_bindgen_test]
	async fn heading_renders_the_clamped_level() {
		console_error_panic_hook::set_once();

		mount_to_body(|| view! { <Heading level=Signal::from(3)>"Title"</Heading> });

		let heading = document().query_selector("h3").unwrap();

		assert!(heading.is_some(), "the level maps to the matching element");
	}

	#[wasm_bindgen_test]
	async fn visually_hidden_content_stays_in_the_tree() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! { <VisuallyHidden>"for screen readers"</VisuallyHidden> }
		});

		let hidden = document()
			.query_selector(".leptos-aria-visually-hidden")
			.unwrap();

		assert!(hidden.is_some(), "the content stays in the DOM for readers");
	}
}
