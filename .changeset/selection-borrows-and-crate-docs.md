---
main: patch
---

# Selection checks no longer clone whole collections

List box, tags, tree, tabs, toggle groups, checkbox groups, combo box, and selection state now borrow their reactive sets instead of cloning them per item and per keypress, removing quadratic work on selection changes. Every crate also gains a real description, a logo'd readme for crates.io, and the leptos_aria logo and favicon on docs.rs.
