### Building Redox using Docker images with the toolchain

*All you need is git, make, qemu, fuse and docker. The method requires a non-privileged user able to run the `docker` command, which is usually achieved by adding the user to the `docker` group.*

```shell
git clone https://github.com/redox-os/redox.git ; cd redox #1
docker build --build-arg LOCAL_UID="$(id -u)" --build-arg LOCAL_GID="$(id -g)" \
    -t redox docker/ #2
git pull --rebase --recurse-submodules && git submodule sync \
    && git submodule update --recursive --init #3
docker run --cap-add MKNOD --cap-add SYS_ADMIN \
    -e LOCAL_UID="$(id -u)" -e LOCAL_GID="$(id -g)" \
    --device /dev/fuse -v "$(pwd):/home/user/src" --rm redox make fetch all #4
make qemu #5
```
To unpack:
1. Creates a local copy of the repository.
2. Creates a new image in the local image repository named `redox` with Redox toolchain installed. You only need to rebuild the image if you want to update the toolchain.
3. Updates all the submodules in the repository.
4. Builds Redox using the `redox` image. The arguments allow the container to use `fuse` and ensure the resulting files are owned by the current user.
5. Runs Redox.

On selinux systems, replace #4 with:
```
docker run --cap-add MKNOD --cap-add SYS_ADMIN \
    -e LOCAL_UID="$(id -u)" -e LOCAL_GID="$(id -g)" \
    --device /dev/fuse -v "$(pwd):/home/user/src" --security-opt seccomp=unconfined --security-opt apparmor=unconfined \
    --rm redox make fetch all
```
