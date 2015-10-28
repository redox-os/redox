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
     File added by Alan Buckley (alan_baa@hotmail.com) for RISC OS compatability
	 23 March 2003

     Implements RISC OS display device management.
	 Routines for full screen and wimp modes are split
	 into other source files.
*/

#include "SDL_video.h"
#include "SDL_mouse.h"
#include "SDL_syswm.h"
#include "../SDL_sysvideo.h"
#include "../SDL_pixels_c.h"
#include "../../events/SDL_events_c.h"

#include "SDL_riscostask.h"
#include "SDL_riscosvideo.h"
#include "SDL_riscosevents_c.h"
#include "SDL_riscosmouse_c.h"

#include "kernel.h"
#include "swis.h"

#define RISCOSVID_DRIVER_NAME "riscos"

/* Initialization/Query functions */
static int RISCOS_VideoInit(_THIS, SDL_PixelFormat *vformat);
static void RISCOS_VideoQuit(_THIS);

static SDL_Rect **RISCOS_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags);
static SDL_Surface *RISCOS_SetVideoMode(_THIS, SDL_Surface *current, int width, int height, int bpp, Uint32 flags);

int RISCOS_GetWmInfo(_THIS, SDL_SysWMinfo *info);

int RISCOS_ToggleFullScreen(_THIS, int fullscreen);
/* Mouse checking */
void RISCOS_CheckMouseMode(_THIS);
extern SDL_GrabMode RISCOS_GrabInput(_THIS, SDL_GrabMode mode);

/* Fullscreen mode functions */
extern SDL_Surface *FULLSCREEN_SetVideoMode(_THIS, SDL_Surface *current, int width, int height, int bpp, Uint32 flags);
extern void FULLSCREEN_BuildModeList(_THIS);
extern void	FULLSCREEN_SetDeviceMode(_THIS);
extern int FULLSCREEN_ToggleFromWimp(_THIS);

/* Wimp mode functions */
extern SDL_Surface *WIMP_SetVideoMode(_THIS, SDL_Surface *current,	int width, int height, int bpp, Uint32 flags);
extern void WIMP_DeleteWindow(_THIS);
extern int WIMP_ToggleFromFullScreen(_THIS);

/* Hardware surface functions - common to WIMP and FULLSCREEN */
static int RISCOS_AllocHWSurface(_THIS, SDL_Surface *surface);
static int RISCOS_LockHWSurface(_THIS, SDL_Surface *surface);
static void RISCOS_UnlockHWSurface(_THIS, SDL_Surface *surface);
static void RISCOS_FreeHWSurface(_THIS, SDL_Surface *surface);

/* RISC OS driver bootstrap functions */

static int RISCOS_Available(void)
{
	return(1);
}

static void RISCOS_DeleteDevice(SDL_VideoDevice *device)
{
	SDL_free(device->hidden);
	SDL_free(device);
}

static SDL_VideoDevice *RISCOS_CreateDevice(int devindex)
{
	SDL_VideoDevice *device;

	/* Initialize all variables that we clean on shutdown */
	device = (SDL_VideoDevice *)SDL_malloc(sizeof(SDL_VideoDevice));
	if ( device ) {
		SDL_memset(device, 0, (sizeof *device));
		device->hidden = (struct SDL_PrivateVideoData *)
				SDL_malloc((sizeof *device->hidden));
	}
	if ( (device == NULL) || (device->hidden == NULL) ) {
		SDL_OutOfMemory();
		if ( device ) {
			SDL_free(device);
		}
		return(0);
	}
	SDL_memset(device->hidden, 0, (sizeof *device->hidden));

	/* Set the function pointers */
	device->VideoInit = RISCOS_VideoInit;
	device->VideoQuit = RISCOS_VideoQuit;

	device->ListModes = RISCOS_ListModes;
	device->SetVideoMode = RISCOS_SetVideoMode;
	device->CreateYUVOverlay = NULL;
	device->AllocHWSurface = RISCOS_AllocHWSurface;
	device->CheckHWBlit = NULL;
	device->FillHWRect = NULL;
	device->SetHWColorKey = NULL;
	device->SetHWAlpha = NULL;
	device->LockHWSurface = RISCOS_LockHWSurface;
	device->UnlockHWSurface = RISCOS_UnlockHWSurface;
	device->FreeHWSurface = RISCOS_FreeHWSurface;
	
	device->FreeWMCursor = RISCOS_FreeWMCursor;
	device->CreateWMCursor = RISCOS_CreateWMCursor;
	device->CheckMouseMode = RISCOS_CheckMouseMode;
    device->GrabInput = RISCOS_GrabInput;

	device->InitOSKeymap = RISCOS_InitOSKeymap;

	device->GetWMInfo = RISCOS_GetWmInfo;

	device->free = RISCOS_DeleteDevice;

/* Can't get Toggle screen to work if program starts up in Full screen mode so
   disable it here and re-enable it when a wimp screen is chosen */
    device->ToggleFullScreen = NULL; /*RISCOS_ToggleFullScreen;*/

	/* Set other entries for fullscreen mode */
	FULLSCREEN_SetDeviceMode(device);

	/* Mouse pointer needs to use the WIMP ShowCursor version so
	   that it doesn't modify the pointer until the SDL Window is
	   entered or the application goes full screen */
	device->ShowWMCursor = WIMP_ShowWMCursor;

	return device;
}

