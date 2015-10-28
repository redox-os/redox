# Makefile.am for the SDL truetype font loading library and viewer

lib_LTLIBRARIES = libSDL_ttf.la

libSDL_ttfincludedir = $(includedir)/SDL
libSDL_ttfinclude_HEADERS =	\
	SDL_ttf.h

libSDL_ttf_la_SOURCES =		\
	SDL_ttf.c

EXTRA_DIST =			\
	Android.mk		\
	CHANGES			\
	version.rc		\
	VisualC			\
	Xcode			\
	Xcode-iOS		\
	MPWmake.sea.bin		\
	Watcom-Win32.zip	\
	SDL_ttf.spec		\
	SDL_ttf.spec.in		\
	gcc-fat.sh		\
	autogen.sh

if USE_VERSION_RC
libSDL_ttf_la_LDFLAGS = 	\
	-no-undefined		\
	-release $(LT_RELEASE)	\
	-version-info $(LT_CURRENT):$(LT_REVISION):$(LT_AGE) -Wl,version.o
libSDL_ttf_la_DEPENDENCIES = version.o
else
libSDL_ttf_la_LDFLAGS = 	\
	-no-undefined		\
	-release $(LT_RELEASE)	\
	-version-info $(LT_CURRENT):$(LT_REVISION):$(LT_AGE)
endif

pkgconfigdir = $(libdir)/pkgconfig
pkgconfig_DATA = SDL_ttf.pc

%.o : %.rc
	$(WINDRES) $< $@

noinst_PROGRAMS = showfont glfont

showfont_LDADD = libSDL_ttf.la
glfont_LDADD = libSDL_ttf.la @GL_LIBS@ @MATHLIB@

# Rule to build tar-gzipped distribution package
$(PACKAGE)-$(VERSION).tar.gz: distcheck

# Rule to build RPM distribution package
rpm: $(PACKAGE)-$(VERSION).tar.gz
	rpmbuild -ta $(PACKAGE)-$(VERSION).tar.gz

