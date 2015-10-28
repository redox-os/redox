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

/* Framebuffer console based SDL video driver implementation.
*/

#include <fcntl.h>
#include <unistd.h>
#include <sys/ioctl.h>
#include <sys/mman.h>

#include "SDL_video.h"
#include "SDL_mouse.h"
#include "../SDL_sysvideo.h"
#include "../SDL_pixels_c.h"
#include "../../events/SDL_events_c.h"
#include "../SDL_cursor_c.h"
#include "SDL_gsvideo.h"
#include "SDL_gsmouse_c.h"
#include "SDL_gsevents_c.h"
#include "SDL_gsyuv_c.h"


/* Initialization/Query functions */
static int GS_VideoInit(_THIS, SDL_PixelFormat *vformat);
static SDL_Rect **GS_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags);
static SDL_Surface *GS_SetVideoMode(_THIS, SDL_Surface *current, int width, int height, int bpp, Uint32 flags);
static int GS_SetColors(_THIS, int firstcolor, int ncolors, SDL_Color *colors);
static void GS_VideoQuit(_THIS);

/* Hardware surface functions */
static int GS_AllocHWSurface(_THIS, SDL_Surface *surface);
static int GS_LockHWSurface(_THIS, SDL_Surface *surface);
static void GS_UnlockHWSurface(_THIS, SDL_Surface *surface);
static void GS_FreeHWSurface(_THIS, SDL_Surface *surface);

/* GS driver bootstrap functions */

static int GS_Available(void)
{
	int console, memory;

	console = open(PS2_DEV_GS, O_RDWR, 0);
	if ( console >= 0 ) {
		close(console);
	}
	memory = open(PS2_DEV_MEM, O_RDWR, 0);
	if ( memory >= 0 ) {
		close(memory);
	}
	return((console >= 0) && (memory >= 0));
}

static void GS_DeleteDevice(SDL_VideoDevice *device)
{
	SDL_free(device->hidden);
	SDL_free(device);
}

static SDL_VideoDevice *GS_CreateDevice(int devindex)
{
	SDL_VideoDevice *this;

	/* Initialize all variables that we clean on shutdown */
	this = (SDL_VideoDevice *)SDL_malloc(sizeof(SDL_VideoDevice));
	if ( this ) {
		SDL_memset(this, 0, (sizeof *this));
		this->hidden = (struct SDL_PrivateVideoData *)
				SDL_malloc((sizeof *this->hidden));
	}
	if ( (this == NULL) || (this->hidden == NULL) ) {
		SDL_OutOfMemory();
		if ( this ) {
			SDL_free(this);
		}
		return(0);
	}
	SDL_memset(this->hidden, 0, (sizeof *this->hidden));
	mouse_fd = -1;
	keyboard_fd = -1;

	/* Set the function pointers */
	this->VideoInit = GS_VideoInit;
	this->ListModes = GS_ListModes;
	this->SetVideoMode = GS_SetVideoMode;
	this->CreateYUVOverlay = GS_CreateYUVOverlay;
	this->SetColors = GS_SetColors;
	this->UpdateRects = NULL;
	this->VideoQuit = GS_VideoQuit;
	this->AllocHWSurface = GS_AllocHWSurface;
	this->CheckHWBlit = NULL;
	this->FillHWRect = NULL;
	this->SetHWColorKey = NULL;
	this->SetHWAlpha = NULL;
	this->LockHWSurface = GS_LockHWSurface;
	this->UnlockHWSurface = GS_UnlockHWSurface;
	this->FlipHWSurface = NULL;
	this->FreeHWSurface = GS_FreeHWSurface;
	this->SetIcon = NULL;
	this->SetCaption = NULL;
	this->GetWMInfo = NULL;
	this->FreeWMCursor = GS_FreeWMCursor;
	this->CreateWMCursor = GS_CreateWMCursor;
	this->ShowWMCursor = GS_ShowWMCursor;
	this->MoveWMCursor = GS_MoveWMCursor;
	this->InitOSKeymap = GS_InitOSKeymap;
	this->PumpEvents = GS_PumpEvents;

	this->free = GS_DeleteDevice;

	return this;
}

