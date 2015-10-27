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

#ifndef _SDL_sysvideo_h
#define _SDL_sysvideo_h

#include "SDL_mouse.h"
#define SDL_PROTOTYPES_ONLY
#include "SDL_syswm.h"
#undef SDL_PROTOTYPES_ONLY

/* This file prototypes the video driver implementation.
   This is designed to be easily converted to C++ in the future.
 */

#if SDL_VIDEO_OPENGL
#include "SDL_opengl.h"
#endif /* SDL_VIDEO_OPENGL */

/* The SDL video driver */
typedef struct SDL_VideoDevice SDL_VideoDevice;

/* Define the SDL video driver structure */
#define _THIS	SDL_VideoDevice *_this
#ifndef _STATUS
#define _STATUS	SDL_status *status
#endif
struct SDL_VideoDevice {
	/* * * */
	/* The name of this video driver */
	const char *name;

	/* * * */
	/* Initialization/Query functions */

	/* Initialize the native video subsystem, filling 'vformat' with the 
	   "best" display pixel format, returning 0 or -1 if there's an error.
	 */
	int (*VideoInit)(_THIS, SDL_PixelFormat *vformat);

	/* List the available video modes for the given pixel format, sorted
	   from largest to smallest.
	 */
	SDL_Rect **(*ListModes)(_THIS, SDL_PixelFormat *format, Uint32 flags);

	/* Set the requested video mode, returning a surface which will be
	   set to the SDL_VideoSurface.  The width and height will already
	   be verified by ListModes(), and the video subsystem is free to
	   set the mode to a supported bit depth different from the one
	   specified -- the desired bpp will be emulated with a shadow
	   surface if necessary.  If a new mode is returned, this function
	   should take care of cleaning up the current mode.
	 */
	SDL_Surface *(*SetVideoMode)(_THIS, SDL_Surface *current,
				int width, int height, int bpp, Uint32 flags);

	/* Toggle the fullscreen mode */
	int (*ToggleFullScreen)(_THIS, int on);

	/* This is called after the video mode has been set, to get the
	   initial mouse state.  It should queue events as necessary to
	   properly represent the current mouse focus and position.
	 */
	void (*UpdateMouse)(_THIS);

	/* Create a YUV video surface (possibly overlay) of the given
	   format.  The hardware should be able to perform at least 2x
	   scaling on display.
	 */
	SDL_Overlay *(*CreateYUVOverlay)(_THIS, int width, int height,
	                                 Uint32 format, SDL_Surface *display);

        /* Sets the color entries { firstcolor .. (firstcolor+ncolors-1) }
	   of the physical palette to those in 'colors'. If the device is
	   using a software palette (SDL_HWPALETTE not set), then the
	   changes are reflected in the logical palette of the screen
	   as well.
	   The return value is 1 if all entries could be set properly
	   or 0 otherwise.
	*/
	int (*SetColors)(_THIS, int firstcolor, int ncolors,
			 SDL_Color *colors);

	/* This pointer should exist in the native video subsystem and should
	   point to an appropriate update function for the current video mode
	 */
	void (*UpdateRects)(_THIS, int numrects, SDL_Rect *rects);

	/* Reverse the effects VideoInit() -- called if VideoInit() fails
	   or if the application is shutting down the video subsystem.
	*/
	void (*VideoQuit)(_THIS);

	/* * * */
	/* Hardware acceleration functions */

	/* Information about the video hardware */
	SDL_VideoInfo info;

	/* The pixel format used when SDL_CreateRGBSurface creates SDL_HWSURFACEs with alpha */
	SDL_PixelFormat* displayformatalphapixel;
	
	/* Allocates a surface in video memory */
	int (*AllocHWSurface)(_THIS, SDL_Surface *surface);

	/* Sets the hardware accelerated blit function, if any, based
	   on the current flags of the surface (colorkey, alpha, etc.)
	 */
	int (*CheckHWBlit)(_THIS, SDL_Surface *src, SDL_Surface *dst);

	/* Fills a surface rectangle with the given color */
	int (*FillHWRect)(_THIS, SDL_Surface *dst, SDL_Rect *rect, Uint32 color);

	/* Sets video mem colorkey and accelerated blit function */
	int (*SetHWColorKey)(_THIS, SDL_Surface *surface, Uint32 key);

	/* Sets per surface hardware alpha value */
	int (*SetHWAlpha)(_THIS, SDL_Surface *surface, Uint8 value);

	/* Returns a readable/writable surface */
	int (*LockHWSurface)(_THIS, SDL_Surface *surface);
	void (*UnlockHWSurface)(_THIS, SDL_Surface *surface);

	/* Performs hardware flipping */
	int (*FlipHWSurface)(_THIS, SDL_Surface *surface);

	/* Frees a previously allocated video surface */
	void (*FreeHWSurface)(_THIS, SDL_Surface *surface);

