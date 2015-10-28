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

// Debugging
//#define DEBUG

#ifdef DEBUG
#define deprintf(fmt, args... ) \
	fprintf( stdout, fmt, ##args ); \
	fflush( stdout );
#else
#define deprintf( fmt, args... )
#endif

struct yuv2rgb_parms_t parms_converter __attribute__((aligned(128)));

/* A maximum of 8 lines Y, therefore 4 lines V, 4 lines U are stored
 * there might be the need to retrieve misaligned data, adjust
 * incoming v and u plane to be able to handle this (add 128)
 */
unsigned char y_plane[2][(MAX_HDTV_WIDTH + 128) * 4] __attribute__((aligned(128)));
unsigned char v_plane[2][(MAX_HDTV_WIDTH + 128) * 2] __attribute__((aligned(128)));
unsigned char u_plane[2][(MAX_HDTV_WIDTH + 128) * 2] __attribute__((aligned(128)));

/* A maximum of 4 lines BGRA are stored, 4 byte per pixel */
unsigned char bgra[4 * MAX_HDTV_WIDTH * 4] __attribute__((aligned(128)));

/* some vectors needed by the float to int conversion */
static const vector float vec_255 = { 255.0f, 255.0f, 255.0f, 255.0f };
static const vector float vec_0_1 = { 0.1f, 0.1f, 0.1f, 0.1f };

void yuv_to_rgb_w16();
void yuv_to_rgb_w32();

void yuv_to_rgb_w16_line(unsigned char* y_addr, unsigned char* v_addr, unsigned char* u_addr, unsigned char* bgra_addr, unsigned int width);
void yuv_to_rgb_w32_line(unsigned char* y_addr, unsigned char* v_addr, unsigned char* u_addr, unsigned char* bgra_addr_, unsigned int width);


int main(unsigned long long spe_id __attribute__((unused)), unsigned long long argp __attribute__ ((unused)))
{
	deprintf("[SPU] yuv2rgb_spu is up... (on SPE #%llu)\n", spe_id);
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
		unsigned int tag_id;
		tag_id = mfc_multi_tag_reserve(1);
		if (tag_id == MFC_TAG_INVALID) {
			deprintf("[SPU] Failed to reserve mfc tags on yuv2rgb_converter\n");
			return 0;
		}

		/* DMA transfer for the input parameters */
		ea_mfc = spu_read_in_mbox();
		deprintf("[SPU] Message on yuv2rgb_converter is %u\n", ea_mfc);
		spu_mfcdma32(&parms_converter, (unsigned int)ea_mfc, sizeof(struct yuv2rgb_parms_t), tag_id, MFC_GET_CMD);
		DMA_WAIT_TAG(tag_id);

		/* There are alignment issues that involve handling of special cases
		 * a width of 32 results in a width of 16 in the chrominance
		 * --> choose the proper handling to optimize the performance
		 */
		deprintf("[SPU] Convert %ix%i from YUV to RGB\n", parms_converter.src_pixel_width, parms_converter.src_pixel_height);
		if (parms_converter.src_pixel_width & 0x1f) {
			deprintf("[SPU] Using yuv_to_rgb_w16\n");
			yuv_to_rgb_w16();
		} else {
			deprintf("[SPU] Using yuv_to_rgb_w32\n");
			yuv_to_rgb_w32();
		}

		mfc_multi_tag_release(tag_id, 1);
		deprintf("[SPU] yuv2rgb_spu... done!\n");
		/* Send FIN message */
		spu_write_out_mbox(SPU_FIN);
	}

	return 0;
}


/*
 * float_to_char()
 *
 * converts a float to a character using saturated
 * arithmetic
 *
 * @param s float for conversion
 * @returns converted character
 */
inline static unsigned char float_to_char(float s) {
	vector float vec_s = spu_splats(s);
	vector unsigned int select_1 = spu_cmpgt(vec_0_1, vec_s);
	vec_s = spu_sel(vec_s, vec_0_1, select_1);

	vector unsigned int select_2 = spu_cmpgt(vec_s, vec_255);
	vec_s = spu_sel(vec_s, vec_255, select_2);
	return (unsigned char) spu_extract(vec_s,0);
}


/*
 * vfloat_to_vuint()
 *
 * converts a float vector to an unsinged int vector using saturated
 * arithmetic
 *
 * @param vec_s float vector for conversion
 * @returns converted unsigned int vector
 */
inline static vector unsigned int vfloat_to_vuint(vector float vec_s) {
	vector unsigned int select_1 = spu_cmpgt(vec_0_1, vec_s);
	vec_s = spu_sel(vec_s, vec_0_1, select_1);

	vector unsigned int select_2 = spu_cmpgt(vec_s, vec_255);
	vec_s = spu_sel(vec_s, vec_255, select_2);
	return spu_convtu(vec_s,0);
}


