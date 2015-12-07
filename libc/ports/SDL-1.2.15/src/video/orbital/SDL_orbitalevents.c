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

#include "SDL_orbitalvideo.h"
#include "SDL_orbitalevents_c.h"
#include "SDL_orbitalscancode.h"

#include <unistd.h>

static SDLKey keymap[128];

#define EVENT_NONE 0
#define EVENT_MOUSE 1
#define EVENT_KEY 2
#define EVENT_QUIT 3

struct Event {
    int code;
    int a;
    int b;
    int c;
} __attribute__((packed));

void ORBITAL_PumpEvents(_THIS)
{
    struct Event event;
    while(read(this->hidden->fd, &event, sizeof(event)) > 0){
        if ( event.code == EVENT_KEY ){
            SDL_keysym keysym;
        	keysym.unicode = event.a;
        	keysym.scancode = event.b;
        	keysym.sym = keymap[event.b];
        	keysym.mod = KMOD_NONE;
            if ( event.c > 0 ) {
                SDL_PrivateKeyboard(SDL_PRESSED, &keysym);
            } else {
                SDL_PrivateKeyboard(SDL_RELEASED, &keysym);
            }
        }else if( event.code == EVENT_MOUSE ){
            SDL_PrivateMouseMotion(event.c, 0, event.a, event.b);
            //SDL_PrivateMouseButton(Uint8 state, Uint8 button, Sint16 x, Sint16 y);
        }
    }
}

void ORBITAL_InitOSKeymap(_THIS)
{
    int i;
    for ( i = 0; i < SDL_arraysize(keymap); ++i )
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
    keymap[SCANCODE_TICK] = SDLK_BACKQUOTE;
    keymap[SCANCODE_LEFTSHIFT] = SDLK_LSHIFT;
    keymap[SCANCODE_RIGHTSHIFT] = SDLK_RSHIFT;
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
    keymap[SCANCODE_F11] = SDLK_F11;
    keymap[SCANCODE_F12] = SDLK_F12;
    keymap[SCANCODE_HOME] = SDLK_HOME;
    keymap[SCANCODE_CURSORBLOCKUP] = SDLK_UP;
    keymap[SCANCODE_PAGEUP] = SDLK_PAGEUP;
    keymap[SCANCODE_CURSORBLOCKLEFT] = SDLK_LEFT;
    keymap[SCANCODE_CURSORBLOCKRIGHT] = SDLK_RIGHT;
    keymap[SCANCODE_END] = SDLK_END;
    keymap[SCANCODE_CURSORBLOCKDOWN] = SDLK_DOWN;
    keymap[SCANCODE_PAGEDOWN] = SDLK_PAGEDOWN;
    keymap[SCANCODE_INSERT] = SDLK_INSERT;
    keymap[SCANCODE_DELETE] = SDLK_DELETE;
}

/* end of SDL_orbitalevents.c ... */
