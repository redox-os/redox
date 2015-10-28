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
 * Xbios SDL video driver
 * 
 * Patrice Mandin
 */

#include <sys/stat.h>
#include <unistd.h>

/* Mint includes */
#include <mint/cookie.h>
#include <mint/osbind.h>
#include <mint/falcon.h>

#include "SDL_video.h"
#include "SDL_mouse.h"
#include "../SDL_sysvideo.h"
#include "../SDL_pixels_c.h"
#include "../../events/SDL_events_c.h"

#include "../ataricommon/SDL_ataric2p_s.h"
#include "../ataricommon/SDL_atarievents_c.h"
#include "../ataricommon/SDL_atarimxalloc_c.h"
#include "../ataricommon/SDL_atarigl_c.h"
#include "SDL_xbios.h"
#include "SDL_xbios_blowup.h"
#include "SDL_xbios_centscreen.h"
#include "SDL_xbios_sb3.h"
#include "SDL_xbios_tveille.h"
#include "SDL_xbios_milan.h"

#define XBIOS_VID_DRIVER_NAME "xbios"

#ifndef C_fVDI
#define C_fVDI 0x66564449L
#endif

/* Debug print info */
#if 0
#define DEBUG_PRINT(what) \
	{ \
		printf what; \
	}
#define DEBUG_VIDEO_XBIOS 1
#else
#define DEBUG_PRINT(what)
#undef DEBUG_VIDEO_XBIOS
#endif

/* Initialization/Query functions */
static int XBIOS_VideoInit(_THIS, SDL_PixelFormat *vformat);
static SDL_Rect **XBIOS_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags);
static SDL_Surface *XBIOS_SetVideoMode(_THIS, SDL_Surface *current, int width, int height, int bpp, Uint32 flags);
static int XBIOS_SetColors(_THIS, int firstcolor, int ncolors, SDL_Color *colors);
static void XBIOS_VideoQuit(_THIS);

/* Hardware surface functions */
static int XBIOS_AllocHWSurface(_THIS, SDL_Surface *surface);
static int XBIOS_LockHWSurface(_THIS, SDL_Surface *surface);
static int XBIOS_FlipHWSurface(_THIS, SDL_Surface *surface);
static void XBIOS_UnlockHWSurface(_THIS, SDL_Surface *surface);
static void XBIOS_FreeHWSurface(_THIS, SDL_Surface *surface);
static void XBIOS_UpdateRects(_THIS, int numrects, SDL_Rect *rects);

#if SDL_VIDEO_OPENGL
/* OpenGL functions */
static void XBIOS_GL_SwapBuffers(_THIS);
#endif

/* To setup palette */

static unsigned short	TT_palette[256];
static unsigned long	F30_palette[256];

/* Default list of video modes */

static const xbiosmode_t stmodes[1]={
	{ST_LOW>>8,320,200,4, XBIOSMODE_C2P}
};

static const xbiosmode_t ttmodes[2]={
	{TT_LOW,320,480,8, XBIOSMODE_C2P},
	{TT_LOW,320,240,8, XBIOSMODE_C2P|XBIOSMODE_DOUBLELINE}
};

static const xbiosmode_t falconrgbmodes[16]={
	{BPS16|COL80|OVERSCAN|VERTFLAG,768,480,16,0},
	{BPS16|COL80|OVERSCAN,768,240,16,0},
	{BPS16|COL80|VERTFLAG,640,400,16,0},
	{BPS16|COL80,640,200,16,0},
	{BPS16|OVERSCAN|VERTFLAG,384,480,16,0},
	{BPS16|OVERSCAN,384,240,16,0},
	{BPS16|VERTFLAG,320,400,16,0},
	{BPS16,320,200,16,0},
	{BPS8|COL80|OVERSCAN|VERTFLAG,768,480,8,XBIOSMODE_C2P},
	{BPS8|COL80|OVERSCAN,768,240,8,XBIOSMODE_C2P},
	{BPS8|COL80|VERTFLAG,640,400,8,XBIOSMODE_C2P},
	{BPS8|COL80,640,200,8,XBIOSMODE_C2P},
	{BPS8|OVERSCAN|VERTFLAG,384,480,8,XBIOSMODE_C2P},
	{BPS8|OVERSCAN,384,240,8,XBIOSMODE_C2P},
	{BPS8|VERTFLAG,320,400,8,XBIOSMODE_C2P},
	{BPS8,320,200,8,XBIOSMODE_C2P}
};

static const xbiosmode_t falconvgamodes[6]={
	{BPS16,320,480,16,0},
	{BPS16|VERTFLAG,320,240,16,0},
	{BPS8|COL80,640,480,8,XBIOSMODE_C2P},
	{BPS8|COL80|VERTFLAG,640,240,8,XBIOSMODE_C2P},
	{BPS8,320,480,8,XBIOSMODE_C2P},
	{BPS8|VERTFLAG,320,240,8,XBIOSMODE_C2P}
};

