{
  description = "Rust dev shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        overrides = builtins.fromTOML (builtins.readFile ./rust-toolchain.toml);
      in
      {
        devShells.default = pkgs.callPackage (
          {
            pkgs,
            stdenv,
            mkShell,
            rustup,
            rustPlatform,
          }:
          mkShell {
            strictDeps = true;
            nativeBuildInputs = [
              rustup
              rustPlatform.bindgenHook
            ];
            # libraries here
            buildInputs = [
              pkgs.cmake
            ];
            RUSTC_VERSION = overrides.toolchain.channel;
            shellHook = ''
              export PATH="''${CARGO_HOME:-~/.cargo}/bin":"$PATH"
              export PATH="''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-${stdenv.hostPlatform.rust.rustcTarget}/bin":"$PATH"
            '';
          }
        ) { };
      }
    );
}
