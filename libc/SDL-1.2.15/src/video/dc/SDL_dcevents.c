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

#include "SDL.h"
#include "../../events/SDL_sysevents.h"
#include "../../events/SDL_events_c.h"
#include "SDL_dcvideo.h"
#include "SDL_dcevents_c.h"

#include <dc/maple.h>
#include <dc/maple/mouse.h>
#include <dc/maple/keyboard.h>

const static unsigned short sdl_key[]= {
	/*0*/	0, 0, 0, 0, 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
		'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't',
		'u', 'v', 'w', 'x', 'y', 'z',
	/*1e*/	'1', '2', '3', '4', '5', '6', '7', '8', '9', '0',
	/*28*/	SDLK_RETURN, SDLK_ESCAPE, SDLK_BACKSPACE, SDLK_TAB, SDLK_SPACE, SDLK_MINUS, SDLK_PLUS, SDLK_LEFTBRACKET, 
	SDLK_RIGHTBRACKET, SDLK_BACKSLASH , 0, SDLK_SEMICOLON, SDLK_QUOTE,
	/*35*/	'~', SDLK_COMMA, SDLK_PERIOD, SDLK_SLASH, SDLK_CAPSLOCK, 
	SDLK_F1, SDLK_F2, SDLK_F3, SDLK_F4, SDLK_F5, SDLK_F6, SDLK_F7, SDLK_F8, SDLK_F9, SDLK_F10, SDLK_F11, SDLK_F12,
	/*46*/	SDLK_PRINT, SDLK_SCROLLOCK, SDLK_PAUSE, SDLK_INSERT, SDLK_HOME, SDLK_PAGEUP, SDLK_DELETE, SDLK_END, SDLK_PAGEDOWN, SDLK_RIGHT, SDLK_LEFT, SDLK_DOWN, SDLK_UP,
	/*53*/	SDLK_NUMLOCK, SDLK_KP_DIVIDE, SDLK_KP_MULTIPLY, SDLK_KP_MINUS, SDLK_KP_PLUS, SDLK_KP_ENTER, 
	SDLK_KP1, SDLK_KP2, SDLK_KP3, SDLK_KP4, SDLK_KP5, SDLK_KP6,
	/*5f*/	SDLK_KP7, SDLK_KP8, SDLK_KP9, SDLK_KP0, SDLK_KP_PERIOD, 0 /* S3 */
};

const static unsigned short sdl_shift[] = {
	SDLK_LCTRL,SDLK_LSHIFT,SDLK_LALT,0 /* S1 */,
	SDLK_RCTRL,SDLK_RSHIFT,SDLK_RALT,0 /* S2 */,
};

#define	MOUSE_WHEELUP 	(1<<4)
#define	MOUSE_WHEELDOWN	(1<<5)

static void mouse_update(void)
{
const	static char sdl_mousebtn[] = {
	MOUSE_LEFTBUTTON,
	MOUSE_RIGHTBUTTON,
	MOUSE_SIDEBUTTON,
	MOUSE_WHEELUP,
	MOUSE_WHEELDOWN
};

	uint8 addr;
	mouse_cond_t	cond;

	static int prev_buttons;
	int buttons,changed;
	int i;

	if ((addr = maple_first_mouse())==0 || mouse_get_cond(addr, &cond)<0) return;

	buttons = cond.buttons^0xff;
	if (cond.dz<0) buttons|=MOUSE_WHEELUP;
	if (cond.dz>0) buttons|=MOUSE_WHEELDOWN;

	if (cond.dx||cond.dy) SDL_PrivateMouseMotion(0,1,cond.dx,cond.dy);

	changed = buttons^prev_buttons;
	for(i=0;i<sizeof(sdl_mousebtn);i++) {
		if (changed & sdl_mousebtn[i]) {
			SDL_PrivateMouseButton((buttons & sdl_mousebtn[i])?SDL_PRESSED:SDL_RELEASED,i,0,0);
		}
	}
	prev_buttons = buttons;
}

static void keyboard_update(void)
{
	static kbd_state_t	old_state;
	static uint8 old_addr;

	kbd_state_t	*state;
	uint8	addr;
	int	port,unit;

	int shiftkeys;
	SDL_keysym keysym;

	int i;

	addr = maple_first_kb();

	if (addr==0) return;

	if (addr!=old_addr) {
		old_addr = addr;
		SDL_memset(&old_state,0,sizeof(old_state));
	}

	maple_raddr(addr,&port,&unit);

	state = maple_dev_state(port,unit);
	if (!state) return;

	shiftkeys = state->shift_keys ^ old_state.shift_keys;
	for(i=0;i<sizeof(sdl_shift);i++) {
		if ((shiftkeys>>i)&1) {
			keysym.sym = sdl_shift[i];
			SDL_PrivateKeyboard(((state->shift_keys>>i)&1)?SDL_PRESSED:SDL_RELEASED,&keysym);
		}
	}

	for(i=0;i<sizeof(sdl_key);i++) {
		if (state->matrix[i]!=old_state.matrix[i]) {
			int key = sdl_key[i];
			if (key) {
				keysym.sym = key;
				SDL_PrivateKeyboard(state->matrix[i]?SDL_PRESSED:SDL_RELEASED,&keysym);
			}
		}
	}

	old_state = *state;
}

void DC_PumpEvents(_THIS)
{
	keyboard_update();
	mouse_update();
}

void DC_InitOSKeymap(_THIS)
{
	/* do nothing. */
}

/* end of SDL_dcevents.c ... */

