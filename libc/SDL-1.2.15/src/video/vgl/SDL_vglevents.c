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

/* Handle the event stream, converting X11 events into SDL events */

#include <stdio.h>

#include <sys/fbio.h>
#include <sys/consio.h>
#include <sys/kbio.h>
#include <vgl.h>

#include "SDL.h"
#include "SDL_thread.h"
#include "../../events/SDL_sysevents.h"
#include "../../events/SDL_events_c.h"
#include "SDL_vglvideo.h"
#include "SDL_vglevents_c.h"

/* The translation tables from a console scancode to a SDL keysym */
/* FIXME: Free the keymap when we shut down the video mode */
static keymap_t *vga_keymap = NULL;
static SDLKey keymap[128];
static SDL_keysym *TranslateKey(int scancode, SDL_keysym *keysym);

static int posted = 0;
static int oldx = -1;
static int oldy = -1;
static struct mouse_info mouseinfo;

/* Ugh, we have to duplicate the kernel's keysym mapping code...
   Oh, it's not so bad. :-)

   FIXME: Add keyboard LED handling code
 */
int VGL_initkeymaps(int fd)
{
	vga_keymap = SDL_malloc(sizeof(keymap_t));
	if ( ! vga_keymap ) {
		SDL_OutOfMemory();
		return(-1);
	}
	if (ioctl(fd, GIO_KEYMAP, vga_keymap) == -1) {
		SDL_free(vga_keymap);
		vga_keymap = NULL;
		SDL_SetError("Unable to get keyboard map");
		return(-1);
	}
	return(0);
}

static void handle_keyboard(_THIS)
{
	SDL_keysym keysym;
	int c, pressed, scancode;

	while ((c = VGLKeyboardGetCh()) != 0) {
		scancode = c & 0x7F;
                if (c & 0x80) {
                        pressed = SDL_RELEASED;
                } else {
                        pressed = SDL_PRESSED;
                }

		posted += SDL_PrivateKeyboard(pressed,
				 TranslateKey(scancode, &keysym));
	}
}

int VGL_initmouse(int fd)
{
	mouseinfo.operation = MOUSE_GETINFO;
	if (ioctl(fd, CONS_MOUSECTL, &mouseinfo) != 0)
		return -1;

	return 0;
}

static void handle_mouse(_THIS)
{
	char buttons;
	int x, y;
	int button_state, state_changed, state;
	int i;

	ioctl(0, CONS_MOUSECTL, &mouseinfo);
	x = mouseinfo.u.data.x;
	y = mouseinfo.u.data.y;
	buttons = mouseinfo.u.data.buttons;

	if ((x != oldx) || (y != oldy)) {
		posted += SDL_PrivateMouseMotion(0, 0, x, y);
		oldx = x;
		oldy = y;
	}

	/* See what's changed */
	button_state = SDL_GetMouseState(NULL, NULL);
	state_changed = button_state ^ buttons;
	for (i = 0; i < 8; i++) {
		if (state_changed & (1<<i)) {
			if (buttons & (1<<i)) {
				state = SDL_PRESSED;
			} else {
				state = SDL_RELEASED;
			}
			posted += SDL_PrivateMouseButton(state, i + 1, 0, 0);
		}
	}
}
	

void VGL_PumpEvents(_THIS)
{
	do {
		posted = 0;
		handle_keyboard(this);
		handle_mouse(this);
	} while (posted != 0);
}

