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
	Milan Xbios video functions

	Patrice Mandin
*/

#ifndef _SDL_xbios_milan_h
#define _SDL_xbios_milan_h

#include "SDL_xbios.h"

/*--- Defines ---*/

/* Vsetscreen() parameters */
#define MI_MAGIC	0x4D49 

enum {
	CMD_GETMODE=0,
	CMD_SETMODE,
	CMD_GETINFO,
	CMD_ALLOCPAGE,
	CMD_FREEPAGE,
	CMD_FLIPPAGE,
	CMD_ALLOCMEM,
	CMD_FREEMEM,
	CMD_SETADR,
	CMD_ENUMMODES
};

enum {
	ENUMMODE_EXIT=0,
	ENUMMODE_CONT
};

enum {
	BLK_ERR=0,
	BLK_OK,
	BLK_CLEARED
};

/* scrFlags */
#define SCRINFO_OK 1

/* scrClut */
#define NO_CLUT 0
#define HARD_CLUT 1
#define SOFT_CLUT 2

/* scrFormat */
#define INTERLEAVE_PLANES 0
#define STANDARD_PLANES  1
#define PACKEDPIX_PLANES 2

/* bitFlags */
#define STANDARD_BITS  1
#define FALCON_BITS   2
#define INTEL_BITS   8

/*--- Structures ---*/

typedef struct _scrblk { 
	unsigned long	size;		/* size of strukture */ 
	unsigned long	blk_status;	/* status bits of blk */ 
	unsigned long	blk_start;	/* Start Adress */ 
	unsigned long	blk_len;	/* length of memblk */ 
	unsigned long	blk_x;		/* x pos in total screen*/ 
	unsigned long	blk_y;		/* y pos in total screen */ 
	unsigned long	blk_w;		/* width */ 
	unsigned long	blk_h;		/* height */ 
	unsigned long	blk_wrap;	/* width in bytes */ 
} SCRMEMBLK;

typedef struct screeninfo { 
	unsigned long	size;		/* Size of structure */ 
	unsigned long	devID;		/* device id number */ 
	unsigned char	name[64];	/* Friendly name of Screen */ 
	unsigned long	scrFlags;	/* some Flags */ 
	unsigned long	frameadr;	/* Adress of framebuffer */ 
	unsigned long	scrHeight;	/* visible X res */ 
	unsigned long	scrWidth;	/* visible Y res */ 
	unsigned long	virtHeight;	/* virtual X res */ 
	unsigned long	virtWidth;	/* virtual Y res */ 
	unsigned long	scrPlanes;	/* color Planes */ 
	unsigned long	scrColors;	/* # of colors */ 
	unsigned long	lineWrap;	/* # of Bytes to next line */ 
	unsigned long	planeWarp;	/* # of Bytes to next plane */ 
	unsigned long	scrFormat;	/* screen Format */ 
	unsigned long	scrClut;	/* type of clut */ 
	unsigned long	redBits;	/* Mask of Red Bits */ 
	unsigned long	greenBits;	/* Mask of Green Bits */ 
	unsigned long	blueBits;	/* Mask of Blue Bits */ 
	unsigned long	alphaBits;	/* Mask of Alpha Bits */ 
	unsigned long	genlockBits;/* Mask of Genlock Bits */ 
	unsigned long	unusedBits;	/* Mask of unused Bits */ 
	unsigned long	bitFlags;	/* Bits organisation flags */ 
	unsigned long	maxmem;		/* max. memory in this mode */ 
	unsigned long	pagemem;	/* needed memory for one page */ 
	unsigned long	max_x;		/* max. possible width */ 
	unsigned long	max_y;		/* max. possible heigth */ 
} SCREENINFO; 

/*--- Functions prototypes ---*/

void SDL_XBIOS_ListMilanModes(_THIS, int actually_add);

#endif /* _SDL_xbios_milan_h */
