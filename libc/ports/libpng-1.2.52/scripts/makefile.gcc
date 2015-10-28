# makefile for libpng using gcc (generic, static library)
# Copyright (C) 2008 Glenn Randers-Pehrson
# Copyright (C) 2000 Cosmin Truta
# Copyright (C) 1995 Guy Eric Schalnat, Group 42, Inc.
#
# This code is released under the libpng license.
# For conditions of distribution and use, see the disclaimer
# and license in png.h

# Location of the zlib library and include files
ZLIBINC = ../zlib
ZLIBLIB = ../zlib

# Compiler, linker, lib and other tools
CC = gcc
LD = $(CC)
AR_RC = ar rcs
RANLIB = ranlib
RM_F = rm -f

CDEBUG = -g -DPNG_DEBUG=5
LDDEBUG =
CRELEASE = -O2
LDRELEASE = -s
#CFLAGS = -W -Wall $(CDEBUG)
CFLAGS = -W -Wall $(CRELEASE)
#LDFLAGS = $(LDDEBUG)
LDFLAGS = $(LDRELEASE)
LIBS = -lz -lm

# File extensions
O=.o
A=.a
EXE=

# Variables
OBJS =  png$(O) pngerror$(O) pngget$(O) pngmem$(O) pngpread$(O) \
	pngread$(O) pngrio$(O) pngrtran$(O) pngrutil$(O) pngset$(O) \
	pngtrans$(O) pngwio$(O) pngwrite$(O) pngwtran$(O) pngwutil$(O)

# Targets
all: static

.c$(O):
	$(CC) -c $(CFLAGS) -I$(ZLIBINC) $<

static: libpng$(A) pngtest$(EXE)

shared:
	@echo This is a generic makefile that cannot create shared libraries.
	@echo Please use a configuration that is specific to your platform.
	@false

libpng$(A): $(OBJS)
	$(AR_RC) $@ $(OBJS)
	$(RANLIB) $@

test: pngtest$(EXE)
	./pngtest$(EXE)

pngtest$(EXE): pngtest$(O) libpng$(A)
	$(LD) $(LDFLAGS) -L$(ZLIBLIB) -o $@ pngtest$(O) libpng$(A) $(LIBS)

clean:
	$(RM_F) *$(O) libpng$(A) pngtest$(EXE) pngout.png

png$(O):      png.h pngconf.h
pngerror$(O): png.h pngconf.h
pngget$(O):   png.h pngconf.h
pngmem$(O):   png.h pngconf.h
pngpread$(O): png.h pngconf.h
pngread$(O):  png.h pngconf.h
pngrio$(O):   png.h pngconf.h
pngrtran$(O): png.h pngconf.h
pngrutil$(O): png.h pngconf.h
pngset$(O):   png.h pngconf.h
pngtrans$(O): png.h pngconf.h
pngwio$(O):   png.h pngconf.h
pngwrite$(O): png.h pngconf.h
pngwtran$(O): png.h pngconf.h
pngwutil$(O): png.h pngconf.h

pngtest$(O):  png.h pngconf.h
