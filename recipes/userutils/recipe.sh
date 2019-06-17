GIT=https://gitlab.redox-os.org/redox-os/userutils.git
BINS=(
    id
    getty
    groupadd
    groupmod
    groupdel
    login
    passwd
    su
    sudo
    useradd
    usermod
    userdel
)

function recipe_stage {
    # Reimplement the entire copy bc of suid
    if [ "$DEBUG" == 1 ]
    then
        build=debug
    else
        build=release
    fi

    mkdir -p "$1/bin"

    for bin in "${BINS[@]}"
    do
        "$STRIP" -v "target/$TARGET/$build/$bin" -o "$1/bin/$bin"
    done

    cp -Rv "res" "$1/etc"

    ln -s id "$1/bin/whoami"
    chmod +s "$1/bin/passwd"
    chmod +s "$1/bin/sudo"
    chmod +s "$1/bin/su"

    docgen ../source ../stage/ref

    skip=1
}
