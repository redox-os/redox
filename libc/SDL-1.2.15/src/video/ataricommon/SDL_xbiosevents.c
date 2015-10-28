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
 *	XBIOS mouse & joystick vectors
 *
 *	Patrice Mandin
 */

#include <mint/osbind.h>

#include "../../events/SDL_events_c.h"
#include "SDL_atarisuper.h"
#include "SDL_xbiosevents_c.h"
#include "SDL_xbiosinterrupt_s.h"

/* Variables */

int SDL_AtariXbios_enabled=0;

/* Local variables */

static _KBDVECS *kbdvecs;		/* Pointer to access system vectors */
static Uint16 atari_prevmouseb;	/* buttons */

/* Functions */

void SDL_AtariXbios_InstallVectors(int vectors_mask)
{
	void *oldpile;

	/* Clear variables */
	SDL_AtariXbios_mouselock =
		SDL_AtariXbios_mouseb =
		SDL_AtariXbios_mousex =
		SDL_AtariXbios_mousey =
		SDL_AtariXbios_joystick =
		atari_prevmouseb = 0;

	if (vectors_mask==0) {
		SDL_AtariXbios_enabled=0;
		return;
	}

	/* Read IKBD vectors base */
	kbdvecs=Kbdvbase();

	/* Go to supervisor mode */
	oldpile=(void *)Super(0);

	/* Install our vectors */
	SDL_AtariXbios_Install(
		kbdvecs,
		(vectors_mask & ATARI_XBIOS_MOUSEEVENTS) ? SDL_AtariXbios_MouseVector : NULL,
		(vectors_mask & ATARI_XBIOS_JOYSTICKEVENTS) ? SDL_AtariXbios_JoystickVector : NULL
	);

	/* Back to user mode */
	SuperToUser(oldpile);

	SDL_AtariXbios_enabled=1;
}

void SDL_AtariXbios_RestoreVectors(void)
{
	void *oldpile;

	if (SDL_AtariXbios_enabled==0) {
		return;
	}

	/* Read IKBD vectors base */
	kbdvecs=Kbdvbase();

	/* Go to supervisor mode */
	oldpile=(void *)Super(NULL);

	/* Reinstall system vector */
	SDL_AtariXbios_Restore(kbdvecs);

	/* Back to user mode */
	SuperToUser(oldpile);
}

static int atari_GetButton(int button)
{
	switch(button)
	{
		case 0:
			return SDL_BUTTON_RIGHT;
			break;
		case 1:
		default:
			return SDL_BUTTON_LEFT;
			break;
	}
}

void SDL_AtariXbios_PostMouseEvents(_THIS, SDL_bool buttonEvents)
{
	if (SDL_AtariXbios_enabled==0) {
		return;
	}

	/* Mouse motion ? */
	if (SDL_AtariXbios_mousex || SDL_AtariXbios_mousey) {
		SDL_PrivateMouseMotion(0, 1, SDL_AtariXbios_mousex, SDL_AtariXbios_mousey);
		SDL_AtariXbios_mousex = SDL_AtariXbios_mousey = 0;
	}
	
	/* Mouse button ? */
	if (buttonEvents && (SDL_AtariXbios_mouseb != atari_prevmouseb)) {
		int i;

		for (i=0;i<2;i++) {
			int curbutton, prevbutton;

			curbutton = SDL_AtariXbios_mouseb & (1<<i);
			prevbutton = atari_prevmouseb & (1<<i);

			if (curbutton && !prevbutton) {
				SDL_PrivateMouseButton(SDL_PRESSED, atari_GetButton(i), 0, 0);
			}
			if (!curbutton && prevbutton) {
				SDL_PrivateMouseButton(SDL_RELEASED, atari_GetButton(i), 0, 0);
			}
		}
		atari_prevmouseb = SDL_AtariXbios_mouseb;
	}
}

void SDL_AtariXbios_LockMousePosition(SDL_bool lockPosition)
{
	SDL_AtariXbios_mouselock = lockPosition;
}
