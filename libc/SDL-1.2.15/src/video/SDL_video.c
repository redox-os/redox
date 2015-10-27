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

/* The high-level video driver subsystem */

#include "SDL.h"
#include "SDL_sysvideo.h"
#include "SDL_blit.h"
#include "SDL_pixels_c.h"
#include "SDL_cursor_c.h"
#include "../events/SDL_sysevents.h"
#include "../events/SDL_events_c.h"

/* Available video drivers */
static VideoBootStrap *bootstrap[] = {
#if SDL_VIDEO_DRIVER_QUARTZ
	&QZ_bootstrap,
#endif
#if SDL_VIDEO_DRIVER_X11
	&X11_bootstrap,
#endif
#if SDL_VIDEO_DRIVER_DGA
	&DGA_bootstrap,
#endif
#if SDL_VIDEO_DRIVER_NANOX
	&NX_bootstrap,
#endif
#if SDL_VIDEO_DRIVER_IPOD
	&iPod_bootstrap,
#endif
#if SDL_VIDEO_DRIVER_QTOPIA
	&Qtopia_bootstrap,
#endif
#if SDL_VIDEO_DRIVER_WSCONS
	&WSCONS_bootstrap,
#endif
#if SDL_VIDEO_DRIVER_FBCON
	&FBCON_bootstrap,
#endif
#if SDL_VIDEO_DRIVER_DIRECTFB
	&DirectFB_bootstrap,
#endif
#if SDL_VIDEO_DRIVER_PS2GS
	&PS2GS_bootstrap,
#endif
#if SDL_VIDEO_DRIVER_PS3
	&PS3_bootstrap,
#endif
#if SDL_VIDEO_DRIVER_GGI
	&GGI_bootstrap,
#endif
#if SDL_VIDEO_DRIVER_VGL
	&VGL_bootstrap,
#endif
#if SDL_VIDEO_DRIVER_SVGALIB
	&SVGALIB_bootstrap,
#endif
#if SDL_VIDEO_DRIVER_GAPI
	&GAPI_bootstrap,
#endif
#if SDL_VIDEO_DRIVER_WINDIB
	&WINDIB_bootstrap,
#endif
#if SDL_VIDEO_DRIVER_DDRAW
	&DIRECTX_bootstrap,
#endif
#if SDL_VIDEO_DRIVER_BWINDOW
	&BWINDOW_bootstrap,
#endif
#if SDL_VIDEO_DRIVER_TOOLBOX
	&TOOLBOX_bootstrap,
#endif
#if SDL_VIDEO_DRIVER_DRAWSPROCKET
	&DSp_bootstrap,
#endif
#if SDL_VIDEO_DRIVER_PHOTON
	&ph_bootstrap,
#endif
#if SDL_VIDEO_DRIVER_EPOC
	&EPOC_bootstrap,
#endif
#if SDL_VIDEO_DRIVER_XBIOS
	&XBIOS_bootstrap,
#endif
#if SDL_VIDEO_DRIVER_GEM
	&GEM_bootstrap,
#endif
#if SDL_VIDEO_DRIVER_PICOGUI
	&PG_bootstrap,
#endif
#if SDL_VIDEO_DRIVER_DC
	&DC_bootstrap,
#endif
#if SDL_VIDEO_DRIVER_NDS
	&NDS_bootstrap,
#endif
#if SDL_VIDEO_DRIVER_RISCOS
	&RISCOS_bootstrap,
#endif
#if SDL_VIDEO_DRIVER_OS2FS
	&OS2FSLib_bootstrap,
#endif
#if SDL_VIDEO_DRIVER_AALIB
	&AALIB_bootstrap,
#endif
#if SDL_VIDEO_DRIVER_CACA
	&CACA_bootstrap,
#endif
#if SDL_VIDEO_DRIVER_DUMMY
	&DUMMY_bootstrap,
#endif
	NULL
};

SDL_VideoDevice *current_video = NULL;

/* Various local functions */
int SDL_VideoInit(const char *driver_name, Uint32 flags);
void SDL_VideoQuit(void);
void SDL_GL_UpdateRectsLock(SDL_VideoDevice* this, int numrects, SDL_Rect* rects);

static SDL_GrabMode SDL_WM_GrabInputOff(void);
#if SDL_VIDEO_OPENGL
static int lock_count = 0;
#endif


/*
 * Initialize the video and event subsystems -- determine native pixel format
 */
int SDL_VideoInit (const char *driver_name, Uint32 flags)
{
	SDL_VideoDevice *video;
	int index;
	int i;
	SDL_PixelFormat vformat;
	Uint32 video_flags;

	/* Toggle the event thread flags, based on OS requirements */
#if defined(MUST_THREAD_EVENTS)
	flags |= SDL_INIT_EVENTTHREAD;
#elif defined(CANT_THREAD_EVENTS)
	if ( (flags & SDL_INIT_EVENTTHREAD) == SDL_INIT_EVENTTHREAD ) {
		SDL_SetError("OS doesn't support threaded events");
		return(-1);
	}
#endif

	/* Check to make sure we don't overwrite 'current_video' */
	if ( current_video != NULL ) {
		SDL_VideoQuit();
	}

	/* Select the proper video driver */
	index = 0;
	video = NULL;
	if ( driver_name != NULL ) {
#if 0	/* This will be replaced with a better driver selection API */
		if ( SDL_strrchr(driver_name, ':') != NULL ) {
			index = atoi(SDL_strrchr(driver_name, ':')+1);
		}
#endif
		for ( i=0; bootstrap[i]; ++i ) {
			if ( SDL_strcasecmp(bootstrap[i]->name, driver_name) == 0) {
				if ( bootstrap[i]->available() ) {
					video = bootstrap[i]->create(index);
					break;
				}
			}
		}
	} else {
		for ( i=0; bootstrap[i]; ++i ) {
			if ( bootstrap[i]->available() ) {
				video = bootstrap[i]->create(index);
				if ( video != NULL ) {
					break;
				}
			}
		}
	}
	if ( video == NULL ) {
		SDL_SetError("No available video device");
		return(-1);
	}
	current_video = video;
	current_video->name = bootstrap[i]->name;

	/* Do some basic variable initialization */
	video->screen = NULL;
	video->shadow = NULL;
	video->visible = NULL;
	video->physpal = NULL;
	video->gammacols = NULL;
	video->gamma = NULL;
	video->wm_title = NULL;
	video->wm_icon  = NULL;
	video->offset_x = 0;
	video->offset_y = 0;
	SDL_memset(&video->info, 0, (sizeof video->info));
	
	video->displayformatalphapixel = NULL;

	/* Set some very sane GL defaults */
	video->gl_config.driver_loaded = 0;
	video->gl_config.dll_handle = NULL;
	video->gl_config.red_size = 3;
	video->gl_config.green_size = 3;
	video->gl_config.blue_size = 2;
	video->gl_config.alpha_size = 0;
	video->gl_config.buffer_size = 0;
	video->gl_config.depth_size = 16;
	video->gl_config.stencil_size = 0;
	video->gl_config.double_buffer = 1;
	video->gl_config.accum_red_size = 0;
	video->gl_config.accum_green_size = 0;
	video->gl_config.accum_blue_size = 0;
	video->gl_config.accum_alpha_size = 0;
	video->gl_config.stereo = 0;
	video->gl_config.multisamplebuffers = 0;
	video->gl_config.multisamplesamples = 0;
	video->gl_config.accelerated = -1; /* not known, don't set */
	video->gl_config.swap_control = -1; /* not known, don't set */
	
	/* Initialize the video subsystem */
	SDL_memset(&vformat, 0, sizeof(vformat));
	if ( video->VideoInit(video, &vformat) < 0 ) {
		SDL_VideoQuit();
		return(-1);
	}

	/* Create a zero sized video surface of the appropriate format */
	video_flags = SDL_SWSURFACE;
	SDL_VideoSurface = SDL_CreateRGBSurface(video_flags, 0, 0,
				vformat.BitsPerPixel,
				vformat.Rmask, vformat.Gmask, vformat.Bmask, 0);
	if ( SDL_VideoSurface == NULL ) {
		SDL_VideoQuit();
		return(-1);
	}
	SDL_PublicSurface = NULL;	/* Until SDL_SetVideoMode() */

#if 0 /* Don't change the current palette - may be used by other programs.
       * The application can't do anything with the display surface until
       * a video mode has been set anyway. :)
       */
	/* If we have a palettized surface, create a default palette */
	if ( SDL_VideoSurface->format->palette ) {
		SDL_PixelFormat *vf = SDL_VideoSurface->format;
		SDL_DitherColors(vf->palette->colors, vf->BitsPerPixel);
		video->SetColors(video,
				 0, vf->palette->ncolors, vf->palette->colors);
	}
