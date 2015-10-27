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

/* This is the XFree86 Xv extension implementation of YUV video overlays */

#if SDL_VIDEO_DRIVER_X11_XV

#include <X11/Xlib.h>
#ifndef NO_SHARED_MEMORY
#include <sys/ipc.h>
#include <sys/shm.h>
#include <X11/extensions/XShm.h>
#endif
#include "../Xext/extensions/Xvlib.h"

#include "SDL_x11yuv_c.h"
#include "../SDL_yuvfuncs.h"

#define XFREE86_REFRESH_HACK
#ifdef XFREE86_REFRESH_HACK
#include "SDL_x11image_c.h"
#endif

/* Workaround when pitch != width */
#define PITCH_WORKAROUND

/* Workaround intel i810 video overlay waiting with failing until the
   first Xv[Shm]PutImage call <sigh> */
#define INTEL_XV_BADALLOC_WORKAROUND

/* Fix for the NVidia GeForce 2 - use the last available adaptor */
/*#define USE_LAST_ADAPTOR*/  /* Apparently the NVidia drivers are fixed */

/* The functions used to manipulate software video overlays */
static struct private_yuvhwfuncs x11_yuvfuncs = {
	X11_LockYUVOverlay,
	X11_UnlockYUVOverlay,
	X11_DisplayYUVOverlay,
	X11_FreeYUVOverlay
};

struct private_yuvhwdata {
	int port;
#ifndef NO_SHARED_MEMORY
	int yuv_use_mitshm;
	XShmSegmentInfo yuvshm;
#endif
	SDL_NAME(XvImage) *image;
};


static int (*X_handler)(Display *, XErrorEvent *) = NULL;

#ifndef NO_SHARED_MEMORY
/* Shared memory error handler routine */
static int shm_error;
static int shm_errhandler(Display *d, XErrorEvent *e)
{
        if ( e->error_code == BadAccess ) {
        	shm_error = True;
        	return(0);
        } else
		return(X_handler(d,e));
}
#endif /* !NO_SHARED_MEMORY */

static int xv_error;
static int xv_errhandler(Display *d, XErrorEvent *e)
{
        if ( e->error_code == BadMatch ) {
        	xv_error = True;
        	return(0);
        } else
		return(X_handler(d,e));
}

#ifdef INTEL_XV_BADALLOC_WORKAROUND
static int intel_errhandler(Display *d, XErrorEvent *e)
{
        if ( e->error_code == BadAlloc ) {
        	xv_error = True;
        	return(0);
        } else
		return(X_handler(d,e));
}

static void X11_ClearYUVOverlay(SDL_Overlay *overlay)
{
	int x,y;
	    
	switch (overlay->format)
	{
	case SDL_YV12_OVERLAY:
	case SDL_IYUV_OVERLAY:
		for (y = 0; y < overlay->h; y++)
			memset(overlay->pixels[0] + y * overlay->pitches[0],
				0, overlay->w);
		
		for (y = 0; y < (overlay->h / 2); y++)
		{
			memset(overlay->pixels[1] + y * overlay->pitches[1],
				-128, overlay->w / 2);
			memset(overlay->pixels[2] + y * overlay->pitches[2],
				-128, overlay->w / 2);
		}
		break;
	case SDL_YUY2_OVERLAY:
	case SDL_YVYU_OVERLAY:
		for (y = 0; y < overlay->h; y++)
		{
			for (x = 0; x < overlay->w; x += 2)
			{
				Uint8 *pixel_pair = overlay->pixels[0] +
					y * overlay->pitches[0] + x * 2;
				pixel_pair[0] = 0;
				pixel_pair[1] = -128;
				pixel_pair[2] = 0;
				pixel_pair[3] = -128;
			}
		}
		break;
	case SDL_UYVY_OVERLAY:
		for (y = 0; y < overlay->h; y++)
		{
			for (x = 0; x < overlay->w; x += 2)
			{
				Uint8 *pixel_pair = overlay->pixels[0] +
					y * overlay->pitches[0] + x * 2;
				pixel_pair[0] = -128;
				pixel_pair[1] = 0;
				pixel_pair[2] = -128;
				pixel_pair[3] = 0;
			}
		}
		break;
	}
}
#endif

