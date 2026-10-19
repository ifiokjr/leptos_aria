#![doc(html_logo_url = "https://ifiokjr.github.io/leptos_aria/logo.svg")]
#![doc(html_favicon_url = "https://ifiokjr.github.io/leptos_aria/favicon.ico")]

use leptos::prelude::*;
pub use leptos_aria_button::BoxedPressEventCallback;
pub use leptos_aria_interactions::UseFocusableProps;
use leptos_aria_interactions::UsePressProps;
pub use leptos_aria_interactions::use_focusable;
use leptos_aria_interactions::use_press;
use leptos_aria_utils::LinkTarget;
use leptos_aria_utils::handle_link_click;

/// A link allows a user to navigate to another page or resource within a web
/// page or application.
///
/// A port of the `react-aria` `useLink` hook: plain clicks navigate through a
/// synthesized anchor, and modifier clicks keep the browser's default link
/// behavior.
#[component]
pub fn Link(
	/// The URL to navigate to.
	#[prop(optional, into)]
	href: String,
	/// The window target for the link, e.g. `_blank`.
	#[prop(optional, into)]
	target: Option<String>,
	/// The relationship between the linked resource and the current page.
	#[prop(optional, into)]
	rel: Option<String>,
	/// Whether the link is disabled.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	/// Handler that is called when the press is released over the target.
	#[prop(optional)]
	on_press: Option<BoxedPressEventCallback>,
	children: Children,
) -> impl IntoView {
	let press = use_press(UsePressProps {
		on_press,
		is_disabled: Some(is_disabled),
		..Default::default()
	});

	let link = LinkTarget {
		href: href.clone(),
		target: target.clone(),
		download: None,
		relationship: rel.clone(),
		window_features: None,
	};

	let navigate = link.clone();

	view! {
		<a
			href=move || if is_disabled.get() { String::new() } else { href.clone() }
			target=move || target.clone()
			rel=move || rel.clone()
			aria-disabled=move || is_disabled.get().to_string()
			on:click=move |event| {
				(press.on_click)(event.clone());

				if !is_disabled.get_untracked() {
					handle_link_click(&event, &navigate);
				}
			}
			on:dragstart=move |event| (press.on_drag_start)(event)
			on:keydown=move |event| (press.on_key_down)(event)
			on:keyup=move |event| (press.on_key_up)(event)
			on:mousedown=move |event| (press.on_mouse_down)(event)
			on:pointerdown=move |event| (press.on_pointer_down)(event)
			on:pointerup=move |event| (press.on_pointer_up)(event)
		>
			{children()}
		</a>
	}
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
	use leptos::mount::mount_to_body;
	use wasm_bindgen_test::*;

	use super::*;

	wasm_bindgen_test_configure!(run_in_browser);

	#[wasm_bindgen_test]
	fn renders_an_accessible_anchor() {
		console_error_panic_hook::set_once();

		mount_to_body(|| view! { <Link href="https://example.com">"Example"</Link> });

		let anchor = document().query_selector("a").unwrap().unwrap();

		assert_eq!(
			anchor.get_attribute("href").as_deref(),
			Some("https://example.com")
		);
		assert_eq!(anchor.text_content().as_deref(), Some("Example"));
	}
}
