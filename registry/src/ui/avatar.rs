//! A round user image with a text fallback.
//!
//! ```rust ignore
//! use crate::components::ui::avatar::Avatar;
//!
//! view! {
//!     <Avatar src="https://github.com/ifiokjr.png" fallback="IJ" />
//!     <Avatar fallback="CN" class="size-10" />
//! }
//! ```

use leptos::prelude::*;
use leptos_aria::Avatar as AriaAvatar;

use super::utils::cn;

/// The root. `leptos_aria` keeps the `img` hidden and the fallback `span`
/// showing until the image loads, then sets `data-state` to `loaded`.
const BASE: &str = "relative flex size-8 shrink-0 overflow-hidden rounded-full \
                    [&>img]:aspect-square [&>img]:size-full [&>span]:flex [&>span]:size-full \
                    [&>span]:items-center [&>span]:justify-center [&>span]:rounded-full \
                    [&>span]:bg-muted";

/// An avatar. Image loading state and the fallback come from
/// `leptos_aria::Avatar`.
#[component]
pub fn Avatar(
	/// The image URL, when there is one.
	#[prop(optional, into)]
	src: Option<String>,
	/// The text shown until the image loads, usually initials. Also the
	/// image's alt text.
	#[prop(into)]
	fallback: String,
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
) -> impl IntoView {
	let class = move || class.with(|class| cn(&[BASE, class]));

	view! { <AriaAvatar src=src fallback=fallback class=class attr:data-slot="avatar" /> }
}
