# Configuration file for the build system dependencies

# Don't check for Rust/Cargo if you will be using Podman
ifneq ($(PODMAN_BUILD),1)

ifeq ($(shell which rustup),)
$(error rustup not found, install from "https://rustup.rs/")
endif

ifeq ($(shell which cbindgen),)
$(error cbindgen not found, install from crates.io or from your package manager)
endif

ifeq ($(shell which nasm),)
$(error nasm not found, install from your package manager)
endif

ifeq ($(shell which just),)
$(error just not found, install from crates.io or from your package manager)
endif

CARGO_CONFIG_VERSION=0.1.1
ifeq ($(shell env -u RUSTUP_TOOLCHAIN cargo install --list | grep '^cargo-config v$(CARGO_CONFIG_VERSION):$$'),)
$(error cargo-config $(CARGO_CONFIG_VERSION) not found, run "cargo install --force --version $(CARGO_CONFIG_VERSION) cargo-config")
endif

endif
