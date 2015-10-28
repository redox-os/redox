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

#include "SDL_stdinc.h"
#include "SDL_nxmodes_c.h"

SDL_Rect ** NX_ListModes (_THIS, SDL_PixelFormat * format, Uint32 flags)
{
    if (flags & SDL_FULLSCREEN)
        return SDL_modelist ;

    if (SDL_Visual.bpp == format -> BitsPerPixel) {
        return ((SDL_Rect **) -1) ;
    } else {
        return ((SDL_Rect **) 0) ;
    }
}

void NX_FreeVideoModes (_THIS)
{
    int i ;

    if (SDL_modelist) {
        for (i = 0; SDL_modelist [i]; ++ i) {
            SDL_free (SDL_modelist [i]) ;
        }
        SDL_free (SDL_modelist) ;
        SDL_modelist = NULL;
    }
}

int NX_EnterFullScreen (_THIS)
{
    if (! currently_fullscreen) {
        GR_SCREEN_INFO si ;

        GrGetScreenInfo (& si) ;
        GrResizeWindow (FSwindow, si.cols, si.rows) ;
        GrUnmapWindow (SDL_Window) ;
        GrMapWindow (FSwindow) ;
        GrRaiseWindow (FSwindow) ;
        GrSetFocus (FSwindow) ;
        currently_fullscreen = 1 ;      
    }

    return 1 ;
}

int NX_LeaveFullScreen (_THIS)
{
    if (currently_fullscreen) {
        GrUnmapWindow (FSwindow) ;
        GrMapWindow (SDL_Window) ;
        GrRaiseWindow (SDL_Window) ;
        GrSetFocus (SDL_Window) ;
        currently_fullscreen = 0 ;
    }

    return 0 ;
}
