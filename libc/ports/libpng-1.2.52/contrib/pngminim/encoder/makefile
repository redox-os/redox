# Makefile for PngMinus (pnm2pngm)
# Linux / Unix

#CC=cc
CC=gcc
LD=$(CC)

RM=rm -f

CFLAGS=-DPNG_USER_CONFIG -DNO_GZIP -DZ_SOLO -I. -O1

C=.c
O=.o
L=.a
E=

ZOBJS  = adler32$(O) compress$(O) crc32$(O) deflate$(O) \
	 dummy_inflate$(O) \
	 trees$(O) uncompr$(O) zutil$(O)

OBJS  = pnm2pngm$(O) png$(O) pngerror$(O) pngget$(O) pngmem$(O) \
	pngset$(O) pngtrans$(O) pngwio$(O) pngwrite$(O) \
	pngwtran$(O) pngwutil$(O) $(ZOBJS)

# implicit make rules -------------------------------------------------------

.c$(O): png.h pngconf.h pngusr.h zlib.h
	$(CC) -c $(CFLAGS) $<

# dependencies

all: pnm2pngm$(E)

pnm2pngm$(E): $(OBJS)
	$(LD) -o pnm2pngm$(E) $(OBJS)
	strip pnm2pngm$(E)

clean:
	$(RM) pnm2pngm$(O)
	$(RM) pnm2pngm$(E)
	$(RM) $(OBJS)

# End of makefile for pnm2pngm
