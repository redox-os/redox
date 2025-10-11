//TODO: pub(crate)
pub static SHARED_PRESCRIPT: &str = r#"
# Build dynamically
function DYNAMIC_INIT {
    COOKBOOK_AUTORECONF="autoreconf"
    autotools_recursive_regenerate() {
        for f in $(find . -name configure.ac -o -name configure.in -type f | sort); do
            echo "* autotools regen in '$(dirname $f)'..."
            ( cd "$(dirname "$f")" && "${COOKBOOK_AUTORECONF}" -fvi "$@" -I${COOKBOOK_HOST_SYSROOT}/share/aclocal )
        done
    }

    if [ "${TARGET}" != "x86_64-unknown-redox" ]
    then
        echo "WARN: ${TARGET} does not support dynamic linking." >&2
        return
    fi

    echo "DEBUG: Program is being compiled dynamically."

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
