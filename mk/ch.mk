# Configuration file for Cloud Hypervisor
#
#   make ch                  # boot $(DISK), serial console on this terminal
#   make ch share=...        # also share host directories, see mk/virtiofs.mk
#   make ch net=tap          # use the tap device $(CH_TAP) instead of passt
#   make ch net=no           # no network device
#   make ch-shutdown         # stop the VM, from another terminal
#   make ch-fetch            # only download Cloud Hypervisor and its firmware
#
# x86_64 only. The guest boots through the CLOUDHV.fd UEFI firmware
# (https://github.com/cloud-hypervisor/edk2/releases). The disks are virtio-blk.
#
# Cloud Hypervisor (cloud-hypervisor-static, ch-remote-static) and the firmware
# are pinned below. They are downloaded on first use into
# build/cloud-hypervisor/$(ARCH) and checked against their sha256. Set CH,
# CH_REMOTE or CH_FIRMWARE to use other files, e.g. CH=cloud-hypervisor for
# the one in PATH.
#
# The serial console puts this terminal in raw mode, so Ctrl-C goes to Redox.
# Redox cannot power off Cloud Hypervisor by itself yet: use make ch-shutdown.
#
# The network is passt by default: a user-mode network stack, started by
# scripts/passt-run.sh, that answers the guest's DHCP and DNS and carries its
# traffic through ordinary host sockets. No tap, root or firewall changes are
# needed. The guest gets the host's address, and nothing reaches it from
# outside unless forwarded. CH_PASST_FLAGS passes options to passt, e.g.
# "-t 8022:22" (host port 8022 to guest port 22) or
# "--pcap $XDG_RUNTIME_DIR/redox.pcap" (capture the traffic).
#
# net=tap uses the tap device $(CH_TAP) instead: a real link that the host
# can reach on any port and watch with tcpdump. Creating it needs root:
# scripts/ch-tap-setup.sh (firewalld only) creates it and makes the host
# the guest's DHCP server and NAT gateway.

# Pinned releases. To update one, change its version and the sha256 of its
# assets, which GitHub shows next to each file on the release page. Every
# version is downloaded into its own directory.
CH_VERSION=v54.0
CH_FIRMWARE_VERSION=ch-97eeb7b09
CH_URL=https://github.com/cloud-hypervisor/cloud-hypervisor/releases/download/$(CH_VERSION)
CH_FIRMWARE_URL=https://github.com/cloud-hypervisor/edk2/releases/download/$(CH_FIRMWARE_VERSION)
CH_DIR=build/cloud-hypervisor/$(ARCH)

# Release assets and their sha256, per ARCH
CH_ASSET_x86_64=cloud-hypervisor-static
CH_SHA256_x86_64=4f4f98a3d661a40f3e5272627cc895a1b2e525f2f6bedf5f895f9cd21ba5c65b
CH_REMOTE_ASSET_x86_64=ch-remote-static
CH_REMOTE_SHA256_x86_64=acd5be240d5c73373effbe818908fc03e313ec8486237904bff11d93b71b1c83
CH_FIRMWARE_ASSET_x86_64=CLOUDHV.fd
CH_FIRMWARE_SHA256_x86_64=dc2fc8f0e43b96712d9fccc52a3a590769606412b3e1bc911d217addd3bef624

ifneq ($(CH_ASSET_$(ARCH)),)
	CH_PINNED=$(CH_DIR)/$(CH_VERSION)/$(CH_ASSET_$(ARCH))
	CH_REMOTE_PINNED=$(CH_DIR)/$(CH_VERSION)/$(CH_REMOTE_ASSET_$(ARCH))
	CH_FIRMWARE_PINNED=$(CH_DIR)/edk2-$(CH_FIRMWARE_VERSION)/$(CH_FIRMWARE_ASSET_$(ARCH))
endif

CH?=$(CH_PINNED)
CH_REMOTE?=$(CH_REMOTE_PINNED)
CH_FIRMWARE?=$(CH_FIRMWARE_PINNED)
# Only the pinned files in use are downloaded.
CH_DOWNLOADS=$(filter $(CH_PINNED) $(CH_REMOTE_PINNED) $(CH_FIRMWARE_PINNED),$(CH) $(CH_REMOTE) $(CH_FIRMWARE))

CH_SMP?=4
CH_MEM?=2048
CH_API_SOCK?=$(BUILD)/ch.sock
CH_LOG?=$(BUILD)/ch.log
CH_TAP?=redox0
CH_MAC?=52:54:00:12:34:56
# Not in $(BUILD): SELinux limits passt to $XDG_RUNTIME_DIR and /tmp.
CH_PASST_SOCK?=$(or $(XDG_RUNTIME_DIR),/tmp)/redox-passt-$(ARCH)-$(CONFIG_NAME).sock
CH_PASST_FLAGS?=
CHFLAGS?=-v --log-file $(CH_LOG)

