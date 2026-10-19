<p align="center">
	<a href="https://ifiokjr.github.io/leptos_aria/"><img src="https://ifiokjr.github.io/leptos_aria/logo.svg" alt="leptos_aria" width="96" height="96" /></a>
</p>

# leptos_aria

> Accessible by default. Styled by you.

`leptos_aria` ports the [React Aria](https://react-spectrum.adobe.com/react-aria/) ecosystem to [Leptos](https://leptos.dev). It brings React Aria's keyboard, focus, and screen-reader behaviour and ships no CSS: every state is exposed as a `data-*` attribute, so Tailwind v4 styles it with the classes you already write.

```bash
cargo add leptos_aria
```

This facade crate re-exports every component, hook, and utility from the `leptos_aria_*` crates: buttons and toggles, form fields, collections and menus, overlays, date and colour pickers, and the press, hover, focus, and keyboard interaction hooks underneath them.

[Website](https://ifiokjr.github.io/leptos_aria/) · [Tailwind guide](https://ifiokjr.github.io/leptos_aria/tailwind.html) · [Porting roadmap](https://ifiokjr.github.io/leptos_aria/roadmap.html) · [API docs](https://docs.rs/leptos_aria) · [Repository](https://github.com/ifiokjr/leptos_aria)