	/* * * */
	/* Gamma support */

	Uint16 *gamma;

	/* Set the gamma correction directly (emulated with gamma ramps) */
	int (*SetGamma)(_THIS, float red, float green, float blue);

	/* Get the gamma correction directly (emulated with gamma ramps) */
	int (*GetGamma)(_THIS, float *red, float *green, float *blue);

	/* Set the gamma ramp */
	int (*SetGammaRamp)(_THIS, Uint16 *ramp);

	/* Get the gamma ramp */
	int (*GetGammaRamp)(_THIS, Uint16 *ramp);

	/* * * */
	/* OpenGL support */

	/* Sets the dll to use for OpenGL and loads it */
	int (*GL_LoadLibrary)(_THIS, const char *path);

	/* Retrieves the address of a function in the gl library */
	void* (*GL_GetProcAddress)(_THIS, const char *proc);

	/* Get attribute information from the windowing system. */
	int (*GL_GetAttribute)(_THIS, SDL_GLattr attrib, int* value);

	/* Make the context associated with this driver current */
	int (*GL_MakeCurrent)(_THIS);

	/* Swap the current buffers in double buffer mode. */
	void (*GL_SwapBuffers)(_THIS);

  	/* OpenGL functions for SDL_OPENGLBLIT */
#if SDL_VIDEO_OPENGL
#if !defined(__WIN32__)
#define WINAPI
#endif
#define SDL_PROC(ret,func,params) ret (WINAPI *func) params;
#include "SDL_glfuncs.h"
#undef SDL_PROC

	/* Texture id */
	GLuint texture;
#endif
	int is_32bit;
 
	/* * * */
	/* Window manager functions */

	/* Set the title and icon text */
	void (*SetCaption)(_THIS, const char *title, const char *icon);

	/* Set the window icon image */
	void (*SetIcon)(_THIS, SDL_Surface *icon, Uint8 *mask);

	/* Iconify the window.
	   This function returns 1 if there is a window manager and the
	   window was actually iconified, it returns 0 otherwise.
	*/
	int (*IconifyWindow)(_THIS);

	/* Grab or ungrab keyboard and mouse input */
	SDL_GrabMode (*GrabInput)(_THIS, SDL_GrabMode mode);

	/* Get some platform dependent window information */
	int (*GetWMInfo)(_THIS, SDL_SysWMinfo *info);

	/* * * */
	/* Cursor manager functions */

	/* Free a window manager cursor
	   This function can be NULL if CreateWMCursor is also NULL.
	 */
	void (*FreeWMCursor)(_THIS, WMcursor *cursor);

	/* If not NULL, create a black/white window manager cursor */
	WMcursor *(*CreateWMCursor)(_THIS,
		Uint8 *data, Uint8 *mask, int w, int h, int hot_x, int hot_y);

	/* Show the specified cursor, or hide if cursor is NULL */
	int (*ShowWMCursor)(_THIS, WMcursor *cursor);

	/* Warp the window manager cursor to (x,y)
	   If NULL, a mouse motion event is posted internally.
	 */
	void (*WarpWMCursor)(_THIS, Uint16 x, Uint16 y);

	/* If not NULL, this is called when a mouse motion event occurs */
	void (*MoveWMCursor)(_THIS, int x, int y);

	/* Determine whether the mouse should be in relative mode or not.
	   This function is called when the input grab state or cursor
	   visibility state changes.
	   If the cursor is not visible, and the input is grabbed, the
	   driver can place the mouse in relative mode, which may result
	   in higher accuracy sampling of the pointer motion.
	*/
	void (*CheckMouseMode)(_THIS);

	/* * * */
	/* Event manager functions */

	/* Initialize keyboard mapping for this driver */
	void (*InitOSKeymap)(_THIS);

	/* Handle any queued OS events */
	void (*PumpEvents)(_THIS);

	/* * * */
	/* Data common to all drivers */
	SDL_Surface *screen;
	SDL_Surface *shadow;
	SDL_Surface *visible;
        SDL_Palette *physpal;	/* physical palette, if != logical palette */
        SDL_Color *gammacols;	/* gamma-corrected colours, or NULL */
	char *wm_title;
	char *wm_icon;
	int offset_x;
	int offset_y;
	SDL_GrabMode input_grab;

	/* Driver information flags */
	int handles_any_size;	/* Driver handles any size video mode */

	/* * * */
	/* Data used by the GL drivers */
	struct {
		int red_size;
		int green_size;
		int blue_size;
		int alpha_size;
		int depth_size;
		int buffer_size;
		int stencil_size;
		int double_buffer;
		int accum_red_size;
		int accum_green_size;
		int accum_blue_size;
		int accum_alpha_size;
		int stereo;
		int multisamplebuffers;
		int multisamplesamples;
		int accelerated;
		int swap_control;
		int driver_loaded;
		char driver_path[256];
		void* dll_handle;
	} gl_config;

