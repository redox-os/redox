#!/bin/sh
#
# Build a fat binary on Mac OS X, thanks Ryan!

# Number of CPUs (for make -j)
NCPU=`sysctl -n hw.ncpu`
if test x$NJOB = x; then
    NJOB=$NCPU
fi

# SDK path
if test x$SDK_PATH = x; then
    SDK_PATH=/Developer/SDKs
fi

# Generic, cross-platform CFLAGS you always want go here.
CFLAGS="-O3 -g -pipe"

# We dynamically load X11, so using the system X11 headers is fine.
BASE_CONFIG_FLAGS="--build=`uname -p`-apple-darwin \
--x-includes=/usr/X11R6/include --x-libraries=/usr/X11R6/lib"

# PowerPC 32-bit compiler flags
CONFIG_PPC="--host=powerpc-apple-darwin"
CC_PPC="gcc-4.0"
CXX_PPC="g++-4.0"
BUILD_FLAGS_PPC="-arch ppc -mmacosx-version-min=10.4"

# Intel 32-bit compiler flags
CONFIG_X86="--host=i386-apple-darwin"
CC_X86="gcc"
CXX_X86="g++"
BUILD_FLAGS_X86="-arch i386 -mmacosx-version-min=10.4"

# Intel 64-bit compiler flags
CONFIG_X64="--host=x86_64-apple-darwin"
CC_X64="gcc"
CXX_X64="g++"
BUILD_FLAGS_X64="-arch x86_64 -mmacosx-version-min=10.6"

#
# Find the configure script
#
srcdir=`dirname $0`/..
auxdir=$srcdir/build-scripts
cd $srcdir

allow_ppc="yes"
which gcc-4.0 >/dev/null 2>/dev/null
if [ "x$?" = "x1" ]; then
    #echo "WARNING: Can't find gcc-4.0, which means you don't have Xcode 3."
    #echo "WARNING: Therefore, we can't do PowerPC support."
    allow_ppc="no"
fi

#
# Figure out which phase to build:
# all,
# configure, configure-ppc, configure-x86, configure-x64
# make, make-ppc, make-x86, make-x64, merge
# install
# clean
if test x"$1" = x; then
    phase=all
else
    phase="$1"
fi
case $phase in
    all)
        configure_ppc="$allow_ppc"
        configure_x86="yes"
        configure_x64="yes"
        make_ppc="$allow_ppc"
        make_x86="yes"
        make_x64="yes"
        merge="yes"
        ;;
    configure)
        configure_ppc="$allow_ppc"
        configure_x86="yes"
        configure_x64="yes"
        ;;
    configure-ppc)
        configure_ppc="$allow_ppc"
        ;;
    configure-x86)
        configure_x86="yes"
        ;;
    configure-x64)
        configure_x64="yes"
        ;;
    make)
        make_ppc="$allow_ppc"
        make_x86="yes"
        make_x64="yes"
        merge="yes"
        ;;
    make-ppc)
        make_ppc="$allow_ppc"
        ;;
    make-x86)
        make_x86="yes"
        ;;
    make-x64)
        make_x64="yes"
        ;;
    merge)
        merge="yes"
        ;;
    install)
        install_bin="yes"
        install_hdrs="yes"
        install_lib="yes"
        install_data="yes"
        install_man="yes"
        ;;
    install-bin)
        install_bin="yes"
        ;;
    install-hdrs)
        install_hdrs="yes"
        ;;
    install-lib)
        install_lib="yes"
        ;;
    install-data)
        install_data="yes"
        ;;
    install-man)
        install_man="yes"
        ;;
    clean)
        clean_ppc="yes"
        clean_x86="yes"
        clean_x64="yes"
        ;;
    clean-ppc)
        clean_ppc="yes"
        ;;
    clean-x86)
        clean_x86="yes"
        ;;
    clean-x64)
        clean_x64="yes"
        ;;
    *)
        echo "Usage: $0 [all|configure[-ppc|-x86|-x64]|make[-ppc|-x86|-x64]|merge|install|clean[-ppc|-x86|-x64]]"
        exit 1
        ;;
esac
case `uname -p` in
    *86)
        native_path=x86
        ;;
    *powerpc)
        native_path=ppc
        ;;
    x86_64)
        native_path=x64
        ;;
    *)
        echo "Couldn't figure out native architecture path"
        exit 1
        ;;
esac

#
# Create the build directories
#
for dir in build build/ppc build/x86 build/x64; do
    if test -d $dir; then
        :
    else
        mkdir $dir || exit 1
    fi
done


#
# Build the PowerPC 32-bit binary
#
if test x$configure_ppc = xyes; then
    (cd build/ppc && \
     sh ../../configure $BASE_CONFIG_FLAGS $CONFIG_PPC CC="$CC_PPC" CXX="$CXX_PPC" CFLAGS="$CFLAGS $BUILD_FLAGS_PPC $CFLAGS_PPC" LDFLAGS="$BUILD_FLAGS_PPC $LFLAGS_PPC") || exit 2
fi
if test x$make_ppc = xyes; then
    (cd build/ppc && make -j$NJOB) || exit 3