VideoBootStrap PS2GS_bootstrap = {
	"ps2gs", "PlayStation 2 Graphics Synthesizer",
	GS_Available, GS_CreateDevice
};

/* These are the pixel formats for the 32, 24, and 16 bit video modes */
static struct {
	int bpp;
	Uint32 r;
	Uint32 g;
	Uint32 b;
} GS_pixelmasks[] = {
	{ 32, 0x000000FF,	/* RGB little-endian */
	      0x0000FF00,
	      0x00FF0000 },
	{ 24, 0x000000FF,	/* RGB little-endian */
	      0x0000FF00,
	      0x00FF0000 },
	{ 16, 0x0000001f,	/* RGB little-endian */
	      0x000003e0,
	      0x00007c00 },
};
/* This is a mapping from SDL bytes-per-pixel to GS pixel format */
static int GS_formatmap[] = {
	-1,		/* 0 bpp, not a legal value */
	-1,		/* 8 bpp, not supported (yet?) */
	PS2_GS_PSMCT16,	/* 16 bpp */
	PS2_GS_PSMCT24,	/* 24 bpp */
	PS2_GS_PSMCT32	/* 32 bpp */
};

static unsigned long long head_tags[] __attribute__((aligned(16))) = {
	4 | (1LL << 60),	/* GIFtag */
	0x0e,			/* A+D */
	0,			/* 2 */
	PS2_GS_BITBLTBUF,
	0,			/* 4 */
	PS2_GS_TRXPOS,
	0,			/* 6 */
	PS2_GS_TRXREG,
	0,			/* 8 */
	PS2_GS_TRXDIR
};

#define MAXIMG		(32767 * 16)
#define MAXTAGS		8

static inline int loadimage_nonblock(int fd, struct ps2_image *image, int size,
                                     unsigned long long *hm,
                                     unsigned long long *im)
{
	struct ps2_plist plist;
	struct ps2_packet packet[1 + MAXTAGS * 2];
	int isize;
	int pnum, it, eop;
	char *data;

	/* initialize the variables */
	data = (char *)image->ptr;
	pnum = it = eop = 0;
	plist.packet = packet;

	/* make BITBLT packet */
	packet[pnum].ptr = hm;
	packet[pnum].len = sizeof(head_tags);
	pnum++;
	hm[2] = ((unsigned long long)image->fbp << 32) |
	        ((unsigned long long)image->fbw << 48) |
	        ((unsigned long long)image->psm << 56);
	hm[4] = ((unsigned long long)image->x << 32) |
	        ((unsigned long long)image->y << 48);
	hm[6] = (unsigned long long)image->w |
	        ((unsigned long long)image->h << 32);

	/* make image mode tags */
	while (!eop) {
		isize = size > MAXIMG ? MAXIMG : size;
		size -= isize;
		eop = (size == 0);

		packet[pnum].ptr = &im[it];
		packet[pnum].len = sizeof(unsigned long long) * 2;
		pnum++;
		im[it++] = (isize >> 4) | (eop ? (1 << 15) : 0) | (2LL << 58);
		im[it++] = 0;

		packet[pnum].ptr = (void *)data;
		packet[pnum].len = isize;
		pnum++;
		data += isize;
	}
	plist.num = pnum;

	return ioctl(fd, PS2IOC_SENDL, &plist);
}

