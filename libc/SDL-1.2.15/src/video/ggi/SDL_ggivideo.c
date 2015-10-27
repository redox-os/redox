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

/* GGI-based SDL video driver implementation.
*/

#include <fcntl.h>
#include <unistd.h>
#include <sys/mman.h>

#include <ggi/ggi.h>
#include <ggi/gii.h>

#include "SDL_video.h"
#include "SDL_mouse.h"
#include "../SDL_sysvideo.h"
#include "../SDL_pixels_c.h"
#include "../../events/SDL_events_c.h"
#include "SDL_ggivideo.h"
#include "SDL_ggimouse_c.h"
#include "SDL_ggievents_c.h"


struct private_hwdata
{
	ggi_visual_t vis;
};

ggi_visual_t VIS;

/* Initialization/Query functions */
static int GGI_VideoInit(_THIS, SDL_PixelFormat *vformat);
static SDL_Rect **GGI_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags);
static SDL_Surface *GGI_SetVideoMode(_THIS, SDL_Surface *current, int width, int height, int bpp, Uint32 flags);
static int GGI_SetColors(_THIS, int firstcolor, int ncolors,
			 SDL_Color *colors);
static void GGI_VideoQuit(_THIS);

/* Hardware surface functions */
static int GGI_AllocHWSurface(_THIS, SDL_Surface *surface);
static int GGI_LockHWSurface(_THIS, SDL_Surface *surface);
static void GGI_UnlockHWSurface(_THIS, SDL_Surface *surface);
static void GGI_FreeHWSurface(_THIS, SDL_Surface *surface);

/* GGI driver bootstrap functions */

static int GGI_Available(void)
{
	ggi_visual_t *vis;

	vis = NULL;
	if (ggiInit() == 0) {
		vis = ggiOpen(NULL);
		if (vis != NULL) {
			ggiClose(vis);
		}
	}
	return (vis != NULL);
}

static void GGI_DeleteDevice(SDL_VideoDevice *device)
{
	SDL_free(device->hidden);
	SDL_free(device);
}

static SDL_VideoDevice *GGI_CreateDevice(int devindex)
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
	device->VideoInit = GGI_VideoInit;
	device->ListModes = GGI_ListModes;
	device->SetVideoMode = GGI_SetVideoMode;
	device->SetColors = GGI_SetColors;
	device->UpdateRects = NULL;
	device->VideoQuit = GGI_VideoQuit;
	device->AllocHWSurface = GGI_AllocHWSurface;
	device->CheckHWBlit = NULL;
	device->FillHWRect = NULL;
	device->SetHWColorKey = NULL;
	device->SetHWAlpha = NULL;
	device->LockHWSurface = GGI_LockHWSurface;
	device->UnlockHWSurface = GGI_UnlockHWSurface;
	device->FlipHWSurface = NULL;
	device->FreeHWSurface = GGI_FreeHWSurface;
	device->SetCaption = NULL;
	device->SetIcon = NULL;
	device->IconifyWindow = NULL;
	device->GrabInput = NULL;
	device->GetWMInfo = NULL;
	device->InitOSKeymap = GGI_InitOSKeymap;
	device->PumpEvents = GGI_PumpEvents;

	device->free = GGI_DeleteDevice;

	return device;
}

VideoBootStrap GGI_bootstrap = {
	"ggi", "General Graphics Interface (GGI)",
	GGI_Available, GGI_CreateDevice
};


static SDL_Rect video_mode;
static SDL_Rect *SDL_modelist[4] = { NULL, NULL, NULL, NULL };

int GGI_VideoInit(_THIS, SDL_PixelFormat *vformat)
{
	ggi_mode mode =
	{
		1,
		{ GGI_AUTO, GGI_AUTO },
		{ GGI_AUTO, GGI_AUTO },
		{ 0, 0 },
		GT_AUTO,
		{ GGI_AUTO, GGI_AUTO }
	};	
	struct private_hwdata *priv;
	ggi_color pal[256], map[256];
	const ggi_directbuffer *db;
	int err, num_bufs;
	ggi_pixel white, black;
	
	priv = SDL_malloc(sizeof(struct private_hwdata));
	if (priv == NULL)
	{
		SDL_SetError("Unhandled GGI mode type!\n");
		GGI_VideoQuit(NULL);
	}
	
	if (ggiInit() != 0)
	{
		SDL_SetError("Unable to initialize GGI!\n");
		GGI_VideoQuit(NULL);
	}
	
	VIS = ggiOpen(NULL);
	if (VIS == NULL)
	{
		SDL_SetError("Unable to open default GGI visual!\n");
		ggiExit();
		GGI_VideoQuit(NULL);
	}
	
	ggiSetFlags(VIS, GGIFLAG_ASYNC);
	
	/* Validate mode, autodetecting any GGI_AUTO or GT_AUTO fields */
	ggiCheckMode(VIS, &mode);
	
	/* At this point we should have a valid mode - try to set it */
	err = ggiSetMode(VIS, &mode);
	
	/* If we couldn't set _any_ modes, something is very wrong */
	if (err)
	{
		SDL_SetError("Can't set a mode!\n");
		ggiClose(VIS);
		ggiExit();
		GGI_VideoQuit(NULL);
	}

	/* Determine the current screen size */
	this->info.current_w = mode.virt.x;
	this->info.current_h = mode.virt.y;

	/* Set a palette for palletized modes */
	if (GT_SCHEME(mode.graphtype) == GT_PALETTE)
	{
		ggiSetColorfulPalette(VIS);
		ggiGetPalette(VIS, 0, 1 << vformat->BitsPerPixel, pal);
	}
	
	/* Now we try to get the DirectBuffer info, which determines whether
	 * SDL can access hardware surfaces directly. */
	
	num_bufs = ggiDBGetNumBuffers(VIS);

	if (num_bufs > 0)
	{
		db = ggiDBGetBuffer(VIS, 0); /* Only handle one DB for now */
		
		vformat->BitsPerPixel = db->buffer.plb.pixelformat->depth;
		
		vformat->Rmask = db->buffer.plb.pixelformat->red_mask;
		vformat->Gmask = db->buffer.plb.pixelformat->green_mask;
		vformat->Bmask = db->buffer.plb.pixelformat->blue_mask;
		
		/* Fill in our hardware acceleration capabilities */
	
		this->info.wm_available = 0;
		this->info.hw_available = 1;
		this->info.video_mem = db->buffer.plb.stride * mode.virt.y;
	}

	video_mode.x = 0;
	video_mode.y = 0;
	video_mode.w = mode.virt.x;
	video_mode.h = mode.virt.y;
	SDL_modelist[((vformat->BitsPerPixel + 7) / 8) - 1] = &video_mode;

	/* We're done! */
	return(0);
}

