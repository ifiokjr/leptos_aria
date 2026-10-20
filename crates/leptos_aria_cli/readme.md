<p align="center">
	<a href="https://ifiokjr.github.io/leptos_aria/"><img src="https://ifiokjr.github.io/leptos_aria/logo.svg" alt="leptos_aria" width="72" height="72" /></a>
</p>

# leptos_aria_cli

> Copy Tailwind-styled `leptos_aria` components into your Leptos project, shadcn-style.

```bash
cargo install leptos_aria_cli
leptos-aria init               # leptos-aria.json, theme CSS variables, the `cn` helper
leptos-aria add button dialog  # copy components into src/components/ui
leptos-aria diff               # compare your copies with the registry
leptos-aria build              # turn your registry.json into a hostable registry
```

The copied files are yours to edit. They layer Tailwind classes over the headless [`leptos_aria`](https://crates.io/crates/leptos_aria) crates, which keep handling roles, keyboard, and focus. Items use shadcn/ui's registry-item schema, and namespaced registries (`leptos-aria add @acme/card`) work as they do with shadcn.

[Components](https://ifiokjr.github.io/leptos_aria/components.html) · [CLI guide](https://ifiokjr.github.io/leptos_aria/cli.html) · [Repository](https://github.com/ifiokjr/leptos_aria)
