---
main: minor
---

# Opt-in `nightly` feature

Every crate that depends on `leptos` now exposes a `nightly` cargo feature that forwards to `leptos`'s own `nightly` feature, unlocking nightly-only signal and renderer optimizations (signals callable as functions, faster view codegen). The facade's feature enables it across all member crates at once: `leptos_aria = { version = "0.1", features = ["nightly"] }`. The feature is additive and a no-op on stable toolchains, and CI now lints the workspace with it enabled. The registry components also wrap their derived class closures in `Signal::derive` explicitly, so the components you copy with the CLI compile unchanged on nightly toolchains.
