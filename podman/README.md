# Using Rootless Podman for your build

To make the Redox build process more consistent across platforms, we are using **Rootless Podman** for major parts of the build. **Podman** is invoked automatically and transparently within the Makefiles.

## Disabling Podman build

By default, the build process should operate as it did in the past. The variable **PODMAN_BUILD** in `mk/config.mk` defaults to zero, so that **Podman** will not be invoked.

## TL;DR - [New](#new-working-directory) or [Existing](#existing-working-directory) Working Directory

### New Working Directory 

- Make sure you have the `curl` command. E.g. for Pop!_OS/Ubuntu/Debian:
```sh
which curl || sudo apt-get install curl 
```
- Make a directory, get a copy of `podman_bootstrap.sh` and run it. This will clone the repository and install **Podman**.
```sh
mkdir -p ~/tryredox
cd ~/tryredox
curl -sf https://gitlab.redox-os.org/redox-os/redox/raw/master/podman_bootstrap.sh -o podman_bootstrap.sh
time bash -e podman_bootstrap.sh
```
- Change to the `redox` directory.
```sh
cd ~/tryredox/redox
```
- Edit `mk/config.mk` and change PODMAN_BUILD to 1.
```sh
gedit mk/config.mk &
```
```
...
PODMAN_BUILD?=1
...
```
- Build the system.
```sh
time make all
```

### Existing Working Directory

- Change to your working directory and get the updates to the build files.
```sh
cd ~/tryredox/redox
git fetch upstream master
git rebase upstream/master
```

