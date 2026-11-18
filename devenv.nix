{ pkgs, ... }:

{
  packages = [
    pkgs.actionlint
    pkgs.binaryen
    pkgs.cargo-make
    pkgs.cargo-nextest
    pkgs.chromedriver
    pkgs.dprint
    pkgs.geckodriver
    pkgs.git
    pkgs.nixfmt
    pkgs.nodejs_24
    pkgs.ripgrep
    pkgs.rustup
    pkgs.tailwindcss_4
    pkgs.taplo
    pkgs.trunk
    pkgs.wasm-bindgen-cli
    pkgs.wasm-pack
    pkgs.zizmor
  ];

  difftastic.enable = true;
  devcontainer.enable = true;

  # Scripts

  scripts."build:all".exec = ''
    cargo build --target wasm32-unknown-unknown
  '';
  scripts."fix:all".exec = ''
    fix:format
    fix:clippy
  '';
  scripts."fix:format".exec = ''
    dprint fmt
  '';
  scripts."fix:clippy".exec = ''
    cargo clippy --fix --allow-dirty --allow-staged --all-targets
  '';
  scripts."lint:all".exec = ''
    lint:format
    lint:clippy
  '';
  scripts."lint:format".exec = ''
    dprint check
  '';
  scripts."lint:clippy".exec = ''
    cargo clippy --all-targets -- -D warnings
  '';
  scripts."test:all".exec = ''
    cargo test
    test:wasm
  '';
  scripts."test:wasm".exec = ''
    cargo test --target wasm32-unknown-unknown
  '';
  scripts."test:e2e".exec = ''
    set -e
    # Trunk's boolean CLI flag does not accept the conventional NO_COLOR=1.
    env -u NO_COLOR site:build
    npm --prefix e2e ci --ignore-scripts
    cd e2e
    npm run check
    npx playwright install chromium firefox webkit
    npm test -- "$@"
  '';
  # Build the static site, then the live component previews into its
  # `preview/` directory. Pass an output directory to build elsewhere.
  scripts."site:build".exec = ''
    set -e
    out="''${1:-target/site}"
    mkdir -p "$out"
    out="$(cd "$out" && pwd)"
    cargo run --locked -p leptos_aria_docs --bin site -- "$out"
    trunk build --release --locked --offline --config crates/leptos_aria_preview/Trunk.toml --dist "$out/preview"
  '';
  scripts."setup:helix".exec = ''
    rm -rf .helix
    cp -r setup/editors/helix .helix
  '';
  scripts."setup:vscode".exec = ''
    rm -rf .vscode
    cp -r ./setup/editors/vscode .vscode
  '';
}
