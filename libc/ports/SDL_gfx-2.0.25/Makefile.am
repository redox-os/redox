# Makefile.am for the SDL_gfx library/DLL

ACLOCAL_AMFLAGS = -I m4

lib_LTLIBRARIES = libSDL_gfx.la

libSDL_gfxincludedir = $(includedir)/SDL
libSDL_gfxinclude_HEADERS =	\
	SDL_gfxPrimitives.h	\
	SDL_gfxPrimitives_font.h	\
	SDL_rotozoom.h	\
	SDL_framerate.h \
	SDL_imageFilter.h \
	SDL_gfxBlitFunc.h

libSDL_gfx_la_SOURCES =		\
	SDL_gfxPrimitives.c	\
	SDL_rotozoom.c	\
	SDL_framerate.c \
	SDL_imageFilter.c \
	SDL_gfxBlitFunc.c

DEFS = @DEFS@ -DBUILD_DLL
libSDL_gfx_la_LDFLAGS = 	\
	-no-undefined -version-info $(LT_CURRENT):$(LT_REVISION):$(LT_AGE)

pkgconfigdir = $(libdir)/pkgconfig
pkgconfig_DATA = SDL_gfx.pc

DISTCLEANFILES = *~ *~c *~h *.cross.cache inc 

distclean-local:
	-rm -rf autom4te.cache
	-rm -f *.suo *.sdf
	