- Install **Podman**. Many distros require additional packages. Check the [Minimum Installation](#minimum-installation) instructions to see what is needed for your distro. Or, run the following in your working directory:
```sh
./podman_bootstrap.sh -d
```

- Set `PODMAN_BUILD` to 1 and run `make`.
```sh
export PODMAN_BUILD=1
make all
```

To ensure `PODMAN_BUILD` is properly set for future builds, edit `mk/config.mk` and change its value.
```sh
gedit mk/config.mk &
```
```
...
PODMAN_BUILD?=1
...
```

## Minimum Installation

Most of the packages required for the build are installed in the container as part of the build process. However, some packages need to be installed on the host computer. You may also need to install an emulator such as **QEMU**. This is done for you in `podman_bootstrap.sh`, but you can do a minimum install by following the instructions below.

### Pop!_OS

```sh
sudo apt-get install podman
```

### Ubuntu

```sh
sudo apt-get install podman curl git make libfuse-dev
```

### ArchLinux

```sh
sudo pacman -S --needed git podman fuse
```

### Fedora

```sh
sudo dnf install podman
```

## Podman Build Overview

**Podman** is a container that executes a virtual machine image. In our case, we are creating an **Ubuntu** image, with a **Rust** installation and all the packages needed to build the system.

The build process is performed in your normal working directory, e.g. `~/tryredox/redox`. Compilation of the Redox components is performed in the container, but the final Redox image (`build/ARCH/CONFIG/harddrive.img` or `build/ARCH/CONFIG/livedisk.iso`) is constructed using [Fuse](https://github.com/libfuse/libfuse) running directly on your host machine.

Setting `PODMAN_BUILD` to 1 in the environment (e.g. `PODMAN_BUILD=1 make all`) or in `mk/config.mk` will cause **Podman** to be invoked when building.

First, a **base image** called `redox_base` will be constructed, with all the necessary packages for the build. A "home" directory will also be created in the image. This is the home directory of your container alter ego, `poduser`. It will contain the `rustup` install, and the `.bashrc`. This takes some time, but is only done when necessary. The *tag* file [build/container.tag](#buildcontainertag) is also created at this time to prevent unnecessary image builds.

Then, various `make` commands are executed in **containers** built from the **base image**. The files are constructed in your working directory tree, just as they would for a non-Podman build. In fact, if all necessary packages are installed on your host system, you can switch Podman on and off relatively seamlessly, although there is no benefit to doing so.

The build process is using **Podman**'s `keep-id` feature, which allows your regular User ID to be mapped to `poduser` in the container. The first time a container is built, it takes some time to set up this mapping. In order to minimize the impact of this, immediately after creating the image, a throw-away container is built. After the first container is built, new containers can be built almost instantly.

### NOTES

- Envionment and Command Line Variables, other than ARCH, CONFIG_NAME and FILESYSTEM_CONFIG, are not passed to the part of `make` that is done in **Podman**. You must set any other config variables in `mk/config.mk` and not on the command line or in your environment.

- If you are building your own software to include in Redox, and you need to install additional packages using `apt-get` for the build, follow [Adding Packages to the Build](#adding-packages-to-the-build).

## build/container.tag

The building of the **image** is controlled by the *tag* file `build/container.tag`. If you run `make all` with **PODMAN_BUILD=1**, the file `build/container.tag` will be created after the **image** is built. This file tells `make` that it can skip updating the **image** after the first time.

Many targets in the Makefiles `mk/*.mk` include `build/container.tag` as a dependency. If the *tag* file is missing, building any of those targets may trigger an image to be created, which can take some time.

When you move to a new working directory, if you want to save a few minutes, and you are confident that your **image** is correct, you can do
```sh
make container_touch
```
This will create the file `build/container.tag` without rebuilding the image. However, it will fail if the image does not exist. If it fails, just do a normal `make`, it will create the container when needed.

## Cleaning Up

To remove the **base image**, any lingering containers, and `build/container.tag`, use
```sh
make container_clean
```

To check that everything has been removed,
```sh
podman ps -a
podman images
```
will show any remaining images or containers. If you need to do further cleanup,
```
podman system reset
```
will remove **all** images and containers. You still may need to remove `build/container.tag` if you did not do `make container_clean`.

**Note:** `make clean` could trigger an image build. It invokes `cargo clean` on various components, which it must run in a container, since the build is designed to not require **Cargo** on your host machine. `make clean` does **not** run `make container_clean` and will **not** remove the container image.

## Debugging your Build Process

If you are developing your own components and wish to do one-time debugging to determine what package you are missing in the **Podman Build** environment, the following instructions can help. Note that your changes will not be persistent. After debugging, **you must** [Add your Packages to the Build](#adding-packages-to-the-build). With **PODMAN_BUILD=1**, run the command:
```sh
make container_shell
```

This will start a `bash` shell in the **Podman** container environment, as a normal user without `sudo` privilege. Within that environment, you can build the Redox components with:
```sh
make repo
```
or, if you need to change `ARCH` or `CONFIG_NAME`,
```sh
./build.sh -a ARCH -c CONFIG_NAME repo
```

If you need `root` privileges, while you are **still running** the above `bash` shell, go to a separate **Terminal** or **Console** window on the host and type:
```sh
podman ps
```

This will list all running containers. There should be only one, but if there is more than one, consider only the newest. In the last column of the display, the container will have a randomly generated name `ADJECTIVE_NOUN`, e.g. `boring_dickens`. Replace the word `CONTAINER` with that name and type:

```sh
podman exec --user=0 -it CONTAINER bash
```

You will then be running bash with `root` privilege in the container, and you can use `apt-get` or whatever tools you need, and it will affect the environment of the user-level `container_shell` above. Do not precede the commands with `sudo` as you are already `root`. And remember that you are in an **Ubuntu** instance.

**Note**: Your changes will not persist once both shells have been exited.

Type `exit` on both shells once you have determined how to solve your problem.

## Adding Packages to the Build

The default **Containerfile**, `podman/redox-base-containerfile`, imports all required packages for a normal Redox build.

However, you cannot easily add packages after the **base image** is created. You must add them to your own Containerfile. 

Copy `podman/redox-base-containerfile` and add to the list of packages in the initial `apt-get`.

Then, edit `mk/config.mk`, and change the variable **CONTAINERFILE** to point to your Containerfile, e.g.
```
CONTAINERFILE?=podman/my-containerfile
```

If your Containerfile is newer than `build/container.tag`, a new **image** will be created. You can force the image to be rebuilt with `make container_clean`.

If you feel the need to have more than one **image**, you can change the variable **IMAGE_TAG** in `mk/podman.mk` to give the image a different name.

## Summary of Podman-related Make Targets and Podman Commands

- `make build/container.tag`: If no container image has been built, build one. It's not necessary to do this, it will be done when needed.

- `make container_touch`: If a container image already exists, but there is no *tag* file, create the *tag* file so a new image is not built.

- `make container_clean`: Remove the container image and the *tag* file.

- `make container_shell`: Start an interactive `bash` shell in the same environment used by `make`.

- `podman exec --user=0 -it CONTAINER bash`: Use this command in combination with `make container_shell` to get root access to the normal build environment, so you can temporarily add packages to the environment. `CONTAINER` is the name of the active container as shown by `podman ps`. For temporary, debugging purposes only.

- `podman system reset`: Use this command when `make container_clean` is not sufficient to solve problems caused by errors in the container image. It will remove all images, use with caution. If you are using **Podman** for any other purpose, those images will be deleted as well.

## Gory Details

If you are interested in how we are able to use your working directory for builds in **Podman**, the following configuration details may be interesting.

We are using **Rootless Podman**'s `--userns keep-id` feature. Because **Podman** is being run **Rootless**, the *container's* `root` user is actually mapped to your User ID on the host. Without the `keep-id` option, a regular user in the container maps to a phantom user outside the container. With the `keep-id` option, a user in the container that has the same User ID as your host User ID, will have the same permissions as you.

During the creation of the **base image**, **Podman** invokes **Buildah**. **Buildah** does not allow User IDs to be shared between the host and the container in the same way that **Podman** does. So the **base image** is created without `keep-id`, then the first container created from the image, with `keep-id` enabled, triggers a remapping. Once that remapping is done, it is reused for each subsequent container.

The working directory is made available in the container by **mounting** it as a **volume**. The **Podman** option 
```
--volume "`pwd`":$(CONTAINER_WORKDIR):Z
```
takes the directory that `make` was started in as the host working directory, and **mounts** it at the location `$CONTAINER_WORKDIR`, normally set to `/mnt/redox`. The `:Z` at the end of the name indicates that the mounted directory should not be shared between simultaneous container instances. It is optional on some Linux distros, and not optional on others.

For our invocation of Podman, we set the PATH environment variable as an option to `podman run`. This is to avoid the need for our `make` command to run `.bashrc`, which would add extra complexity. The `ARCH`, `CONFIG_NAME` and `FILESYSTEM_CONFIG` variables are passed in the environment to allow you to override the values in `mk/config.mk`, e.g. by setting them on your `make` command line or by using `build.sh`.

We also set `PODMAN_BUILD=0` in the environment, to ensure that the instance of `make` running in the container knows not to invoke **Podman**. This overrides the value set in `mk/config.mk`.

In the **Containerfile**, we use as few `RUN` commands as possible, as **Podman** commits the image after each command. And we use `CMD` rather than `ENTRYPOINT` to allow us to specify the command to run as a list of arguments, rather than just a string to be processed as a command by the entrypoint shell.

Containers in our build process are run with `--rm` to ensure the container is discarded after each use. This prevents a proliferation of used containers. However, when you use `make container_clean`, you may notice multiple items being deleted. These are the partial images created as each `RUN` command is executed while building.

Container images and container data is normally stored in the directory `$HOME/.local/share/containers/storage`. The command
```sh
podman system reset
```
removes that directory in its entirety. However, the contents of any **volume** are left alone.
