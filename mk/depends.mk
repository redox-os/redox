# Dependencies

CARGO_CONFIG_VERSION=0.1.1
ifeq ($(shell cargo install --list | grep '^cargo-config v$(CARGO_CONFIG_VERSION):$$'),)
$(error cargo-config $(CARGO_CONFIG_VERSION) not found, run "cargo install --force --version $(CARGO_CONFIG_VERSION) cargo-config")
endif

XARGO_VERSION=0.3.20
ifeq ($(shell cargo install --list | grep '^xargo v$(XARGO_VERSION):$$'),)
$(error xargo $(XARGO_VERSION) not found, run "cargo install --force --version $(XARGO_VERSION) xargo")
endif
