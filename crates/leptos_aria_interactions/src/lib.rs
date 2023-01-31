pub use context::*;
use leptos::Scope;
use leptos_aria_utils::ContextProvider;
pub(crate) use text_selection::*;
pub use use_press::*;

pub fn inject_providers(cx: Scope) {
  UserSelectContext::provide(cx);
  ElementListContext::provide(cx);
  SelectionContext::provide(cx);
}

mod context;
mod text_selection;
mod use_press;
