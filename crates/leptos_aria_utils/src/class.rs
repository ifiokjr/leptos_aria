use leptos::prelude::*;

/// The reactive `class` attribute for a component root: its built-in base
/// class followed by the caller's classes.
///
/// Components declare `#[prop(optional, into)] class: Signal<String>`, so
/// callers write `class="data-[state=open]:font-bold"` directly, or pass a
/// closure for classes that change. The base class stays first so the
/// stable `leptos-aria-*` hook is always present. With neither a base nor a
/// caller class the result is `None`, which renders no `class` attribute.
///
/// ```rust ignore
/// view! { <button class=merge_class("leptos-aria-button", class)>...</button> }
/// ```
pub fn merge_class(
	base: &'static str,
	class: Signal<String>,
) -> impl Fn() -> Option<String> + Copy {
	move || {
		class.with(|class| {
			let class = class.trim();

			match (base.is_empty(), class.is_empty()) {
				(true, true) => None,
				(false, true) => Some(base.to_string()),
				(true, false) => Some(class.to_string()),
				(false, false) => Some(format!("{base} {class}")),
			}
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn joins_the_base_and_caller_classes() {
		let owner = Owner::new();
		owner.set();

		let class = RwSignal::new(String::new());
		let merged = merge_class("leptos-aria-button", class.into());

		assert_eq!(merged().as_deref(), Some("leptos-aria-button"));

		class.set("  px-4 data-[pressed]:scale-95 ".to_string());
		assert_eq!(
			merged().as_deref(),
			Some("leptos-aria-button px-4 data-[pressed]:scale-95")
		);

		let bare = merge_class("", Signal::from("px-4"));
		assert_eq!(bare().as_deref(), Some("px-4"));

		let empty = merge_class("", Signal::from(" "));
		assert_eq!(empty(), None);
	}
}
