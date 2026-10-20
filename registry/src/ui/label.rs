//! A caption for a form control.
//!
//! ```rust ignore
//! use crate::components::ui::label::Label;
//!
//! view! {
//!     <Label html_for="email">"Email"</Label>
//!     <input id="email" type="email" />
//! }
//! ```

use leptos::prelude::*;

use super::utils::cn;

const BASE: &str = "flex items-center gap-2 text-sm leading-none font-medium select-none \
                    group-data-[disabled=true]:pointer-events-none \
                    group-data-[disabled=true]:opacity-50 peer-disabled:cursor-not-allowed \
                    peer-disabled:opacity-50";

/// A caption for a form control.
#[component]
pub fn Label(
	/// The `id` of the control this label names.
	#[prop(optional, into)]
	html_for: MaybeProp<String>,
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let class = move || class.with(|class| cn(&[BASE, class]));

	view! {
		<label data-slot="label" for=move || html_for.get() class=class>
			{children()}
		</label>
	}
}
