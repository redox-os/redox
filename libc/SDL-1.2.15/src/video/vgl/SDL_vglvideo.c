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

/* libvga based SDL video driver implementation.
*/

#include <err.h>
#include <osreldate.h>
#include <unistd.h>
#include <sys/stat.h>

#include <sys/fbio.h>
#include <sys/consio.h>
#include <sys/kbio.h>
#include <vgl.h>

#include "SDL_video.h"
#include "SDL_mouse.h"
#include "../SDL_sysvideo.h"
#include "../SDL_pixels_c.h"
#include "../../events/SDL_events_c.h"
#include "SDL_vglvideo.h"
#include "SDL_vglevents_c.h"
#include "SDL_vglmouse_c.h"


/* Initialization/Query functions */
static int VGL_VideoInit(_THIS, SDL_PixelFormat *vformat);
static SDL_Rect **VGL_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags);
static SDL_Surface *VGL_SetVideoMode(_THIS, SDL_Surface *current, int width, int height, int bpp, Uint32 flags);
static int VGL_SetColors(_THIS, int firstcolor, int ncolors,
			  SDL_Color *colors);
static void VGL_VideoQuit(_THIS);

/* Hardware surface functions */
static int VGL_AllocHWSurface(_THIS, SDL_Surface *surface);
static int VGL_LockHWSurface(_THIS, SDL_Surface *surface);
static int VGL_FlipHWSurface(_THIS, SDL_Surface *surface);
static void VGL_UnlockHWSurface(_THIS, SDL_Surface *surface);
static void VGL_FreeHWSurface(_THIS, SDL_Surface *surface);

/* Misc function */
static VGLMode ** VGLListModes(int depth, int mem_model);

/* VGL driver bootstrap functions */

static int VGL_Available(void)
{
	/*
	 * Check to see if we are root and stdin is a
	 * virtual console. Also try to ensure that
	 * modes other than 320x200 are available
	 */
	int console, hires_available, i;
	VGLMode **modes;

	console = STDIN_FILENO;
	if ( console >= 0 ) {
		struct stat sb;
		struct vt_mode dummy;

		if ( (fstat(console, &sb) < 0) ||
		     (ioctl(console, VT_GETMODE, &dummy) < 0) ) {
			console = -1;
		}
	}
	if (geteuid() != 0 && console == -1)
		return 0;

	modes = VGLListModes(8, V_INFO_MM_DIRECT | V_INFO_MM_PACKED);
	hires_available = 0;
	for (i = 0; modes[i] != NULL; i++) {
		if ((modes[i]->ModeInfo.Xsize > 320) &&
		    (modes[i]->ModeInfo.Ysize > 200) &&
		    ((modes[i]->ModeInfo.Type == VIDBUF8) ||
		     (modes[i]->ModeInfo.Type == VIDBUF16) ||
		     (modes[i]->ModeInfo.Type == VIDBUF32))) {
			hires_available = 1;
			break;
		}
	}
	return hires_available;
}

static void VGL_DeleteDevice(SDL_VideoDevice *device)
{
	SDL_free(device->hidden);
	SDL_free(device);
}

static SDL_VideoDevice *VGL_CreateDevice(int devindex)
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
	device->VideoInit = VGL_VideoInit;
	device->ListModes = VGL_ListModes;
	device->SetVideoMode = VGL_SetVideoMode;
	device->SetColors = VGL_SetColors;
	device->UpdateRects = NULL;
	device->VideoQuit = VGL_VideoQuit;
	device->AllocHWSurface = VGL_AllocHWSurface;
	device->CheckHWBlit = NULL;
	device->FillHWRect = NULL;
	device->SetHWColorKey = NULL;
	device->SetHWAlpha = NULL;
	device->LockHWSurface = VGL_LockHWSurface;
	device->UnlockHWSurface = VGL_UnlockHWSurface;
	device->FlipHWSurface = VGL_FlipHWSurface;
	device->FreeHWSurface = VGL_FreeHWSurface;
	device->SetIcon = NULL;
	device->SetCaption = NULL;
	device->GetWMInfo = NULL;
	device->FreeWMCursor = VGL_FreeWMCursor;
	device->CreateWMCursor = VGL_CreateWMCursor;
	device->ShowWMCursor = VGL_ShowWMCursor;
	device->WarpWMCursor = VGL_WarpWMCursor;
	device->InitOSKeymap = VGL_InitOSKeymap;
	device->PumpEvents = VGL_PumpEvents;

	device->free = VGL_DeleteDevice;

	return device;
}

