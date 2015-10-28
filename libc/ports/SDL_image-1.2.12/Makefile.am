# Makefile.am for the SDL sample image loading library and viewer

lib_LTLIBRARIES = libSDL_image.la

libSDL_imageincludedir = $(includedir)/SDL
libSDL_imageinclude_HEADERS =	\
	SDL_image.h

if USE_IMAGEIO
IMAGEIO_SOURCE = IMG_ImageIO.m
endif

libSDL_image_la_SOURCES =	\
	IMG.c			\
	IMG_bmp.c		\
	IMG_gif.c		\
	IMG_jpg.c		\
	IMG_lbm.c		\
	IMG_pcx.c		\
	IMG_png.c		\
	IMG_pnm.c		\
	IMG_tga.c		\
	IMG_tif.c		\
	IMG_xcf.c		\
	IMG_xpm.c		\
	IMG_xv.c		\
	IMG_webp.c		\
	$(IMAGEIO_SOURCE)

EXTRA_DIST =			\
	Android.mk		\
	CHANGES			\
	version.rc		\
	VisualC			\
	VisualCE		\
	Xcode			\
	Xcode-iOS		\
	MPWmake.sea.bin		\
	Watcom-OS2.zip		\
	IMG_xxx.c		\
	IMG_UIImage.m		\
	SDL_image.spec		\
	gcc-fat.sh		\
	autogen.sh

if USE_VERSION_RC
libSDL_image_la_LDFLAGS = 	\
	-no-undefined		\
	-release $(LT_RELEASE)	\
	-version-info $(LT_CURRENT):$(LT_REVISION):$(LT_AGE) -Wl,version.o
libSDL_image_la_LIBADD = $(IMG_LIBS)
libSDL_image_la_DEPENDENCIES = version.o
else
libSDL_image_la_LDFLAGS = 	\
	-no-undefined		\
	-release $(LT_RELEASE)	\
	-version-info $(LT_CURRENT):$(LT_REVISION):$(LT_AGE)
libSDL_image_la_LIBADD = $(IMG_LIBS)
endif

pkgconfigdir = $(libdir)/pkgconfig
pkgconfig_DATA = SDL_image.pc

%.o : %.rc
	$(WINDRES) $< $@

noinst_PROGRAMS = showimage

showimage_LDADD = libSDL_image.la

# Rule to build tar-gzipped distribution package
$(PACKAGE)-$(VERSION).tar.gz: distcheck

# Rule to build RPM distribution package
rpm: $(PACKAGE)-$(VERSION).tar.gz
	rpmbuild -ta $(PACKAGE)-$(VERSION).tar.gz

