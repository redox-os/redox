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

/* Being a nds driver, there's no event stream. We just define stubs for
   most of the API. */
#include <nds.h>
#include "SDL.h"
#include "../../events/SDL_sysevents.h"
#include "../../events/SDL_events_c.h"
#include "SDL_ndsvideo.h"
#include "SDL_ndsevents_c.h"

static SDLKey keymap[NDS_NUMKEYS];
char keymem[NDS_NUMKEYS];	/* memorize states of buttons */

void NDS_PumpEvents(_THIS)
{
	scanKeys();
	int i;
	SDL_keysym keysym;
	keysym.mod=KMOD_NONE;
	for(i=0;i<NDS_NUMKEYS;i++)
	{
		keysym.scancode=i;
		keysym.sym=keymap[i];
		if(keysHeld()&(1<<i) && !keymem[i])
		{
			keymem[i]=1;
			//printf("key released %d\n",i);
			SDL_PrivateKeyboard(SDL_RELEASED, &keysym);
		}
		if(!(keysHeld()&(1<<i)) && keymem[i])
		{
			keymem[i]=0;
			//printf("key pressed %d\n",i);
			SDL_PrivateKeyboard(SDL_PRESSED, &keysym);
		}
	}
	//touchPosition touch;
	//touch=touchReadXY();
	//if (touch.px!=0 || touch.py!=0)
	//	SDL_PrivateMouseMotion(SDL_PRESSED, 0, touch.px, touch.py);
}

void NDS_InitOSKeymap(_THIS)
{
	SDL_memset(keymem,1,NDS_NUMKEYS);
	keymap[KEY_A]=SDLK_a;
	keymap[KEY_B]=SDLK_s;
	keymap[KEY_X]=SDLK_w;
	keymap[KEY_Y]=SDLK_d;
	keymap[KEY_L]=SDLK_q;
	keymap[KEY_R]=SDLK_e;
	keymap[KEY_UP]=SDLK_UP;
	keymap[KEY_DOWN]=SDLK_DOWN;
	keymap[KEY_LEFT]=SDLK_LEFT;
	keymap[KEY_RIGHT]=SDLK_RIGHT;
	keymap[KEY_SELECT]=SDLK_SPACE;
	keymap[KEY_START]=SDLK_RETURN;
}

/* end of SDL_gbaevents.c ... */

