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

/* AAlib based SDL video driver implementation.
*/

#include <unistd.h>
#include <sys/stat.h>


#include "SDL_video.h"
#include "SDL_mouse.h"
#include "../SDL_sysvideo.h"
#include "../SDL_pixels_c.h"
#include "../../events/SDL_events_c.h"

#include "SDL_aavideo.h"
#include "SDL_aaevents_c.h"
#include "SDL_aamouse_c.h"

#include <aalib.h>

/* Initialization/Query functions */
static int AA_VideoInit(_THIS, SDL_PixelFormat *vformat);
static SDL_Rect **AA_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags);
static SDL_Surface *AA_SetVideoMode(_THIS, SDL_Surface *current, int width, int height, int bpp, Uint32 flags);
static int AA_SetColors(_THIS, int firstcolor, int ncolors, SDL_Color *colors);
static void AA_VideoQuit(_THIS);

/* Hardware surface functions */
static int AA_AllocHWSurface(_THIS, SDL_Surface *surface);
static int AA_LockHWSurface(_THIS, SDL_Surface *surface);
static int AA_FlipHWSurface(_THIS, SDL_Surface *surface);
static void AA_UnlockHWSurface(_THIS, SDL_Surface *surface);
static void AA_FreeHWSurface(_THIS, SDL_Surface *surface);

/* Cache the VideoDevice struct */
static struct SDL_VideoDevice *local_this;

/* AAlib driver bootstrap functions */

static int AA_Available(void)
{
	return 1; /* Always available ! */
}

static void AA_DeleteDevice(SDL_VideoDevice *device)
{
	SDL_free(device->hidden);
	SDL_free(device);
}

static SDL_VideoDevice *AA_CreateDevice(int devindex)
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
	device->VideoInit = AA_VideoInit;
	device->ListModes = AA_ListModes;
	device->SetVideoMode = AA_SetVideoMode;
	device->CreateYUVOverlay = NULL;
	device->SetColors = AA_SetColors;
	device->UpdateRects = NULL;
	device->VideoQuit = AA_VideoQuit;
	device->AllocHWSurface = AA_AllocHWSurface;
	device->CheckHWBlit = NULL;
	device->FillHWRect = NULL;
	device->SetHWColorKey = NULL;
	device->SetHWAlpha = NULL;
	device->LockHWSurface = AA_LockHWSurface;
	device->UnlockHWSurface = AA_UnlockHWSurface;
	device->FlipHWSurface = NULL;
	device->FreeHWSurface = AA_FreeHWSurface;
	device->SetCaption = NULL;
	device->SetIcon = NULL;
	device->IconifyWindow = NULL;
	device->GrabInput = NULL;
	device->GetWMInfo = NULL;
	device->InitOSKeymap = AA_InitOSKeymap;
	device->PumpEvents = AA_PumpEvents;

	device->free = AA_DeleteDevice;

	return device;
}

VideoBootStrap AALIB_bootstrap = {
	"aalib", "ASCII Art Library",
	AA_Available, AA_CreateDevice
};

static void AA_ResizeHandler(aa_context *);

