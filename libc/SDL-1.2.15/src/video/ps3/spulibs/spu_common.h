/*
 * SDL - Simple DirectMedia Layer
 * CELL BE Support for PS3 Framebuffer
 * Copyright (C) 2008, 2009 International Business Machines Corporation
 *
 * This library is free software; you can redistribute it and/or modify it
 * under the terms of the GNU Lesser General Public License as published
 * by the Free Software Foundation; either version 2.1 of the License, or
 * (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful, but
 * WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301
 * USA
 *
 *  Martin Lowinski  <lowinski [at] de [dot] ibm [ibm] com>
 *  Dirk Herrendoerfer <d.herrendoerfer [at] de [dot] ibm [dot] com>
 *  SPE code based on research by:
 *  Rene Becker
 *  Thimo Emmerich
 */

/* Common definitions/makros for SPUs */

#ifndef _SPU_COMMON_H
#define _SPU_COMMON_H

#include <stdio.h>
#include <stdint.h>
#include <string.h>

/* Tag management */
#define DMA_WAIT_TAG(_tag)     \
    mfc_write_tag_mask(1<<(_tag)); \
    mfc_read_tag_status_all();

/* SPU mailbox messages */
#define SPU_READY	0
#define SPU_START	1
#define SPU_FIN		2
#define SPU_EXIT	3

/* Tags */
#define RETR_BUF	0
#define STR_BUF		1
#define TAG_INIT	2

/* Buffersizes */
#define MAX_HDTV_WIDTH 1920
#define MAX_HDTV_HEIGHT 1080
/* One stride of HDTV */
#define BUFFER_SIZE 7680

/* fb_writer ppu/spu exchange parms */
struct fb_writer_parms_t {
	uint8_t *data;
	uint8_t *center;
	uint32_t out_line_stride;
	uint32_t in_line_stride;
	uint32_t bounded_input_height;
	uint32_t bounded_input_width;
	uint32_t fb_pixel_size;

	/* This padding is to fulfill the need for 16 byte alignment. On parm change, update! */
	char padding[4];
} __attribute__((aligned(128)));

/* yuv2rgb ppu/spu exchange parms */
struct yuv2rgb_parms_t {
	uint8_t* y_plane;
	uint8_t* v_plane;
	uint8_t* u_plane;

	uint8_t* dstBuffer;

	unsigned int src_pixel_width;
	unsigned int src_pixel_height;

	/* This padding is to fulfill the need for 16 byte alignment. On parm change, update! */
	char padding[128 - ((4 * sizeof(uint8_t *) + 2 * sizeof(unsigned int)) & 0x7F)];
} __attribute__((aligned(128)));

/* bilin_scaler ppu/spu exchange parms */
struct scale_parms_t {
	uint8_t* y_plane;
	uint8_t* v_plane;
	uint8_t* u_plane;

	uint8_t* dstBuffer;

	unsigned int src_pixel_width;
	unsigned int src_pixel_height;

	unsigned int dst_pixel_width;
	unsigned int dst_pixel_height;

	/* This padding is to fulfill the need for 16 byte alignment. On parm change, update! */
	char padding[128 - ((4 * sizeof(uint8_t *) + 4 * sizeof(unsigned int)) & 0x7F)];
} __attribute__((aligned(128)));

#endif /* _SPU_COMMON_H */


