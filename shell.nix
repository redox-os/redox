with import <nixpkgs> {};
stdenv.mkDerivation {
  name = "redox";

  hardeningDisable = [ "all" ];

  nativeBuildInputs = [ gnumake cmake nasm pkgconfig gcc automake autoconf bison gperf qemu ];
  buildInputs = [ openssl gettext libtool flex libpng perl perlPackages.HTMLParser ];
}
