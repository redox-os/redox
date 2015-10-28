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

/* DGA 2.0 based SDL video driver implementation.
*/

#include <stdio.h>

#include <X11/Xlib.h>
#include "../Xext/extensions/xf86dga.h"

#include "SDL_video.h"
#include "SDL_mouse.h"
#include "../SDL_sysvideo.h"
#include "../SDL_pixels_c.h"
#include "../../events/SDL_events_c.h"
#include "SDL_dgavideo.h"
#include "SDL_dgamouse_c.h"
#include "SDL_dgaevents_c.h"

/* get function pointers... */
#include "../x11/SDL_x11dyn.h"

/*#define DGA_DEBUG*/

/* Initialization/Query functions */
static int DGA_VideoInit(_THIS, SDL_PixelFormat *vformat);
static SDL_Rect **DGA_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags);
static SDL_Surface *DGA_SetVideoMode(_THIS, SDL_Surface *current, int width, int height, int bpp, Uint32 flags);
static int DGA_SetColors(_THIS, int firstcolor, int ncolors,
			 SDL_Color *colors);
static int DGA_SetGammaRamp(_THIS, Uint16 *ramp);
static void DGA_VideoQuit(_THIS);

/* Hardware surface functions */
static int DGA_InitHWSurfaces(_THIS, SDL_Surface *screen, Uint8 *base, int size);
static void DGA_FreeHWSurfaces(_THIS);
static int DGA_AllocHWSurface(_THIS, SDL_Surface *surface);
static int DGA_FillHWRect(_THIS, SDL_Surface *dst, SDL_Rect *rect, Uint32 color);
static int DGA_CheckHWBlit(_THIS, SDL_Surface *src, SDL_Surface *dst);
static int DGA_LockHWSurface(_THIS, SDL_Surface *surface);
static void DGA_UnlockHWSurface(_THIS, SDL_Surface *surface);
static void DGA_FreeHWSurface(_THIS, SDL_Surface *surface);
static int DGA_FlipHWSurface(_THIS, SDL_Surface *surface);

/* DGA driver bootstrap functions */

static int DGA_Available(void)
{
	const char *display = NULL;
	Display *dpy = NULL;
	int available = 0;

	/* The driver is available is available if the display is local
	   and the DGA 2.0+ extension is available, and we can map mem.
	*/
	if ( SDL_X11_LoadSymbols() ) {
		if ( (SDL_strncmp(XDisplayName(display), ":", 1) == 0) ||
		     (SDL_strncmp(XDisplayName(display), "unix:", 5) == 0) ) {
			dpy = XOpenDisplay(display);
			if ( dpy ) {
				int events, errors, major, minor;

				if ( SDL_NAME(XDGAQueryExtension)(dpy, &events, &errors) &&
				     SDL_NAME(XDGAQueryVersion)(dpy, &major, &minor) ) {
					int screen;

					screen = DefaultScreen(dpy);
					if ( (major >= 2) && 
					     SDL_NAME(XDGAOpenFramebuffer)(dpy, screen) ) {
						available = 1;
						SDL_NAME(XDGACloseFramebuffer)(dpy, screen);
					}
				}
				XCloseDisplay(dpy);
			}
		}
		SDL_X11_UnloadSymbols();
	}
	return(available);
}

static void DGA_DeleteDevice(SDL_VideoDevice *device)
{
	if (device != NULL) {
		SDL_free(device->hidden);
		SDL_free(device);
		SDL_X11_UnloadSymbols();
	}
}

static SDL_VideoDevice *DGA_CreateDevice(int devindex)
{
	SDL_VideoDevice *device = NULL;

	/* Initialize all variables that we clean on shutdown */
	if (SDL_X11_LoadSymbols()) {
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
			SDL_X11_UnloadSymbols();
			return(0);
		}
		SDL_memset(device->hidden, 0, (sizeof *device->hidden));

		/* Set the function pointers */
		device->VideoInit = DGA_VideoInit;
		device->ListModes = DGA_ListModes;
		device->SetVideoMode = DGA_SetVideoMode;
		device->SetColors = DGA_SetColors;
		device->UpdateRects = NULL;
		device->VideoQuit = DGA_VideoQuit;
		device->AllocHWSurface = DGA_AllocHWSurface;
		device->CheckHWBlit = DGA_CheckHWBlit;
		device->FillHWRect = DGA_FillHWRect;
		device->SetHWColorKey = NULL;
		device->SetHWAlpha = NULL;
		device->LockHWSurface = DGA_LockHWSurface;
		device->UnlockHWSurface = DGA_UnlockHWSurface;
		device->FlipHWSurface = DGA_FlipHWSurface;
		device->FreeHWSurface = DGA_FreeHWSurface;
		device->SetGammaRamp = DGA_SetGammaRamp;
		device->GetGammaRamp = NULL;
		device->SetCaption = NULL;
		device->SetIcon = NULL;
		device->IconifyWindow = NULL;
		device->GrabInput = NULL;
		device->GetWMInfo = NULL;
		device->InitOSKeymap = DGA_InitOSKeymap;
		device->PumpEvents = DGA_PumpEvents;

		device->free = DGA_DeleteDevice;
	}

	return device;
}