#endif
	video->info.vfmt = SDL_VideoSurface->format;

	/* Start the event loop */
	if ( SDL_StartEventLoop(flags) < 0 ) {
		SDL_VideoQuit();
		return(-1);
	}
	SDL_CursorInit(flags & SDL_INIT_EVENTTHREAD);

	/* We're ready to go! */
	return(0);
}

char *SDL_VideoDriverName(char *namebuf, int maxlen)
{
	if ( current_video != NULL ) {
		SDL_strlcpy(namebuf, current_video->name, maxlen);
		return(namebuf);
	}
	return(NULL);
}

/*
 * Get the current display surface
 */
SDL_Surface *SDL_GetVideoSurface(void)
{
	SDL_Surface *visible;

	visible = NULL;
	if ( current_video ) {
		visible = current_video->visible;
	}
	return(visible);
}

/*
 * Get the current information about the video hardware
 */
const SDL_VideoInfo *SDL_GetVideoInfo(void)
{
	const SDL_VideoInfo *info;

	info = NULL;
	if ( current_video ) {
		info = &current_video->info;
	}
	return(info);
}

/*
 * Return a pointer to an array of available screen dimensions for the
 * given format, sorted largest to smallest.  Returns NULL if there are
 * no dimensions available for a particular format, or (SDL_Rect **)-1
 * if any dimension is okay for the given format.  If 'format' is NULL,
 * the mode list will be for the format given by SDL_GetVideoInfo()->vfmt
 */
SDL_Rect ** SDL_ListModes (SDL_PixelFormat *format, Uint32 flags)
{
	SDL_VideoDevice *video = current_video;
	SDL_VideoDevice *this  = current_video;
	SDL_Rect **modes;

	modes = NULL;
	if ( SDL_VideoSurface ) {
		if ( format == NULL ) {
			format = SDL_VideoSurface->format;
		}
		modes = video->ListModes(this, format, flags);
	}
	return(modes);
}

/*
 * Check to see if a particular video mode is supported.
 * It returns 0 if the requested mode is not supported under any bit depth,
 * or returns the bits-per-pixel of the closest available mode with the
 * given width and height.  If this bits-per-pixel is different from the
 * one used when setting the video mode, SDL_SetVideoMode() will succeed,
 * but will emulate the requested bits-per-pixel with a shadow surface.
 */
static Uint8 SDL_closest_depths[4][8] = {
	/* 8 bit closest depth ordering */
	{ 0, 8, 16, 15, 32, 24, 0, 0 },
	/* 15,16 bit closest depth ordering */
	{ 0, 16, 15, 32, 24, 8, 0, 0 },
	/* 24 bit closest depth ordering */
	{ 0, 24, 32, 16, 15, 8, 0, 0 },
	/* 32 bit closest depth ordering */
	{ 0, 32, 16, 15, 24, 8, 0, 0 }
};


#ifdef __MACOS__ /* MPW optimization bug? */
#define NEGATIVE_ONE 0xFFFFFFFF
#else
#define NEGATIVE_ONE -1
#endif

int SDL_VideoModeOK (int width, int height, int bpp, Uint32 flags)
{
	int table, b, i;
	int supported;
	SDL_PixelFormat format;
	SDL_Rect **sizes;

	/* Currently 1 and 4 bpp are not supported */
	if ( bpp < 8 || bpp > 32 ) {
		return(0);
	}
	if ( (width <= 0) || (height <= 0) ) {
		return(0);
	}

	/* Search through the list valid of modes */
	SDL_memset(&format, 0, sizeof(format));
	supported = 0;
	table = ((bpp+7)/8)-1;
	SDL_closest_depths[table][0] = bpp;
	SDL_closest_depths[table][7] = 0;
	for ( b = 0; !supported && SDL_closest_depths[table][b]; ++b ) {
		format.BitsPerPixel = SDL_closest_depths[table][b];
		sizes = SDL_ListModes(&format, flags);
		if ( sizes == (SDL_Rect **)0 ) {
			/* No sizes supported at this bit-depth */
			continue;
		} else 
		if (sizes == (SDL_Rect **)NEGATIVE_ONE) {
			/* Any size supported at this bit-depth */
			supported = 1;
			continue;
		} else if (current_video->handles_any_size) {
			/* Driver can center a smaller surface to simulate fullscreen */
			for ( i=0; sizes[i]; ++i ) {
				if ((sizes[i]->w >= width) && (sizes[i]->h >= height)) {
					supported = 1; /* this mode can fit the centered window. */
					break;
				}
			}
		} else
		for ( i=0; sizes[i]; ++i ) {
			if ((sizes[i]->w == width) && (sizes[i]->h == height)) {
				supported = 1;
				break;
			}
		}
	}
	if ( supported ) {
		--b;
		return(SDL_closest_depths[table][b]);
	} else {
		return(0);
	}
}

/*
 * Get the closest non-emulated video mode to the one requested
 */
static int SDL_GetVideoMode (int *w, int *h, int *BitsPerPixel, Uint32 flags)
{
	int table, b, i;
	int supported;
	int native_bpp;
	SDL_PixelFormat format;
	SDL_Rect **sizes;

	/* Check parameters */
	if ( *BitsPerPixel < 8 || *BitsPerPixel > 32 ) {
		SDL_SetError("Invalid bits per pixel (range is {8...32})");
		return(0);
	}
	if ((*w <= 0) || (*h <= 0)) {
		SDL_SetError("Invalid width or height");
		return(0);
	}

	/* Try the original video mode, get the closest depth */
	native_bpp = SDL_VideoModeOK(*w, *h, *BitsPerPixel, flags);
	if ( native_bpp == *BitsPerPixel ) {
		return(1);
	}
	if ( native_bpp > 0 ) {
		*BitsPerPixel = native_bpp;
		return(1);
	}

	/* No exact size match at any depth, look for closest match */
	SDL_memset(&format, 0, sizeof(format));
	supported = 0;
	table = ((*BitsPerPixel+7)/8)-1;
	SDL_closest_depths[table][0] = *BitsPerPixel;
	SDL_closest_depths[table][7] = SDL_VideoSurface->format->BitsPerPixel;
	for ( b = 0; !supported && SDL_closest_depths[table][b]; ++b ) {
		int best;

		format.BitsPerPixel = SDL_closest_depths[table][b];
		sizes = SDL_ListModes(&format, flags);
		if ( sizes == (SDL_Rect **)0 ) {
			/* No sizes supported at this bit-depth */
			continue;
		}
		best=0;
		for ( i=0; sizes[i]; ++i ) {
			/* Mode with both dimensions bigger or equal than asked ? */
			if ((sizes[i]->w >= *w) && (sizes[i]->h >= *h)) {
				/* Mode with any dimension smaller or equal than current best ? */
				if ((sizes[i]->w <= sizes[best]->w) || (sizes[i]->h <= sizes[best]->h)) {
					/* Now choose the mode that has less pixels */
					if ((sizes[i]->w * sizes[i]->h) <= (sizes[best]->w * sizes[best]->h)) {
						best=i;
						supported = 1;
					}
				}
			}
		}
		if (supported) {
			*w=sizes[best]->w;
			*h=sizes[best]->h;
			*BitsPerPixel = SDL_closest_depths[table][b];
		}
	}
	if ( ! supported ) {
		SDL_SetError("No video mode large enough for %dx%d", *w, *h);
	}
	return(supported);
}

/* This should probably go somewhere else -- like SDL_surface.c */
static void SDL_ClearSurface(SDL_Surface *surface)
{
	Uint32 black;

	black = SDL_MapRGB(surface->format, 0, 0, 0);
	SDL_FillRect(surface, NULL, black);
	if ((surface->flags&SDL_HWSURFACE) && (surface->flags&SDL_DOUBLEBUF)) {
		SDL_Flip(surface);
		SDL_FillRect(surface, NULL, black);
	}
	if (surface->flags&SDL_FULLSCREEN) {
		SDL_Flip(surface);
	}
}

