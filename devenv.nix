{ pkgs, ... }:

{
  packages = [ 
    pkgs.cargo-all-features
    pkgs.cargo-generate
    pkgs.cargo-insta
    pkgs.cargo-make
    pkgs.cargo-workspaces
    pkgs.deno
    pkgs.dprint
    pkgs.fnm
    pkgs.git
    pkgs.ripgrep
    pkgs.rust-analyzer
    pkgs.rustup
    pkgs.trunk
  ];

  difftastic.enable = true;
  devcontainer.enable = true;


  # Scripts

  scripts."build:all".exec = ''
    cargo build
  '';
  scripts."fix:all".exec = ''
    fix:format
    fix:clippy
  '';
  scripts."fix:format".exec = ''
    dprint fmt
  '';
  scripts."fix:clippy".exec = ''
    cargo clippy --fix --allow-dirty --allow-staged
  '';
  scripts."lint:all".exec = ''
    lint:format
    lint:clippy
  '';
  scripts."lint:format".exec = ''
    dprint check
  '';
  scripts."lint:clippy".exec = ''
    cargo clippy
  '';
  scripts."test:snapshot".exec = ''
    cargo insta accept
  '';
  scripts."test:all".exec = ''
    cargo test
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