VideoBootStrap DGA_bootstrap = {
	"dga", "XFree86 DGA 2.0",
	DGA_Available, DGA_CreateDevice
};

static int DGA_AddMode(_THIS, int bpp, int w, int h)
{
	SDL_Rect *mode;
	int index;
	int next_mode;

	/* Check to see if we already have this mode */
	if ( bpp < 8 ) {  /* Not supported */
		return(0);
	}
	index = ((bpp+7)/8)-1;
	if ( SDL_nummodes[index] > 0 ) {
		mode = SDL_modelist[index][SDL_nummodes[index]-1];
		if ( (mode->w == w) && (mode->h == h) ) {
			return(0);
		}
	}

	/* Set up the new video mode rectangle */
	mode = (SDL_Rect *)SDL_malloc(sizeof *mode);
	if ( mode == NULL ) {
		SDL_OutOfMemory();
		return(-1);
	}
	mode->x = 0;
	mode->y = 0;
	mode->w = w;
	mode->h = h;

	/* Allocate the new list of modes, and fill in the new mode */
	next_mode = SDL_nummodes[index];
	SDL_modelist[index] = (SDL_Rect **)
	       SDL_realloc(SDL_modelist[index], (1+next_mode+1)*sizeof(SDL_Rect *));
	if ( SDL_modelist[index] == NULL ) {
		SDL_OutOfMemory();
		SDL_nummodes[index] = 0;
		SDL_free(mode);
		return(-1);
	}
	SDL_modelist[index][next_mode] = mode;
	SDL_modelist[index][next_mode+1] = NULL;
	SDL_nummodes[index]++;

	return(0);
}

/* This whole function is a hack. :) */
static Uint32 get_video_size(_THIS)
{
	/* This is a non-exported function from libXxf86dga.a */
	extern unsigned char *SDL_NAME(XDGAGetMappedMemory)(int screen);
	FILE *proc;
	unsigned long mem;
	unsigned start, stop;
	char line[BUFSIZ];
	Uint32 size;

	mem = (unsigned long)SDL_NAME(XDGAGetMappedMemory)(DGA_Screen);
	size = 0;
	proc = fopen("/proc/self/maps", "r");
	if ( proc ) {
		while ( fgets(line, sizeof(line)-1, proc) ) {
			SDL_sscanf(line, "%x-%x", &start, &stop);
			if ( start == mem ) {
				size = (Uint32)((stop-start)/1024);
				break;
			}
		}
		fclose(proc);
	}
	return(size);
}

#ifdef DGA_DEBUG
static void PrintMode(SDL_NAME(XDGAMode) *mode)
{
	printf("Mode: %s (%dx%d) at %d bpp (%f refresh, %d pitch) num: %d\n",
		mode->name,
		mode->viewportWidth, mode->viewportHeight,
		mode->depth == 24 ? mode->bitsPerPixel : mode->depth,
		mode->verticalRefresh, mode->bytesPerScanline, mode->num);
	printf("\tRGB: 0x%8.8x 0x%8.8x 0x%8.8x (%d - %s)\n",
		mode->redMask, mode->greenMask, mode->blueMask,
		mode->visualClass,
		mode->visualClass == TrueColor ? "truecolor" :
		mode->visualClass == DirectColor ? "directcolor" :
		mode->visualClass == PseudoColor ? "pseudocolor" : "unknown");
	printf("\tFlags: ");
	if ( mode->flags & XDGAConcurrentAccess )
		printf(" XDGAConcurrentAccess");
	if ( mode->flags & XDGASolidFillRect )
		printf(" XDGASolidFillRect");
	if ( mode->flags & XDGABlitRect )
		printf(" XDGABlitRect");
	if ( mode->flags & XDGABlitTransRect )
		printf(" XDGABlitTransRect");
	if ( mode->flags & XDGAPixmap )
		printf(" XDGAPixmap");
	if ( mode->flags & XDGAInterlaced )
		printf(" XDGAInterlaced");
	if ( mode->flags & XDGADoublescan )
		printf(" XDGADoublescan");
	if ( mode->viewportFlags & XDGAFlipRetrace )
		printf(" XDGAFlipRetrace");
	if ( mode->viewportFlags & XDGAFlipImmediate )
		printf(" XDGAFlipImmediate");
	printf("\n");
}
#endif /* DGA_DEBUG */

