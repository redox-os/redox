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

/*
	GEM SDL video driver
	Window manager functions

	Patrice Mandin
*/

/* Mint includes */
#include <gem.h>

#include "SDL_gemwm_c.h"

/* Defines */

#define ICONWIDTH 64
#define ICONHEIGHT 64

/* Functions */

void GEM_SetCaption(_THIS, const char *title, const char *icon)
{
	if (title) {
		GEM_title_name = title;
		GEM_refresh_name = SDL_TRUE;
	}

	if (icon) {
		GEM_icon_name = icon;
		GEM_refresh_name = SDL_TRUE;
	}
}

void GEM_SetIcon(_THIS, SDL_Surface *icon, Uint8 *mask)
{
	SDL_Surface *sicon;
	SDL_Rect bounds;

#if 0
	if ((GEM_wfeatures & (1<<WF_ICONIFY))==0) {
		return;
	}
#endif

	if (icon == NULL) {
		return;
	}
	
	/* Convert icon to the screen format */
	sicon = SDL_CreateRGBSurface(SDL_SWSURFACE, icon->w, icon->h,
		VDI_bpp, VDI_redmask, VDI_greenmask, VDI_bluemask, 0);
	if ( sicon == NULL ) {
		return;
	}

	bounds.x = 0;
	bounds.y = 0;
	bounds.w = icon->w;
	bounds.h = icon->h;
	if ( SDL_LowerBlit(icon, &bounds, sicon, &bounds) < 0 ) {
		SDL_FreeSurface(sicon);
		return;
	}

	GEM_icon = sicon;
}

int GEM_IconifyWindow(_THIS)
{
	if ((GEM_wfeatures & (1<<WF_ICONIFY))==0)
		return 0;

	GEM_message[0] = WM_ICONIFY;
	GEM_message[1] = gl_apid;
	GEM_message[2] = 0;
	GEM_message[3] = GEM_handle;
	GEM_message[4] = 0;
	GEM_message[5] = GEM_desk_h-ICONHEIGHT;
	GEM_message[6] = ICONWIDTH;
	GEM_message[7] = ICONHEIGHT;

	appl_write(gl_apid, sizeof(GEM_message), GEM_message);

	return 1;
}

SDL_GrabMode GEM_GrabInput(_THIS, SDL_GrabMode mode)
{
	if (this->screen == NULL) {
		return SDL_GRAB_OFF;
	}

	return mode;
}