void VGL_InitOSKeymap(_THIS)
{
	int i;

	/* Initialize the BeOS key translation table */
	for ( i=0; i<SDL_arraysize(keymap); ++i )
		keymap[i] = SDLK_UNKNOWN;

	keymap[SCANCODE_ESCAPE] = SDLK_ESCAPE;
	keymap[SCANCODE_1] = SDLK_1;
	keymap[SCANCODE_2] = SDLK_2;
	keymap[SCANCODE_3] = SDLK_3;
	keymap[SCANCODE_4] = SDLK_4;
	keymap[SCANCODE_5] = SDLK_5;
	keymap[SCANCODE_6] = SDLK_6;
	keymap[SCANCODE_7] = SDLK_7;
	keymap[SCANCODE_8] = SDLK_8;
	keymap[SCANCODE_9] = SDLK_9;
	keymap[SCANCODE_0] = SDLK_0;
	keymap[SCANCODE_MINUS] = SDLK_MINUS;
	keymap[SCANCODE_EQUAL] = SDLK_EQUALS;
	keymap[SCANCODE_BACKSPACE] = SDLK_BACKSPACE;
	keymap[SCANCODE_TAB] = SDLK_TAB;
	keymap[SCANCODE_Q] = SDLK_q;
	keymap[SCANCODE_W] = SDLK_w;
	keymap[SCANCODE_E] = SDLK_e;
	keymap[SCANCODE_R] = SDLK_r;
	keymap[SCANCODE_T] = SDLK_t;
	keymap[SCANCODE_Y] = SDLK_y;
	keymap[SCANCODE_U] = SDLK_u;
	keymap[SCANCODE_I] = SDLK_i;
	keymap[SCANCODE_O] = SDLK_o;
	keymap[SCANCODE_P] = SDLK_p;
	keymap[SCANCODE_BRACKET_LEFT] = SDLK_LEFTBRACKET;
	keymap[SCANCODE_BRACKET_RIGHT] = SDLK_RIGHTBRACKET;
	keymap[SCANCODE_ENTER] = SDLK_RETURN;
	keymap[SCANCODE_LEFTCONTROL] = SDLK_LCTRL;
	keymap[SCANCODE_A] = SDLK_a;
	keymap[SCANCODE_S] = SDLK_s;
	keymap[SCANCODE_D] = SDLK_d;
	keymap[SCANCODE_F] = SDLK_f;
	keymap[SCANCODE_G] = SDLK_g;
	keymap[SCANCODE_H] = SDLK_h;
	keymap[SCANCODE_J] = SDLK_j;
	keymap[SCANCODE_K] = SDLK_k;
	keymap[SCANCODE_L] = SDLK_l;
	keymap[SCANCODE_SEMICOLON] = SDLK_SEMICOLON;
	keymap[SCANCODE_APOSTROPHE] = SDLK_QUOTE;
	keymap[SCANCODE_GRAVE] = SDLK_BACKQUOTE;
	keymap[SCANCODE_LEFTSHIFT] = SDLK_LSHIFT;
	keymap[SCANCODE_BACKSLASH] = SDLK_BACKSLASH;
	keymap[SCANCODE_Z] = SDLK_z;
	keymap[SCANCODE_X] = SDLK_x;
	keymap[SCANCODE_C] = SDLK_c;
	keymap[SCANCODE_V] = SDLK_v;
	keymap[SCANCODE_B] = SDLK_b;
	keymap[SCANCODE_N] = SDLK_n;
	keymap[SCANCODE_M] = SDLK_m;
	keymap[SCANCODE_COMMA] = SDLK_COMMA;
	keymap[SCANCODE_PERIOD] = SDLK_PERIOD;
	keymap[SCANCODE_SLASH] = SDLK_SLASH;
	keymap[SCANCODE_RIGHTSHIFT] = SDLK_RSHIFT;
	keymap[SCANCODE_KEYPADMULTIPLY] = SDLK_KP_MULTIPLY;
	keymap[SCANCODE_LEFTALT] = SDLK_LALT;
	keymap[SCANCODE_SPACE] = SDLK_SPACE;
	keymap[SCANCODE_CAPSLOCK] = SDLK_CAPSLOCK;
	keymap[SCANCODE_F1] = SDLK_F1;
	keymap[SCANCODE_F2] = SDLK_F2;
	keymap[SCANCODE_F3] = SDLK_F3;
	keymap[SCANCODE_F4] = SDLK_F4;
	keymap[SCANCODE_F5] = SDLK_F5;
	keymap[SCANCODE_F6] = SDLK_F6;
	keymap[SCANCODE_F7] = SDLK_F7;
	keymap[SCANCODE_F8] = SDLK_F8;
	keymap[SCANCODE_F9] = SDLK_F9;
	keymap[SCANCODE_F10] = SDLK_F10;
	keymap[SCANCODE_NUMLOCK] = SDLK_NUMLOCK;
	keymap[SCANCODE_SCROLLLOCK] = SDLK_SCROLLOCK;
	keymap[SCANCODE_KEYPAD7] = SDLK_KP7;
	keymap[SCANCODE_CURSORUPLEFT] = SDLK_KP7;
	keymap[SCANCODE_KEYPAD8] = SDLK_KP8;
	keymap[SCANCODE_CURSORUP] = SDLK_KP8;
	keymap[SCANCODE_KEYPAD9] = SDLK_KP9;
	keymap[SCANCODE_CURSORUPRIGHT] = SDLK_KP9;
	keymap[SCANCODE_KEYPADMINUS] = SDLK_KP_MINUS;
	keymap[SCANCODE_KEYPAD4] = SDLK_KP4;
	keymap[SCANCODE_CURSORLEFT] = SDLK_KP4;
	keymap[SCANCODE_KEYPAD5] = SDLK_KP5;
	keymap[SCANCODE_KEYPAD6] = SDLK_KP6;
	keymap[SCANCODE_CURSORRIGHT] = SDLK_KP6;
	keymap[SCANCODE_KEYPADPLUS] = SDLK_KP_PLUS;
	keymap[SCANCODE_KEYPAD1] = SDLK_KP1;
	keymap[SCANCODE_CURSORDOWNLEFT] = SDLK_KP1;
	keymap[SCANCODE_KEYPAD2] = SDLK_KP2;
	keymap[SCANCODE_CURSORDOWN] = SDLK_KP2;
	keymap[SCANCODE_KEYPAD3] = SDLK_KP3;
	keymap[SCANCODE_CURSORDOWNRIGHT] = SDLK_KP3;
	keymap[SCANCODE_KEYPAD0] = SDLK_KP0;
	keymap[SCANCODE_KEYPADPERIOD] = SDLK_KP_PERIOD;
	keymap[SCANCODE_LESS] = SDLK_LESS;
	keymap[SCANCODE_F11] = SDLK_F11;
	keymap[SCANCODE_F12] = SDLK_F12;
	keymap[SCANCODE_KEYPADENTER] = SDLK_KP_ENTER;
	keymap[SCANCODE_RIGHTCONTROL] = SDLK_RCTRL;
	keymap[SCANCODE_CONTROL] = SDLK_RCTRL;
	keymap[SCANCODE_KEYPADDIVIDE] = SDLK_KP_DIVIDE;
	keymap[SCANCODE_PRINTSCREEN] = SDLK_PRINT;
	keymap[SCANCODE_RIGHTALT] = SDLK_RALT;
	keymap[SCANCODE_BREAK] = SDLK_BREAK;
	keymap[SCANCODE_BREAK_ALTERNATIVE] = SDLK_UNKNOWN;
	keymap[SCANCODE_HOME] = SDLK_HOME;
	keymap[SCANCODE_CURSORBLOCKUP] = SDLK_UP;
	keymap[SCANCODE_PAGEUP] = SDLK_PAGEUP;
	keymap[SCANCODE_CURSORBLOCKLEFT] = SDLK_LEFT;
	keymap[SCANCODE_CURSORBLOCKRIGHT] = SDLK_RIGHT;
	keymap[SCANCODE_END] = SDLK_END;
	keymap[SCANCODE_CURSORBLOCKDOWN] = SDLK_DOWN;
	keymap[SCANCODE_PAGEDOWN] = SDLK_PAGEDOWN;
	keymap[SCANCODE_INSERT] = SDLK_INSERT;
	keymap[SCANCODE_REMOVE] = SDLK_DELETE;
	keymap[119] = SDLK_PAUSE;
	keymap[SCANCODE_RIGHTWIN] = SDLK_RSUPER;
	keymap[SCANCODE_LEFTWIN] = SDLK_LSUPER;
	keymap[127] = SDLK_MENU;
}

static SDL_keysym *TranslateKey(int scancode, SDL_keysym *keysym)
{
	/* Set the keysym information */
	keysym->scancode = scancode;
	keysym->sym = keymap[scancode];
	keysym->mod = KMOD_NONE;

	/* If UNICODE is on, get the UNICODE value for the key */
	keysym->unicode = 0;
	if ( SDL_TranslateUNICODE && vga_keymap ) {
		int map;
		SDLMod modstate;

		modstate = SDL_GetModState();
		map = 0;
		if ( modstate & KMOD_SHIFT ) {
			map += 1;
		}
		if ( modstate & KMOD_CTRL ) {
			map += 2;
		}
		if ( modstate & KMOD_ALT ) {
			map += 4;
		}
		if ( !(vga_keymap->key[scancode].spcl & (0x80 >> map)) ) {
			keysym->unicode = vga_keymap->key[scancode].map[map];
		}

	}
	return(keysym);
}