static int cmpmodes(const void *va, const void *vb)
{
    const SDL_NAME(XDGAMode) *a = (const SDL_NAME(XDGAMode) *)va;
    const SDL_NAME(XDGAMode) *b = (const SDL_NAME(XDGAMode) *)vb;

    if ( (a->viewportWidth == b->viewportWidth) &&
         (b->viewportHeight == a->viewportHeight) ) {
        /* Prefer 32 bpp over 24 bpp, 16 bpp over 15 bpp */
        int a_bpp = a->depth == 24 ? a->bitsPerPixel : a->depth;
        int b_bpp = b->depth == 24 ? b->bitsPerPixel : b->depth;
        if ( a_bpp != b_bpp ) {
            return b_bpp - a_bpp;
        }
        /* Prefer DirectColor visuals, for gamma support */
        if ( a->visualClass == DirectColor && b->visualClass != DirectColor )
            return -1;
        if ( b->visualClass == DirectColor && a->visualClass != DirectColor )
            return 1;
        /* Maintain server refresh rate sorting */
        return a->num - b->num;
    } else if ( a->viewportWidth == b->viewportWidth ) {
        return b->viewportHeight - a->viewportHeight;
    } else {
        return b->viewportWidth - a->viewportWidth;
    }
}
static void UpdateHWInfo(_THIS, SDL_NAME(XDGAMode) *mode)
{
	this->info.wm_available = 0;
	this->info.hw_available = 1;
	if ( mode->flags & XDGABlitRect ) {
		this->info.blit_hw = 1;
	} else {
		this->info.blit_hw = 0;
	}
	if ( mode->flags & XDGABlitTransRect ) {
		this->info.blit_hw_CC = 1;
	} else {
		this->info.blit_hw_CC = 0;
	}
	if ( mode->flags & XDGASolidFillRect ) {
		this->info.blit_fill = 1;
	} else {
		this->info.blit_fill = 0;
	}
	this->info.video_mem = get_video_size(this);
}

static int DGA_VideoInit(_THIS, SDL_PixelFormat *vformat)
{
	const char *env;
	const char *display;
	int event_base, error_base;
	int major_version, minor_version;
	Visual *visual;
	SDL_NAME(XDGAMode) *modes;
	int i, num_modes;

	/* Open the X11 display */
	display = NULL;		/* Get it from DISPLAY environment variable */

	DGA_Display = XOpenDisplay(display);
	if ( DGA_Display == NULL ) {
		SDL_SetError("Couldn't open X11 display");
		return(-1);
	}

	/* Check for the DGA extension */
	if ( ! SDL_NAME(XDGAQueryExtension)(DGA_Display, &event_base, &error_base) ||
	     ! SDL_NAME(XDGAQueryVersion)(DGA_Display, &major_version, &minor_version) ) {
		SDL_SetError("DGA extension not available");
		XCloseDisplay(DGA_Display);
		return(-1);
	}
	if ( major_version < 2 ) {
		SDL_SetError("DGA driver requires DGA 2.0 or newer");
		XCloseDisplay(DGA_Display);
		return(-1);
	}
	DGA_event_base = event_base;

	/* Determine the current screen size */
	this->info.current_w = DisplayWidth(DGA_Display, DGA_Screen);
	this->info.current_h = DisplayHeight(DGA_Display, DGA_Screen);

	/* Determine the current screen depth */
	visual = DefaultVisual(DGA_Display, DGA_Screen);
	{
		XPixmapFormatValues *pix_format;
		int i, num_formats;

		vformat->BitsPerPixel = DefaultDepth(DGA_Display, DGA_Screen);
		pix_format = XListPixmapFormats(DGA_Display, &num_formats);
		if ( pix_format == NULL ) {
			SDL_SetError("Couldn't determine screen formats");
			XCloseDisplay(DGA_Display);
			return(-1);
		}
		for ( i=0; i<num_formats; ++i ) {
			if ( vformat->BitsPerPixel == pix_format[i].depth )
				break;
		}
		if ( i != num_formats )
			vformat->BitsPerPixel = pix_format[i].bits_per_pixel;
		XFree((char *)pix_format);
	}
	if ( vformat->BitsPerPixel > 8 ) {
		vformat->Rmask = visual->red_mask;
		vformat->Gmask = visual->green_mask;
		vformat->Bmask = visual->blue_mask;
	}

	/* Open access to the framebuffer */
	if ( ! SDL_NAME(XDGAOpenFramebuffer)(DGA_Display, DGA_Screen) ) {
		SDL_SetError("Unable to map the video memory");
		XCloseDisplay(DGA_Display);
		return(-1);
	}

	/* Allow environment override of screensaver disable. */
	env = SDL_getenv("SDL_VIDEO_ALLOW_SCREENSAVER");
	if ( env ) {
		allow_screensaver = SDL_atoi(env);
	} else {
#ifdef SDL_VIDEO_DISABLE_SCREENSAVER
		allow_screensaver = 0;
#else
		allow_screensaver = 1;
#endif
	}

	/* Query for the list of available video modes */
	modes = SDL_NAME(XDGAQueryModes)(DGA_Display, DGA_Screen, &num_modes);
	SDL_qsort(modes, num_modes, sizeof *modes, cmpmodes);
	for ( i=0; i<num_modes; ++i ) {
		if ( ((modes[i].visualClass == PseudoColor) ||
		      (modes[i].visualClass == DirectColor) ||
		      (modes[i].visualClass == TrueColor)) && 
		     !(modes[i].flags & (XDGAInterlaced|XDGADoublescan)) ) {
#ifdef DGA_DEBUG
			PrintMode(&modes[i]);
#endif
			DGA_AddMode(this, modes[i].bitsPerPixel,
			            modes[i].viewportWidth,
			            modes[i].viewportHeight);
		}
	}
	UpdateHWInfo(this, modes);
	XFree(modes);

	/* Create the hardware surface lock mutex */
	hw_lock = SDL_CreateMutex();
	if ( hw_lock == NULL ) {
		SDL_SetError("Unable to create lock mutex");
		DGA_VideoQuit(this);
		return(-1);
	}

#ifdef LOCK_DGA_DISPLAY
	/* Create the event lock so we're thread-safe.. :-/ */
	event_lock = SDL_CreateMutex();
#endif /* LOCK_DGA_DISPLAY */

	/* We're done! */
	return(0);
}

