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
 *	Atari keyboard events manager
 *
 *	Patrice Mandin
 *
 *	This routines choose what the final event manager will be
 */

#include <mint/cookie.h>
#include <mint/osbind.h>

#include "../../events/SDL_sysevents.h"
#include "../../events/SDL_events_c.h"

#include "SDL_atarikeys.h"
#include "SDL_atarievents_c.h"
#include "SDL_biosevents_c.h"
#include "SDL_gemdosevents_c.h"
#include "SDL_ikbdevents_c.h"

enum {
	MCH_ST=0,
	MCH_STE,
	MCH_TT,
	MCH_F30,
	MCH_CLONE,
	MCH_ARANYM
};

/* The translation tables from a console scancode to a SDL keysym */
static SDLKey keymap[ATARIBIOS_MAXKEYS];
static char *keytab_normal;

void (*Atari_ShutdownEvents)(void);

static void Atari_InitializeEvents(_THIS)
{
	const char *envr;
	long cookie_mch;

	/* Test if we are on an Atari machine or not */
	if (Getcookie(C__MCH, &cookie_mch) == C_NOTFOUND) {
		cookie_mch = 0;
	}
	cookie_mch >>= 16;

	/* Default is Ikbd, the faster except for clones */
	switch(cookie_mch) {
		case MCH_ST:
		case MCH_STE:
		case MCH_TT:
		case MCH_F30:
		case MCH_ARANYM:
			this->InitOSKeymap=AtariIkbd_InitOSKeymap;
			this->PumpEvents=AtariIkbd_PumpEvents;
			Atari_ShutdownEvents=AtariIkbd_ShutdownEvents;
			break;
		default:
			this->InitOSKeymap=AtariGemdos_InitOSKeymap;
			this->PumpEvents=AtariGemdos_PumpEvents;
			Atari_ShutdownEvents=AtariGemdos_ShutdownEvents;
			break;
	}

	envr = SDL_getenv("SDL_ATARI_EVENTSDRIVER");

 	if (!envr) {
		return;
	}

	if (SDL_strcmp(envr, "ikbd") == 0) {
		this->InitOSKeymap=AtariIkbd_InitOSKeymap;
		this->PumpEvents=AtariIkbd_PumpEvents;
		Atari_ShutdownEvents=AtariIkbd_ShutdownEvents;
	}

	if (SDL_strcmp(envr, "gemdos") == 0) {
		this->InitOSKeymap=AtariGemdos_InitOSKeymap;
		this->PumpEvents=AtariGemdos_PumpEvents;
		Atari_ShutdownEvents=AtariGemdos_ShutdownEvents;
	}

	if (SDL_strcmp(envr, "bios") == 0) {
		this->InitOSKeymap=AtariBios_InitOSKeymap;
		this->PumpEvents=AtariBios_PumpEvents;
		Atari_ShutdownEvents=AtariBios_ShutdownEvents;
	}
}

void Atari_InitOSKeymap(_THIS)
{
	Atari_InitializeEvents(this);

	SDL_Atari_InitInternalKeymap(this);

	/* Call choosen routine */
	this->InitOSKeymap(this);
}

void SDL_Atari_InitInternalKeymap(_THIS)
{
	int i;
	_KEYTAB *key_tables;

	/* Read system tables for scancode -> ascii translation */
	key_tables = (_KEYTAB *) Keytbl(KT_NOCHANGE, KT_NOCHANGE, KT_NOCHANGE);
	keytab_normal = key_tables->unshift;

	/* Initialize keymap */
	for ( i=0; i<ATARIBIOS_MAXKEYS; i++ )
		keymap[i] = SDLK_UNKNOWN;

	/* Functions keys */
	for ( i = 0; i<10; i++ )
		keymap[SCANCODE_F1 + i] = SDLK_F1+i;

	/* Cursor keypad */
	keymap[SCANCODE_HELP] = SDLK_HELP;
	keymap[SCANCODE_UNDO] = SDLK_UNDO;
	keymap[SCANCODE_INSERT] = SDLK_INSERT;
	keymap[SCANCODE_CLRHOME] = SDLK_HOME;
	keymap[SCANCODE_UP] = SDLK_UP;
	keymap[SCANCODE_DOWN] = SDLK_DOWN;
	keymap[SCANCODE_RIGHT] = SDLK_RIGHT;
	keymap[SCANCODE_LEFT] = SDLK_LEFT;

	/* Special keys */
	keymap[SCANCODE_ESCAPE] = SDLK_ESCAPE;
	keymap[SCANCODE_BACKSPACE] = SDLK_BACKSPACE;
	keymap[SCANCODE_TAB] = SDLK_TAB;
	keymap[SCANCODE_ENTER] = SDLK_RETURN;
	keymap[SCANCODE_DELETE] = SDLK_DELETE;
	keymap[SCANCODE_LEFTCONTROL] = SDLK_LCTRL;
	keymap[SCANCODE_LEFTSHIFT] = SDLK_LSHIFT;
	keymap[SCANCODE_RIGHTSHIFT] = SDLK_RSHIFT;
	keymap[SCANCODE_LEFTALT] = SDLK_LALT;
	keymap[SCANCODE_CAPSLOCK] = SDLK_CAPSLOCK;
}

