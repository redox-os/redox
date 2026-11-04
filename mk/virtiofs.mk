# Share host directories with the guest through virtio-fs (QEMU and Cloud Hypervisor).
#   make qemu share=/path/to/dir                     # one share, tag is "host" by default
#   make qemu share="tag=/path/to/dir"               # one share, tag is specified as "tag"
#   make qemu share=/path/a share_tag=tag            # one share, tag is specified as "tag"
#   make qemu share="tag_1=/path/a tag_2=/path/b"    # several shares: tag=dir ...
#   make qemu share=redox                            # this build tree, tag "redox"
#   make qemu share="redox=/other/path"              # ordinary share, tag "redox"
#   make qemu share=/path/to/dir:ro                  # read-only, also tag=dir:ro and redox:ro
#   make ch share=...                                # the same, on Cloud Hypervisor
#
# Each share gets its own host-side virtiofsd daemon (started and stopped by
# scripts/virtiofs-run.sh), a vhost-user-fs device, and a virtio-fsd scheme
# process on Redox. In the guest, a share appears at /scheme/virtio-fsd.<tag>.
# The emulator flags for each share are added in mk/qemu.mk and mk/ch.mk.
# Shared directories must exist. The guest cannot change anything in a :ro share.
#
# WARNING: share=redox exposes the whole build tree to the guest, including
# the harddrive.img Redox is booted from, .git and .config. Writing to them
# from Redox can corrupt the build system. share=redox:ro avoids that.
#
# virtiofsd runs on the host. The first one found is used: VIRTIOFSD, then
# virtiofsd in PATH, /usr/libexec/virtiofsd or /usr/lib/qemu/virtiofsd. When
# there is none, x86_64 Linux hosts download a static build into
# build/virtiofsd/$(HOST_ARCH) (make virtiofsd-fetch does only that). Other
# hosts must install virtiofsd, e.g. from their distribution (the package is
# virtiofsd on Fedora, Arch and recent Debian/Ubuntu).
#
# TODO: the download is the CI build of upstream's main branch, so it is
# neither pinned nor checked: virtiofsd releases do not ship binaries or
# checksums yet (their CI only builds main, and those artifacts expire). Once
# they do, pin it to a release and check its sha256 like Cloud Hypervisor in
# mk/ch.mk.

share?=
share_tag?=host
share_sockdir?=$(BUILD)
VIRTIOFS_RUN=
VIRTIOFS_DEPS=

VIRTIOFSD_URL?=https://gitlab.com/virtio-fs/virtiofsd/-/jobs/artifacts/main/raw/target/x86_64-unknown-linux-musl/release/virtiofsd?job=publish
VIRTIOFSD_DOWNLOAD=build/virtiofsd/$(HOST_ARCH)/virtiofsd

ifneq ($(strip $(share)),)
	share_norm:=$(foreach s,$(share),$(if $(filter redox redox:ro,$(s)),redox=$(ROOT)$(if $(filter %:ro,$(s)),:ro),$(s)))
	ifeq ($(words $(share_norm)),1)
		share_list:=$(if $(findstring =,$(share_norm)),$(share_norm),$(share_tag)=$(share_norm))
	else
		share_list:=$(share_norm)
		share_bare:=$(foreach s,$(share_list),$(if $(findstring =,$(s)),,$(s)))
		ifneq ($(strip $(share_bare)),)
			# Keep the next line free of leading tabs: make would read it as a recipe.
$(error share: every entry must be tag=dir when sharing several directories (bad: $(share_bare)); directories containing spaces are not supported)
		endif
	endif
	share_idx:=$(shell seq 1 $(words $(share_list)))
	vfs_entry=$(word $(1),$(share_list))
	vfs_tag=$(firstword $(subst =, ,$(call vfs_entry,$(1))))
	vfs_sock=$(share_sockdir)/virtiofsd-$(1).sock

	ifeq ($(strip $(VIRTIOFSD)),)
		VIRTIOFSD:=$(firstword $(shell command -v virtiofsd 2>/dev/null) $(wildcard /usr/libexec/virtiofsd /usr/lib/qemu/virtiofsd) $(VIRTIOFSD_DOWNLOAD))
	endif
	VIRTIOFS_DEPS=$(filter $(VIRTIOFSD_DOWNLOAD),$(VIRTIOFSD))

	VIRTIOFS_RUN=VIRTIOFSD="$(VIRTIOFSD)" scripts/virtiofs-run.sh $(foreach i,$(share_idx),--share "$(call vfs_sock,$(i))" "$(call vfs_entry,$(i))") --
endif

.PHONY: virtiofsd-fetch

virtiofsd-fetch: $(VIRTIOFSD_DOWNLOAD)

# Goes through $@.partial, so $@ only ever holds a complete ELF binary.
$(VIRTIOFSD_DOWNLOAD):
	$(if $(and $(filter x86_64,$(HOST_ARCH)),$(filter Linux,$(shell uname -s))),,$(error virtiofsd: no prebuilt virtiofsd for this $(shell uname -s) $(HOST_ARCH) host. Install virtiofsd (e.g. dnf install virtiofsd or pacman -S virtiofsd) or set VIRTIOFSD=/path/to/virtiofsd))
	@echo "virtiofsd: downloading the CI build of upstream's main branch (not pinned or checked)"
	mkdir -p "$(@D)"
	curl -fL --retry 3 --retry-delay 1 -o "$@.partial" "$(VIRTIOFSD_URL)"
	@if [ "$$(head -c 4 "$@.partial")" != "$$(printf '\177ELF')" ]; then \
		rm -f "$@.partial"; \
		echo "virtiofsd: the download is not an ELF binary (see $(VIRTIOFSD_URL))" >&2; \
		exit 1; \
	fi
	chmod +x "$@.partial"
	mv "$@.partial" "$@"