SDL_Rect **DGA_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags)
{
	return(SDL_modelist[((format->BitsPerPixel+7)/8)-1]);
}

/* Various screen update functions available */
static void DGA_DirectUpdate(_THIS, int numrects, SDL_Rect *rects);

SDL_Surface *DGA_SetVideoMode(_THIS, SDL_Surface *current,
				int width, int height, int bpp, Uint32 flags)
{
	SDL_NAME(XDGAMode) *modes;
	int i, num_modes;
	SDL_NAME(XDGADevice) *mode;
	int screen_len;
	Uint8 *surfaces_mem;
	int surfaces_len;

	/* Free any previous colormap */
	if ( DGA_colormap ) {
		XFreeColormap(DGA_Display, DGA_colormap);
		DGA_colormap = 0;
	}

	/* Search for a matching video mode */
	modes = SDL_NAME(XDGAQueryModes)(DGA_Display, DGA_Screen, &num_modes);
	SDL_qsort(modes, num_modes, sizeof *modes, cmpmodes);
	for ( i=0; i<num_modes; ++i ) {
		int depth;

		depth = modes[i].depth;
		if ( depth == 24 ) { /* Distinguish between 24 and 32 bpp */
			depth = modes[i].bitsPerPixel;
		}
		if ( (depth == bpp) &&
		     (modes[i].viewportWidth == width) &&
		     (modes[i].viewportHeight == height) &&
		     ((modes[i].visualClass == PseudoColor) ||
		      (modes[i].visualClass == DirectColor) ||
		      (modes[i].visualClass == TrueColor)) &&
		     !(modes[i].flags & (XDGAInterlaced|XDGADoublescan)) ) {
			break;
		}
	}
	if ( i == num_modes ) {
		SDL_SetError("No matching video mode found");
		return(NULL);
	}
#ifdef DGA_DEBUG
	PrintMode(&modes[i]);
#endif

	/* Set the video mode */
	mode = SDL_NAME(XDGASetMode)(DGA_Display, DGA_Screen, modes[i].num);
	XFree(modes);
	if ( mode == NULL ) {
		SDL_SetError("Unable to switch to requested mode");
		return(NULL);
	}
	DGA_visualClass = mode->mode.visualClass;
	memory_base = (Uint8 *)mode->data;
	memory_pitch = mode->mode.bytesPerScanline;

	/* Set up the new mode framebuffer */
	current->flags = (SDL_FULLSCREEN|SDL_HWSURFACE);
	current->w = mode->mode.viewportWidth;
	current->h = mode->mode.viewportHeight;
	current->pitch = memory_pitch;
	current->pixels = memory_base;
	if ( ! SDL_ReallocFormat(current, mode->mode.bitsPerPixel,
	                                  mode->mode.redMask,
	                                  mode->mode.greenMask,
	                                  mode->mode.blueMask, 0) ) {
		return(NULL);
	}
	screen_len = current->h*current->pitch;

	/* Create a colormap if necessary */
	if ( (DGA_visualClass == PseudoColor) ||
             (DGA_visualClass == DirectColor) ) {
		DGA_colormap = SDL_NAME(XDGACreateColormap)(DGA_Display, DGA_Screen,
							mode, AllocAll);
		if ( DGA_visualClass == PseudoColor ) {
			current->flags |= SDL_HWPALETTE;
		} else {
	    		/* Initialize the colormap to the identity mapping */
	    		SDL_GetGammaRamp(0, 0, 0);
	    		this->screen = current;
	    		DGA_SetGammaRamp(this, this->gamma);
			this->screen = NULL;
		}
	} else {
		DGA_colormap = SDL_NAME(XDGACreateColormap)(DGA_Display, DGA_Screen,
							mode, AllocNone);
	}
	SDL_NAME(XDGAInstallColormap)(DGA_Display, DGA_Screen, DGA_colormap);

	/* Update the hardware capabilities */
	UpdateHWInfo(this, &mode->mode);

	/* Set up the information for hardware surfaces */
	surfaces_mem = (Uint8 *)current->pixels + screen_len;
	surfaces_len = (mode->mode.imageHeight*current->pitch - screen_len);

	/* Update for double-buffering, if we can */
	SDL_NAME(XDGASetViewport)(DGA_Display, DGA_Screen, 0, 0, XDGAFlipRetrace);
	if ( flags & SDL_DOUBLEBUF ) {
		if ( mode->mode.imageHeight >= (current->h*2) ) {
			current->flags |= SDL_DOUBLEBUF;
			flip_page = 0;
			flip_yoffset[0] = 0;
			flip_yoffset[1] = current->h;
			flip_address[0] = memory_base;
			flip_address[1] = memory_base+screen_len;
			surfaces_mem += screen_len;
			surfaces_len -= screen_len;
		}
	}

	/* Allocate memory tracking for hardware surfaces */
	DGA_FreeHWSurfaces(this);
	if ( surfaces_len < 0 ) {
		surfaces_len = 0;
	}
	DGA_InitHWSurfaces(this, current, surfaces_mem, surfaces_len);

	/* Expose the back buffer as surface memory */
	if ( current->flags & SDL_DOUBLEBUF ) {
		this->screen = current;
		DGA_FlipHWSurface(this, current);
		this->screen = NULL;
	}

	/* Set the update rectangle function */
	this->UpdateRects = DGA_DirectUpdate;

	/* Enable mouse and keyboard support */
	{ long input_mask;
	  input_mask = (KeyPressMask | KeyReleaseMask);
	  input_mask |= (ButtonPressMask | ButtonReleaseMask);
	  input_mask |= PointerMotionMask;
	  SDL_NAME(XDGASelectInput)(DGA_Display, DGA_Screen, input_mask);
	}

	/* We're done */
	return(current);
}

