{
  description = "Dim (or brighten) your screen";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustPlatform = pkgs.makeRustPlatform {
          cargo = pkgs.rust-bin.stable.latest.default;
          rustc = pkgs.rust-bin.stable.latest.default;
        };

        dim = rustPlatform.buildRustPackage {
          name = "dim";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
        };
      in
      with pkgs;
      {
        devShells.default = mkShell {
          buildInputs = [
            rust-bin.stable.latest.default
          ];
          shellHook = ''
            user_shell=$(getent passwd "$(whoami)" |cut -d: -f 7)
            exec "$user_shell"
          '';
        };

        packages = {
          default = dim;
        };
      }
    );
}
