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

share?=
share_tag?=host
share_sockdir?=$(BUILD)
VIRTIOFS_RUN=

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

	VIRTIOFS_RUN=scripts/virtiofs-run.sh $(foreach i,$(share_idx),--share "$(call vfs_sock,$(i))" "$(call vfs_entry,$(i))") --
endif
