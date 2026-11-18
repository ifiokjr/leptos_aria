# Changelog

All notable changes to this project will be documented in this file.

This changelog is managed by [monochange](https://github.com/monochange/monochange).

## [0.1.2](https://github.com/ifiokjr/leptos_aria/releases/tag/v0.1.2) (2026-10-23)

Grouped release for `main`.

### Features

- **Browser accessibility regression coverage.** Production demos and documentation now run through Playwright in Chromium, Firefox, WebKit, dark mode, and a mobile viewport, with WCAG audits and keyboard journeys required by CI. Space activation no longer panics when the press ends on a document listener. Tooltips retain working event handlers, describe their focused control, dismiss on Escape, and cancel stale hover opens. Accordions link their triggers and panels; selects link their combobox and listbox; menus expose menu/menuitem semantics through the new `ListBoxRole` prop instead of nested listbox roles. Radio navigation wraps in WebKit and skips disabled options. Decorative separators and modal backdrops omit unsupported ARIA, destructive examples keep sufficient text contrast, and scrollable documentation code, install commands, and tables are keyboard reachable.
  _Owner:_ [@ifiokjr](https://github.com/ifiokjr) · _Review:_ [PR #55](https://github.com/ifiokjr/leptos_aria/pull/55)

### Fixes

- **Prebuilt CLI binaries on every release.** Every GitHub release now ships `leptos-aria` archives for Linux (glibc and musl), macOS, and Windows on both x86_64 and aarch64, each with SHA-256 and SHA-512 checksums and a build provenance attestation verifiable with `gh attestation verify`. `cargo binstall leptos_aria_cli` downloads the matching archive instead of compiling. The release is published only after every archive is attached and attested and the crates are on crates.io, so a release that is visible is always installable. Release notes now list what changed instead of the release step's name, and release pull requests keep every `Cargo.lock` in step with the bumped versions.
  _Owner:_ [@ifiokjr](https://github.com/ifiokjr) · _Review:_ [PR #52](https://github.com/ifiokjr/leptos_aria/pull/52)
- **No font flash on the website's first paint.** The website self-hosts Bricolage Grotesque and Fragment Mono (latin and latin-ext woff2 subsets, with their SIL OFL 1.1 licences) and preloads both from every page's head. First paint uses the brand fonts instead of flashing the system stack while the Google Fonts stylesheet and files arrive, and the third-party font round trips are gone.
  _Owner:_ [@ifiokjr](https://github.com/ifiokjr) · _Review:_ [PR #50](https://github.com/ifiokjr/leptos_aria/pull/50)
- **Keyboard focus follows the selected tab.** Arrow keys, Home and End in a tab list moved the selection but left keyboard focus on the tab you started from, so the next key press went to the wrong tab and screen readers kept announcing the old one. Focus now moves with the selection, skips disabled tabs like the selection does, and a click focuses the tab it selects (Safari does not focus a button on click). `data-focused` used to mirror the selection; it now reports the tab that has focus, so style selection with `aria-selected` instead.
  _Owner:_ [@ifiokjr](https://github.com/ifiokjr) · _Review:_ [PR #53](https://github.com/ifiokjr/leptos_aria/pull/53)
