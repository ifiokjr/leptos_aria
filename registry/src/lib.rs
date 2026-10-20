//! The source of the `leptos_aria` component registry.
//!
//! `leptos-aria add <name>` copies files from `src/ui` into a project, the
//! way shadcn/ui does. Keeping them in a crate means CI compiles and lints
//! every component exactly as users receive it. `registry.json` describes
//! each item; `leptos-aria build` turns it into the `/r/<name>.json` files the
//! website serves.

pub mod ui;