static unsigned long long tex_tags[] __attribute__((aligned(16))) = {
	3 | (1LL << 60),	/* GIFtag */
	0x0e,			/* A+D */
	0,			/* 2 */
	PS2_GS_TEX0_1,
	(1 << 5) + (1 << 6),
	PS2_GS_TEX1_1,
	0,
	PS2_GS_TEXFLUSH
};
static unsigned long long scale_tags[] __attribute__((aligned(16))) = {
	5 | (1LL << 60),	/* GIFtag */
	0x0e,			/* A+D */
	6 + (1 << 4) + (1 << 8),
	PS2_GS_PRIM,
	((unsigned long long)0 * 16) + (((unsigned long long)0 * 16) << 16),
	PS2_GS_UV,
	((unsigned long long)0 * 16) + (((unsigned long long)0 * 16) << 16),
	PS2_GS_XYZ2,
	0,			/* 8 */
	PS2_GS_UV,
	0,			/* 10 */
	PS2_GS_XYZ2
};


int scaleimage_nonblock(int fd, unsigned long long *tm, unsigned long long *sm)
{
	struct ps2_plist plist;
	struct ps2_packet packet[2];

	/* initialize the variables */
	plist.num = 2;
	plist.packet = packet;

	packet[0].ptr = tm;
	packet[0].len = sizeof(tex_tags);
	packet[1].ptr = sm;
	packet[1].len = sizeof(scale_tags);

	return ioctl(fd, PS2IOC_SENDL, &plist);
}

static int power_of_2(int value)
{
	int shift;

	for ( shift = 0; (1<<shift) < value; ++shift ) {
		/* Keep looking */ ;
	}
	return(shift);
}

static int GS_VideoInit(_THIS, SDL_PixelFormat *vformat)
{
	struct ps2_screeninfo vinfo;

	/* Initialize the library */
	console_fd = open(PS2_DEV_GS, O_RDWR, 0);
	if ( console_fd < 0 ) {
		SDL_SetError("Unable to open %s", PS2_DEV_GS);
		return(-1);
	}
	memory_fd = open(PS2_DEV_MEM, O_RDWR, 0);
	if ( memory_fd < 0 ) {
		close(console_fd);
		console_fd = -1;
		SDL_SetError("Unable to open %s", PS2_DEV_MEM);
		return(-1);
	}

	if ( ioctl(console_fd, PS2IOC_GSCREENINFO, &vinfo) < 0 ) {
		close(memory_fd);
		close(console_fd);
		console_fd = -1;
		SDL_SetError("Couldn't get console pixel format");
		return(-1);
	}

	/* Determine the current screen size */
	this->info.current_w = vinfo.w;
	this->info.current_h = vinfo.h;

	/* Determine the current screen depth */
	switch (vinfo.psm) {
	    /* Supported pixel formats */
	    case PS2_GS_PSMCT32:
	    case PS2_GS_PSMCT24:
	    case PS2_GS_PSMCT16:
		break;
	    default:
		GS_VideoQuit(this);
		SDL_SetError("Unknown console pixel format: %d", vinfo.psm);
		return(-1);
	}
	vformat->BitsPerPixel = GS_pixelmasks[vinfo.psm].bpp;
	vformat->Rmask = GS_pixelmasks[vinfo.psm].r;
	vformat->Gmask = GS_pixelmasks[vinfo.psm].g;
	vformat->Bmask = GS_pixelmasks[vinfo.psm].b;
	saved_vinfo = vinfo;

	/* Enable mouse and keyboard support */
	if ( GS_OpenKeyboard(this) < 0 ) {
		GS_VideoQuit(this);
		SDL_SetError("Unable to open keyboard");
		return(-1);
	}
	if ( GS_OpenMouse(this) < 0 ) {
		const char *sdl_nomouse;

		sdl_nomouse = SDL_getenv("SDL_NOMOUSE");
		if ( ! sdl_nomouse ) {
			GS_VideoQuit(this);
			SDL_SetError("Unable to open mouse");
			return(-1);
		}
	}

	/* We're done! */
	return(0);
}

