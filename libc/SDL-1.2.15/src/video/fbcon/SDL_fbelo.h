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

#ifndef SDL_fbelo_h
#define SDL_fbelo_h

#include "SDL_fbvideo.h"

/* ELO */
#define ELO_PACKET_SIZE	10
#define ELO_START_BYTE		'U'

/*	eloConvertXY
	Convert the raw coordinates from the ELO controller
	to a screen position.
*/
void eloConvertXY(_THIS, int *dx,  int *dy);

/*	eloInitController(int fd)
	Initialize the ELO serial touchscreen controller
*/
int eloInitController(int fd);

/*	eloParsePacket
	extract position and button state from a packet
*/
int eloParsePacket(unsigned char* mousebuf, int* dx, int* dy, int* button_state);

/*	eloReadPosition
	read a packet and get the cursor position
*/

int eloReadPosition(_THIS, int fd, int* x, int* y, int* button_state, int* realx, int* realy);

#endif	/* SDL_fbelo_h */
