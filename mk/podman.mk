# Configuration file of the Podman commands

# Configuration variables for running make in Podman
## Tag the podman image $IMAGE_TAG
IMAGE_TAG?=redox-base
## Working Directory in Podman
CONTAINER_WORKDIR?=/mnt/redox

## Flag passed to the Podman volumes. :Z can be used only with SELinux
USE_SELINUX=1
ifeq ($(USE_SELINUX),1)
PODMAN_VOLUME_FLAG=:Z
else
PODMAN_VOLUME_FLAG=
endif

## Podman Home Directory
PODMAN_HOME?=$(ROOT)/build/podman
## Podman downloaded repo binaries cache
PODMAN_PKG_TMP?=$(ROOT)/build/$(ARCH)/pkg-download
## Podman command with its many arguments
PODMAN_VOLUMES?=--volume $(ROOT):$(CONTAINER_WORKDIR)$(PODMAN_VOLUME_FLAG) --volume $(PODMAN_HOME):/home$(PODMAN_VOLUME_FLAG) --volume $(PODMAN_PKG_TMP):/tmp/pkg_download$(PODMAN_VOLUME_FLAG)
PODMAN_ENV?=--env PATH=/home/poduser/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin --env PODMAN_BUILD=0
PODMAN_CONFIG?=--env ARCH=$(ARCH) --env BOARD=$(BOARD) --env CONFIG_NAME=$(CONFIG_NAME) --env FILESYSTEM_CONFIG=$(FILESYSTEM_CONFIG)
PODMAN_OPTIONS?=--rm --workdir $(CONTAINER_WORKDIR) --userns keep-id --user `id -u` --interactive --tty --env TERM=$(TERM)
PODMAN_RUN?=podman run $(PODMAN_OPTIONS) $(PODMAN_VOLUMES) $(PODMAN_ENV) $(PODMAN_CONFIG) $(IMAGE_TAG)

container_shell: build/container.tag
ifeq ($(PODMAN_BUILD),1)
# TODO: how to ensure to do this anytime when $(PODMAN_RUN) is invoked?
	mkdir -p $(PODMAN_PKG_TMP) && touch $(PODMAN_PKG_TMP)/prefer_cache
	podman run $(PODMAN_VOLUMES) $(PODMAN_OPTIONS) $(PODMAN_ENV) --tty $(IMAGE_TAG) bash
else
	@echo PODMAN_BUILD=$(PODMAN_BUILD), please set it to 1 in mk/config.mk
endif

container_su: FORCE
	podman exec --user=0 --latest --interactive --tty bash

container_clean: FORCE
	rm -f build/container.tag
	@echo "If podman dir cannot be removed, remove with \"sudo rm\"."
	-rm -rf $(PODMAN_HOME) || true
	@echo "For complete clean of images and containers, use \"podman system reset\""
	-podman image rm --force $(IMAGE_TAG) || true

container_touch: FORCE
ifeq ($(PODMAN_BUILD),1)
	rm -f build/container.tag
	podman image exists $(IMAGE_TAG) || (echo "Image does not exist, it will be rebuilt during normal make."; exit 1)
	touch build/container.tag
else
	@echo PODMAN_BUILD=$(PODMAN_BUILD), container not required.
endif

container_kill: FORCE
	podman kill --latest --signal SIGKILL

## Must match the value of CONTAINER_TAG in config.mk
build/container.tag: $(CONTAINERFILE)
ifeq ($(PODMAN_BUILD),1)
	rm -f build/container.tag
	@echo "If podman_home dir cannot be removed, remove with \"sudo rm\"."
	-rm -rf $(PODMAN_HOME) || true
	-podman image rm --force $(IMAGE_TAG) || true
	mkdir -p $(PODMAN_HOME) $(PODMAN_PKG_TMP)
	touch $(PODMAN_PKG_TMP)/prefer_cache
	@echo "Building Podman image. This may take some time."
	sed s/_UID_/`id -u`/ $(CONTAINERFILE) | podman build --file - $(PODMAN_VOLUMES) --tag $(IMAGE_TAG)
	@echo "Mapping Podman user space. Please wait."
	$(PODMAN_RUN) bash -e podman/rustinstall.sh
	mkdir -p build
	touch $@
	@echo "Podman ready!"
else
	@echo PODMAN_BUILD=$(PODMAN_BUILD), container not required.
endif

KERNEL_PATH := cookbook/recipes/core/kernel
KERNEL_PATH_SOURCE := $(ROOT)/$(KERNEL_PATH)/source
KERNEL_PATH_TARGET := $(ROOT)/$(KERNEL_PATH)/target/$(TARGET)

# TODO: make this work using `make debug.kernel` and remove this
kernel_debugger:
	@echo "Building and running gdbgui container..."
	podman build -t redox-kernel-debug - < $(ROOT)/podman/redox-gdb-containerfile
	podman run --rm -p 5000:5000 -it --name redox-gdb \
		-v "$(KERNEL_PATH_TARGET)/build/kernel.sym:/kernel.sym" \
		-v "$(KERNEL_PATH_SOURCE)/src:/src" \
		redox-kernel-debug --gdb-cmd "gdb -ex 'set confirm off' \
			-ex 'add-symbol-file /kernel.sym' \
			-ex 'target remote host.containers.internal:1234'"