void yuv_to_rgb_w16() {
	// Pixel dimensions of the picture
	uint32_t width, height;

	// Extract parameters
	width = parms_converter.src_pixel_width;
	height = parms_converter.src_pixel_height;

	// Plane data management
	// Y
	unsigned char* ram_addr_y = parms_converter.y_plane;
	// V
	unsigned char* ram_addr_v = parms_converter.v_plane;
	// U
	unsigned char* ram_addr_u = parms_converter.u_plane;

	// BGRA
	unsigned char* ram_addr_bgra = parms_converter.dstBuffer;

	// Strides
	unsigned int stride_y = width;
	unsigned int stride_vu = width>>1;

	// Buffer management
	unsigned int buf_idx = 0;
	unsigned int size_4lines_y = stride_y<<2;
	unsigned int size_2lines_y = stride_y<<1;
	unsigned int size_2lines_vu = stride_vu<<1;

	// 2*width*4byte_per_pixel
	unsigned int size_2lines_bgra = width<<3;


	// start double-buffered processing
	// 4 lines y
	spu_mfcdma32(y_plane[buf_idx], (unsigned int) ram_addr_y, size_4lines_y, RETR_BUF+buf_idx, MFC_GET_CMD);

	// 2 lines v
	spu_mfcdma32(v_plane[buf_idx], (unsigned int) ram_addr_v, size_2lines_vu, RETR_BUF+buf_idx, MFC_GET_CMD);

	// 2 lines u
	spu_mfcdma32(u_plane[buf_idx], (unsigned int) ram_addr_u, size_2lines_vu, RETR_BUF+buf_idx, MFC_GET_CMD);

	// Wait for these transfers to be completed
	DMA_WAIT_TAG((RETR_BUF + buf_idx));

	unsigned int i;
	for(i=0; i<(height>>2)-1; i++) {

		buf_idx^=1;

		// 4 lines y
		spu_mfcdma32(y_plane[buf_idx], (unsigned int) ram_addr_y+size_4lines_y, size_4lines_y, RETR_BUF+buf_idx, MFC_GET_CMD);

		// 2 lines v
		spu_mfcdma32(v_plane[buf_idx], (unsigned int) ram_addr_v+size_2lines_vu, size_2lines_vu, RETR_BUF+buf_idx, MFC_GET_CMD);

		// 2 lines u
		spu_mfcdma32(u_plane[buf_idx], (unsigned int) ram_addr_u+size_2lines_vu, size_2lines_vu, RETR_BUF+buf_idx, MFC_GET_CMD);

		DMA_WAIT_TAG((RETR_BUF + buf_idx));

		buf_idx^=1;


		// Convert YUV to BGRA, store it back (first two lines)
		yuv_to_rgb_w16_line(y_plane[buf_idx], v_plane[buf_idx], u_plane[buf_idx], bgra, width);

		// Next two lines
		yuv_to_rgb_w16_line(y_plane[buf_idx] + size_2lines_y,
				v_plane[buf_idx] + stride_vu,
				u_plane[buf_idx] + stride_vu,
				bgra + size_2lines_bgra,
				width);

		// Wait for previous storing transfer to be completed
		DMA_WAIT_TAG(STR_BUF);

		// Store converted lines in two steps->max transfer size 16384
		spu_mfcdma32(bgra, (unsigned int) ram_addr_bgra, size_2lines_bgra, STR_BUF, MFC_PUT_CMD);
		ram_addr_bgra += size_2lines_bgra;
		spu_mfcdma32(bgra+size_2lines_bgra, (unsigned int) ram_addr_bgra, size_2lines_bgra, STR_BUF, MFC_PUT_CMD);
		ram_addr_bgra += size_2lines_bgra;

		// Move 4 lines
		ram_addr_y += size_4lines_y;
		ram_addr_v += size_2lines_vu;
		ram_addr_u += size_2lines_vu;

		buf_idx^=1;
	}

	// Convert YUV to BGRA, store it back (first two lines)
	yuv_to_rgb_w16_line(y_plane[buf_idx], v_plane[buf_idx], u_plane[buf_idx], bgra, width);

	// Next two lines
	yuv_to_rgb_w16_line(y_plane[buf_idx] + size_2lines_y,
			v_plane[buf_idx] + stride_vu,
			u_plane[buf_idx] + stride_vu,
			bgra + size_2lines_bgra,
			width);

	// Wait for previous storing transfer to be completed
	DMA_WAIT_TAG(STR_BUF);
	spu_mfcdma32(bgra, (unsigned int) ram_addr_bgra, size_2lines_bgra, STR_BUF, MFC_PUT_CMD);
	ram_addr_bgra += size_2lines_bgra;
	spu_mfcdma32(bgra+size_2lines_bgra, (unsigned int) ram_addr_bgra, size_2lines_bgra, STR_BUF, MFC_PUT_CMD);

	// wait for previous storing transfer to be completed
	DMA_WAIT_TAG(STR_BUF);

}


