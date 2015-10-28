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
 *	Atari keyboard events manager, using BIOS
 *
 *	Patrice Mandin
 */

/* Mint includes */
#include <mint/osbind.h>
#include <mint/cookie.h>

#include "../../events/SDL_sysevents.h"
#include "../../events/SDL_events_c.h"

#include "SDL_atarikeys.h"
#include "SDL_atarievents_c.h"
#include "SDL_xbiosevents_c.h"
#include "SDL_ataridevmouse_c.h"

static unsigned char bios_currentkeyboard[ATARIBIOS_MAXKEYS];
static unsigned char bios_previouskeyboard[ATARIBIOS_MAXKEYS];
static SDL_bool use_dev_mouse = SDL_FALSE;

static void UpdateSpecialKeys(int special_keys_state);

void AtariBios_InitOSKeymap(_THIS)
{
	int vectors_mask;
/*	unsigned long dummy;*/

	SDL_memset(bios_currentkeyboard, 0, sizeof(bios_currentkeyboard));
	SDL_memset(bios_previouskeyboard, 0, sizeof(bios_previouskeyboard));

	use_dev_mouse = (SDL_AtariDevMouse_Open()!=0) ? SDL_TRUE : SDL_FALSE;

	vectors_mask = ATARI_XBIOS_JOYSTICKEVENTS;	/* XBIOS joystick events */
	if (!use_dev_mouse) {
		vectors_mask |= ATARI_XBIOS_MOUSEEVENTS;	/* XBIOS mouse events */
	}
/*	if (Getcookie(C_MiNT, &dummy)==C_FOUND) {
		vectors_mask = 0;
	}*/

	SDL_AtariXbios_InstallVectors(vectors_mask);
}

void AtariBios_PumpEvents(_THIS)
{
	int i;
	SDL_keysym keysym;

	/* Update pressed keys */
	SDL_memset(bios_currentkeyboard, 0, ATARIBIOS_MAXKEYS);

	while (Bconstat(_CON)) {
		unsigned long key_pressed;
		key_pressed=Bconin(_CON);
		bios_currentkeyboard[(key_pressed>>16)&(ATARIBIOS_MAXKEYS-1)]=0xFF;
	}

	/* Read special keys */
	UpdateSpecialKeys(Kbshift(-1));

	/* Now generate events */
	for (i=0; i<ATARIBIOS_MAXKEYS; i++) {
		/* Key pressed ? */
		if (bios_currentkeyboard[i] && !bios_previouskeyboard[i])
			SDL_PrivateKeyboard(SDL_PRESSED,
				SDL_Atari_TranslateKey(i, &keysym, SDL_TRUE));
			
		/* Key unpressed ? */
		if (bios_previouskeyboard[i] && !bios_currentkeyboard[i])
			SDL_PrivateKeyboard(SDL_RELEASED,
				SDL_Atari_TranslateKey(i, &keysym, SDL_FALSE));
	}

	if (use_dev_mouse) {
		SDL_AtariDevMouse_PostMouseEvents(this, SDL_TRUE);
	} else {
		SDL_AtariXbios_PostMouseEvents(this, SDL_TRUE);
	}

	/* Will be previous table */
	SDL_memcpy(bios_previouskeyboard, bios_currentkeyboard, sizeof(bios_previouskeyboard));
}

static void UpdateSpecialKeys(int special_keys_state)
{
#define UPDATE_SPECIAL_KEYS(numbit,scancode) \
	{	\
		if (special_keys_state & (1<<(numbit))) { \
			bios_currentkeyboard[scancode]=0xFF; \
		}	\
	}

	UPDATE_SPECIAL_KEYS(K_RSHIFT, SCANCODE_RIGHTSHIFT);
	UPDATE_SPECIAL_KEYS(K_LSHIFT, SCANCODE_LEFTSHIFT);
	UPDATE_SPECIAL_KEYS(K_CTRL, SCANCODE_LEFTCONTROL);
	UPDATE_SPECIAL_KEYS(K_ALT, SCANCODE_LEFTALT);
	UPDATE_SPECIAL_KEYS(K_CAPSLOCK, SCANCODE_CAPSLOCK);
}

void AtariBios_ShutdownEvents(void)
{
	SDL_AtariXbios_RestoreVectors();
	if (use_dev_mouse) {
		SDL_AtariDevMouse_Close();
	}
}
