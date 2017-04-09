with (import <nixpkgs> {});
with pkgs;

let rustFuns = callPackage ./rust-nightly-nix {};
    rust = rustFuns.rust {
        date = "2017-04-08";
        hash = "0zwn5a7j8pvixwxxwvnlw78wcchvdmd4ff856wdlvsxcw2h5gdxl";
}; in

stdenv.mkDerivation {
    name = "redox";
    buildInputs = [

        rust         # duh
        pkgconfig    # ?
        gcc          # compiler-builtins, libc-artifacts
        fuse procps  # TFS image creation
        nasm         # bootloader / raw disk image 


        # rather optional:

        cdrkit syslinux  # ISO image
        qemu

    ];

    # The CC wrapper enforces relro (which we could disable with hardeningDisable)
    # and dynamically-linked libc (which I couldn't figure out how to disable otherwise).
    # Luckily, we aren't particularily interested in being able to find NixOS'
    # system libraries so let's just use the raw one.
    GCC="${gcc.cc}/bin/gcc"; 

    # There's *got* to be a proper way to get at the setuid wrapper (that I don't know of)
    FUMOUNT="/var/setuid-wrappers/fusermount -u";
}