/* Xbios driver bootstrap functions */

static int XBIOS_Available(void)
{
	long cookie_vdo, /*cookie_mil,*/ cookie_hade, cookie_scpn;
	long cookie_fvdi;
	const char *envr = SDL_getenv("SDL_VIDEODRIVER");

	/* Milan/Hades Atari clones do not have an Atari video chip */
	if ( /*(Getcookie(C__MIL, &cookie_mil) == C_FOUND) ||*/
		(Getcookie(C_hade, &cookie_hade) == C_FOUND) ) {
		return 0;
	}

	/* fVDI means graphic card, so no Xbios with it */
	if (Getcookie(C_fVDI, &cookie_fvdi) == C_FOUND) {
		if (!envr) {
			return 0;
		}
		if (SDL_strcmp(envr, XBIOS_VID_DRIVER_NAME)!=0) {
			return 0;
		}
		/* Except if we force Xbios usage, through env var */
	}

	/* Cookie _VDO present ? if not, assume ST machine */
	if (Getcookie(C__VDO, &cookie_vdo) != C_FOUND) {
		cookie_vdo = VDO_ST << 16;
	}

	/* Test if we have a monochrome monitor plugged in */
	switch( cookie_vdo >>16) {
		case VDO_ST:
		case VDO_STE:
			if ( Getrez() == (ST_HIGH>>8) )
				return 0;
			break;
		case VDO_TT:
			if ( (EgetShift() & ES_MODE) == TT_HIGH)
				return 0;
			break;
		case VDO_F30:
			if ( VgetMonitor() == MONITOR_MONO)
				return 0;
			if (Getcookie(C_SCPN, &cookie_scpn) == C_FOUND) {
				if (!SDL_XBIOS_SB3Usable((scpn_cookie_t *)cookie_scpn)) {
					return 0;
				}
			}
			break;
		case VDO_MILAN:
			break;
		default:
			return 0;
	}

	return 1;
}

static void XBIOS_DeleteDevice(SDL_VideoDevice *device)
{
	SDL_free(device->hidden);
	SDL_free(device);
}

static SDL_VideoDevice *XBIOS_CreateDevice(int devindex)
{
	SDL_VideoDevice *device;

	/* Initialize all variables that we clean on shutdown */
	device = (SDL_VideoDevice *)SDL_malloc(sizeof(SDL_VideoDevice));
	if ( device ) {
		SDL_memset(device, 0, (sizeof *device));
		device->hidden = (struct SDL_PrivateVideoData *)
				SDL_malloc((sizeof *device->hidden));
		device->gl_data = (struct SDL_PrivateGLData *)
				SDL_malloc((sizeof *device->gl_data));
	}
	if ( (device == NULL) || (device->hidden == NULL) ) {
		SDL_OutOfMemory();
		if ( device ) {
			SDL_free(device);
		}
		return(0);
	}
	SDL_memset(device->hidden, 0, (sizeof *device->hidden));
	SDL_memset(device->gl_data, 0, sizeof(*device->gl_data));

	/* Video functions */
	device->VideoInit = XBIOS_VideoInit;
	device->ListModes = XBIOS_ListModes;
	device->SetVideoMode = XBIOS_SetVideoMode;
	device->SetColors = XBIOS_SetColors;
	device->UpdateRects = NULL;
	device->VideoQuit = XBIOS_VideoQuit;
	device->AllocHWSurface = XBIOS_AllocHWSurface;
	device->LockHWSurface = XBIOS_LockHWSurface;
	device->UnlockHWSurface = XBIOS_UnlockHWSurface;
	device->FlipHWSurface = XBIOS_FlipHWSurface;
	device->FreeHWSurface = XBIOS_FreeHWSurface;

#if SDL_VIDEO_OPENGL
	/* OpenGL functions */
	device->GL_LoadLibrary = SDL_AtariGL_LoadLibrary;
	device->GL_GetProcAddress = SDL_AtariGL_GetProcAddress;
	device->GL_GetAttribute = SDL_AtariGL_GetAttribute;
	device->GL_MakeCurrent = SDL_AtariGL_MakeCurrent;
	device->GL_SwapBuffers = XBIOS_GL_SwapBuffers;
#endif

	/* Events */
	device->InitOSKeymap = Atari_InitOSKeymap;
	device->PumpEvents = Atari_PumpEvents;

	device->free = XBIOS_DeleteDevice;

	return device;
}

VideoBootStrap XBIOS_bootstrap = {
	XBIOS_VID_DRIVER_NAME, "Atari Xbios driver",
	XBIOS_Available, XBIOS_CreateDevice
};