VideoBootStrap RISCOS_bootstrap = {
	RISCOSVID_DRIVER_NAME, "RISC OS video driver",
	RISCOS_Available, RISCOS_CreateDevice
};


int RISCOS_VideoInit(_THIS, SDL_PixelFormat *vformat)
{
	_kernel_swi_regs regs;
	int vars[4], vals[3];

	if (RISCOS_InitTask() == 0)
	{
		SDL_SetError("Unable to start task");
		return 0;
	}

	vars[0] = 9;  /* Log base 2 bpp */
	vars[1] = 11; /* XWndLimit - num x pixels -1 */
	vars[2] = 12; /* YWndLimit - num y pixels -1 */
	vars[3] = -1; /* Terminate list */
	regs.r[0] = (int)vars;
	regs.r[1] = (int)vals;

	_kernel_swi(OS_ReadVduVariables, &regs, &regs);
	vformat->BitsPerPixel = (1 << vals[0]);

	/* Determine the current screen size */
	this->info.current_w = vals[1] + 1;
	this->info.current_h = vals[2] + 1;

	/* Minimum bpp for SDL is 8 */
	if (vformat->BitsPerPixel < 8) vformat->BitsPerPixel = 8;


	switch (vformat->BitsPerPixel)
	{
		case 15:
		case 16:
			vformat->Bmask = 0x00007c00;
			vformat->Gmask = 0x000003e0;
			vformat->Rmask = 0x0000001f;
			vformat->BitsPerPixel = 16; /* SDL wants actual number of bits used */
			vformat->BytesPerPixel = 2;
			break;

		case 24:
		case 32:
			vformat->Bmask = 0x00ff0000;
			vformat->Gmask = 0x0000ff00;
			vformat->Rmask = 0x000000ff;
			vformat->BytesPerPixel = 4;
			break;

		default:
			vformat->Bmask = 0;
			vformat->Gmask = 0;
			vformat->Rmask = 0;
			vformat->BytesPerPixel = 1;			
			break;
	}

	/* Fill in some window manager capabilities */
	this->info.wm_available = 1;

	/* We're done! */
	return(0);
}

/* Note:  If we are terminated, this could be called in the middle of
   another SDL video routine -- notably UpdateRects.
*/
void RISCOS_VideoQuit(_THIS)
{
	RISCOS_ExitTask();

	if (this->hidden->alloc_bank) SDL_free(this->hidden->alloc_bank);
	this->hidden->alloc_bank = 0;
}


SDL_Rect **RISCOS_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags)
{
	if (flags & SDL_FULLSCREEN)
	{
		/* Build mode list when first required. */
		if (SDL_nummodes[0] == 0) FULLSCREEN_BuildModeList(this);

		return(SDL_modelist[((format->BitsPerPixel+7)/8)-1]);
	} else
		return (SDL_Rect **)-1;
}


/* Set up video mode */
SDL_Surface *RISCOS_SetVideoMode(_THIS, SDL_Surface *current,
				int width, int height, int bpp, Uint32 flags)
{
	if (flags & SDL_FULLSCREEN)
	{
	    RISCOS_StoreWimpMode();
		/* Dump wimp window on switch to full screen */
  	    if (this->hidden->window_handle) WIMP_DeleteWindow(this);

		return FULLSCREEN_SetVideoMode(this, current, width, height, bpp, flags);
	} else
	{
	    RISCOS_RestoreWimpMode();
		return WIMP_SetVideoMode(this, current, width, height, bpp, flags);
	}
}


/* We don't actually allow hardware surfaces other than the main one */
static int RISCOS_AllocHWSurface(_THIS, SDL_Surface *surface)
{
	return(-1);
}
static void RISCOS_FreeHWSurface(_THIS, SDL_Surface *surface)
{
	return;
}

/* We need to wait for vertical retrace on page flipped displays */
static int RISCOS_LockHWSurface(_THIS, SDL_Surface *surface)
{
	return(0);
}

static void RISCOS_UnlockHWSurface(_THIS, SDL_Surface *surface)
{
	return;
}


int RISCOS_GetWmInfo(_THIS, SDL_SysWMinfo *info)
{
	SDL_VERSION(&(info->version));
	info->wimpVersion = RISCOS_GetWimpVersion();
	info->taskHandle = RISCOS_GetTaskHandle();
	info->window = this->hidden->window_handle;

	return 1;
}
/* Toggle full screen mode.
   Returns 1 if successful otherwise 0
*/

int RISCOS_ToggleFullScreen(_THIS, int fullscreen)
{
    if (fullscreen)
    {
       return FULLSCREEN_ToggleFromWimp(this);
    } else
    {
       return WIMP_ToggleFromFullScreen(this);
    }

   return 0;
}