/*
 * Create a shadow surface suitable for fooling the app. :-)
 */
static void SDL_CreateShadowSurface(int depth)
{
	Uint32 Rmask, Gmask, Bmask;

	/* Allocate the shadow surface */
	if ( depth == (SDL_VideoSurface->format)->BitsPerPixel ) {
		Rmask = (SDL_VideoSurface->format)->Rmask;
		Gmask = (SDL_VideoSurface->format)->Gmask;
		Bmask = (SDL_VideoSurface->format)->Bmask;
	} else {
		Rmask = Gmask = Bmask = 0;
	}
	SDL_ShadowSurface = SDL_CreateRGBSurface(SDL_SWSURFACE,
				SDL_VideoSurface->w, SDL_VideoSurface->h,
						depth, Rmask, Gmask, Bmask, 0);
	if ( SDL_ShadowSurface == NULL ) {
		return;
	}

	/* 8-bit shadow surfaces report that they have exclusive palette */
	if ( SDL_ShadowSurface->format->palette ) {
		SDL_ShadowSurface->flags |= SDL_HWPALETTE;
		if ( depth == (SDL_VideoSurface->format)->BitsPerPixel ) {
			SDL_memcpy(SDL_ShadowSurface->format->palette->colors,
				SDL_VideoSurface->format->palette->colors,
				SDL_VideoSurface->format->palette->ncolors*
							sizeof(SDL_Color));
		} else {
			SDL_DitherColors(
			SDL_ShadowSurface->format->palette->colors, depth);
		}
	}

	/* If the video surface is resizable, the shadow should say so */
	if ( (SDL_VideoSurface->flags & SDL_RESIZABLE) == SDL_RESIZABLE ) {
		SDL_ShadowSurface->flags |= SDL_RESIZABLE;
	}
	/* If the video surface has no frame, the shadow should say so */
	if ( (SDL_VideoSurface->flags & SDL_NOFRAME) == SDL_NOFRAME ) {
		SDL_ShadowSurface->flags |= SDL_NOFRAME;
	}
	/* If the video surface is fullscreen, the shadow should say so */
	if ( (SDL_VideoSurface->flags & SDL_FULLSCREEN) == SDL_FULLSCREEN ) {
		SDL_ShadowSurface->flags |= SDL_FULLSCREEN;
	}
	/* If the video surface is flippable, the shadow should say so */
	if ( (SDL_VideoSurface->flags & SDL_DOUBLEBUF) == SDL_DOUBLEBUF ) {
		SDL_ShadowSurface->flags |= SDL_DOUBLEBUF;
	}
	return;
}

#ifdef __QNXNTO__
    #include <sys/neutrino.h>
#endif /* __QNXNTO__ */

#ifdef WIN32
	extern int sysevents_mouse_pressed;
#endif

/*
 * Set the requested video mode, allocating a shadow buffer if necessary.
 */
