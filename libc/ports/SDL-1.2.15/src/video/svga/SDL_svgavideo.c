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

/* SVGAlib based SDL video driver implementation.
*/

#include <unistd.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/ioctl.h>
#include <fcntl.h>

#if defined(__LINUX__)
#include <linux/vt.h>
#elif defined(__FREEBSD__)
#include <sys/consio.h>
#else
#error You must choose your operating system here
#endif
#include <vga.h>
#include <vgamouse.h>
#include <vgakeyboard.h>

#include "SDL_video.h"
#include "SDL_mouse.h"
#include "../SDL_sysvideo.h"
#include "../SDL_pixels_c.h"
#include "../../events/SDL_events_c.h"
#include "SDL_svgavideo.h"
#include "SDL_svgaevents_c.h"
#include "SDL_svgamouse_c.h"

/* Initialization/Query functions */
static int SVGA_VideoInit(_THIS, SDL_PixelFormat *vformat);
static SDL_Rect **SVGA_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags);
static SDL_Surface *SVGA_SetVideoMode(_THIS, SDL_Surface *current, int width, int height, int bpp, Uint32 flags);
static int SVGA_SetColors(_THIS, int firstcolor, int ncolors,
			  SDL_Color *colors);
static void SVGA_VideoQuit(_THIS);

/* Hardware surface functions */
static int SVGA_AllocHWSurface(_THIS, SDL_Surface *surface);
static int SVGA_LockHWSurface(_THIS, SDL_Surface *surface);
static int SVGA_FlipHWSurface(_THIS, SDL_Surface *surface);
static void SVGA_UnlockHWSurface(_THIS, SDL_Surface *surface);
static void SVGA_FreeHWSurface(_THIS, SDL_Surface *surface);

/* SVGAlib driver bootstrap functions */

static int SVGA_Available(void)
{
	/* Check to see if we are root and stdin is a virtual console */
	int console;
	
	/* SVGALib 1.9.x+ doesn't require root (via /dev/svga) */
	int svgalib2 = -1;

	/* See if we are connected to a virtual terminal */
	console = STDIN_FILENO;
#if 0 /* This is no longer needed, SVGAlib can switch consoles for us */
	if ( console >= 0 ) {
		struct stat sb;
		struct vt_mode dummy;

		if ( (fstat(console, &sb) < 0) ||
		     (ioctl(console, VT_GETMODE, &dummy) < 0) ) {
			console = -1;
		}
	}
#endif /* 0 */

	/* See if SVGAlib 2.0 is available */
	svgalib2 = open("/dev/svga", O_RDONLY);
	if (svgalib2 != -1) {
		close(svgalib2);
	}

	return(((svgalib2 != -1) || (geteuid() == 0)) && (console >= 0));
}

static void SVGA_DeleteDevice(SDL_VideoDevice *device)
{
	SDL_free(device->hidden);
	SDL_free(device);
}

static SDL_VideoDevice *SVGA_CreateDevice(int devindex)
{
	SDL_VideoDevice *device;

	/* Initialize all variables that we clean on shutdown */
	device = (SDL_VideoDevice *)SDL_malloc(sizeof(SDL_VideoDevice));
	if ( device ) {
		SDL_memset(device, 0, (sizeof *device));
		device->hidden = (struct SDL_PrivateVideoData *)
				SDL_malloc((sizeof *device->hidden));
	}
	if ( (device == NULL) || (device->hidden == NULL) ) {
		SDL_OutOfMemory();
		if ( device ) {
			SDL_free(device);
		}
		return(0);
	}
	SDL_memset(device->hidden, 0, (sizeof *device->hidden));

	/* Set the function pointers */
	device->VideoInit = SVGA_VideoInit;
	device->ListModes = SVGA_ListModes;
	device->SetVideoMode = SVGA_SetVideoMode;
	device->SetColors = SVGA_SetColors;
	device->UpdateRects = NULL;
	device->VideoQuit = SVGA_VideoQuit;
	device->AllocHWSurface = SVGA_AllocHWSurface;
	device->CheckHWBlit = NULL;
	device->FillHWRect = NULL;
	device->SetHWColorKey = NULL;
	device->SetHWAlpha = NULL;
	device->LockHWSurface = SVGA_LockHWSurface;
	device->UnlockHWSurface = SVGA_UnlockHWSurface;
	device->FlipHWSurface = SVGA_FlipHWSurface;
	device->FreeHWSurface = SVGA_FreeHWSurface;
	device->SetCaption = NULL;
	device->SetIcon = NULL;
	device->IconifyWindow = NULL;
	device->GrabInput = NULL;
	device->GetWMInfo = NULL;
	device->InitOSKeymap = SVGA_InitOSKeymap;
	device->PumpEvents = SVGA_PumpEvents;

	device->free = SVGA_DeleteDevice;

	return device;
}

