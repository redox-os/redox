/*
    SDL - Simple DirectMedia Layer
    Copyright (C) 2003  Sam Hocevar

    This library is free software; you can redistribute it and/or
    modify it under the terms of the GNU Library General Public
    License as published by the Free Software Foundation; either
    version 2 of the License, or (at your option) any later version.

    This library is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
    Library General Public License for more details.

    You should have received a copy of the GNU Library General Public
    License along with this library; if not, write to the Free
    Foundation, Inc., 59 Temple Place, Suite 330, Boston, MA  02111-1307  USA

    Sam Hocevar
    sam@zoy.org
*/

#ifdef SAVE_RCSID
static char rcsid =
 "@(#) $Id: libsdl-1.2.11-libcaca.patch,v 1.1 2006/09/18 16:06:06 mr_bones_ Exp $";
#endif

/* libcaca based SDL video driver implementation.
*/

#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>
#include <sys/stat.h>


#include "SDL.h"
#include "SDL_error.h"
#include "SDL_video.h"
#include "SDL_mouse.h"
#include "../SDL_sysvideo.h"
#include "../SDL_pixels_c.h"
#include "../../events/SDL_events_c.h"

#include "SDL_cacavideo.h"
#include "SDL_cacaevents_c.h"

#include <caca.h>
#ifdef CACA_API_VERSION_1
#include <caca0.h>
#endif

/* Initialization/Query functions */
static int Caca_VideoInit(_THIS, SDL_PixelFormat *vformat);
static SDL_Rect **Caca_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags);
static SDL_Surface *Caca_SetVideoMode(_THIS, SDL_Surface *current, int width, int height, int bpp, Uint32 flags);
static void Caca_VideoQuit(_THIS);

/* Hardware surface functions */
static int Caca_AllocHWSurface(_THIS, SDL_Surface *surface);
static int Caca_LockHWSurface(_THIS, SDL_Surface *surface);
static int Caca_FlipHWSurface(_THIS, SDL_Surface *surface);
static void Caca_UnlockHWSurface(_THIS, SDL_Surface *surface);
static void Caca_FreeHWSurface(_THIS, SDL_Surface *surface);

/* Cache the VideoDevice struct */
static struct SDL_VideoDevice *local_this;

/* libcaca driver bootstrap functions */

static int Caca_Available(void)
{
	return 1; /* Always available ! */
}

static void Caca_DeleteDevice(SDL_VideoDevice *device)
{
	free(device->hidden);
	free(device);
}
static SDL_VideoDevice *Caca_CreateDevice(int devindex)
{
	SDL_VideoDevice *device;

	/* Initialize all variables that we clean on shutdown */
	device = (SDL_VideoDevice *)malloc(sizeof(SDL_VideoDevice));
	if ( device ) {
		memset(device, 0, (sizeof *device));
		device->hidden = (struct SDL_PrivateVideoData *)
				malloc((sizeof *device->hidden));
	}
	if ( (device == NULL) || (device->hidden == NULL) ) {
		SDL_OutOfMemory();
		if ( device ) {
			free(device);
		}
		return(0);
	}
	memset(device->hidden, 0, (sizeof *device->hidden));

	/* Set the function pointers */
	device->VideoInit = Caca_VideoInit;
	device->ListModes = Caca_ListModes;
	device->SetVideoMode = Caca_SetVideoMode;
	device->CreateYUVOverlay = NULL;
	device->SetColors = NULL;
	device->UpdateRects = NULL;
	device->VideoQuit = Caca_VideoQuit;
	device->AllocHWSurface = Caca_AllocHWSurface;
	device->CheckHWBlit = NULL;
	device->FillHWRect = NULL;
	device->SetHWColorKey = NULL;
	device->SetHWAlpha = NULL;
	device->LockHWSurface = Caca_LockHWSurface;
	device->UnlockHWSurface = Caca_UnlockHWSurface;
	device->FlipHWSurface = NULL;
	device->FreeHWSurface = Caca_FreeHWSurface;
	device->SetCaption = NULL;
	device->SetIcon = NULL;
	device->IconifyWindow = NULL;
	device->GrabInput = NULL;
	device->GetWMInfo = NULL;
	device->InitOSKeymap = Caca_InitOSKeymap;
	device->PumpEvents = Caca_PumpEvents;

	device->free = Caca_DeleteDevice;

	return device;
}

VideoBootStrap CACA_bootstrap = {
	"caca", "Color ASCII Art Library",
	Caca_Available, Caca_CreateDevice
};

