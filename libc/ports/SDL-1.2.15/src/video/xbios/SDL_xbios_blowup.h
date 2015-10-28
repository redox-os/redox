/*
    SDL - Simple DirectMedia Layer
    Copyright (C) 1997-2012 Sam Lantinga

    This library is free software; you can redistribute it and/or
    modify it under the terms of the GNU Library General Public
    License as published by the Free Software Foundation; either
    version 2 of the License, or (at your option) any later version.

    This library is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
    Library General Public License for more details.

    You should have received a copy of the GNU Library General Public
    License along with this library; if not, write to the Free
    Foundation, Inc., 59 Temple Place, Suite 330, Boston, MA  02111-1307  USA

    Sam Lantinga
    slouken@libsdl.org
*/
#include "SDL_config.h"

/*
	Blowup extension definitions

	Patrice Mandin
*/

#ifndef _SDL_xbios_blowup_h
#define _SDL_xbios_blowup_h

#include "SDL_xbios.h"

/*--- Types ---*/

typedef struct {
	/* 64 bytes */
	unsigned short	enabled;		/* Extended mode enabled ? 0=yes, <>0=no */
	unsigned short	dummy10[6];
	unsigned short	registers_0E;	/* value for register 0xffff820e */
	unsigned short	registers_10;	/* value for register 0xffff8210 */
	unsigned short	dummy11[23];

	/* 64 bytes */
	unsigned short	width;			/* width-1 */
	unsigned short	height;			/* height-1 */
	unsigned short	dummy20;
	unsigned long	screensize;		/* screensize in bytes */
	unsigned short	dummy21[8];
	unsigned short	virtual;		/* Virtual screen ? */
	unsigned short	virwidth;		/* Virtual screen width */
	unsigned short	virheight;		/* Virtual screen height */

	unsigned short dummy22;
	unsigned short monitor;			/* Monitor defined for this mode */
	unsigned short extension;		/* Extended mode defined ? 0=yes, 1=no */
	unsigned short dummy23[13];

	/* 64 bytes */
	unsigned short	dummy30;
	unsigned short	registers_82[6];	/* values for registers 0xffff8282-8c */
	unsigned short	dummy31[9];

	unsigned short	dummy32;
	unsigned short	registers_A2[6];	/* values for registers 0xffff82a2-ac */
	unsigned short	dummy33[9];

	/* 64 bytes */
	unsigned short	registers_C0;	/* value for register 0xffff82c0 */
	unsigned short	registers_C2;	/* value for register 0xffff82c2 */
	unsigned short	dummy40[30];
} blow_mode_t;

typedef struct {
	blow_mode_t	blowup_modes[10];
	unsigned char	num_mode[6];
	unsigned long	dummy;
	unsigned short	montype;
} blow_cookie_t;

/*--- Functions prototypes ---*/

void SDL_XBIOS_ListBlowupModes(_THIS, int actually_add, blow_cookie_t *cookie_blow);

#endif /* _SDL_xbios_blowup_h */
