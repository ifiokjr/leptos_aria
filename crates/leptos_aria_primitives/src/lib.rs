#![doc(html_logo_url = "https://ifiokjr.github.io/leptos_aria/logo.svg")]
#![doc(html_favicon_url = "https://ifiokjr.github.io/leptos_aria/favicon.ico")]
#![allow(clippy::needless_pass_by_value)]

use leptos::prelude::*;
use leptos_aria_utils::merge_class;

/// Heading renders a semantic heading level.
#[component]
pub fn Heading(
	/// The heading level, 2 by default.
	#[prop(optional, into)]
	level: Signal<u8>,
	/// Classes added after the built-in `leptos-aria-heading` class.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	// The level is resolved once at mount: headings do not change level in
	// place, and re-calling the children per level change is not possible with
	// `FnOnce` children.
	let level = level.get_untracked().clamp(1, 6);
	let class = merge_class("leptos-aria-heading", class);

	match level {
		1 => view! { <h1 class=class>{children()}</h1> }.into_any(),
		2 => view! { <h2 class=class>{children()}</h2> }.into_any(),
		3 => view! { <h3 class=class>{children()}</h3> }.into_any(),
		4 => view! { <h4 class=class>{children()}</h4> }.into_any(),
		5 => view! { <h5 class=class>{children()}</h5> }.into_any(),
		_ => view! { <h6 class=class>{children()}</h6> }.into_any(),
	}
}

/// Text renders text with a size, mapped to the matching heading level for
/// semantics while remaining visually independent of it.
#[component]
pub fn Text(
	/// The text size, one of the semantic sizes.
	#[prop(optional, into)]
	size: Signal<TextSize>,
	/// Classes added after the built-in `leptos-aria-text` and
	/// `leptos-aria-text--{size}` classes.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	view! {
		<span class=move || merge_class(size.get().class(), class)()>
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
	/// The built-in classes of the text root: the base class and its size
	/// modifier.
	fn class(self) -> &'static str {
		match self {
			TextSize::Body => "leptos-aria-text leptos-aria-text--body",
			TextSize::Caption => "leptos-aria-text leptos-aria-text--caption",
			TextSize::Heading => "leptos-aria-text leptos-aria-text--heading",
			TextSize::Title => "leptos-aria-text leptos-aria-text--title",
		}
	}
}

/// VisuallyHidden renders content that is available to screen readers but
/// invisible on screen.
#[component]
pub fn VisuallyHidden(
	/// Classes added after the built-in `leptos-aria-visually-hidden` class.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	view! {
		<span class=merge_class("leptos-aria-visually-hidden", class)>
			{children()}
		</span>
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

	#[wasm_bindgen_test]
	async fn text_appends_the_caller_class_after_the_built_in_classes() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! {
				<Text size=TextSize::Caption class="primitives-test-text text-sm">
					"Caption"
				</Text>
			}
		});

		let text = document()
			.query_selector(".primitives-test-text")
			.unwrap()
			.unwrap();

		assert_eq!(
			text.get_attribute("class").as_deref(),
			Some("leptos-aria-text leptos-aria-text--caption primitives-test-text text-sm")
		);
	}

	/// A one-pixel SVG, so the image loads without the network.
	const PIXEL: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' \
	                     width='1' height='1'/%3E";

	/// Wait up to a second for `done` to hold, yielding to the browser
	/// between checks so image events can fire.
	async fn wait_for(done: impl Fn() -> bool) -> bool {
		for _ in 0..100 {
			if done() {
				return true;
			}

			let promise = js_sys::Promise::new(&mut |resolve, _| {
				window()
					.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 10)
					.unwrap();
			});
			wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
		}

		done()
	}

	#[wasm_bindgen_test]
	async fn avatar_shows_its_image_once_it_loads() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! {
				<Avatar src=PIXEL.to_string() fallback="IJ" class="avatar-test-loads" />
			}
		});

		let avatar = document()
			.query_selector(".avatar-test-loads")
			.unwrap()
			.unwrap();
		let loaded =
			wait_for(|| avatar.get_attribute("data-state").as_deref() == Some("loaded")).await;
		let image = avatar.query_selector("img").unwrap().unwrap();

		assert!(loaded, "the load event marks the avatar loaded");
		assert!(!image.has_attribute("hidden"), "the loaded image shows");
		assert!(
			avatar
				.query_selector(".leptos-aria-avatar__fallback")
				.unwrap()
				.is_none(),
			"the fallback leaves once the image shows"
		);
	}

	#[wasm_bindgen_test]
	async fn avatar_keeps_its_fallback_when_the_image_fails() {
		console_error_panic_hook::set_once();

		mount_to_body(|| {
			view! {
				<Avatar src="data:image/png;base64,broken".to_string() fallback="IJ" class="avatar-test-fails" />
			}
		});

		let avatar = document()
			.query_selector(".avatar-test-fails")
			.unwrap()
			.unwrap();
		let image = avatar.query_selector("img").unwrap().unwrap();
		let failed = wait_for(|| {
			image
				.unchecked_ref::<web_sys::HtmlImageElement>()
				.complete()
		})
		.await;

		assert!(failed, "the broken image finishes loading");
		assert!(image.has_attribute("hidden"), "a broken image stays hidden");
		assert_eq!(
			avatar.get_attribute("data-state").as_deref(),
			Some("fallback")
		);
		assert_eq!(avatar.text_content().as_deref(), Some("IJ"));
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
	/// Classes added after the built-in `leptos-aria-separator` class.
	#[prop(optional, into)]
	class: Signal<String>,
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
			class=merge_class("leptos-aria-separator", class)
			role=move || role.get()
			aria-orientation=move || has_label.then(|| orientation.get())
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
	/// Classes added after the built-in `leptos-aria-aspect-ratio` class.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	view! {
		<div
			class=merge_class("leptos-aria-aspect-ratio", class)
			style=move || format!("aspect-ratio: {} / {};", ratio.get(), 1.0)
		>
			{children()}
		</div>
	}
}

/// An avatar shows an image with text and image-loading fallbacks.
///
/// The image stays in the tree, `hidden`, while it loads, so its `load` event
/// can fire; the fallback shows until then, and for good when the load fails
/// or no image is set.
#[component]
pub fn Avatar(
	/// The image URL when there is one.
	#[prop(into)]
	src: Option<String>,
	/// The text shown while no image is available.
	#[prop(into)]
	fallback: String,
	/// Classes added after the built-in `leptos-aria-avatar` class.
	#[prop(optional, into)]
	class: Signal<String>,
) -> impl IntoView {
	let loaded = RwSignal::new(false);
	let alt = fallback.clone();

	view! {
		<span class=merge_class("leptos-aria-avatar", class) data-state=move || {
			if loaded.get() { "loaded" } else { "fallback" }
		}>
			{src
				.map(|src| {
					view! {
						<img
							class="leptos-aria-avatar__image"
							src=src
							alt=alt
							hidden=move || !loaded.get()
							on:load=move |_| loaded.set(true)
							on:error=move |_| loaded.set(false)
						/>
					}
				})}
			{move || {
				(!loaded.get())
					.then(|| {
						view! {
							<span class="leptos-aria-avatar__fallback">{fallback.clone()}</span>
						}
					})
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
pub fn ScrollArea(
	/// Classes added after the built-in `leptos-aria-scroll-area` class.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	view! {
		<div class=merge_class("leptos-aria-scroll-area", class) role="group" style="overflow: auto;">
			{children()}
		</div>
	}
}
