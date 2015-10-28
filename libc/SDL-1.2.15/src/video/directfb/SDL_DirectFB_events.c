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

/* Handle the event stream, converting DirectFB input events into SDL events */

#include <sys/types.h>
#include <sys/time.h>
#include <unistd.h>
#include <fcntl.h>
#include <termios.h>

#include <directfb.h>

#include "SDL.h"
#include "../SDL_sysvideo.h"
#include "../../events/SDL_sysevents.h"
#include "../../events/SDL_events_c.h"
#include "SDL_DirectFB_video.h"
#include "SDL_DirectFB_events.h"

/* The translation tables from a DirectFB keycode to a SDL keysym */
static SDLKey keymap[256];
static SDL_keysym *DirectFB_TranslateKey (DFBInputEvent *ev, SDL_keysym *keysym);
static int DirectFB_TranslateButton (DFBInputEvent *ev);

static int posted = 0;


void DirectFB_PumpEvents (_THIS)
{
  DFBInputEvent evt;

  while (HIDDEN->eventbuffer->GetEvent (HIDDEN->eventbuffer,
                                        DFB_EVENT (&evt)) == DFB_OK)
    {
      SDL_keysym keysym;

      switch (evt.type)
        {
        case DIET_BUTTONPRESS:
          posted += SDL_PrivateMouseButton(SDL_PRESSED,
                                           DirectFB_TranslateButton (&evt), 0, 0);
          break;
        case DIET_BUTTONRELEASE:
          posted += SDL_PrivateMouseButton(SDL_RELEASED,
                                           DirectFB_TranslateButton (&evt), 0, 0);
          break;
        case DIET_KEYPRESS:
          posted += SDL_PrivateKeyboard(SDL_PRESSED, DirectFB_TranslateKey(&evt, &keysym));
          break;
        case DIET_KEYRELEASE:
          posted += SDL_PrivateKeyboard(SDL_RELEASED, DirectFB_TranslateKey(&evt, &keysym));
          break;
        case DIET_AXISMOTION:
          if (evt.flags & DIEF_AXISREL)
            {
              if (evt.axis == DIAI_X)
                posted += SDL_PrivateMouseMotion(0, 1, evt.axisrel, 0);
              else if (evt.axis == DIAI_Y)
                posted += SDL_PrivateMouseMotion(0, 1, 0, evt.axisrel);
            }
          else if (evt.flags & DIEF_AXISABS)
            {
              static int last_x, last_y;
              if (evt.axis == DIAI_X)
                last_x = evt.axisabs;
              else if (evt.axis == DIAI_Y)
                last_y = evt.axisabs;
              posted += SDL_PrivateMouseMotion(0, 0, last_x, last_y);
            }
          break;
        default:
          ;
        }
    }
}

