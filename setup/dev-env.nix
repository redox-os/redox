let
  pkgs    = import <nixpkgs> {};
  stdenv  = pkgs.stdenv;
  lib     = pkgs.lib;

in rec {
  devEnv = stdenv.mkDerivation rec {
    name = "redox-dev-env";
    src = ./.;
    buildInputs = with pkgs; [
      git
      rustUnstable.rustc
      gnumake
      qemu
      nasm
    ];
  };
}
