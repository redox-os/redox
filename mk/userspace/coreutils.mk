coreutils: \
	filesystem/bin/basename \
	filesystem/bin/cat \
	filesystem/bin/chmod \
	filesystem/bin/clear \
	filesystem/bin/cp \
	filesystem/bin/cut \
	filesystem/bin/date \
	filesystem/bin/dd \
	filesystem/bin/df \
	filesystem/bin/du \
	filesystem/bin/echo \
	filesystem/bin/env \
	filesystem/bin/false \
	filesystem/bin/free \
	filesystem/bin/head \
	filesystem/bin/kill \
	filesystem/bin/ln \
	filesystem/bin/ls \
	filesystem/bin/mkdir \
	filesystem/bin/mv \
	filesystem/bin/printenv \
	filesystem/bin/ps \
	filesystem/bin/pwd \
	filesystem/bin/realpath \
	filesystem/bin/reset \
	filesystem/bin/rmdir \
	filesystem/bin/rm \
	filesystem/bin/seq \
	filesystem/bin/shutdown \
	filesystem/bin/sleep \
	filesystem/bin/sort \
	filesystem/bin/tail \
	filesystem/bin/tee \
	filesystem/bin/test \
	filesystem/bin/time \
	filesystem/bin/touch \
	filesystem/bin/true \
	filesystem/bin/wc \
	filesystem/bin/which \
	filesystem/bin/yes

filesystem/bin/%: programs/coreutils/Cargo.toml programs/coreutils/src/bin/%.rs $(BUILD)/libstd.rlib
	mkdir -p filesystem/bin
	$(CARGO) rustc --manifest-path $< --bin $* $(CARGOFLAGS) -o $@
	$(STRIP) $@
