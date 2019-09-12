GIT=https://gitlab.redox-os.org/redox-os/drivers.git
CARGOBUILD="build"
CARGOFLAGS="--all"

function recipe_version {
    echo "0.1.1"
    skip=1
}

function recipe_stage {
    mkdir -pv "$1/etc/pcid"
    cp -v initfs.toml "$1/etc/pcid/initfs.toml"

    mkdir -pv "$1/etc/pcid.d"
    for conf in `find . -maxdepth 2 -type f -name 'config.toml'`; do
        driver=$(echo $conf | cut -d '/' -f2)
        cp -v $conf "$1/etc/pcid.d/$driver.toml"
    done

}
