pub use extend::*;
pub use focus_without_scrolling::*;
pub use global_listeners::*;
use leptos::Scope;
pub use platform::*;
pub use run_after_transition::*;
pub use silly_map::*;
pub use traits::*;
pub use virtual_event::*;

mod extend;
mod focus_without_scrolling;
mod global_listeners;
mod platform;
mod run_after_transition;
mod silly_map;
mod traits;
mod virtual_event;

/// Provide any context and values into the scope.
pub fn use_provider(cx: Scope) {
  ElementTransitionsContext::provide(cx);
  TransitionCallbacksContext::provide(cx);

  setup_transition_listener(cx);
}
