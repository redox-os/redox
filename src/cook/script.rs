pub(crate) static SHARED_PRESCRIPT: &str = r#"
# Build dynamically
function DYNAMIC_INIT {
    COOKBOOK_AUTORECONF="autoreconf"
    autotools_recursive_regenerate() {
        for f in $(find . -name configure.ac -o -name configure.in -type f | sort); do
            echo "* autotools regen in '$(dirname $f)'..."
            ( cd "$(dirname "$f")" && "${COOKBOOK_AUTORECONF}" -fvi "$@" -I${COOKBOOK_HOST_SYSROOT}/share/aclocal )
        done
    }

    case "${TARGET}" in
        "x86_64-unknown-redox")
            ;;
        "x86_64-unknown-linux-gnu")
            ;;
        *)
            [ -z "${COOKBOOK_VERBOSE}" ] || echo "WARN: ${TARGET} does not support dynamic linking." >&2
            return
            ;;
    esac

    [ -z "${COOKBOOK_VERBOSE}" ] || echo "DEBUG: Program is being compiled dynamically."

    COOKBOOK_CONFIGURE_FLAGS=(
        --host="${GNU_TARGET}"
        --prefix="/usr"
        --enable-shared
        --disable-static
    )

    COOKBOOK_CMAKE_FLAGS=(
        -DBUILD_SHARED_LIBS=True
        -DENABLE_SHARED=True
        -DENABLE_STATIC=False
    )

    COOKBOOK_MESON_FLAGS=(
        --buildtype release
        --wrap-mode nofallback
        --strip
        -Ddefault_library=shared
        -Dprefix=/usr
    )

    # TODO: check paths for spaces
    export LDFLAGS="-Wl,-rpath-link,${COOKBOOK_SYSROOT}/lib -L${COOKBOOK_SYSROOT}/lib"
    export RUSTFLAGS="-C target-feature=-crt-static"
    export COOKBOOK_DYNAMIC=1
}

# Build both dynamically and statically
function DYNAMIC_STATIC_INIT {
    DYNAMIC_INIT
    if [ "${COOKBOOK_DYNAMIC}" == "1" ]
    then
        COOKBOOK_CONFIGURE_FLAGS=(
            --host="${GNU_TARGET}"
            --prefix="/usr"
            --enable-shared
            --enable-static
        )

        COOKBOOK_CMAKE_FLAGS=(
            -DBUILD_SHARED_LIBS=True
            -DENABLE_SHARED=True
            -DENABLE_STATIC=True
        )

        COOKBOOK_MESON_FLAGS=(
            --buildtype release
            --wrap-mode nofallback
            --strip
            -Ddefault_library=both
            -Dprefix=/usr
        )
    fi
}

function GNU_CONFIG_GET {
  wget -O "$1" "https://gitlab.redox-os.org/redox-os/gnu-config/-/raw/master/config.sub?inline=false"
}
"#;

pub(crate) static BUILD_PRESCRIPT: &str = r#"
# Add cookbook bins to path
if [ -z "${IS_REDOX}" ]
then
export PATH="${COOKBOOK_ROOT}/bin:${PATH}"
fi

# This puts cargo build artifacts in the build directory
export CARGO_TARGET_DIR="${COOKBOOK_BUILD}/target"

# This adds the sysroot includes for most C compilation
#TODO: check paths for spaces!
export CFLAGS="-I${COOKBOOK_SYSROOT}/include"
export CPPFLAGS="-I${COOKBOOK_SYSROOT}/include"

# This adds the sysroot libraries and compiles binaries statically for most C compilation
#TODO: check paths for spaces!
export LDFLAGS="-L${COOKBOOK_SYSROOT}/lib --static"

# These ensure that pkg-config gets the right flags from the sysroot
if [ "${TARGET}" != "${COOKBOOK_HOST_TARGET}" ]
then
    export PKG_CONFIG_ALLOW_CROSS=1
    export PKG_CONFIG_PATH=
    export PKG_CONFIG_LIBDIR="${COOKBOOK_SYSROOT}/lib/pkgconfig"
    export PKG_CONFIG_SYSROOT_DIR="${COOKBOOK_SYSROOT}"
