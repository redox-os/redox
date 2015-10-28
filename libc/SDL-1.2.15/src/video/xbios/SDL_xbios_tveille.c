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
	Turbo veille screensaver

	Patrice Mandin
*/

#include <mint/cookie.h>

#include "SDL_xbios.h"
#include "SDL_xbios_tveille.h"

static tveille_t *cookie_veil = NULL;
static int status;

int SDL_XBIOS_TveillePresent(_THIS)
{
	long dummy;

	cookie_veil = NULL;
	if (Getcookie(C_VeiL, &dummy) == C_FOUND) {
		cookie_veil = (tveille_t *) dummy;
	}

	return (cookie_veil != NULL);
}

void SDL_XBIOS_TveilleDisable(_THIS)
{
	if (cookie_veil) {
		status = cookie_veil->enabled;
		cookie_veil->enabled = 0xff;
	}
}

void SDL_XBIOS_TveilleEnable(_THIS)
{
	if (cookie_veil) {
		cookie_veil->enabled = status;
	}
}
