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

/* This is the DirectFB implementation of YUV video overlays */

#include "SDL_video.h"
#include "SDL_DirectFB_video.h"

extern SDL_Overlay *DirectFB_CreateYUVOverlay(_THIS, int width, int height, Uint32 format, SDL_Surface *display);

extern int DirectFB_LockYUVOverlay(_THIS, SDL_Overlay *overlay);

extern void DirectFB_UnlockYUVOverlay(_THIS, SDL_Overlay *overlay);

extern int DirectFB_DisplayYUVOverlay(_THIS, SDL_Overlay *overlay, SDL_Rect *src, SDL_Rect *dst);

extern void DirectFB_FreeYUVOverlay(_THIS, SDL_Overlay *overlay);

