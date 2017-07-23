### Building Redox using Docker images with the toolchain

*All you need is git, make, qemu, fuse and docker. The method requires a non-privileged user able to run the `docker` command, which is usually achieved by adding the user to the `docker` group.*

```shell
git clone https://github.com/redox-os/redox.git ; cd redox #1
make pull #2
docker build -t redox docker/ #3
docker run --cap-add MKNOD --cap-add SYS_ADMIN \
    --device /dev/fuse -e LOCAL_USER_ID="$(id -u)" \
    -v "$(pwd):/src" --rm redox make all #4
make qemu #5
```
To unpack:
1. Creates a local copy of the repository.
2. Updates all the submodules in the repository.
3. Creates a new image in the local image repository named `redox` with Redox toolchain installed. You only need to rebuild the image if you want to update the toolchain.
4. Builds Redox using the `redox` image. The arguments allow the container to use `fuse` and ensure the resulting files are owned by the current user.
5. Runs Redox.

On selinux systems, replace #4 with:
```
docker run --cap-add MKNOD --cap-add SYS_ADMIN \
    --device /dev/fuse -e LOCAL_USER_ID="$(id -u)" \
    -v "$(pwd):/src" --security-opt label=disable \
    --rm redox make all
```
