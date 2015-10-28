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

#ifndef _RIVA_REGS_H
#define _RIVA_REGS_H

/* This information comes from the XFree86 NVidia hardware driver */

/* mapped_io register offsets */
#define PGRAPH_OFFSET	0x00400000
#define FIFO_OFFSET	0x00800000
#define ROP_OFFSET	FIFO_OFFSET+0x00000000
#define CLIP_OFFSET	FIFO_OFFSET+0x00002000
#define PATT_OFFSET	FIFO_OFFSET+0x00004000
#define PIXMAP_OFFSET	FIFO_OFFSET+0x00006000
#define BLT_OFFSET	FIFO_OFFSET+0x00008000
#define BITMAP_OFFSET	FIFO_OFFSET+0x0000A000
#define LINE_OFFSET	FIFO_OFFSET+0x0000C000
#define TRI03_OFFSET	FIFO_OFFSET+0x0000E000
#define PCIO_OFFSET	0x00601000

#endif /* _RIVA_REGS_H */

