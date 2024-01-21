{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs = inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" "aarch64-linux" ];

      perSystem = { self', pkgs, system, ... }: {
        _module.args.pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = with inputs; [
            rust-overlay.overlays.default
          ];
        };

        devShells = {
          default = self'.devShells.podman;

          podman =
            let
              rust-toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
            in
            pkgs.mkShell {
              packages = with pkgs; [
                git
                podman
                fuse3
                qemu
                gnumake
                wget
              ] ++ [ rust-toolchain ];
            };
        };
      };
    };
}
