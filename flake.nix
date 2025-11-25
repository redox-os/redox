{
  description = "The Nix-flake for Redox development on NixOS";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{
      nixpkgs,
      flake-parts,
      rust-overlay,
      ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } (
      top@{
        config,
        withSystem,
        moduleWithSystem,
        ...
      }:
      {
        systems = [
          "i686-linux"
          "x86_64-linux"
          "aarch64-linux"
          "x86_64-darwin"
          "aarch64-darwin"
        ];
        perSystem =
          {
            system,
            lib,
            inputs',
            ...
          }:
          let
            pkgs = import nixpkgs {
              inherit system;

              overlays = [ rust-overlay.overlays.default ];
            };
            rust-bin = pkgs.rust-bin.nightly."2025-10-03".default.override {
              extensions = [
                "rust-analyzer"
                "rust-src"
              ];
              targets = [ "x86_64-unknown-redox" ];
            };
          in
          {
            formatter = pkgs.nixfmt-rfc-style;

            # TODO: Create Redox OS Image as package
            # TODO: No cross-compile for now, as there is no pkgsCross.aarch64-unknown-redox and so on
            # TODO: Get rid of make env step: package custom libtool and setup rust toolchain properly
            devShells = {
              # Podman config taken from https://nixos.wiki/wiki/Podman and https://gist.github.com/adisbladis/187204cb772800489ee3dac4acdd9947
              # Provides a script that copies required files to ~/
              default =
                let
                  rustPlatform = pkgs.makeRustPlatform {
                    cargo = rust-bin;
                    rustc = rust-bin;
                  };

                  podmanSetupScript =
                    let
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
                  dockerCompat = pkgs.runCommand "docker-podman-compat" { } ''
                    mkdir -p $out/bin
                    ln -s ${pkgs.podman}/bin/podman $out/bin/docker
                  '';

                in
                pkgs.mkShell rec {
                  buildInputs = with pkgs; [
                    # Podman
                    dockerCompat
                    podman # Docker compat
                    runc # Container runtime
                    conmon # Container runtime monitor
                    skopeo # Interact with container registry
                    slirp4netns # User-mode networking for unprivileged namespaces
                    fuse-overlayfs # CoW for images, much faster than default vfs

                    # with FSTOOLS_IN_PODMAN=1 these are not needed
                    # without it, the installer fails to link FUSE somehow
                    #fuse
                    #rust-bin
                    qemu_kvm
                  ];

                  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
                  NIX_SHELL_BUILD = "1";
                  FSTOOLS_IN_PODMAN = "1";
                  shellHook = ''
                    # Install required configuration
                    ${podmanSetupScript}
                    echo "Redox podman build environment loaded"
                  '';
                };

              #TODO: This isn't tested yet, use at your own risk
              native = pkgs.mkShell rec {
                nativeBuildInputs =
                  let
                    autoreconf269 = pkgs.writeShellScriptBin "autoreconf2.69" "${pkgs.autoconf269}/bin/autoreconf";
                  in
                  with pkgs;
                  [
                    ant
                    autoconf
                    autoreconf269 # gnu-binutils
                    automake
                    bison
                    cmake
                    curl
                    doxygen
                    file
                    flex
                    gettext
                    gnumake
                    gnupatch
                    gperf
                    help2man
                    just
                    llvmPackages.clang
                    llvmPackages.llvm
                    lua
                    m4
                    meson
                    nasm
                    ninja
                    perl
                    perl540Packages.HTMLParser
                    perl540Packages.Po4a
                    pkg-config
                    pkgconf
                    (python3.withPackages (ps: with ps; [ mako ]))
                    qemu_kvm
                    rust-cbindgen
                    scons
                    texinfo
                    unzip
                    waf
                    wget
                    xdg-utils
                    xxd
                    zip
                  ] ++ pkgs.lib.optionals pkgs.stdenv.hostPlatform.isx86 [
                    pkgs.syslinux
                  ];

                buildInputs = with pkgs; [
                  rust-bin
                  fuse # fuser
                  libpng # netsurf
                  fontconfig # orbutils
                  SDL # prboom
                  xorg.utilmacros # libX11
                  xorg.xtrans # libX11
                ];

                LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
                PERL_PATH = "${pkgs.perl}/bin/perl";
                NIX_SHELL_BUILD = "1";
                PODMAN_BUILD = "0";
                shellHook = with pkgs; ''
                  export PKG_CONFIG_PATH="${fuse.dev}/lib/pkgconfig\
                  :${libpng.dev}/lib/pkgconfig"
                '';
              };
            };
          };
      }
    );
}
