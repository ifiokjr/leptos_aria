#!/usr/bin/env bash
# Bootstrap a Cursor cloud agent for leptos_aria.
#
# Idempotent. Installs single-user Nix (this image has no systemd, so the
# multi-user daemon is not available), enables flakes, installs the devenv
# and cachix pinned by CI, and trusts the devenv binary cache. Nix and
# devenv are placed on the default PATH for both login and non-login shells.
# The devenv shell is then evaluated and the repo's pinned Rust toolchain
# is installed, so later agents do not repeat that download.
#
# After this script, from the repository root:
#   nix --version
#   devenv version
#   devenv shell -- build:all
#   devenv shell -- lint:all
#   devenv shell -- test:all
set -euo pipefail

cd "$(dirname "$(readlink -f "$0")")/.."

# Same nixpkgs snapshot as .github/actions/devenv. Floating nixpkgs-unstable
# has shipped uninstallable stdenv bootstraps; this revision is devenv 2.2.2.
NIXPKGS_REV="0af3d1402dec3fc7e93635e511d1f7428c89cebf"
STAMP="${HOME}/.local/state/leptos-aria-cloud/devenv-nixpkgs-rev"

if ! command -v curl >/dev/null 2>&1 || ! command -v xz >/dev/null 2>&1; then
	sudo apt-get update
	sudo apt-get install -y --no-install-recommends ca-certificates curl xz-utils
fi

if [[ ! -x "${HOME}/.nix-profile/bin/nix" ]]; then
	conf="$(mktemp)"
	cat >"${conf}" <<'EOF'
extra-experimental-features = nix-command flakes
download-attempts = 5
EOF
	installer="$(mktemp)"
	curl -fsSL -o "${installer}" https://nixos.org/nix/install
	sh "${installer}" --no-daemon --yes --nix-extra-conf-file "${conf}"
	rm -f "${installer}" "${conf}"
fi

if [[ ! -f "${HOME}/.nix-profile/etc/profile.d/nix.sh" ]]; then
	echo "Nix installed but ${HOME}/.nix-profile/etc/profile.d/nix.sh is missing" >&2
	exit 1
fi

# shellcheck disable=SC1091
. "${HOME}/.nix-profile/etc/profile.d/nix.sh"

nix_conf="${XDG_CONFIG_HOME:-${HOME}/.config}/nix/nix.conf"
mkdir -p "$(dirname "${nix_conf}")"
touch "${nix_conf}"
if ! grep -q 'nix-command' "${nix_conf}"; then
	printf '\nextra-experimental-features = nix-command flakes\n' >>"${nix_conf}"
fi
if ! grep -q 'download-attempts' "${nix_conf}"; then
	printf 'download-attempts = 5\n' >>"${nix_conf}"
fi

install_profile() {
	local pkg="$1"
	local attempt

	for attempt in 1 2 3; do
		# Nix 2.35 deprecates `nix profile install` as an alias of `add`.
		if nix profile add --accept-flake-config "${pkg}"; then
			return 0
		fi
		if [[ "${attempt}" -eq 3 ]]; then
			echo "failed to install ${pkg}" >&2
			return 1
		fi
		sleep $((attempt * 10))
	done
}

installed_rev="$(cat "${STAMP}" 2>/dev/null || true)"
if [[ ! -x "${HOME}/.nix-profile/bin/cachix" || ! -x "${HOME}/.nix-profile/bin/devenv" || "${installed_rev}" != "${NIXPKGS_REV}" ]]; then
	if [[ ! -x "${HOME}/.nix-profile/bin/cachix" ]]; then
		install_profile "github:NixOS/nixpkgs/${NIXPKGS_REV}#cachix"
	fi
	# Public pull cache. devenv.cachix.org is also a default devenv pull, but
	# configuring it before the shell evaluation lets the CLI and the shell
	# substitute instead of building.
	if ! grep -q 'devenv.cachix.org' "${nix_conf}"; then
		cachix use devenv
	fi
	if [[ ! -x "${HOME}/.nix-profile/bin/devenv" || "${installed_rev}" != "${NIXPKGS_REV}" ]]; then
		install_profile "github:NixOS/nixpkgs/${NIXPKGS_REV}#devenv"
	fi
	mkdir -p "$(dirname "${STAMP}")"
	printf '%s\n' "${NIXPKGS_REV}" >"${STAMP}"