SDL_Overlay *X11_CreateYUVOverlay(_THIS, int width, int height, Uint32 format, SDL_Surface *display)
{
	SDL_Overlay *overlay;
	struct private_yuvhwdata *hwdata;
	int xv_port;
	unsigned int i, j, k;
	unsigned int adaptors;
	SDL_NAME(XvAdaptorInfo) *ainfo;
	int bpp;
#ifndef NO_SHARED_MEMORY
	XShmSegmentInfo *yuvshm;
#endif
#ifdef INTEL_XV_BADALLOC_WORKAROUND
	int intel_adapter = False;
#endif

	/* Look for the XVideo extension with a valid port for this format */
	xv_port = -1;
	if ( (Success == SDL_NAME(XvQueryExtension)(GFX_Display, &j, &j, &j, &j, &j)) &&
	     (Success == SDL_NAME(XvQueryAdaptors)(GFX_Display,
	                                 RootWindow(GFX_Display, SDL_Screen),
	                                 &adaptors, &ainfo)) ) {
#ifdef USE_LAST_ADAPTOR
		for ( i=0; i < adaptors; ++i )
#else
		for ( i=0; (i < adaptors) && (xv_port == -1); ++i )
#endif /* USE_LAST_ADAPTOR */
		{
			/* Check to see if the visual can be used */
			if ( BUGGY_XFREE86(<=, 4001) ) {
				int visual_ok = 0;
				for ( j=0; j<ainfo[i].num_formats; ++j ) {
					if ( ainfo[i].formats[j].visual_id ==
							SDL_Visual->visualid ) {
						visual_ok = 1;
						break;
					}
				}
				if ( ! visual_ok ) {
					continue;
				}
			}
#ifdef INTEL_XV_BADALLOC_WORKAROUND
			if ( !strcmp(ainfo[i].name, "Intel(R) Video Overla"))
				intel_adapter = True;
			else
				intel_adapter = False;
#endif
			if ( (ainfo[i].type & XvInputMask) &&
			     (ainfo[i].type & XvImageMask) ) {
				int num_formats;
				SDL_NAME(XvImageFormatValues) *formats;
				formats = SDL_NAME(XvListImageFormats)(GFX_Display,
				              ainfo[i].base_id, &num_formats);
#ifdef USE_LAST_ADAPTOR
				for ( j=0; j < num_formats; ++j )
#else
				for ( j=0; (j < num_formats) && (xv_port == -1); ++j )
#endif /* USE_LAST_ADAPTOR */
				{
					if ( (Uint32)formats[j].id == format ) {
						for ( k=0; k < ainfo[i].num_ports; ++k ) {
							if ( Success == SDL_NAME(XvGrabPort)(GFX_Display, ainfo[i].base_id+k, CurrentTime) ) {
								xv_port = ainfo[i].base_id+k;
								break;
							}
						}
					}
				}
				if ( formats ) {
					XFree(formats);
				}
			}
		}
		SDL_NAME(XvFreeAdaptorInfo)(ainfo);
	}

	/* Precalculate the bpp for the pitch workaround below */
	switch (format) {
	    /* Add any other cases we need to support... */
	    case SDL_YUY2_OVERLAY:
	    case SDL_UYVY_OVERLAY:
	    case SDL_YVYU_OVERLAY:
		bpp = 2;
		break;
	    default:
		bpp = 1;
		break;
	}

#if 0
    /*
     * !!! FIXME:
     * "Here are some diffs for X11 and yuv.  Note that the last part 2nd
     *  diff should probably be a new call to XvQueryAdaptorFree with ainfo
     *  and the number of adaptors, instead of the loop through like I did."
     *
     *  ACHTUNG: This is broken! It looks like XvFreeAdaptorInfo does this
     *  for you, so we end up with a double-free. I need to look at this
     *  more closely...  --ryan.
     */
 	for ( i=0; i < adaptors; ++i ) {
 	  if (ainfo[i].name != NULL) Xfree(ainfo[i].name);
 	  if (ainfo[i].formats != NULL) Xfree(ainfo[i].formats);
   	}
 	Xfree(ainfo);
#endif

	if ( xv_port == -1 ) {
		SDL_SetError("No available video ports for requested format");
		return(NULL);
	}

	/* Enable auto-painting of the overlay colorkey */
	{
		static const char *attr[] = { "XV_AUTOPAINT_COLORKEY", "XV_AUTOPAINT_COLOURKEY" };
		unsigned int i;

		SDL_NAME(XvSelectPortNotify)(GFX_Display, xv_port, True);
		X_handler = XSetErrorHandler(xv_errhandler);
		for ( i=0; i < sizeof(attr)/(sizeof attr[0]); ++i ) {
			Atom a;

			xv_error = False;
			a = XInternAtom(GFX_Display, attr[i], True);
			if ( a != None ) {
     				SDL_NAME(XvSetPortAttribute)(GFX_Display, xv_port, a, 1);
				XSync(GFX_Display, True);
				if ( ! xv_error ) {
					break;
				}
			}
		}
		XSetErrorHandler(X_handler);
		SDL_NAME(XvSelectPortNotify)(GFX_Display, xv_port, False);
	}

	/* Create the overlay structure */
	overlay = (SDL_Overlay *)SDL_malloc(sizeof *overlay);
	if ( overlay == NULL ) {
		SDL_NAME(XvUngrabPort)(GFX_Display, xv_port, CurrentTime);
		SDL_OutOfMemory();
		return(NULL);
	}
	SDL_memset(overlay, 0, (sizeof *overlay));

	/* Fill in the basic members */
	overlay->format = format;
	overlay->w = width;
	overlay->h = height;

	/* Set up the YUV surface function structure */
	overlay->hwfuncs = &x11_yuvfuncs;
	overlay->hw_overlay = 1;

	/* Create the pixel data and lookup tables */
	hwdata = (struct private_yuvhwdata *)SDL_malloc(sizeof *hwdata);
	overlay->hwdata = hwdata;
	if ( hwdata == NULL ) {
		SDL_NAME(XvUngrabPort)(GFX_Display, xv_port, CurrentTime);
		SDL_OutOfMemory();
		SDL_FreeYUVOverlay(overlay);
		return(NULL);
	}
	hwdata->port = xv_port;
#ifndef NO_SHARED_MEMORY
	yuvshm = &hwdata->yuvshm;
	SDL_memset(yuvshm, 0, sizeof(*yuvshm));
	hwdata->image = SDL_NAME(XvShmCreateImage)(GFX_Display, xv_port, format,
						   0, width, height, yuvshm);
#ifdef PITCH_WORKAROUND
	if ( hwdata->image != NULL && hwdata->image->pitches[0] != (width*bpp) ) {
		/* Ajust overlay width according to pitch */ 
		width = hwdata->image->pitches[0] / bpp;
		XFree(hwdata->image);
		hwdata->image = SDL_NAME(XvShmCreateImage)(GFX_Display, xv_port, format,
							   0, width, height, yuvshm);
	}
#endif /* PITCH_WORKAROUND */
	hwdata->yuv_use_mitshm = (hwdata->image != NULL);
	if ( hwdata->yuv_use_mitshm ) {
		yuvshm->shmid = shmget(IPC_PRIVATE, hwdata->image->data_size,
				       IPC_CREAT | 0777);
		if ( yuvshm->shmid >= 0 ) {
			yuvshm->shmaddr = (char *)shmat(yuvshm->shmid, 0, 0);
			yuvshm->readOnly = False;
			if ( yuvshm->shmaddr != (char *)-1 ) {
				shm_error = False;
				X_handler = XSetErrorHandler(shm_errhandler);
				XShmAttach(GFX_Display, yuvshm);
				XSync(GFX_Display, True);
				XSetErrorHandler(X_handler);
				if ( shm_error )
					shmdt(yuvshm->shmaddr);
			} else {
				shm_error = True;
			}
			shmctl(yuvshm->shmid, IPC_RMID, NULL);
		} else {
			shm_error = True;
		}
		if ( shm_error ) {
			XFree(hwdata->image);
			hwdata->yuv_use_mitshm = 0;
		} else {
			hwdata->image->data = yuvshm->shmaddr;
		}
	}
	if ( !hwdata->yuv_use_mitshm )
#endif /* NO_SHARED_MEMORY */
	{
		hwdata->image = SDL_NAME(XvCreateImage)(GFX_Display, xv_port, format,
							0, width, height);

#ifdef PITCH_WORKAROUND
		if ( hwdata->image != NULL && hwdata->image->pitches[0] != (width*bpp) ) {
			/* Ajust overlay width according to pitch */ 
			XFree(hwdata->image);
			width = hwdata->image->pitches[0] / bpp;
			hwdata->image = SDL_NAME(XvCreateImage)(GFX_Display, xv_port, format,
								0, width, height);
		}
#endif /* PITCH_WORKAROUND */
		if ( hwdata->image == NULL ) {
			SDL_SetError("Couldn't create XVideo image");
			SDL_FreeYUVOverlay(overlay);
			return(NULL);
		}
		hwdata->image->data = SDL_malloc(hwdata->image->data_size);
		if ( hwdata->image->data == NULL ) {
			SDL_OutOfMemory();
			SDL_FreeYUVOverlay(overlay);
			return(NULL);
		}
	}

	/* Find the pitch and offset values for the overlay */
	overlay->planes = hwdata->image->num_planes;
	overlay->pitches = (Uint16 *)SDL_malloc(overlay->planes * sizeof(Uint16));
	overlay->pixels = (Uint8 **)SDL_malloc(overlay->planes * sizeof(Uint8 *));
	if ( !overlay->pitches || !overlay->pixels ) {
		SDL_OutOfMemory();
		SDL_FreeYUVOverlay(overlay);
		return(NULL);
	}
	for ( i=0; i<overlay->planes; ++i ) {
		overlay->pitches[i] = hwdata->image->pitches[i];
		overlay->pixels[i] = (Uint8 *)hwdata->image->data +
		                              hwdata->image->offsets[i];
	}

#ifdef XFREE86_REFRESH_HACK
	/* Work around an XFree86 X server bug (?)
	   We can't perform normal updates in windows that have video
	   being output to them.  See SDL_x11image.c for more details.
	 */
	X11_DisableAutoRefresh(this);
#endif

#ifdef INTEL_XV_BADALLOC_WORKAROUND
	/* HACK, GRRR sometimes (i810) creating the overlay succeeds, but the
	   first call to XvShm[Put]Image to a mapped window fails with:
	   "BadAlloc (insufficient resources for operation)". This happens with
	   certain formats when the XvImage is too large to the i810's liking.

	   We work around this by doing a test XvShm[Put]Image with a black
	   Xv image, this may cause some flashing, so only do this check if we
	   are running on an intel Xv-adapter. */
	if (intel_adapter)
	{
		xv_error = False;
		X_handler = XSetErrorHandler(intel_errhandler);
		
		X11_ClearYUVOverlay(overlay);

		/* We set the destination height and width to 1 pixel to avoid
		   putting a large black rectangle over the screen, thus
		   strongly reducing possible flashing. */
#ifndef NO_SHARED_MEMORY
		if ( hwdata->yuv_use_mitshm ) {
			SDL_NAME(XvShmPutImage)(GFX_Display, hwdata->port,
				SDL_Window, SDL_GC,
				hwdata->image,
				0, 0, overlay->w, overlay->h,
				0, 0, 1, 1, False);
		}
		else
#endif
		{
			SDL_NAME(XvPutImage)(GFX_Display, hwdata->port,
				SDL_Window, SDL_GC,
				hwdata->image,
				0, 0, overlay->w, overlay->h,
				0, 0, 1, 1);
		}
		XSync(GFX_Display, False);
		XSetErrorHandler(X_handler);

		if (xv_error)
		{
			X11_FreeYUVOverlay(this, overlay);
			return NULL;
		}
		/* Repair the (1 pixel worth of) damage we've just done */
		X11_RefreshDisplay(this);
	}
#endif

	/* We're all done.. */
	return(overlay);
}