void yuv_to_rgb_w32() {
	// Pixel dimensions of the picture
	uint32_t width, height;

	// Extract parameters
	width = parms_converter.src_pixel_width;
	height = parms_converter.src_pixel_height;

	// Plane data management
	// Y
	unsigned char* ram_addr_y = parms_converter.y_plane;
	// V
	unsigned char* ram_addr_v = parms_converter.v_plane;
	// U
	unsigned char* ram_addr_u = parms_converter.u_plane;

	// BGRA
	unsigned char* ram_addr_bgra = parms_converter.dstBuffer;

	// Strides
	unsigned int stride_y = width;
	unsigned int stride_vu = width>>1;

	// Buffer management
	unsigned int buf_idx = 0;
	unsigned int size_4lines_y = stride_y<<2;
	unsigned int size_2lines_y = stride_y<<1;
	unsigned int size_2lines_vu = stride_vu<<1;

	// 2*width*4byte_per_pixel
	unsigned int size_2lines_bgra = width<<3;

	// start double-buffered processing
	// 4 lines y
	spu_mfcdma32(y_plane[buf_idx], (unsigned int) ram_addr_y, size_4lines_y, RETR_BUF + buf_idx, MFC_GET_CMD);
	// 2 lines v
	spu_mfcdma32(v_plane[buf_idx], (unsigned int) ram_addr_v, size_2lines_vu, RETR_BUF + buf_idx, MFC_GET_CMD);
	// 2 lines u
	spu_mfcdma32(u_plane[buf_idx], (unsigned int) ram_addr_u, size_2lines_vu, RETR_BUF + buf_idx, MFC_GET_CMD);

	// Wait for these transfers to be completed
	DMA_WAIT_TAG((RETR_BUF + buf_idx));

	unsigned int i;
	for(i=0; i < (height>>2)-1; i++) {
		buf_idx^=1;
		// 4 lines y
		spu_mfcdma32(y_plane[buf_idx], (unsigned int) ram_addr_y+size_4lines_y, size_4lines_y, RETR_BUF + buf_idx, MFC_GET_CMD);
		deprintf("4lines = %d\n", size_4lines_y);
		// 2 lines v
		spu_mfcdma32(v_plane[buf_idx], (unsigned int) ram_addr_v+size_2lines_vu, size_2lines_vu, RETR_BUF + buf_idx, MFC_GET_CMD);
		deprintf("2lines = %d\n", size_2lines_vu);
		// 2 lines u
		spu_mfcdma32(u_plane[buf_idx], (unsigned int) ram_addr_u+size_2lines_vu, size_2lines_vu, RETR_BUF + buf_idx, MFC_GET_CMD);
		deprintf("2lines = %d\n", size_2lines_vu);

		DMA_WAIT_TAG((RETR_BUF + buf_idx));

		buf_idx^=1;

		// Convert YUV to BGRA, store it back (first two lines)
		yuv_to_rgb_w32_line(y_plane[buf_idx], v_plane[buf_idx], u_plane[buf_idx], bgra, width);

		// Next two lines
		yuv_to_rgb_w32_line(y_plane[buf_idx] + size_2lines_y,
				v_plane[buf_idx] + stride_vu,
				u_plane[buf_idx] + stride_vu,
				bgra + size_2lines_bgra,
				width);

		// Wait for previous storing transfer to be completed
		DMA_WAIT_TAG(STR_BUF);

		// Store converted lines in two steps->max transfer size 16384
		spu_mfcdma32(bgra, (unsigned int)ram_addr_bgra, size_2lines_bgra, STR_BUF, MFC_PUT_CMD);
		ram_addr_bgra += size_2lines_bgra;
		spu_mfcdma32(bgra + size_2lines_bgra, (unsigned int)ram_addr_bgra, size_2lines_bgra, STR_BUF, MFC_PUT_CMD);
		ram_addr_bgra += size_2lines_bgra;

		// Move 4 lines
		ram_addr_y += size_4lines_y;
		ram_addr_v += size_2lines_vu;
		ram_addr_u += size_2lines_vu;

		buf_idx^=1;
	}

	// Convert YUV to BGRA, store it back (first two lines)
	yuv_to_rgb_w32_line(y_plane[buf_idx], v_plane[buf_idx], u_plane[buf_idx], bgra, width);

	// Next two lines
	yuv_to_rgb_w32_line(y_plane[buf_idx] + size_2lines_y,
			v_plane[buf_idx] + stride_vu,
			u_plane[buf_idx] + stride_vu,
			bgra + size_2lines_bgra,
			width);

	// Wait for previous storing transfer to be completed
	DMA_WAIT_TAG(STR_BUF);
	spu_mfcdma32(bgra, (unsigned int) ram_addr_bgra, size_2lines_bgra, STR_BUF, MFC_PUT_CMD);
	ram_addr_bgra += size_2lines_bgra;
	spu_mfcdma32(bgra + size_2lines_bgra, (unsigned int) ram_addr_bgra, size_2lines_bgra, STR_BUF, MFC_PUT_CMD);

	// Wait for previous storing transfer to be completed
	DMA_WAIT_TAG(STR_BUF);
}


