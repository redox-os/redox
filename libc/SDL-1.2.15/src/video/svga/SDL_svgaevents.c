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

#include <vga.h>
#include <vgamouse.h>
#include <vgakeyboard.h>
#if defined(__LINUX__)
#include <linux/kd.h>
#include <linux/keyboard.h>
#elif defined(__FREEBSD__)
#include <sys/kbio.h>
#else
#error You must choose your operating system here
#endif

#include "../../events/SDL_sysevents.h"
#include "../../events/SDL_events_c.h"
#include "SDL_svgavideo.h"
#include "SDL_svgaevents_c.h"

/* The translation tables from a console scancode to a SDL keysym */
#if defined(linux)
#define NUM_VGAKEYMAPS	(1<<KG_CAPSSHIFT)
static Uint16 vga_keymap[NUM_VGAKEYMAPS][NR_KEYS];
#elif defined(__FREEBSD__)
/* FIXME: Free the keymap when we shut down the video mode */
static keymap_t *vga_keymap = NULL;
#else
#error You must choose your operating system here
#endif
static SDLKey keymap[128];
static SDL_keysym *TranslateKey(int scancode, SDL_keysym *keysym);

/* Ugh, we have to duplicate the kernel's keysym mapping code...
   Oh, it's not so bad. :-)

   FIXME: Add keyboard LED handling code
 */
#if defined(linux)
int SVGA_initkeymaps(int fd)
{
	struct kbentry entry;
	int map, i;

	/* Load all the keysym mappings */
	for ( map=0; map<NUM_VGAKEYMAPS; ++map ) {
		SDL_memset(vga_keymap[map], 0, NR_KEYS*sizeof(Uint16));
		for ( i=0; i<NR_KEYS; ++i ) {
			entry.kb_table = map;
			entry.kb_index = i;
			if ( ioctl(fd, KDGKBENT, &entry) == 0 ) {
				/* The "Enter" key is a special case */
				if ( entry.kb_value == K_ENTER ) {
					entry.kb_value = K(KT_ASCII,13);
				}
				/* Handle numpad specially as well */
				if ( KTYP(entry.kb_value) == KT_PAD ) {
				    switch ( entry.kb_value ) {
					case K_P0:
					case K_P1:
					case K_P2:
					case K_P3:
					case K_P4:
					case K_P5:
					case K_P6:
					case K_P7:
					case K_P8:
					case K_P9:
					    vga_keymap[map][i]=entry.kb_value;
					    vga_keymap[map][i]+= '0';
					    break;
                                        case K_PPLUS:
					    vga_keymap[map][i]=K(KT_ASCII,'+');
					    break;
                                        case K_PMINUS:
					    vga_keymap[map][i]=K(KT_ASCII,'-');
					    break;
                                        case K_PSTAR:
					    vga_keymap[map][i]=K(KT_ASCII,'*');
					    break;
                                        case K_PSLASH:
					    vga_keymap[map][i]=K(KT_ASCII,'/');
					    break;
                                        case K_PENTER:
					    vga_keymap[map][i]=K(KT_ASCII,'\r');
					    break;
                                        case K_PCOMMA:
					    vga_keymap[map][i]=K(KT_ASCII,',');
					    break;
                                        case K_PDOT:
					    vga_keymap[map][i]=K(KT_ASCII,'.');
					    break;
					default:
					    break;
				    }
				}
				/* Do the normal key translation */
				if ( (KTYP(entry.kb_value) == KT_LATIN) ||
				     (KTYP(entry.kb_value) == KT_ASCII) ||
				     (KTYP(entry.kb_value) == KT_LETTER) ) {
					vga_keymap[map][i] = entry.kb_value;
				}
			}
		}
	}
	return(0);
}
#elif defined(__FREEBSD__)
int SVGA_initkeymaps(int fd)
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
#else
#error You must choose your operating system here
#endif

int posted = 0;