VideoBootStrap SVGALIB_bootstrap = {
	"svgalib", "SVGAlib",
	SVGA_Available, SVGA_CreateDevice
};

static int SVGA_AddMode(_THIS, int mode, int actually_add)
{
	int i, j;
	vga_modeinfo *modeinfo;

	modeinfo = vga_getmodeinfo(mode);

	i = modeinfo->bytesperpixel-1;
	if ( i < 0 ) {
		return 0;
	}
	if ( actually_add ) {
		SDL_Rect saved_rect[2];
		int      saved_mode[2];
		int b;

		/* Add the mode, sorted largest to smallest */
		b = 0;
		j = 0;
		while ( (SDL_modelist[i][j]->w > modeinfo->width) ||
			(SDL_modelist[i][j]->h > modeinfo->height) ) {
			++j;
		}
		/* Skip modes that are already in our list */
		if ( (SDL_modelist[i][j]->w == modeinfo->width) &&
		     (SDL_modelist[i][j]->h == modeinfo->height) ) {
			return(0);
		}
		/* Insert the new mode */
		saved_rect[b] = *SDL_modelist[i][j];
		saved_mode[b] = SDL_vgamode[i][j];
		SDL_modelist[i][j]->w = modeinfo->width;
		SDL_modelist[i][j]->h = modeinfo->height;
		SDL_vgamode[i][j] = mode;
		/* Everybody scoot down! */
		if ( saved_rect[b].w && saved_rect[b].h ) {
		    for ( ++j; SDL_modelist[i][j]->w; ++j ) {
			saved_rect[!b] = *SDL_modelist[i][j];
			saved_mode[!b] = SDL_vgamode[i][j];
			*SDL_modelist[i][j] = saved_rect[b];
			SDL_vgamode[i][j] = saved_mode[b];
			b = !b;
		    }
		    *SDL_modelist[i][j] = saved_rect[b];
		    SDL_vgamode[i][j] = saved_mode[b];
		}
	} else {
		++SDL_nummodes[i];
	}
	return(1);
}

static void SVGA_UpdateVideoInfo(_THIS)
{
	vga_modeinfo *modeinfo;

	this->info.wm_available = 0;
	this->info.hw_available = (banked ? 0 : 1);
	modeinfo = vga_getmodeinfo(vga_getcurrentmode());
	this->info.video_mem = modeinfo->memory;
	/* FIXME: Add hardware accelerated blit information */
#ifdef SVGALIB_DEBUG
	printf("Hardware accelerated blit: %savailable\n", modeinfo->haveblit ? "" : "not ");
#endif
}

int SVGA_VideoInit(_THIS, SDL_PixelFormat *vformat)
{
	int keyboard;
	int i, j;
	int mode, total_modes;

	/* Initialize all variables that we clean on shutdown */
	for ( i=0; i<NUM_MODELISTS; ++i ) {
		SDL_nummodes[i] = 0;
		SDL_modelist[i] = NULL;
		SDL_vgamode[i] = NULL;
	}

	/* Initialize the library */
	vga_disabledriverreport();
	if ( vga_init() < 0 ) {
		SDL_SetError("Unable to initialize SVGAlib");
		return(-1);
	}
	vga_setmode(TEXT);

	/* Enable mouse and keyboard support */
	vga_setmousesupport(1);
	keyboard = keyboard_init_return_fd();
	if ( keyboard < 0 ) {
		SDL_SetError("Unable to initialize keyboard");
		return(-1);
	}
	if ( SVGA_initkeymaps(keyboard) < 0 ) {
		return(-1);
	}
	keyboard_seteventhandler(SVGA_keyboardcallback);

	/* Determine the current screen size */
	this->info.current_w = 0;
	this->info.current_h = 0;

	/* Determine the screen depth (use default 8-bit depth) */
	vformat->BitsPerPixel = 8;

	/* Enumerate the available fullscreen modes */
	total_modes = 0;
	for ( mode=vga_lastmodenumber(); mode; --mode ) {
		if ( vga_hasmode(mode) ) {
			if ( SVGA_AddMode(this, mode, 0) ) {
				++total_modes;
			}
		}
	}
	if ( SVGA_AddMode(this, G320x200x256, 0) ) ++total_modes;
	if ( total_modes == 0 ) {
		SDL_SetError("No linear video modes available");
		return(-1);
	}
	for ( i=0; i<NUM_MODELISTS; ++i ) {
		SDL_vgamode[i] = (int *)SDL_malloc(SDL_nummodes[i]*sizeof(int));
		if ( SDL_vgamode[i] == NULL ) {
			SDL_OutOfMemory();
			return(-1);
		}
		SDL_modelist[i] = (SDL_Rect **)
				SDL_malloc((SDL_nummodes[i]+1)*sizeof(SDL_Rect *));
		if ( SDL_modelist[i] == NULL ) {
			SDL_OutOfMemory();
			return(-1);
		}
		for ( j=0; j<SDL_nummodes[i]; ++j ) {
			SDL_modelist[i][j]=(SDL_Rect *)SDL_malloc(sizeof(SDL_Rect));
			if ( SDL_modelist[i][j] == NULL ) {
				SDL_OutOfMemory();
				return(-1);
			}
			SDL_memset(SDL_modelist[i][j], 0, sizeof(SDL_Rect));
		}
		SDL_modelist[i][j] = NULL;
	}
	for ( mode=vga_lastmodenumber(); mode; --mode ) {
		if ( vga_hasmode(mode) ) {
			SVGA_AddMode(this, mode, 1);
		}
	}
	SVGA_AddMode(this, G320x200x256, 1);

	/* Free extra (duplicated) modes */
	for ( i=0; i<NUM_MODELISTS; ++i ) {
		j = 0;
		while ( SDL_modelist[i][j] && SDL_modelist[i][j]->w ) {
			j++;
		}
		while ( SDL_modelist[i][j] ) {
			SDL_free(SDL_modelist[i][j]);
			SDL_modelist[i][j] = NULL;
			j++;
		}
	}

	/* Fill in our hardware acceleration capabilities */
	SVGA_UpdateVideoInfo(this);

	/* We're done! */
	return(0);
}

