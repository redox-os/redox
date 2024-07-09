# This script run the Docker image of Redox

docker run --privileged --cap-add MKNOD --cap-add SYS_ADMIN --device /dev/fuse \
    -e LOCAL_UID="$(id -u)" -e LOCAL_GID="$(id -g)" \
    -v redox-"$(id -u)-$(id -g)"-cargo:/usr/local/cargo \
    -v redox-"$(id -u)-$(id -g)"-rustup:/usr/local/rustup \
    -v "$(pwd):$(pwd)" -w "$(pwd)" --rm -it redoxos/redox "$@"
