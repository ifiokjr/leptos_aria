#![doc(html_logo_url = "https://ifiokjr.github.io/leptos_aria/logo.svg")]
#![doc(html_favicon_url = "https://ifiokjr.github.io/leptos_aria/favicon.ico")]

pub use focus_safely::focus_safely;
pub use focus_without_scrolling::*;
pub use global_listeners::*;
pub use interaction_modality::InteractionModality;
pub use interaction_modality::interaction_modality;
pub use interaction_modality::is_focus_visible;
pub use is_focusable::is_focusable;
pub use is_focusable::is_tabbable;
pub use open_link::LinkTarget;
pub use open_link::handle_link_click;
pub use open_link::is_anchor_with_href;
pub use open_link::open_link;
pub use platform::*;
pub use run_after_transition::*;
pub use silly_map::*;
pub use use_id::use_id;
pub use use_label::FieldProps;
pub use use_label::LabelElementType;
pub use use_label::LabelProps;
pub use use_label::UseLabelProps;
pub use use_label::UseLabelResult;
pub use use_label::UseLabelsProps;
pub use use_label::use_label;
pub use use_label::use_labels;
pub use virtual_event::*;

mod focus_safely;
mod focus_without_scrolling;
mod global_listeners;
mod interaction_modality;
mod is_focusable;
mod open_link;
mod platform;
mod run_after_transition;
mod silly_map;
mod use_id;
mod use_label;
mod virtual_event;
