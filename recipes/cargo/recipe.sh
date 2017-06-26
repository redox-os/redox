GIT=https://github.com/ids1024/cargo.git
BRANCH=redox

function recipe_build {
    if [ -d openssl-redox ]
    then
        git -C openssl-redox pull
    else
        git clone https://github.com/ids1024/openssl.git -b redox --depth 1 openssl-redox
    fi

    rm -rf openssl-prefix
    mkdir openssl-prefix

    pushd openssl-redox
        ./Configure no-shared no-dgram redox-x86_64 --prefix="/"
	make -j"$(nproc)"
	make DESTDIR="$PWD/../openssl-prefix" install
    popd

    export OPENSSL_DIR=$PWD/openssl-prefix
}