VideoBootStrap VGL_bootstrap = {
	"vgl", "FreeBSD libVGL",
	VGL_Available, VGL_CreateDevice
};

static int VGL_AddMode(_THIS, VGLMode *inmode)
{
	SDL_Rect *mode;

	int i, index;
	int next_mode;

	/* Check to see if we already have this mode */
	if (inmode->Depth < 8) {  /* Not supported */
		return 0;
	}
	index = ((inmode->Depth + 7) / 8) - 1;
	for (i=0; i<SDL_nummodes[index]; ++i) {
		mode = SDL_modelist[index][i];
		if ((mode->w == inmode->ModeInfo.Xsize) &&
		    (mode->h == inmode->ModeInfo.Ysize))
			return 0;
	}

	/* Set up the new video mode rectangle */
	mode = (SDL_Rect *)SDL_malloc(sizeof *mode);
	if (mode == NULL) {
		SDL_OutOfMemory();
		return -1;
	}
	mode->x = 0;
	mode->y = 0;
	mode->w = inmode->ModeInfo.Xsize;
	mode->h = inmode->ModeInfo.Ysize;

	/* Allocate the new list of modes, and fill in the new mode */
	next_mode = SDL_nummodes[index];
	SDL_modelist[index] = (SDL_Rect **)
		SDL_realloc(SDL_modelist[index], (1+next_mode+1)*sizeof(SDL_Rect *));
	if (SDL_modelist[index] == NULL) {
		SDL_OutOfMemory();
		SDL_nummodes[index] = 0;
		SDL_free(mode);
		return -1;
	}
	SDL_modelist[index][next_mode] = mode;
	SDL_modelist[index][next_mode+1] = NULL;
	SDL_nummodes[index]++;

	return 0;
}

static void VGL_UpdateVideoInfo(_THIS)
{
	this->info.wm_available = 0;
	this->info.hw_available = 1;
	this->info.video_mem = 0;
	if (VGLCurMode == NULL) {
		return;
	}
	if (VGLCurMode->ModeInfo.PixelBytes > 0) {
		this->info.video_mem = VGLCurMode->ModeInfo.PixelBytes *
				       VGLCurMode->ModeInfo.Xsize *
				       VGLCurMode->ModeInfo.Ysize;
	}
}

int VGL_VideoInit(_THIS, SDL_PixelFormat *vformat)
{
	int i;
	int total_modes;
	VGLMode **modes;

	/* Initialize all variables that we clean on shutdown */
	for ( i=0; i<NUM_MODELISTS; ++i ) {
		SDL_nummodes[i] = 0;
		SDL_modelist[i] = NULL;
	}

	/* Enable mouse and keyboard support */
	if (SDL_getenv("SDL_NO_RAWKBD") == NULL) {
		if (VGLKeyboardInit(VGL_CODEKEYS) != 0) {
			SDL_SetError("Unable to initialize keyboard");
			return -1;
		}
	} else {
		warnx("Requiest to put keyboard into a raw mode ignored");
	}
	if (VGL_initkeymaps(STDIN_FILENO) != 0) {
		SDL_SetError("Unable to initialize keymap");
		return -1;
	}
	if (VGL_initmouse(STDIN_FILENO) != 0) {
		SDL_SetError("Unable to initialize mouse");
		return -1;
	}

	/* Determine the current screen size */
	if (VGLCurMode != NULL) {
		this->info.current_w = VGLCurMode->ModeInfo.Xsize;
		this->info.current_h = VGLCurMode->ModeInfo.Ysize;
	}

	/* Determine the screen depth */
	if (VGLCurMode != NULL)
		vformat->BitsPerPixel = VGLCurMode->Depth;
	else
		vformat->BitsPerPixel = 16;	/* Good default */

	/* Query for the list of available video modes */
	total_modes = 0;
	modes = VGLListModes(-1, V_INFO_MM_DIRECT | V_INFO_MM_PACKED);
	for (i = 0; modes[i] != NULL; i++) {
		if ((modes[i]->ModeInfo.Type == VIDBUF8) ||
		    (modes[i]->ModeInfo.Type == VIDBUF16) ||
		    (modes[i]->ModeInfo.Type == VIDBUF32)) {
			VGL_AddMode(this, modes[i]);
			total_modes++;
		}
	}
	if (total_modes == 0) {
		SDL_SetError("No linear video modes available");
		return -1;
	}

	/* Fill in our hardware acceleration capabilities */
	VGL_UpdateVideoInfo(this);

	/* Create the hardware surface lock mutex */
	hw_lock = SDL_CreateMutex();
	if (hw_lock == NULL) {
		SDL_SetError("Unable to create lock mutex");
		VGL_VideoQuit(this);
		return -1;
	}

	/* We're done! */
	return 0;
}

