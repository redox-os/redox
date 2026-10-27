# Configuration file for Cloud Hypervisor
#
#   make ch                  # boot $(DISK), serial console on this terminal
#   make ch share=...        # also share host directories, see mk/virtiofs.mk
#   make ch net=tap          # use the tap device $(CH_TAP) instead of passt
#   make ch net=no           # no network device
#   make ch-shutdown         # stop the VM, from another terminal
#
# x86_64 only. The guest boots through the CLOUDHV.fd UEFI firmware
# (https://github.com/cloud-hypervisor/edk2/releases), taken from the repo
# root unless CH_FIRMWARE is set. The disks are virtio-blk.
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

CH?=$(shell command -v cloud-hypervisor || command -v ch)
CH_REMOTE?=ch-remote
CH_FIRMWARE?=$(wildcard CLOUDHV.fd)
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

.PHONY: ch ch-check ch-deps ch-shutdown

ch-check:
	$(if $(filter x86_64,$(ARCH)),,$(error Cloud Hypervisor: only ARCH=x86_64 is supported))
	$(if $(CH),,$(error cloud-hypervisor not found. Get cloud-hypervisor-static from https://github.com/cloud-hypervisor/cloud-hypervisor/releases and put it in PATH or set CH=/path/to/cloud-hypervisor))
	$(if $(wildcard $(CH_FIRMWARE)),,$(error CLOUDHV.fd not found. Get it from https://github.com/cloud-hypervisor/edk2/releases and put it in the repo root or set CH_FIRMWARE=/path/to/CLOUDHV.fd))
	$(if $(filter tap,$(net)),$(if $(ch_tap_found),,$(error no tap device $(CH_TAP). Create it with scripts/ch-tap-setup.sh or leave out net=tap to use passt)))

ch-deps: $(DISK) $(EXTRA_DISK)

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

ch-shutdown:
	$(if $(shell command -v $(CH_REMOTE)),,$(error ch-remote not found. Get ch-remote-static from https://github.com/cloud-hypervisor/cloud-hypervisor/releases and put it in PATH or set CH_REMOTE=/path/to/ch-remote))
# shutdown alone would leave Cloud Hypervisor running.
	$(CH_REMOTE) --api-socket "$(CH_API_SOCK)" shutdown-vmm
