# Podman needs to be installed and configured on the system for all this to work:
# https://nixos.wiki/wiki/Podman
{
  description = "The Nix-flake for Redox development on NixOS";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };

    crane = {
      url = "github:ipetkov/crane";
    };

    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
    crane,
  } @ inputs:
    flake-utils.lib.eachSystem ["x86_64-linux" "aarch64-linux"] (system: let
      overlays = [(import rust-overlay)];
      pkgs = import nixpkgs {
        inherit system overlays;
      };
      inherit (pkgs) lib;

      rust-bin =
        pkgs.rust-bin.nightly."2025-01-12".default.override
        {
          extensions = ["rust-analyzer" "rust-src"];
          targets = ["x86_64-unknown-redox"];
        };
      craneLib = (inputs.crane.mkLib pkgs).overrideToolchain rust-bin;

      # Podman config taken from https://nixos.wiki/wiki/Podman and https://gist.github.com/adisbladis/187204cb772800489ee3dac4acdd9947
      # Provides a script that copies required files to ~/
      podmanSetupScript = let
        registriesConf = pkgs.writeText "registries.conf" ''
          [registries.search]
          registries = ['docker.io']
          [registries.block]
          registries = []
        '';
      in
        pkgs.writeScript "podman-setup" ''
          #!${pkgs.runtimeShell}
          # Dont overwrite customised configuration
          if ! test -f ~/.config/containers/policy.json; then
            install -Dm555 ${pkgs.skopeo.src}/default-policy.json ~/.config/containers/policy.json
          fi
          if ! test -f ~/.config/containers/registries.conf; then
            install -Dm555 ${registriesConf} ~/.config/containers/registries.conf
          fi
          systemctl --user start podman.socket || true
          export PODMAN_SYSTEMD_UNIT=podman.socket
        '';

      # Provides a fake "docker" binary mapping to podman
      dockerCompat = pkgs.runCommandNoCC "docker-podman-compat" {} ''
        mkdir -p $out/bin
        ln -s ${pkgs.podman}/bin/podman $out/bin/docker
      '';

      cargo-config =
        craneLib.buildPackage
        {
          pname = "cargo-config";
          version = "0.1.1";
          cargoArtifacts = null;

          src = pkgs.fetchFromGitHub {
            owner = "wesleywiser";
            repo = "cargo-config";
            rev = "cf576faf65913615ed424914daa960800ed3ebc4";
            sha256 = "sha256-HrITNTfjBppOH1MhfZHfzHc6N8ymcm7vaiBI94ctUOA=";
            fetchSubmodules = true;
          };
        };

      buildInputs = with pkgs; [
        # Compilation
        rust-bin

        # Utils
        cowsay
        lolcat
        tokio-console

        # Cargo utilities
        bacon
        cargo-bloat # check binaries size (which is fun but not terriby useful?)
        cargo-cache # cargo cache -a
        cargo-config
        cargo-expand # for macro expension
        cargo-spellcheck # Spellcheck documentation

        # Podman
        dockerCompat
        podman # Docker compat
        runc # Container runtime
        conmon # Container runtime monitor
        skopeo # Interact with container registry
        slirp4netns # User-mode networking for unprivileged namespaces
        fuse-overlayfs # CoW for images, much faster than default vfs

        # Build Redox
        ant
        autoconf
        automake
        bison
        cmake
        curl
        doxygen
        expat
        expect
        file
        flex
        fuse
        gmp
        gnumake
        gnupatch
        gperf
        just
        libjpeg
        libpng
        libtool
        llvmPackages.clang
        llvmPackages.llvm
        lua
        m4
        meson
        nasm
        perl
        perl540Packages.HTMLParser
        perl540Packages.Po4a
        pkgconf
        podman
        protobuf
        (python3.withPackages (ps: with ps; [mako]))
        qemu_kvm
        rust-cbindgen
        scons
        SDL
        syslinux
        texinfo
        unzip
        waf
        wget
        xdg-utils
        zip
      ];
    in {
      devShells.default = with pkgs;
        mkShell {
          inherit buildInputs;

          LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
          NIX_SHELL_BUILD = "1";
          shellHook = ''
            # Install required configuration
            ${podmanSetupScript}
            echo "Redox environment loaded" | cowsay | lolcat
          '';
        };
    });
}
