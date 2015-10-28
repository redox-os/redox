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
	Centscreen extension definitions

	Patrice Mandin
*/

#ifndef _SDL_xbios_centscreen_h
#define _SDL_xbios_centscreen_h

#include <mint/falcon.h>	/* for trap_14_xxx macros */

#include "SDL_xbios.h"

/*--- Defines ---*/

#define CSCREEN_ENERGYSTAR		(1<<9)
#define CSCREEN_SAVER			(1<<10)
#define CSCREEN_VIRTUAL			(1<<11)
#define CSCREEN_EXTCLOCK_CT2	(1<<12)
#define CSCREEN_EXTCLOCK		(1<<13)
#define CSCREEN_STANDARD		(1<<14)
#define CSCREEN_DEFAULT			(1<<15)

/*--- Structures ---*/

typedef struct {
	unsigned short	handle;	/* videomode handle */
	unsigned short	mode;	/* Falcon videomode code */
	unsigned short	physx;	/* visible width */
	unsigned short	physy;	/* visible height */
	unsigned short	plan;	/* bitplanes */
	unsigned short	logx;	/* virtual width */
	unsigned short	logy;	/* virtual height */
	unsigned short	eco;	/* screen saver delay */
	unsigned short	eco2;	/* energy star screen saver delay */
	unsigned short	wsize;	/* screen width (mm) */
	unsigned short	hsize;	/* screen height (mm) */
	unsigned short	dummy[21];
	unsigned char	name[32];	/* videomode name */
} centscreen_mode_t;

/*--- Functions prototypes ---*/

#define Vread(current_mode)	\
	(void)trap_14_wl((short)0x41,(long)(current_mode))
#define Vwrite(init_vdi, inparam, outparam)	\
	(long)trap_14_wwll((short)0x42,(short)(init_vdi),(long)(inparam),(long)(outparam))
#define Vattrib(inparam, outparam)	\
	(void)trap_14_wll((short)0x43,(long)(inparam),(long)(outparam))
#define Vcreate(inparam, outparam)	\
	(void)trap_14_wll((short)0x44,(long)(inparam),(long)(outparam))
#define Vdelete(handle)	\
	(long)trap_14_ww((short)0x45,(short)(handle))
#define Vfirst(mask,mode)	\
	(long)trap_14_wll((short)0x46,(long)(mask),(long)(mode))
#define Vnext(mask,mode)	\
	(long)trap_14_wll((short)0x47,(long)(mask),(long)(mode))
#define Vvalid(handle)	\
	(long)trap_14_ww((short)0x48,(short)(handle))
#define Vload()	\
	(long)trap_14_w((short)0x49)
#define Vsave()	\
	(long)trap_14_w((short)0x4a)
#define Vopen()	\
	(long)trap_14_w((short)0x4b)
#define Vclose()	\
	(long)trap_14_w((short)0x4c)
#define Vscroll(scrollmode)	\
	(long)trap_14_ww((short)0x4d,(short)(scrollmode))
#define Voffset()	\
	(long)trap_14_w((short)0x4e)
#define Vseek()	\
	(long)trap_14_w((short)0x4f)
#define Vlock(cmd)	\
	(long)trap_14_ww((short)0x50,(short)(cmd))
#define SetMon(montype)	\
	(long)trap_14_ww((short)0x51,(short)(montype))
#define MultiMon(cmd)	\
	(long)trap_14_ww((short)0x52,(short)(cmd))
#define VSizeComp()	\
	(long)trap_14_w((short)0x53)
#define Vsize(mode)	\
	(long)trap_14_wl((short)0x54,(long)(mode))

/*--- Functions prototypes ---*/

int SDL_XBIOS_ListCentscreenModes(_THIS, int actually_add);
void SDL_XBIOS_CentscreenSetmode(_THIS, int width, int height, int planes);
void SDL_XBIOS_CentscreenRestore(_THIS, int prev_handle);

#endif /* _SDL_xbios_centscreen_h */
