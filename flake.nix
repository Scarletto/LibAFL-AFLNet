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
            pkg-config,
            python314,
            llvmPackages
          }:
          mkShell {
            strictDeps = true;
            nativeBuildInputs = [
              rustup
              rustPlatform.bindgenHook
              cmake
              pkg-config
              python314
            ];
            # libraries here
            buildInputs = [
              openssl
              llvmPackages.compiler-rt.dev
            ];
            RUSTC_VERSION = overrides.toolchain.channel;
            shellHook = ''
                export CPATH="${llvmPackages.compiler-rt.dev}/include''${CPATH:+:$CPATH}"
                export BINDGEN_EXTRA_CLANG_ARGS="$BINDGEN_EXTRA_CLANG_ARGS -isystem ${llvmPackages.compiler-rt.dev}/include"
              export PATH="''${CARGO_HOME:-$HOME/.cargo}/bin":"$PATH"
              export PATH="''${RUSTUP_HOME:-$HOME/.rustup}/toolchains/$RUSTC_VERSION-${stdenv.hostPlatform.rust.rustcTarget}/bin":"$PATH"
            '';
          }
        ) { };
      }
    );
}