fi
#
# Build the Intel 32-bit binary
#
if test x$configure_x86 = xyes; then
    (cd build/x86 && \
     sh ../../configure $BASE_CONFIG_FLAGS $CONFIG_X86 CC="$CC_X86" CXX="$CXX_X86" CFLAGS="$CFLAGS $BUILD_FLAGS_X86 $CFLAGS_X86" LDFLAGS="$BUILD_FLAGS_X86 $LFLAGS_X86") || exit 2
fi
if test x$make_x86 = xyes; then
    (cd build/x86 && make -j$NJOB) || exit 3
fi

#
# Build the Intel 64-bit binary
#
if test x$configure_x64 = xyes; then
    (cd build/x64 && \
     sh ../../configure $BASE_CONFIG_FLAGS $CONFIG_X64 CC="$CC_X64" CXX="$CXX_X64" CFLAGS="$CFLAGS $BUILD_FLAGS_X64 $CFLAGS_X64" LDFLAGS="$BUILD_FLAGS_X64 $LFLAGS_X64") || exit 2
fi
if test x$make_x64 = xyes; then
    (cd build/x64 && make -j$NJOB) || exit 3
fi

#
# Combine into fat binary
#
if test x$merge = xyes; then
    output=.libs
    sh $auxdir/mkinstalldirs build/$output
    cd build
    target=`find . -mindepth 4 -maxdepth 4 -type f -name '*.dylib' | head -1 | sed 's|.*/||'`
    (lipo -create -o $output/$target `find . -mindepth 4 -maxdepth 4 -type f -name "*.dylib"` &&
     ln -sf $target $output/libSDL.dylib &&
     lipo -create -o $output/libSDL.a */build/.libs/libSDL.a &&
     cp $native_path/build/.libs/libSDL.la $output &&
     cp $native_path/build/.libs/libSDL.lai $output &&
     cp $native_path/build/libSDL.la . &&
     lipo -create -o $output/libSDLmain.a */build/.libs/libSDLmain.a &&
     cp $native_path/build/.libs/libSDLmain.la $output &&
     cp $native_path/build/.libs/libSDLmain.lai $output &&
     cp $native_path/build/libSDLmain.la . &&
     echo "Build complete!" &&
     echo "Files can be found in the build directory.") || exit 4
    cd ..
fi

#
# Install
#
do_install()
{
    echo $*
    $* || exit 5
}
if test x$prefix = x; then
    prefix=/usr/local
fi
if test x$exec_prefix = x; then
    exec_prefix=$prefix
fi
if test x$bindir = x; then
    bindir=$exec_prefix/bin
fi
if test x$libdir = x; then
    libdir=$exec_prefix/lib
fi
if test x$includedir = x; then
    includedir=$prefix/include
fi
if test x$datadir = x; then
    datadir=$prefix/share
fi
if test x$mandir = x; then
    mandir=$prefix/man
fi
if test x$install_bin = xyes; then
    do_install sh $auxdir/mkinstalldirs $bindir
    do_install /usr/bin/install -c -m 755 build/$native_path/sdl-config $bindir/sdl-config
fi
if test x$install_hdrs = xyes; then
    do_install sh $auxdir/mkinstalldirs $includedir/SDL
    for src in $srcdir/include/*.h; do \
        file=`echo $src | sed -e 's|^.*/||'`; \
        do_install /usr/bin/install -c -m 644 $src $includedir/SDL/$file; \
    done
    do_install /usr/bin/install -c -m 644 $srcdir/include/SDL_config_macosx.h $includedir/SDL/SDL_config.h
fi
if test x$install_lib = xyes; then
    do_install sh $auxdir/mkinstalldirs $libdir
    do_install sh build/$native_path/libtool --mode=install /usr/bin/install -c  build/libSDL.la $libdir/libSDL.la
    do_install sh build/$native_path/libtool --mode=install /usr/bin/install -c  build/libSDLmain.la $libdir/libSDLmain.la
fi
if test x$install_data = xyes; then
    do_install sh $auxdir/mkinstalldirs $datadir/aclocal
    do_install /usr/bin/install -c -m 644 $srcdir/sdl.m4 $datadir/aclocal/sdl.m4
    do_install sh $auxdir/mkinstalldirs $libdir/pkgconfig
    do_install /usr/bin/install -m 644 build/$native_path/sdl.pc $libdir/pkgconfig/sdl.pc
fi
if test x$install_man = xyes; then
    do_install sh $auxdir/mkinstalldirs $mandir/man3
    for src in $srcdir/docs/man3/*.3; do \
        file=`echo $src | sed -e 's|^.*/||'`; \
        do_install /usr/bin/install -c -m 644 $src $mandir/man3/$file; \
    done
fi

#
# Clean up
#
do_clean()
{
    echo $*
    $* || exit 6
}
if test x$clean_ppc = xyes; then
    do_clean rm -r build/ppc
fi
if test x$clean_x86 = xyes; then
    do_clean rm -r build/x86
fi
if test x$clean_x64 = xyes; then
    do_clean rm -r build/x64
fi