fi

# To build the debug version of a Cargo program, add COOKBOOK_DEBUG=true, and
# to not strip symbols from the final package, add COOKBOOK_NOSTRIP=true to the recipe
# (or to your environment) before calling cookbook_cargo or cookbook_cargo_packages
build_type=release
install_flags=
build_flags=--release
if [ ! -z "${COOKBOOK_DEBUG}" ]
then
    install_flags=--debug
    build_flags=
    build_type=debug
    export CFLAGS="${CFLAGS} -g"
    export CPPFLAGS="${CPPFLAGS} -g"
fi

if [ ! -z "${COOKBOOK_OFFLINE}" ]
then
build_flags+=" --offline"
install_flags+=" --offline"
fi

# cargo template
COOKBOOK_CARGO="${COOKBOOK_REDOXER}"
function cookbook_cargo {
    "${COOKBOOK_CARGO}" install \
        --path "${COOKBOOK_SOURCE}/${PACKAGE_PATH}" \
        --root "${COOKBOOK_STAGE}/usr" \
        --locked \
        --no-track \
        ${install_flags} \
         -j "${COOKBOOK_MAKE_JOBS}" "$@"
}

# helper for installing binaries that are cargo examples
function cookbook_cargo_examples {
    recipe="$(basename "${COOKBOOK_RECIPE}")"
    for example in "$@"
    do
        "${COOKBOOK_CARGO}" build \
            --manifest-path "${COOKBOOK_SOURCE}/${PACKAGE_PATH}/Cargo.toml" \
            --example "${example}" \
            ${build_flags} -j "${COOKBOOK_MAKE_JOBS}"
        mkdir -pv "${COOKBOOK_STAGE}/usr/bin"
        cp -v \
            "target/${TARGET}/${build_type}/examples/${example}" \
            "${COOKBOOK_STAGE}/usr/bin/${recipe}_${example}"
    done
}

# helper for installing binaries that are cargo packages
function cookbook_cargo_packages {
    recipe="$(basename "${COOKBOOK_RECIPE}")"
    for package in "$@"
    do
        "${COOKBOOK_CARGO}" build \
            --manifest-path "${COOKBOOK_SOURCE}/${PACKAGE_PATH}/Cargo.toml" \
            --package "${package}" \
            ${build_flags} -j "${COOKBOOK_MAKE_JOBS}"
        mkdir -pv "${COOKBOOK_STAGE}/usr/bin"
        cp -v \
            "target/${TARGET}/${build_type}/${package}" \
            "${COOKBOOK_STAGE}/usr/bin/${recipe}_${package}"
    done
}

# configure template
COOKBOOK_CONFIGURE="${COOKBOOK_SOURCE}/configure"
COOKBOOK_CONFIGURE_FLAGS=(
    --host="${GNU_TARGET}"
    --prefix="/usr"
    --disable-shared
    --enable-static
)
COOKBOOK_MAKE="make"

if [ -z "${COOKBOOK_MAKE_JOBS}" ]
then
if [ -z "${IS_REDOX}" ]
then
COOKBOOK_MAKE_JOBS="$(nproc)"
else
COOKBOOK_MAKE_JOBS="1"
fi
fi

function cookbook_configure {
    "${COOKBOOK_CONFIGURE}" "${COOKBOOK_CONFIGURE_FLAGS[@]}" "$@"
    "${COOKBOOK_MAKE}" -j "${COOKBOOK_MAKE_JOBS}"
    "${COOKBOOK_MAKE}" install DESTDIR="${COOKBOOK_STAGE}"
}

