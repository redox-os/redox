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

#include "SDL_video.h"
#include "../SDL_blit.h"
#include "SDL_fbriva.h"
#include "riva_mmio.h"
#include "riva_regs.h"


static int FifoEmptyCount = 0;
static int FifoFreeCount = 0;

/* Wait for vertical retrace */
static void WaitVBL(_THIS)
{
	volatile Uint8 *port = (Uint8 *)(mapped_io + PCIO_OFFSET + 0x3DA);

	while (  (*port & 0x08) )
		;
	while ( !(*port & 0x08) )
		;
}
static void NV3WaitIdle(_THIS)
{
	RivaRop *Rop = (RivaRop *)(mapped_io + ROP_OFFSET);
	while ( (Rop->FifoFree < FifoEmptyCount) ||
	        (*(mapped_io + PGRAPH_OFFSET + 0x000006B0) & 0x01) )
		;
}
static void NV4WaitIdle(_THIS)
{
	RivaRop *Rop = (RivaRop *)(mapped_io + ROP_OFFSET);
	while ( (Rop->FifoFree < FifoEmptyCount) ||
	        (*(mapped_io + PGRAPH_OFFSET + 0x00000700) & 0x01) )
		;
}

#if 0 /* Not yet implemented? */
/* Sets video mem colorkey and accelerated blit function */
static int SetHWColorKey(_THIS, SDL_Surface *surface, Uint32 key)
{
	return(0);
}

/* Sets per surface hardware alpha value */
static int SetHWAlpha(_THIS, SDL_Surface *surface, Uint8 value)
{
	return(0);
}
#endif /* Not yet implemented */

static int FillHWRect(_THIS, SDL_Surface *dst, SDL_Rect *rect, Uint32 color)
{
	int dstX, dstY;
	int dstW, dstH;
	RivaBitmap *Bitmap = (RivaBitmap *)(mapped_io + BITMAP_OFFSET);

	/* Don't blit to the display surface when switched away */
	if ( switched_away ) {
		return -2; /* no hardware access */
	}
	if ( dst == this->screen ) {
		SDL_mutexP(hw_lock);
	}

	/* Set up the X/Y base coordinates */
	dstW = rect->w;
	dstH = rect->h;
	FB_dst_to_xy(this, dst, &dstX, &dstY);

	/* Adjust for the current rectangle */
	dstX += rect->x;
	dstY += rect->y;

	RIVA_FIFO_FREE(Bitmap, 1);
	Bitmap->Color1A = color;

	RIVA_FIFO_FREE(Bitmap, 2);
	Bitmap->UnclippedRectangle[0].TopLeft     = (dstX << 16) | dstY; 
	Bitmap->UnclippedRectangle[0].WidthHeight = (dstW << 16) | dstH;

	FB_AddBusySurface(dst);

	if ( dst == this->screen ) {
		SDL_mutexV(hw_lock);
	}
	return(0);
}

static int HWAccelBlit(SDL_Surface *src, SDL_Rect *srcrect,
                       SDL_Surface *dst, SDL_Rect *dstrect)
{
	SDL_VideoDevice *this = current_video;
	int srcX, srcY;
	int dstX, dstY;
	int dstW, dstH;
	RivaScreenBlt *Blt = (RivaScreenBlt *)(mapped_io + BLT_OFFSET);

	/* FIXME: For now, only blit to display surface */
	if ( dst->pitch != SDL_VideoSurface->pitch ) {
		return(src->map->sw_blit(src, srcrect, dst, dstrect));
	}

	/* Don't blit to the display surface when switched away */
	if ( switched_away ) {
		return -2; /* no hardware access */
	}
	if ( dst == this->screen ) {
		SDL_mutexP(hw_lock);
	}

	/* Calculate source and destination base coordinates (in pixels) */
	dstW = dstrect->w;
	dstH = dstrect->h;
	FB_dst_to_xy(this, src, &srcX, &srcY);
	FB_dst_to_xy(this, dst, &dstX, &dstY);

	/* Adjust for the current blit rectangles */
	srcX += srcrect->x;
	srcY += srcrect->y;
	dstX += dstrect->x;
	dstY += dstrect->y;

	RIVA_FIFO_FREE(Blt, 3);
	Blt->TopLeftSrc  = (srcY << 16) | srcX;
	Blt->TopLeftDst  = (dstY << 16) | dstX;
	Blt->WidthHeight = (dstH  << 16) | dstW;

	FB_AddBusySurface(src);
	FB_AddBusySurface(dst);

	if ( dst == this->screen ) {
		SDL_mutexV(hw_lock);
	}
	return(0);
}

static int CheckHWBlit(_THIS, SDL_Surface *src, SDL_Surface *dst)
{
	int accelerated;

	/* Set initial acceleration on */
	src->flags |= SDL_HWACCEL;

	/* Set the surface attributes */
	if ( (src->flags & SDL_SRCALPHA) == SDL_SRCALPHA ) {
		if ( ! this->info.blit_hw_A ) {
			src->flags &= ~SDL_HWACCEL;
		}
	}
	if ( (src->flags & SDL_SRCCOLORKEY) == SDL_SRCCOLORKEY ) {
		if ( ! this->info.blit_hw_CC ) {
			src->flags &= ~SDL_HWACCEL;
		}
	}

	/* Check to see if final surface blit is accelerated */
	accelerated = !!(src->flags & SDL_HWACCEL);
	if ( accelerated ) {
		src->map->hw_blit = HWAccelBlit;
	}
	return(accelerated);
}

void FB_RivaAccel(_THIS, __u32 card)
{
	RivaRop *Rop = (RivaRop *)(mapped_io + ROP_OFFSET);

	/* We have hardware accelerated surface functions */
	this->CheckHWBlit = CheckHWBlit;
	wait_vbl = WaitVBL;
	switch (card) {
	    case FB_ACCEL_NV3:
		wait_idle = NV3WaitIdle;
		break;
	    case FB_ACCEL_NV4:
		wait_idle = NV4WaitIdle;
		break;
	    default:
		/* Hmm... FIXME */
		break;
	}
	FifoEmptyCount = Rop->FifoFree;

	/* The Riva has an accelerated color fill */
	this->info.blit_fill = 1;
	this->FillHWRect = FillHWRect;

	/* The Riva has accelerated normal and colorkey blits. */
	this->info.blit_hw = 1;
#if 0 /* Not yet implemented? */
	this->info.blit_hw_CC = 1;
	this->SetHWColorKey = SetHWColorKey;
#endif

#if 0 /* Not yet implemented? */
	/* The Riva has an accelerated alpha blit */
	this->info.blit_hw_A = 1;
	this->SetHWAlpha = SetHWAlpha;
#endif
}