SDL_Rect **VGL_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags)
{
	return SDL_modelist[((format->BitsPerPixel+7)/8)-1];
}

/* Various screen update functions available */
static void VGL_DirectUpdate(_THIS, int numrects, SDL_Rect *rects);

SDL_Surface *VGL_SetVideoMode(_THIS, SDL_Surface *current,
			      int width, int height, int bpp, Uint32 flags)
{
	int mode_found;
	int i;
	VGLMode **modes;

	modes = VGLListModes(bpp, V_INFO_MM_DIRECT | V_INFO_MM_PACKED);
	mode_found = 0;
	for (i = 0; modes[i] != NULL; i++) {
		if ((modes[i]->ModeInfo.Xsize == width) &&
		    (modes[i]->ModeInfo.Ysize == height) &&
		    ((modes[i]->ModeInfo.Type == VIDBUF8) ||
		     (modes[i]->ModeInfo.Type == VIDBUF16) ||
		     (modes[i]->ModeInfo.Type == VIDBUF32))) {
			mode_found = 1;
			break;
		}
	}
	if (mode_found == 0) {
		SDL_SetError("No matching video mode found");
		return NULL;
	}

	/* Shutdown previous videomode (if any) */
	if (VGLCurMode != NULL)
		VGLEnd();

	/* Try to set the requested linear video mode */
	if (VGLInit(modes[i]->ModeId) != 0) {
		SDL_SetError("Unable to switch to requested mode");
		return NULL;
	}

	VGLCurMode = SDL_realloc(VGLCurMode, sizeof(VGLMode));
	VGLCurMode->ModeInfo = *VGLDisplay;
	VGLCurMode->Depth = modes[i]->Depth;
	VGLCurMode->ModeId = modes[i]->ModeId;
	VGLCurMode->Rmask = modes[i]->Rmask;
	VGLCurMode->Gmask = modes[i]->Gmask;
	VGLCurMode->Bmask = modes[i]->Bmask;

	/* Workaround a bug in libvgl */
	if (VGLCurMode->ModeInfo.PixelBytes == 0)
		(VGLCurMode->ModeInfo.PixelBytes = 1);

	current->w = VGLCurMode->ModeInfo.Xsize;
	current->h = VGLCurMode->ModeInfo.Ysize;
	current->pixels = VGLCurMode->ModeInfo.Bitmap;
	current->pitch = VGLCurMode->ModeInfo.Xsize *
			 VGLCurMode->ModeInfo.PixelBytes;
	current->flags = (SDL_FULLSCREEN | SDL_HWSURFACE);

	/* Check if we are in a pseudo-color mode */
	if (VGLCurMode->ModeInfo.Type == VIDBUF8)
		current->flags |= SDL_HWPALETTE;

	/* Check if we can do doublebuffering */
	if (flags & SDL_DOUBLEBUF) {
		if (VGLCurMode->ModeInfo.Xsize * 2 <=
		    VGLCurMode->ModeInfo.VYsize) {
			current->flags |= SDL_DOUBLEBUF;
			flip_page = 0;
			flip_address[0] = (byte *)current->pixels;
			flip_address[1] = (byte *)current->pixels +
					  current->h * current->pitch;
			VGL_FlipHWSurface(this, current);
		}
	}

	if (! SDL_ReallocFormat(current, modes[i]->Depth, VGLCurMode->Rmask,
				VGLCurMode->Gmask, VGLCurMode->Bmask, 0)) {
		return NULL;
	}

	/* Update hardware acceleration info */
	VGL_UpdateVideoInfo(this);

	/* Set the blit function */
	this->UpdateRects = VGL_DirectUpdate;

	/* We're done */
	return current;
}

