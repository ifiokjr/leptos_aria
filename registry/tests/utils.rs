//! `cn` merges classes the way the components rely on.

use leptos_aria_registry::ui::utils::cn;

#[test]
fn later_classes_win_conflicts() {
	assert_eq!(cn(&["px-2 py-1", "px-4"]), "py-1 px-4");
	assert_eq!(cn(&["bg-primary", ""]), "bg-primary");
	assert_eq!(
		cn(&["rounded-md data-[state=open]:bg-accent", "rounded-full"]),
		"data-[state=open]:bg-accent rounded-full"
	);
}

/// `tw_merge` reads `ring-[3px]` as a colour, so components use `ring-3`,
/// which keeps both the width and the colour.
#[test]
fn keeps_ring_width_and_colour() {
	assert_eq!(
		cn(&["focus-visible:ring-3 focus-visible:ring-ring/50", ""]),
		"focus-visible:ring-3 focus-visible:ring-ring/50"
	);
}
