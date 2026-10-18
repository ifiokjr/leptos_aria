use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::Element;
use web_sys::MouseEvent;

#[wasm_bindgen(inline_js = r#"
  export function leptos_aria_open_link(href, target, download, relationship, window_features) {
    const link = document.createElement("a");
    link.href = href;
    if (target) { link.target = target; }
    if (download) { link.setAttribute("download", download); }
    if (relationship) { link.rel = relationship; }
    if (window_features) { link.setAttribute("data-leptos-aria-window-features", window_features); }
    document.body.appendChild(link);
    link.click();
    link.remove();
  }"#)]
extern "C" {
	/// Navigate to `href` by clicking a synthesized anchor, so browser-level
	/// link semantics (downloads, targets) apply.
	#[wasm_bindgen(catch)]
	fn leptos_aria_open_link(
		href: &str,
		target: Option<String>,
		download: Option<String>,
		relationship: Option<String>,
		window_features: Option<String>,
	) -> Result<(), JsValue>;
}

/// How a link click should navigate.
#[derive(Clone, Debug, Default)]
pub struct LinkTarget {
	/// The URL to navigate to.
	pub href: String,

	/// The window target for the link, e.g. `_blank`.
	pub target: Option<String>,

	/// The download attribute for downloading a resource.
	pub download: Option<String>,

	/// The relationship between the linked resource and the current page.
	pub relationship: Option<String>,

	/// The `window.open` features for custom windows.
	pub window_features: Option<String>,
}

/// Open a link, handling the modifier-key behaviors browsers apply to plain
/// anchors: ctrl/cmd clicks open a new tab, shift clicks a new window, and
/// alt clicks download — all left to the default behavior.
///
/// Returns whether the click was handled by opening the link.
pub fn open_link(event: &MouseEvent, link: &LinkTarget) -> bool {
	if should_open_link_normally(event, link) {
		leptos_aria_open_link(
			&link.href,
			link.target.clone(),
			link.download.clone(),
			link.relationship.clone(),
			link.window_features.clone(),
		)
		.unwrap_or(());
		return true;
	}

	false
}

/// The modifier state which defers to the browser's default link handling.
fn should_open_link_normally(event: &MouseEvent, link: &LinkTarget) -> bool {
	let modifier_click =
		event.shift_key() || event.ctrl_key() || event.meta_key() || event.alt_key();
	let has_target = link.target.is_some();

	!(modifier_click || has_target)
}

/// The `on_click` behavior for a custom link element: plain clicks navigate,
/// modifier clicks keep the browser default.
pub fn handle_link_click(event: &MouseEvent, link: &LinkTarget) {
	if !should_open_link_normally(event, link) {
		return;
	}

	event.prevent_default();
	open_link(event, link);
}

/// Whether an anchor element has an href making it a link.
pub fn is_anchor_with_href(element: &Element) -> bool {
	element
		.get_attribute("href")
		.is_some_and(|href| !href.is_empty())
}