/* We don't actually allow hardware surfaces other than the main one */
static int VGL_AllocHWSurface(_THIS, SDL_Surface *surface)
{
	return -1;
}
static void VGL_FreeHWSurface(_THIS, SDL_Surface *surface)
{
	return;
}

/* We need to wait for vertical retrace on page flipped displays */
static int VGL_LockHWSurface(_THIS, SDL_Surface *surface)
{
	if (surface == SDL_VideoSurface) {
		SDL_mutexP(hw_lock);
	}
	return 0;
}
static void VGL_UnlockHWSurface(_THIS, SDL_Surface *surface)
{
	if (surface == SDL_VideoSurface) {
		SDL_mutexV(hw_lock);
	}
}

static int VGL_FlipHWSurface(_THIS, SDL_Surface *surface)
{
	if (VGLPanScreen(VGLDisplay, 0, flip_page * surface->h) < 0) {
		SDL_SetError("VGLPanSreen() failed");
                return -1;
        }

	flip_page = !flip_page;
	surface->pixels = flip_address[flip_page];

	return 0;
}

static void VGL_DirectUpdate(_THIS, int numrects, SDL_Rect *rects)
{
	return;
}

int VGL_SetColors(_THIS, int firstcolor, int ncolors, SDL_Color *colors)
{
        int i;

	for(i = 0; i < ncolors; i++) {
	        VGLSetPaletteIndex(firstcolor + i,
			       colors[i].r>>2,
			       colors[i].g>>2,
			       colors[i].b>>2);
	}
	return 1;
}

/* Note:  If we are terminated, this could be called in the middle of
   another SDL video routine -- notably UpdateRects.
*/
void VGL_VideoQuit(_THIS)
{
	int i, j;

	/* Return the keyboard to the normal state */
	VGLKeyboardEnd();

	/* Reset the console video mode if we actually initialised one */
	if (VGLCurMode != NULL) {
		VGLEnd();
		SDL_free(VGLCurMode);
		VGLCurMode = NULL;
	}

	/* Clear the lock mutex */
	if (hw_lock != NULL) {
		SDL_DestroyMutex(hw_lock);
		hw_lock = NULL;
	}

	/* Free video mode lists */
	for (i = 0; i < NUM_MODELISTS; i++) {
		if (SDL_modelist[i] != NULL) {
			for (j = 0; SDL_modelist[i][j] != NULL; ++j) {
				SDL_free(SDL_modelist[i][j]);
			}
			SDL_free(SDL_modelist[i]);
			SDL_modelist[i] = NULL;
		}
	}

	if ( this->screen && (this->screen->flags & SDL_HWSURFACE) ) {
	/* Direct screen access, not a memory buffer */
		this->screen->pixels = NULL;
	}
}

#define VGL_RED_INDEX	0
#define VGL_GREEN_INDEX	1
#define VGL_BLUE_INDEX	2