void SDL_XBIOS_AddMode(_THIS, int actually_add, const xbiosmode_t *modeinfo)
{
	int i = 0;

	switch(modeinfo->depth) {
		case 15:
		case 16:
			i = 1;
			break;
		case 24:
			i = 2;
			break;
		case 32:
			i = 3;
			break;
	}

	if ( actually_add ) {
		SDL_Rect saved_rect[2];
		xbiosmode_t saved_mode[2];
		int b, j;

		/* Add the mode, sorted largest to smallest */
		b = 0;
		j = 0;
		while ( (SDL_modelist[i][j]->w > modeinfo->width) ||
			(SDL_modelist[i][j]->h > modeinfo->height) ) {
			++j;
		}
		/* Skip modes that are already in our list */
		if ( (SDL_modelist[i][j]->w == modeinfo->width) &&
		     (SDL_modelist[i][j]->h == modeinfo->height) ) {
			return;
		}
		/* Insert the new mode */
		saved_rect[b] = *SDL_modelist[i][j];
		SDL_memcpy(&saved_mode[b], SDL_xbiosmode[i][j], sizeof(xbiosmode_t));
		SDL_modelist[i][j]->w = modeinfo->width;
		SDL_modelist[i][j]->h = modeinfo->height;
		SDL_memcpy(SDL_xbiosmode[i][j], modeinfo, sizeof(xbiosmode_t));
		/* Everybody scoot down! */
		if ( saved_rect[b].w && saved_rect[b].h ) {
		    for ( ++j; SDL_modelist[i][j]->w; ++j ) {
			saved_rect[!b] = *SDL_modelist[i][j];
			memcpy(&saved_mode[!b], SDL_xbiosmode[i][j], sizeof(xbiosmode_t));
			*SDL_modelist[i][j] = saved_rect[b];
			SDL_memcpy(SDL_xbiosmode[i][j], &saved_mode[b], sizeof(xbiosmode_t));
			b = !b;
		    }
		    *SDL_modelist[i][j] = saved_rect[b];
		    SDL_memcpy(SDL_xbiosmode[i][j], &saved_mode[b], sizeof(xbiosmode_t));
		}
	} else {
		++SDL_nummodes[i];
	}
}

static void XBIOS_ListSTModes(_THIS, int actually_add)
{
	SDL_XBIOS_AddMode(this, actually_add, &stmodes[0]);
}

static void XBIOS_ListTTModes(_THIS, int actually_add)
{
	int i;

	for (i=0; i<2; i++) {
		SDL_XBIOS_AddMode(this, actually_add, &ttmodes[i]);
	}
}

static void XBIOS_ListFalconRgbModes(_THIS, int actually_add)
{
	int i;

	for (i=0; i<16; i++) {
		xbiosmode_t modeinfo;

		SDL_memcpy(&modeinfo, &falconrgbmodes[i], sizeof(xbiosmode_t));
		modeinfo.number &= ~(VGA|PAL);
		modeinfo.number |= XBIOS_oldvmode & (VGA|PAL);

		SDL_XBIOS_AddMode(this, actually_add, &modeinfo);
	}
}

static void XBIOS_ListFalconVgaModes(_THIS, int actually_add)
{
	int i;

	for (i=0; i<6; i++) {
		xbiosmode_t modeinfo;

		SDL_memcpy(&modeinfo, &falconvgamodes[i], sizeof(xbiosmode_t));
		modeinfo.number &= ~(VGA|PAL);
		modeinfo.number |= XBIOS_oldvmode & (VGA|PAL);

		SDL_XBIOS_AddMode(this, actually_add, &modeinfo);
	}
}

