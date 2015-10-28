/*
    SDL - Simple DirectMedia Layer
    Copyright (C) 1997-2012 Sam Lantinga

    This library is free software; you can redistribute it and/or
    modify it under the terms of the GNU Lesser General Public
    License as published by the Free Software Foundation; either
    version 2.1 of the License, or (at your option) any later version.

    This library is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
    Lesser General Public License for more details.

    You should have received a copy of the GNU Lesser General Public
    License along with this library; if not, write to the Free Software
    Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA  02110-1301  USA

    Sam Lantinga
    slouken@libsdl.org
*/
#include "SDL_config.h"

/* This is the Playstation 2 implementation of YUV video overlays */

#include <fcntl.h>
#include <unistd.h>
#include <sys/ioctl.h>
#include <sys/mman.h>
#include <asm/page.h>		/* For definition of PAGE_SIZE */

#include "SDL_video.h"
#include "SDL_gsyuv_c.h"
#include "../SDL_yuvfuncs.h"

/* The maximum number of 16x16 pixel block converted at once */
#define MAX_MACROBLOCKS	1024	/* 2^10 macroblocks at once */

/* The functions used to manipulate video overlays */
static struct private_yuvhwfuncs gs_yuvfuncs = {
	GS_LockYUVOverlay,
	GS_UnlockYUVOverlay,
	GS_DisplayYUVOverlay,
	GS_FreeYUVOverlay
};

struct private_yuvhwdata {
	int ipu_fd;
	Uint8 *pixels;
	int macroblocks;
	int dma_len;
	caddr_t dma_mem;
	caddr_t ipu_imem;
	caddr_t ipu_omem;
	caddr_t dma_tags;
	unsigned long long *stretch_x1y1;
	unsigned long long *stretch_x2y2;
	struct ps2_plist plist;

	/* These are just so we don't have to allocate them separately */
	Uint16 pitches[3];
	Uint8 *planes[3];
};

static int power_of_2(int value)
{
	int shift;

	for ( shift = 0; (1<<shift) < value; ++shift ) {
		/* Keep looking */ ;
	}
	return(shift);
}

