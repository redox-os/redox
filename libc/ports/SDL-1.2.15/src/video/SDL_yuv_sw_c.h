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

#include "SDL_video.h"
#include "SDL_sysvideo.h"

/* This is the software implementation of the YUV video overlay support */

extern SDL_Overlay *SDL_CreateYUV_SW(_THIS, int width, int height, Uint32 format, SDL_Surface *display);

extern int SDL_LockYUV_SW(_THIS, SDL_Overlay *overlay);

extern void SDL_UnlockYUV_SW(_THIS, SDL_Overlay *overlay);

extern int SDL_DisplayYUV_SW(_THIS, SDL_Overlay *overlay, SDL_Rect *src, SDL_Rect *dst);

extern void SDL_FreeYUV_SW(_THIS, SDL_Overlay *overlay);
