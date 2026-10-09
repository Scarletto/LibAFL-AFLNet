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
            stdenv,
            mkShell,
            rustup,
            rustPlatform,
            cmake,
            openssl,
            pkg-config
          }:
          mkShell {
            strictDeps = true;
            nativeBuildInputs = [
              rustup
              rustPlatform.bindgenHook
              cmake
              openssl
              pkg-config
            ];
            # libraries here
            buildInputs = [ ];
            RUSTC_VERSION = overrides.toolchain.channel;
            shellHook = ''
              export PATH="''${CARGO_HOME:-$HOME/.cargo}/bin":"$PATH"
              export PATH="''${RUSTUP_HOME:-$HOME/.rustup}/toolchains/$RUSTC_VERSION-${stdenv.hostPlatform.rust.rustcTarget}/bin":"$PATH"
              export PATH="''${PKG_CONFIG_PATH:-${pkgs.openssl.dev}/lib/pkgconfig}":"$PATH"
              export PATH="''${OPENSSL_DIR:-${pkgs.openssl}}":"$PATH"

            '';
          }
        ) { };
      }
    );
}