static int XBIOS_VideoInit(_THIS, SDL_PixelFormat *vformat)
{
	int i;
	long cookie_blow, cookie_scpn, cookie_cnts;

	/* Initialize all variables that we clean on shutdown */
	for ( i=0; i<NUM_MODELISTS; ++i ) {
		SDL_nummodes[i] = 0;
		SDL_modelist[i] = NULL;
		SDL_xbiosmode[i] = NULL;
	}

	/* Cookie _VDO present ? if not, assume ST machine */
	if (Getcookie(C__VDO, &XBIOS_cvdo) != C_FOUND) {
		XBIOS_cvdo = VDO_ST << 16;
	}

	/* Allocate memory for old palette */
	XBIOS_oldpalette = (void *)SDL_malloc(256*sizeof(long));
	if ( !XBIOS_oldpalette ) {
		SDL_SetError("Unable to allocate memory for old palette\n");
		return(-1);
	}

	/* Initialize video mode list */
	/* and save current screen status (palette, screen address, video mode) */
	XBIOS_centscreen = SDL_FALSE;
	XBIOS_oldvbase = Physbase();

	/* Determine the current screen size */
	this->info.current_w = 0;
	this->info.current_h = 0;

	/* Determine the screen depth (use default 8-bit depth) */
	vformat->BitsPerPixel = 8;

	/* First allocate room for needed video modes */
	switch (XBIOS_cvdo >>16) {
		case VDO_ST:
		case VDO_STE:
			{
				short *oldpalette;
			
				XBIOS_oldvmode=Getrez();
				switch(XBIOS_oldvmode << 8) {
					case ST_LOW:
						XBIOS_oldnumcol=16;
						break;
					case ST_MED:
						XBIOS_oldnumcol=4;
						break;
					case ST_HIGH:
						XBIOS_oldnumcol=2;
						break;
				}

				oldpalette= (short *) XBIOS_oldpalette;
				for (i=0;i<XBIOS_oldnumcol;i++) {
					*oldpalette++=Setcolor(i,-1);
				}

				XBIOS_ListSTModes(this, 0);
			}
			break;
		case VDO_TT:
			XBIOS_oldvmode=EgetShift();

			switch(XBIOS_oldvmode & ES_MODE) {
				case TT_LOW:
					XBIOS_oldnumcol=256;
					break;
				case ST_LOW:
				case TT_MED:
					XBIOS_oldnumcol=16;
					break;
				case ST_MED:
					XBIOS_oldnumcol=4;
					break;
				case ST_HIGH:
				case TT_HIGH:
					XBIOS_oldnumcol=2;
					break;
			}
			if (XBIOS_oldnumcol) {
				EgetPalette(0, XBIOS_oldnumcol, XBIOS_oldpalette);
			}

			XBIOS_ListTTModes(this, 0);
			break;
		case VDO_F30:
			XBIOS_oldvmode=VsetMode(-1);

			XBIOS_oldnumcol= 1<< (1 << (XBIOS_oldvmode & NUMCOLS));
			if (XBIOS_oldnumcol > 256) {
				XBIOS_oldnumcol = 0;
			}
			if (XBIOS_oldnumcol) {
				VgetRGB(0, XBIOS_oldnumcol, XBIOS_oldpalette);
			}

			vformat->BitsPerPixel = 16;

			/* ScreenBlaster 3 ? */
			if (Getcookie(C_SCPN, &cookie_scpn) == C_FOUND) {
				SDL_XBIOS_ListSB3Modes(this, 0, (scpn_cookie_t *)cookie_scpn);
			} else
			/* Centscreen ? */
			if (Getcookie(C_CNTS, &cookie_cnts) == C_FOUND) {
				XBIOS_oldvmode = SDL_XBIOS_ListCentscreenModes(this, 0);
				XBIOS_centscreen = SDL_TRUE;
			} else
			/* Standard, with or without Blowup */
			{
				switch (VgetMonitor())
				{
					case MONITOR_RGB:
					case MONITOR_TV:
						XBIOS_ListFalconRgbModes(this, 0);
						break;
					case MONITOR_VGA:
						XBIOS_ListFalconVgaModes(this, 0);
						break;
				}

				if (Getcookie(C_BLOW, &cookie_blow) == C_FOUND) {
					SDL_XBIOS_ListBlowupModes(this, 0, (blow_cookie_t *)cookie_blow);
				}
			}
			break;
		case VDO_MILAN:
			{
				SCREENINFO si;

				/* Read infos about current mode */ 
				VsetScreen(-1, &XBIOS_oldvmode, MI_MAGIC, CMD_GETMODE);

				si.size = sizeof(SCREENINFO);
				si.devID = XBIOS_oldvmode;
				si.scrFlags = 0;
				VsetScreen(-1, &si, MI_MAGIC, CMD_GETINFO);

				this->info.current_w = si.scrWidth;
				this->info.current_h = si.scrHeight;

				XBIOS_oldnumcol = 0;
				if (si.scrFlags & SCRINFO_OK) {
					if (si.scrPlanes <= 8) {
						XBIOS_oldnumcol = 1<<si.scrPlanes;
					}
				}
				if (XBIOS_oldnumcol) {
					VgetRGB(0, XBIOS_oldnumcol, XBIOS_oldpalette);
				}

				SDL_XBIOS_ListMilanModes(this, 0);
			}
			break;
	}

	for ( i=0; i<NUM_MODELISTS; ++i ) {
		int j;

		SDL_xbiosmode[i] = (xbiosmode_t **)
			SDL_malloc((SDL_nummodes[i]+1)*sizeof(xbiosmode_t *));
		if ( SDL_xbiosmode[i] == NULL ) {
			SDL_OutOfMemory();
			return(-1);
		}
		for ( j=0; j<SDL_nummodes[i]; ++j ) {
			SDL_xbiosmode[i][j]=(xbiosmode_t *)SDL_malloc(sizeof(xbiosmode_t));
			if ( SDL_xbiosmode[i][j] == NULL ) {
				SDL_OutOfMemory();
				return(-1);
			}
			SDL_memset(SDL_xbiosmode[i][j], 0, sizeof(xbiosmode_t));
		}
		SDL_xbiosmode[i][j] = NULL;

		SDL_modelist[i] = (SDL_Rect **)
				SDL_malloc((SDL_nummodes[i]+1)*sizeof(SDL_Rect *));
		if ( SDL_modelist[i] == NULL ) {
			SDL_OutOfMemory();
			return(-1);
		}
		for ( j=0; j<SDL_nummodes[i]; ++j ) {
			SDL_modelist[i][j]=(SDL_Rect *)SDL_malloc(sizeof(SDL_Rect));
			if ( SDL_modelist[i][j] == NULL ) {
				SDL_OutOfMemory();
				return(-1);
			}
			SDL_memset(SDL_modelist[i][j], 0, sizeof(SDL_Rect));
		}
		SDL_modelist[i][j] = NULL;
	}

	/* Now fill the mode list */
	switch (XBIOS_cvdo >>16) {
		case VDO_ST:
		case VDO_STE:
			XBIOS_ListSTModes(this, 1);
			break;
		case VDO_TT:
			XBIOS_ListTTModes(this, 1);
			break;
		case VDO_F30:
			/* ScreenBlaster 3 ? */
			if (Getcookie(C_SCPN, &cookie_scpn) == C_FOUND) {
				SDL_XBIOS_ListSB3Modes(this, 1, (scpn_cookie_t *)cookie_scpn);
			} else
			/* Centscreen ? */
			if (Getcookie(C_CNTS, &cookie_cnts) == C_FOUND) {
				XBIOS_oldvmode = SDL_XBIOS_ListCentscreenModes(this, 1);
				XBIOS_centscreen = SDL_TRUE;
			} else
			/* Standard, with or without Blowup */
			{
				switch (VgetMonitor())
				{
					case MONITOR_RGB:
					case MONITOR_TV:
						XBIOS_ListFalconRgbModes(this, 1);
						break;
					case MONITOR_VGA:
						XBIOS_ListFalconVgaModes(this, 1);
						break;
				}

				if (Getcookie(C_BLOW, &cookie_blow) == C_FOUND) {
					SDL_XBIOS_ListBlowupModes(this, 1, (blow_cookie_t *)cookie_blow);
				}
			}
			break;
		case VDO_MILAN:
			SDL_XBIOS_ListMilanModes(this, 1);
			break;
	}

	XBIOS_screens[0]=NULL;
	XBIOS_screens[1]=NULL;
	XBIOS_shadowscreen=NULL;

	/* Update hardware info */
	this->info.hw_available = 1;
	this->info.video_mem = (Uint32) Atari_SysMalloc(-1L, MX_STRAM);

	/* Init chunky to planar routine */
	SDL_Atari_C2pConvert = SDL_Atari_C2pConvert8;

#if SDL_VIDEO_OPENGL
	SDL_AtariGL_InitPointers(this);
#endif

	/* Disable screensavers */
	if (SDL_XBIOS_TveillePresent(this)) {
		SDL_XBIOS_TveilleDisable(this);
	}

	/* We're done! */
	return(0);
}