void SVGA_mousecallback(int button, int dx, int dy,
                          int u1,int u2,int u3, int u4)
{
	if ( dx || dy ) {
		posted += SDL_PrivateMouseMotion(0, 1, dx, dy);
	}
	if ( button & MOUSE_LEFTBUTTON ) {
		if ( !(SDL_GetMouseState(NULL, NULL) & SDL_BUTTON(1)) ) {
			posted += SDL_PrivateMouseButton(SDL_PRESSED, 1, 0, 0);
		}
	} else {
		if ( (SDL_GetMouseState(NULL, NULL) & SDL_BUTTON(1)) ) {
			posted += SDL_PrivateMouseButton(SDL_RELEASED, 1, 0, 0);
		}
	}
	if ( button & MOUSE_MIDDLEBUTTON ) {
		if ( !(SDL_GetMouseState(NULL, NULL) & SDL_BUTTON(2)) ) {
			posted += SDL_PrivateMouseButton(SDL_PRESSED, 2, 0, 0);
		}
	} else {
		if ( (SDL_GetMouseState(NULL, NULL) & SDL_BUTTON(2)) ) {
			posted += SDL_PrivateMouseButton(SDL_RELEASED, 2, 0, 0);
		}
	}
	if ( button & MOUSE_RIGHTBUTTON ) {
		if ( !(SDL_GetMouseState(NULL, NULL) & SDL_BUTTON(3)) ) {
			posted += SDL_PrivateMouseButton(SDL_PRESSED, 3, 0, 0);
		}
	} else {
		if ( (SDL_GetMouseState(NULL, NULL) & SDL_BUTTON(3)) ) {
			posted += SDL_PrivateMouseButton(SDL_RELEASED, 3, 0, 0);
		}
	}
}

void SVGA_keyboardcallback(int scancode, int pressed)
{
	SDL_keysym keysym;

	if ( pressed ) {
		posted += SDL_PrivateKeyboard(SDL_PRESSED,
			    TranslateKey(scancode, &keysym));
	} else {
		posted += SDL_PrivateKeyboard(SDL_RELEASED,
			    TranslateKey(scancode, &keysym));
	}
}

void SVGA_PumpEvents(_THIS)
{
	do {
		posted = 0;
		mouse_update();
		keyboard_update();
	} while ( posted );
}

void SVGA_InitOSKeymap(_THIS)
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

#if defined(linux)
static SDL_keysym *TranslateKey(int scancode, SDL_keysym *keysym)
{
	/* Set the keysym information */
	keysym->scancode = scancode;
	keysym->sym = keymap[scancode];
	keysym->mod = KMOD_NONE;

	/* If UNICODE is on, get the UNICODE value for the key */
	keysym->unicode = 0;
	if ( SDL_TranslateUNICODE ) {
		int map;
		SDLMod modstate;

		modstate = SDL_GetModState();
		map = 0;
		if ( modstate & KMOD_SHIFT ) {
			map |= (1<<KG_SHIFT);
		}
		if ( modstate & KMOD_CTRL ) {
			map |= (1<<KG_CTRL);
		}
		if ( modstate & KMOD_ALT ) {
			map |= (1<<KG_ALT);
		}
		if ( modstate & KMOD_MODE ) {
			map |= (1<<KG_ALTGR);
		}
		if ( KTYP(vga_keymap[map][scancode]) == KT_LETTER ) {
			if ( modstate & KMOD_CAPS ) {
				map ^= (1<<KG_SHIFT);
			}
		}
		if ( KTYP(vga_keymap[map][scancode]) == KT_PAD ) {
			if ( modstate & KMOD_NUM ) {
				keysym->unicode=KVAL(vga_keymap[map][scancode]);
			}
		} else {
			keysym->unicode = KVAL(vga_keymap[map][scancode]);
		}
	}
	return(keysym);
}
#elif defined(__FREEBSD__)
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
#else
#error You must choose your operating system here
#endif
