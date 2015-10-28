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

#include "SDL_nxvideo.h"

extern int NX_SetupImage (_THIS, SDL_Surface * screen) ;
extern void NX_DestroyImage (_THIS, SDL_Surface * screen) ;
extern int NX_ResizeImage (_THIS, SDL_Surface * screen, Uint32 flags) ;

extern void NX_NormalUpdate (_THIS, int numrects, SDL_Rect * rects) ;
extern void NX_RefreshDisplay (_THIS) ;
