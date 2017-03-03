netutils: \
	filesystem/bin/dhcpd \
	filesystem/bin/dns \
	filesystem/bin/httpd \
	filesystem/bin/irc \
	filesystem/bin/nc \
	filesystem/bin/ntp \
	filesystem/bin/telnetd \
	filesystem/bin/wget \
	filesystem/bin/ip

filesystem/bin/%: programs/netutils/Cargo.toml programs/netutils/src/%/**.rs $(BUILD)/libstd.rlib
	mkdir -p filesystem/bin
	$(CARGO) rustc --manifest-path $< --bin $* $(CARGOFLAGS) -o $@
	$(STRIP) $@