void Atari_PumpEvents(_THIS)
{
	Atari_InitializeEvents(this);

	/* Call choosen routine */
	this->PumpEvents(this);
}

/* Atari to Unicode charset translation table */

Uint16 SDL_AtariToUnicodeTable[256]={
	/* Standard ASCII characters from 0x00 to 0x7e */
	/* Unicode stuff from 0x7f to 0xff */

	0x0000,0x0001,0x0002,0x0003,0x0004,0x0005,0x0006,0x0007,
	0x0008,0x0009,0x000A,0x000B,0x000C,0x000D,0x000E,0x000F,
	0x0010,0x0011,0x0012,0x0013,0x0014,0x0015,0x0016,0x0017,
	0x0018,0x0019,0x001A,0x001B,0x001C,0x001D,0x001E,0x001F,
	0x0020,0x0021,0x0022,0x0023,0x0024,0x0025,0x0026,0x0027,
	0x0028,0x0029,0x002A,0x002B,0x002C,0x002D,0x002E,0x002F,
	0x0030,0x0031,0x0032,0x0033,0x0034,0x0035,0x0036,0x0037,
	0x0038,0x0039,0x003A,0x003B,0x003C,0x003D,0x003E,0x003F,

	0x0040,0x0041,0x0042,0x0043,0x0044,0x0045,0x0046,0x0047,
	0x0048,0x0049,0x004A,0x004B,0x004C,0x004D,0x004E,0x004F,
	0x0050,0x0051,0x0052,0x0053,0x0054,0x0055,0x0056,0x0057,
	0x0058,0x0059,0x005A,0x005B,0x005C,0x005D,0x005E,0x005F,
	0x0060,0x0061,0x0062,0x0063,0x0064,0x0065,0x0066,0x0067,
	0x0068,0x0069,0x006A,0x006B,0x006C,0x006D,0x006E,0x006F,
	0x0070,0x0071,0x0072,0x0073,0x0074,0x0075,0x0076,0x0077,
	0x0078,0x0079,0x007A,0x007B,0x007C,0x007D,0x007E,0x0394,

	0x00C7,0x00FC,0x00E9,0x00E2,0x00E4,0x00E0,0x00E5,0x00E7,
	0x00EA,0x00EB,0x00E8,0x00EF,0x00EE,0x00EC,0x00C4,0x00C5,
	0x00C9,0x00E6,0x00C6,0x00F4,0x00F6,0x00F2,0x00FB,0x00F9,
	0x00FF,0x00D6,0x00DC,0x00A2,0x00A3,0x00A5,0x00DF,0x0192,
	0x00E1,0x00ED,0x00F3,0x00FA,0x00F1,0x00D1,0x00AA,0x00BA,
	0x00BF,0x2310,0x00AC,0x00BD,0x00BC,0x00A1,0x00AB,0x00BB,
	0x00C3,0x00F5,0x00D8,0x00F8,0x0153,0x0152,0x00C0,0x00C3,
	0x00D5,0x00A8,0x00B4,0x2020,0x00B6,0x00A9,0x00AE,0x2122,

	0x0133,0x0132,0x05D0,0x05D1,0x05D2,0x05D3,0x05D4,0x05D5,
	0x05D6,0x05D7,0x05D8,0x05D9,0x05DB,0x05DC,0x05DE,0x05E0,
	0x05E1,0x05E2,0x05E4,0x05E6,0x05E7,0x05E8,0x05E9,0x05EA,
	0x05DF,0x05DA,0x05DD,0x05E3,0x05E5,0x00A7,0x2038,0x221E,
	0x03B1,0x03B2,0x0393,0x03C0,0x03A3,0x03C3,0x00B5,0x03C4,
	0x03A6,0x0398,0x03A9,0x03B4,0x222E,0x03C6,0x2208,0x2229,
	0x2261,0x00B1,0x2265,0x2264,0x2320,0x2321,0x00F7,0x2248,
	0x00B0,0x2022,0x00B7,0x221A,0x207F,0x00B2,0x00B3,0x00AF
};

SDL_keysym *SDL_Atari_TranslateKey(int scancode, SDL_keysym *keysym,
	SDL_bool pressed)
{
	int asciicode = 0;

	/* Set the keysym information */
	keysym->scancode = scancode;
	keysym->mod = KMOD_NONE;
	keysym->sym = keymap[scancode];
	keysym->unicode = 0;

	if (keysym->sym == SDLK_UNKNOWN) {
		keysym->sym = asciicode = keytab_normal[scancode];		
	}

	if (SDL_TranslateUNICODE && pressed) {
		keysym->unicode = SDL_AtariToUnicodeTable[asciicode];
	}

	return(keysym);
}
