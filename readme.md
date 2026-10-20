<p align="center">
	<a href="https://ifiokjr.github.io/leptos_aria/"><img src="website/public/logo.svg" alt="leptos_aria" width="96" height="96" /></a>
</p>

# leptos_aria

> Accessible by default. Styled by you.

`leptos_aria` ports the [React Aria](https://react-spectrum.adobe.com/react-aria/) ecosystem to [Leptos](https://leptos.dev). It brings React Aria's keyboard, focus, and screen-reader behaviour and ships no CSS: every state is exposed as a `data-*` attribute, so Tailwind v4 styles it with the classes you already write.

[Website](https://ifiokjr.github.io/leptos_aria/) · [Tailwind guide](https://ifiokjr.github.io/leptos_aria/tailwind.html) · [Porting roadmap](PORTING.md) · [API docs](https://docs.rs/leptos_aria)

## Install

```bash
cargo add leptos_aria
```

The `leptos_aria` crate re-exports every component, hook, and utility. Each component family also ships as its own `leptos_aria_*` crate if you want a smaller dependency.

## Style with Tailwind v4

Components are headless. Pass classes with each component's `class` prop and target state with Tailwind's built-in `data-[...]` and `aria-*` variants:

```rust
view! {
	<AccordionItem key="faq".to_string()>
		<AccordionTrigger class="data-[state=open]:font-bold">"Is it accessible?"</AccordionTrigger>
		<AccordionPanel>"By default."</AccordionPanel>
	</AccordionItem>
}
```

See [docs/tailwind.md](docs/tailwind.md) for the full attribute contract and `@custom-variant` recipes.

## Copy styled components with the CLI

```bash
cargo install leptos_aria_cli
leptos-aria init               # leptos-aria.json, theme CSS variables, the `cn` helper
leptos-aria add button dialog  # copy components into src/components/ui
leptos-aria diff               # compare your copies with the registry
```

The same workflow as shadcn/ui: copied components are yours to edit, with Tailwind classes on top and the headless crates underneath. Browse the [components](https://ifiokjr.github.io/leptos_aria/components.html) or read the [CLI guide](docs/cli.md), which also covers theming and hosting your own registry. The registry's source is the [`registry`](registry) crate, compiled and tested in CI so every component builds exactly as you receive it.

## Contributing

[`devenv`](https://devenv.sh/) provides a reproducible development environment. Follow its [getting started guide](https://devenv.sh/getting-started/), then [install direnv](https://devenv.sh/automatic-shell-activation/) and allow this repository's `.envrc`:

```bash
direnv allow .
```

Editor configuration for VS Code and Helix is available with `setup:vscode` and `setup:helix`.

These scripts are available inside the devenv shell:

```bash
build:all   # build every crate for the wasm32 target
fix:all     # run dprint fmt and cargo clippy --fix
lint:all    # run dprint check and cargo clippy
test:all    # run the native and wasm32 browser test suites
test:wasm   # run the browser test suite only
site:build  # render the website into target/site
```

Browser tests run through `wasm-bindgen-test-runner` with a local Chrome driven by `chromedriver`; both come from the devenv shell. The `wasm-bindgen` dependency is pinned to the `wasm-bindgen-cli` version in the devenv lock so the runner and the crate always match.

### The website

The site at [ifiokjr.github.io/leptos_aria](https://ifiokjr.github.io/leptos_aria/) is built by the `site` binary in [`crates/leptos_aria_docs`](crates/leptos_aria_docs). It renders [`website/layout.html`](website/layout.html) around the landing page and the repository's own markdown (this readme, [docs/tailwind.md](docs/tailwind.md), and [PORTING.md](PORTING.md)), so the site, `llms-full.txt`, and the docs never drift. Static assets, including the logo, live in [`website/public`](website/public).

## License

[Unlicense](license): public domain.
