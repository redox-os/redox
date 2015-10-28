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

#ifndef __SDL_PH_MODES_H__
#define __SDL_PH_MODES_H__

#include "SDL_ph_video.h"

#define PH_MAX_VIDEOMODES 127

#define PH_ENTER_DIRECTMODE  0
#define PH_IGNORE_DIRECTMODE 1

extern SDL_Rect **ph_ListModes(_THIS,SDL_PixelFormat *format, Uint32 flags);
extern void ph_FreeVideoModes(_THIS);
extern int ph_ResizeFullScreen(_THIS);
extern int ph_EnterFullScreen(_THIS, SDL_Surface* screen, int fmode);
extern int ph_LeaveFullScreen(_THIS);
extern int ph_GetVideoMode(int width, int height, int bpp);
extern int get_mode_any_format(int width, int height, int bpp);
extern int ph_ToggleFullScreen(_THIS, int on);

#endif /* __SDL_PH_MODES_H__ */