int Caca_VideoInit(_THIS, SDL_PixelFormat *vformat)
{
	int i;

	/* Initialize all variables that we clean on shutdown */
	for ( i=0; i<SDL_NUMMODES; ++i ) {
		SDL_modelist[i] = malloc(sizeof(SDL_Rect));
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

	Caca_mutex = SDL_CreateMutex();

	/* Initialize the library */
	if ( caca_init() != 0 ) {
		SDL_SetError("Unable to initialize libcaca");
		return(-1);
	}

	/* Initialize private variables */
	Caca_lastkey = 0;
	Caca_bitmap = NULL;
	Caca_buffer = NULL;

	local_this = this;

	/* Determine the screen depth (use default 8-bit depth) */
	vformat->BitsPerPixel = 8;
	vformat->BytesPerPixel = 1;

	/* We're done! */
	return(0);
}

SDL_Rect **Caca_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags)
{
     if(format->BitsPerPixel != 8)
 		return NULL;

	 if ( flags & SDL_FULLSCREEN ) {
		 return SDL_modelist;
	 } else {
		 return (SDL_Rect **) -1;
	 }
}

/* Various screen update functions available */
static void Caca_DirectUpdate(_THIS, int numrects, SDL_Rect *rects);

SDL_Surface *Caca_SetVideoMode(_THIS, SDL_Surface *current,
				int width, int height, int bpp, Uint32 flags)
{
	if ( Caca_buffer ) {
		free( Caca_buffer );
		Caca_buffer = NULL;
	}

	if ( Caca_bitmap ) {
		caca_free_bitmap( Caca_bitmap );
		Caca_bitmap = NULL;
	}

	Caca_buffer = malloc(2 * ((width + 15) & ~15) * height);
	if ( ! Caca_buffer ) {
		SDL_SetError("Couldn't allocate buffer for requested mode");
		return(NULL);
	}

	memset(Caca_buffer, 0, 2 * ((width + 15) & ~15) * height);

	/* Allocate the new pixel format for the screen */
	if ( ! SDL_ReallocFormat(current, 16, 0xf800, 0x07e0, 0x001f, 0) ) {
		return(NULL);
	}

	/* Set up the new mode framebuffer */
	current->flags = SDL_FULLSCREEN;
	Caca_w = current->w = width;
	Caca_h = current->h = height;
	current->pitch = 2 * ((width + 15) & ~15);
	current->pixels = Caca_buffer;

	/* Create the libcaca bitmap */
	Caca_bitmap = caca_create_bitmap( 16, width, height, current->pitch, 0xf800, 0x07e0, 0x001f, 0x0000 );
	if ( ! Caca_bitmap ) {
		SDL_SetError("Couldn't allocate libcaca bitmap");
		return(NULL);
	}

	/* Set the blit function */
	this->UpdateRects = Caca_DirectUpdate;

	/* We're done */
	return(current);
}

/* We don't actually allow hardware surfaces other than the main one */
static int Caca_AllocHWSurface(_THIS, SDL_Surface *surface)
{
	return(-1);
}
static void Caca_FreeHWSurface(_THIS, SDL_Surface *surface)
{
	return;
}

/* We need to wait for vertical retrace on page flipped displays */
static int Caca_LockHWSurface(_THIS, SDL_Surface *surface)
{
	/* TODO ? */
	return(0);
}
static void Caca_UnlockHWSurface(_THIS, SDL_Surface *surface)
{
	return;
}

/* FIXME: How is this done with libcaca? */
static int Caca_FlipHWSurface(_THIS, SDL_Surface *surface)
{
	SDL_mutexP(Caca_mutex);
	caca_refresh();
	SDL_mutexV(Caca_mutex);
	return(0);
}

static void Caca_DirectUpdate(_THIS, int numrects, SDL_Rect *rects)
{
	SDL_mutexP(Caca_mutex);
	caca_draw_bitmap( 0, 0, caca_get_width() - 1, caca_get_height() - 1,
			  Caca_bitmap, Caca_buffer );
	caca_refresh();
	SDL_mutexV(Caca_mutex);
	return;
}

/* Note:  If we are terminated, this could be called in the middle of
   another SDL video routine -- notably UpdateRects.
*/
void Caca_VideoQuit(_THIS)
{
	int i;

	/* Free video mode lists */
	for ( i=0; i<SDL_NUMMODES; ++i ) {
		if ( SDL_modelist[i] != NULL ) {
			free(SDL_modelist[i]);
			SDL_modelist[i] = NULL;
		}
	}

	if ( Caca_bitmap ) {
		caca_free_bitmap( Caca_bitmap );
		Caca_bitmap = NULL;
	}

	caca_end();

	SDL_DestroyMutex(Caca_mutex);
}