static SDL_Rect **XBIOS_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags)
{
	return(SDL_modelist[((format->BitsPerPixel+7)/8)-1]);
}

static void XBIOS_FreeBuffers(_THIS)
{
	int i;

	for (i=0;i<2;i++) {
		if (XBIOS_screensmem[i]!=NULL) {
			if ((XBIOS_cvdo>>16) == VDO_MILAN) {
				if (i==1) {
					VsetScreen(-1, -1, MI_MAGIC, CMD_FREEPAGE);
				}
			} else {
				Mfree(XBIOS_screensmem[i]);
			}
			XBIOS_screensmem[i]=NULL;
		}
	}

	if (XBIOS_shadowscreen!=NULL) {
		Mfree(XBIOS_shadowscreen);
		XBIOS_shadowscreen=NULL;
	}
}

static SDL_Surface *XBIOS_SetVideoMode(_THIS, SDL_Surface *current,
				int width, int height, int bpp, Uint32 flags)
{
	int mode, new_depth;
	int i, num_buffers;
	xbiosmode_t *new_video_mode;
	Uint32 new_screen_size;
	Uint32 modeflags;

	/* Free current buffers */
	XBIOS_FreeBuffers(this);

	/* Try to set the requested linear video mode */
	bpp = (bpp+7)/8-1;
	for ( mode=0; SDL_modelist[bpp][mode]; ++mode ) {
		if ( (SDL_modelist[bpp][mode]->w == width) &&
		     (SDL_modelist[bpp][mode]->h == height) ) {
			break;
		}
	}
	if ( SDL_modelist[bpp][mode] == NULL ) {
		SDL_SetError("Couldn't find requested mode in list");
		return(NULL);
	}
	new_video_mode = SDL_xbiosmode[bpp][mode];

	modeflags = SDL_FULLSCREEN | SDL_PREALLOC;

	/* Allocate needed buffers: simple/double buffer and shadow surface */
	new_depth = new_video_mode->depth;
	if (new_depth == 4) {
		SDL_Atari_C2pConvert = SDL_Atari_C2pConvert4;
		new_depth=8;
		modeflags |= SDL_SWSURFACE|SDL_HWPALETTE;
	} else if (new_depth == 8) {
		SDL_Atari_C2pConvert = SDL_Atari_C2pConvert8;
		modeflags |= SDL_SWSURFACE|SDL_HWPALETTE;
	} else {
		modeflags |= SDL_HWSURFACE;
	}

	new_screen_size = width * height * ((new_depth)>>3);
	new_screen_size += 256; /* To align on a 256 byte adress */	

	if (new_video_mode->flags & XBIOSMODE_C2P) {
		XBIOS_shadowscreen = Atari_SysMalloc(new_screen_size, MX_PREFTTRAM);

		if (XBIOS_shadowscreen == NULL) {
			SDL_SetError("Can not allocate %d KB for shadow buffer", new_screen_size>>10);
			return (NULL);
		}
		SDL_memset(XBIOS_shadowscreen, 0, new_screen_size);
	}

	/* Output buffer needs to be twice in size for the software double-line mode */
	if (new_video_mode->flags & XBIOSMODE_DOUBLELINE) {
		new_screen_size <<= 1;
	}

	/* Double buffer ? */
	num_buffers = 1;

#if SDL_VIDEO_OPENGL
	if (flags & SDL_OPENGL) {
		if (this->gl_config.double_buffer) {
			flags |= SDL_DOUBLEBUF;
		}
	}
#endif
	if ((flags & SDL_DOUBLEBUF) && ((XBIOS_cvdo>>16) != VDO_MILAN)) {
		num_buffers = 2;
		modeflags |= SDL_DOUBLEBUF;
	}

	/* Allocate buffers */
	for (i=0; i<num_buffers; i++) {
		if ((XBIOS_cvdo>>16) == VDO_MILAN) {
			if (i==0) {
				XBIOS_screensmem[i] = XBIOS_oldvbase;
			} else {
				VsetScreen(-1, &XBIOS_screensmem[i], MI_MAGIC, CMD_ALLOCPAGE);
			}
		} else {
			XBIOS_screensmem[i] = Atari_SysMalloc(new_screen_size, MX_STRAM);
		}

		if (XBIOS_screensmem[i]==NULL) {
			XBIOS_FreeBuffers(this);
			SDL_SetError("Can not allocate %d KB for buffer %d", new_screen_size>>10, i);
			return (NULL);
		}
		SDL_memset(XBIOS_screensmem[i], 0, new_screen_size);

		XBIOS_screens[i]=(void *) (( (long) XBIOS_screensmem[i]+256) & 0xFFFFFF00UL);
	}

	/* Allocate the new pixel format for the screen */
	if ( ! SDL_ReallocFormat(current, new_depth, 0, 0, 0, 0) ) {
		XBIOS_FreeBuffers(this);
		SDL_SetError("Couldn't allocate new pixel format for requested mode");
		return(NULL);
	}

	XBIOS_current = new_video_mode;
	current->w = width;
	current->h = height;
	current->pitch = (width * new_depth)>>3;

	/* this is for C2P conversion */
	XBIOS_pitch = (new_video_mode->width * new_video_mode->depth)>>3;

	if (new_video_mode->flags & XBIOSMODE_C2P)
		current->pixels = XBIOS_shadowscreen;
	else
		current->pixels = XBIOS_screens[0];

	XBIOS_fbnum = 0;

#if SDL_VIDEO_OPENGL
	if (flags & SDL_OPENGL) {
		if (!SDL_AtariGL_Init(this, current)) {
			XBIOS_FreeBuffers(this);
			SDL_SetError("Can not create OpenGL context");
			return NULL;
		}

		modeflags |= SDL_OPENGL;
	}
#endif

	current->flags = modeflags;

#ifndef DEBUG_VIDEO_XBIOS
	/* Now set the video mode */
	if ((XBIOS_cvdo>>16) == VDO_MILAN) {
		VsetScreen(-1, XBIOS_screens[0], MI_MAGIC, CMD_SETADR);
	} else {
		Setscreen(-1,XBIOS_screens[0],-1);
	}

	switch(XBIOS_cvdo >> 16) {
		case VDO_ST:
			Setscreen(-1,-1,new_video_mode->number);

			/* Reset palette */
			for (i=0;i<16;i++) {
				TT_palette[i]= ((i>>1)<<8) | (((i*8)/17)<<4) | (i>>1);
			}
			Setpalette(TT_palette);
			break;
		case VDO_STE:
			Setscreen(-1,-1,new_video_mode->number);

			/* Reset palette */
			for (i=0;i<16;i++)
			{
				int c;

				c=((i&1)<<3)|((i>>1)&7);
				TT_palette[i]=(c<<8)|(c<<4)|c;
			}
			Setpalette(TT_palette);
			break;
		case VDO_TT:
			EsetShift(new_video_mode->number);
			break;
		case VDO_F30:
			if (XBIOS_centscreen) {
				SDL_XBIOS_CentscreenSetmode(this, width, height, new_depth);
			} else {
				VsetMode(new_video_mode->number);
			}

			/* Set hardware palette to black in True Colour */
			if (new_depth > 8) {
				SDL_memset(F30_palette, 0, sizeof(F30_palette));
				VsetRGB(0,256,F30_palette);
			}
			break;
		case VDO_MILAN:
			VsetScreen(-1, new_video_mode->number, MI_MAGIC, CMD_SETMODE);

			/* Set hardware palette to black in True Colour */
			if (new_depth > 8) {
				SDL_memset(F30_palette, 0, sizeof(F30_palette));
				VsetRGB(0,256,F30_palette);
			}
			break;
	}

	Vsync();
#endif

	this->UpdateRects = XBIOS_UpdateRects;

	return (current);
}

