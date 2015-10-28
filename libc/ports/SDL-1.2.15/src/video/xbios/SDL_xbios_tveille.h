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
	Turbo veille screensaver

	Patrice Mandin
*/

#ifndef _SDL_xbios_tveille_h
#define _SDL_xbios_tveille_h

#include "SDL_xbios.h"

/*--- Structures ---*/

typedef struct {
	unsigned long	version;
	void		(*prg_ptr)();
	void		(*kbd_ptr)();
	void		(*vbl_ptr)();
	unsigned long	vbl_count;
	void		(*oldkbd_ptr)();
	unsigned long	off_count;
	unsigned long	prg_size;
	unsigned long	dummy1[4];
	unsigned char	dummy2;
	unsigned char	status;
	unsigned short	freq;
	unsigned short	dummy3;
	unsigned char	clear_first;
	unsigned char	enabled;	/* 0=enabled, 0xff=disabled */
	unsigned char	serial_redir;
	unsigned char	dummy4;
	void		(*oldserial_ptr)();
} tveille_t;

/*--- Functions prototypes ---*/

int SDL_XBIOS_TveillePresent(_THIS);
void SDL_XBIOS_TveilleDisable(_THIS);
void SDL_XBIOS_TveilleEnable(_THIS);

#endif /* _SDL_xbios_tveille_h */
