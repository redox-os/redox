/*
    SDL - Simple DirectMedia Layer
    Copyright (C) 1997-2012 Sam Lantinga

    This library is free software; you can redistribute it and/or
    modify it under the terms of the GNU Library General Public
    License as published by the Free Software Foundation; either
    version 2 of the License, or (at your option) any later version.

    This library is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
    Library General Public License for more details.

    You should have received a copy of the GNU Library General Public
    License along with this library; if not, write to the Free
    Foundation, Inc., 59 Temple Place, Suite 330, Boston, MA  02111-1307  USA

    Sam Lantinga
    slouken@devolution.com
*/

/*
    SDL_epocevents_c.h
    Handle the event stream, converting Epoc events into SDL events

    Epoc version by Hannu Viitala (hannu.j.viitala@mbnet.fi) and Markus Mertama
    
*/


#ifdef SAVE_RCSID
static char rcsid =
 "@(#) $Id: SDL_aaevents_c.h,v 1.1.2.2 2000/03/16 15:20:39 hercules Exp $";
#endif

extern "C" {
#include "SDL_sysvideo.h"
//#include "SDL_epocvideo.h"
}



#define MAX_SCANCODE 255

/* Variables and functions exported by SDL_sysevents.c to other parts 
   of the native video subsystem (SDL_sysvideo.c)
*/

/* Hidden "this" pointer for the video functions */
#define _THIS	SDL_VideoDevice *_this
#define Private	_this->hidden

extern "C" {
extern void EPOC_InitOSKeymap(_THIS);
extern void EPOC_PumpEvents(_THIS);
}

extern TBool isCursorVisible;