/* Some vectors needed by the yuv 2 rgb conversion algorithm */
const vector float vec_minus_128 = { -128.0f, -128.0f, -128.0f, -128.0f };
const vector unsigned char vec_null = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };
const vector unsigned char vec_char2int_first = { 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x11, 0x00, 0x00, 0x00, 0x12, 0x00, 0x00, 0x00, 0x13 };
const vector unsigned char vec_char2int_second = { 0x00, 0x00, 0x00, 0x14, 0x00, 0x00, 0x00, 0x15, 0x00, 0x00, 0x00, 0x16, 0x00, 0x00, 0x00, 0x17 };
const vector unsigned char vec_char2int_third = { 0x00, 0x00, 0x00, 0x18, 0x00, 0x00, 0x00, 0x19, 0x00, 0x00, 0x00, 0x1A, 0x00, 0x00, 0x00, 0x1B };
const vector unsigned char vec_char2int_fourth = { 0x00, 0x00, 0x00, 0x1C, 0x00, 0x00, 0x00, 0x1D, 0x00, 0x00, 0x00, 0x1E, 0x00, 0x00, 0x00, 0x1F };

const vector float vec_R_precalc_coeff = {1.403f, 1.403f, 1.403f, 1.403f};
const vector float vec_Gu_precalc_coeff = {-0.344f, -0.344f, -0.344f, -0.344f};
const vector float vec_Gv_precalc_coeff = {-0.714f, -0.714f, -0.714f, -0.714f};
const vector float vec_B_precalc_coeff = {1.773f, 1.773f, 1.773f, 1.773f};

const vector unsigned int vec_alpha =  { 255 << 24, 255 << 24, 255 << 24, 255 << 24 };

const vector unsigned char vec_select_floats_upper = { 0x00, 0x01, 0x02, 0x03, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x04, 0x05, 0x06, 0x07 };
const vector unsigned char vec_select_floats_lower = { 0x08, 0x09, 0x0A, 0x0B, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x0C, 0x0D, 0x0E, 0x0F };


/*
 * yuv_to_rgb_w16()
 *
 * processes to line of yuv-input, width has to be a multiple of 16
 * two lines of yuv are taken as input
 *
 * @param y_addr address of the y plane in local store
 * @param v_addr address of the v plane in local store
 * @param u_addr address of the u plane in local store
 * @param bgra_addr_ address of the bgra output buffer
 * @param width the width in pixel
 */
void yuv_to_rgb_w16_line(unsigned char* y_addr, unsigned char* v_addr, unsigned char* u_addr, unsigned char* bgra_addr_, unsigned int width) {
	// each pixel is stored as an integer
	unsigned int* bgra_addr = (unsigned int*) bgra_addr_;

	unsigned int x;
	for(x = 0; x < width; x+=2) {
		// Gehe zweischrittig durch die zeile, da jeder u und v wert fuer 4 pixel(zwei hoch, zwei breit) gilt
		const unsigned char Y_1 = *(y_addr + x);
		const unsigned char Y_2 = *(y_addr + x + 1);
		const unsigned char Y_3 = *(y_addr + x + width);
		const unsigned char Y_4 = *(y_addr + x + width + 1);
		const unsigned char U = *(u_addr + (x >> 1));
		const unsigned char V = *(v_addr + (x >> 1));

		float V_minus_128 = (float)((float)V - 128.0f);
		float U_minus_128 = (float)((float)U - 128.0f);

		float R_precalculate = 1.403f * V_minus_128;
		float G_precalculate = -(0.344f * U_minus_128 + 0.714f * V_minus_128);
		float B_precalculate = 1.773f * U_minus_128;

		const unsigned char R_1 = float_to_char((Y_1 + R_precalculate));
		const unsigned char R_2 = float_to_char((Y_2 + R_precalculate));
		const unsigned char R_3 = float_to_char((Y_3 + R_precalculate));
		const unsigned char R_4 = float_to_char((Y_4 + R_precalculate));
		const unsigned char G_1 = float_to_char((Y_1 + G_precalculate));
		const unsigned char G_2 = float_to_char((Y_2 + G_precalculate));
		const unsigned char G_3 = float_to_char((Y_3 + G_precalculate));
		const unsigned char G_4 = float_to_char((Y_4 + G_precalculate));
		const unsigned char B_1 = float_to_char((Y_1 + B_precalculate));
		const unsigned char B_2 = float_to_char((Y_2 + B_precalculate));
		const unsigned char B_3 = float_to_char((Y_3 + B_precalculate));
		const unsigned char B_4 = float_to_char((Y_4 + B_precalculate));

		*(bgra_addr + x) = (B_1 << 0)| (G_1 << 8) | (R_1 << 16) | (255 << 24);
		*(bgra_addr + x + 1) = (B_2 << 0)| (G_2 << 8) | (R_2 << 16) | (255 << 24);
		*(bgra_addr + x + width) = (B_3 << 0)| (G_3 << 8) | (R_3 << 16) | (255 << 24);
		*(bgra_addr + x + width + 1) = (B_4 << 0)| (G_4 << 8) | (R_4 << 16) | (255 << 24);
	}
}


