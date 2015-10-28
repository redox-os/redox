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

#ifndef _SDL_x11video_h
#define _SDL_x11video_h

#include <X11/Xlib.h>
#include <X11/Xutil.h>
#include <X11/Xatom.h>

#include "SDL_mouse.h"
#include "../SDL_sysvideo.h"

#if SDL_VIDEO_DRIVER_X11_DGAMOUSE
#include "../Xext/extensions/xf86dga.h"
#endif
#if SDL_VIDEO_DRIVER_X11_XINERAMA
#include "../Xext/extensions/Xinerama.h"
#endif 
#if SDL_VIDEO_DRIVER_X11_XRANDR
#include <X11/extensions/Xrandr.h>
#endif
#if SDL_VIDEO_DRIVER_X11_VIDMODE
#include "../Xext/extensions/xf86vmode.h"
#endif
#if SDL_VIDEO_DRIVER_X11_XME
#include "../Xext/extensions/xme.h"
#endif

#include "SDL_x11dyn.h"

/* Hidden "this" pointer for the video functions */
#define _THIS	SDL_VideoDevice *this

/* Private display data */
struct SDL_PrivateVideoData {
    int local_X11;		/* Flag: true if local display */
    Display *X11_Display;	/* Used for events and window management */
    Display *GFX_Display;	/* Used for graphics and colormap stuff */
    Visual *SDL_Visual;		/* The visual used by our window */
    Window WMwindow;		/* Input window, managed by window manager */
    Window FSwindow;		/* Fullscreen window, completely unmanaged */
    Window SDL_Window;		/* Shared by both displays (no X security?) */
    Atom WM_DELETE_WINDOW;	/* "close-window" protocol atom */
    WMcursor *BlankCursor;	/* The invisible cursor */
    XIM X11_IM;		/* Used to communicate with the input method (IM) server */
    XIC X11_IC;		/* Used for retaining the state, properties, and semantics of communication with                                                  the input method (IM) server */

    char *SDL_windowid;		/* Flag: true if we have been passed a window */

    /* Direct Graphics Access extension information */
    int using_dga;

#ifndef NO_SHARED_MEMORY
    /* MIT shared memory extension information */
    int use_mitshm;
    XShmSegmentInfo shminfo;
#endif

    /* The variables used for displaying graphics */
    XImage *Ximage;		/* The X image for our window */
    GC	gc;			/* The graphic context for drawing */

    /* The current width and height of the fullscreen mode */
    int window_w;
    int window_h;

    /* Support for internal mouse warping */
    struct {
        int x;
        int y;
    } mouse_last;
    struct {
        int numerator;
        int denominator;
        int threshold;
    } mouse_accel;
    int mouse_relative;

    /* The current list of available video modes */
    SDL_Rect **modelist;

    /* available visuals of interest to us, sorted deepest first */
    struct {
	Visual *visual;
	int depth;		/* number of significant bits/pixel */
	int bpp;		/* pixel quantum in bits */
    } visuals[2*5];		/* at most 2 entries for 8, 15, 16, 24, 32 */
    int nvisuals;

    Visual *vis;		/* current visual in use */
    int depth;			/* current visual depth (not bpp) */

    /* Variables used by the X11 video mode code */
#if SDL_VIDEO_DRIVER_X11_XINERAMA
    SDL_NAME(XineramaScreenInfo) xinerama_info;
#endif
#if SDL_VIDEO_DRIVER_X11_XRANDR
    XRRScreenConfiguration* screen_config;
    int saved_size_id;
    Rotation saved_rotation;
#endif
#if SDL_VIDEO_DRIVER_X11_VIDMODE
    SDL_NAME(XF86VidModeModeInfo) saved_mode;
    struct {
        int x, y;
    } saved_view;
#endif
#if SDL_VIDEO_DRIVER_X11_XME /* XiG XME fullscreen */
    XiGMiscResolutionInfo saved_res;
#endif

    int use_xinerama;
    int use_xrandr;
    int use_vidmode;
    int use_xme;
    int currently_fullscreen;

