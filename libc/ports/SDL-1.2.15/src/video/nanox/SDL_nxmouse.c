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

#include "../../events/SDL_events_c.h"

#include "SDL_nxmouse_c.h"

// The implementation dependent data for the window manager cursor
struct WMcursor {
    int unused ;
} ;

WMcursor * NX_CreateWMCursor (_THIS,
        Uint8 * data, Uint8 * mask, int w, int h, int hot_x, int hot_y)
{
    WMcursor * cursor ;

    Dprintf ("enter NX_CreateWMCursor\n") ;

    cursor = (WMcursor *) SDL_malloc (sizeof (WMcursor)) ;
    if (cursor == NULL) {
        SDL_OutOfMemory () ;
        return NULL ;
    }

    Dprintf ("leave NX_CreateWMCursor\n") ;
    return cursor ;
}

void NX_FreeWMCursor (_THIS, WMcursor * cursor)
{
    Dprintf ("NX_FreeWMCursor\n") ;
    SDL_free (cursor) ;
    return ;
}

void NX_WarpWMCursor(_THIS, Uint16 x, Uint16 y)
{
    GR_WINDOW_INFO info ;

    Dprintf ("enter NX_WarpWMCursor\n") ;
    SDL_Lock_EventThread () ;
    
    GrGetWindowInfo (SDL_Window, & info) ;
    GrMoveCursor (info.x + x, info.y + y) ;

    SDL_Unlock_EventThread () ;
    Dprintf ("leave NX_WarpWMCursor\n") ;
}

int NX_ShowWMCursor (_THIS, WMcursor * cursor)
{
    Dprintf ("NX_ShowWMCursor\n") ;
    return 1 ;
}