CHFLAGS+=--cpus boot=$(CH_SMP)
CHFLAGS+=--firmware $(CH_FIRMWARE)
CHFLAGS+=--serial tty --console off
CHFLAGS+=--api-socket path=$(CH_API_SOCK)
CHFLAGS+= \
	--disk path=$(DISK),image_type=raw,serial=NVME_SERIAL \
	--disk path=$(EXTRA_DISK),image_type=raw,serial=NVME_EXTRA

PASST_RUN=

ifeq ($(net),tap)
	# The Redox virtio-net driver only acknowledges VIRTIO_NET_F_MAC at the moment.
	CHFLAGS+=--net tap=$(CH_TAP),mac=$(CH_MAC),offload_tso=off,offload_ufo=off,offload_csum=off
else ifneq ($(net),no)
	CHFLAGS+=--net vhost_user=true,socket=$(CH_PASST_SOCK),mac=$(CH_MAC)
	PASST_RUN=scripts/passt-run.sh "$(CH_PASST_SOCK)" $(CH_PASST_FLAGS) --
endif

ch_tap_found=$(shell ip link show "$(CH_TAP)" >/dev/null 2>&1 && echo yes)

# vhost-user backends (virtiofsd, passt) need shared guest RAM.
ifneq ($(strip $(share))$(PASST_RUN),)
	CHFLAGS+=--memory size=$(CH_MEM)M,shared=on
else
	CHFLAGS+=--memory size=$(CH_MEM)M
endif

ifneq ($(strip $(share)),)
	CHFLAGS+=$(foreach i,$(share_idx),--fs tag=$(call vfs_tag,$(i)),socket=$(call vfs_sock,$(i)))
endif

# $(1): CH, CH_REMOTE or CH_FIRMWARE. When it is set to something other than
# the pinned download, it must be an existing file or a command in PATH.
ch_check_set=$(if $(filter-out $($(1)_PINNED),$($(1))),$(if $(or $(wildcard $($(1))),$(shell command -v $($(1)))),,$(error $(1)=$($(1)) not found. Leave $(1) unset to use the pinned download)))

.PHONY: ch ch-check ch-deps ch-fetch ch-shutdown

ch-check:
	$(if $(filter x86_64,$(ARCH)),,$(error Cloud Hypervisor: only ARCH=x86_64 is supported))
	$(if $(filter $(HOST_ARCH),$(ARCH)),,$(error Cloud Hypervisor: ARCH=$(ARCH) cannot run on this $(HOST_ARCH) host))
	$(call ch_check_set,CH)
	$(call ch_check_set,CH_REMOTE)
	$(call ch_check_set,CH_FIRMWARE)
	$(if $(filter tap,$(net)),$(if $(ch_tap_found),,$(error no tap device $(CH_TAP). Create it with scripts/ch-tap-setup.sh or leave out net=tap to use passt)))

ch-deps: $(DISK) $(EXTRA_DISK) $(CH_DOWNLOADS)

ch-fetch: ch-check $(CH_DOWNLOADS)

ch: ch-check ch-deps
ifneq ($(wildcard $(MOUNT_DIR)),)
	@echo "Disk is mounted. Unmounting the disk..."
	$(MAKE) unmount
endif
	@if $(CH_REMOTE) --api-socket "$(CH_API_SOCK)" ping >/dev/null 2>&1; then \
		echo "ch: a VM is already running on $(CH_API_SOCK); stop it with make ch-shutdown" >&2; \
		exit 1; \
	fi
# A leftover socket file makes Cloud Hypervisor fail with "Address in use".
	rm -f "$(CH_API_SOCK)"
	$(VIRTIOFS_RUN) $(PASST_RUN) $(CH) $(CHFLAGS)

ch-shutdown: $(filter $(CH_REMOTE_PINNED),$(CH_REMOTE))
	$(call ch_check_set,CH_REMOTE)
# shutdown alone would leave Cloud Hypervisor running.
	$(CH_REMOTE) --api-socket "$(CH_API_SOCK)" shutdown-vmm

# Downloads $(1) to $@ if its sha256 is $(2), and sets mode $(3) if given.
# It goes through $@.partial, so $@ only ever holds a checked download.
define ch_download
	mkdir -p "$(@D)"
	curl -fL --retry 3 --retry-delay 1 -o "$@.partial" "$(1)"
	echo "$(2)  $@.partial" | sha256sum --check --quiet - \
		|| { rm -f "$@.partial"; echo "ch: $(1) does not match the pinned sha256 $(2)" >&2; exit 1; }
	$(if $(3),chmod $(3) "$@.partial")
	mv "$@.partial" "$@"
endef

ifneq ($(CH_PINNED),)
$(CH_PINNED):
	$(call ch_download,$(CH_URL)/$(@F),$(CH_SHA256_$(ARCH)),+x)

$(CH_REMOTE_PINNED):
	$(call ch_download,$(CH_URL)/$(@F),$(CH_REMOTE_SHA256_$(ARCH)),+x)

$(CH_FIRMWARE_PINNED):
	$(call ch_download,$(CH_FIRMWARE_URL)/$(@F),$(CH_FIRMWARE_SHA256_$(ARCH)))
endif