SDL_Rect **SVGA_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags)
{
	return(SDL_modelist[((format->BitsPerPixel+7)/8)-1]);
}

/* Various screen update functions available */
static void SVGA_DirectUpdate(_THIS, int numrects, SDL_Rect *rects);
static void SVGA_BankedUpdate(_THIS, int numrects, SDL_Rect *rects);

SDL_Surface *SVGA_SetVideoMode(_THIS, SDL_Surface *current,
				int width, int height, int bpp, Uint32 flags)
{
	int mode;
	int vgamode;
	vga_modeinfo *modeinfo;
	int screenpage_len;

	/* Free old pixels if we were in banked mode */
	if ( banked && current->pixels ) {
		free(current->pixels);
		current->pixels = NULL;
	}

	/* Try to set the requested linear video mode */
	bpp = (bpp+7)/8-1;
	for ( mode=0; SDL_modelist[bpp][mode]; ++mode ) {
		if ( (SDL_modelist[bpp][mode]->w == width) &&
		     (SDL_modelist[bpp][mode]->h == height) ) {
			break;
		}
	}
	if ( SDL_modelist[bpp][mode] == NULL ) {
		SDL_SetError("Couldn't find requested mode in list");
		return(NULL);
	}
	vgamode = SDL_vgamode[bpp][mode];
	vga_setmode(vgamode);
	vga_setpage(0);

	if ( (vga_setlinearaddressing() < 0) && (vgamode != G320x200x256) ) {
		banked = 1;
	} else {
		banked = 0;
	}
    
	modeinfo = vga_getmodeinfo(SDL_vgamode[bpp][mode]);

	/* Update hardware acceleration info */
	SVGA_UpdateVideoInfo(this);

	/* Allocate the new pixel format for the screen */
	bpp = (bpp+1)*8;
	if ( (bpp == 16) && (modeinfo->colors == 32768) ) {
		bpp = 15;
	}
	if ( ! SDL_ReallocFormat(current, bpp, 0, 0, 0, 0) ) {
		return(NULL);
	}

	/* Set up the new mode framebuffer */
	current->flags = SDL_FULLSCREEN;
	if ( !banked ) {
		current->flags |= SDL_HWSURFACE;
	}
	if ( bpp == 8 ) {
		/* FIXME: What about DirectColor? */
		current->flags |= SDL_HWPALETTE;
	}
	current->w = width;
	current->h = height;
	current->pitch = modeinfo->linewidth;
	if ( banked ) {
		current->pixels = SDL_malloc(current->h * current->pitch);
		if ( !current->pixels ) {
			SDL_OutOfMemory();
			return(NULL);
		}
	} else {
		current->pixels = vga_getgraphmem();
	}

	/* set double-buffering */
	if ( (flags & SDL_DOUBLEBUF) && !banked )
	{
	    /* length of one screen page in bytes */
	    screenpage_len=current->h*modeinfo->linewidth;

	    /* if start address should be aligned */
	    if ( modeinfo->linewidth_unit )
	    {
		if ( screenpage_len % modeinfo->linewidth_unit )    
		{
		    screenpage_len += modeinfo->linewidth_unit - ( screenpage_len % modeinfo->linewidth_unit );
		}
	    }

	    /* if we heve enough videomemory =  ak je dost videopamete  */
	    if ( modeinfo->memory > ( screenpage_len * 2 / 1024 ) )
	    {
		current->flags |= SDL_DOUBLEBUF;
		flip_page = 0;
		flip_offset[0] = 0;
		flip_offset[1] = screenpage_len;
		flip_address[0] = vga_getgraphmem();
		flip_address[1] = flip_address[0]+screenpage_len;
		SVGA_FlipHWSurface(this,current);
	    }
	} 

	/* Set the blit function */
	if ( banked ) {
		this->UpdateRects = SVGA_BankedUpdate;
	} else {
		this->UpdateRects = SVGA_DirectUpdate;
	}

	/* Set up the mouse handler again (buggy SVGAlib 1.40) */
	mouse_seteventhandler(SVGA_mousecallback);

	/* We're done */
	return(current);
}

