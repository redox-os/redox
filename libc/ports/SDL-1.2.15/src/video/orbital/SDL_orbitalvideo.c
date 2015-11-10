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

/* Orbital SDL video driver implementation
 *
 * Initial work by Ryan C. Gordon (icculus@icculus.org). A good portion
 *  of this was cut-and-pasted from Stephane Peter's work in the AAlib
 *  SDL video driver.  Renamed to "DUMMY" by Sam Lantinga.
 *  Repurposed to ORBITAL by Jeremy Soller.
 */

#include "SDL_video.h"
#include "SDL_mouse.h"
#include "../SDL_sysvideo.h"
#include "../SDL_pixels_c.h"
#include "../../events/SDL_events_c.h"

#include "SDL_orbitalvideo.h"
#include "SDL_orbitalevents_c.h"
#include "SDL_orbitalmouse_c.h"

#include <sys/types.h>
#include <sys/stat.h>
#include <sys/fcntl.h>

#define ORBITALVID_DRIVER_NAME "orbital"

/* Initialization/Query functions */
static int ORBITAL_VideoInit(_THIS, SDL_PixelFormat *vformat);
static SDL_Rect **ORBITAL_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags);
static SDL_Surface *ORBITAL_SetVideoMode(_THIS, SDL_Surface *current, int width, int height, int bpp, Uint32 flags);
static int ORBITAL_SetColors(_THIS, int firstcolor, int ncolors, SDL_Color *colors);
static void ORBITAL_VideoQuit(_THIS);

/* Hardware surface functions */
static int ORBITAL_AllocHWSurface(_THIS, SDL_Surface *surface);
static int ORBITAL_LockHWSurface(_THIS, SDL_Surface *surface);
static void ORBITAL_UnlockHWSurface(_THIS, SDL_Surface *surface);
static void ORBITAL_FreeHWSurface(_THIS, SDL_Surface *surface);

/* etc. */
static void ORBITAL_UpdateRects(_THIS, int numrects, SDL_Rect *rects);

/* ORBITAL driver bootstrap functions */

static int ORBITAL_Available(void)
{
	return(1);
}

static void ORBITAL_DeleteDevice(SDL_VideoDevice *device)
{
	SDL_free(device->hidden);
	SDL_free(device);
}

static SDL_VideoDevice *ORBITAL_CreateDevice(int devindex)
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
	device->VideoInit = ORBITAL_VideoInit;
	device->ListModes = ORBITAL_ListModes;
	device->SetVideoMode = ORBITAL_SetVideoMode;
	device->CreateYUVOverlay = NULL;
	device->SetColors = ORBITAL_SetColors;
	device->UpdateRects = ORBITAL_UpdateRects;
	device->VideoQuit = ORBITAL_VideoQuit;
	device->AllocHWSurface = ORBITAL_AllocHWSurface;
	device->CheckHWBlit = NULL;
	device->FillHWRect = NULL;
	device->SetHWColorKey = NULL;
	device->SetHWAlpha = NULL;
	device->LockHWSurface = ORBITAL_LockHWSurface;
	device->UnlockHWSurface = ORBITAL_UnlockHWSurface;
	device->FlipHWSurface = NULL;
	device->FreeHWSurface = ORBITAL_FreeHWSurface;
	device->SetCaption = NULL;
	device->SetIcon = NULL;
	device->IconifyWindow = NULL;
	device->GrabInput = NULL;
	device->GetWMInfo = NULL;
	device->InitOSKeymap = ORBITAL_InitOSKeymap;
	device->PumpEvents = ORBITAL_PumpEvents;

	device->free = ORBITAL_DeleteDevice;

	return device;
}

VideoBootStrap ORBITAL_bootstrap = {
	ORBITALVID_DRIVER_NAME, "SDL orbital video driver",
	ORBITAL_Available, ORBITAL_CreateDevice
};


int ORBITAL_VideoInit(_THIS, SDL_PixelFormat *vformat)
{
	fprintf(stderr, "WARNING: You are using the SDL orbital video driver!\n");

	/* Determine the screen depth (use default 32-bit depth) */
	/* we change this during the SDL_SetVideoMode implementation... */
	vformat->BitsPerPixel = 32;
	vformat->BytesPerPixel = 4;

	/* We're done! */
	return(0);
}

SDL_Rect **ORBITAL_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags)
{
   	 return (SDL_Rect **) -1;
}

SDL_Surface *ORBITAL_SetVideoMode(_THIS, SDL_Surface *current,
				int width, int height, int bpp, Uint32 flags)
{
	if ( this->hidden->fd ) {
		close( this->hidden->fd );
	}

	if ( this->hidden->buffer ) {
		SDL_free( this->hidden->buffer );
	}

	char path[4096];
	snprintf(path, 4096, "orbital:///0/0/%d/%d/SDL", width, height);
	this->hidden->fd = open(path, O_RDONLY);
	printf("%s at %d\n", path, this->hidden->fd);

	this->hidden->buffer = SDL_malloc(width * height * (bpp / 8));
	if ( ! this->hidden->buffer ) {
		SDL_SetError("Couldn't allocate buffer for requested mode");
		return(NULL);
	}

 	printf("Setting mode %dx%d@%d\n", width, height, bpp);

	SDL_memset(this->hidden->buffer, 0, width * height * (bpp / 8));

	/* Allocate the new pixel format for the screen */
	if ( ! SDL_ReallocFormat(current, bpp, 0, 0, 0, 0) ) {
		SDL_free(this->hidden->buffer);
		this->hidden->buffer = NULL;
		SDL_SetError("Couldn't allocate new pixel format for requested mode");
		return(NULL);
	}

	/* Set up the new mode framebuffer */
	current->flags = flags & SDL_FULLSCREEN;
	this->hidden->w = current->w = width;
	this->hidden->h = current->h = height;
	current->pitch = current->w * (bpp / 8);
	current->pixels = this->hidden->buffer;

	/* We're done */
	return(current);
}

/* We don't actually allow hardware surfaces other than the main one */
static int ORBITAL_AllocHWSurface(_THIS, SDL_Surface *surface)
{
	return(-1);
}
static void ORBITAL_FreeHWSurface(_THIS, SDL_Surface *surface)
{
	return;
}

/* We need to wait for vertical retrace on page flipped displays */
static int ORBITAL_LockHWSurface(_THIS, SDL_Surface *surface)
{
	return(0);
}

static void ORBITAL_UnlockHWSurface(_THIS, SDL_Surface *surface)
{
	return;
}

static void ORBITAL_UpdateRects(_THIS, int numrects, SDL_Rect *rects)
{
	lseek(this->hidden->fd, 0, 0);
	write(this->hidden->fd, this->hidden->buffer, this->hidden->w * this->hidden->h * 4);
	fsync(this->hidden->fd);
}

int ORBITAL_SetColors(_THIS, int firstcolor, int ncolors, SDL_Color *colors)
{
	/* do nothing of note. */
	return(1);
}

/* Note:  If we are terminated, this could be called in the middle of
   another SDL video routine -- notably UpdateRects.
*/
void ORBITAL_VideoQuit(_THIS)
{
	if (this->screen->pixels != NULL)
	{
		SDL_free(this->screen->pixels);
		this->screen->pixels = NULL;
	}
}
