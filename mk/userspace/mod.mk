userspace: \
	drivers \
	coreutils \
	extrautils \
	games \
	installer \
	ion \
	netutils \
	orbutils \
	pkgutils \
	userutils \
	schemes \
	filesystem/bin/acid \
	filesystem/bin/contain \
	filesystem/bin/redox_installer \
	filesystem/bin/smith \
	filesystem/bin/tar

include mk/userspace/binutils.mk
include mk/userspace/coreutils.mk
include mk/userspace/drivers.mk
include mk/userspace/extrautils.mk
include mk/userspace/games.mk
include mk/userspace/installer.mk
include mk/userspace/ion.mk
include mk/userspace/netutils.mk
include mk/userspace/orbutils.mk
include mk/userspace/pkgutils.mk
include mk/userspace/schemes.mk
include mk/userspace/userutils.mk

$(BUILD)/libstd.rlib: rust/src/libstd/Cargo.toml rust/src/libstd/**
	mkdir -p $(BUILD)
	$(CARGO) rustc --manifest-path $< --features "panic-unwind" $(CARGOFLAGS) -L native=libc-artifacts/usr/lib -o $@
	cp rust/src/target/$(TARGET)/release/deps/*.rlib $(BUILD)

$(BUILD)/libtest.rlib: rust/src/libtest/Cargo.toml rust/src/libtest/** $(BUILD)/libstd.rlib
	mkdir -p $(BUILD)
	$(CARGO) rustc --manifest-path $< $(CARGOFLAGS) -L native=libc-artifacts/usr/lib -o $@
	cp rust/src/target/$(TARGET)/release/deps/*.rlib $(BUILD)

filesystem/bin/%: programs/%/Cargo.toml programs/%/src/** $(BUILD)/libstd.rlib
	mkdir -p filesystem/bin
	$(CARGO) rustc --manifest-path $< --bin $* $(CARGOFLAGS) -o $@
	$(STRIP) $@