/* We don't actually allow hardware surfaces other than the main one */
static int SVGA_AllocHWSurface(_THIS, SDL_Surface *surface)
{
	return(-1);
}
static void SVGA_FreeHWSurface(_THIS, SDL_Surface *surface)
{
	return;
}

/* We need to wait for vertical retrace on page flipped displays */
static int SVGA_LockHWSurface(_THIS, SDL_Surface *surface)
{
	/* The waiting is done in SVGA_FlipHWSurface() */
	return(0);
}
static void SVGA_UnlockHWSurface(_THIS, SDL_Surface *surface)
{
	return;
}

static int SVGA_FlipHWSurface(_THIS, SDL_Surface *surface)
{
	if ( !banked ) {
		vga_setdisplaystart(flip_offset[flip_page]);
		flip_page=!flip_page;
		surface->pixels=flip_address[flip_page];
		vga_waitretrace();
	}
	return(0);
}

static void SVGA_DirectUpdate(_THIS, int numrects, SDL_Rect *rects)
{
	return;
}

static void SVGA_BankedUpdate(_THIS, int numrects, SDL_Rect *rects)
{
	int i, j;
	SDL_Rect *rect;
	int page, vp;
	int x, y, w, h;
	unsigned char *src;
	unsigned char *dst;
	int bpp = this->screen->format->BytesPerPixel;
	int pitch = this->screen->pitch;

	dst = vga_getgraphmem();
	for ( i=0; i < numrects; ++i ) {
		rect = &rects[i];
		x = rect->x;
		y = rect->y;
		w = rect->w * bpp;
		h = rect->h;

		vp = y * pitch + x * bpp;
		src = (unsigned char *)this->screen->pixels + vp;
		page = vp >> 16;
		vp &= 0xffff;
		vga_setpage(page);
		for (j = 0; j < h; j++) {
			if (vp + w > 0x10000) {
				if (vp >= 0x10000) {
					page++;
					vga_setpage(page);
					vp &= 0xffff;
				} else {
					SDL_memcpy(dst + vp, src, 0x10000 - vp);
					page++;
					vga_setpage(page);
					SDL_memcpy(dst, src + 0x10000 - vp,
						 (vp + w) & 0xffff);
					vp = (vp + pitch) & 0xffff;
					src += pitch;
					continue;
				}
			}
			SDL_memcpy(dst + vp, src, w);
			src += pitch;
			vp += pitch;
		}
	}
}

int SVGA_SetColors(_THIS, int firstcolor, int ncolors, SDL_Color *colors)
{
        int i;

	for(i = 0; i < ncolors; i++) {
	        vga_setpalette(firstcolor + i,
			       colors[i].r>>2,
			       colors[i].g>>2,
			       colors[i].b>>2);
	}
	return(1);
}

/* Note:  If we are terminated, this could be called in the middle of
   another SDL video routine -- notably UpdateRects.
*/
void SVGA_VideoQuit(_THIS)
{
	int i, j;

	/* Reset the console video mode */
	if ( this->screen && (this->screen->w && this->screen->h) ) {
		vga_setmode(TEXT);
	}
	keyboard_close();

	/* Free video mode lists */
	for ( i=0; i<NUM_MODELISTS; ++i ) {
		if ( SDL_modelist[i] != NULL ) {
			for ( j=0; SDL_modelist[i][j]; ++j )
				SDL_free(SDL_modelist[i][j]);
			SDL_free(SDL_modelist[i]);
			SDL_modelist[i] = NULL;
		}
		if ( SDL_vgamode[i] != NULL ) {
			SDL_free(SDL_vgamode[i]);
			SDL_vgamode[i] = NULL;
		}
	}
	if ( this->screen ) {
		if ( banked && this->screen->pixels ) {
			SDL_free(this->screen->pixels);
		}
		this->screen->pixels = NULL;
	}
}

