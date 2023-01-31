# leptos_aria

> A port of the `react-aria` ecosystem for the leptos framework.

## Motivation

[`leptos`](https://github.com/leptos-rs/leptos) is an amazing rust web framework. It is still in its
infancy and needs to better accessibility support.

This is an attempt to provide a port of the `react-aria` ecosystem to `leptos`.

## Contributing

[`devenv`](https://devenv.sh/) is used to provide a reproducible development environment for this
project. Follow the [getting started instructions](https://devenv.sh/getting-started/).

To automatically load the environment you should
[install direnv](https://devenv.sh/automatic-shell-activation/) and then load the `direnv`.

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
