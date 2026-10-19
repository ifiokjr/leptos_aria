# leptos_aria

> A port of the `react-aria` ecosystem for the leptos framework.

## Motivation

[`leptos`](https://github.com/leptos-rs/leptos) is an amazing rust web framework. It is still in its infancy and needs to better accessibility support.

This is an attempt to provide a port of the `react-aria` ecosystem to `leptos`.

## Contributing

[`devenv`](https://devenv.sh/) is used to provide a reproducible development environment for this project. Follow the [getting started instructions](https://devenv.sh/getting-started/).

To automatically load the environment you should [install direnv](https://devenv.sh/automatic-shell-activation/) and then load the `direnv`.

```bash
# The security mechanism didn't allow to load the `.envrc`.
# Since we trust it, let's allow it execution.
direnv allow .
```

At this point you should see the `nix` commands available in your terminal.

To setup recommended configuration for your favourite editor run the following commands.

```bash
setup:vscode # Setup vscode
setup:helix  # Setup helix configuration
```

## Development

The following scripts are available inside the devenv shell.

```bash
build:all   # build every crate for the wasm32 target
fix:all     # run dprint fmt and cargo clippy --fix
lint:all    # run dprint check and cargo clippy
test:all    # run the native and wasm32 browser test suites
test:wasm   # run the browser test suite only
```

The browser tests run through `wasm-bindgen-test-runner` with a local Chrome driven by `chromedriver`; both come from the devenv shell. The `wasm-bindgen` dependency is pinned to the `wasm-bindgen-cli` version in the devenv lock so the runner and the crate always match.

## Tailwind CSS v4

Every component is headless — no styles ship, state is exposed as `data-*` attributes, and Tailwind v4's built-in `data-[...]`/`aria-*` variants style it with zero configuration. See [docs/tailwind.md](docs/tailwind.md) for the attribute contract and `@custom-variant` recipes.

## shadcn-style CLI

```
cargo install leptos_aria_cli
leptos-aria init          # write leptos-aria.json
leptos-aria add button    # copy a styled wrapper into src/aria
leptos-aria list          # name the registry
```

Copied components are yours: Tailwind classes in, headless crates underneath, exactly the shadcn workflow.