COOKBOOK_CMAKE="cmake"
COOKBOOK_NINJA="ninja"
COOKBOOK_CMAKE_FLAGS=(
    -DBUILD_SHARED_LIBS=False
    -DENABLE_SHARED=False
    -DENABLE_STATIC=True
)
function cookbook_cmake {
    cat > cross_file.cmake <<EOF
set(CMAKE_AR ${GNU_TARGET}-ar)
set(CMAKE_CXX_COMPILER ${GNU_TARGET}-g++)
set(CMAKE_C_COMPILER ${GNU_TARGET}-gcc)
set(CMAKE_FIND_ROOT_PATH ${COOKBOOK_SYSROOT})
set(CMAKE_FIND_ROOT_PATH_MODE_INCLUDE ONLY)
set(CMAKE_FIND_ROOT_PATH_MODE_LIBRARY ONLY)
set(CMAKE_FIND_ROOT_PATH_MODE_PROGRAM NEVER)
set(CMAKE_PLATFORM_USES_PATH_WHEN_NO_SONAME 1)
set(CMAKE_PREFIX_PATH, ${COOKBOOK_SYSROOT})
set(CMAKE_RANLIB ${GNU_TARGET}-ranlib)
set(CMAKE_SHARED_LIBRARY_SONAME_C_FLAG "-Wl,-soname,")
set(CMAKE_SYSTEM_NAME UnixPaths)
set(CMAKE_SYSTEM_PROCESSOR $(echo "${TARGET}" | cut -d - -f1))
EOF

    if [ -n "${CC_WRAPPER}" ]
    then
        echo "set(CMAKE_C_COMPILER_LAUNCHER ${CC_WRAPPER})" >> cross_file.cmake
        echo "set(CMAKE_CXX_COMPILER_LAUNCHER ${CC_WRAPPER})" >> cross_file.cmake
    fi

    "${COOKBOOK_CMAKE}" "${COOKBOOK_SOURCE}" \
        -DCMAKE_BUILD_TYPE=Release \
        -DCMAKE_CROSSCOMPILING=True \
        -DCMAKE_INSTALL_INCLUDEDIR=include \
        -DCMAKE_INSTALL_LIBDIR=lib \
        -DCMAKE_INSTALL_OLDINCLUDEDIR=/include \
        -DCMAKE_INSTALL_PREFIX=/usr \
        -DCMAKE_INSTALL_SBINDIR=bin \
        -DCMAKE_TOOLCHAIN_FILE=cross_file.cmake \
        -GNinja \
        -Wno-dev \
        "${COOKBOOK_CMAKE_FLAGS[@]}" \
        "$@"

    "${COOKBOOK_NINJA}" -j"${COOKBOOK_MAKE_JOBS}"
    DESTDIR="${COOKBOOK_STAGE}" "${COOKBOOK_NINJA}" install -j"${COOKBOOK_MAKE_JOBS}"
}

COOKBOOK_MESON="meson"
COOKBOOK_MESON_FLAGS=(
    --buildtype release
    --wrap-mode nofallback
    --strip
    -Ddefault_library=static
    -Dprefix=/usr
)
function cookbook_meson {
    echo "[binaries]" > cross_file.txt
    echo "c = [$(printf "'%s', " $CC | sed 's/, $//')]"  >> cross_file.txt
    echo "cpp = [$(printf "'%s', " $CXX | sed 's/, $//')]" >> cross_file.txt
    echo "ar = '${AR}'" >> cross_file.txt
    echo "strip = '${STRIP}'" >> cross_file.txt
    echo "pkg-config = '${PKG_CONFIG}'" >> cross_file.txt
    echo "llvm-config = '${TARGET}-llvm-config'" >> cross_file.txt
    echo "glib-compile-resources = 'glib-compile-resources'" >> cross_file.txt
    echo "glib-compile-schemas = 'glib-compile-schemas'" >> cross_file.txt

    echo "[host_machine]" >> cross_file.txt
    echo "system = 'redox'" >> cross_file.txt
    echo "cpu_family = '$(echo "${TARGET}" | cut -d - -f1)'" >> cross_file.txt
    echo "cpu = '$(echo "${TARGET}" | cut -d - -f1)'" >> cross_file.txt
    echo "endian = 'little'" >> cross_file.txt

    echo "[paths]" >> cross_file.txt
    echo "prefix = '/usr'" >> cross_file.txt
    echo "libdir = 'lib'" >> cross_file.txt
    echo "bindir = 'bin'" >> cross_file.txt

    echo "[properties]" >> cross_file.txt
    echo "needs_exe_wrapper = true" >> cross_file.txt
    echo "sys_root = '${COOKBOOK_SYSROOT}'" >> cross_file.txt
    echo "c_args = [$(printf "'%s', " $CFLAGS | sed 's/, $//')]" >> cross_file.txt
    echo "cpp_args = [$(printf "'%s', " $CPPFLAGS | sed 's/, $//')]" >> cross_file.txt
    echo "c_link_args = [$(printf "'%s', " $LDFLAGS | sed 's/, $//')]" >> cross_file.txt

    unset AR
    unset AS
    unset CC
    unset CXX
    unset LD
    unset NM
    unset OBJCOPY
    unset OBJDUMP
    unset PKG_CONFIG
    unset RANLIB
    unset READELF
    unset STRIP

    "${COOKBOOK_MESON}" setup \
        "${COOKBOOK_SOURCE}" \
        . \
        --cross-file cross_file.txt \
        "${COOKBOOK_MESON_FLAGS[@]}" \
        "$@"
    "${COOKBOOK_NINJA}" -j"${COOKBOOK_MAKE_JOBS}"
    DESTDIR="${COOKBOOK_STAGE}" "${COOKBOOK_NINJA}" install -j"${COOKBOOK_MAKE_JOBS}"
}
"#;

