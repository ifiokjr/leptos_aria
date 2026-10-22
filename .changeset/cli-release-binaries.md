---
main: patch
---

# Prebuilt CLI binaries on every release

Every GitHub release now ships `leptos-aria` archives for Linux (glibc and musl), macOS, and Windows on both x86_64 and aarch64, each with SHA-256 and SHA-512 checksums and a build provenance attestation verifiable with `gh attestation verify`. `cargo binstall leptos_aria_cli` downloads the matching archive instead of compiling. The release is published only after every archive is attached and attested and the crates are on crates.io, so a release that is visible is always installable. Release notes now list what changed instead of the release step's name, and release pull requests keep every `Cargo.lock` in step with the bumped versions.