#ifdef DGA_DEBUG
static void DGA_DumpHWSurfaces(_THIS)
{
	vidmem_bucket *bucket;

	printf("Memory left: %d (%d total)\n", surfaces_memleft, surfaces_memtotal);
	printf("\n");
	printf("         Base  Size\n");
	for ( bucket=&surfaces; bucket; bucket=bucket->next ) {
		printf("Bucket:  %p, %d (%s)\n", bucket->base, bucket->size, bucket->used ? "used" : "free");
		if ( bucket->prev ) {
			if ( bucket->base != bucket->prev->base+bucket->prev->size ) {
				printf("Warning, corrupt bucket list! (prev)\n");
			}
		} else {
			if ( bucket != &surfaces ) {
				printf("Warning, corrupt bucket list! (!prev)\n");
			}
		}
		if ( bucket->next ) {
			if ( bucket->next->base != bucket->base+bucket->size ) {
				printf("Warning, corrupt bucket list! (next)\n");
			}
		}
	}
	printf("\n");
}
#endif

static int DGA_InitHWSurfaces(_THIS, SDL_Surface *screen, Uint8 *base, int size)
{
	vidmem_bucket *bucket;

	surfaces_memtotal = size;
	surfaces_memleft = size;

	if ( surfaces_memleft > 0 ) {
		bucket = (vidmem_bucket *)SDL_malloc(sizeof(*bucket));
		if ( bucket == NULL ) {
			SDL_OutOfMemory();
			return(-1);
		}
		bucket->prev = &surfaces;
		bucket->used = 0;
		bucket->dirty = 0;
		bucket->base = base;
		bucket->size = size;
		bucket->next = NULL;
	} else {
		bucket = NULL;
	}

	surfaces.prev = NULL;
	surfaces.used = 1;
	surfaces.dirty = 0;
	surfaces.base = screen->pixels;
	surfaces.size = (unsigned int)((long)base - (long)surfaces.base);
	surfaces.next = bucket;
	screen->hwdata = (struct private_hwdata *)((char*)&surfaces);
	return(0);
}
static void DGA_FreeHWSurfaces(_THIS)
{
	vidmem_bucket *bucket, *freeable;

	bucket = surfaces.next;
	while ( bucket ) {
		freeable = bucket;
		bucket = bucket->next;
		SDL_free(freeable);
	}
	surfaces.next = NULL;
}