/* We don't actually allow hardware surfaces other than the main one */
static int XBIOS_AllocHWSurface(_THIS, SDL_Surface *surface)
{
	return(-1);
}

static void XBIOS_FreeHWSurface(_THIS, SDL_Surface *surface)
{
	return;
}

static int XBIOS_LockHWSurface(_THIS, SDL_Surface *surface)
{
	return(0);
}

static void XBIOS_UnlockHWSurface(_THIS, SDL_Surface *surface)
{
	return;
}

static void XBIOS_UpdateRects(_THIS, int numrects, SDL_Rect *rects)
{
	SDL_Surface *surface;

	surface = this->screen;

	if (XBIOS_current->flags & XBIOSMODE_C2P) {
		int i;
		int doubleline = (XBIOS_current->flags & XBIOSMODE_DOUBLELINE ? 1 : 0);

		for (i=0;i<numrects;i++) {
			void *source,*destination;
			int x1,x2;

			x1 = rects[i].x & ~15;
			x2 = rects[i].x+rects[i].w;
			if (x2 & 15) {
				x2 = (x2 | 15) +1;
			}

			source = surface->pixels;
			source += surface->pitch * rects[i].y;
			source += x1;

			destination = XBIOS_screens[XBIOS_fbnum];
			destination += XBIOS_pitch * rects[i].y;
			destination += x1;

			/* Convert chunky to planar screen */
			SDL_Atari_C2pConvert(
				source,
				destination,
				x2-x1,
				rects[i].h,
				doubleline,
				surface->pitch,
				XBIOS_pitch
			);
		}
	}

#ifndef DEBUG_VIDEO_XBIOS
	if ((XBIOS_cvdo>>16) == VDO_MILAN) {
		VsetScreen(-1, XBIOS_screens[XBIOS_fbnum], MI_MAGIC, CMD_SETADR);
	} else {
		Setscreen(-1,XBIOS_screens[XBIOS_fbnum],-1);
	}

	Vsync();
#endif

	if ((surface->flags & SDL_DOUBLEBUF) == SDL_DOUBLEBUF) {
		XBIOS_fbnum ^= 1;
		if ((XBIOS_current->flags & XBIOSMODE_C2P) == 0) {
			surface->pixels=XBIOS_screens[XBIOS_fbnum];
		}
	}
}