static SDL_Rect **GS_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags)
{
	static SDL_Rect GS_vesa_mode_list[] = {
		{ 0, 0, 1280, 1024 },
		{ 0, 0, 1024, 768 },
		{ 0, 0, 800, 600 },
		{ 0, 0, 640, 480 }
	};
	static SDL_Rect *GS_vesa_modes[] = {
		&GS_vesa_mode_list[0],
		&GS_vesa_mode_list[1],
		&GS_vesa_mode_list[2],
		&GS_vesa_mode_list[3],
		NULL
	};
	static SDL_Rect GS_tvout_stretch;
	static SDL_Rect GS_tvout_mode;
	static SDL_Rect *GS_tvout_modes[3];
	SDL_Rect **modes = NULL;

	switch (format->BitsPerPixel) {
	    case 16:
	    case 24:
	    case 32:
		if ( saved_vinfo.mode == PS2_GS_VESA ) {
			modes = GS_vesa_modes;
		} else {
			int i, j = 0;

// FIXME - what's wrong with the stretch code at 16 bpp?
if ( format->BitsPerPixel != 32 ) break;
			/* Add a mode that we could possibly stretch to */
			for ( i=0; GS_vesa_modes[i]; ++i ) {
				if ( (GS_vesa_modes[i]->w == saved_vinfo.w) &&
				     (GS_vesa_modes[i]->h != saved_vinfo.h) ) {
					GS_tvout_stretch.w=GS_vesa_modes[i]->w;
					GS_tvout_stretch.h=GS_vesa_modes[i]->h;
					GS_tvout_modes[j++] = &GS_tvout_stretch;
					break;
				}
			}
			/* Add the current TV video mode */
			GS_tvout_mode.w = saved_vinfo.w;
			GS_tvout_mode.h = saved_vinfo.h;
			GS_tvout_modes[j++] = &GS_tvout_mode;
			GS_tvout_modes[j++] = NULL;

			/* Return the created list of modes */
			modes = GS_tvout_modes;
		}
		break;
	    default:
		break;
	}
	return(modes);
}

/* Various screen update functions available */
static void GS_DMAFullUpdate(_THIS, int numrects, SDL_Rect *rects);

