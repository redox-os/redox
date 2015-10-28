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

    Micah Dowty
    micahjd@users.sourceforge.net
*/
#include "SDL_config.h"

#include "SDL.h"
#include "../../events/SDL_sysevents.h"
#include "../../events/SDL_events_c.h"
#include "SDL_pgvideo.h"
#include "SDL_pgevents_c.h"

int PG_HandleClose(struct pgEvent *evt)
{
        SDL_PrivateQuit();
	return 1;               /* Intercept the event's normal quit handling */
}

int PG_HandleResize(struct pgEvent *evt)
{
        SDL_PrivateResize(evt->e.size.w, evt->e.size.h);
	return 0;
}

int PG_HandleKey(struct pgEvent *evt)
{
        SDL_keysym sym;
	SDL_memset(&sym,0,sizeof(sym));
	sym.sym = evt->e.kbd.key;
	sym.mod = evt->e.kbd.mods;
        SDL_PrivateKeyboard(evt->type == PG_WE_KBD_KEYDOWN, &sym);
	return 0;
}

int PG_HandleChar(struct pgEvent *evt)
{
        SDL_keysym sym;
	SDL_memset(&sym,0,sizeof(sym));
	sym.unicode = evt->e.kbd.key;
	sym.mod = evt->e.kbd.mods;
        SDL_PrivateKeyboard(evt->type == PG_WE_KBD_KEYDOWN, &sym);
	return 0;
}

int PG_HandleMouseButton(struct pgEvent *evt)
{        
        /* We need to focus the canvas when it's clicked */
        if (evt->extra) {
	        SDL_VideoDevice *this = (SDL_VideoDevice *) evt->extra;
		pgFocus(this->hidden->wCanvas);
	}
        SDL_PrivateMouseButton(evt->type == PG_WE_PNTR_DOWN, evt->e.pntr.chbtn,
			       evt->e.pntr.x, evt->e.pntr.y);
	return 0;
}

int PG_HandleMouseMotion(struct pgEvent *evt)
{
        SDL_PrivateMouseMotion(evt->e.pntr.btn,0,evt->e.pntr.x, evt->e.pntr.y);
	return 0;
}

void PG_PumpEvents(_THIS)
{
        /* Process all pending events */
        pgEventPoll();
}

void PG_InitOSKeymap(_THIS)
{
        /* We need no keymap */
}

void PG_InitEvents(_THIS)
{
        /* Turn on all the mouse and keyboard triggers for our canvas, normally less important
	 * events like mouse movement are ignored to save bandwidth. */
        pgSetWidget(this->hidden->wCanvas, PG_WP_TRIGGERMASK, 
		    pgGetWidget(this->hidden->wCanvas, PG_WP_TRIGGERMASK) |
		    PG_TRIGGER_UP | PG_TRIGGER_DOWN | PG_TRIGGER_MOVE |
		    PG_TRIGGER_KEYUP | PG_TRIGGER_KEYDOWN | PG_TRIGGER_CHAR,0);

	/* Start our canvas out focused, so we get keyboard input */
	pgFocus(this->hidden->wCanvas);

        /* Set up bindings for all the above event handlers */
        pgBind(this->hidden->wApp,    PG_WE_CLOSE, &PG_HandleClose, NULL);
        pgBind(this->hidden->wCanvas, PG_WE_BUILD, &PG_HandleResize, NULL);
        pgBind(this->hidden->wCanvas, PG_WE_KBD_CHAR, &PG_HandleChar, NULL);
        pgBind(this->hidden->wCanvas, PG_WE_KBD_KEYUP, &PG_HandleKey, NULL);
        pgBind(this->hidden->wCanvas, PG_WE_KBD_KEYDOWN, &PG_HandleKey, NULL);
        pgBind(this->hidden->wCanvas, PG_WE_PNTR_MOVE, &PG_HandleMouseMotion, NULL);
        pgBind(this->hidden->wCanvas, PG_WE_PNTR_UP, &PG_HandleMouseButton, NULL);
        pgBind(this->hidden->wCanvas, PG_WE_PNTR_DOWN, &PG_HandleMouseButton, this);
}

/* end of SDL_pgevents.c ... */