static VGLMode **
VGLListModes(int depth, int mem_model)
{
  static VGLMode **modes = NULL;

  VGLBitmap *vminfop;
  VGLMode **modesp, *modescp;
  video_info_t minfo;
  int adptype, i, modenum;

  if (modes == NULL) {
    modes = SDL_malloc(sizeof(VGLMode *) * M_VESA_MODE_MAX);
    bzero(modes, sizeof(VGLMode *) * M_VESA_MODE_MAX);
  }
  modesp = modes;

  for (modenum = 0; modenum < M_VESA_MODE_MAX; modenum++) {
    minfo.vi_mode = modenum;
    if (ioctl(0, CONS_MODEINFO, &minfo) || ioctl(0, CONS_CURRENT, &adptype))
      continue;
    if (minfo.vi_mode != modenum)
      continue;
    if ((minfo.vi_flags & V_INFO_GRAPHICS) == 0)
      continue;
    if ((mem_model != -1) && ((minfo.vi_mem_model & mem_model) == 0))
      continue;
    if ((depth > 1) && (minfo.vi_depth != depth))
      continue;

    /* reallocf can fail */
    if ((*modesp = reallocf(*modesp, sizeof(VGLMode))) == NULL)
      return NULL;
    modescp = *modesp;

    vminfop = &(modescp->ModeInfo);
    bzero(vminfop, sizeof(VGLBitmap));

    vminfop->Type = NOBUF;

    vminfop->PixelBytes = 1;	/* Good default value */
    switch (minfo.vi_mem_model) {
    case V_INFO_MM_PLANAR:
      /* we can handle EGA/VGA planar modes only */
      if (!(minfo.vi_depth != 4 || minfo.vi_planes != 4
	    || (adptype != KD_EGA && adptype != KD_VGA)))
	vminfop->Type = VIDBUF4;
      break;
    case V_INFO_MM_PACKED:
      /* we can do only 256 color packed modes */
      if (minfo.vi_depth == 8)
	vminfop->Type = VIDBUF8;
      break;
    case V_INFO_MM_VGAX:
      vminfop->Type = VIDBUF8X;
      break;
#if defined(__FREEBSD__) && (defined(__DragonFly__) || __FreeBSD_version >= 500000)
    case V_INFO_MM_DIRECT:
      vminfop->PixelBytes = minfo.vi_pixel_size;
      switch (vminfop->PixelBytes) {
      case 2:
	vminfop->Type = VIDBUF16;
	break;
#if notyet
      case 3:
	vminfop->Type = VIDBUF24;
	break;
#endif
      case 4:
	vminfop->Type = VIDBUF32;
	break;
      default:
	break;
      }
#endif
    default:
      break;
    }
    if (vminfop->Type == NOBUF)
      continue;

    switch (vminfop->Type) {
    case VIDBUF16:
    case VIDBUF32:
      modescp->Rmask = ((1 << minfo.vi_pixel_fsizes[VGL_RED_INDEX]) - 1) <<
		       minfo.vi_pixel_fields[VGL_RED_INDEX];
      modescp->Gmask = ((1 << minfo.vi_pixel_fsizes[VGL_GREEN_INDEX]) - 1) <<
		       minfo.vi_pixel_fields[VGL_GREEN_INDEX];
      modescp->Bmask = ((1 << minfo.vi_pixel_fsizes[VGL_BLUE_INDEX]) - 1) <<
		       minfo.vi_pixel_fields[VGL_BLUE_INDEX];
      break;

    default:
      break;
    }

    vminfop->Xsize = minfo.vi_width;
    vminfop->Ysize = minfo.vi_height;
    modescp->Depth = minfo.vi_depth;

    /* XXX */
    if (minfo.vi_mode >= M_VESA_BASE)
      modescp->ModeId = _IO('V', minfo.vi_mode - M_VESA_BASE);
    else
      modescp->ModeId = _IO('S', minfo.vi_mode);

    /* Sort list */
    for (i = 0; modes + i < modesp ; i++) {
      if (modes[i]->ModeInfo.Xsize * modes[i]->ModeInfo.Ysize >
	  vminfop->Xsize * modes[i]->ModeInfo.Ysize)
	continue;
      if ((modes[i]->ModeInfo.Xsize * modes[i]->ModeInfo.Ysize ==
	   vminfop->Xsize * vminfop->Ysize) &&
	  (modes[i]->Depth >= modescp->Depth))
	continue;
      *modesp = modes[i];
      modes[i] = modescp;
      modescp = *modesp;
      vminfop = &(modescp->ModeInfo);
    }

    modesp++;
  }

  if (*modesp != NULL) {
    SDL_free(*modesp);
    *modesp = NULL;
  }

  return modes;
}
