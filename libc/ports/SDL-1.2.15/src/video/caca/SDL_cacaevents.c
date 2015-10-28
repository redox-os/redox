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

#ifdef SAVE_RCSID
static char rcsid =
 "@(#) $Id: libsdl-1.2.11-libcaca.patch,v 1.1 2006/09/18 16:06:06 mr_bones_ Exp $";
#endif

#include <stdio.h>

#include <caca.h>
#ifdef CACA_API_VERSION_1
#include <caca0.h>
#endif

#include "SDL.h"
#include "../../events/SDL_sysevents.h"
#include "../../events/SDL_events_c.h"
#include "SDL_cacavideo.h"
#include "SDL_cacaevents_c.h"

void Caca_PumpEvents(_THIS)
{
	int posted = 0;
	int event;
	SDL_keysym keysym;

	if( ! this->screen ) /* Wait till we got the screen initialised */
	  return;

	do {
		posted = 0;

		/* Get libcaca event */
		SDL_mutexP(Caca_mutex);
		event = caca_get_event(CACA_EVENT_ANY);
		SDL_mutexV(Caca_mutex);

		if ( event & (CACA_EVENT_KEY_PRESS | CACA_EVENT_KEY_RELEASE)) {
			int key;
			switch ( event & 0xffffff )
			{
				case CACA_KEY_LEFT: key = SDLK_LEFT; break;
				case CACA_KEY_RIGHT: key = SDLK_RIGHT; break;
				case CACA_KEY_UP: key = SDLK_UP; break;
				case CACA_KEY_DOWN: key = SDLK_DOWN; break;
				default: key = event & 0xff; break;
			}
			/* Key pressed */
/*    		printf("Key pressed: %d (%c)\n", key, key); */
			keysym.scancode = key;
			keysym.sym = key;
			keysym.mod = KMOD_NONE;
			keysym.unicode = 0;
			if ( SDL_TranslateUNICODE ) {
				keysym.unicode = key;
			}
			posted += SDL_PrivateKeyboard((event & CACA_EVENT_KEY_PRESS) ? SDL_PRESSED : SDL_RELEASED, &keysym);
		}
		else if ( event & (CACA_EVENT_MOUSE_PRESS | CACA_EVENT_MOUSE_RELEASE) ) {
			/* FIXME: we currently ignore the button type! */
			int button = event & 0x00ffffff;
			if ( button > 3 ) {
				button = 1;
			}
			posted += SDL_PrivateMouseButton((event & CACA_EVENT_MOUSE_PRESS) ? SDL_PRESSED : SDL_RELEASED, button, 0, 0);
		}
		else if ( event & CACA_EVENT_MOUSE_MOTION ) {
			int new_x = 0, new_y = 0;
			new_x = ((event & 0x00fff000) >> 12) * Caca_w / caca_get_width();
			new_y = ((event & 0x00000fff) >> 0) * Caca_h / caca_get_height();
			posted += SDL_PrivateMouseMotion(0, 0, new_x, new_y);
		}
	} while ( posted );
}

void Caca_InitOSKeymap(_THIS)
{
    return;
}


