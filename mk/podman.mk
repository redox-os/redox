
##############################################
# podman.mk - Use Podman to build components #
##############################################

# Configuration variables for running make in Podman
## Tag the podman image $IMAGE_TAG
IMAGE_TAG?=redox-base
## Working Directory in Podman
CONTAINER_WORKDIR?=/mnt/redox
## Podman command with its many arguments
PODMAN_VOLUMES?=--volume "`pwd`":$(CONTAINER_WORKDIR):Z
PODMAN_ENV?=--env PATH=/home/poduser/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin --env PODMAN_BUILD=0
PODMAN_OPTIONS?=--rm --workdir $(CONTAINER_WORKDIR) --userns keep-id --user `id -u` --interactive
PODMAN_RUN?=podman run $(PODMAN_OPTIONS) $(PODMAN_VOLUMES) $(PODMAN_ENV) $(IMAGE_TAG)

container_shell: build/container.tag
ifeq ($(PODMAN_BUILD),1)
	podman run $(PODMAN_VOLUMES) $(PODMAN_OPTIONS) $(PODMAN_ENV) --tty $(IMAGE_TAG) bash
else
	@echo PODMAN_BUILD=$(PODMAN_BUILD), please set it to 1 in mk/config.mk
endif

container_clean: FORCE
	rm -f build/container.tag
	@echo "For complete clean of images and containers, use \"podman system reset\""
	-podman image rm --force $(IMAGE_TAG) || true

container_touch: FORCE
ifeq ($(PODMAN_BUILD),1)
	@echo If you get an error, the image does not exist. Just do a normal make.
	podman image exists $(IMAGE_TAG)
	touch build/container.tag
else
	@echo PODMAN_BUILD=$(PODMAN_BUILD), container not required.
endif

## Must match the value of CONTAINER_TAG in config.mk
build/container.tag: $(CONTAINERFILE)
ifeq ($(PODMAN_BUILD),1)
	rm -f build/container.tag
	-podman image rm --force $(IMAGE_TAG) || true
	@echo "Building Podman image. This may take some time."
	sed s/_UID_/`id -u`/ $(CONTAINERFILE) | podman build --file - $(PODMAN_VOLUMES) --tag $(IMAGE_TAG)
	@echo "Mapping Podman user space. Please wait."
	$(PODMAN_RUN) echo "Podman ready!"
	mkdir -p build
	touch $@
else
	@echo PODMAN_BUILD=$(PODMAN_BUILD), container not required.
endif