static int XBIOS_FlipHWSurface(_THIS, SDL_Surface *surface)
{
	if (XBIOS_current->flags & XBIOSMODE_C2P) {
		void *destscr;
		int destx;
		int doubleline = (XBIOS_current->flags & XBIOSMODE_DOUBLELINE ? 1 : 0);
			
		/* Center on destination screen */
		destscr = XBIOS_screens[XBIOS_fbnum];
		destscr += XBIOS_pitch * ((XBIOS_current->height - surface->h) >> 1);
		destx = (XBIOS_current->width - surface->w) >> 1;
		destx &= ~15;
		destscr += destx;

		/* Convert chunky to planar screen */
		SDL_Atari_C2pConvert(
			surface->pixels,
			destscr,
			surface->w,
			surface->h,
			doubleline,
			surface->pitch,
			XBIOS_pitch
		);
	}

#ifndef DEBUG_VIDEO_XBIOS
	if ((XBIOS_cvdo>>16) == VDO_MILAN) {
		VsetScreen(-1, XBIOS_screens[XBIOS_fbnum], MI_MAGIC, CMD_SETADR);
	} else {
		Setscreen(-1,XBIOS_screens[XBIOS_fbnum],-1);
	}

	Vsync();
#endif

	if ((surface->flags & SDL_DOUBLEBUF) == SDL_DOUBLEBUF) {
		XBIOS_fbnum ^= 1;
		if ((XBIOS_current->flags & XBIOSMODE_C2P) == 0) {
			surface->pixels=XBIOS_screens[XBIOS_fbnum];
		}
	}

	return(0);
}