elif ! grep -q 'devenv.cachix.org' "${nix_conf}"; then
	cachix use devenv
fi

# devenv's chromedriver is built from the locked nixpkgs and does not match
# the image's Chrome. wasm-bindgen-test-runner uses $CHROMEDRIVER when it is
# set, so install the driver that matches google-chrome and export it.
CHROMEDRIVER_BIN="/usr/local/lib/cloud-agent/chromedriver"
if command -v google-chrome >/dev/null 2>&1; then
	chrome_version="$(google-chrome --version | grep -oE '[0-9]+(\.[0-9]+){3}' | head -n 1)"
	if ! "${CHROMEDRIVER_BIN}" --version 2>/dev/null | grep -q "${chrome_version}"; then
		archive="$(mktemp -d)"
		curl -fsSL -L \
			"https://storage.googleapis.com/chrome-for-testing-public/${chrome_version}/linux64/chromedriver-linux64.zip" \
			-o "${archive}/chromedriver.zip"
		unzip -q -o "${archive}/chromedriver.zip" -d "${archive}"
		sudo mkdir -p "$(dirname "${CHROMEDRIVER_BIN}")"
		sudo install -m 0755 "${archive}/chromedriver-linux64/chromedriver" "${CHROMEDRIVER_BIN}"
		rm -rf "${archive}"
	fi
fi

# Login shells read /etc/profile.d. Non-login shells do not, so wrap the
# tools onto the default PATH and source nix.sh inside the wrapper.
sudo tee /etc/profile.d/nix-devenv.sh >/dev/null <<'EOF'
if [ -x /usr/local/lib/cloud-agent/chromedriver ]; then
	export CHROMEDRIVER=/usr/local/lib/cloud-agent/chromedriver
fi
if [ -f "${HOME}/.nix-profile/etc/profile.d/nix.sh" ]; then
	. "${HOME}/.nix-profile/etc/profile.d/nix.sh"
fi
EOF
sudo chmod 644 /etc/profile.d/nix-devenv.sh

install_wrapper() {
	local tool="$1"
	local target="${HOME}/.nix-profile/bin/${tool}"

	if [[ ! -x "${target}" ]]; then
		return 0
	fi
	local wrapper
	wrapper="$(mktemp)"
	cat >"${wrapper}" <<EOF
#!/bin/sh
if [ -f "\${HOME}/.nix-profile/etc/profile.d/nix.sh" ]; then
	. "\${HOME}/.nix-profile/etc/profile.d/nix.sh"
fi
if [ "${tool}" = "devenv" ] && [ -z "\${CHROMEDRIVER:-}" ] && [ -x /usr/local/lib/cloud-agent/chromedriver ]; then
	export CHROMEDRIVER=/usr/local/lib/cloud-agent/chromedriver
fi
exec "${target}" "\$@"
EOF
	sudo install -m 0755 "${wrapper}" "/usr/local/bin/${tool}"
	rm -f "${wrapper}"
}

for tool in nix nix-build nix-channel nix-collect-garbage nix-env nix-instantiate nix-shell nix-store cachix devenv; do
	install_wrapper "${tool}"
done

# shellcheck disable=SC1091
. "${HOME}/.nix-profile/etc/profile.d/nix.sh"

for attempt in 1 2 3; do
	if devenv shell -- true; then
		break
	fi
	if [[ "${attempt}" -eq 3 ]]; then
		echo "devenv shell failed after ${attempt} attempts" >&2
		exit 1
	fi
	sleep $((attempt * 20))
done

toolchain="$(sed -n 's/^channel = "\(.*\)"/\1/p' rust-toolchain.toml)"
devenv shell -- rustup toolchain install "${toolchain}" --profile default \
	--component cargo \
	--component clippy \
	--component rustfmt
devenv shell -- rustup target add wasm32-unknown-unknown --toolchain "${toolchain}"
devenv shell -- cargo fetch --locked
devenv shell -- cargo fetch --locked --manifest-path crates/leptos_aria_cli/Cargo.toml
