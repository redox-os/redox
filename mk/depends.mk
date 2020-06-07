# Dependencies

XARGO_VERSION=0.3.20
ifeq ($(shell cargo install --list | grep '^xargo v$(XARGO_VERSION):$$'),)
$(error xargo $(XARGO_VERSION) not found, run "cargo install --force --version $(XARGO_VERSION) xargo")
endif
