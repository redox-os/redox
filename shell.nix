{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell rec {
  hardeningDisable = [ "all" ];

  # used in mk/prefix.mk to patch interpreter when PREFIX_BINARY=1
  NIX_INTERPRETER = "${pkgs.stdenv.cc.libc}/lib/ld-linux-x86-64.so.2";

  LIBRARY_PATH = pkgs.lib.makeLibraryPath [
    pkgs.gcc-unwrapped pkgs.stdenv.cc.libc
    (toString prefix/x86_64-unknown-redox)
  ];
  LD_LIBRARY_PATH = LIBRARY_PATH;

  nativeBuildInputs = with pkgs; [ gnumake cmake nasm pkgconfig gcc automake autoconf bison gperf qemu rustup ];
  buildInputs = with pkgs; [ fuse openssl gettext libtool flex libpng perl perlPackages.HTMLParser ];

  shellHook = ''
    export PATH="/run/wrappers/bin:$PATH"
  '';
}
