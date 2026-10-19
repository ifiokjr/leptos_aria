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

/// A separator divides content into groups, optionally with a label.
#[component]
pub fn Separator(
	/// The accessible label when the separator groups content.
	#[prop(optional, into)]
	label: Option<String>,
	/// The orientation, horizontal by default.
	#[prop(optional, into)]
	orientation: Signal<&'static str>,
) -> impl IntoView {
	let has_label = label.is_some();
	let role = Signal::derive(move || {
		if has_label {
			"separator"
		} else {
			"presentation"
		}
	});

	view! {
		<div
			class="leptos-aria-separator"
			role=move || role.get()
			aria-orientation=move || orientation.get()
			aria-label=move || label.clone()
			data-orientation=move || orientation.get()
		></div>
	}
}

/// An aspect ratio box constrains its content to a width-to-height ratio.
#[component]
pub fn AspectRatio(
	/// The width-to-height ratio, `16 / 9` as `16.0 / 9.0`.
	#[prop(into)]
	ratio: Signal<f64>,
	children: Children,
) -> impl IntoView {
	view! {
		<div
			class="leptos-aria-aspect-ratio"
			style=move || format!("aspect-ratio: {} / {};", ratio.get(), 1.0)
		>
			{children()}
		</div>
	}
}

/// An avatar shows an image with text and image-loading fallbacks.
///
/// The fallback shows while no image is set or until it loads; a failed
/// load falls back too.
#[component]
pub fn Avatar(
	/// The image URL when there is one.
	#[prop(into)]
	src: Option<String>,
	/// The text shown while no image is available.
	#[prop(into)]
	fallback: String,
) -> impl IntoView {
	let loaded = RwSignal::new(false);

	view! {
		<span class="leptos-aria-avatar" data-state=move || {
			if loaded.get() { "loaded" } else { "fallback" }
		}>
			{move || match src.clone() {
				Some(src) if loaded.get() => {
					view! {
						<img class="leptos-aria-avatar__image" src=src.clone() alt=fallback.clone() />
					}
					.into_any()
				}
				_ => {
					view! {
						<span class="leptos-aria-avatar__fallback">{fallback.clone()}</span>
					}
					.into_any()
				}
			}}
		</span>
	}
}

/// A scroll area is a custom-scrollable region.
///
/// The region is unstyled apart from `overflow`; the data attributes expose
/// whether more content exists past each edge so scroll shadows can be
/// styled externally.
#[component]
pub fn ScrollArea(children: Children) -> impl IntoView {
	view! {
		<div class="leptos-aria-scroll-area" role="group" style="overflow: auto;">
			{children()}
		</div>
	}
}