	/* * * */
	/* Data private to this driver */
	struct SDL_PrivateVideoData *hidden;
	struct SDL_PrivateGLData *gl_data;

	/* * * */
	/* The function used to dispose of this structure */
	void (*free)(_THIS);
};
#undef _THIS

typedef struct VideoBootStrap {
	const char *name;
	const char *desc;
	int (*available)(void);
	SDL_VideoDevice *(*create)(int devindex);
} VideoBootStrap;

#if SDL_VIDEO_DRIVER_QUARTZ
extern VideoBootStrap QZ_bootstrap;
#endif
#if SDL_VIDEO_DRIVER_X11
extern VideoBootStrap X11_bootstrap;
#endif
#if SDL_VIDEO_DRIVER_DGA
extern VideoBootStrap DGA_bootstrap;
#endif
#if SDL_VIDEO_DRIVER_NANOX
extern VideoBootStrap NX_bootstrap;
#endif
#if SDL_VIDEO_DRIVER_IPOD
extern VideoBootStrap iPod_bootstrap;
#endif
#if SDL_VIDEO_DRIVER_QTOPIA
extern VideoBootStrap Qtopia_bootstrap;
#endif
#if SDL_VIDEO_DRIVER_WSCONS
extern VideoBootStrap WSCONS_bootstrap;
#endif
#if SDL_VIDEO_DRIVER_FBCON
extern VideoBootStrap FBCON_bootstrap;
#endif
#if SDL_VIDEO_DRIVER_DIRECTFB
extern VideoBootStrap DirectFB_bootstrap;
#endif
#if SDL_VIDEO_DRIVER_PS2GS
extern VideoBootStrap PS2GS_bootstrap;
#endif
#if SDL_VIDEO_DRIVER_PS3
extern VideoBootStrap PS3_bootstrap;
#endif
#if SDL_VIDEO_DRIVER_GGI
extern VideoBootStrap GGI_bootstrap;
#endif
#if SDL_VIDEO_DRIVER_VGL
extern VideoBootStrap VGL_bootstrap;
#endif
#if SDL_VIDEO_DRIVER_SVGALIB
extern VideoBootStrap SVGALIB_bootstrap;
#endif
#if SDL_VIDEO_DRIVER_GAPI
extern VideoBootStrap GAPI_bootstrap;
#endif
#if SDL_VIDEO_DRIVER_WINDIB
extern VideoBootStrap WINDIB_bootstrap;
#endif
#if SDL_VIDEO_DRIVER_DDRAW
extern VideoBootStrap DIRECTX_bootstrap;
#endif
#if SDL_VIDEO_DRIVER_BWINDOW
extern VideoBootStrap BWINDOW_bootstrap;
#endif
#if SDL_VIDEO_DRIVER_TOOLBOX
extern VideoBootStrap TOOLBOX_bootstrap;
#endif
#if SDL_VIDEO_DRIVER_DRAWSPROCKET
extern VideoBootStrap DSp_bootstrap;
#endif
#if SDL_VIDEO_DRIVER_PHOTON
extern VideoBootStrap ph_bootstrap;
#endif
#if SDL_VIDEO_DRIVER_EPOC
extern VideoBootStrap EPOC_bootstrap;
#endif
#if SDL_VIDEO_DRIVER_XBIOS
extern VideoBootStrap XBIOS_bootstrap;
#endif
#if SDL_VIDEO_DRIVER_GEM
extern VideoBootStrap GEM_bootstrap;
#endif
#if SDL_VIDEO_DRIVER_PICOGUI
extern VideoBootStrap PG_bootstrap;
#endif
#if SDL_VIDEO_DRIVER_DC
extern VideoBootStrap DC_bootstrap;
#endif
#if SDL_VIDEO_DRIVER_NDS
extern VideoBootStrap NDS_bootstrap;
#endif
#if SDL_VIDEO_DRIVER_RISCOS
extern VideoBootStrap RISCOS_bootstrap;
#endif
#if SDL_VIDEO_DRIVER_OS2FS
extern VideoBootStrap OS2FSLib_bootstrap;
#endif
#if SDL_VIDEO_DRIVER_AALIB
extern VideoBootStrap AALIB_bootstrap;
#endif
#if SDL_VIDEO_DRIVER_CACA
extern VideoBootStrap CACA_bootstrap;
#endif
#if SDL_VIDEO_DRIVER_DUMMY
extern VideoBootStrap DUMMY_bootstrap;
#endif

/* This is the current video device */
extern SDL_VideoDevice *current_video;

#define SDL_VideoSurface	(current_video->screen)
#define SDL_ShadowSurface	(current_video->shadow)
#define SDL_PublicSurface	(current_video->visible)

#endif /* _SDL_sysvideo_h */
