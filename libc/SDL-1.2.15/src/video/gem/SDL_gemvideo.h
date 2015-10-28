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

#ifndef _SDL_gemvideo_h
#define _SDL_gemvideo_h

#include "SDL_mutex.h"
#include "../SDL_sysvideo.h"

/* The implementation dependent data for the window manager cursor */
struct WMcursor {
	MFORM *mform_p;
};

/* Hidden "this" pointer for the video functions */
#define _THIS	SDL_VideoDevice *this

/* Functions prototypes */
void GEM_wind_redraw(_THIS, int winhandle, short *inside);

/* Private display data */

#define B2S_C2P_1TO2		(1<<0)	/* C2P convert buffer 1 to buffer 2 */
#define B2S_C2P_1TOS		(1<<1)	/* C2P convert buffer 1 to screen */
#define B2S_VROCPYFM_1TOS	(1<<2)	/* vro_cpyfm() buffer 1 to screen */
#define B2S_VROCPYFM_2TOS	(1<<3)	/* vro_cpyfm() buffer 2 to screen */

#define SDL_NUMMODES	1		/* Fullscreen */

struct SDL_PrivateVideoData {
	Uint16	buf2scr_ops;		/* Operations to get buffer to screen */
	void *buffer1;				/* Our shadow buffers */
	void *buffer2;

	/* VDI infos */
	short vdi_handle;			/* VDI handle */
	short full_w, full_h;		/* Fullscreen size */
	short bpp;					/* Colour depth */
	short pixelsize;			/* Bytes per pixel */
	short old_numcolors;		/* Number of colors in saved palette */
	Uint16 pitch;				/* Line length */
	Uint16 format;				/* Screen format */
	void *screen;				/* Screen address */
	Uint32 red, green, blue, alpha;	/* Screen components */
	Uint32 screensize;
	short	blit_coords[8];		/* Coordinates for bitblt */
	MFDB	src_mfdb, dst_mfdb;	/* VDI MFDB for bitblt */
	Uint16 old_palette[256][3];	/* Saved current palette */
	Uint16 cur_palette[256][3];	/* SDL application palette */
								/* Function to set/restore palette */
	void (*setpalette)(_THIS, Uint16 newpal[256][3]);

	/* GEM infos */
	short desk_x, desk_y;		/* Desktop properties */
	short desk_w, desk_h;
	short win_handle;			/* Our window handle */
	int window_type;			/* Window type */
	const char *title_name;		/* Window title */
	const char *icon_name;		/* Icon title */
	short version;				/* AES version */
	short wfeatures;			/* AES window features */
	SDL_bool refresh_name;		/* Change window title ? */
	SDL_bool window_fulled;		/* Window maximized ? */
	SDL_bool mouse_relative;	/* Report relative mouse movement */
	SDL_bool locked;			/* AES locked for fullscreen ? */
	SDL_bool lock_redraw;		/* Prevent redraw till buffers are setup */
	short message[8];			/* To self-send an AES message */
	void *menubar;				/* Menu bar save buffer when going fullscreen */
	SDL_bool use_dev_mouse;		/* Use /dev/mouse ? */
	WMcursor *cursor;			/* To restore cursor when leaving/entering window */

	SDL_bool fullscreen;		/* Fullscreen or windowed mode ? */
	SDL_Rect *SDL_modelist[SDL_NUMMODES+1];	/* Mode list */
	SDL_Surface *icon;			/* The icon */
};

/* Hidden structure -> variables names */
#define VDI_handle			(this->hidden->vdi_handle)
#define VDI_w				(this->hidden->full_w)
#define VDI_h				(this->hidden->full_h)
#define VDI_bpp				(this->hidden->bpp)
#define VDI_pixelsize		(this->hidden->pixelsize)
#define VDI_oldnumcolors	(this->hidden->old_numcolors)
#define VDI_oldpalette		(this->hidden->old_palette)
#define VDI_curpalette		(this->hidden->cur_palette)
#define VDI_setpalette		(this->hidden->setpalette)
#define VDI_pitch			(this->hidden->pitch)
#define VDI_format			(this->hidden->format)
#define VDI_screen			(this->hidden->screen)
#define VDI_redmask			(this->hidden->red)
#define VDI_greenmask		(this->hidden->green)
#define VDI_bluemask		(this->hidden->blue)
#define VDI_alphamask		(this->hidden->alpha)
#define VDI_screensize		(this->hidden->screensize)
#define VDI_src_mfdb		(this->hidden->src_mfdb)
#define VDI_dst_mfdb		(this->hidden->dst_mfdb)
#define VDI_blit_coords		(this->hidden->blit_coords)

#define GEM_desk_x			(this->hidden->desk_x)
#define GEM_desk_y			(this->hidden->desk_y)
#define GEM_desk_w			(this->hidden->desk_w)
#define GEM_desk_h			(this->hidden->desk_h)
#define GEM_handle			(this->hidden->win_handle)
#define GEM_win_type		(this->hidden->window_type)
#define GEM_title_name		(this->hidden->title_name)
#define GEM_icon_name		(this->hidden->icon_name)
#define GEM_refresh_name	(this->hidden->refresh_name)
#define GEM_version			(this->hidden->version)
#define GEM_wfeatures		(this->hidden->wfeatures)
#define GEM_win_fulled		(this->hidden->window_fulled)
#define GEM_mouse_relative	(this->hidden->mouse_relative)
#define GEM_locked			(this->hidden->locked)
#define GEM_lock_redraw		(this->hidden->lock_redraw)
#define GEM_message			(this->hidden->message)
#define SDL_modelist		(this->hidden->SDL_modelist)
#define GEM_icon			(this->hidden->icon)
#define GEM_fullscreen		(this->hidden->fullscreen)
#define GEM_menubar			(this->hidden->menubar)
#define GEM_usedevmouse		(this->hidden->use_dev_mouse)
#define GEM_cursor			(this->hidden->cursor)

#define GEM_buffer1			(this->hidden->buffer1)
#define GEM_buffer2			(this->hidden->buffer2)
#define GEM_bufops			(this->hidden->buf2scr_ops)

#define VDI_FBMASK(amask, rmask, gmask, bmask) \
	VDI_alphamask = (amask); \
	VDI_redmask = (rmask); \
	VDI_greenmask = (gmask); \
	VDI_bluemask = (bmask);

/*
	Possible buffer to screen operations:

	TC: 8 (chunky),15,16,24,32 bpp
	8I: 8 bpp planes
	FB: screen framebuffer address available
	FS: fullscreen

	TC, FB, FS:
		- draw to screen
	8I, FB, FS:
		- draw to buffer 1
		- C2P from buffer 1 to screen

	TC, !FB, FS:
		- draw to buffer 1
		- vro_cpyfm() from buffer 1 to screen
	8I, !FB, FS:
		- draw to buffer 1
		- C2P from buffer 1 to buffer 2
		- vro_cpyfm() from buffer 2 to screen

	TC, FB, !FS:
		- draw to buffer 1
		- vro_cpyfm() from buffer 1 to screen
	8I, FB, !FS:
		- draw to buffer 1
		- C2P from buffer 1 to buffer 2
		- vro_cpyfm() from buffer 2 to screen

	TC, !FB, !FS:
		- draw to buffer 1
		- vro_cpyfm() from buffer 1 to screen
	8I, !FB, !FS:
		- draw to buffer 1
		- C2P from buffer 1 to buffer 2
		- vro_cpyfm() from buffer 2 to screen
*/

#endif /* _SDL_gemvideo_h */