static SDL_Surface *GS_SetVideoMode(_THIS, SDL_Surface *current,
				int width, int height, int bpp, Uint32 flags)
{
	struct ps2_screeninfo vinfo;

	/* Set the terminal into graphics mode */
	if ( GS_EnterGraphicsMode(this) < 0 ) {
		return(NULL);
	}

	/* Set the video mode and get the final screen format */
	if ( ioctl(console_fd, PS2IOC_GSCREENINFO, &vinfo) < 0 ) {
		SDL_SetError("Couldn't get console screen info");
		return(NULL);
	}
	if ( (vinfo.w != width) || (vinfo.h != height) ||
	     (GS_pixelmasks[vinfo.psm].bpp != bpp) ) {
		/* If we're not in VESA mode, we have to scale resolution */
		if ( saved_vinfo.mode == PS2_GS_VESA ) {
			switch (width) {
			    case 640:
				vinfo.res = PS2_GS_640x480;
				break;
			    case 800:
				vinfo.res = PS2_GS_800x600;
				break;
			    case 1024:
				vinfo.res = PS2_GS_1024x768;
				break;
			    case 1280:
				vinfo.res = PS2_GS_1280x1024;
				break;
			    default:
				SDL_SetError("Unsupported resolution: %dx%d\n",
					     width, height);
				return(NULL);
			}
			vinfo.res |= (PS2_GS_75Hz << 8);
			vinfo.w = width;
			vinfo.h = height;
		}
		vinfo.fbp = 0;
		vinfo.psm = GS_formatmap[bpp/8];
		if ( vinfo.psm < 0 ) {
			SDL_SetError("Unsupported depth: %d bpp\n", bpp);
			return(NULL);
		}
		if ( ioctl(console_fd, PS2IOC_SSCREENINFO, &vinfo) < 0 ) {
			SDL_SetError("Couldn't set console screen info");
			return(NULL);
		}

		/* Unmap the previous DMA buffer */
		if ( mapped_mem ) {
			munmap(mapped_mem, mapped_len);
			mapped_mem = NULL;
		}
	}
	if ( ! SDL_ReallocFormat(current, GS_pixelmasks[vinfo.psm].bpp,
	                                  GS_pixelmasks[vinfo.psm].r,
	                                  GS_pixelmasks[vinfo.psm].g,
	                                  GS_pixelmasks[vinfo.psm].b, 0) ) {
		return(NULL);
	}

	/* Set up the new mode framebuffer */
	current->flags = SDL_FULLSCREEN;
	current->w = width;
	current->h = height;
	current->pitch = SDL_CalculatePitch(current);

	/* Memory map the DMA area for block memory transfer */
	if ( ! mapped_mem ) {
		pixels_len = height * current->pitch;
		mapped_len = pixels_len +
		             /* Screen update DMA command area */
		             sizeof(head_tags) + ((2 * MAXTAGS) * 16);
		if ( saved_vinfo.mode != PS2_GS_VESA ) {
			mapped_len += sizeof(tex_tags) + sizeof(scale_tags);
		}
		mapped_mem = mmap(0, mapped_len, PROT_READ|PROT_WRITE,
		                  MAP_SHARED, memory_fd, 0);
		if ( mapped_mem == MAP_FAILED ) {
			SDL_SetError("Unable to map %d bytes for DMA",
			             mapped_len);
			mapped_mem = NULL;
			return(NULL);
		}

		/* Set up the entire screen for DMA transfer */
		screen_image.ptr = mapped_mem;
		screen_image.fbp = 0;
		screen_image.fbw = (vinfo.w + 63) / 64;
		screen_image.psm = vinfo.psm;
		screen_image.x = 0;
		if ( vinfo.h == height ) {
			screen_image.y = 0;
		} else {
			/* Put image offscreen and scale to screen height */
			screen_image.y = vinfo.h;
		}
		screen_image.w = current->w;
		screen_image.h = current->h;

		/* get screen image data size (qword aligned) */
		screen_image_size = (screen_image.w * screen_image.h);
		switch (screen_image.psm) {
		    case PS2_GS_PSMCT32:
			screen_image_size *= 4;
			break;
		    case PS2_GS_PSMCT24:
			screen_image_size *= 3;
			break;
		    case PS2_GS_PSMCT16:
			screen_image_size *= 2;
			break;
		}
		screen_image_size = (screen_image_size + 15) & ~15;

		/* Set up the memory for screen update DMA commands */
		head_tags_mem = (unsigned long long *)
		                (mapped_mem + pixels_len);
		image_tags_mem = (unsigned long long *)
		                 ((caddr_t)head_tags_mem + sizeof(head_tags));
		SDL_memcpy(head_tags_mem, head_tags, sizeof(head_tags));
		if ( saved_vinfo.mode != PS2_GS_VESA ) {
			tex_tags_mem = (unsigned long long *)
		                 ((caddr_t)image_tags_mem + ((2*MAXTAGS)*16));
			scale_tags_mem = (unsigned long long *)
		                 ((caddr_t)tex_tags_mem + sizeof(tex_tags));
			SDL_memcpy(tex_tags_mem, tex_tags, sizeof(tex_tags));
			tex_tags_mem[2] = 
				(vinfo.h * vinfo.w) / 64 +
	          		((unsigned long long)screen_image.fbw << 14) +
	          		((unsigned long long)screen_image.psm << 20) +
	          		((unsigned long long)power_of_2(screen_image.w) << 26) +
	          		((unsigned long long)power_of_2(screen_image.h) << 30) +
	          		((unsigned long long)1 << 34) +
	          		((unsigned long long)1 << 35);
			SDL_memcpy(scale_tags_mem, scale_tags, sizeof(scale_tags));
			scale_tags_mem[8] =
				((unsigned long long)screen_image.w * 16) +
			         (((unsigned long long)screen_image.h * 16) << 16);
			scale_tags_mem[10] =
				((unsigned long long)vinfo.w * 16) +
			         (((unsigned long long)vinfo.h * 16) << 16);
		}
	}
	current->pixels = NULL;
	if ( SDL_getenv("SDL_FULLSCREEN_UPDATE") ) {
		/* Correct semantics */
		current->flags |= SDL_ASYNCBLIT;
	} else {
		/* We lie here - the screen memory isn't really the visible
		   display memory and still requires an update, but this
		   has the desired effect for most applications.
		 */
		current->flags |= SDL_HWSURFACE;
	}

	/* Set the update rectangle function */
	this->UpdateRects = GS_DMAFullUpdate;

	/* We're done */
	return(current);
}

