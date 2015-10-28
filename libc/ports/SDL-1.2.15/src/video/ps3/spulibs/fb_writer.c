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

#include "spu_common.h"

#include <spu_intrinsics.h>
#include <spu_mfcio.h>
#include <stdio.h>
#include <string.h>

// Debugging
//#define DEBUG

#ifdef DEBUG
#define deprintf(fmt, args... ) \
	fprintf( stdout, fmt, ##args ); \
	fflush( stdout );
#else
#define deprintf( fmt, args... )
#endif

void cpy_to_fb(unsigned int);

/* fb_writer_spu parms */
static volatile struct fb_writer_parms_t parms __attribute__ ((aligned(128)));

/* Code running on SPU */
int main(unsigned long long spe_id __attribute__ ((unused)), unsigned long long argp __attribute__ ((unused)))
{
	deprintf("[SPU] fb_writer_spu is up... (on SPE #%llu)\n", spe_id);
	uint32_t ea_mfc, mbox;
	// send ready message
	spu_write_out_mbox(SPU_READY);

	while (1) {
		/* Check mailbox */
		mbox = spu_read_in_mbox();
		deprintf("[SPU] Message is %u\n", mbox);
		switch (mbox) {
			case SPU_EXIT:
				deprintf("[SPU] fb_writer goes down...\n");
				return 0;
			case SPU_START:
				break;
			default:
				deprintf("[SPU] Cannot handle message\n");
				continue;
		}

		/* Tag Manager setup */
		unsigned int tags;
		tags = mfc_multi_tag_reserve(5);
		if (tags == MFC_TAG_INVALID) {
			deprintf("[SPU] Failed to reserve mfc tags on fb_writer\n");
			return 0;
		}

		/* Framebuffer parms */
		ea_mfc = spu_read_in_mbox();
		deprintf("[SPU] Message on fb_writer is %u\n", ea_mfc);
		spu_mfcdma32(&parms, (unsigned int)ea_mfc,
				sizeof(struct fb_writer_parms_t), tags,
				MFC_GET_CMD);
		deprintf("[SPU] argp = %u\n", (unsigned int)argp);
		DMA_WAIT_TAG(tags);

		/* Copy parms->data to framebuffer */
		deprintf("[SPU] Copying to framebuffer started\n");
		cpy_to_fb(tags);
		deprintf("[SPU] Copying to framebuffer done!\n");

		mfc_multi_tag_release(tags, 5);
		deprintf("[SPU] fb_writer_spu... done!\n");
		/* Send FIN msg */
		spu_write_out_mbox(SPU_FIN);
	}

	return 0;
}

void cpy_to_fb(unsigned int tag_id_base)
{
	unsigned int i;
	unsigned char current_buf;
	uint8_t *in = parms.data;

	/* Align fb pointer which was centered before */
	uint8_t *fb =
	    (unsigned char *)((unsigned int)parms.center & 0xFFFFFFF0);

	uint32_t bounded_input_height = parms.bounded_input_height;
	uint32_t bounded_input_width = parms.bounded_input_width;
	uint32_t fb_pixel_size = parms.fb_pixel_size;

	uint32_t out_line_stride = parms.out_line_stride;
	uint32_t in_line_stride = parms.in_line_stride;
	uint32_t in_line_size = bounded_input_width * fb_pixel_size;

	current_buf = 0;

	/* Local store buffer */
	static volatile uint8_t buf[4][BUFFER_SIZE]
	    __attribute__ ((aligned(128)));
	/* do 4-times multibuffering using DMA list, process in two steps */
	for (i = 0; i < bounded_input_height >> 2; i++) {
		/* first buffer */
		DMA_WAIT_TAG(tag_id_base + 1);
		// retrieve buffer
		spu_mfcdma32(buf[0], (unsigned int)in, in_line_size,
			     tag_id_base + 1, MFC_GETB_CMD);
		DMA_WAIT_TAG(tag_id_base + 1);
		// store buffer
		spu_mfcdma32(buf[0], (unsigned int)fb, in_line_size,
			     tag_id_base + 1, MFC_PUTB_CMD);
		in += in_line_stride;
		fb += out_line_stride;
		deprintf("[SPU] 1st buffer copied in=0x%x, fb=0x%x\n", in,
		       fb);

		/* second buffer */
		DMA_WAIT_TAG(tag_id_base + 2);
		// retrieve buffer
		spu_mfcdma32(buf[1], (unsigned int)in, in_line_size,
			     tag_id_base + 2, MFC_GETB_CMD);
		DMA_WAIT_TAG(tag_id_base + 2);
		// store buffer
		spu_mfcdma32(buf[1], (unsigned int)fb, in_line_size,
			     tag_id_base + 2, MFC_PUTB_CMD);
		in += in_line_stride;
		fb += out_line_stride;
		deprintf("[SPU] 2nd buffer copied in=0x%x, fb=0x%x\n", in,
		       fb);

		/* third buffer */
		DMA_WAIT_TAG(tag_id_base + 3);
		// retrieve buffer
		spu_mfcdma32(buf[2], (unsigned int)in, in_line_size,
			     tag_id_base + 3, MFC_GETB_CMD);
		DMA_WAIT_TAG(tag_id_base + 3);
		// store buffer
		spu_mfcdma32(buf[2], (unsigned int)fb, in_line_size,
			     tag_id_base + 3, MFC_PUTB_CMD);
		in += in_line_stride;
		fb += out_line_stride;
		deprintf("[SPU] 3rd buffer copied in=0x%x, fb=0x%x\n", in,
		       fb);

		/* fourth buffer */
		DMA_WAIT_TAG(tag_id_base + 4);
		// retrieve buffer
		spu_mfcdma32(buf[3], (unsigned int)in, in_line_size,
			     tag_id_base + 4, MFC_GETB_CMD);
		DMA_WAIT_TAG(tag_id_base + 4);
		// store buffer
		spu_mfcdma32(buf[3], (unsigned int)fb, in_line_size,
			     tag_id_base + 4, MFC_PUTB_CMD);
		in += in_line_stride;
		fb += out_line_stride;
		deprintf("[SPU] 4th buffer copied in=0x%x, fb=0x%x\n", in,
		       fb);
		deprintf("[SPU] Loop #%i, bounded_input_height=%i\n", i,
		       bounded_input_height >> 2);
	}
	DMA_WAIT_TAG(tag_id_base + 2);
	DMA_WAIT_TAG(tag_id_base + 3);
	DMA_WAIT_TAG(tag_id_base + 4);
}


