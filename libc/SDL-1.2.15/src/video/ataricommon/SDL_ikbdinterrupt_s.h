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

/*
 *	Mouse vector
 *
 *	Patrice Mandin
 */

#ifndef _SDL_IKBDINTERRUPT_S_H_
#define _SDL_IKBDINTERRUPT_S_H_

#include <mint/osbind.h>

#include "SDL_stdinc.h"

/* Const */

#define IKBD_JOY_UP		(1<<0)
#define IKBD_JOY_DOWN	(1<<1)
#define IKBD_JOY_LEFT	(1<<2)
#define IKBD_JOY_RIGHT	(1<<3)
#define IKBD_JOY_FIRE	(1<<7)

/* Variables */

extern volatile Uint8  SDL_AtariIkbd_keyboard[128];	/* Keyboard table */
extern volatile Uint16 SDL_AtariIkbd_mouseb;	/* Mouse on port 0, buttons */
extern volatile Sint16 SDL_AtariIkbd_mousex;	/* Mouse X relative motion */
extern volatile Sint16 SDL_AtariIkbd_mousey;	/* Mouse Y relative motion */
extern volatile Uint16 SDL_AtariIkbd_joystick;	/* Joystick on port 1 */

/* For joystick driver to know if this is usable */
extern Uint16 SDL_AtariIkbd_enabled;
										
/* Functions */ 

extern void SDL_AtariIkbdInstall(void);
extern void SDL_AtariIkbdUninstall(void);

#endif /* _SDL_IKBDINTERRUPT_S_H_ */
