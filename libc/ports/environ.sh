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
        fetch)
            if [ ! -f "$(basename "${SRC}")" ]
            then
                curl "${SRC}" -o "$(basename "${SRC}")"
            fi
            ;;
        extract)
            if [ ! -d "${DIR}" ]
            then
                tar xvf "$(basename "${SRC}")"
            fi
            ;;
        add)
            fetch_template fetch
            fetch_template extract
            ;;
        unextract)
            if [ -d "${DIR}" ]
            then
                rm -rf "${DIR}"
            fi
            ;;
        unfetch)
            if [ -f "$(basename "${SRC}")" ]
            then
                rm -f "$(basename "${SRC}")"
            fi
            ;;
        remove)
            fetch_template unextract
            fetch_template unfetch
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
            make_template uninstall
            make_template clean
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
            configure_template configure
            make_template add
            ;;
        distclean)
            make -C "${BUILD}/${DIR}" -j `nproc` distclean $DISTCLEAN_ARGS
            ;;
        remove)
            make_template remove
            configure_template distclean
            ;;
        *)
            make_template $*
            ;;
    esac
}
