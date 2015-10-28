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

/* Utilities for getting and setting the X display mode */

#include "SDL_x11video.h"

/* Define this if you want to grab the keyboard in fullscreen mode.
   If you do not define this, SDL will return from SDL_SetVideoMode()
   immediately, but will not actually go fullscreen until the window
   manager is idle.
*/
#define GRAB_FULLSCREEN

extern int X11_GetVideoModes(_THIS);
extern SDL_Rect **X11_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags);
extern void X11_FreeVideoModes(_THIS);
extern int X11_ResizeFullScreen(_THIS);
extern void X11_WaitMapped(_THIS, Window win);
extern void X11_WaitUnmapped(_THIS, Window win);
extern void X11_QueueEnterFullScreen(_THIS);
extern int X11_EnterFullScreen(_THIS);
extern int X11_LeaveFullScreen(_THIS);
