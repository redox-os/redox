{
  description = "Redox";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.11";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = inputs @ {
    nixpkgs,
    rust-overlay,
    ...
  }: let
    systems = [
      "x86_64-linux"
      "aarch64-linux"
    ];
    forAllSystems = nixpkgs.lib.genAttrs systems;

    nixpkgsFor = system:
      import nixpkgs {
        inherit system;
        overlays = [
          (import rust-overlay)
        ];
      };
  in {
    devShells = forAllSystems (system: let
      pkgs = nixpkgsFor system;
    in rec {
      default = redox;
      redox = pkgs.mkShell {
        NIX_SHELL_BUILD = "1";
        shellHook = '''';
        buildInputs = with pkgs; [
          cmake
          fuse
          gperf
          perl
          perl538Packages.HTMLParser
          perl538Packages.Po4a
          nasm
          wget
          texinfo
          bison
          flex
          autoconf
          automake
          curl
          file
          gnupatch
          gnumake
          scons
          waf
          expat
          gmp
          libtool
          libpng
          libjpeg
          SDL
          m4
          pkgconf
          syslinux
          meson
          (python3.withPackages (ps: with ps; [mako]))
          xdg-utils
          zip
          unzip
          doxygen
          lua
          ant
          protobuf
          llvmPackages_17.clang
          llvmPackages_17.llvm

          qemu_kvm

          (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
        ];
      };
    });
  };
}
