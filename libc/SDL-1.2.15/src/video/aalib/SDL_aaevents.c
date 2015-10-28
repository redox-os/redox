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

/* Handle the event stream, converting AA events into SDL events */

#include <stdio.h>

#include <aalib.h>

#include "SDL.h"
#include "../../events/SDL_sysevents.h"
#include "../../events/SDL_events_c.h"
#include "SDL_aavideo.h"
#include "SDL_aaevents_c.h"

/* The translation tables from a console scancode to a SDL keysym */
static SDLKey keymap[401];

static SDL_keysym *TranslateKey(int scancode, SDL_keysym *keysym);


void AA_PumpEvents(_THIS)
{
	int posted = 0;
	int mouse_button, mouse_x, mouse_y;
	int evt;
	SDL_keysym keysym;

	static int prev_button = -1, prev_x = -1, prev_y = -1;

	if( ! this->screen ) /* Wait till we got the screen initialized */
	  return;

	do {
		posted = 0;
		/* Gather events */

		/* Get mouse status */
		SDL_mutexP(AA_mutex);
		aa_getmouse (AA_context, &mouse_x, &mouse_y, &mouse_button);
		SDL_mutexV(AA_mutex);
		mouse_x = mouse_x * this->screen->w / aa_scrwidth (AA_context);
		mouse_y = mouse_y * this->screen->h / aa_scrheight (AA_context);

		/* Compare against previous state and generate events */
		if( prev_button != mouse_button ) {
			if( mouse_button & AA_BUTTON1 ) {
				if ( ! (prev_button & AA_BUTTON1) ) {
					posted += SDL_PrivateMouseButton(SDL_PRESSED, 1, 0, 0);
				}
			} else {
				if ( prev_button & AA_BUTTON1 ) {
					posted += SDL_PrivateMouseButton(SDL_RELEASED, 1, 0, 0);
				}
			}
			if( mouse_button & AA_BUTTON2 ) {
				if ( ! (prev_button & AA_BUTTON2) ) {
					posted += SDL_PrivateMouseButton(SDL_PRESSED, 2, 0, 0);
				}
			} else {
				if ( prev_button & AA_BUTTON2 ) {
					posted += SDL_PrivateMouseButton(SDL_RELEASED, 2, 0, 0);
				}
			}
			if( mouse_button & AA_BUTTON3 ) {
				if ( ! (prev_button & AA_BUTTON3) ) {
					posted += SDL_PrivateMouseButton(SDL_PRESSED, 3, 0, 0);
				}
			} else {
				if ( prev_button & AA_BUTTON3 ) {
					posted += SDL_PrivateMouseButton(SDL_RELEASED, 3, 0, 0);
				}
			}
		}
		if ( prev_x != mouse_x || prev_y != mouse_y ) {
			posted += SDL_PrivateMouseMotion(0, 0, mouse_x, mouse_y);
		}

		prev_button = mouse_button;
		prev_x = mouse_x; prev_y = mouse_y;

		/* Get keyboard event */
		SDL_mutexP(AA_mutex);
		evt = aa_getevent(AA_context, 0);
		SDL_mutexV(AA_mutex);
		if ( (evt > AA_NONE) && (evt < AA_RELEASE) && (evt != AA_MOUSE) && (evt != AA_RESIZE) ) {
			/* Key pressed */
/*    			printf("Key pressed: %d (%c)\n", evt, evt); */
			posted += SDL_PrivateKeyboard(SDL_PRESSED, TranslateKey(evt, &keysym));
		} else if ( evt >= AA_RELEASE ) {
			/* Key released */
			evt &= ~AA_RELEASE;
/*  			printf("Key released: %d (%c)\n", evt, evt); */
			posted += SDL_PrivateKeyboard(SDL_RELEASED, TranslateKey(evt, &keysym));
		}
	} while ( posted );
}

void AA_InitOSKeymap(_THIS)
{
	int i;
	static const char *std_keys = " 01234567890&#'()_-|$*+-=/\\:;.,!?<>{}[]@~%^\x9";
	const char *std;

	/* Initialize the AAlib key translation table */
	for ( i=0; i<SDL_arraysize(keymap); ++i )
		keymap[i] = SDLK_UNKNOWN;

	/* Alphabet keys */
	for ( i = 0; i<26; ++i ){
		keymap['a' + i] = SDLK_a+i;
		keymap['A' + i] = SDLK_a+i;
	}
	/* Function keys */
	for ( i = 0; i<12; ++i ){
		keymap[334 + i] = SDLK_F1+i;
	}
	/* Keys that have the same symbols and don't have to be translated */
	for( std = std_keys; *std; std ++ ) {
		keymap[*std] = *std;
	}

	keymap[13] = SDLK_RETURN;
	keymap[AA_BACKSPACE] = SDLK_BACKSPACE;

	keymap[369] = SDLK_LSHIFT;
	keymap[370] = SDLK_RSHIFT;
	keymap[371] = SDLK_LCTRL;
	keymap[372] = SDLK_RCTRL;
	keymap[377] = SDLK_LALT;
	keymap[270] = SDLK_RALT;
	keymap[271] = SDLK_NUMLOCK;
	keymap[373] = SDLK_CAPSLOCK;
	keymap[164] = SDLK_SCROLLOCK;

	keymap[243] = SDLK_INSERT;
	keymap[304] = SDLK_DELETE;
	keymap[224] = SDLK_HOME;
	keymap[231] = SDLK_END;
	keymap[229] = SDLK_PAGEUP;
	keymap[230] = SDLK_PAGEDOWN;

	keymap[241] = SDLK_PRINT;
	keymap[163] = SDLK_BREAK;

	keymap[302] = SDLK_KP0;
	keymap[300] = SDLK_KP1;
	keymap[297] = SDLK_KP2;
	keymap[299] = SDLK_KP3;
	keymap[294] = SDLK_KP4;
	keymap[301] = SDLK_KP5;
	keymap[296] = SDLK_KP6;
	keymap[293] = SDLK_KP7;
	keymap[295] = SDLK_KP8;
	keymap[298] = SDLK_KP9;

	keymap[AA_ESC] = SDLK_ESCAPE;
	keymap[AA_UP] = SDLK_UP;
	keymap[AA_DOWN] = SDLK_DOWN;
	keymap[AA_LEFT] = SDLK_LEFT;
	keymap[AA_RIGHT] = SDLK_RIGHT;
}

static SDL_keysym *TranslateKey(int scancode, SDL_keysym *keysym)
{
	/* Sanity check */
	if ( scancode >= SDL_arraysize(keymap) )
		scancode = AA_UNKNOWN;

	/* Set the keysym information */
	keysym->scancode = scancode;
	keysym->sym = keymap[scancode];
	keysym->mod = KMOD_NONE;

	/* If UNICODE is on, get the UNICODE value for the key */
	keysym->unicode = 0;
	if ( SDL_TranslateUNICODE ) {
		/* Populate the unicode field with the ASCII value */
		keysym->unicode = scancode;
	}
	return(keysym);
}
