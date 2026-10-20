---
main: minor
---

# shadcn-style component registry and CLI

`leptos-aria init` writes `leptos-aria.json`, shadcn's theme CSS variables, and a `cn` helper backed by `tw_merge`. `leptos-aria add <name>` copies Tailwind-styled components into `src/components/ui`, declares their modules, runs `cargo add`, and writes their CSS. `list`, `view`, and `diff` read the registry, and `build` turns any `registry.json` into a hostable registry in shadcn's registry-item schema, with namespaced registries supported. The website gains a component index, a page per component, and a CLI guide.
