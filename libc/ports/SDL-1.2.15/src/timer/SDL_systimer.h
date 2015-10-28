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

/* The system dependent timer handling functions */

#include "SDL_timer.h"
#include "SDL_timer_c.h"


/* Initialize the system dependent timer subsystem */
extern int SDL_SYS_TimerInit(void);

/* Quit the system dependent timer subsystem */
extern void SDL_SYS_TimerQuit(void);

/* Start a timer set up by SDL_SetTimer() */
extern int SDL_SYS_StartTimer(void);

/* Stop a previously started timer */
extern void SDL_SYS_StopTimer(void);