static __inline__ void DGA_AddBusySurface(SDL_Surface *surface)
{
	((vidmem_bucket *)surface->hwdata)->dirty = 1;
}

static __inline__ int DGA_IsSurfaceBusy(SDL_Surface *surface)
{
	return ((vidmem_bucket *)surface->hwdata)->dirty;
}

static __inline__ void DGA_WaitBusySurfaces(_THIS)
{
	vidmem_bucket *bucket;

	/* Wait for graphic operations to complete */
	SDL_NAME(XDGASync)(DGA_Display, DGA_Screen);

	/* Clear all surface dirty bits */
	for ( bucket=&surfaces; bucket; bucket=bucket->next ) {
		bucket->dirty = 0;
	}
}

static int DGA_AllocHWSurface(_THIS, SDL_Surface *surface)
{
	vidmem_bucket *bucket;
	int size;
	int extra;
	int retval = 0;

/* Temporarily, we only allow surfaces the same width as display.
   Some blitters require the pitch between two hardware surfaces
   to be the same.  Others have interesting alignment restrictions.
*/
if ( surface->pitch > SDL_VideoSurface->pitch ) {
	SDL_SetError("Surface requested wider than screen");
	return(-1);
}
surface->pitch = SDL_VideoSurface->pitch;
	size = surface->h * surface->pitch;
#ifdef DGA_DEBUG
	fprintf(stderr, "Allocating bucket of %d bytes\n", size);
#endif
	LOCK_DISPLAY();

	/* Quick check for available mem */
	if ( size > surfaces_memleft ) {
		SDL_SetError("Not enough video memory");
		retval = -1;
		goto done;
	}

	/* Search for an empty bucket big enough */
	for ( bucket=&surfaces; bucket; bucket=bucket->next ) {
		if ( ! bucket->used && (size <= bucket->size) ) {
			break;
		}
	}
	if ( bucket == NULL ) {
		SDL_SetError("Video memory too fragmented");
		retval = -1;
		goto done;
	}

	/* Create a new bucket for left-over memory */
	extra = (bucket->size - size);
	if ( extra ) {
		vidmem_bucket *newbucket;

#ifdef DGA_DEBUG
	fprintf(stderr, "Adding new free bucket of %d bytes\n", extra);
#endif
		newbucket = (vidmem_bucket *)SDL_malloc(sizeof(*newbucket));
		if ( newbucket == NULL ) {
			SDL_OutOfMemory();
			retval = -1;
			goto done;
		}
		newbucket->prev = bucket;
		newbucket->used = 0;
		newbucket->base = bucket->base+size;
		newbucket->size = extra;
		newbucket->next = bucket->next;
		if ( bucket->next ) {
			bucket->next->prev = newbucket;
		}
		bucket->next = newbucket;
	}

	/* Set the current bucket values and return it! */
	bucket->used = 1;
	bucket->size = size;
	bucket->dirty = 0;
#ifdef DGA_DEBUG
	fprintf(stderr, "Allocated %d bytes at %p\n", bucket->size, bucket->base);
#endif
	surfaces_memleft -= size;
	surface->flags |= SDL_HWSURFACE;
	surface->pixels = bucket->base;
	surface->hwdata = (struct private_hwdata *)bucket;
done:
	UNLOCK_DISPLAY();
	return(retval);
}
static void DGA_FreeHWSurface(_THIS, SDL_Surface *surface)
{
	vidmem_bucket *bucket, *freeable;

	/* Wait for any pending operations involving this surface */
	if ( DGA_IsSurfaceBusy(surface) ) {
		LOCK_DISPLAY();
		DGA_WaitBusySurfaces(this);
		UNLOCK_DISPLAY();
	}

	/* Look for the bucket in the current list */
	for ( bucket=&surfaces; bucket; bucket=bucket->next ) {
		if ( bucket == (vidmem_bucket *)surface->hwdata ) {
			break;
		}
	}
	if ( bucket && bucket->used ) {
		/* Add the memory back to the total */
#ifdef DGA_DEBUG
	printf("Freeing bucket of %d bytes\n", bucket->size);
#endif
		surfaces_memleft += bucket->size;

		/* Can we merge the space with surrounding buckets? */
		bucket->used = 0;
		if ( bucket->next && ! bucket->next->used ) {
#ifdef DGA_DEBUG
	printf("Merging with next bucket, for %d total bytes\n", bucket->size+bucket->next->size);
#endif
			freeable = bucket->next;
			bucket->size += bucket->next->size;
			bucket->next = bucket->next->next;
			if ( bucket->next ) {
				bucket->next->prev = bucket;
			}
			SDL_free(freeable);
		}
		if ( bucket->prev && ! bucket->prev->used ) {
#ifdef DGA_DEBUG
	printf("Merging with previous bucket, for %d total bytes\n", bucket->prev->size+bucket->size);
#endif
			freeable = bucket;
			bucket->prev->size += bucket->size;
			bucket->prev->next = bucket->next;
			if ( bucket->next ) {
				bucket->next->prev = bucket->prev;
			}
			SDL_free(freeable);
		}
	}
	surface->pixels = NULL;
	surface->hwdata = NULL;
}

