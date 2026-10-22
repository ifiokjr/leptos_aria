# CLI

`leptos-aria` copies styled, accessible components into your Leptos project, the way the [shadcn/ui](https://ui.shadcn.com) CLI does for React. The files become yours: change the classes, the markup, or the API. Behaviour (keyboard, focus, ARIA) keeps coming from the headless `leptos_aria` crates underneath.

```bash
cargo install leptos_aria_cli
```

Skip the compile with a prebuilt binary through [`cargo-binstall`](https://github.com/cargo-bins/cargo-binstall):

```bash
cargo binstall leptos_aria_cli
```

Every [GitHub release](https://github.com/ifiokjr/leptos_aria/releases) also carries `leptos-aria` archives for Linux (glibc and musl), macOS, and Windows on x86_64 and aarch64, each with SHA-256 and SHA-512 checksums. The archives are built by the release workflow and carry a build provenance attestation, which `gh` can verify:

```bash
gh attestation verify leptos-aria-x86_64-unknown-linux-gnu-v0.1.2.tar.gz --repo ifiokjr/leptos_aria
```

## init

Run once, from the crate that renders your UI:

```bash
leptos-aria init
```

It writes `leptos-aria.json`, then installs the `default` style:

- **Theme:** the theme's CSS variables (`--background`, `--primary`, `--radius`, ...) and their `@theme inline` mapping go into your Tailwind CSS file, plus a `dark` variant driven by `class="dark"` on `<html>`.
- **Helper:** `src/components/ui/utils.rs` gets the `cn` helper, which merges Tailwind classes so the ones you pass win conflicts.
- **Modules:** the module declarations (`mod components;`, `pub mod ui;`, `pub mod utils;`) are added so everything compiles.
- **Crates:** `cargo add leptos_aria tw_merge`.

| Flag          | Default             | What it does                                                                                                                        |
| ------------- | ------------------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| `--css`       | detected            | The Tailwind entry CSS file. Common locations such as `style/tailwind.css` and `input.css` are checked for `@import "tailwindcss"`. |
| `--ui`        | `src/components/ui` | The module directory that receives components.                                                                                      |
| `--force`     |                     | Replace an existing `leptos-aria.json`.                                                                                             |
| `--skip-deps` |                     | Print the `cargo add` command instead of running it.                                                                                |

## add

```bash
leptos-aria add button dialog
leptos-aria add --all
```

Each component arrives with everything it builds on (`registryDependencies`), in dependency order. Its file lands in the component directory, its `pub mod` line is added to `mod.rs`, its crates are added with `cargo add`, and any CSS it needs is written into your Tailwind file.

Running `add` again is safe. Files that already match are left alone. Files you have edited are kept unless you pass `--overwrite`. Each item's CSS lives between `/* leptos-aria: <name> */` markers, so re-installing replaces its block instead of duplicating it.

| Flag          | What it does                                         |
| ------------- | ---------------------------------------------------- |
| `--all`       | Add every component in the registry.                 |
| `--overwrite` | Replace files that differ from the registry.         |
| `--dry-run`   | Show what would change without writing anything.     |
| `--skip-deps` | Print the `cargo add` command instead of running it. |

Besides bare names, `add` accepts a URL (`https://example.com/r/card.json`), a local path ending in `.json`, or a namespaced name from your own registries (`@acme/card`).

## list, view, and diff

```bash
leptos-aria list            # every item the registry serves
leptos-aria view button     # an item's JSON, file contents included
leptos-aria diff            # which installed components differ from the registry
leptos-aria diff button     # a unified diff of one component
```

Use `diff` after upgrading to see what changed upstream, then copy over the parts you want, or run `add <name> --overwrite` to take all of it.

## leptos-aria.json

```json
{
	"$schema": "https://ifiokjr.github.io/leptos_aria/schema.json",
	"style": "default",
	"tailwind": { "css": "style/tailwind.css" },
	"paths": { "ui": "src/components/ui" },
	"registries": {
		"@acme": "https://acme.dev/r/{name}.json"
	}
}
```

- **`tailwind.css`:** the CSS file that imports Tailwind. Theme variables and component CSS are written here.
- **`paths.ui`:** where component files land. A path under `src/` is wired into your crate automatically.
- **`registries`:** extra registries by namespace. `{name}` is replaced with the item name, so `leptos-aria add @acme/card` reads `https://acme.dev/r/card.json`. The template can also be a local path.

## Theming

Components use semantic colours (`bg-primary`, `text-muted-foreground`, `border-input`, `ring-ring`) backed by CSS variables, exactly like shadcn/ui. Re-theme everything by editing the variables in the `/* leptos-aria: default */` block of your CSS file:

```css
:root {
	--primary: oklch(0.55 0.22 30);
	--primary-foreground: oklch(0.98 0 0);
	--radius: 0.5rem;
}
```

The variable names match shadcn/ui's, so its published themes paste straight in.

## Host your own registry

A registry is a folder of JSON files. Item files use shadcn's [`registry-item.json` schema](https://ui.shadcn.com/schema/registry-item.json), so registry tooling that understands shadcn understands yours. Describe your items in a `registry.json`:

```json
{
	"$schema": "https://ui.shadcn.com/schema/registry.json",
	"name": "acme",
	"items": [
		{
			"name": "card",
			"type": "registry:ui",
			"title": "Card",
			"description": "Our card",
			"dependencies": ["leptos_aria"],
			"registryDependencies": ["utils"],
			"files": [{ "path": "src/ui/card.rs", "type": "registry:ui" }]
		}
	]
}
```

Then build it and serve the output from any static host:

```bash
leptos-aria build registry.json --output public/r
```

`build` validates names and dependencies and inlines every file's contents. It writes `public/r/<name>.json` for each item and `public/r/registry.json` for `list`.

- **Bare dependencies:** names in `registryDependencies` refer to the official registry, so your items can build on `utils` or `button`.
- **Install locations:** a file's `target` places it explicitly. `@ui/` is the component directory and `~/` is the project root.

Set `LEPTOS_ARIA_REGISTRY_URL` to point the default registry somewhere else, such as a local build while you develop:

```bash
LEPTOS_ARIA_REGISTRY_URL=./public/r/{name}.json leptos-aria add card
```