SDL_Surface * SDL_SetVideoMode (int width, int height, int bpp, Uint32 flags)
{
	SDL_VideoDevice *video, *this;
	SDL_Surface *prev_mode, *mode;
	int video_w;
	int video_h;
	int video_bpp;
	int is_opengl;
	SDL_GrabMode saved_grab;

	#ifdef WIN32
		sysevents_mouse_pressed = 0;
	#endif

	/* Start up the video driver, if necessary..
	   WARNING: This is the only function protected this way!
	 */
	if ( ! current_video ) {
		if ( SDL_Init(SDL_INIT_VIDEO|SDL_INIT_NOPARACHUTE) < 0 ) {
			return(NULL);
		}
	}
	this = video = current_video;

	/* Default to the current width and height */
	if ( width == 0 ) {
		width = video->info.current_w;
	}
	if ( height == 0 ) {
		height = video->info.current_h;
	}
	/* Default to the current video bpp */
	if ( bpp == 0 ) {
		flags |= SDL_ANYFORMAT;
		bpp = SDL_VideoSurface->format->BitsPerPixel;
	}

	/* Get a good video mode, the closest one possible */
	video_w = width;
	video_h = height;
	video_bpp = bpp;
	if ( ! SDL_GetVideoMode(&video_w, &video_h, &video_bpp, flags) ) {
		return(NULL);
	}

	/* Check the requested flags */
	/* There's no palette in > 8 bits-per-pixel mode */
	if ( video_bpp > 8 ) {
		flags &= ~SDL_HWPALETTE;
	}
#if 0
	if ( (flags&SDL_FULLSCREEN) != SDL_FULLSCREEN ) {
		/* There's no windowed double-buffering */
		flags &= ~SDL_DOUBLEBUF;
	}
#endif
	if ( (flags&SDL_DOUBLEBUF) == SDL_DOUBLEBUF ) {
		/* Use hardware surfaces when double-buffering */
		flags |= SDL_HWSURFACE;
	}

	is_opengl = ( ( flags & SDL_OPENGL ) == SDL_OPENGL );
	if ( is_opengl ) {
		/* These flags are for 2D video modes only */
		flags &= ~(SDL_HWSURFACE|SDL_DOUBLEBUF);
	}

	/* Reset the keyboard here so event callbacks can run */
	SDL_ResetKeyboard();
	SDL_ResetMouse();
	SDL_SetMouseRange(width, height);
	SDL_cursorstate &= ~CURSOR_USINGSW;

	/* Clean up any previous video mode */
	if ( SDL_PublicSurface != NULL ) {
		SDL_PublicSurface = NULL;
	}
	if ( SDL_ShadowSurface != NULL ) {
		SDL_Surface *ready_to_go;
		ready_to_go = SDL_ShadowSurface;
		SDL_ShadowSurface = NULL;
		SDL_FreeSurface(ready_to_go);
	}
	if ( video->physpal ) {
		SDL_free(video->physpal->colors);
		SDL_free(video->physpal);
		video->physpal = NULL;
	}
	if( video->gammacols) {
		SDL_free(video->gammacols);
		video->gammacols = NULL;
	}

	/* Save the previous grab state and turn off grab for mode switch */
	saved_grab = SDL_WM_GrabInputOff();

	/* Try to set the video mode, along with offset and clipping */
	prev_mode = SDL_VideoSurface;
	SDL_LockCursor();
	SDL_VideoSurface = NULL;	/* In case it's freed by driver */
	mode = video->SetVideoMode(this, prev_mode,video_w,video_h,video_bpp,flags);
	if ( mode ) { /* Prevent resize events from mode change */
          /* But not on OS/2 */
#ifndef __OS2__
	    SDL_PrivateResize(mode->w, mode->h);
#endif

	    /* Sam - If we asked for OpenGL mode, and didn't get it, fail */
	    if ( is_opengl && !(mode->flags & SDL_OPENGL) ) {
		mode = NULL;
		SDL_SetError("OpenGL not available");
	    }
	}
	/*
	 * rcg11292000
	 * If you try to set an SDL_OPENGL surface, and fail to find a
	 * matching  visual, then the next call to SDL_SetVideoMode()
	 * will segfault, since  we no longer point to a dummy surface,
	 * but rather NULL.
	 * Sam 11/29/00
	 * WARNING, we need to make sure that the previous mode hasn't
	 * already been freed by the video driver.  What do we do in
	 * that case?  Should we call SDL_VideoInit() again?
	 */
	SDL_VideoSurface = (mode != NULL) ? mode : prev_mode;

	if ( (mode != NULL) && (!is_opengl) ) {
		/* Sanity check */
		if ( (mode->w < width) || (mode->h < height) ) {
			SDL_SetError("Video mode smaller than requested");
			return(NULL);
		}

		/* If we have a palettized surface, create a default palette */
		if ( mode->format->palette ) {
			SDL_PixelFormat *vf = mode->format;
			SDL_DitherColors(vf->palette->colors, vf->BitsPerPixel);
			video->SetColors(this, 0, vf->palette->ncolors,
			                           vf->palette->colors);
		}

		/* Clear the surface to black */
		video->offset_x = 0;
		video->offset_y = 0;
		mode->offset = 0;
		SDL_SetClipRect(mode, NULL);
		SDL_ClearSurface(mode);

		/* Now adjust the offsets to match the desired mode */
		video->offset_x = (mode->w-width)/2;
		video->offset_y = (mode->h-height)/2;
		mode->offset = video->offset_y*mode->pitch +
				video->offset_x*mode->format->BytesPerPixel;
#ifdef DEBUG_VIDEO
  fprintf(stderr,
	"Requested mode: %dx%dx%d, obtained mode %dx%dx%d (offset %d)\n",
		width, height, bpp,
		mode->w, mode->h, mode->format->BitsPerPixel, mode->offset);
#endif
		mode->w = width;
		mode->h = height;
		SDL_SetClipRect(mode, NULL);
	}
	SDL_ResetCursor();
	SDL_UnlockCursor();

	/* If we failed setting a video mode, return NULL... (Uh Oh!) */
	if ( mode == NULL ) {
		return(NULL);
	}

	/* If there is no window manager, set the SDL_NOFRAME flag */
	if ( ! video->info.wm_available ) {
		mode->flags |= SDL_NOFRAME;
	}

	/* Reset the mouse cursor and grab for new video mode */
	SDL_SetCursor(NULL);
	if ( video->UpdateMouse ) {
		video->UpdateMouse(this);
	}
	SDL_WM_GrabInput(saved_grab);
	SDL_GetRelativeMouseState(NULL, NULL); /* Clear first large delta */

#if SDL_VIDEO_OPENGL
	/* Load GL symbols (before MakeCurrent, where we need glGetString). */
	if ( flags & (SDL_OPENGL | SDL_OPENGLBLIT) ) {

#if defined(__QNXNTO__) && (_NTO_VERSION < 630)
#define __SDL_NOGETPROCADDR__
#elif defined(__MINT__)
#define __SDL_NOGETPROCADDR__
#endif
#ifdef __SDL_NOGETPROCADDR__
    #define SDL_PROC(ret,func,params) video->func=func;
#else
    #define SDL_PROC(ret,func,params) \
    do { \
        video->func = SDL_GL_GetProcAddress(#func); \
        if ( ! video->func ) { \
            SDL_SetError("Couldn't load GL function %s: %s\n", #func, SDL_GetError()); \
        return(NULL); \
        } \
    } while ( 0 );

#endif /* __SDL_NOGETPROCADDR__ */

#include "SDL_glfuncs.h"
#undef SDL_PROC	
	}
#endif /* SDL_VIDEO_OPENGL */

	/* If we're running OpenGL, make the context current */
	if ( (video->screen->flags & SDL_OPENGL) &&
	      video->GL_MakeCurrent ) {
		if ( video->GL_MakeCurrent(this) < 0 ) {
			return(NULL);
		}
	}

	/* Set up a fake SDL surface for OpenGL "blitting" */
	if ( (flags & SDL_OPENGLBLIT) == SDL_OPENGLBLIT ) {
		/* Load GL functions for performing the texture updates */
#if SDL_VIDEO_OPENGL

		/* Create a software surface for blitting */
#ifdef GL_VERSION_1_2
		/* If the implementation either supports the packed pixels
		   extension, or implements the core OpenGL 1.2 API, it will
		   support the GL_UNSIGNED_SHORT_5_6_5 texture format.
		 */
		if ( (bpp == 16) &&
		     (SDL_strstr((const char *)video->glGetString(GL_EXTENSIONS), "GL_EXT_packed_pixels") ||
		     (SDL_atof((const char *)video->glGetString(GL_VERSION)) >= 1.2f))
		   ) {
			video->is_32bit = 0;
			SDL_VideoSurface = SDL_CreateRGBSurface(
				flags, 
				width, 
				height,  
				16,
				31 << 11,
				63 << 5,
				31,
				0
				);
		}
		else
#endif /* OpenGL 1.2 */
		{
			video->is_32bit = 1;
			SDL_VideoSurface = SDL_CreateRGBSurface(
				flags, 
				width, 
				height, 
				32, 
#if SDL_BYTEORDER == SDL_LIL_ENDIAN
				0x000000FF,
				0x0000FF00,
				0x00FF0000,
				0xFF000000
#else
				0xFF000000,
				0x00FF0000,
				0x0000FF00,
				0x000000FF
#endif
				);
		}
		if ( ! SDL_VideoSurface ) {
			return(NULL);
		}
		SDL_VideoSurface->flags = mode->flags | SDL_OPENGLBLIT;

		/* Free the original video mode surface (is this safe?) */
		SDL_FreeSurface(mode);

		/* Set the surface completely opaque & white by default */
		SDL_memset( SDL_VideoSurface->pixels, 255, SDL_VideoSurface->h * SDL_VideoSurface->pitch );
		video->glGenTextures( 1, &video->texture );
		video->glBindTexture( GL_TEXTURE_2D, video->texture );
		video->glTexImage2D(
			GL_TEXTURE_2D,
			0,
			video->is_32bit ? GL_RGBA : GL_RGB,
			256,
			256,
			0,
			video->is_32bit ? GL_RGBA : GL_RGB,
#ifdef GL_VERSION_1_2
			video->is_32bit ? GL_UNSIGNED_BYTE : GL_UNSIGNED_SHORT_5_6_5,
#else
			GL_UNSIGNED_BYTE,
#endif
			NULL);

		video->UpdateRects = SDL_GL_UpdateRectsLock;
#else
		SDL_SetError("Somebody forgot to #define SDL_VIDEO_OPENGL");
		return(NULL);
#endif
	}

	/* Create a shadow surface if necessary */
	/* There are three conditions under which we create a shadow surface:
		1.  We need a particular bits-per-pixel that we didn't get.
		2.  We need a hardware palette and didn't get one.
		3.  We need a software surface and got a hardware surface.
	*/
	if ( !(SDL_VideoSurface->flags & SDL_OPENGL) &&
	     (
	     (  !(flags&SDL_ANYFORMAT) &&
			(SDL_VideoSurface->format->BitsPerPixel != bpp)) ||
	     (   (flags&SDL_HWPALETTE) && 
				!(SDL_VideoSurface->flags&SDL_HWPALETTE)) ||
		/* If the surface is in hardware, video writes are visible
		   as soon as they are performed, so we need to buffer them
		 */
	     (   ((flags&SDL_HWSURFACE) == SDL_SWSURFACE) &&
				(SDL_VideoSurface->flags&SDL_HWSURFACE)) ||
	     (   (flags&SDL_DOUBLEBUF) &&
				(SDL_VideoSurface->flags&SDL_HWSURFACE) &&
				!(SDL_VideoSurface->flags&SDL_DOUBLEBUF))
	     ) ) {
		SDL_CreateShadowSurface(bpp);
		if ( SDL_ShadowSurface == NULL ) {
			SDL_SetError("Couldn't create shadow surface");
			return(NULL);
		}
		SDL_PublicSurface = SDL_ShadowSurface;
	} else {
		SDL_PublicSurface = SDL_VideoSurface;
	}
	video->info.vfmt = SDL_VideoSurface->format;
	video->info.current_w = SDL_VideoSurface->w;
	video->info.current_h = SDL_VideoSurface->h;

	/* We're done! */
	return(SDL_PublicSurface);
}

/* 
 * Convert a surface into the video pixel format.
 */
SDL_Surface * SDL_DisplayFormat (SDL_Surface *surface)
{
	Uint32 flags;

	if ( ! SDL_PublicSurface ) {
		SDL_SetError("No video mode has been set");
		return(NULL);
	}
	/* Set the flags appropriate for copying to display surface */
	if (((SDL_PublicSurface->flags&SDL_HWSURFACE) == SDL_HWSURFACE) && current_video->info.blit_hw)
		flags = SDL_HWSURFACE;
	else 
		flags = SDL_SWSURFACE;
#ifdef AUTORLE_DISPLAYFORMAT
	flags |= (surface->flags & (SDL_SRCCOLORKEY|SDL_SRCALPHA));
	flags |= SDL_RLEACCELOK;
#else
	flags |= surface->flags & (SDL_SRCCOLORKEY|SDL_SRCALPHA|SDL_RLEACCELOK);
#endif
	return(SDL_ConvertSurface(surface, SDL_PublicSurface->format, flags));
}

/*
 * Convert a surface into a format that's suitable for blitting to
 * the screen, but including an alpha channel.
 */
SDL_Surface *SDL_DisplayFormatAlpha(SDL_Surface *surface)
{
	SDL_PixelFormat *vf;
	SDL_PixelFormat *format;
	SDL_Surface *converted;
	Uint32 flags;
	/* default to ARGB8888 */
	Uint32 amask = 0xff000000;
	Uint32 rmask = 0x00ff0000;
	Uint32 gmask = 0x0000ff00;
	Uint32 bmask = 0x000000ff;

	if ( ! SDL_PublicSurface ) {
		SDL_SetError("No video mode has been set");
		return(NULL);
	}
	vf = SDL_PublicSurface->format;

	switch(vf->BytesPerPixel) {
	    case 2:
		/* For XGY5[56]5, use, AXGY8888, where {X, Y} = {R, B}.
		   For anything else (like ARGB4444) it doesn't matter
		   since we have no special code for it anyway */
		if ( (vf->Rmask == 0x1f) &&
		     (vf->Bmask == 0xf800 || vf->Bmask == 0x7c00)) {
			rmask = 0xff;
			bmask = 0xff0000;
		}
		break;

	    case 3:
	    case 4:
		/* Keep the video format, as long as the high 8 bits are
		   unused or alpha */
		if ( (vf->Rmask == 0xff) && (vf->Bmask == 0xff0000) ) {
			rmask = 0xff;
			bmask = 0xff0000;
		} else if ( vf->Rmask == 0xFF00 && (vf->Bmask == 0xFF000000) ) {
			amask = 0x000000FF;
			rmask = 0x0000FF00;
			gmask = 0x00FF0000;
			bmask = 0xFF000000;
		}
		break;

	    default:
		/* We have no other optimised formats right now. When/if a new
		   optimised alpha format is written, add the converter here */
		break;
	}
	format = SDL_AllocFormat(32, rmask, gmask, bmask, amask);
	flags = SDL_PublicSurface->flags & SDL_HWSURFACE;
	flags |= surface->flags & (SDL_SRCALPHA | SDL_RLEACCELOK);
	converted = SDL_ConvertSurface(surface, format, flags);
	SDL_FreeFormat(format);
	return(converted);
}

/*
 * Update a specific portion of the physical screen
 */
void SDL_UpdateRect(SDL_Surface *screen, Sint32 x, Sint32 y, Uint32 w, Uint32 h)
{
	if ( screen ) {
		SDL_Rect rect;

		/* Perform some checking */
		if ( w == 0 )
			w = screen->w;
		if ( h == 0 )
			h = screen->h;
		if ( (int)(x+w) > screen->w )
			return;
		if ( (int)(y+h) > screen->h )
			return;

		/* Fill the rectangle */
		rect.x = (Sint16)x;
		rect.y = (Sint16)y;
		rect.w = (Uint16)w;
		rect.h = (Uint16)h;
		SDL_UpdateRects(screen, 1, &rect);
	}
}
void SDL_UpdateRects (SDL_Surface *screen, int numrects, SDL_Rect *rects)
{
	int i;
	SDL_VideoDevice *video = current_video;
	SDL_VideoDevice *this = current_video;

	if ( (screen->flags & (SDL_OPENGL | SDL_OPENGLBLIT)) == SDL_OPENGL ) {
		SDL_SetError("OpenGL active, use SDL_GL_SwapBuffers()");
		return;
	}
	if ( screen == SDL_ShadowSurface ) {
		/* Blit the shadow surface using saved mapping */
		SDL_Palette *pal = screen->format->palette;
		SDL_Color *saved_colors = NULL;
		if ( pal && !(SDL_VideoSurface->flags & SDL_HWPALETTE) ) {
			/* simulated 8bpp, use correct physical palette */
			saved_colors = pal->colors;
			if ( video->gammacols ) {
				/* gamma-corrected palette */
				pal->colors = video->gammacols;
			} else if ( video->physpal ) {
				/* physical palette different from logical */
				pal->colors = video->physpal->colors;
			}
		}
		if ( SHOULD_DRAWCURSOR(SDL_cursorstate) ) {
			SDL_LockCursor();
			SDL_DrawCursor(SDL_ShadowSurface);
			for ( i=0; i<numrects; ++i ) {
				SDL_LowerBlit(SDL_ShadowSurface, &rects[i], 
						SDL_VideoSurface, &rects[i]);
			}
			SDL_EraseCursor(SDL_ShadowSurface);
			SDL_UnlockCursor();
		} else {
			for ( i=0; i<numrects; ++i ) {
				SDL_LowerBlit(SDL_ShadowSurface, &rects[i], 
						SDL_VideoSurface, &rects[i]);
			}
		}
		if ( saved_colors ) {
			pal->colors = saved_colors;
		}

		/* Fall through to video surface update */
		screen = SDL_VideoSurface;
	}
	if ( screen == SDL_VideoSurface ) {
		/* Update the video surface */
		if ( screen->offset ) {
			for ( i=0; i<numrects; ++i ) {
				rects[i].x += video->offset_x;
				rects[i].y += video->offset_y;
			}
			video->UpdateRects(this, numrects, rects);
			for ( i=0; i<numrects; ++i ) {
				rects[i].x -= video->offset_x;
				rects[i].y -= video->offset_y;
			}
		} else {
			video->UpdateRects(this, numrects, rects);
		}
	}
}

/*
 * Performs hardware double buffering, if possible, or a full update if not.
 */
int SDL_Flip(SDL_Surface *screen)
{
	SDL_VideoDevice *video = current_video;
	/* Copy the shadow surface to the video surface */
	if ( screen == SDL_ShadowSurface ) {
		SDL_Rect rect;
		SDL_Palette *pal = screen->format->palette;
		SDL_Color *saved_colors = NULL;
		if ( pal && !(SDL_VideoSurface->flags & SDL_HWPALETTE) ) {
			/* simulated 8bpp, use correct physical palette */
			saved_colors = pal->colors;
			if ( video->gammacols ) {
				/* gamma-corrected palette */
				pal->colors = video->gammacols;
			} else if ( video->physpal ) {
				/* physical palette different from logical */
				pal->colors = video->physpal->colors;
			}
		}

		rect.x = 0;
		rect.y = 0;
		rect.w = screen->w;
		rect.h = screen->h;
		if ( SHOULD_DRAWCURSOR(SDL_cursorstate) ) {
			SDL_LockCursor();
			SDL_DrawCursor(SDL_ShadowSurface);
			SDL_LowerBlit(SDL_ShadowSurface, &rect,
					SDL_VideoSurface, &rect);
			SDL_EraseCursor(SDL_ShadowSurface);
			SDL_UnlockCursor();
		} else {
			SDL_LowerBlit(SDL_ShadowSurface, &rect,
					SDL_VideoSurface, &rect);
		}
		if ( saved_colors ) {
			pal->colors = saved_colors;
		}

		/* Fall through to video surface update */
		screen = SDL_VideoSurface;
	}
	if ( (screen->flags & SDL_DOUBLEBUF) == SDL_DOUBLEBUF ) {
		SDL_VideoDevice *this  = current_video;
		return(video->FlipHWSurface(this, SDL_VideoSurface));
	} else {
		SDL_UpdateRect(screen, 0, 0, 0, 0);
	}
	return(0);
}

static void SetPalette_logical(SDL_Surface *screen, SDL_Color *colors,
			       int firstcolor, int ncolors)
{
	SDL_Palette *pal = screen->format->palette;
	SDL_Palette *vidpal;

	if ( colors != (pal->colors + firstcolor) ) {
		SDL_memcpy(pal->colors + firstcolor, colors,
		       ncolors * sizeof(*colors));
	}

	if ( current_video && SDL_VideoSurface ) {
		vidpal = SDL_VideoSurface->format->palette;
		if ( (screen == SDL_ShadowSurface) && vidpal ) {
			/*
			 * This is a shadow surface, and the physical
			 * framebuffer is also indexed. Propagate the
			 * changes to its logical palette so that
			 * updates are always identity blits
			 */
			SDL_memcpy(vidpal->colors + firstcolor, colors,
			       ncolors * sizeof(*colors));
		}
	}
	SDL_FormatChanged(screen);
}

static int SetPalette_physical(SDL_Surface *screen,
                               SDL_Color *colors, int firstcolor, int ncolors)
{
	SDL_VideoDevice *video = current_video;
	int gotall = 1;

	if ( video->physpal ) {
		/* We need to copy the new colors, since we haven't
		 * already done the copy in the logical set above.
		 */
		SDL_memcpy(video->physpal->colors + firstcolor,
		       colors, ncolors * sizeof(*colors));
	}
	if ( screen == SDL_ShadowSurface ) {
		if ( SDL_VideoSurface->flags & SDL_HWPALETTE ) {
			/*
			 * The real screen is also indexed - set its physical
			 * palette. The physical palette does not include the
			 * gamma modification, we apply it directly instead,
			 * but this only happens if we have hardware palette.
			 */
			screen = SDL_VideoSurface;
		} else {
			/*
			 * The video surface is not indexed - invalidate any
			 * active shadow-to-video blit mappings.
			 */
			if ( screen->map->dst == SDL_VideoSurface ) {
				SDL_InvalidateMap(screen->map);
			}
			if ( video->gamma ) {
				if( ! video->gammacols ) {
					SDL_Palette *pp = video->physpal;
					if(!pp)
						pp = screen->format->palette;
					video->gammacols = SDL_malloc(pp->ncolors
							  * sizeof(SDL_Color));
					SDL_ApplyGamma(video->gamma,
						       pp->colors,
						       video->gammacols,
						       pp->ncolors);
				} else {
					SDL_ApplyGamma(video->gamma, colors,
						       video->gammacols
						       + firstcolor,
						       ncolors);
				}
			}
			SDL_UpdateRect(screen, 0, 0, 0, 0);
		}
	}

	if ( screen == SDL_VideoSurface ) {
		SDL_Color gcolors[256];

		if ( video->gamma ) {
			SDL_ApplyGamma(video->gamma, colors, gcolors, ncolors);
			colors = gcolors;
		}
		gotall = video->SetColors(video, firstcolor, ncolors, colors);
		if ( ! gotall ) {
			/* The video flags shouldn't have SDL_HWPALETTE, and
			   the video driver is responsible for copying back the
			   correct colors into the video surface palette.
			*/
			;
		}
		SDL_CursorPaletteChanged();
	}
	return gotall;
}

/*
 * Set the physical and/or logical colormap of a surface:
 * Only the screen has a physical colormap. It determines what is actually
 * sent to the display.
 * The logical colormap is used to map blits to/from the surface.
 * 'which' is one or both of SDL_LOGPAL, SDL_PHYSPAL
 *
 * Return nonzero if all colours were set as requested, or 0 otherwise.
 */
int SDL_SetPalette(SDL_Surface *screen, int which,
		   SDL_Color *colors, int firstcolor, int ncolors)
{
	SDL_Palette *pal;
	int gotall;
	int palsize;

	if ( !screen ) {
		return 0;
	}
	if ( !current_video || screen != SDL_PublicSurface ) {
		/* only screens have physical palettes */
		which &= ~SDL_PHYSPAL;
	} else if ( (screen->flags & SDL_HWPALETTE) != SDL_HWPALETTE ) {
		/* hardware palettes required for split colormaps */
		which |= SDL_PHYSPAL | SDL_LOGPAL;
	}

	/* Verify the parameters */
	pal = screen->format->palette;
	if( !pal ) {
		return 0;	/* not a palettized surface */
	}
	gotall = 1;
	palsize = 1 << screen->format->BitsPerPixel;
	if ( ncolors > (palsize - firstcolor) ) {
		ncolors = (palsize - firstcolor);
		gotall = 0;
	}

	if ( which & SDL_LOGPAL ) {
		/*
		 * Logical palette change: The actual screen isn't affected,
		 * but the internal colormap is altered so that the
		 * interpretation of the pixel values (for blits etc) is
		 * changed.
		 */
		SetPalette_logical(screen, colors, firstcolor, ncolors);
	}
	if ( which & SDL_PHYSPAL ) {
		SDL_VideoDevice *video = current_video;
		/*
		 * Physical palette change: This doesn't affect the
		 * program's idea of what the screen looks like, but changes
		 * its actual appearance.
		 */
		if ( !video->physpal && !(which & SDL_LOGPAL) ) {
			/* Lazy physical palette allocation */
			int size;
			SDL_Palette *pp = SDL_malloc(sizeof(*pp));
			if ( !pp ) {
				return 0;
			}
			video->physpal = pp;
			pp->ncolors = pal->ncolors;
			size = pp->ncolors * sizeof(SDL_Color);
			pp->colors = SDL_malloc(size);
			if ( !pp->colors ) {
				return 0;
			}
			SDL_memcpy(pp->colors, pal->colors, size);
		}
		if ( ! SetPalette_physical(screen,
		                           colors, firstcolor, ncolors) ) {
			gotall = 0;
		}
	}
	return gotall;
}

int SDL_SetColors(SDL_Surface *screen, SDL_Color *colors, int firstcolor,
		  int ncolors)
{
	return SDL_SetPalette(screen, SDL_LOGPAL | SDL_PHYSPAL,
			      colors, firstcolor, ncolors);
}

/*
 * Clean up the video subsystem
 */
void SDL_VideoQuit (void)
{
	SDL_Surface *ready_to_go;

	if ( current_video ) {
		SDL_VideoDevice *video = current_video;
		SDL_VideoDevice *this  = current_video;

		/* Halt event processing before doing anything else */
		SDL_StopEventLoop();

		/* Clean up allocated window manager items */
		if ( SDL_PublicSurface ) {
			SDL_PublicSurface = NULL;
		}
		SDL_CursorQuit();

		/* Just in case... */
		SDL_WM_GrabInputOff();

		/* Clean up the system video */
		video->VideoQuit(this);

		/* Free any lingering surfaces */
		ready_to_go = SDL_ShadowSurface;
		SDL_ShadowSurface = NULL;
		SDL_FreeSurface(ready_to_go);
		if ( SDL_VideoSurface != NULL ) {
			ready_to_go = SDL_VideoSurface;
			SDL_VideoSurface = NULL;
			SDL_FreeSurface(ready_to_go);
		}
		SDL_PublicSurface = NULL;

		/* Clean up miscellaneous memory */
		if ( video->physpal ) {
			SDL_free(video->physpal->colors);
			SDL_free(video->physpal);
			video->physpal = NULL;
		}
		if ( video->gammacols ) {
			SDL_free(video->gammacols);
			video->gammacols = NULL;
		}
		if ( video->gamma ) {
			SDL_free(video->gamma);
			video->gamma = NULL;
		}
		if ( video->wm_title != NULL ) {
			SDL_free(video->wm_title);
			video->wm_title = NULL;
		}
		if ( video->wm_icon != NULL ) {
			SDL_free(video->wm_icon);
			video->wm_icon = NULL;
		}

		/* Finish cleaning up video subsystem */
		video->free(this);
		current_video = NULL;
	}
	return;
}

/* Load the GL driver library */
int SDL_GL_LoadLibrary(const char *path)
{
	SDL_VideoDevice *video = current_video;
	SDL_VideoDevice *this = current_video;
	int retval;

	retval = -1;
	if ( video == NULL ) {
		SDL_SetError("Video subsystem has not been initialized");
	} else {
		if ( video->GL_LoadLibrary ) {
			retval = video->GL_LoadLibrary(this, path);
		} else {
			SDL_SetError("No dynamic GL support in video driver");
		}
	}
	return(retval);
}

void *SDL_GL_GetProcAddress(const char* proc)
{
	SDL_VideoDevice *video = current_video;
	SDL_VideoDevice *this = current_video;
	void *func;

	func = NULL;
	if ( video->GL_GetProcAddress ) {
		if ( video->gl_config.driver_loaded ) {
			func = video->GL_GetProcAddress(this, proc);
		} else {
			SDL_SetError("No GL driver has been loaded");
		}
	} else {
		SDL_SetError("No dynamic GL support in video driver");
	}
	return func;
}

/* Set the specified GL attribute for setting up a GL video mode */
int SDL_GL_SetAttribute( SDL_GLattr attr, int value )
{
	int retval;
	SDL_VideoDevice *video = current_video;

	retval = 0;
	switch (attr) {
		case SDL_GL_RED_SIZE:
			video->gl_config.red_size = value;
			break;
		case SDL_GL_GREEN_SIZE:
			video->gl_config.green_size = value;
			break;
		case SDL_GL_BLUE_SIZE:
			video->gl_config.blue_size = value;
			break;
		case SDL_GL_ALPHA_SIZE:
			video->gl_config.alpha_size = value;
			break;
		case SDL_GL_DOUBLEBUFFER:
			video->gl_config.double_buffer = value;
			break;
		case SDL_GL_BUFFER_SIZE:
			video->gl_config.buffer_size = value;
			break;
		case SDL_GL_DEPTH_SIZE:
			video->gl_config.depth_size = value;
			break;
		case SDL_GL_STENCIL_SIZE:
			video->gl_config.stencil_size = value;
			break;
		case SDL_GL_ACCUM_RED_SIZE:
			video->gl_config.accum_red_size = value;
			break;
		case SDL_GL_ACCUM_GREEN_SIZE:
			video->gl_config.accum_green_size = value;
			break;
		case SDL_GL_ACCUM_BLUE_SIZE:
			video->gl_config.accum_blue_size = value;
			break;
		case SDL_GL_ACCUM_ALPHA_SIZE:
			video->gl_config.accum_alpha_size = value;
			break;
		case SDL_GL_STEREO:
			video->gl_config.stereo = value;
			break;
		case SDL_GL_MULTISAMPLEBUFFERS:
			video->gl_config.multisamplebuffers = value;
			break;
		case SDL_GL_MULTISAMPLESAMPLES:
			video->gl_config.multisamplesamples = value;
			break;
		case SDL_GL_ACCELERATED_VISUAL:
			video->gl_config.accelerated = value;
			break;
		case SDL_GL_SWAP_CONTROL:
			video->gl_config.swap_control = value;
			break;
		default:
			SDL_SetError("Unknown OpenGL attribute");
			retval = -1;
			break;
	}
	return(retval);
}

/* Retrieve an attribute value from the windowing system. */
int SDL_GL_GetAttribute(SDL_GLattr attr, int* value)
{
	int retval = -1;
	SDL_VideoDevice* video = current_video;
	SDL_VideoDevice* this = current_video;

	if ( video->GL_GetAttribute ) {
		retval = this->GL_GetAttribute(this, attr, value);
	} else {
		*value = 0;
		SDL_SetError("GL_GetAttribute not supported");
	}
	return retval;
}

/* Perform a GL buffer swap on the current GL context */
void SDL_GL_SwapBuffers(void)
{
	SDL_VideoDevice *video = current_video;
	SDL_VideoDevice *this = current_video;

	if ( video->screen->flags & SDL_OPENGL ) {
		video->GL_SwapBuffers(this);
	} else {
		SDL_SetError("OpenGL video mode has not been set");
	}
}

/* Update rects with locking */
void SDL_GL_UpdateRectsLock(SDL_VideoDevice* this, int numrects, SDL_Rect *rects)
{
	SDL_GL_Lock();
 	SDL_GL_UpdateRects(numrects, rects);
	SDL_GL_Unlock();
}

/* Update rects without state setting and changing (the caller is responsible for it) */
void SDL_GL_UpdateRects(int numrects, SDL_Rect *rects)
{
#if SDL_VIDEO_OPENGL
	SDL_VideoDevice *this = current_video;
	SDL_Rect update, tmp;
	int x, y, i;

	for ( i = 0; i < numrects; i++ )
	{
		tmp.y = rects[i].y;
		tmp.h = rects[i].h;
		for ( y = 0; y <= rects[i].h / 256; y++ )
		{
			tmp.x = rects[i].x;
			tmp.w = rects[i].w;
			for ( x = 0; x <= rects[i].w / 256; x++ )
			{
				update.x = tmp.x;
				update.y = tmp.y;
				update.w = tmp.w;
				update.h = tmp.h;

				if ( update.w > 256 )
					update.w = 256;

				if ( update.h > 256 )
					update.h = 256;
			
				this->glFlush();
				this->glTexSubImage2D( 
					GL_TEXTURE_2D, 
					0, 
					0, 
					0, 
					update.w, 
					update.h, 
					this->is_32bit? GL_RGBA : GL_RGB,
#ifdef GL_VERSION_1_2
					this->is_32bit ? GL_UNSIGNED_BYTE : GL_UNSIGNED_SHORT_5_6_5,
#else
					GL_UNSIGNED_BYTE,
#endif
					(Uint8 *)this->screen->pixels + 
						this->screen->format->BytesPerPixel * update.x + 
						update.y * this->screen->pitch );
	
				this->glFlush();
				/*
				* Note the parens around the function name:
				* This is because some OpenGL implementations define glTexCoord etc 
				* as macros, and we don't want them expanded here.
				*/
				this->glBegin(GL_TRIANGLE_STRIP);
					(this->glTexCoord2f)( 0.0, 0.0 );	
					(this->glVertex2i)( update.x, update.y );
					(this->glTexCoord2f)( (float)(update.w / 256.0), 0.0 );	
					(this->glVertex2i)( update.x + update.w, update.y );
					(this->glTexCoord2f)( 0.0, (float)(update.h / 256.0) );
					(this->glVertex2i)( update.x, update.y + update.h );
					(this->glTexCoord2f)( (float)(update.w / 256.0), (float)(update.h / 256.0) );	
					(this->glVertex2i)( update.x + update.w	, update.y + update.h );
				this->glEnd();	
			
				tmp.x += 256;
				tmp.w -= 256;
			}
			tmp.y += 256;
			tmp.h -= 256;
		}
	}
#endif
}

/* Lock == save current state */
void SDL_GL_Lock()
{
#if SDL_VIDEO_OPENGL
	lock_count--;
	if (lock_count==-1)
	{
		SDL_VideoDevice *this = current_video;

		this->glPushAttrib( GL_ALL_ATTRIB_BITS );	/* TODO: narrow range of what is saved */
#ifdef GL_CLIENT_PIXEL_STORE_BIT
		this->glPushClientAttrib( GL_CLIENT_PIXEL_STORE_BIT );
#endif

		this->glEnable(GL_TEXTURE_2D);
		this->glEnable(GL_BLEND);
		this->glDisable(GL_FOG);
		this->glDisable(GL_ALPHA_TEST);
		this->glDisable(GL_DEPTH_TEST);
		this->glDisable(GL_SCISSOR_TEST);	
		this->glDisable(GL_STENCIL_TEST);
		this->glDisable(GL_CULL_FACE);

		this->glBindTexture( GL_TEXTURE_2D, this->texture );
		this->glTexEnvf( GL_TEXTURE_ENV, GL_TEXTURE_ENV_MODE, GL_MODULATE );
		this->glTexParameteri( GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST );
		this->glTexParameteri( GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST );
		this->glTexParameteri( GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_REPEAT );
		this->glTexParameteri( GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_REPEAT );

		this->glPixelStorei( GL_UNPACK_ROW_LENGTH, this->screen->pitch / this->screen->format->BytesPerPixel );
		this->glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
		(this->glColor4f)(1.0, 1.0, 1.0, 1.0);		/* Solaris workaround */

		this->glViewport(0, 0, this->screen->w, this->screen->h);
		this->glMatrixMode(GL_PROJECTION);
		this->glPushMatrix();
		this->glLoadIdentity();

		this->glOrtho(0.0, (GLdouble) this->screen->w, (GLdouble) this->screen->h, 0.0, 0.0, 1.0);

		this->glMatrixMode(GL_MODELVIEW);
		this->glPushMatrix();
		this->glLoadIdentity();
	}
#endif
}

/* Unlock == restore saved state */
void SDL_GL_Unlock()
{
#if SDL_VIDEO_OPENGL
	lock_count++;
	if (lock_count==0)
	{
		SDL_VideoDevice *this = current_video;

		this->glPopMatrix();
		this->glMatrixMode(GL_PROJECTION);
		this->glPopMatrix();

		this->glPopClientAttrib();
		this->glPopAttrib();
	}
#endif
}


void SDL_Audio_SetCaption(const char *caption);

/*
 * Sets/Gets the title and icon text of the display window, if any.
 */
void SDL_WM_SetCaption (const char *title, const char *icon)
{
	SDL_VideoDevice *video = current_video;
	SDL_VideoDevice *this  = current_video;

	if ( video ) {
		if ( title ) {
			if ( video->wm_title ) {
				SDL_free(video->wm_title);
			}
			video->wm_title = SDL_strdup(title);
		}
		if ( icon ) {
			if ( video->wm_icon ) {
				SDL_free(video->wm_icon);
			}
			video->wm_icon = SDL_strdup(icon);
		}
		if ( (title || icon) && (video->SetCaption != NULL) ) {
			video->SetCaption(this, video->wm_title,video->wm_icon);
		}
	}

	/* PulseAudio can make use of this information. */
	SDL_Audio_SetCaption(title);
}

void SDL_WM_GetCaption (char **title, char **icon)
{
	SDL_VideoDevice *video = current_video;

	if ( video ) {
		if ( title ) {
			*title = video->wm_title;
		}
		if ( icon ) {
			*icon = video->wm_icon;
		}
	}
}

/* Utility function used by SDL_WM_SetIcon();
 * flags & 1 for color key, flags & 2 for alpha channel. */
static void CreateMaskFromColorKeyOrAlpha(SDL_Surface *icon, Uint8 *mask, int flags)
{
	int x, y;
	Uint32 colorkey;
#define SET_MASKBIT(icon, x, y, mask) \
	mask[(y*((icon->w+7)/8))+(x/8)] &= ~(0x01<<(7-(x%8)))

	colorkey = icon->format->colorkey;
	switch (icon->format->BytesPerPixel) {
		case 1: { Uint8 *pixels;
			for ( y=0; y<icon->h; ++y ) {
				pixels = (Uint8 *)icon->pixels + y*icon->pitch;
				for ( x=0; x<icon->w; ++x ) {
					if ( *pixels++ == colorkey ) {
						SET_MASKBIT(icon, x, y, mask);
					}
				}
			}
		}
		break;

		case 2: { Uint16 *pixels;
			for ( y=0; y<icon->h; ++y ) {
				pixels = (Uint16 *)icon->pixels +
				                   y*icon->pitch/2;
				for ( x=0; x<icon->w; ++x ) {
					if ( (flags & 1) && *pixels == colorkey ) {
						SET_MASKBIT(icon, x, y, mask);
					} else if((flags & 2) && (*pixels & icon->format->Amask) == 0) {
						SET_MASKBIT(icon, x, y, mask);
					}
					pixels++;
				}
			}
		}
		break;

		case 4: { Uint32 *pixels;
			for ( y=0; y<icon->h; ++y ) {
				pixels = (Uint32 *)icon->pixels +
				                   y*icon->pitch/4;
				for ( x=0; x<icon->w; ++x ) {
					if ( (flags & 1) && *pixels == colorkey ) {
						SET_MASKBIT(icon, x, y, mask);
					} else if((flags & 2) && (*pixels & icon->format->Amask) == 0) {
						SET_MASKBIT(icon, x, y, mask);
					}
					pixels++;
				}
			}
		}
		break;
	}
}

/*
 * Sets the window manager icon for the display window.
 */
void SDL_WM_SetIcon (SDL_Surface *icon, Uint8 *mask)
{
	SDL_VideoDevice *video = current_video;
	SDL_VideoDevice *this  = current_video;

	if ( icon && video->SetIcon ) {
		/* Generate a mask if necessary, and create the icon! */
		if ( mask == NULL ) {
			int mask_len = icon->h*(icon->w+7)/8;
			int flags = 0;
			mask = (Uint8 *)SDL_malloc(mask_len);
			if ( mask == NULL ) {
				return;
			}
			SDL_memset(mask, ~0, mask_len);
			if ( icon->flags & SDL_SRCCOLORKEY ) flags |= 1;
			if ( icon->flags & SDL_SRCALPHA ) flags |= 2;
			if( flags ) {
				CreateMaskFromColorKeyOrAlpha(icon, mask, flags);
			}
			video->SetIcon(video, icon, mask);
			SDL_free(mask);
		} else {
			video->SetIcon(this, icon, mask);
		}
	}
}

/*
 * Grab or ungrab the keyboard and mouse input.
 * This function returns the final grab mode after calling the
 * driver dependent function.
 */
static SDL_GrabMode SDL_WM_GrabInputRaw(SDL_GrabMode mode)
{
	SDL_VideoDevice *video = current_video;
	SDL_VideoDevice *this  = current_video;

	/* Only do something if we have support for grabs */
	if ( video->GrabInput == NULL ) {
		return(video->input_grab);
	}

	/* If the final grab mode if off, only then do we actually grab */
#ifdef DEBUG_GRAB
  printf("SDL_WM_GrabInputRaw(%d) ... ", mode);
#endif
	if ( mode == SDL_GRAB_OFF ) {
		if ( video->input_grab != SDL_GRAB_OFF ) {
			mode = video->GrabInput(this, mode);
		}
	} else {
		if ( video->input_grab == SDL_GRAB_OFF ) {
			mode = video->GrabInput(this, mode);
		}
	}
	if ( mode != video->input_grab ) {
		video->input_grab = mode;
		if ( video->CheckMouseMode ) {
			video->CheckMouseMode(this);
		}
	}
#ifdef DEBUG_GRAB
  printf("Final mode %d\n", video->input_grab);
#endif

	/* Return the final grab state */
	if ( mode >= SDL_GRAB_FULLSCREEN ) {
		mode -= SDL_GRAB_FULLSCREEN;
	}
	return(mode);
}
SDL_GrabMode SDL_WM_GrabInput(SDL_GrabMode mode)
{
	SDL_VideoDevice *video = current_video;

	/* If the video isn't initialized yet, we can't do anything */
	if ( ! video ) {
		return SDL_GRAB_OFF;
	}

	/* Return the current mode on query */
	if ( mode == SDL_GRAB_QUERY ) {
		mode = video->input_grab;
		if ( mode >= SDL_GRAB_FULLSCREEN ) {
			mode -= SDL_GRAB_FULLSCREEN;
		}
		return(mode);
	}

#ifdef DEBUG_GRAB
  printf("SDL_WM_GrabInput(%d) ... ", mode);
#endif
	/* If the video surface is fullscreen, we always grab */
	if ( mode >= SDL_GRAB_FULLSCREEN ) {
		mode -= SDL_GRAB_FULLSCREEN;
	}
	if ( SDL_VideoSurface && (SDL_VideoSurface->flags & SDL_FULLSCREEN) ) {
		mode += SDL_GRAB_FULLSCREEN;
	}
	return(SDL_WM_GrabInputRaw(mode));
}
static SDL_GrabMode SDL_WM_GrabInputOff(void)
{
	SDL_GrabMode mode;

	/* First query the current grab state */
	mode = SDL_WM_GrabInput(SDL_GRAB_QUERY);

	/* Now explicitly turn off input grab */
	SDL_WM_GrabInputRaw(SDL_GRAB_OFF);

	/* Return the old state */
	return(mode);
}

/*
 * Iconify the window in window managed environments.
 * A successful iconification will result in an SDL_APPACTIVE loss event.
 */
int SDL_WM_IconifyWindow(void)
{
	SDL_VideoDevice *video = current_video;
	SDL_VideoDevice *this  = current_video;
	int retval;

	retval = 0;
	if ( video->IconifyWindow ) {
		retval = video->IconifyWindow(this);
	}
	return(retval);
}

/*
 * Toggle fullscreen mode
 */
int SDL_WM_ToggleFullScreen(SDL_Surface *surface)
{
	SDL_VideoDevice *video = current_video;
	SDL_VideoDevice *this  = current_video;
	int toggled;

	toggled = 0;
	if ( SDL_PublicSurface && (surface == SDL_PublicSurface) &&
	     video->ToggleFullScreen ) {
		if ( surface->flags & SDL_FULLSCREEN ) {
			toggled = video->ToggleFullScreen(this, 0);
			if ( toggled ) {
				SDL_VideoSurface->flags &= ~SDL_FULLSCREEN;
				SDL_PublicSurface->flags &= ~SDL_FULLSCREEN;
			}
		} else {
			toggled = video->ToggleFullScreen(this, 1);
			if ( toggled ) {
				SDL_VideoSurface->flags |= SDL_FULLSCREEN;
				SDL_PublicSurface->flags |= SDL_FULLSCREEN;
			}
		}
		/* Double-check the grab state inside SDL_WM_GrabInput() */
		if ( toggled ) {
			SDL_WM_GrabInput(video->input_grab);
		}
	}
	return(toggled);
}

/*
 * Get some platform dependent window manager information
 */
int SDL_GetWMInfo (SDL_SysWMinfo *info)
{
	SDL_VideoDevice *video = current_video;
	SDL_VideoDevice *this  = current_video;

	if ( video && video->GetWMInfo ) {
		return(video->GetWMInfo(this, info));
	} else {
		return(0);
	}
}
