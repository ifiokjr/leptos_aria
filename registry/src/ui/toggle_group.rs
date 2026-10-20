//! A row of toggles joined into one control. One is pressed at a time unless
//! `mode` is `ToggleGroupMode::Multiple`.
//!
//! ```rust ignore
//! use crate::components::ui::toggle::ToggleVariant;
//! use crate::components::ui::toggle_group::ToggleGroup;
//! use crate::components::ui::toggle_group::ToggleGroupItem;
//!
//! view! {
//!     <ToggleGroup variant=ToggleVariant::Outline default_value=vec!["left".to_string()]>
//!         <ToggleGroupItem value="left" attr:aria-label="Align left">"L"</ToggleGroupItem>
//!         <ToggleGroupItem value="center" attr:aria-label="Align center">"C"</ToggleGroupItem>
//!         <ToggleGroupItem value="right" attr:aria-label="Align right">"R"</ToggleGroupItem>
//!     </ToggleGroup>
//! }
//! ```

use leptos::prelude::*;
use leptos_aria::BoxedToggleGroupChangeCallback;
use leptos_aria::ToggleGroup as AriaToggleGroup;
use leptos_aria::ToggleGroupItem as AriaToggleGroupItem;
use leptos_aria::ToggleGroupMode;

use super::toggle::TOGGLE_BASE;
use super::toggle::ToggleSize;
use super::toggle::ToggleVariant;
use super::utils::cn;

const ITEM: &str = "min-w-0 flex-1 shrink-0 rounded-none shadow-none first:rounded-l-md \
                    last:rounded-r-md focus:z-10 focus-visible:z-10";

/// The look every item of a group shares.
#[derive(Clone, Copy)]
struct ToggleGroupStyle {
	variant: Signal<ToggleVariant>,
	size: Signal<ToggleSize>,
}

/// A group of toggles. Selection and `aria-pressed` come from
/// `leptos_aria::ToggleGroup`.
#[component]
pub fn ToggleGroup(
	/// The look of every item.
	#[prop(optional, into)]
	variant: Signal<ToggleVariant>,
	/// The size of every item.
	#[prop(optional, into)]
	size: Signal<ToggleSize>,
	/// Whether one or many items can be pressed, one by default.
	#[prop(optional, into)]
	mode: Signal<ToggleGroupMode>,
	/// The values of the items pressed at first (uncontrolled).
	#[prop(optional, into)]
	default_value: Vec<String>,
	/// The values of the pressed items (controlled).
	#[prop(optional, into)]
	value: Option<Signal<Vec<String>>>,
	/// Called with every pressed value when one changes.
	#[prop(optional)]
	on_change: Option<BoxedToggleGroupChangeCallback>,
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	provide_context(ToggleGroupStyle { variant, size });

	let class = move || {
		class.with(|class| {
			let shadow = match variant.get() {
				ToggleVariant::Default => "",
				ToggleVariant::Outline => "shadow-xs",
			};

			cn(&[
				"group/toggle-group flex w-fit items-center rounded-md",
				shadow,
				class,
			])
		})
	};

	view! {
		<AriaToggleGroup
			mode=mode
			default_value=default_value
			nostrip:value=value
			nostrip:on_change=on_change
			class=class
			attr:data-slot="toggle-group"
		>
			{children()}
		</AriaToggleGroup>
	}
}

/// One toggle inside a [`ToggleGroup`], styled by the group's variant and
/// size.
#[component]
pub fn ToggleGroupItem(
	/// The value this item adds to the group's selection.
	#[prop(into)]
	value: String,
	/// Whether this item is disabled.
	#[prop(optional, into)]
	is_disabled: Signal<bool>,
	/// Classes merged over the defaults; yours win conflicts.
	#[prop(optional, into)]
	class: Signal<String>,
	children: Children,
) -> impl IntoView {
	let style = expect_context::<ToggleGroupStyle>();

	let class = move || {
		class.with(|class| {
			let variant = style.variant.get();
			let border = match variant {
				ToggleVariant::Default => "",
				ToggleVariant::Outline => "border-l-0 first:border-l",
			};

			cn(&[
				TOGGLE_BASE,
				variant.class(),
				style.size.get().class(),
				ITEM,
				border,
				class,
			])
		})
	};

	view! {
		<AriaToggleGroupItem
			value=value
			is_disabled=is_disabled
			class=class
			attr:data-slot="toggle-group-item"
		>
			{children()}
		</AriaToggleGroupItem>
	}
}
