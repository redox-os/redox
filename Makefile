ARCH?=x86_64

# Kernel variables
KTARGET=$(ARCH)-unknown-none
KBUILD=build/kernel
KRUSTC=./krustc.sh
KRUSTCFLAGS=--target $(KTARGET).json -C soft-float
KCARGO=RUSTC="$(KRUSTC)" cargo
KCARGOFLAGS=--target $(KTARGET).json -- -C soft-float

# Userspace variables
TARGET=$(ARCH)-unknown-redox
BUILD=build/userspace
RUSTC=./rustc.sh
RUSTCFLAGS=--target $(TARGET).json -C soft-float --cfg redox
CARGO=RUSTC="$(RUSTC)" cargo
CARGOFLAGS=--target $(TARGET).json -- -C soft-float --cfg redox

# Default targets
.PHONY: all clean qemu bochs FORCE

all: $(KBUILD)/harddrive.bin

clean:
	cargo clean
	cargo clean --manifest-path libstd/Cargo.toml
	cargo clean --manifest-path init/Cargo.toml
	cargo clean --manifest-path ion/Cargo.toml
	cargo clean --manifest-path drivers/atkbd/Cargo.toml
	cargo clean --manifest-path drivers/pcid/Cargo.toml
	rm -rf build

FORCE:

# Emulation
QEMU=qemu-system-$(ARCH)
QEMUFLAGS=-serial mon:stdio -d guest_errors
ifeq ($(ARCH),arm)
	LD=$(ARCH)-none-eabi-ld
	QEMUFLAGS+=-cpu arm1176 -machine integratorcp
	QEMUFLAGS+=-nographic

build/%.list: build/%
	$(ARCH)-none-eabi-objdump -C -D $< > $@

$(KBUILD)/harddrive.bin: $(KBUILD)/kernel
	cp $< $@

qemu: $(KBUILD)/harddrive.bin
	$(QEMU) $(QEMUFLAGS) -kernel $<
else
	LD=ld
	QEMUFLAGS+=-machine q35 -smp 4
	ifeq ($(kvm),yes)
		QEMUFLAGS+=-enable-kvm -cpu host
	endif
	ifeq ($(vga),no)
		QEMUFLAGS+=-nographic -vga none
	endif
	#,int,pcall
	#-device intel-iommu

	UNAME := $(shell uname)
	ifeq ($(UNAME),Darwin)
		LD=$(ARCH)-elf-ld
	endif

build/%.list: build/%
	objdump -C -M intel -D $< > $@

$(KBUILD)/harddrive.bin: $(KBUILD)/kernel bootloader/$(ARCH)/**
	nasm -f bin -o $@ -D ARCH_$(ARCH) -ibootloader/$(ARCH)/ bootloader/$(ARCH)/harddrive.asm

qemu: $(KBUILD)/harddrive.bin
	$(QEMU) $(QEMUFLAGS) -drive file=$<,format=raw,index=0,media=disk
endif

bochs: $(KBUILD)/harddrive.bin
	bochs -f bochs.$(ARCH)

# Kernel recipes
$(KBUILD)/libcore.rlib: rust/src/libcore/lib.rs
	mkdir -p $(KBUILD)
	$(KRUSTC) $(KRUSTCFLAGS) -o $@ $<

$(KBUILD)/librand.rlib: rust/src/librand/lib.rs $(KBUILD)/libcore.rlib
	$(KRUSTC) $(KRUSTCFLAGS) -o $@ $<

$(KBUILD)/liballoc.rlib: rust/src/liballoc/lib.rs $(KBUILD)/libcore.rlib
	$(KRUSTC) $(KRUSTCFLAGS) -o $@ $<

$(KBUILD)/librustc_unicode.rlib: rust/src/librustc_unicode/lib.rs $(KBUILD)/libcore.rlib
	$(KRUSTC) $(KRUSTCFLAGS) -o $@ $<

$(KBUILD)/libcollections.rlib: rust/src/libcollections/lib.rs $(KBUILD)/libcore.rlib $(KBUILD)/liballoc.rlib $(KBUILD)/librustc_unicode.rlib
	$(KRUSTC) $(KRUSTCFLAGS) -o $@ $<

$(KBUILD)/libkernel.a: kernel/** $(KBUILD)/libcore.rlib $(KBUILD)/liballoc.rlib $(KBUILD)/libcollections.rlib $(BUILD)/initfs.rs FORCE
	$(KCARGO) rustc $(KCARGOFLAGS) -o $@

$(KBUILD)/kernel: $(KBUILD)/libkernel.a
	$(LD) --gc-sections -z max-page-size=0x1000 -T arch/$(ARCH)/src/linker.ld -o $@ $<

# Userspace recipes
$(BUILD)/libcore.rlib: rust/src/libcore/lib.rs
	mkdir -p $(BUILD)
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

$(BUILD)/librand.rlib: rust/src/librand/lib.rs $(BUILD)/libcore.rlib
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

$(BUILD)/liballoc.rlib: rust/src/liballoc/lib.rs $(BUILD)/libcore.rlib
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

$(BUILD)/librustc_unicode.rlib: rust/src/librustc_unicode/lib.rs $(BUILD)/libcore.rlib
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

$(BUILD)/libcollections.rlib: rust/src/libcollections/lib.rs $(BUILD)/libcore.rlib $(BUILD)/liballoc.rlib $(BUILD)/librustc_unicode.rlib
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

$(BUILD)/libstd.rlib: libstd/Cargo.toml libstd/src/** $(BUILD)/libcore.rlib $(BUILD)/liballoc.rlib $(BUILD)/librustc_unicode.rlib $(BUILD)/libcollections.rlib $(BUILD)/librand.rlib
	$(CARGO) rustc --verbose --manifest-path $< $(CARGOFLAGS) -o $@
	cp libstd/target/$(TARGET)/debug/deps/*.rlib $(BUILD)

$(BUILD)/init: init/Cargo.toml init/src/*.rs $(BUILD)/libstd.rlib
	$(CARGO) rustc --manifest-path $< $(CARGOFLAGS) -o $@
	strip $@

$(BUILD)/ion: ion/Cargo.toml ion/src/*.rs $(BUILD)/libstd.rlib
	$(CARGO) rustc --manifest-path $< $(CARGOFLAGS) -o $@
	strip $@

$(BUILD)/pcid: drivers/pcid/Cargo.toml drivers/pcid/src/** $(BUILD)/libstd.rlib
	$(CARGO) rustc --manifest-path $< $(CARGOFLAGS) -o $@
	strip $@

$(BUILD)/ps2d: drivers/ps2d/Cargo.toml drivers/ps2d/src/** $(BUILD)/libstd.rlib
	$(CARGO) rustc --manifest-path $< $(CARGOFLAGS) -o $@
	strip $@

$(BUILD)/initfs.rs: $(BUILD)/init $(BUILD)/ion $(BUILD)/pcid $(BUILD)/ps2d
