# Makefile.am:
#   Source file for Makefile.in (and hence Makefile)
#
# Makefile.am need only be changed on a major version number
# change (e.g. libpng12 --> libpng14).  In that case seach
# this file for every instance of the old base name (libpng12)
# and change to the new one (libpng14), then change the
# -version-number settings below so that the new values have
# the correct major part (first field).

PNGLIB_BASENAME= libpng@PNGLIB_MAJOR@@PNGLIB_MINOR@

# libpng does not follow GNU file name conventions
AUTOMAKE_OPTIONS = foreign

# test programs - run on make check, make distcheck
check_PROGRAMS= pngtest
pngtest_SOURCES = pngtest.c
pngtest_LDADD = libpng12.la
TESTS = test-pngtest.sh
TESTS_ENVIRONMENT= srcdir=$(srcdir)

# man pages
dist_man_MANS= libpng.3 libpngpf.3 png.5

# generate the -config scripts if required
binconfigs= libpng12-config
EXTRA_SCRIPTS= libpng-config libpng12-config
bin_SCRIPTS= @binconfigs@

# rules to build libpng, only build the old library on request
lib_LTLIBRARIES=libpng12.la @compatlib@
EXTRA_LTLIBRARIES= libpng.la
libpng12_la_SOURCES = png.c pngset.c pngget.c pngrutil.c pngtrans.c pngwutil.c \
	pngread.c pngrio.c pngwio.c pngwrite.c pngrtran.c \
	pngwtran.c pngmem.c pngerror.c pngpread.c \
	png.h pngconf.h
libpng_la_SOURCES = $(libpng12_la_SOURCES)

libpng_la_CPPFLAGS = @LIBPNG_DEFINES@
libpng12_la_CPPFLAGS = @LIBPNG_DEFINES@

# MAJOR UPGRADE: the version-number settings below must be changed.
libpng12_la_LDFLAGS = -no-undefined -export-dynamic \
	-version-number 0:@PNGLIB_RELEASE@:0
# -rpath is needed as automake doesn't know the directory
libpng_la_LDFLAGS = -rpath '$(libdir)' -no-undefined -export-dynamic \
	-version-number 3:@PNGLIB_RELEASE@:0

if HAVE_LD_VERSION_SCRIPT
  # Versioned symbols and restricted exports
  libpng12_la_LDFLAGS += -Wl,--version-script=libpng.vers
  libpng12_la_DEPENDENCIES = libpng.vers
else
  # Only restricted exports when possible
  libpng12_la_LDFLAGS += -export-symbols libpng.sym
  libpng12_la_DEPENDENCIES = libpng.sym
endif
libpng_la_DEPENDENCIES = $(libpng12_la_DEPENDENCIES)

# Avoid depending upon Character Ranges.
AN = '_abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789'

#distribute headers in /usr/include/libpng/*
pkgincludedir= $(includedir)/$(PNGLIB_BASENAME)
pkginclude_HEADERS= png.h pngconf.h

# pkg-config stuff, note that libpng.pc is always required in order
# to get the correct library
pkgconfigdir = @pkgconfigdir@
pkgconfig_DATA = libpng12.pc

#extra source distribution files.
EXTRA_DIST= \
	ANNOUNCE CHANGES INSTALL KNOWNBUG LICENSE README TODO Y2KINFO \
	pngtest.png pngbar.png pngnow.png pngbar.jpg autogen.sh \
        CMakeLists.txt \
	${srcdir}/projects/cbuilder5/* \
	${srcdir}/projects/beos/* \
	${srcdir}/projects/visualc6/* \
	${srcdir}/projects/visualc71/* \
	${srcdir}/projects/wince.txt \
	${srcdir}/projects/netware.txt \
	${srcdir}/projects/xcode/* \
	${srcdir}/scripts/* \
	${srcdir}/contrib/gregbook/* \
	${srcdir}/contrib/pngminim/* \
	${srcdir}/contrib/pngminus/* \
	${srcdir}/contrib/pngsuite/* \
	${srcdir}/contrib/visupng/* \
	$(TESTS) \
	example.c libpng-1.2.52.txt pnggccrd.c pngvcrd.c

CLEANFILES= pngout.png libpng12.pc libpng12-config libpng.vers \
libpng.sym

MAINTAINERCLEANFILES = Makefile.in aclocal.m4 config.guess config.h.in \
config.sub configure depcomp install-sh ltmain.sh missing

$(PNGLIB_BASENAME).pc: libpng.pc
	cp libpng.pc $@

$(PNGLIB_BASENAME)-config: libpng-config
	cp libpng-config $@

libpng.sym: png.h pngconf.h
	rm -f $@ $@.new
	$(CPP) @LIBPNG_DEFINES@ $(CPPFLAGS) -DPNG_BUILDSYMS $(srcdir)/png.h | \
		$(SED) -n -e \
		's|^.*PNG_FUNCTION_EXPORT[ 	]*\([$(AN)]*\).*$$|$(SYMBOL_PREFIX)\1|p' \
		-e 's|^.*PNG_DATA_EXPORT[ 	]*\([$(AN)]*\).*$$|$(SYMBOL_PREFIX)\1|p' \
			>$@.new
	mv $@.new $@

libpng.vers: libpng.sym
	rm -f $@ $@.new
	echo PNG@PNGLIB_MAJOR@@PNGLIB_MINOR@_0 '{global:' > $@.new
	$(SED) s/$$/\;/ libpng.sym >> $@.new
	echo 'local: *; };' >> $@.new
	mv $@.new $@

test: check

# install the .../include headers as links to the new ones
install-data-hook:
	cd $(DESTDIR)$(includedir); rm -f png.h pngconf.h
	cd $(DESTDIR)$(includedir); $(LN_S) $(PNGLIB_BASENAME)/png.h png.h
	cd $(DESTDIR)$(includedir); $(LN_S) $(PNGLIB_BASENAME)/pngconf.h pngconf.h
	cd $(DESTDIR)$(pkgconfigdir); rm -f libpng.pc
	cd $(DESTDIR)$(pkgconfigdir); $(LN_S) $(PNGLIB_BASENAME).pc libpng.pc

# do evil things to libpng to cause libpng12 to be used
install-exec-hook:
	cd $(DESTDIR)$(bindir); rm -f libpng-config
	cd $(DESTDIR)$(bindir); $(LN_S) $(PNGLIB_BASENAME)-config libpng-config
	@set -x;\
	cd $(DESTDIR)$(libdir);\
	for ext in a la so sl dylib; do\
		rm -f libpng.$$ext;\
                if test -f $(PNGLIB_BASENAME).$$ext; then\
                       $(LN_S) $(PNGLIB_BASENAME).$$ext libpng.$$ext;\
                fi;\
	done

uninstall-hook:
	cd $(DESTDIR)$(includedir); rm -f png.h pngconf.h
	rm -f $(DESTDIR)$(pkgconfigdir)/libpng.pc
	rm -f $(DESTDIR)$(bindir)/libpng-config
	@if test -n "@compatlib@"; then\
		set -x;\
		cd $(DESTDIR)$(libdir);\
		for ext in a la so sl dylib; do\
			rm -f libpng.$$ext;\
		done;\
	fi
