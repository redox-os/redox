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

/* Application focus/iconification handling code for SDL */

#include "SDL_events.h"
#include "SDL_events_c.h"


/* These are static for our active event handling code */
static Uint8 SDL_appstate = 0;

/* Public functions */
int SDL_AppActiveInit(void)
{
	/* Start completely active */
	SDL_appstate = (SDL_APPACTIVE|SDL_APPINPUTFOCUS|SDL_APPMOUSEFOCUS);

	/* That's it! */
	return(0);
}
void SDL_AppActiveQuit(void)
{
}

Uint8 SDL_GetAppState(void)
{
	return(SDL_appstate);
}

/* This is global for SDL_eventloop.c */
int SDL_PrivateAppActive(Uint8 gain, Uint8 state)
{
	int posted;
	Uint8 new_state;

	/* Modify the current state with the given mask */
	if ( gain ) {
		new_state = (SDL_appstate | state);
	} else {
		new_state = (SDL_appstate & ~state);
	}

	/* Drop events that don't change state */
	if ( new_state == SDL_appstate ) {
		return(0);
	}

	/* Update internal active state */
	SDL_appstate = new_state;

	/* Post the event, if desired */
	posted = 0;
	if ( SDL_ProcessEvents[SDL_ACTIVEEVENT] == SDL_ENABLE ) {
		SDL_Event event;
		SDL_memset(&event, 0, sizeof(event));
		event.type = SDL_ACTIVEEVENT;
		event.active.gain = gain;
		event.active.state = state;
		if ( (SDL_EventOK == NULL) || (*SDL_EventOK)(&event) ) {
			posted = 1;
			SDL_PushEvent(&event);
		}
	}

	/* If we lost keyboard focus, post key-up events */
	if ( (state & SDL_APPINPUTFOCUS) && !gain ) {
		SDL_ResetKeyboard();
	}
	/* If we were minimized, post button-up events */
	if ( (state & SDL_APPACTIVE) && !gain ) {
		SDL_ResetMouse();
	}
	return(posted);
}
