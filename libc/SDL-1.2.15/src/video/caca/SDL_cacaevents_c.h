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

#ifdef SAVE_RCSID
static char rcsid =
 "@(#) $Id: libsdl-1.2.11-libcaca.patch,v 1.1 2006/09/18 16:06:06 mr_bones_ Exp $";
#endif

#include "SDL_cacavideo.h"

/* Variables and functions exported by SDL_sysevents.c to other parts.
   of the native video subsystem (SDL_sysvideo.c)
*/
extern void Caca_PumpEvents(_THIS);
extern void Caca_InitOSKeymap(_THIS);

