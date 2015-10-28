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

/* General mouse handling code for SDL */

#include "SDL_events.h"
#include "SDL_events_c.h"
#include "../video/SDL_cursor_c.h"
#include "../video/SDL_sysvideo.h"


/* These are static for our mouse handling code */
static Sint16 SDL_MouseX = 0;
static Sint16 SDL_MouseY = 0;
static Sint16 SDL_DeltaX = 0;
static Sint16 SDL_DeltaY = 0;
static Sint16 SDL_MouseMaxX = 0;
static Sint16 SDL_MouseMaxY = 0;
static Uint8  SDL_ButtonState = 0;


/* Public functions */
int SDL_MouseInit(void)
{
	/* The mouse is at (0,0) */
	SDL_MouseX = 0;
	SDL_MouseY = 0;
	SDL_DeltaX = 0;
	SDL_DeltaY = 0;
	SDL_MouseMaxX = 0;
	SDL_MouseMaxY = 0;
	SDL_ButtonState = 0;

	/* That's it! */
	return(0);
}
void SDL_MouseQuit(void)
{
}

/* We lost the mouse, so post button up messages for all pressed buttons */
void SDL_ResetMouse(void)
{
	Uint8 i;
	for ( i = 0; i < sizeof(SDL_ButtonState)*8; ++i ) {
		if ( SDL_ButtonState & SDL_BUTTON(i) ) {
			SDL_PrivateMouseButton(SDL_RELEASED, i, 0, 0);
		}
	}
}

Uint8 SDL_GetMouseState (int *x, int *y)
{
	if ( x ) {
		*x = SDL_MouseX;
	}
	if ( y ) {
		*y = SDL_MouseY;
	}
	return(SDL_ButtonState);
}

Uint8 SDL_GetRelativeMouseState (int *x, int *y)
{
	if ( x )
		*x = SDL_DeltaX;
	if ( y )
		*y = SDL_DeltaY;
	SDL_DeltaX = 0;
	SDL_DeltaY = 0;
	return(SDL_ButtonState);
}

static void ClipOffset(Sint16 *x, Sint16 *y)
{
	/* This clips absolute mouse coordinates when the apparent
	   display surface is smaller than the real display surface.
	 */
	if ( SDL_VideoSurface && SDL_VideoSurface->offset ) {
		*y -= SDL_VideoSurface->offset/SDL_VideoSurface->pitch;
		*x -= (SDL_VideoSurface->offset%SDL_VideoSurface->pitch)/
				SDL_VideoSurface->format->BytesPerPixel;
	}
}

void SDL_SetMouseRange(int maxX, int maxY)
{
	SDL_MouseMaxX = (Sint16)maxX;
	SDL_MouseMaxY = (Sint16)maxY;
}

/* These are global for SDL_eventloop.c */
int SDL_PrivateMouseMotion(Uint8 buttonstate, int relative, Sint16 x, Sint16 y)
{
	int posted;
	Uint16 X, Y;
	Sint16 Xrel;
	Sint16 Yrel;

	/* Default buttonstate is the current one */
	if ( ! buttonstate ) {
		buttonstate = SDL_ButtonState;
	}

	Xrel = x;
	Yrel = y;
	if ( relative ) {
		/* Push the cursor around */
		x = (SDL_MouseX+x);
		y = (SDL_MouseY+y);
	} else {
		/* Do we need to clip {x,y} ? */
		ClipOffset(&x, &y);
	}

	/* Mouse coordinates range from 0 - width-1 and 0 - height-1 */
	if ( x < 0 )
		X = 0;
	else
	if ( x >= SDL_MouseMaxX )
		X = SDL_MouseMaxX-1;
	else
		X = (Uint16)x;

	if ( y < 0 )
		Y = 0;
	else
	if ( y >= SDL_MouseMaxY )
		Y = SDL_MouseMaxY-1;
	else
		Y = (Uint16)y;

	/* If not relative mode, generate relative motion from clamped X/Y.
	   This prevents lots of extraneous large delta relative motion when
	   the screen is windowed mode and the mouse is outside the window.
	*/
	if ( ! relative ) {
		Xrel = X-SDL_MouseX;
		Yrel = Y-SDL_MouseY;
	}

	/* Drop events that don't change state */
	if ( ! Xrel && ! Yrel ) {
#if 0
printf("Mouse event didn't change state - dropped!\n");
#endif
		return(0);
	}

	/* Update internal mouse state */
	SDL_ButtonState = buttonstate;
	SDL_MouseX = X;
	SDL_MouseY = Y;
	SDL_DeltaX += Xrel;
	SDL_DeltaY += Yrel;
        SDL_MoveCursor(SDL_MouseX, SDL_MouseY);

	/* Post the event, if desired */
	posted = 0;
	if ( SDL_ProcessEvents[SDL_MOUSEMOTION] == SDL_ENABLE ) {
		SDL_Event event;
		SDL_memset(&event, 0, sizeof(event));
		event.type = SDL_MOUSEMOTION;
		event.motion.state = buttonstate;
		event.motion.x = X;
		event.motion.y = Y;
		event.motion.xrel = Xrel;
		event.motion.yrel = Yrel;
		if ( (SDL_EventOK == NULL) || (*SDL_EventOK)(&event) ) {
			posted = 1;
			SDL_PushEvent(&event);
		}
	}
	return(posted);
}

int SDL_PrivateMouseButton(Uint8 state, Uint8 button, Sint16 x, Sint16 y)
{
	SDL_Event event;
	int posted;
	int move_mouse;
	Uint8 buttonstate;

	SDL_memset(&event, 0, sizeof(event));

	/* Check parameters */
	if ( x || y ) {
		ClipOffset(&x, &y);
		move_mouse = 1;
		/* Mouse coordinates range from 0 - width-1 and 0 - height-1 */
		if ( x < 0 )
			x = 0;
		else
		if ( x >= SDL_MouseMaxX )
			x = SDL_MouseMaxX-1;

		if ( y < 0 )
			y = 0;
		else
		if ( y >= SDL_MouseMaxY )
			y = SDL_MouseMaxY-1;
	} else {
		move_mouse = 0;
	}
	if ( ! x )
		x = SDL_MouseX;
	if ( ! y )
		y = SDL_MouseY;

	/* Figure out which event to perform */
	buttonstate = SDL_ButtonState;
	switch ( state ) {
		case SDL_PRESSED:
			event.type = SDL_MOUSEBUTTONDOWN;
			buttonstate |= SDL_BUTTON(button);
			break;
		case SDL_RELEASED:
			event.type = SDL_MOUSEBUTTONUP;
			buttonstate &= ~SDL_BUTTON(button);
			break;
		default:
			/* Invalid state -- bail */
			return(0);
	}

	/* Update internal mouse state */
	SDL_ButtonState = buttonstate;
	if ( move_mouse ) {
		SDL_MouseX = x;
		SDL_MouseY = y;
		SDL_MoveCursor(SDL_MouseX, SDL_MouseY);
	}

	/* Post the event, if desired */
	posted = 0;
	if ( SDL_ProcessEvents[event.type] == SDL_ENABLE ) {
		event.button.state = state;
		event.button.button = button;
		event.button.x = x;
		event.button.y = y;
		if ( (SDL_EventOK == NULL) || (*SDL_EventOK)(&event) ) {
			posted = 1;
			SDL_PushEvent(&event);
		}
	}
	return(posted);
}