SDL_Overlay *GS_CreateYUVOverlay(_THIS, int width, int height, Uint32 format, SDL_Surface *display)
{
	SDL_Overlay *overlay;
	struct private_yuvhwdata *hwdata;
	int map_offset;
	unsigned long long *tags;
	caddr_t base;
	int bpp;
	int fbp, fbw, psm;
	int x, y, w, h;
	int pnum;
	struct ps2_packet *packet;
	struct ps2_packet tex_packet;

	/* We can only decode blocks of 16x16 pixels */
	if ( (width & 15) || (height & 15) ) {
		SDL_SetError("Overlay width/height must be multiples of 16");
		return(NULL);
	}
	/* Make sure the image isn't too large for a single DMA transfer */
	if ( ((width/16) * (height/16)) > MAX_MACROBLOCKS ) {
		SDL_SetError("Overlay too large (maximum size: %d pixels)",
		             MAX_MACROBLOCKS * 16 * 16);
		return(NULL);
	}

	/* Double-check the requested format.  For simplicity, we'll only
	   support planar YUV formats.
	 */
	switch (format) {
	    case SDL_YV12_OVERLAY:
	    case SDL_IYUV_OVERLAY:
		/* Supported planar YUV format */
		break;
	    default:
		SDL_SetError("Unsupported YUV format");
		return(NULL);
	}

	/* Create the overlay structure */
	overlay = (SDL_Overlay *)SDL_malloc(sizeof *overlay);
	if ( overlay == NULL ) {
		SDL_OutOfMemory();
		return(NULL);
	}
	SDL_memset(overlay, 0, (sizeof *overlay));

	/* Fill in the basic members */
	overlay->format = format;
	overlay->w = width;
	overlay->h = height;

	/* Set up the YUV surface function structure */
	overlay->hwfuncs = &gs_yuvfuncs;
	overlay->hw_overlay = 1;

	/* Create the pixel data */
	hwdata = (struct private_yuvhwdata *)SDL_malloc(sizeof *hwdata);
	overlay->hwdata = hwdata;
	if ( hwdata == NULL ) {
		SDL_FreeYUVOverlay(overlay);
		SDL_OutOfMemory();
		return(NULL);
	}
	hwdata->ipu_fd = -1;
	hwdata->pixels = (Uint8 *)SDL_malloc(width*height*2);
	if ( hwdata->pixels == NULL ) {
		SDL_FreeYUVOverlay(overlay);
		SDL_OutOfMemory();
		return(NULL);
	}
	hwdata->macroblocks = (width/16) * (height/16);

	/* Find the pitch and offset values for the overlay */
	overlay->pitches = hwdata->pitches;
	overlay->pixels = hwdata->planes;
	switch (format) {
	    case SDL_YV12_OVERLAY:
	    case SDL_IYUV_OVERLAY:
		overlay->pitches[0] = overlay->w;
		overlay->pitches[1] = overlay->pitches[0] / 2;
		overlay->pitches[2] = overlay->pitches[0] / 2;
	        overlay->pixels[0] = hwdata->pixels;
	        overlay->pixels[1] = overlay->pixels[0] +
		                     overlay->pitches[0] * overlay->h;
	        overlay->pixels[2] = overlay->pixels[1] +
		                     overlay->pitches[1] * overlay->h / 2;
		overlay->planes = 3;
		break;
	    default:
		/* We should never get here (caught above) */
		break;
	}

	/* Theoretically we could support several concurrent decode
	   streams queueing up on the same file descriptor, but for
	   simplicity we'll support only one.  Opening the IPU more
	   than once will fail with EBUSY.
	*/
	hwdata->ipu_fd = open("/dev/ps2ipu", O_RDWR);
	if ( hwdata->ipu_fd < 0 ) {
		SDL_FreeYUVOverlay(overlay);
		SDL_SetError("Playstation 2 IPU busy");
		return(NULL);
	}

	/* Allocate a DMA area for pixel conversion */
	bpp = this->screen->format->BytesPerPixel;
	map_offset = (mapped_len + (sysconf(_SC_PAGESIZE) - 1)) & ~(sysconf(_SC_PAGESIZE) - 1);
	hwdata->dma_len = hwdata->macroblocks * (16 * 16 + 8 * 8 + 8 * 8) +
	                  width * height * bpp +
	                  hwdata->macroblocks * (16 * sizeof(long long)) +
	                  12 * sizeof(long long);
	hwdata->dma_mem = mmap(0, hwdata->dma_len, PROT_READ|PROT_WRITE,
	                       MAP_SHARED, memory_fd, map_offset);
	if ( hwdata->dma_mem == MAP_FAILED ) {
		hwdata->ipu_imem = (caddr_t)0;
		SDL_FreeYUVOverlay(overlay);
		SDL_SetError("Unable to map %d bytes for DMA", hwdata->dma_len);
		return(NULL);
	}
	hwdata->ipu_imem = hwdata->dma_mem;
	hwdata->ipu_omem = hwdata->ipu_imem +
	                   hwdata->macroblocks * (16 * 16 + 8 * 8 + 8 * 8);
	hwdata->dma_tags = hwdata->ipu_omem + width * height * bpp;

	/* Allocate memory for the DMA packets */
	hwdata->plist.num = hwdata->macroblocks * 4 + 1;
	hwdata->plist.packet = (struct ps2_packet *)SDL_malloc(
	                       hwdata->plist.num*sizeof(struct ps2_packet));
	if ( ! hwdata->plist.packet ) {
		SDL_FreeYUVOverlay(overlay);
		SDL_OutOfMemory();
		return(NULL);
	}
	pnum = 0;
	packet = hwdata->plist.packet;

	/* Set up the tags to send the image to the screen */
	tags = (unsigned long long *)hwdata->dma_tags;
	base = hwdata->ipu_omem;
	fbp = screen_image.fbp;
	fbw = screen_image.fbw;
	psm = screen_image.psm;
	y = screen_image.y + screen_image.h;	/* Offscreen video memory */
	for ( h=height/16; h; --h ) {
		x = 0;			/* Visible video memory */
		for ( w=width/16; w; --w ) {
			/* The head tag */
			packet[pnum].ptr = &tags[0];
			packet[pnum].len = 10 * sizeof(*tags);
			++pnum;
			tags[0] = 4 | (1LL << 60);	/* GIFtag */
			tags[1] = 0x0e;			/* A+D */
			tags[2] = ((unsigned long long)fbp << 32) |
			          ((unsigned long long)fbw << 48) |
			          ((unsigned long long)psm << 56);
			tags[3] = PS2_GS_BITBLTBUF;
			tags[4] = ((unsigned long long)x << 32) |
			          ((unsigned long long)y << 48);
			tags[5] = PS2_GS_TRXPOS;
			tags[6] = (unsigned long long)16 |
			          ((unsigned long long)16 << 32);
			tags[7] = PS2_GS_TRXREG;
			tags[8] = 0;
			tags[9] = PS2_GS_TRXDIR;
			/* Now the actual image data */
			packet[pnum].ptr = &tags[10];
			packet[pnum].len = 2 * sizeof(*tags);
			++pnum;
			tags[10] = ((16*16*bpp) >> 4) | (2LL << 58);
			tags[11] = 0;
			packet[pnum].ptr = (void *)base;
			packet[pnum].len = 16 * 16 * bpp;
			++pnum;
			packet[pnum].ptr = &tags[12];
			packet[pnum].len = 2 * sizeof(*tags);
			++pnum;
			tags[12] = (0 >> 4) | (1 << 15) | (2LL << 58);
			tags[13] = 0;

			tags += 16;
			base += 16 * 16 * bpp;

			x += 16;
		}
		y += 16;
	}

	/* Set up the texture memory area for the video */
	tex_packet.ptr = tags;
	tex_packet.len = 8 * sizeof(*tags);
	tags[0] = 3 | (1LL << 60);	/* GIFtag */
	tags[1] = 0x0e;			/* A+D */
	tags[2] = ((screen_image.y + screen_image.h) * screen_image.w) / 64 +
	          ((unsigned long long)fbw << 14) +
	          ((unsigned long long)psm << 20) +
	          ((unsigned long long)power_of_2(width) << 26) +
	          ((unsigned long long)power_of_2(height) << 30) +
	          ((unsigned long long)1 << 34) +
	          ((unsigned long long)1 << 35);
	tags[3] = PS2_GS_TEX0_1;
	tags[4] = (1 << 5) + (1 << 6);
	tags[5] = PS2_GS_TEX1_1;
	tags[6] = 0;
	tags[7] = PS2_GS_TEXFLUSH;
	ioctl(console_fd, PS2IOC_SEND, &tex_packet);

	/* Set up the tags for scaling the image */
	packet[pnum].ptr = tags;
	packet[pnum].len = 12 * sizeof(*tags);
	++pnum;
	tags[0] = 5 | (1LL << 60);	/* GIFtag */
	tags[1] = 0x0e;			/* A+D */
	tags[2] = 6 + (1 << 4) + (1 << 8);
	tags[3] = PS2_GS_PRIM;
	tags[4] = ((unsigned long long)0 * 16) +
	           (((unsigned long long)0 * 16) << 16);
	tags[5] = PS2_GS_UV;
	tags[6] = 0; /* X1, Y1 */
	tags[7] = PS2_GS_XYZ2;
	hwdata->stretch_x1y1 = &tags[6];
	tags[8] = ((unsigned long long)overlay->w * 16) +
	           (((unsigned long long)overlay->h * 16) << 16);
	tags[9] = PS2_GS_UV;
	tags[10] = 0; /* X2, Y2 */
	tags[11] = PS2_GS_XYZ2;
	hwdata->stretch_x2y2 = &tags[10];

	/* We're all done.. */
	return(overlay);
}

