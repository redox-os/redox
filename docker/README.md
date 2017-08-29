## Building Redox using a Docker image with the pre-built toolchain

*All you need is `git`, `make`, `qemu`, `fuse` and `docker`. The method requires
a non-privileged user able to run the `docker` command, which is usually achieved
by adding the user to the `docker` group.*

It's a four-steps process with variations depending on the platform.

### <a name='get_the_sources'></a>Get the sources
```
git clone https://github.com/redox-os/redox.git ; cd redox
```

### Build the container
This will prepare an Ubuntu 17.04 docker image with the required
dependencies and the pre-built toolchain. As long as you rely on this particular
dependencies and toolchain versions, you don't need to rebuild the container.
#### Linux
```shell
docker build --build-arg LOCAL_UID="$(id -u)" --build-arg LOCAL_GID="$(id -g)" \
    -t redox docker/
```
#### MacOS
```shell
docker build -t redox docker/
```

### Upate the source tree
Note: if you use the container on a different host or
with a different user, [get the sources first](#get_the_sources).
```shell
git pull --rebase --recurse-submodules && git submodule sync \
    && git submodule update --recursive --init
```

### Run the container to build Redox
#### Linux without security modules
```shell
docker run --cap-add MKNOD --cap-add SYS_ADMIN --device /dev/fuse \
    -e LOCAL_UID="$(id -u)" -e LOCAL_GID="$(id -g)" \
    -v redox-"$(id -u)"-"$(id -g)"-cargo:/home/user/.cargo \
    -v "$(pwd):/home/user/src" --rm redox make fetch all
```
#### Linux with security modules<br>
Add the following options depending on the security modules activated on your system:
```shell
--security-opt label=disable         // disable SELinux
--security-opt seccomp=unconfined    // disable seccomp
--security-opt apparmor=unconfined   // disable AppArmor
```
Ex.: for a SELinux only system such as Fedora or CentOS
```shell
docker run --cap-add MKNOD --cap-add SYS_ADMIN --device /dev/fuse \
    -e LOCAL_UID="$(id -u)" -e LOCAL_GID="$(id -g)" \
    --security-opt label=disable \
    -v redox-"$(id -u)"-"$(id -g)"-cargo:/home/user/.cargo \
    -v "$(pwd):/home/user/src" --rm redox make fetch all
```
#### MacOS
```shell
docker run --cap-add MKNOD --cap-add SYS_ADMIN --device /dev/fuse \
    -v redox-cargo:/home/user/.cargo \
    -v "$(pwd):/home/user/src" --rm redox make fetch all
```

### Clear the named volume containing the cargo cache
#### Linux
```shell
docker volume rm redox-"$(id -u)"-"$(id -g)"-cargo
```

#### MacOS
```shell
docker volume rm redox-cargo
```