static __inline__ void DGA_dst_to_xy(_THIS, SDL_Surface *dst, int *x, int *y)
{
	*x = (long)((Uint8 *)dst->pixels - memory_base)%memory_pitch;
	*y = (long)((Uint8 *)dst->pixels - memory_base)/memory_pitch;
}

static int DGA_FillHWRect(_THIS, SDL_Surface *dst, SDL_Rect *rect, Uint32 color)
{
	int x, y;
	unsigned int w, h;

	/* Don't fill the visible part of the screen, wait until flipped */
	LOCK_DISPLAY();
	if ( was_flipped && (dst == this->screen) ) {
		while ( SDL_NAME(XDGAGetViewportStatus)(DGA_Display, DGA_Screen) )
			/* Keep waiting for the hardware ... */ ;
		was_flipped = 0;
	}
	DGA_dst_to_xy(this, dst, &x, &y);
	x += rect->x;
	y += rect->y;
	w = rect->w;
	h = rect->h;
#if 0
  printf("Hardware accelerated rectangle fill: %dx%d at %d,%d\n", w, h, x, y);
#endif
	SDL_NAME(XDGAFillRectangle)(DGA_Display, DGA_Screen, x, y, w, h, color);
	if ( !(this->screen->flags & SDL_DOUBLEBUF) ) {
		XFlush(DGA_Display);
	}
	DGA_AddBusySurface(dst);
	UNLOCK_DISPLAY();
	return(0);
}

static int HWAccelBlit(SDL_Surface *src, SDL_Rect *srcrect,
                       SDL_Surface *dst, SDL_Rect *dstrect)
{
	SDL_VideoDevice *this;
	int srcx, srcy;
	int dstx, dsty;
	unsigned int w, h;

	this = current_video;
	/* Don't blit to the visible part of the screen, wait until flipped */
	LOCK_DISPLAY();
	if ( was_flipped && (dst == this->screen) ) {
		while ( SDL_NAME(XDGAGetViewportStatus)(DGA_Display, DGA_Screen) )
			/* Keep waiting for the hardware ... */ ;
		was_flipped = 0;
	}
	DGA_dst_to_xy(this, src, &srcx, &srcy);
	srcx += srcrect->x;
	srcy += srcrect->y;
	DGA_dst_to_xy(this, dst, &dstx, &dsty);
	dstx += dstrect->x;
	dsty += dstrect->y;
	w = srcrect->w;
	h = srcrect->h;
#if 0
  printf("Blitting %dx%d from %d,%d to %d,%d\n", w, h, srcx, srcy, dstx, dsty);
#endif
	if ( (src->flags & SDL_SRCCOLORKEY) == SDL_SRCCOLORKEY ) {
		SDL_NAME(XDGACopyTransparentArea)(DGA_Display, DGA_Screen,
			srcx, srcy, w, h, dstx, dsty, src->format->colorkey);
	} else {
		SDL_NAME(XDGACopyArea)(DGA_Display, DGA_Screen,
			srcx, srcy, w, h, dstx, dsty);
	}
	if ( !(this->screen->flags & SDL_DOUBLEBUF) ) {
		XFlush(DGA_Display);
	}
	DGA_AddBusySurface(src);
	DGA_AddBusySurface(dst);
	UNLOCK_DISPLAY();
	return(0);
}

static int DGA_CheckHWBlit(_THIS, SDL_Surface *src, SDL_Surface *dst)
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

static __inline__ void DGA_WaitFlip(_THIS)
{
	if ( was_flipped ) {
		while ( SDL_NAME(XDGAGetViewportStatus)(DGA_Display, DGA_Screen) )
			/* Keep waiting for the hardware ... */ ;
		was_flipped = 0;
	}
}

static int DGA_LockHWSurface(_THIS, SDL_Surface *surface)
{
	if ( surface == this->screen ) {
		SDL_mutexP(hw_lock);
		LOCK_DISPLAY();
		if ( DGA_IsSurfaceBusy(surface) ) {
			DGA_WaitBusySurfaces(this);
		}
		DGA_WaitFlip(this);
		UNLOCK_DISPLAY();
	} else {
		if ( DGA_IsSurfaceBusy(surface) ) {
			LOCK_DISPLAY();
			DGA_WaitBusySurfaces(this);
			UNLOCK_DISPLAY();
		}
	}
	return(0);
}
static void DGA_UnlockHWSurface(_THIS, SDL_Surface *surface)
{
	if ( surface == this->screen ) {
		SDL_mutexV(hw_lock);
	}
}

