#!/bin/bash
set -e

HOST="i386-elf-redox"
BUILD="$(dirname "${PWD}")/build"
PREFIX="${BUILD}/sysroot/usr"
export PATH="${BUILD}/prefix/bin:${PREFIX}/bin:$PATH"
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
            if [ -n "${SRC}" ]
            then
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
            elif [ -n "${GIT}" ]
            then
                if [ ! -d "${BUILD}/${DIR}" ]
                then
                    pushd "${BUILD}"
                    git clone "${GIT}"
                    popd
                fi
            fi
            if [ -d "${DIR}" ]
            then
                cp -rv "${DIR}"/* "${BUILD}/${DIR}"
            fi
            ;;
        remove)
            if [ -d "${BUILD}/${DIR}" ]
            then
                rm -rfv "${BUILD}/${DIR}"
            fi
            if [ -n "${SRC}" -a -f "${BUILD}/$(basename "${SRC}")" ]
            then
                rm -fv "${BUILD}/$(basename "${SRC}")"
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

function autoconf_template {
    case $1 in
        autoconf)
            pushd "${BUILD}/${DIR}"
                autoconf $AUTOCONF_ARGS
            popd
            ;;
        add)
            fetch_template add
            autoconf_template autoconf
            configure_template configure
            make_template build
            make_template install
            ;;
        *)
            configure_template $*
            ;;
    esac
}

function autogen_template {
    case $1 in
        autogen)
            pushd "${BUILD}/${DIR}"
                ./autogen.sh $AUTOGEN_ARGS
            popd
            ;;
        add)
            fetch_template add
            autogen_template autogen
            configure_template configure
            make_template build
            make_template install
            ;;
        *)
            configure_template $*
            ;;
    esac
}
