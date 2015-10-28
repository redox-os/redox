# This Makefile is for building the CELL BE SPU libs
# libfb_writer_spu.so, libyuv2rgb_spu.so, libbilin_scaler_spu.so

# Toolchain
SPU_GCC=/usr/bin/spu-gcc
PPU_GCC=/usr/bin/gcc
PPU_EMBEDSPU=/usr/bin/embedspu
PPU_AR=/usr/bin/ar
PPU_LD=/usr/bin/ld
INSTALL=/usr/bin/install

SPU_CFLAGS=-W -Wall -Winline -Wno-main -I. -I /usr/spu/include -I /opt/cell/sdk/usr/spu/include -finline-limit=10000 -Winline -ftree-vectorize -funroll-loops -fmodulo-sched -ffast-math -fPIC -O2

# Usually /usr/lib, depending on your distribution
PREFIX=/usr/lib


all: libfb_writer_spu.a libfb_writer_spu.so \
				libyuv2rgb_spu.so libyuv2rgb_spu.a \
				libbilin_scaler_spu.so libbilin_scaler_spu.a


# fb_writer
fb_writer_spu-embed.o: fb_writer.c spu_common.h
	$(SPU_GCC) $(SPU_CFLAGS) -o fb_writer_spu fb_writer.c -lm
	$(PPU_EMBEDSPU) -m32 fb_writer_spu fb_writer_spu fb_writer_spu-embed.o

libfb_writer_spu.so: fb_writer_spu-embed.o
	$(PPU_LD) -o libfb_writer_spu.so -shared -soname=libfb_writer_spu.so fb_writer_spu-embed.o

libfb_writer_spu.a: fb_writer_spu-embed.o
	$(PPU_AR) -qcs libfb_writer_spu.a fb_writer_spu-embed.o


# yuv2rgb_converter
yuv2rgb_spu-embed.o: yuv2rgb_converter.c spu_common.h
	$(SPU_GCC) $(SPU_CFLAGS) -o yuv2rgb_spu yuv2rgb_converter.c -lm
	$(PPU_EMBEDSPU) -m32 yuv2rgb_spu yuv2rgb_spu yuv2rgb_spu-embed.o

libyuv2rgb_spu.a: yuv2rgb_spu-embed.o
	$(PPU_AR) -qcs libyuv2rgb_spu.a yuv2rgb_spu-embed.o

libyuv2rgb_spu.so: yuv2rgb_spu-embed.o
	$(PPU_LD) -o libyuv2rgb_spu.so -shared -soname=libyuv2rgb_spu.so yuv2rgb_spu-embed.o


# bilin_scaler
bilin_scaler_spu-embed.o: bilin_scaler.c spu_common.h
	$(SPU_GCC) $(SPU_CFLAGS) -o bilin_scaler_spu bilin_scaler.c -lm
	$(PPU_EMBEDSPU) -m32 bilin_scaler_spu bilin_scaler_spu bilin_scaler_spu-embed.o

libbilin_scaler_spu.a: bilin_scaler_spu-embed.o
	$(PPU_AR) -qcs libbilin_scaler_spu.a bilin_scaler_spu-embed.o

libbilin_scaler_spu.so: bilin_scaler_spu-embed.o
	$(PPU_LD) -o libbilin_scaler_spu.so -shared -soname=libbilin_scaler_spu.so bilin_scaler_spu-embed.o

install: libfb_writer_spu.a libfb_writer_spu.so \
				libyuv2rgb_spu.so libyuv2rgb_spu.a \
				libbilin_scaler_spu.so libbilin_scaler_spu.a
	$(INSTALL) -c -m 0755 libfb_writer_spu.so $(PREFIX)/.
	$(INSTALL) -c -m 0655 libfb_writer_spu.a $(PREFIX)/.
	$(INSTALL) -c -m 0755 libyuv2rgb_spu.so $(PREFIX)/.
	$(INSTALL) -c -m 0655 libyuv2rgb_spu.a $(PREFIX)/.
	$(INSTALL) -c -m 0755 libbilin_scaler_spu.so $(PREFIX)/.
	$(INSTALL) -c -m 0655 libbilin_scaler_spu.a $(PREFIX)/.


uninstall: $(PREFIX)/libfb_writer_spu.so $(PREFIX)/libfb_writer_spu.a \
		$(PREFIX)/libyuv2rgb_spu.so $(PREFIX)/libyuv2rgb_spu.a \
		$(PREFIX)/libbilin_scaler_spu.so $(PREFIX)/libbilin_scaler_spu.a
	rm -f $(PREFIX)/libfb_writer_spu.a
	rm -f $(PREFIX)/libfb_writer_spu.so
	rm -f $(PREFIX)/libyuv2rgb_spu.so
	rm -f $(PREFIX)/libyuv2rgb_spu.a
	rm -f $(PREFIX)/libbilin_scaler_spu.so
	rm -f $(PREFIX)/libbilin_scaler_spu.a


clean:
	rm -f bilin_scaler_spu-embed.o libbilin_scaler_spu.so libbilin_scaler_spu.a bilin_scaler_spu
	rm -f yuv2rgb_spu-embed.o libyuv2rgb_spu.so libyuv2rgb_spu.a yuv2rgb_spu
	rm -f fb_writer_spu-embed.o libfb_writer_spu.so libfb_writer_spu.a fb_writer_spu