/*
 * yuv_to_rgb_w32()
 *
 * processes to line of yuv-input, width has to be a multiple of 32
 * two lines of yuv are taken as input
 *
 * @param y_addr address of the y plane in local store
 * @param v_addr address of the v plane in local store
 * @param u_addr address of the u plane in local store
 * @param bgra_addr_ address of the bgra output buffer
 * @param width the width in pixel
 */
void yuv_to_rgb_w32_line(unsigned char* y_addr, unsigned char* v_addr, unsigned char* u_addr, unsigned char* bgra_addr_, unsigned int width) {
	// each pixel is stored as an integer
	unsigned int* bgra_addr = (unsigned int*) bgra_addr_;

	unsigned int x;
	for(x = 0; x < width; x+=32) {
		// Gehe zweischrittig durch die zeile, da jeder u und v wert fuer 4 pixel(zwei hoch, zwei breit) gilt

		const vector unsigned char vchar_Y_1 = *((vector unsigned char*)(y_addr + x));
		const vector unsigned char vchar_Y_2 = *((vector unsigned char*)(y_addr + x + 16));
		const vector unsigned char vchar_Y_3 = *((vector unsigned char*)(y_addr + x + width));
		const vector unsigned char vchar_Y_4 = *((vector unsigned char*)(y_addr + x + width + 16));
		const vector unsigned char vchar_U = *((vector unsigned char*)(u_addr + (x >> 1)));
		const vector unsigned char vchar_V = *((vector unsigned char*)(v_addr + (x >> 1)));

		const vector float vfloat_U_1 = spu_add(spu_convtf((vector unsigned int)spu_shuffle(vec_null, vchar_U, vec_char2int_first), 0),vec_minus_128);
		const vector float vfloat_U_2 = spu_add(spu_convtf((vector unsigned int)spu_shuffle(vec_null, vchar_U, vec_char2int_second), 0),vec_minus_128);
		const vector float vfloat_U_3 = spu_add(spu_convtf((vector unsigned int)spu_shuffle(vec_null, vchar_U, vec_char2int_third), 0),vec_minus_128);
		const vector float vfloat_U_4 = spu_add(spu_convtf((vector unsigned int)spu_shuffle(vec_null, vchar_U, vec_char2int_fourth), 0),vec_minus_128);

		const vector float vfloat_V_1 = spu_add(spu_convtf((vector unsigned int)spu_shuffle(vec_null, vchar_V, vec_char2int_first), 0),vec_minus_128);
		const vector float vfloat_V_2 = spu_add(spu_convtf((vector unsigned int)spu_shuffle(vec_null, vchar_V, vec_char2int_second), 0),vec_minus_128);
		const vector float vfloat_V_3 = spu_add(spu_convtf((vector unsigned int)spu_shuffle(vec_null, vchar_V, vec_char2int_third), 0),vec_minus_128);
		const vector float vfloat_V_4 = spu_add(spu_convtf((vector unsigned int)spu_shuffle(vec_null, vchar_V, vec_char2int_fourth), 0),vec_minus_128);

		vector float Y_1 = spu_convtf((vector unsigned int)spu_shuffle(vec_null, vchar_Y_1, vec_char2int_first), 0);
		vector float Y_2 = spu_convtf((vector unsigned int)spu_shuffle(vec_null, vchar_Y_1, vec_char2int_second), 0);
		vector float Y_3 = spu_convtf((vector unsigned int)spu_shuffle(vec_null, vchar_Y_1, vec_char2int_third), 0);
		vector float Y_4 = spu_convtf((vector unsigned int)spu_shuffle(vec_null, vchar_Y_1, vec_char2int_fourth), 0);
		vector float Y_5 = spu_convtf((vector unsigned int)spu_shuffle(vec_null, vchar_Y_2, vec_char2int_first), 0);
		vector float Y_6 = spu_convtf((vector unsigned int)spu_shuffle(vec_null, vchar_Y_2, vec_char2int_second), 0);
		vector float Y_7 = spu_convtf((vector unsigned int)spu_shuffle(vec_null, vchar_Y_2, vec_char2int_third), 0);
		vector float Y_8 = spu_convtf((vector unsigned int)spu_shuffle(vec_null, vchar_Y_2, vec_char2int_fourth), 0);
		vector float Y_9 = spu_convtf((vector unsigned int)spu_shuffle(vec_null, vchar_Y_3, vec_char2int_first), 0);
		vector float Y_10 = spu_convtf((vector unsigned int)spu_shuffle(vec_null, vchar_Y_3, vec_char2int_second), 0);
		vector float Y_11 = spu_convtf((vector unsigned int)spu_shuffle(vec_null, vchar_Y_3, vec_char2int_third), 0);
		vector float Y_12 = spu_convtf((vector unsigned int)spu_shuffle(vec_null, vchar_Y_3, vec_char2int_fourth), 0);
		vector float Y_13 = spu_convtf((vector unsigned int)spu_shuffle(vec_null, vchar_Y_4, vec_char2int_first), 0);
		vector float Y_14 = spu_convtf((vector unsigned int)spu_shuffle(vec_null, vchar_Y_4, vec_char2int_second), 0);
		vector float Y_15 = spu_convtf((vector unsigned int)spu_shuffle(vec_null, vchar_Y_4, vec_char2int_third), 0);
		vector float Y_16 = spu_convtf((vector unsigned int)spu_shuffle(vec_null, vchar_Y_4, vec_char2int_fourth), 0);

		const vector float R1a_precalculate = spu_mul(vec_R_precalc_coeff, vfloat_V_1);
		const vector float R2a_precalculate = spu_mul(vec_R_precalc_coeff, vfloat_V_2);
		const vector float R3a_precalculate = spu_mul(vec_R_precalc_coeff, vfloat_V_3);
		const vector float R4a_precalculate = spu_mul(vec_R_precalc_coeff, vfloat_V_4);

		const vector float R1_precalculate = spu_shuffle(R1a_precalculate,  R1a_precalculate, vec_select_floats_upper);
		const vector float R2_precalculate = spu_shuffle(R1a_precalculate,  R1a_precalculate, vec_select_floats_lower);
		const vector float R3_precalculate = spu_shuffle(R2a_precalculate,  R2a_precalculate, vec_select_floats_upper);
		const vector float R4_precalculate = spu_shuffle(R2a_precalculate,  R2a_precalculate, vec_select_floats_lower);
		const vector float R5_precalculate = spu_shuffle(R3a_precalculate,  R3a_precalculate, vec_select_floats_upper);
		const vector float R6_precalculate = spu_shuffle(R3a_precalculate,  R3a_precalculate, vec_select_floats_lower);
		const vector float R7_precalculate = spu_shuffle(R4a_precalculate,  R4a_precalculate, vec_select_floats_upper);
		const vector float R8_precalculate = spu_shuffle(R4a_precalculate,  R4a_precalculate, vec_select_floats_lower);


		const vector float G1a_precalculate = spu_madd(vec_Gu_precalc_coeff, vfloat_U_1, spu_mul(vfloat_V_1, vec_Gv_precalc_coeff));
		const vector float G2a_precalculate = spu_madd(vec_Gu_precalc_coeff, vfloat_U_2, spu_mul(vfloat_V_2, vec_Gv_precalc_coeff));
		const vector float G3a_precalculate = spu_madd(vec_Gu_precalc_coeff, vfloat_U_3, spu_mul(vfloat_V_3, vec_Gv_precalc_coeff));
		const vector float G4a_precalculate = spu_madd(vec_Gu_precalc_coeff, vfloat_U_4, spu_mul(vfloat_V_4, vec_Gv_precalc_coeff));

		const vector float G1_precalculate = spu_shuffle(G1a_precalculate,  G1a_precalculate, vec_select_floats_upper);
		const vector float G2_precalculate = spu_shuffle(G1a_precalculate,  G1a_precalculate, vec_select_floats_lower);
		const vector float G3_precalculate = spu_shuffle(G2a_precalculate,  G2a_precalculate, vec_select_floats_upper);
		const vector float G4_precalculate = spu_shuffle(G2a_precalculate,  G2a_precalculate, vec_select_floats_lower);
		const vector float G5_precalculate = spu_shuffle(G3a_precalculate,  G3a_precalculate, vec_select_floats_upper);
		const vector float G6_precalculate = spu_shuffle(G3a_precalculate,  G3a_precalculate, vec_select_floats_lower);
		const vector float G7_precalculate = spu_shuffle(G4a_precalculate,  G4a_precalculate, vec_select_floats_upper);
		const vector float G8_precalculate = spu_shuffle(G4a_precalculate,  G4a_precalculate, vec_select_floats_lower);


		const vector float B1a_precalculate = spu_mul(vec_B_precalc_coeff, vfloat_U_1);
		const vector float B2a_precalculate = spu_mul(vec_B_precalc_coeff, vfloat_U_2);
		const vector float B3a_precalculate = spu_mul(vec_B_precalc_coeff, vfloat_U_3);
		const vector float B4a_precalculate = spu_mul(vec_B_precalc_coeff, vfloat_U_4);

		const vector float B1_precalculate = spu_shuffle(B1a_precalculate,  B1a_precalculate, vec_select_floats_upper);
		const vector float B2_precalculate = spu_shuffle(B1a_precalculate,  B1a_precalculate, vec_select_floats_lower);
		const vector float B3_precalculate = spu_shuffle(B2a_precalculate,  B2a_precalculate, vec_select_floats_upper);
		const vector float B4_precalculate = spu_shuffle(B2a_precalculate,  B2a_precalculate, vec_select_floats_lower);
		const vector float B5_precalculate = spu_shuffle(B3a_precalculate,  B3a_precalculate, vec_select_floats_upper);
		const vector float B6_precalculate = spu_shuffle(B3a_precalculate,  B3a_precalculate, vec_select_floats_lower);
		const vector float B7_precalculate = spu_shuffle(B4a_precalculate,  B4a_precalculate, vec_select_floats_upper);
		const vector float B8_precalculate = spu_shuffle(B4a_precalculate,  B4a_precalculate, vec_select_floats_lower);


		const vector unsigned int  R_1 = vfloat_to_vuint(spu_add( Y_1, R1_precalculate));
		const vector unsigned int  R_2 = vfloat_to_vuint(spu_add( Y_2, R2_precalculate));
		const vector unsigned int  R_3 = vfloat_to_vuint(spu_add( Y_3, R3_precalculate));
		const vector unsigned int  R_4 = vfloat_to_vuint(spu_add( Y_4, R4_precalculate));
		const vector unsigned int  R_5 = vfloat_to_vuint(spu_add( Y_5, R5_precalculate));
		const vector unsigned int  R_6 = vfloat_to_vuint(spu_add( Y_6, R6_precalculate));
		const vector unsigned int  R_7 = vfloat_to_vuint(spu_add( Y_7, R7_precalculate));
		const vector unsigned int  R_8 = vfloat_to_vuint(spu_add( Y_8, R8_precalculate));
		const vector unsigned int  R_9 = vfloat_to_vuint(spu_add( Y_9, R1_precalculate));
		const vector unsigned int R_10 = vfloat_to_vuint(spu_add(Y_10, R2_precalculate));
		const vector unsigned int R_11 = vfloat_to_vuint(spu_add(Y_11, R3_precalculate));
		const vector unsigned int R_12 = vfloat_to_vuint(spu_add(Y_12, R4_precalculate));
		const vector unsigned int R_13 = vfloat_to_vuint(spu_add(Y_13, R5_precalculate));
		const vector unsigned int R_14 = vfloat_to_vuint(spu_add(Y_14, R6_precalculate));
		const vector unsigned int R_15 = vfloat_to_vuint(spu_add(Y_15, R7_precalculate));
		const vector unsigned int R_16 = vfloat_to_vuint(spu_add(Y_16, R8_precalculate));

		const vector unsigned int  G_1 = vfloat_to_vuint(spu_add( Y_1, G1_precalculate));
		const vector unsigned int  G_2 = vfloat_to_vuint(spu_add( Y_2, G2_precalculate));
		const vector unsigned int  G_3 = vfloat_to_vuint(spu_add( Y_3, G3_precalculate));
		const vector unsigned int  G_4 = vfloat_to_vuint(spu_add( Y_4, G4_precalculate));
		const vector unsigned int  G_5 = vfloat_to_vuint(spu_add( Y_5, G5_precalculate));
		const vector unsigned int  G_6 = vfloat_to_vuint(spu_add( Y_6, G6_precalculate));
		const vector unsigned int  G_7 = vfloat_to_vuint(spu_add( Y_7, G7_precalculate));
		const vector unsigned int  G_8 = vfloat_to_vuint(spu_add( Y_8, G8_precalculate));
		const vector unsigned int  G_9 = vfloat_to_vuint(spu_add( Y_9, G1_precalculate));
		const vector unsigned int G_10 = vfloat_to_vuint(spu_add(Y_10, G2_precalculate));
		const vector unsigned int G_11 = vfloat_to_vuint(spu_add(Y_11, G3_precalculate));
		const vector unsigned int G_12 = vfloat_to_vuint(spu_add(Y_12, G4_precalculate));
		const vector unsigned int G_13 = vfloat_to_vuint(spu_add(Y_13, G5_precalculate));
		const vector unsigned int G_14 = vfloat_to_vuint(spu_add(Y_14, G6_precalculate));
		const vector unsigned int G_15 = vfloat_to_vuint(spu_add(Y_15, G7_precalculate));
		const vector unsigned int G_16 = vfloat_to_vuint(spu_add(Y_16, G8_precalculate));

		const vector unsigned int  B_1 = vfloat_to_vuint(spu_add( Y_1, B1_precalculate));
		const vector unsigned int  B_2 = vfloat_to_vuint(spu_add( Y_2, B2_precalculate));
		const vector unsigned int  B_3 = vfloat_to_vuint(spu_add( Y_3, B3_precalculate));
		const vector unsigned int  B_4 = vfloat_to_vuint(spu_add( Y_4, B4_precalculate));
		const vector unsigned int  B_5 = vfloat_to_vuint(spu_add( Y_5, B5_precalculate));
		const vector unsigned int  B_6 = vfloat_to_vuint(spu_add( Y_6, B6_precalculate));
		const vector unsigned int  B_7 = vfloat_to_vuint(spu_add( Y_7, B7_precalculate));
		const vector unsigned int  B_8 = vfloat_to_vuint(spu_add( Y_8, B8_precalculate));
		const vector unsigned int  B_9 = vfloat_to_vuint(spu_add( Y_9, B1_precalculate));
		const vector unsigned int B_10 = vfloat_to_vuint(spu_add(Y_10, B2_precalculate));
		const vector unsigned int B_11 = vfloat_to_vuint(spu_add(Y_11, B3_precalculate));
		const vector unsigned int B_12 = vfloat_to_vuint(spu_add(Y_12, B4_precalculate));
		const vector unsigned int B_13 = vfloat_to_vuint(spu_add(Y_13, B5_precalculate));
		const vector unsigned int B_14 = vfloat_to_vuint(spu_add(Y_14, B6_precalculate));
		const vector unsigned int B_15 = vfloat_to_vuint(spu_add(Y_15, B7_precalculate));
		const vector unsigned int B_16 = vfloat_to_vuint(spu_add(Y_16, B8_precalculate));

		*((vector unsigned int*)(bgra_addr + x)) = spu_or(spu_or(vec_alpha,  B_1), spu_or(spu_slqwbyte( R_1, 2),spu_slqwbyte(G_1, 1)));
		*((vector unsigned int*)(bgra_addr + x + 4)) = spu_or(spu_or(vec_alpha,  B_2), spu_or(spu_slqwbyte( R_2, 2),spu_slqwbyte(G_2, 1)));
		*((vector unsigned int*)(bgra_addr + x + 8)) = spu_or(spu_or(vec_alpha,  B_3), spu_or(spu_slqwbyte( R_3, 2),spu_slqwbyte(G_3, 1)));
		*((vector unsigned int*)(bgra_addr + x + 12)) = spu_or(spu_or(vec_alpha,  B_4), spu_or(spu_slqwbyte( R_4, 2),spu_slqwbyte(G_4, 1)));
		*((vector unsigned int*)(bgra_addr + x + 16)) = spu_or(spu_or(vec_alpha,  B_5), spu_or(spu_slqwbyte( R_5, 2),spu_slqwbyte(G_5, 1)));
		*((vector unsigned int*)(bgra_addr + x + 20)) = spu_or(spu_or(vec_alpha,  B_6), spu_or(spu_slqwbyte( R_6, 2),spu_slqwbyte(G_6, 1)));
		*((vector unsigned int*)(bgra_addr + x + 24)) = spu_or(spu_or(vec_alpha,  B_7), spu_or(spu_slqwbyte( R_7, 2),spu_slqwbyte(G_7, 1)));
		*((vector unsigned int*)(bgra_addr + x + 28)) = spu_or(spu_or(vec_alpha,  B_8), spu_or(spu_slqwbyte( R_8, 2),spu_slqwbyte(G_8, 1)));
		*((vector unsigned int*)(bgra_addr + x + width)) = spu_or(spu_or(vec_alpha,  B_9), spu_or(spu_slqwbyte( R_9, 2),spu_slqwbyte(G_9, 1)));
		*((vector unsigned int*)(bgra_addr + x + width + 4)) = spu_or(spu_or(vec_alpha, B_10), spu_or(spu_slqwbyte(R_10, 2),spu_slqwbyte(G_10, 1)));
		*((vector unsigned int*)(bgra_addr + x + width + 8)) = spu_or(spu_or(vec_alpha, B_11), spu_or(spu_slqwbyte(R_11, 2),spu_slqwbyte(G_11, 1)));
		*((vector unsigned int*)(bgra_addr + x + width + 12)) = spu_or(spu_or(vec_alpha, B_12), spu_or(spu_slqwbyte(R_12, 2),spu_slqwbyte(G_12, 1)));
		*((vector unsigned int*)(bgra_addr + x + width + 16)) = spu_or(spu_or(vec_alpha, B_13), spu_or(spu_slqwbyte(R_13, 2),spu_slqwbyte(G_13, 1)));
		*((vector unsigned int*)(bgra_addr + x + width + 20)) = spu_or(spu_or(vec_alpha, B_14), spu_or(spu_slqwbyte(R_14, 2),spu_slqwbyte(G_14, 1)));
		*((vector unsigned int*)(bgra_addr + x + width + 24)) = spu_or(spu_or(vec_alpha, B_15), spu_or(spu_slqwbyte(R_15, 2),spu_slqwbyte(G_15, 1)));
		*((vector unsigned int*)(bgra_addr + x + width + 28)) = spu_or(spu_or(vec_alpha, B_16), spu_or(spu_slqwbyte(R_16, 2),spu_slqwbyte(G_16, 1)));
	}
}