static SDL_Rect **GGI_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags)
{
	return(&SDL_modelist[((format->BitsPerPixel + 7) / 8) - 1]);
}

/* Various screen update functions available */
static void GGI_DirectUpdate(_THIS, int numrects, SDL_Rect *rects);

SDL_Surface *GGI_SetVideoMode(_THIS, SDL_Surface *current, int width, int height, int bpp, Uint32 flags)
{
	ggi_mode mode =
	{
		1,
		{ GGI_AUTO, GGI_AUTO },
		{ GGI_AUTO, GGI_AUTO },
		{ 0, 0 },
		GT_AUTO,
		{ GGI_AUTO, GGI_AUTO }
	};
        const ggi_directbuffer *db;
	ggi_color pal[256];
	int err;

	fprintf(stderr, "GGI_SetVideoMode()\n");
	
	mode.visible.x = mode.virt.x = width;
	mode.visible.y = mode.virt.y = height;
	
	/* Translate requested SDL bit depth into a GGI mode */	
	switch (bpp)
	{
		case 1:  mode.graphtype = GT_1BIT;  break;
		case 2:  mode.graphtype = GT_2BIT;  break;
		case 4:  mode.graphtype = GT_4BIT;  break;
		case 8:  mode.graphtype = GT_8BIT;  break;
		case 15: mode.graphtype = GT_15BIT; break;
		case 16: mode.graphtype = GT_16BIT; break;
		case 24: mode.graphtype = GT_24BIT; break;
		case 32: mode.graphtype = GT_32BIT; break;
		default:
		SDL_SetError("Unknown SDL bit depth, using GT_AUTO....\n");
		mode.graphtype = GT_AUTO;
	}
	
	/* Validate mode, autodetecting any GGI_AUTO or GT_AUTO fields */
	ggiCheckMode(VIS, &mode);
	
	/* At this point we should have a valid mode - try to set it */
	err = ggiSetMode(VIS, &mode);

	/* If we couldn't set _any_ modes, something is very wrong */
	if (err)
	{
		SDL_SetError("Can't set a mode!\n");
		ggiClose(VIS);
		ggiExit();
		GGI_VideoQuit(NULL);
	}
	
	/* Set a palette for palletized modes */
	if (GT_SCHEME(mode.graphtype) == GT_PALETTE)
	{
		ggiSetColorfulPalette(VIS);
		ggiGetPalette(VIS, 0, 1 << bpp, pal);
	}
	
	db = ggiDBGetBuffer(VIS, 0);
	
	/* Set up the new mode framebuffer */
	current->flags = (SDL_FULLSCREEN | SDL_HWSURFACE);
	current->w = mode.virt.x;
	current->h = mode.virt.y;
	current->pitch = db->buffer.plb.stride;
	current->pixels = db->read;

	/* Set the blit function */
	this->UpdateRects = GGI_DirectUpdate;

	/* We're done */
	return(current);
}

static int GGI_AllocHWSurface(_THIS, SDL_Surface *surface)
{
	return(-1);
}
static void GGI_FreeHWSurface(_THIS, SDL_Surface *surface)
{
	return;
}
static int GGI_LockHWSurface(_THIS, SDL_Surface *surface)
{
	return(0);
}
static void GGI_UnlockHWSurface(_THIS, SDL_Surface *surface)
{
	return;
}

static void GGI_DirectUpdate(_THIS, int numrects, SDL_Rect *rects)
{
	int i;
	
/*	ggiFlush(VIS); */
	
	for (i = 0; i < numrects; i++)
	{
		ggiFlushRegion(VIS, rects[i].x, rects[i].y, rects[i].w, rects[i].h);
	}
	return;
}

int GGI_SetColors(_THIS, int firstcolor, int ncolors, SDL_Color *colors)
{
	int i;
	ggi_color pal[256];

	/* Set up the colormap */
	for (i = 0; i < ncolors; i++)
	{
		pal[i].r = (colors[i].r << 8) | colors[i].r;
		pal[i].g = (colors[i].g << 8) | colors[i].g;
		pal[i].b = (colors[i].b << 8) | colors[i].b;
	}
	
	ggiSetPalette(VIS, firstcolor, ncolors, pal);
	
	return 1;
}
	
void GGI_VideoQuit(_THIS)
{
}
void GGI_FinalQuit(void) 
{
}