int AA_VideoInit(_THIS, SDL_PixelFormat *vformat)
{
	int keyboard;
	int i;

	/* Initialize all variables that we clean on shutdown */
	for ( i=0; i<SDL_NUMMODES; ++i ) {
		SDL_modelist[i] = SDL_malloc(sizeof(SDL_Rect));
		SDL_modelist[i]->x = SDL_modelist[i]->y = 0;
	}
	/* Modes sorted largest to smallest */
	SDL_modelist[0]->w = 1024; SDL_modelist[0]->h = 768;
	SDL_modelist[1]->w = 800; SDL_modelist[1]->h = 600;
	SDL_modelist[2]->w = 640; SDL_modelist[2]->h = 480;
	SDL_modelist[3]->w = 320; SDL_modelist[3]->h = 400;
	SDL_modelist[4]->w = 320; SDL_modelist[4]->h = 240;
	SDL_modelist[5]->w = 320; SDL_modelist[5]->h = 200;
	SDL_modelist[6] = NULL;

	/* Initialize the library */

	AA_mutex = SDL_CreateMutex();

	aa_parseoptions (NULL, NULL, NULL, NULL);

	AA_context = aa_autoinit(&aa_defparams);
	if ( ! AA_context ) {
		SDL_SetError("Unable to initialize AAlib");
		return(-1);
	}

	/* Enable mouse and keyboard support */

	if ( ! aa_autoinitkbd (AA_context, AA_SENDRELEASE) ) {
		SDL_SetError("Unable to initialize AAlib keyboard");
		return(-1);
	}
	if ( ! aa_autoinitmouse (AA_context, AA_SENDRELEASE) ) {
		fprintf(stderr,"Warning: Unable to initialize AAlib mouse");
	}
	AA_rparams = aa_getrenderparams();

	local_this = this;

	aa_resizehandler(AA_context, AA_ResizeHandler);

	fprintf(stderr,"Using AAlib driver: %s (%s)\n", AA_context->driver->name, AA_context->driver->shortname);

	AA_in_x11 = (SDL_strcmp(AA_context->driver->shortname,"X11") == 0);
	/* Determine the screen depth (use default 8-bit depth) */
	vformat->BitsPerPixel = 8;
	vformat->BytesPerPixel = 1;

	/* We're done! */
	return(0);
}

SDL_Rect **AA_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags)
{
     if(format->BitsPerPixel != 8)
 		return NULL;

	 if ( flags & SDL_FULLSCREEN ) {
		 return SDL_modelist;
	 } else {
		 return (SDL_Rect **) -1;
	 }
}

/* From aavga.c
   AAlib does not give us the choice of the actual resolution, thus we have to simulate additional
   resolution by scaling down manually each frame
*/
static void fastscale (register char *b1, register char *b2, int x1, int x2, int y1, int y2)
{
	register int ex, spx = 0, ddx, ddx1;
	int ddy1, ddy, spy = 0, ey;
	int x;
	char *bb1 = b1;
	if (!x1 || !x2 || !y1 || !y2)
		return;
	ddx = x1 + x1;
	ddx1 = x2 + x2;
	if (ddx1 < ddx)
		spx = ddx / ddx1, ddx %= ddx1;
	ddy = y1 + y1;
	ddy1 = y2 + y2;
	if (ddy1 < ddy)
		spy = (ddy / ddy1) * x1, ddy %= ddy1;
	ey = -ddy1;
	for (; y2; y2--) {
		ex = -ddx1;
		for (x = x2; x; x--) {
			*b2 = *b1;
			b2++;
			b1 += spx;
			ex += ddx;
			if (ex > 0) {
				b1++;
				ex -= ddx1;
			}
		}
		bb1 += spy;
		ey += ddy;
		if (ey > 0) {
			bb1 += x1;
			ey -= ddy1;
		}
		b1 = bb1;
	}
}

/* Various screen update functions available */
static void AA_DirectUpdate(_THIS, int numrects, SDL_Rect *rects);

SDL_Surface *AA_SetVideoMode(_THIS, SDL_Surface *current,
				int width, int height, int bpp, Uint32 flags)
{
	int mode;

	if ( AA_buffer ) {
		SDL_free( AA_buffer );
	}

	AA_buffer = SDL_malloc(width * height);
	if ( ! AA_buffer ) {
		SDL_SetError("Couldn't allocate buffer for requested mode");
		return(NULL);
	}

/* 	printf("Setting mode %dx%d\n", width, height); */

	SDL_memset(aa_image(AA_context), 0, aa_imgwidth(AA_context) * aa_imgheight(AA_context));
	SDL_memset(AA_buffer, 0, width * height);

	/* Allocate the new pixel format for the screen */
	if ( ! SDL_ReallocFormat(current, 8, 0, 0, 0, 0) ) {
		return(NULL);
	}

	/* Set up the new mode framebuffer */
	current->flags = SDL_FULLSCREEN;
	AA_w = current->w = width;
	AA_h = current->h = height;
	current->pitch = current->w;
	current->pixels = AA_buffer;

	AA_x_ratio = ((double)aa_imgwidth(AA_context)) / ((double)width);
	AA_y_ratio = ((double)aa_imgheight(AA_context)) / ((double)height);

	/* Set the blit function */
	this->UpdateRects = AA_DirectUpdate;

	/* We're done */
	return(current);
}