int X11_LockYUVOverlay(_THIS, SDL_Overlay *overlay)
{
	return(0);
}

void X11_UnlockYUVOverlay(_THIS, SDL_Overlay *overlay)
{
	return;
}

int X11_DisplayYUVOverlay(_THIS, SDL_Overlay *overlay, SDL_Rect *src, SDL_Rect *dst)
{
	struct private_yuvhwdata *hwdata;

	hwdata = overlay->hwdata;

#ifndef NO_SHARED_MEMORY
	if ( hwdata->yuv_use_mitshm ) {
		SDL_NAME(XvShmPutImage)(GFX_Display, hwdata->port, SDL_Window, SDL_GC,
	              hwdata->image,
		      src->x, src->y, src->w, src->h,
		      dst->x, dst->y, dst->w, dst->h, False);
	}
	else
#endif
	{
		SDL_NAME(XvPutImage)(GFX_Display, hwdata->port, SDL_Window, SDL_GC,
				     hwdata->image,
		                     src->x, src->y, src->w, src->h,
		                     dst->x, dst->y, dst->w, dst->h);
	}
	XSync(GFX_Display, False);
	return(0);
}

void X11_FreeYUVOverlay(_THIS, SDL_Overlay *overlay)
{
	struct private_yuvhwdata *hwdata;

	hwdata = overlay->hwdata;
	if ( hwdata ) {
		SDL_NAME(XvUngrabPort)(GFX_Display, hwdata->port, CurrentTime);
#ifndef NO_SHARED_MEMORY
		if ( hwdata->yuv_use_mitshm ) {
			XShmDetach(GFX_Display, &hwdata->yuvshm);
			shmdt(hwdata->yuvshm.shmaddr);
		}
#endif
		if ( hwdata->image ) {
			XFree(hwdata->image);
		}
		SDL_free(hwdata);
	}
	if ( overlay->pitches ) {
		SDL_free(overlay->pitches);
		overlay->pitches = NULL;
	}
	if ( overlay->pixels ) {
		SDL_free(overlay->pixels);
		overlay->pixels = NULL;
	}
#ifdef XFREE86_REFRESH_HACK
	X11_EnableAutoRefresh(this);
#endif
}

#endif /* SDL_VIDEO_DRIVER_X11_XV */