static int XBIOS_SetColors(_THIS, int firstcolor, int ncolors, SDL_Color *colors)
{
#ifndef DEBUG_VIDEO_XBIOS
	int		i;
	int		r,v,b;

	switch( XBIOS_cvdo >> 16) {
		case VDO_ST:
		case VDO_STE:
		 	for (i=0;i<ncolors;i++)
			{
				r = colors[i].r;	
				v = colors[i].g;
				b = colors[i].b;

				TT_palette[firstcolor+i]=((r*30)+(v*59)+(b*11))/100;
			}
			SDL_Atari_C2pConvert4_pal(TT_palette); /* convert the lighting */
			break;
		case VDO_TT:
			for(i = 0; i < ncolors; i++)
			{
				r = colors[i].r;	
				v = colors[i].g;
				b = colors[i].b;
					
				TT_palette[i]=((r>>4)<<8)|((v>>4)<<4)|(b>>4);
			}
			EsetPalette(firstcolor,ncolors,TT_palette);
			break;
		case VDO_F30:
		case VDO_MILAN:
			for(i = 0; i < ncolors; i++)
			{
				r = colors[i].r;	
				v = colors[i].g;
				b = colors[i].b;

				F30_palette[i]=(r<<16)|(v<<8)|b;
			}
			VsetRGB(firstcolor,ncolors,F30_palette);
			break;
	}
#endif

	return(1);
}

/* Note:  If we are terminated, this could be called in the middle of
   another SDL video routine -- notably UpdateRects.
*/
static void XBIOS_VideoQuit(_THIS)
{
	int i,j;

	Atari_ShutdownEvents();

	/* Restore video mode and palette */
#ifndef DEBUG_VIDEO_XBIOS
	switch(XBIOS_cvdo >> 16) {
		case VDO_ST:
		case VDO_STE:
			Setscreen(-1,XBIOS_oldvbase,XBIOS_oldvmode);
			if (XBIOS_oldnumcol) {
				Setpalette(XBIOS_oldpalette);
			}
			break;
		case VDO_TT:
			Setscreen(-1,XBIOS_oldvbase,-1);
			EsetShift(XBIOS_oldvmode);
			if (XBIOS_oldnumcol) {
				EsetPalette(0, XBIOS_oldnumcol, XBIOS_oldpalette);
			}
			break;
		case VDO_F30:
			Setscreen(-1, XBIOS_oldvbase, -1);
			if (XBIOS_centscreen) {
				SDL_XBIOS_CentscreenRestore(this, XBIOS_oldvmode);
			} else {
				VsetMode(XBIOS_oldvmode);
			}
			if (XBIOS_oldnumcol) {
				VsetRGB(0, XBIOS_oldnumcol, XBIOS_oldpalette);
			}
			break;
		case VDO_MILAN:
			VsetScreen(-1, &XBIOS_oldvbase, MI_MAGIC, CMD_SETADR);
			VsetScreen(-1, &XBIOS_oldvmode, MI_MAGIC, CMD_SETMODE);
			if (XBIOS_oldnumcol) {
				VsetRGB(0, XBIOS_oldnumcol, XBIOS_oldpalette);
			}
			break;
	}
	Vsync();
#endif

#if SDL_VIDEO_OPENGL
	if (gl_active) {
		SDL_AtariGL_Quit(this, SDL_TRUE);
	}
#endif

	if (XBIOS_oldpalette) {
		SDL_free(XBIOS_oldpalette);
		XBIOS_oldpalette=NULL;
	}
	XBIOS_FreeBuffers(this);

	/* Free mode list */
	for ( i=0; i<NUM_MODELISTS; ++i ) {
		if ( SDL_modelist[i] != NULL ) {
			for ( j=0; SDL_modelist[i][j]; ++j )
				SDL_free(SDL_modelist[i][j]);
			SDL_free(SDL_modelist[i]);
			SDL_modelist[i] = NULL;
		}
		if ( SDL_xbiosmode[i] != NULL ) {
			for ( j=0; SDL_xbiosmode[i][j]; ++j )
				SDL_free(SDL_xbiosmode[i][j]);
			SDL_free(SDL_xbiosmode[i]);
			SDL_xbiosmode[i] = NULL;
		}
	}

	this->screen->pixels = NULL;	

	/* Restore screensavers */
	if (SDL_XBIOS_TveillePresent(this)) {
		SDL_XBIOS_TveilleEnable(this);
	}
}

#if SDL_VIDEO_OPENGL

static void XBIOS_GL_SwapBuffers(_THIS)
{
	SDL_AtariGL_SwapBuffers(this);
	XBIOS_FlipHWSurface(this, this->screen);
	SDL_AtariGL_MakeCurrent(this);
}

#endif