static void AA_ResizeHandler(aa_context *context)
{
	aa_resize(context);
	local_this->hidden->x_ratio = ((double)aa_imgwidth(context)) / ((double)local_this->screen->w);
	local_this->hidden->y_ratio = ((double)aa_imgheight(context)) / ((double)local_this->screen->h);

	fastscale (local_this->hidden->buffer, aa_image(context), local_this->hidden->w, aa_imgwidth (context), local_this->hidden->h, aa_imgheight (context));
	aa_renderpalette(context, local_this->hidden->palette, local_this->hidden->rparams, 0, 0, aa_scrwidth(context), aa_scrheight(context));
	aa_flush(context);
}

/* We don't actually allow hardware surfaces other than the main one */
static int AA_AllocHWSurface(_THIS, SDL_Surface *surface)
{
	return(-1);
}
static void AA_FreeHWSurface(_THIS, SDL_Surface *surface)
{
	return;
}

/* We need to wait for vertical retrace on page flipped displays */
static int AA_LockHWSurface(_THIS, SDL_Surface *surface)
{
	/* TODO ? */
	return(0);
}
static void AA_UnlockHWSurface(_THIS, SDL_Surface *surface)
{
	return;
}

/* FIXME: How is this done with AAlib? */
static int AA_FlipHWSurface(_THIS, SDL_Surface *surface)
{
	SDL_mutexP(AA_mutex);
	aa_flush(AA_context);
	SDL_mutexV(AA_mutex);
	return(0);
}

static void AA_DirectUpdate(_THIS, int numrects, SDL_Rect *rects)
{
	int i;
	SDL_Rect *rect;

	fastscale (AA_buffer, aa_image(AA_context), AA_w, aa_imgwidth (AA_context), AA_h, aa_imgheight (AA_context));
#if 1
	aa_renderpalette(AA_context, AA_palette, AA_rparams, 0, 0, aa_scrwidth(AA_context), aa_scrheight(AA_context));
#else
	/* Render only the rectangles in the list */
	printf("Update rects : ");
	for ( i=0; i < numrects; ++i ) {
		rect = &rects[i];
		printf("(%d,%d-%d,%d)", rect->x, rect->y, rect->w, rect->h);
		aa_renderpalette(AA_context, AA_palette, AA_rparams, rect->x * AA_x_ratio, rect->y * AA_y_ratio, rect->w * AA_x_ratio, rect->h * AA_y_ratio);
	}
	printf("\n");
#endif
	SDL_mutexP(AA_mutex);
	aa_flush(AA_context);
	SDL_mutexV(AA_mutex);
	return;
}

int AA_SetColors(_THIS, int firstcolor, int ncolors, SDL_Color *colors)
{
	int i;

	for ( i=0; i < ncolors; i++ ) {
	        aa_setpalette(AA_palette, firstcolor + i,
			      colors[i].r>>2,
			      colors[i].g>>2,
			      colors[i].b>>2);
	}
	return(1);
}

/* Note:  If we are terminated, this could be called in the middle of
   another SDL video routine -- notably UpdateRects.
*/
void AA_VideoQuit(_THIS)
{
	int i;

	aa_uninitkbd(AA_context);
	aa_uninitmouse(AA_context);

	/* Free video mode lists */
	for ( i=0; i<SDL_NUMMODES; ++i ) {
		if ( SDL_modelist[i] != NULL ) {
			SDL_free(SDL_modelist[i]);
			SDL_modelist[i] = NULL;
		}
	}
	
	aa_close(AA_context);

	SDL_DestroyMutex(AA_mutex);

	this->screen->pixels = NULL;	
}