void DirectFB_InitOSKeymap (_THIS)
{
  int i;
	
  /* Initialize the DirectFB key translation table */
  for (i=0; i<SDL_arraysize(keymap); ++i)
    keymap[i] = SDLK_UNKNOWN;

  keymap[DIKI_A - DIKI_UNKNOWN] = SDLK_a;
  keymap[DIKI_B - DIKI_UNKNOWN] = SDLK_b;
  keymap[DIKI_C - DIKI_UNKNOWN] = SDLK_c;
  keymap[DIKI_D - DIKI_UNKNOWN] = SDLK_d;
  keymap[DIKI_E - DIKI_UNKNOWN] = SDLK_e;
  keymap[DIKI_F - DIKI_UNKNOWN] = SDLK_f;
  keymap[DIKI_G - DIKI_UNKNOWN] = SDLK_g;
  keymap[DIKI_H - DIKI_UNKNOWN] = SDLK_h;
  keymap[DIKI_I - DIKI_UNKNOWN] = SDLK_i;
  keymap[DIKI_J - DIKI_UNKNOWN] = SDLK_j;
  keymap[DIKI_K - DIKI_UNKNOWN] = SDLK_k;
  keymap[DIKI_L - DIKI_UNKNOWN] = SDLK_l;
  keymap[DIKI_M - DIKI_UNKNOWN] = SDLK_m;
  keymap[DIKI_N - DIKI_UNKNOWN] = SDLK_n;
  keymap[DIKI_O - DIKI_UNKNOWN] = SDLK_o;
  keymap[DIKI_P - DIKI_UNKNOWN] = SDLK_p;
  keymap[DIKI_Q - DIKI_UNKNOWN] = SDLK_q;
  keymap[DIKI_R - DIKI_UNKNOWN] = SDLK_r;
  keymap[DIKI_S - DIKI_UNKNOWN] = SDLK_s;
  keymap[DIKI_T - DIKI_UNKNOWN] = SDLK_t;
  keymap[DIKI_U - DIKI_UNKNOWN] = SDLK_u;
  keymap[DIKI_V - DIKI_UNKNOWN] = SDLK_v;
  keymap[DIKI_W - DIKI_UNKNOWN] = SDLK_w;
  keymap[DIKI_X - DIKI_UNKNOWN] = SDLK_x;
  keymap[DIKI_Y - DIKI_UNKNOWN] = SDLK_y;
  keymap[DIKI_Z - DIKI_UNKNOWN] = SDLK_z;
  
  keymap[DIKI_0 - DIKI_UNKNOWN] = SDLK_0;
  keymap[DIKI_1 - DIKI_UNKNOWN] = SDLK_1;
  keymap[DIKI_2 - DIKI_UNKNOWN] = SDLK_2;
  keymap[DIKI_3 - DIKI_UNKNOWN] = SDLK_3;
  keymap[DIKI_4 - DIKI_UNKNOWN] = SDLK_4;
  keymap[DIKI_5 - DIKI_UNKNOWN] = SDLK_5;
  keymap[DIKI_6 - DIKI_UNKNOWN] = SDLK_6;
  keymap[DIKI_7 - DIKI_UNKNOWN] = SDLK_7;
  keymap[DIKI_8 - DIKI_UNKNOWN] = SDLK_8;
  keymap[DIKI_9 - DIKI_UNKNOWN] = SDLK_9;
  
  keymap[DIKI_F1 - DIKI_UNKNOWN] = SDLK_F1;
  keymap[DIKI_F2 - DIKI_UNKNOWN] = SDLK_F2;
  keymap[DIKI_F3 - DIKI_UNKNOWN] = SDLK_F3;
  keymap[DIKI_F4 - DIKI_UNKNOWN] = SDLK_F4;
  keymap[DIKI_F5 - DIKI_UNKNOWN] = SDLK_F5;
  keymap[DIKI_F6 - DIKI_UNKNOWN] = SDLK_F6;
  keymap[DIKI_F7 - DIKI_UNKNOWN] = SDLK_F7;
  keymap[DIKI_F8 - DIKI_UNKNOWN] = SDLK_F8;
  keymap[DIKI_F9 - DIKI_UNKNOWN] = SDLK_F9;
  keymap[DIKI_F10 - DIKI_UNKNOWN] = SDLK_F10;
  keymap[DIKI_F11 - DIKI_UNKNOWN] = SDLK_F11;
  keymap[DIKI_F12 - DIKI_UNKNOWN] = SDLK_F12;
  
  keymap[DIKI_ESCAPE - DIKI_UNKNOWN] = SDLK_ESCAPE;
  keymap[DIKI_LEFT - DIKI_UNKNOWN] = SDLK_LEFT;
  keymap[DIKI_RIGHT - DIKI_UNKNOWN] = SDLK_RIGHT;
  keymap[DIKI_UP - DIKI_UNKNOWN] = SDLK_UP;
  keymap[DIKI_DOWN - DIKI_UNKNOWN] = SDLK_DOWN;
  keymap[DIKI_CONTROL_L - DIKI_UNKNOWN] = SDLK_LCTRL;
  keymap[DIKI_CONTROL_R - DIKI_UNKNOWN] = SDLK_RCTRL;
  keymap[DIKI_SHIFT_L - DIKI_UNKNOWN] = SDLK_LSHIFT;
  keymap[DIKI_SHIFT_R - DIKI_UNKNOWN] = SDLK_RSHIFT;
  keymap[DIKI_ALT_L - DIKI_UNKNOWN] = SDLK_LALT;
  keymap[DIKI_ALT_R - DIKI_UNKNOWN] = SDLK_RALT;
  keymap[DIKI_TAB - DIKI_UNKNOWN] = SDLK_TAB;
  keymap[DIKI_ENTER - DIKI_UNKNOWN] = SDLK_RETURN;
  keymap[DIKI_SPACE - DIKI_UNKNOWN] = SDLK_SPACE;
  keymap[DIKI_BACKSPACE - DIKI_UNKNOWN] = SDLK_BACKSPACE;
  keymap[DIKI_INSERT - DIKI_UNKNOWN] = SDLK_INSERT;
  keymap[DIKI_DELETE - DIKI_UNKNOWN] = SDLK_DELETE;
  keymap[DIKI_HOME - DIKI_UNKNOWN] = SDLK_HOME;
  keymap[DIKI_END - DIKI_UNKNOWN] = SDLK_END;
  keymap[DIKI_PAGE_UP - DIKI_UNKNOWN] = SDLK_PAGEUP;
  keymap[DIKI_PAGE_DOWN - DIKI_UNKNOWN] = SDLK_PAGEDOWN;
  keymap[DIKI_CAPS_LOCK - DIKI_UNKNOWN] = SDLK_CAPSLOCK;
  keymap[DIKI_NUM_LOCK - DIKI_UNKNOWN] = SDLK_NUMLOCK;
  keymap[DIKI_SCROLL_LOCK - DIKI_UNKNOWN] = SDLK_SCROLLOCK;
  keymap[DIKI_PRINT - DIKI_UNKNOWN] = SDLK_PRINT;
  keymap[DIKI_PAUSE - DIKI_UNKNOWN] = SDLK_PAUSE;
  keymap[DIKI_KP_DIV - DIKI_UNKNOWN] = SDLK_KP_DIVIDE;
  keymap[DIKI_KP_MULT - DIKI_UNKNOWN] = SDLK_KP_MULTIPLY;
  keymap[DIKI_KP_MINUS - DIKI_UNKNOWN] = SDLK_KP_MINUS;
  keymap[DIKI_KP_PLUS - DIKI_UNKNOWN] = SDLK_KP_PLUS;
  keymap[DIKI_KP_ENTER - DIKI_UNKNOWN] = SDLK_KP_ENTER;
}


static SDL_keysym *DirectFB_TranslateKey (DFBInputEvent *ev, SDL_keysym *keysym)
{
  /* Set the keysym information */
  keysym->scancode = ev->key_id;
  keysym->mod = KMOD_NONE; /* FIXME */
  keysym->unicode = (DFB_KEY_TYPE (ev->key_symbol) == DIKT_UNICODE) ? ev->key_symbol : 0;

  if (ev->key_symbol > 0 && ev->key_symbol < 128)
    keysym->sym = ev->key_symbol;
  else
    keysym->sym = keymap[ev->key_id - DIKI_UNKNOWN];

  return keysym;
}

static int DirectFB_TranslateButton (DFBInputEvent *ev)
{
  switch (ev->button)
    {
    case DIBI_LEFT:
      return 1;
    case DIBI_MIDDLE:
      return 2;
    case DIBI_RIGHT:
      return 3;
    default:
      return 0;
    }
}
