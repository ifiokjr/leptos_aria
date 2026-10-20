//! Class helpers shared by every registry component.

/// Merge Tailwind classes so later classes win conflicts: `cn(&["px-2 py-1",
/// "px-4"])` is `"py-1 px-4"`. The Rust counterpart of shadcn/ui's `cn`, so a
/// `class` you pass to a component overrides its defaults instead of fighting
/// them.
pub fn cn(classes: &[&str]) -> String {
	tw_merge::merge::tw_merge_slice(classes)
}
