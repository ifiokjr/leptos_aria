//! The `leptos_aria` documentation engine: markdown with embedded component
//! fences, so docs pages can render live `leptos_aria` components inline the
//! way MDX does for React, plus the static site assembly for the website.

pub use markdown::*;

mod markdown;
pub mod site;
