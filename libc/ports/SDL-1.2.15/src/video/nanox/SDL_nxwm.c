/*
    SDL - Simple DirectMedia Layer
    Copyright (C) 1997-2012 Sam Lantinga
    Copyright (C) 2001  Hsieh-Fu Tsai

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
    slouken@libsdl.org
    
    Hsieh-Fu Tsai
    clare@setabox.com
*/
#include "SDL_config.h"

#include "SDL_syswm.h"
#include "../../events/SDL_events_c.h"

#include "SDL_nxwm_c.h"

void NX_SetCaption (_THIS, const char * title, const char * icon)
{
    Dprintf ("enter NX_SetCaption\n") ;

    // Lock the event thread, in multi-threading environments
    SDL_Lock_EventThread () ;
    
    if (SDL_Window) 
        GrSetWindowTitle (SDL_Window, title) ;
    
    SDL_Unlock_EventThread () ;
    Dprintf ("leave NX_SetCaption\n") ;
}

int NX_GetWMInfo (_THIS, SDL_SysWMinfo * info)
{
    Dprintf ("enter NX_GetWMInfo\n") ;

    if (info -> version.major <= SDL_MAJOR_VERSION) {
        info -> window = SDL_Window ;
        return 1 ;
    } else {
        SDL_SetError("Application not compiled with SDL %d.%d\n",
            SDL_MAJOR_VERSION, SDL_MINOR_VERSION) ;
        return -1 ;
    }

    Dprintf ("leave NX_GetWMInfo\n") ;
}