pub(crate) static BUILD_POSTSCRIPT: &str = r#"
# Strip binaries
for dir in "${COOKBOOK_STAGE}/bin" "${COOKBOOK_STAGE}/usr/bin"
do
    if [ -d "${dir}" ] && [ -z "${COOKBOOK_NOSTRIP}" ]
    then
        find "${dir}" -type f -exec "${GNU_TARGET}-strip" -v {} ';'
    fi
done

# Remove libtool files
for dir in "${COOKBOOK_STAGE}/lib" "${COOKBOOK_STAGE}/usr/lib"
do
    if [ -d "${dir}" ]
    then
        find "${dir}" -type f -name '*.la' -exec rm -fv {} ';'
    fi
done

# Remove cargo install files
for file in .crates.toml .crates2.json
do
    if [ -f "${COOKBOOK_STAGE}/${file}" ]
    then
        rm -v "${COOKBOOK_STAGE}/${file}"
    fi
done

# Add pkgname to appstream metadata
for dir in "${COOKBOOK_STAGE}/share/metainfo" "${COOKBOOK_STAGE}/usr/share/metainfo"
do
    if [ -d "${dir}" ]
    then
        find "${dir}" -type f -name '*.xml' -exec sed -i 's|</component>|<pkgname>'"${COOKBOOK_NAME}"'</pkgname></component>|g' {} ';'
    fi
done
"#;

pub(crate) static GIT_RESET_BRANCH: &str = r#"
ORIGIN_BRANCH="$(git branch --remotes | grep '^  origin/HEAD -> ' | cut -d ' ' -f 5-)"
if [ -n "$BRANCH" ]
then
    ORIGIN_BRANCH="origin/$BRANCH"
fi

if [ "$(git rev-parse HEAD)" != "$(git rev-parse $ORIGIN_BRANCH)" ]
then
    git checkout -B "$(echo "$ORIGIN_BRANCH" | cut -d / -f 2-)" "$ORIGIN_BRANCH"
fi"#;

pub static KILL_ALL_PID: &str = r#"
THISPID=$$
CHILDREN=$(ps -o pid= --ppid $PID | grep -v $THISPID);

ALL_DESCENDANTS='';

while [ -n "$CHILDREN" ]; do
    ALL_DESCENDANTS="$ALL_DESCENDANTS $CHILDREN";
    CHILDREN=$(ps -o pid= --ppid $(echo $CHILDREN) | tr '\n' ' ');
done;

if [ -n "$ALL_DESCENDANTS" ]; then
    kill -9 $ALL_DESCENDANTS;
fi
"#;