    /* Automatic mode switching support (entering/leaving fullscreen) */
    Uint32 switch_waiting;
    Uint32 switch_time;

    /* Prevent too many XSync() calls */
    int blit_queued;

    /* Colormap handling */
    Colormap DisplayColormap;	/* The default display colormap */
    Colormap XColorMap;		/* The current window colormap */
    int *XPixels;		/* pixels value allocation counts */
    float gamma_saved[3];	/* Saved gamma values for VidMode gamma */
    int gamma_changed;		/* flag: has VidMode gamma been modified? */

    short *iconcolors;		/* List of colors used by the icon */

    /* Screensaver settings */
    int allow_screensaver;
};

/* Old variable names */
#define local_X11		(this->hidden->local_X11)
#define SDL_Display		(this->hidden->X11_Display)
#define GFX_Display		(this->hidden->GFX_Display)
#define SDL_Screen		DefaultScreen(this->hidden->X11_Display)
#define SDL_Visual		(this->hidden->vis)
#define SDL_Root		RootWindow(SDL_Display, SDL_Screen)
#define WMwindow		(this->hidden->WMwindow)
#define FSwindow		(this->hidden->FSwindow)
#define SDL_Window		(this->hidden->SDL_Window)
#define WM_DELETE_WINDOW	(this->hidden->WM_DELETE_WINDOW)
#define SDL_BlankCursor		(this->hidden->BlankCursor)
#define SDL_IM			(this->hidden->X11_IM)
#define SDL_IC			(this->hidden->X11_IC)
#define SDL_windowid		(this->hidden->SDL_windowid)
#define using_dga		(this->hidden->using_dga)
#define use_mitshm		(this->hidden->use_mitshm)
#define shminfo			(this->hidden->shminfo)
#define SDL_Ximage		(this->hidden->Ximage)
#define SDL_GC			(this->hidden->gc)
#define window_w		(this->hidden->window_w)
#define window_h		(this->hidden->window_h)
#define mouse_last		(this->hidden->mouse_last)
#define mouse_accel		(this->hidden->mouse_accel)
#define mouse_relative		(this->hidden->mouse_relative)
#define SDL_modelist		(this->hidden->modelist)
#define xinerama_info		(this->hidden->xinerama_info)
#define saved_mode		(this->hidden->saved_mode)
#define saved_view		(this->hidden->saved_view)
#define saved_res		(this->hidden->saved_res)
#define screen_config		(this->hidden->screen_config)
#define saved_size_id		(this->hidden->saved_size_id)
#define saved_rotation		(this->hidden->saved_rotation)
#define use_xinerama		(this->hidden->use_xinerama)
#define use_vidmode		(this->hidden->use_vidmode)
#define use_xrandr		(this->hidden->use_xrandr)
#define use_xme			(this->hidden->use_xme)
#define currently_fullscreen	(this->hidden->currently_fullscreen)
#define switch_waiting		(this->hidden->switch_waiting)
#define switch_time		(this->hidden->switch_time)
#define blit_queued		(this->hidden->blit_queued)
#define SDL_DisplayColormap	(this->hidden->DisplayColormap)
#define SDL_PrivateColormap	(this->hidden->PrivateColormap)
#define SDL_XColorMap		(this->hidden->XColorMap)
#define SDL_XPixels		(this->hidden->XPixels)
#define gamma_saved		(this->hidden->gamma_saved)
#define gamma_changed		(this->hidden->gamma_changed)
#define SDL_iconcolors		(this->hidden->iconcolors)
#define allow_screensaver	(this->hidden->allow_screensaver)

/* Some versions of XFree86 have bugs - detect if this is one of them */
#define BUGGY_XFREE86(condition, buggy_version) \
((SDL_strcmp(ServerVendor(SDL_Display), "The XFree86 Project, Inc") == 0) && \
 (VendorRelease(SDL_Display) condition buggy_version))

#endif /* _SDL_x11video_h */
