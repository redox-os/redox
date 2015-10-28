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

/* Refresh event handling code for SDL */

#include "SDL_events.h"
#include "SDL_events_c.h"


/* This is global for SDL_eventloop.c */
int SDL_PrivateExpose(void)
{
	int posted;
	SDL_Event events[32];

	/* Pull out all old refresh events */
	SDL_PeepEvents(events, sizeof(events)/sizeof(events[0]),
	                    SDL_GETEVENT, SDL_VIDEOEXPOSEMASK);

	/* Post the event, if desired */
	posted = 0;
	if ( SDL_ProcessEvents[SDL_VIDEOEXPOSE] == SDL_ENABLE ) {
		SDL_Event event;
		event.type = SDL_VIDEOEXPOSE;
		if ( (SDL_EventOK == NULL) || (*SDL_EventOK)(&event) ) {
			posted = 1;
			SDL_PushEvent(&event);
		}
	}
	return(posted);
}