int GS_LockYUVOverlay(_THIS, SDL_Overlay *overlay)
{
	return(0);
}

void GS_UnlockYUVOverlay(_THIS, SDL_Overlay *overlay)
{
	return;
}

int GS_DisplayYUVOverlay(_THIS, SDL_Overlay *overlay, SDL_Rect *src, SDL_Rect *dst)
{
	struct private_yuvhwdata *hwdata;
	__u32 cmd;
	struct ps2_packet packet;
	int h, w, i;
	Uint32 *lum, *Cr, *Cb;
	int lum_pitch;
	int crb_pitch;
	Uint32 *lum_src, *Cr_src, *Cb_src;
	Uint32 *srcp, *dstp;
	unsigned int x, y;
	SDL_Surface *screen;

	/* Find out where the various portions of the image are */
	hwdata = overlay->hwdata;
	switch (overlay->format) {
	    case SDL_YV12_OVERLAY:
		lum = (Uint32 *)overlay->pixels[0];
		Cr =  (Uint32 *)overlay->pixels[1];
		Cb =  (Uint32 *)overlay->pixels[2];
		break;
	    case SDL_IYUV_OVERLAY:
		lum = (Uint32 *)overlay->pixels[0];
		Cr =  (Uint32 *)overlay->pixels[2];
		Cb =  (Uint32 *)overlay->pixels[1];
	    default:
		SDL_SetError("Unsupported YUV format in blit (?)");
		return(-1);
	}
	dstp = (Uint32 *)hwdata->ipu_imem;
	lum_pitch = overlay->w/4;
	crb_pitch = (overlay->w/2)/4;

	/* Copy blocks of 16x16 pixels to the DMA area */
	for ( h=overlay->h/16; h; --h ) {
		lum_src = lum;
		Cr_src = Cr;
		Cb_src = Cb;
		for ( w=overlay->w/16; w; --w ) {
			srcp = lum_src;
			for ( i=0; i<16; ++i ) {
				dstp[0] = srcp[0];
				dstp[1] = srcp[1];
				dstp[2] = srcp[2];
				dstp[3] = srcp[3];
				srcp += lum_pitch;
				dstp += 4;
			}
			srcp = Cb_src;
			for ( i=0; i<8; ++i ) {
				dstp[0] = srcp[0];
				dstp[1] = srcp[1];
				srcp += crb_pitch;
				dstp += 2;
			}
			srcp = Cr_src;
			for ( i=0; i<8; ++i ) {
				dstp[0] = srcp[0];
				dstp[1] = srcp[1];
				srcp += crb_pitch;
				dstp += 2;
			}
			lum_src += 16 / 4;
			Cb_src += 8 / 4;
			Cr_src += 8 / 4;
		}
		lum += lum_pitch * 16;
		Cr += crb_pitch * 8;
		Cb += crb_pitch * 8;
	}

	/* Send the macroblock data to the IPU */
#ifdef DEBUG_YUV
	fprintf(stderr, "Sending data to IPU..\n");
#endif
	packet.ptr = hwdata->ipu_imem;
	packet.len = hwdata->macroblocks * (16 * 16 + 8 * 8 + 8 * 8);
	ioctl(hwdata->ipu_fd, PS2IOC_SENDA, &packet);

	/* Trigger the DMA to the IPU for conversion */
#ifdef DEBUG_YUV
	fprintf(stderr, "Trigging conversion command\n");
#endif
	cmd = (7 << 28) + hwdata->macroblocks;
	if ( screen_image.psm == PS2_GS_PSMCT16 ) {
		cmd += (1 << 27) +	/* Output RGB 555 */
		       (1 << 26);	/* Dither output */
	}
	ioctl(hwdata->ipu_fd, PS2IOC_SIPUCMD, &cmd);

	/* Retrieve the converted image from the IPU */
#ifdef DEBUG_YUV
	fprintf(stderr, "Retrieving data from IPU..\n");
#endif
	packet.ptr = hwdata->ipu_omem;
	packet.len = overlay->w * overlay->h *
	             this->screen->format->BytesPerPixel;
	ioctl(hwdata->ipu_fd, PS2IOC_RECV, &packet);

#ifdef DEBUG_YUV
	fprintf(stderr, "Copying image to screen..\n");
#endif
	/* Wait for previous DMA to complete */
	ioctl(console_fd, PS2IOC_SENDQCT, 1);

	/* Send the current image to the screen and scale it */
	screen = this->screen;
	x = (unsigned int)dst->x;
	y = (unsigned int)dst->y;
	if ( screen->offset ) {
		x += (screen->offset % screen->pitch) /
		     screen->format->BytesPerPixel;
		y += (screen->offset / screen->pitch);
	}
	y += screen_image.y;
	*hwdata->stretch_x1y1 = (x * 16) + ((y * 16) << 16);
	x += (unsigned int)dst->w;
	y += (unsigned int)dst->h;
	*hwdata->stretch_x2y2 = (x * 16) + ((y * 16) << 16);
	return ioctl(console_fd, PS2IOC_SENDL, &hwdata->plist);
}

void GS_FreeYUVOverlay(_THIS, SDL_Overlay *overlay)
{
	struct private_yuvhwdata *hwdata;

	hwdata = overlay->hwdata;
	if ( hwdata ) {
		if ( hwdata->ipu_fd >= 0 ) {
			close(hwdata->ipu_fd);
		}
		if ( hwdata->dma_mem ) {
			munmap(hwdata->dma_mem, hwdata->dma_len);
		}
		if ( hwdata->plist.packet ) {
			SDL_free(hwdata->plist.packet);
		}
		if ( hwdata->pixels ) {
			SDL_free(hwdata->pixels);
		}
		SDL_free(hwdata);
	}
}
