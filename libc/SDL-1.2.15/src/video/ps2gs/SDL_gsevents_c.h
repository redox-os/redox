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

#include "SDL_gsvideo.h"

/* Variables and functions exported by SDL_sysevents.c to other parts 
   of the native video subsystem (SDL_sysvideo.c)
*/
extern int GS_OpenKeyboard(_THIS);
extern void GS_CloseKeyboard(_THIS);
extern int GS_OpenMouse(_THIS);
extern void GS_CloseMouse(_THIS);
extern int GS_EnterGraphicsMode(_THIS);
extern int GS_InGraphicsMode(_THIS);
extern void GS_LeaveGraphicsMode(_THIS);

extern void GS_InitOSKeymap(_THIS);
extern void GS_PumpEvents(_THIS);