/* We don't support hardware surfaces yet */
static int GS_AllocHWSurface(_THIS, SDL_Surface *surface)
{
	return(-1);
}
static void GS_FreeHWSurface(_THIS, SDL_Surface *surface)
{
	return;
}
static int GS_LockHWSurface(_THIS, SDL_Surface *surface)
{
	if ( surface == this->screen ) {
		/* Since mouse motion affects 'pixels', lock it */
		SDL_LockCursor();

		/* Make sure any pending DMA has completed */
		if ( dma_pending ) {
			ioctl(console_fd, PS2IOC_SENDQCT, 1);
			dma_pending = 0;
		}

		/* If the cursor is drawn on the DMA area, remove it */
		if ( cursor_drawn ) {
			surface->pixels = mapped_mem + surface->offset;
			SDL_EraseCursorNoLock(this->screen);
			cursor_drawn = 0;
		}

		/* Set the surface pixels to the base of the DMA area */
		surface->pixels = mapped_mem;

		/* We're finished! */
		SDL_UnlockCursor();
	}
	return(0);
}
static void GS_UnlockHWSurface(_THIS, SDL_Surface *surface)
{
	if ( surface == this->screen ) {
		/* Since mouse motion affects 'pixels', lock it */
		SDL_LockCursor();
		surface->pixels = NULL;
		SDL_UnlockCursor();
	}
}

static void GS_DMAFullUpdate(_THIS, int numrects, SDL_Rect *rects)
{
	/* Lock so we aren't interrupted by a mouse update */
	SDL_LockCursor();

	/* Make sure any pending DMA has completed */
	if ( dma_pending ) {
		ioctl(console_fd, PS2IOC_SENDQCT, 1);
		dma_pending = 0;
	}

	/* If the mouse is visible, draw it on the DMA area */
	if ( (SDL_cursorstate & CURSOR_VISIBLE) && !cursor_drawn ) {
		this->screen->pixels = mapped_mem + this->screen->offset;
		SDL_DrawCursorNoLock(this->screen);
		this->screen->pixels = NULL;
		cursor_drawn = 1;
	}

	/* Put the image onto the screen */
	loadimage_nonblock(console_fd,
	                   &screen_image, screen_image_size,
	                   head_tags_mem, image_tags_mem);
	if ( screen_image.y > 0 ) {
		/* Need to scale offscreen image to TV output */
		ioctl(console_fd, PS2IOC_SENDQCT, 1);
		dma_pending = 0;
		scaleimage_nonblock(console_fd, tex_tags_mem, scale_tags_mem);
	} else {
		dma_pending = 1;
	}

	/* We're finished! */
	SDL_UnlockCursor();
}

static int GS_SetColors(_THIS, int firstcolor, int ncolors, SDL_Color *colors)
{
	return(0);
}

static void GS_VideoQuit(_THIS)
{
	/* Close console and input file descriptors */
	if ( console_fd > 0 ) {
		/* Unmap the video framebuffer */
		if ( mapped_mem ) {
			/* Unmap the video framebuffer */
			munmap(mapped_mem, mapped_len);
			mapped_mem = NULL;
		}
		close(memory_fd);

		/* Restore the original video mode */
		if ( GS_InGraphicsMode(this) ) {
			ioctl(console_fd, PS2IOC_SSCREENINFO, &saved_vinfo);
		}

		/* We're all done with the graphics device */
		close(console_fd);
		console_fd = -1;
	}
	GS_CloseMouse(this);
	GS_CloseKeyboard(this);
}
