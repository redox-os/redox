GIT=https://github.com/redox-os/relibc.git

function recipe_build {
    cp -r "$ROOT/Xargo.toml" .
    xargo build --target "$TARGET" --release
    xargo rustc --manifest-path src/crt0/Cargo.toml --target "$TARGET" --release -- -v --emit obj="target/$TARGET/release/crt0.o"
    skip=1
}

function recipe_stage {
    dest="$(realpath $1)"
    mkdir -pv "$dest/lib"
    mkdir -pv "$dest/include"
    cp -rv "include"/* "$dest/include"
    cp -rv "target/include"/* "$dest/include"
    cp -v "target/$TARGET/release/libc.a" "$dest/lib"
    cp -v "target/$TARGET/release/crt0.o" "$dest/lib"
    skip=1
}
