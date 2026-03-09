# Configuration file for the build system dependencies

# Don't check for dependencies if you will be using Podman
ifneq ($(PODMAN_BUILD),1)
# Don't check for dependencies if you will be using Hosted Redox
ifneq ($(HOSTED_REDOX),1)

# don't check for Rust and Cargo if building on a Nix system
ifneq ($(NIX_SHELL_BUILD),1)
ifeq ($(shell which rustup),)
$(error rustup not found, install from "https://rustup.rs/")
endif
endif

# don't check for compile tools, used internally when installing fstools on host
ifneq ($(SKIP_CHECK_TOOLS),1)
ifeq ($(shell which cbindgen),)
$(error cbindgen not found, install from crates.io or from your package manager)
endif
ifeq ($(shell which nasm),)
$(error nasm not found, install from your package manager)
endif
ifeq ($(shell which just),)
$(error 'just' not found, install from crates.io or from your package manager)
endif
endif

endif
endif
