#!/usr/bin/env bash
# Bootstrap a Cursor cloud agent for leptos_aria.
#
# Idempotent. Installs the toolchain pinned in rust-toolchain.toml, the
# wasm-bindgen CLI that matches the workspace pin, the formatters dprint
# runs, and a ChromeDriver that matches the image's Chrome. Then fetches
# both locked dependency graphs.
#
# The library build, lint, and test commands this prepares:
#   cargo build --target wasm32-unknown-unknown --locked
#   cargo fmt --all -- --check
#   dprint check
#   cargo clippy --all-targets --locked -- -D warnings
#   cargo clippy --manifest-path crates/leptos_aria_cli/Cargo.toml --all-targets --locked -- -D warnings
#   cargo clippy --all-targets --locked --features leptos_aria/nightly -- -D warnings
#   cargo test --locked
#   cargo test --manifest-path crates/leptos_aria_cli/Cargo.toml --locked
#   cargo test --target wasm32-unknown-unknown --locked
#
# site:build and test:e2e stay on the devenv shell. They need trunk,
# Tailwind v4, Node 24, and Playwright's browsers, which this script does
# not install.
set -euo pipefail

cd "$(dirname "$(readlink -f "$0")")/.."

TOOLCHAIN="nightly-2026-09-07"
WASM_BINDGEN_VERSION="0.2.129"
DPRINT_VERSION="0.61.1"
NIXFMT_VERSION="1.5.0"

if ! command -v rustup >/dev/null 2>&1; then
	curl --proto '=https' --tlsv1.2 -fsSL https://sh.rustup.rs \
		| sh -s -- -y --default-toolchain none --profile minimal
	for tool in rustup cargo rustc rustdoc rustfmt cargo-fmt cargo-clippy clippy-driver; do
		sudo ln -sfn "${HOME}/.cargo/bin/${tool}" "/usr/local/bin/${tool}"
	done
fi

rustup toolchain install "${TOOLCHAIN}" --profile default \
	--component cargo \
	--component clippy \
	--component rustfmt
rustup target add wasm32-unknown-unknown --toolchain "${TOOLCHAIN}"

install_file() {
	local url="$1"
	local dest="$2"
	local tmp

	tmp="$(mktemp)"
	curl -fsSL -L "${url}" -o "${tmp}"
	sudo install -m 0755 "${tmp}" "${dest}"
	rm -f "${tmp}"
}

if ! wasm-bindgen --version 2>/dev/null | grep -q "${WASM_BINDGEN_VERSION}"; then
	archive="$(mktemp -d)"
	curl -fsSL -L \
		"https://github.com/rustwasm/wasm-bindgen/releases/download/${WASM_BINDGEN_VERSION}/wasm-bindgen-${WASM_BINDGEN_VERSION}-x86_64-unknown-linux-musl.tar.gz" \
		| tar -xz -C "${archive}"
	sudo install -m 0755 "${archive}/wasm-bindgen-${WASM_BINDGEN_VERSION}-x86_64-unknown-linux-musl/wasm-bindgen" /usr/local/bin/wasm-bindgen
	sudo install -m 0755 "${archive}/wasm-bindgen-${WASM_BINDGEN_VERSION}-x86_64-unknown-linux-musl/wasm-bindgen-test-runner" /usr/local/bin/wasm-bindgen-test-runner
	rm -rf "${archive}"
fi

if ! dprint --version 2>/dev/null | grep -q "${DPRINT_VERSION}"; then
	archive="$(mktemp -d)"
	curl -fsSL -L \
		"https://github.com/dprint/dprint/releases/download/${DPRINT_VERSION}/dprint-x86_64-unknown-linux-gnu.zip" \
		-o "${archive}/dprint.zip"
	unzip -q -o "${archive}/dprint.zip" -d "${archive}"
	sudo install -m 0755 "${archive}/dprint" /usr/local/bin/dprint
	rm -rf "${archive}"
fi

if ! nixfmt --version 2>/dev/null | grep -q "${NIXFMT_VERSION}"; then
	install_file \
		"https://github.com/NixOS/nixfmt/releases/download/v${NIXFMT_VERSION}/nixfmt" \
		/usr/local/bin/nixfmt
fi

if ! command -v google-chrome >/dev/null 2>&1; then
	echo "google-chrome is required for the wasm32 browser tests" >&2
	exit 1
fi

chrome_version="$(google-chrome --version | grep -oE '[0-9]+(\.[0-9]+){3}' | head -n 1)"
if ! chromedriver --version 2>/dev/null | grep -q "${chrome_version}"; then
	archive="$(mktemp -d)"
	curl -fsSL -L \
		"https://storage.googleapis.com/chrome-for-testing-public/${chrome_version}/linux64/chromedriver-linux64.zip" \
		-o "${archive}/chromedriver.zip"
	unzip -q -o "${archive}/chromedriver.zip" -d "${archive}"
	sudo install -m 0755 "${archive}/chromedriver-linux64/chromedriver" /usr/local/bin/chromedriver
	rm -rf "${archive}"
fi

# Cache dprint's wasm plugins so a later `dprint check` does not need to
# download them again.
dprint output-file-paths >/dev/null

cargo fetch --locked
cargo fetch --locked --manifest-path crates/leptos_aria_cli/Cargo.toml
