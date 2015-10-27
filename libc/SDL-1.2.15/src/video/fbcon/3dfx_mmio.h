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

/* 3Dfx register definitions */

#include "3dfx_regs.h"

/* 3Dfx control macros */

#define tdfx_in8(reg)		*(volatile Uint8  *)(mapped_io + (reg))
#define tdfx_in32(reg)		*(volatile Uint32 *)(mapped_io + (reg))

#define tdfx_out8(reg,v)	*(volatile Uint8  *)(mapped_io + (reg)) = v;
#define tdfx_out32(reg,v)	*(volatile Uint32 *)(mapped_io + (reg)) = v;


/* Wait for fifo space */
#define tdfx_wait(space)						\
{									\
	while ( (tdfx_in8(TDFX_STATUS) & 0x1F) < space )		\
		;							\
}


/* Wait for idle accelerator */
#define tdfx_waitidle()							\
{									\
	int i = 0;							\
									\
	tdfx_wait(1);							\
	tdfx_out32(COMMAND_3D, COMMAND_3D_NOP);				\
	do {								\
		i = (tdfx_in32(TDFX_STATUS) & STATUS_BUSY) ? 0 : i + 1;	\
	} while ( i != 3 );						\
}

