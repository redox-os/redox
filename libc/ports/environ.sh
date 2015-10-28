#!/bin/bash
set -e

HOST="i386-elf-redox"
BUILD="$(dirname "${PWD}")/build"
PREFIX="${BUILD}/sysroot/usr"
export PATH="${BUILD}/prefix/bin:$PATH"
export AR="${HOST}-ar"
export AS="${HOST}-as"
export CC="${HOST}-gcc"
export CXX="${HOST}-g++"
export LD="${HOST}-ld"
export NM="${HOST}-nm"
export OBJCOPY="${HOST}-objcopy"
export OBJDUMP="${HOST}-objdump"
export RANLIB="${HOST}-ranlib"
export READELF="${HOST}-readelf"
export STRIP="${HOST}-strip"

function fetch_template {
    case $1 in
        add)
            if [ ! -f "${BUILD}/$(basename "${SRC}")" ]
            then
                wget "${SRC}" -O "${BUILD}/$(basename "${SRC}")"
            fi
            if [ ! -d "${BUILD}/${DIR}" ]
            then
                pushd "${BUILD}"
                tar xvf "$(basename "${SRC}")"
                popd
            fi
            if [ -d "${DIR}" ]
            then
                cp -r -v "${DIR}"/* "${BUILD}/${DIR}"
            fi
            ;;
        remove)
            if [ -d "${BUILD}/${DIR}" ]
            then
                rm -rf "${BUILD}/${DIR}"
            fi
            if [ -f "${BUILD}/$(basename "${SRC}")" ]
            then
                rm -f "${BUILD}/$(basename "${SRC}")"
            fi
            ;;
        *)
            echo "$0: Unknown argument: '$1'. Try running with 'add' or 'remove'"
            ;;
    esac
}


function make_template {
    case $1 in
        build)
            make -C "${BUILD}/${DIR}" -j `nproc` $BUILD_ARGS
            ;;
        install)
            make -C "${BUILD}/${DIR}" -j `nproc` install $INSTALL_ARGS
            ;;
        add)
            fetch_template add
            make_template build
            make_template install
            ;;
        clean)
            make -C "${BUILD}/${DIR}" -j `nproc` clean $CLEAN_ARGS
            ;;
        uninstall)
            make -C "${BUILD}/${DIR}" -j `nproc` uninstall $UNINSTALL_ARGS
            ;;
        remove)
            make_template uninstall || true
            make_template clean || true
            fetch_template remove
            ;;
        *)
            fetch_template $*
            ;;
    esac
}

function configure_template {
    case $1 in
        configure)
            pushd "${BUILD}/${DIR}"
            ./configure --prefix="${PREFIX}" $CONFIGURE_ARGS
            popd
            ;;
        add)
            fetch_template add
            configure_template configure
            make_template build
            make_template install
            ;;
        distclean)
            make -C "${BUILD}/${DIR}" -j `nproc` distclean $DISTCLEAN_ARGS
            ;;
        remove)
            make_template uninstall || true
            configure_template distclean || true
            fetch_template remove
            ;;
        *)
            make_template $*
            ;;
    esac
}

function autogen_template {
    case $1 in
        add)
            fetch_template add
            pushd "${BUILD}/${DIR}"
                ./autogen.sh
            popd
            configure_template configure
            make_template build
            make_template install
            ;;
        *)
            configure_template $*
            ;;
    esac
}
