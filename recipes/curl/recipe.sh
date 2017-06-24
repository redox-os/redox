GIT=https://github.com/ids1024/curl.git
BRANCH=redox

HOST=x86_64-elf-redox

function recipe_version {
    printf "r%s.%s" "$(git rev-list --count HEAD)" "$(git rev-parse --short HEAD)"
    skip=1
}

function recipe_update {
    echo "skipping update"
    skip=1
}

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

    rm -rf openssl-prefix/lib/pkgconfig # pkg-config returns paths based on / prefix, breaking cross compile

    ./configure --prefix=/ --host=${HOST} --disable-tftp --disable-ftp --disable-ntlm-wb --with-ssl="$PWD/openssl-prefix" --with-ca-path=/ssl/certs
    make

    skip=1
}

function recipe_test {
    echo "skipping test"
    skip=1
}

function recipe_clean {
    make clean
    skip=1
}

function recipe_stage {
    mkdir "$1/bin"
    cp src/curl "$1/bin"
    skip=1
}