static int DGA_FlipHWSurface(_THIS, SDL_Surface *surface)
{
	/* Wait for vertical retrace and then flip display */
	LOCK_DISPLAY();
	if ( DGA_IsSurfaceBusy(this->screen) ) {
		DGA_WaitBusySurfaces(this);
	}
	DGA_WaitFlip(this);
	SDL_NAME(XDGASetViewport)(DGA_Display, DGA_Screen,
	                0, flip_yoffset[flip_page], XDGAFlipRetrace);
	XFlush(DGA_Display);
	UNLOCK_DISPLAY();
	was_flipped = 1;
	flip_page = !flip_page;

	surface->pixels = flip_address[flip_page];
	return(0);
}

static void DGA_DirectUpdate(_THIS, int numrects, SDL_Rect *rects)
{
	/* The application is already updating the visible video memory */
	return;
}

static int DGA_SetColors(_THIS, int firstcolor, int ncolors, SDL_Color *colors)
{
        int i;
	XColor  *xcmap;

	/* This happens on initialization */
	if ( ! DGA_colormap ) {
		return(0);
	}
	xcmap = SDL_stack_alloc(XColor, ncolors);
	for ( i=0; i<ncolors; ++i ) {
		xcmap[i].pixel = firstcolor + i;
		xcmap[i].red   = (colors[i].r<<8)|colors[i].r;
		xcmap[i].green = (colors[i].g<<8)|colors[i].g;
		xcmap[i].blue  = (colors[i].b<<8)|colors[i].b;
		xcmap[i].flags = (DoRed|DoGreen|DoBlue);
	}
	LOCK_DISPLAY();
	XStoreColors(DGA_Display, DGA_colormap, xcmap, ncolors);
	XSync(DGA_Display, False);
	UNLOCK_DISPLAY();
	SDL_stack_free(xcmap);

	/* That was easy. :) */
	return(1);
}

int DGA_SetGammaRamp(_THIS, Uint16 *ramp)
{
	int i, ncolors;
	XColor xcmap[256];

	/* See if actually setting the gamma is supported */
	if ( DGA_visualClass != DirectColor ) {
	    SDL_SetError("Gamma correction not supported on this visual");
	    return(-1);
	}

	/* Calculate the appropriate palette for the given gamma ramp */
	if ( this->screen->format->BitsPerPixel <= 16 ) {
		ncolors = 64; /* Is this right? */
	} else {
		ncolors = 256;
	}
	for ( i=0; i<ncolors; ++i ) {
		Uint8 c = (256 * i / ncolors);
		xcmap[i].pixel = SDL_MapRGB(this->screen->format, c, c, c);
		xcmap[i].red   = ramp[0*256+c];
		xcmap[i].green = ramp[1*256+c];
		xcmap[i].blue  = ramp[2*256+c];
		xcmap[i].flags = (DoRed|DoGreen|DoBlue);
	}
	LOCK_DISPLAY();
	XStoreColors(DGA_Display, DGA_colormap, xcmap, ncolors);
	XSync(DGA_Display, False);
	UNLOCK_DISPLAY();
	return(0);
}

void DGA_VideoQuit(_THIS)
{
	int i, j;

	if ( DGA_Display ) {
		/* Free colormap, if necessary */
		if ( DGA_colormap ) {
			XFreeColormap(DGA_Display, DGA_colormap);
			DGA_colormap = 0;
		}

		/* Unmap memory and reset video mode */
		SDL_NAME(XDGACloseFramebuffer)(DGA_Display, DGA_Screen);
		if ( this->screen ) {
			/* Tell SDL not to free the pixels */
			DGA_FreeHWSurface(this, this->screen);
		}
		SDL_NAME(XDGASetMode)(DGA_Display, DGA_Screen, 0);

		/* Clear the lock mutex */
		if ( hw_lock != NULL ) {
			SDL_DestroyMutex(hw_lock);
			hw_lock = NULL;
		}
#ifdef LOCK_DGA_DISPLAY
		if ( event_lock != NULL ) {
			SDL_DestroyMutex(event_lock);
			event_lock = NULL;
		}
#endif /* LOCK_DGA_DISPLAY */

		/* Clean up defined video modes */
		for ( i=0; i<NUM_MODELISTS; ++i ) {
			if ( SDL_modelist[i] != NULL ) {
				for ( j=0; SDL_modelist[i][j]; ++j ) {
					SDL_free(SDL_modelist[i][j]);
				}
				SDL_free(SDL_modelist[i]);
				SDL_modelist[i] = NULL;
			}
		}

		/* Clean up the memory bucket list */
		DGA_FreeHWSurfaces(this);

		/* Close up the display */
		XCloseDisplay(DGA_Display);
	}
}
