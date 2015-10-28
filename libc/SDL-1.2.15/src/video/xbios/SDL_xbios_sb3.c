/*
    SDL - Simple DirectMedia Layer
    Copyright (C) 1997-2012 Sam Lantinga

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

    Sam Lantinga
    slouken@libsdl.org
*/
#include "SDL_config.h"

/*
	ScreenBlaster 3 functions

	Patrice Mandin
*/

/*--- Includes ---*/

#include "SDL_stdinc.h"
#include "SDL_xbios.h"
#include "SDL_xbios_sb3.h"

/*--- Defines ---*/

const int SDL_XBIOS_scpn_planes_device[]={
	SCPN_DEV_1BPP,
	SCPN_DEV_4BPP,
	SCPN_DEV_8BPP,
	SCPN_DEV_16BPP,
	SCPN_DEV_2BPP,
	SCPN_DEV_4BPP,
	SCPN_DEV_1BPP
};

/*--- Functions ---*/

int SDL_XBIOS_SB3Usable(scpn_cookie_t *cookie_scpn)
{
	scpn_screeninfo_t *scrinfo;
	int bpp;

	/* Check if current SB3 mode is usable, i.e. 8 or 16bpp */
	scrinfo = cookie_scpn->screen_info;
	bpp = 1<<(SDL_XBIOS_scpn_planes_device[scrinfo->device]);

	if ((bpp==8) || (bpp==16)) {
		return 1;
	}

	return 0;
}

void SDL_XBIOS_ListSB3Modes(_THIS, int actually_add, scpn_cookie_t *cookie_scpn)
{
	scpn_screeninfo_t *scrinfo;
	xbiosmode_t modeinfo;

	scrinfo = cookie_scpn->screen_info;
	if (actually_add) {
		scrinfo->h_pos = scrinfo->v_pos = 0;
	}

	modeinfo.number = -1;
	modeinfo.width = scrinfo->virtual_width;
	modeinfo.height = scrinfo->virtual_height;
	modeinfo.depth = 1<<(SDL_XBIOS_scpn_planes_device[scrinfo->device]);
	modeinfo.flags = (modeinfo.depth == 8 ? XBIOSMODE_C2P : 0);

	SDL_XBIOS_AddMode(this, actually_add, &modeinfo);
}
