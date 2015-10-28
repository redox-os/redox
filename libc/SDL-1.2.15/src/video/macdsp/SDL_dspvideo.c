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
 Written by Darrell Walisser <dwaliss1@purdue.edu>

 Implementation notes ----------------------------------------------------------------------

 A bit on GWorlds in VRAM from technote 1182:

 There are two important things to note about GWorld's allocated in
 VRAM. First, the base address retrieved through GetPixBaseAddr or
 read directly from the PixMap structure can become invalid anytime
 memory is allocated in VRAM. This can occur either by explicit
 allocations, such as calls to NewGWorld, or by implicit ones, such as
 those associated with the internal texture allocation of OpenGL. The
 stored pixel images themselves will still be valid but may have been
 moved in VRAM, thus rendering any stored base addresses invalid.
 You should never store an image's base address for longer than is
 necessary and especially never across calls to NewGWorld or
 texture-creation routines. 

 Secondly, an offscreen pixel image allocated in VRAM can be
 purged at system task time by the display driver. This means any
 time your application yields time such by calling WaitNextEvent or
 SystemTask you can lose your VRAM GWorld contents. While this
 happens infrequently, usually associated with display resolution or
 pixel depth changes you must code for this eventuality. This purge
 can occur whether or not the GWorld is locked or not. A return value
 of false from LockPixels, a NULL return value from GetPixBaseAddr
 or NULL in the baseAddr field of the PixMap mean that the pixel
 image has been purged. To reallocate it you can either call
 UpdateGWorld or Dispose your current GWorld through
 DisposeGWorld and reallocate it via NewGWorld. Either way you must
 then rebuild the pixel image. 

------------------------------------------------------------------------------------

  Currently, I don't account for (1). In my testing, NewGWorld never invalidated
  other existing GWorlds in VRAM. However, I do have protection for (2).
  Namely, I am using GetOSEvent() instead of WaitNextEvent() so that there are no
  context switches (the app hogs the CPU). Eventually a book-keeping system should
  be coded to take care of (1) and (2).
  
------------------------------------------------------------------------------------

  System requirements (* denotes optional):
  
  1. DrawSprocket 1.7.3
  2. *MacOS 9 or later (but *not* Mac OS X) for hardware accelerated blit / fill
  3. *May also require certain graphics hardware for (2). I trust that all Apple OEM
     hardware will work. Third party accelerators may work if they have QuickDraw
     acceleration in the drivers and the drivers have been updated for OS 9. The current
     Voodoo 3 drivers (1.0b12) do not work.
  
  Coding suggestions:
  
  1. Use SDL_UpdateRects !
  
    If no QuickDraw acceleration is present, double-buffered surfaces will use a back buffer
    in System memory. I recommend you use SDL_UpdateRects with double-buffered surfaces
    for best performance on these cards, since the overhead is nearly zero for VRAM back buffer.
    
  2. Load most-resident surfaces first.
  
    If you fill up VRAM or AGP memory, there is no contingency for purging to make room for the next one.
    Therefore, you should load the surfaces you plan to use the most frequently first.
    Sooner or later, I will code LRU replacement to help this.
  
  TODO:
  Some kind of posterized mode for resolutions < 640x480.
  Window support / fullscreen toggle.
  Figure out how much VRAM is available. Put in video->info->video_mem.
  Track VRAM usage.
  
  BUGS:
  I can't create a hardware surface the same size as the screen?! How to fix?
  
  

   COMPILE OPTIONS:
   
   DSP_TRY_CC_AND_AA - Define if you want to try HWA color-key and alpha blitters
                       HW color-key blitting gives substantial improvements,
                       but hw alpha is neck-and-neck with SDL's soft bitter.

   DSP_NO_SYNC_VBL   - Define for HWA double-buffered surfaces: don't sync
                       pseudo-flip to monitor redraw.

   DSP_NO_SYNC_OPENGL - Define for OpenGL surfaces: don't sync buffer swap. Synching buffer
                        swap may result in reduced performance, but can eliminate some
                        tearing artifacts.
   CHANGELOG:
   09/17/00 Lots of little tweaks. Build modelist in reverse order so largest contexts
            list first. Compared various methods with ROM methods and fixed rez switch
            crashing bug in GL Tron. (Woohoo!)
*/

#define DSP_TRY_CC_AND_AA

/* #define DSP_NO_SYNC_VBL */

#define DSP_NO_SYNC_OPENGL


#if defined(__APPLE__) && defined(__MACH__)
#include <Carbon/Carbon.h>
#include <DrawSprocket/DrawSprocket.h>
#elif TARGET_API_MAC_CARBON && (UNIVERSAL_INTERFACES_VERSION > 0x0335)
#include <Carbon.h>
#include <DrawSprocket.h>
#else
#include <LowMem.h>
#include <Gestalt.h>
#include <Devices.h>
#include <DiskInit.h>
#include <QDOffscreen.h>
#include <DrawSprocket.h>
#endif

#include "SDL_video.h"
#include "SDL_syswm.h"
#include "../SDL_sysvideo.h"
#include "../SDL_blit.h"
#include "../SDL_pixels_c.h"
#include "SDL_dspvideo.h"
#include "../maccommon/SDL_macgl_c.h"
#include "../maccommon/SDL_macwm_c.h"
#include "../maccommon/SDL_macmouse_c.h"
#include "../maccommon/SDL_macevents_c.h"

/* Initialization/Query functions */
static int DSp_VideoInit(_THIS, SDL_PixelFormat *vformat);
static SDL_Rect **DSp_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags);
static SDL_Surface *DSp_SetVideoMode(_THIS, SDL_Surface *current, int width, int height, int bpp, Uint32 flags);
static int DSp_SetColors(_THIS, int firstcolor, int ncolors,
			 SDL_Color *colors);
static int DSp_CreatePalette(_THIS);
static int DSp_DestroyPalette(_THIS);
static void DSp_VideoQuit(_THIS);

static int DSp_GetMainDevice (_THIS, GDHandle *device);
static void DSp_IsHWAvailable (_THIS, SDL_PixelFormat *vformat);
static void DSp_DSpUpdate(_THIS, int numrects, SDL_Rect *sdl_rects);
static void DSp_DirectUpdate(_THIS, int numrects, SDL_Rect *sdl_rects);

/* Hardware surface functions */
static int DSp_SetHWAlpha(_THIS, SDL_Surface *surface, UInt8 alpha);
static int DSp_SetHWColorKey(_THIS, SDL_Surface *surface, Uint32 key);
static int DSp_NewHWSurface(_THIS, CGrafPtr *port, int depth, int width, int height);
static int DSp_AllocHWSurface(_THIS, SDL_Surface *surface);
static int DSp_LockHWSurface(_THIS, SDL_Surface *surface);
static void DSp_UnlockHWSurface(_THIS, SDL_Surface *surface);
static void DSp_FreeHWSurface(_THIS, SDL_Surface *surface);
static int DSp_FlipHWSurface(_THIS, SDL_Surface *surface);
static int DSp_CheckHWBlit(_THIS, SDL_Surface *src, SDL_Surface *dest);
static int DSp_HWAccelBlit(SDL_Surface *src, SDL_Rect *srcrect,
                           SDL_Surface *dst, SDL_Rect *dstrect);
static int DSp_FillHWRect(_THIS, SDL_Surface *dst, SDL_Rect *rect, Uint32 color);

#if SDL_VIDEO_OPENGL
   static void DSp_GL_SwapBuffers (_THIS);
#endif

#if ! TARGET_API_MAC_CARBON

    #define GetPortPixRowBytes(x)  ( (*(x->portPixMap))->rowBytes )
   #define GetGDevPixMap(x) ((**(x)).gdPMap)   
   #define GetPortPixMap(x) ((*(x)).portPixMap)
   
   #define GetPixDepth(y)    ((**(y)).pixelSize)
   //#define GetPixRowBytes(y) ((**(y)).rowBytes)
   //#define GetPixBaseAddr(y) ((**(y)).baseAddr)
   #define GetPixCTab(y)     ((**(y)).pmTable)
    #define GetPortBitMapForCopyBits(x) (&(((GrafPtr)(x))->portBits))
   
#else
    #define GetPortPixRowBytes(x) (GetPixRowBytes(GetPortPixMap(x)) )
    #define GetGDevPixMap(x) ((**(x)).gdPMap)

#endif

typedef struct private_hwdata {

  GWorldPtr offscreen;    // offscreen gworld in VRAM or AGP
  
  #ifdef DSP_TRY_CC_AND_AA
    GWorldPtr mask;         // transparent mask
    RGBColor  alpha;        // alpha color
    RGBColor  trans;        // transparent color
  #endif
  
} private_hwdata;

typedef private_hwdata private_swdata ; /* have same fields */

/* Macintosh toolbox driver bootstrap functions */

static int DSp_Available(void)
{
	/* Check for DrawSprocket */
#if ! TARGET_API_MAC_OSX
	/* This check is only meaningful if you weak-link DrawSprocketLib */  
	return ((Ptr)DSpStartup != (Ptr)kUnresolvedCFragSymbolAddress);
#else
	return 1; // DrawSprocket.framework doesn't have it all, but it's there
#endif
}

static void DSp_DeleteDevice(SDL_VideoDevice *device)
{
	/* -dw- taking no chances with null pointers */
	if (device) {
		
   	if (device->hidden) {
   	   
   	   if (device->hidden->dspinfo)
	         SDL_free(device->hidden->dspinfo);
   	   
   	   SDL_free(device->hidden);
   	}
	   SDL_free(device);	
	}
}

static SDL_VideoDevice *DSp_CreateDevice(int devindex)
{
	SDL_VideoDevice *device;

	/* Initialize all variables that we clean on shutdown */
	device = (SDL_VideoDevice *)SDL_malloc(sizeof(SDL_VideoDevice));
	if ( device ) {
		SDL_memset(device, 0, sizeof (*device));
		device->hidden = (struct SDL_PrivateVideoData *)
				SDL_malloc((sizeof *device->hidden));
	    if (device->hidden)
	        SDL_memset(device->hidden, 0, sizeof ( *(device->hidden) ) );
	}
	if ( (device == NULL) || (device->hidden == NULL) ) {
		SDL_OutOfMemory();
			
		if ( device ) {
			
			if (device->hidden)
				SDL_free(device->hidden);			
			
			SDL_free(device);
		}
		
		return(NULL);
	}
	
	/* Allocate DrawSprocket information */
	device->hidden->dspinfo = (struct DSpInfo *)SDL_malloc(
					(sizeof *device->hidden->dspinfo));
	if ( device->hidden->dspinfo == NULL ) {
		SDL_OutOfMemory();
		SDL_free(device->hidden);
		SDL_free(device);
		return(0);
	}
	SDL_memset(device->hidden->dspinfo, 0, (sizeof *device->hidden->dspinfo));

	/* Set the function pointers */
	device->VideoInit       = DSp_VideoInit;
	device->ListModes       = DSp_ListModes;
	device->SetVideoMode    = DSp_SetVideoMode;
	device->SetColors       = DSp_SetColors;
	device->UpdateRects     = NULL;
	device->VideoQuit       = DSp_VideoQuit;
	device->AllocHWSurface  = DSp_AllocHWSurface;
	device->CheckHWBlit     = NULL;
	device->FillHWRect      = NULL;
	device->SetHWColorKey   = NULL;
	device->SetHWAlpha      = NULL;
	device->LockHWSurface   = DSp_LockHWSurface;
	device->UnlockHWSurface = DSp_UnlockHWSurface;
	device->FlipHWSurface   = DSp_FlipHWSurface;
	device->FreeHWSurface   = DSp_FreeHWSurface;
#if SDL_MACCLASSIC_GAMMA_SUPPORT
	device->SetGammaRamp    = Mac_SetGammaRamp;
	device->GetGammaRamp    = Mac_GetGammaRamp;
#endif
#if SDL_VIDEO_OPENGL
	device->GL_MakeCurrent  = Mac_GL_MakeCurrent;
	device->GL_SwapBuffers  = DSp_GL_SwapBuffers;
	device->GL_LoadLibrary = Mac_GL_LoadLibrary;
	device->GL_GetProcAddress = Mac_GL_GetProcAddress;
#endif
	device->SetCaption = NULL;
	device->SetIcon = NULL;
	device->IconifyWindow = NULL;
	device->GrabInput = NULL;
	device->GetWMInfo = NULL;
	device->FreeWMCursor    = Mac_FreeWMCursor;
	device->CreateWMCursor  = Mac_CreateWMCursor;
	device->ShowWMCursor    = Mac_ShowWMCursor;
	device->WarpWMCursor    = Mac_WarpWMCursor;
	device->InitOSKeymap    = Mac_InitOSKeymap;
	device->PumpEvents      = Mac_PumpEvents;
	
	device->GrabInput      = NULL;
	device->CheckMouseMode = NULL;
	
	device->free = DSp_DeleteDevice;

	return device;
}

VideoBootStrap DSp_bootstrap = {
	"DSp", "MacOS DrawSprocket",
	DSp_Available, DSp_CreateDevice
};

/* Use DSp/Display Manager to build mode list for given screen */
static SDL_Rect**  DSp_BuildModeList (const GDHandle gDevice, int *displayWidth, int *displayHeight)
{
	DSpContextAttributes  attributes;
	DSpContextReference   context;
	DisplayIDType         displayID;
	SDL_Rect temp_list [16];
	SDL_Rect **mode_list;
	int width, height, i, j;
        
        #if TARGET_API_MAC_OSX		
	
        displayID = 0;
        
        #else
        /* Ask Display Manager for integer id of screen device */
	if ( DMGetDisplayIDByGDevice (gDevice, &displayID, SDL_TRUE) != noErr ) {
		return NULL;
	}
	#endif
	/* Get the first possible DSp context on this device */
	if ( DSpGetFirstContext (displayID, &context) != noErr ) {
		return NULL;
	}
	
	if ( DSpContext_GetAttributes (context, &attributes) != noErr )
		return NULL;

	*displayWidth = attributes.displayWidth;
	*displayHeight = attributes.displayHeight;
			
	for ( i = 0; i < SDL_arraysize(temp_list); i++ ) {
		width  = attributes.displayWidth;
		height = attributes.displayHeight;
		
		temp_list [i].x = 0 | attributes.displayBestDepth;
		temp_list [i].y = 0;
		temp_list [i].w = width;
		temp_list [i].h = height;
	
		/* DSp will report many different contexts with the same width and height. */
		/* They will differ in bit depth and refresh rate. */
		/* We will ignore them until we reach one with a different width/height */
		/* When there are no more contexts to look at, we will quit building the list*/
		while ( width == attributes.displayWidth && height == attributes.displayHeight ) {
		
			OSStatus err = DSpGetNextContext (context, &context);
			if (err != noErr)
				if (err == kDSpContextNotFoundErr)
					goto done;
				else
					return NULL;		
			
			if ( DSpContext_GetAttributes (context, &attributes) != noErr )
				return NULL;
				
			temp_list [i].x |= attributes.displayBestDepth;
		}
	}
done:
	i++;          /* i was not incremented before kicking out of the loop */
	
	mode_list = (SDL_Rect**) SDL_malloc (sizeof (SDL_Rect*) * (i+1));
	if (mode_list) {
	
	   /* -dw- new stuff: build in reverse order so largest sizes list first */
		for (j = i-1; j >= 0; j--) {
			mode_list [j] = (SDL_Rect*) SDL_malloc (sizeof (SDL_Rect));	
			if (mode_list [j])
				SDL_memcpy (mode_list [j], &(temp_list [j]), sizeof (SDL_Rect));
			else {
				SDL_OutOfMemory ();
				return NULL;
			}
		}
		mode_list [i] = NULL;		/* append null to the end */
	}
	else {
		SDL_OutOfMemory ();
		return NULL;
	}
		
	return mode_list;
}

static void DSp_IsHWAvailable (_THIS, SDL_PixelFormat *vformat)
{
  /* 
     VRAM GWorlds are only available on OS 9 or later.
     Even with OS 9, some display drivers won't support it, 
     so we create a test GWorld and check for errors. 
  */

  long versionSystem;

  dsp_vram_available = SDL_FALSE;
  dsp_agp_available  = SDL_FALSE;
  
  Gestalt ('sysv', &versionSystem);
  if (0x00000860 < (versionSystem & 0x0000FFFF)) {
    
    GWorldPtr offscreen;
    OSStatus  err;
    Rect      bounds;
    
    SetRect (&bounds, 0, 0, 320, 240);
    
#if useDistantHdwrMem && useLocalHdwrMem
    err = NewGWorld (&offscreen, vformat->BitsPerPixel, &bounds, NULL, SDL_Display, useDistantHdwrMem | noNewDevice);
    if (err == noErr) {
      dsp_vram_available = SDL_TRUE;
      DisposeGWorld (offscreen);
    }
	
    err = NewGWorld (&offscreen, vformat->BitsPerPixel, &bounds, NULL, SDL_Display, useLocalHdwrMem | noNewDevice);
    if (err == noErr) {
      DisposeGWorld (offscreen);
      dsp_agp_available = SDL_TRUE;
    }
#endif
  }
}

static int DSp_GetMainDevice (_THIS, GDHandle *device)
{
    
#if TARGET_API_MAC_OSX
        /* DSpUserSelectContext not available on OS X */
        *device = GetMainDevice();
        return 0;
#else
        
	DSpContextAttributes attrib;
	DSpContextReference  context;
	DisplayIDType        display_id;
	GDHandle             main_device;
	GDHandle             device_list;
	
	device_list = GetDeviceList ();
	main_device = GetMainDevice ();
	
	/* Quick check to avoid slower method when only one display exists */
	if ( (**device_list).gdNextGD == NULL ) {
	  *device = main_device;
	  return 0;
	}
		
	SDL_memset (&attrib, 0, sizeof (DSpContextAttributes));

	/* These attributes are hopefully supported on all devices...*/
	attrib.displayWidth         = 640;
	attrib.displayHeight        = 480;
	attrib.displayBestDepth     = 8;
	attrib.backBufferBestDepth  = 8;
	attrib.displayDepthMask     = kDSpDepthMask_All;
	attrib.backBufferDepthMask  = kDSpDepthMask_All;
	attrib.colorNeeds           = kDSpColorNeeds_Require;
	attrib.pageCount            = 1;
			 
	if (noErr != DMGetDisplayIDByGDevice (main_device, &display_id, SDL_FALSE)) {
		SDL_SetError ("Display Manager couldn't associate GDevice with a Display ID");
		return (-1);
	}
	
	/* Put up dialog on main display to select which display to use */
	if (noErr != DSpUserSelectContext (&attrib, display_id, NULL, &context)) {
		SDL_SetError ("DrawSprocket couldn't create a context");
		return (-1);
	}
         
	if (noErr != DSpContext_GetDisplayID (context, &display_id)) {
		SDL_SetError ("DrawSprocket couldn't get display ID");
		return (-1);
	}
  
	if (noErr != DMGetGDeviceByDisplayID  (display_id, &main_device, SDL_FALSE)) {
		SDL_SetError ("Display Manager couldn't associate Display ID with GDevice");
		return (-1);  
	}

	*device = main_device;
	return (0);
#endif
}

static int DSp_VideoInit(_THIS, SDL_PixelFormat *vformat)
{
	NumVersion dsp_version = { 0x01, 0x00, 0x00, 0x00 };
	
#if UNIVERSAL_INTERFACES_VERSION > 0x0320
	dsp_version = DSpGetVersion ();
#endif
	
	if (  (dsp_version.majorRev == 1 && dsp_version.minorAndBugRev < 0x73) ||
	      (dsp_version.majorRev < 1)  ) {                          
	    
	   /* StandardAlert (kAlertStopAlert, "\pError!", 
	                "\pI need DrawSprocket 1.7.3 or later!\n"
	                  "You can find a newer version at http://www.apple.com/swupdates.",
	                   NULL, NULL);
	    */              
	    SDL_SetError ("DrawSprocket version is too old. Need 1.7.3 or later.");
	    return (-1);
	}
	
	if ( DSpStartup () != noErr ) {
		SDL_SetError ("DrawSprocket couldn't startup");
		return(-1);
	}
	
	/* Start DSpintosh events */
	Mac_InitEvents(this);

	/* Get a handle to the main monitor, or choose one on multiple monitor setups */
	if ( DSp_GetMainDevice(this, &SDL_Display) <  0)
		return (-1);

	/* Determine pixel format */
    vformat->BitsPerPixel = GetPixDepth ( (**SDL_Display).gdPMap );
	dsp_old_depth = vformat->BitsPerPixel;
		
	switch (vformat->BitsPerPixel) {
		case 16:	
			vformat->Rmask = 0x00007c00;
			vformat->Gmask = 0x000003e0;
			vformat->Bmask = 0x0000001f;
			break;
		default:
			break;
	}
   
	if ( DSp_CreatePalette (this) < 0 ) {
		SDL_SetError ("Could not create palette");
		return (-1);
	}
   
	/* Get a list of available fullscreen modes */
	SDL_modelist = DSp_BuildModeList (SDL_Display,
	                                  &this->info.current_w, &this->info.current_h);
	if (SDL_modelist == NULL) {
		SDL_SetError ("DrawSprocket could not build a mode list");
		return (-1);
	}
	
	/* Check for VRAM and AGP GWorlds for HW Blitting */
	DSp_IsHWAvailable (this, vformat);
	
	this->info.wm_available = 0;

	if (dsp_vram_available || dsp_agp_available) {
    
	  this->info.hw_available = SDL_TRUE;
	  
	  this->CheckHWBlit  = DSp_CheckHWBlit;
	  this->info.blit_hw = SDL_TRUE; 
  
	  this->FillHWRect     = DSp_FillHWRect;
	  this->info.blit_fill = SDL_TRUE;
	  
	#ifdef DSP_TRY_CC_AND_AA  
	  this->SetHWColorKey   = DSp_SetHWColorKey;
	  this->info.blit_hw_CC = SDL_TRUE;
	  
	  this->SetHWAlpha      = DSp_SetHWAlpha;
	  this->info.blit_hw_A  = SDL_TRUE;
	#endif
	
	}  
    
	return(0);
}

static SDL_Rect **DSp_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags)
{
	static SDL_Rect *dsp_modes[16];
	int i = 0, j = 0;
	
	if ( format->BitsPerPixel == 0 )
	   return ( (SDL_Rect**) NULL );
	
	while (SDL_modelist[i] != NULL) {
	
	   if (SDL_modelist[i]->x & format->BitsPerPixel) {
	      dsp_modes[j] = SDL_modelist[i];
	      j++;
	   }
	   i++;
	}
	
	dsp_modes[j] = NULL;

	return dsp_modes;
}

/* Various screen update functions available */
static void DSp_DirectUpdate(_THIS, int numrects, SDL_Rect *rects);

#if ! TARGET_API_MAC_OSX

static volatile unsigned int retrace_count = 0; /* -dw- need volatile because it updates asychronously */

Boolean DSp_VBLProc ( DSpContextReference context, void *ref_con )
{
	retrace_count++;
	
	return 1; /* Darrell, is this right? */
}

static void DSp_SetHWError (OSStatus err, int is_agp)
{
	char message[1024];
	const char *fmt, *mem;

	if ( is_agp ) {
		mem = "AGP Memory";
	} else {
		mem = "VRAM";
	}
	switch(err) {
	    case memFullErr:
		fmt = "Hardware surface possible but not enough %s available";
		break;
	    case cDepthErr:
		fmt = "Hardware surface possible but invalid color depth";
		break;
	    default:
		fmt = "Hardware surface could not be allocated in %s - unknown error";
		break;
	}
	SDL_snprintf(message, SDL_arraysize(message), fmt, mem);
	SDL_SetError(message);
}
#endif // TARGET_API_MAC_OSX

/* put up a dialog to verify display change */
static int DSp_ConfirmSwitch () {

  /* resource id's for dialog */
  const int rDialog = 1002;
  const int bCancel = 1;
  const int bOK     = 2;
  
  DialogPtr dialog;
  OSStatus  err;
  SInt32    response;
  DialogItemIndex       item = 0;
  GrafPtr   savePort;
    
  GetPort (&savePort);
  
  dialog = GetNewDialog (rDialog, NULL, (WindowPtr) -1);
  if (dialog == NULL)
	 return (0);
  
#if TARGET_API_MAC_CARBON
  SetPort (GetDialogPort(dialog));
#else
  SetPort ((WindowPtr) dialog);
#endif
  
  SetDialogDefaultItem (dialog, bCancel);
  SetDialogCancelItem  (dialog, bCancel);
  
  SetEventMask (everyEvent);  
  FlushEvents (everyEvent, 0);
   
   /* On MacOS 8.5 or later, we can make the dialog go away after 15 seconds */
   /* This is good since it's possible user can't even see the dialog! */
   /* Requires linking to DialogsLib */
   err = Gestalt(gestaltSystemVersion,&response);
   if (err == noErr && response >= 0x00000850) {
   	SetDialogTimeout(dialog, bCancel, 15);
   }

   do {      
    
    ModalDialog ( NULL, &item );  

   } while ( item != bCancel && item != bOK && err != noErr);


  DisposeDialog (dialog);
  SetPort (savePort);
  
  SetEventMask(everyEvent - autoKeyMask);
  FlushEvents(everyEvent, 0);
   
  return (item - 1);
}

static void DSp_UnsetVideoMode(_THIS, SDL_Surface *current)
{
			
		
	 if ( current->flags & SDL_OPENGL )  { 
	   Mac_GL_Quit (this);	   	
	}
		
	if (dsp_context != NULL) {
		
		GWorldPtr front;
		DSpContext_GetFrontBuffer (dsp_context, &front);
		
		if (front != dsp_back_buffer)
		   DisposeGWorld (dsp_back_buffer);
		
		if (current->hwdata)
		   SDL_free(current->hwdata);
		   
		DSpContext_SetState (dsp_context, kDSpContextState_Inactive );
		DSpContext_Release  (dsp_context);
		
		dsp_context = NULL;
	}
    	
    if (SDL_Window != NULL) {
        DisposeWindow (SDL_Window);
        SDL_Window = NULL;
    }    
    
    current->pixels = NULL;
    current->flags  = 0;
}

static SDL_Surface *DSp_SetVideoMode(_THIS,
	SDL_Surface *current, int width, int height, int bpp, Uint32 flags)
{
	
#if !TARGET_API_MAC_OSX
    DisplayIDType        display_id;
	Fixed freq;
#endif
	DSpContextAttributes attrib;
	OSStatus err;
	UInt32 rmask = 0, gmask = 0, bmask = 0;
		
	int   page_count;
	int   double_buf;
	int   hw_surface;
	int   use_dsp_back_buffer;
     
	DSp_UnsetVideoMode (this, current);
       
    if (bpp != dsp_old_depth)
        DSp_DestroyPalette (this);
   
	double_buf = (flags & SDL_DOUBLEBUF) != 0;
	hw_surface = (flags & SDL_HWSURFACE) != 0;
	use_dsp_back_buffer = !dsp_vram_available || !hw_surface ;
	
	current->flags |= SDL_FULLSCREEN;

rebuild:  
  
	if ( double_buf && use_dsp_back_buffer ) {
		page_count = 2;
	} else {
		page_count = 1;
	}

	SDL_memset (&attrib, 0, sizeof (DSpContextAttributes));
	attrib.displayWidth         = width;
	attrib.displayHeight        = height;
	attrib.displayBestDepth     = bpp;
	attrib.backBufferBestDepth  = bpp;
	attrib.displayDepthMask     = kDSpDepthMask_All;
	attrib.backBufferDepthMask  = kDSpDepthMask_All;
	attrib.colorNeeds           = kDSpColorNeeds_Require;
	attrib.colorTable           = 0;
	attrib.pageCount            = page_count;
        #if TARGET_API_MAC_OSX || UNIVERSAL_INTERFACES_VERSION == 0x0320
        
        if ( DSpFindBestContext (&attrib, &dsp_context) != noErr ) {
            SDL_SetError ("DrawSprocket couldn't find a context");
            return NULL;
        }
        
        #else
	if ( noErr != DMGetDisplayIDByGDevice (SDL_Display, &display_id, SDL_FALSE) ) {
		SDL_SetError ("Display Manager couldn't associate GDevice with display_id");
		return NULL;
	}	
	if ( DSpFindBestContextOnDisplayID(&attrib, &dsp_context, display_id) != noErr ) {
		SDL_SetError ("DrawSprocket couldn't find a suitable context on given display");
		return NULL;
	}
	
        #endif		
	if ( DSpContext_Reserve (dsp_context, &attrib) != noErr ) {
		SDL_SetError ("DrawSprocket couldn't get the needed resources to build the display");
		return NULL;
	}
	
	if ( (err = DSpContext_SetState (dsp_context, kDSpContextState_Active)) != noErr ) {
		
		if (err == kDSpConfirmSwitchWarning) {     
		  
		   if ( ! DSp_ConfirmSwitch () ) {
		   
		      DSpContext_Release (dsp_context);
		      dsp_context = NULL;
		      SDL_SetError ("User cancelled display switch");
		      return NULL;
		   }
		   else
		     /* Have to reactivate context. Why? */
		     DSpContext_SetState (dsp_context, kDSpContextState_Active);
		      
	   }
	   else {
	      SDL_SetError ("DrawSprocket couldn't activate the context");
		  return NULL;
	   }
	}
   
   
	if (bpp != dsp_old_depth) {
   	
   	    DSp_CreatePalette  (this);
   
       	/* update format if display depth changed */
       	if (bpp == 16) {
       	
       	   rmask = 0x00007c00;
       	   gmask = 0x000003e0;
       	   bmask = 0x0000001f;
       	}
       	if ( ! SDL_ReallocFormat (current, bpp, rmask, gmask, bmask, 0 ) ) {
       		
       	   SDL_SetError ("Could not reallocate video format.");
       	   return(NULL);
       	}
	}
	
	if (!double_buf) {
		
		/* single-buffer context */
		DSpContext_GetFrontBuffer (dsp_context, &dsp_back_buffer);
			
		current->hwdata   = (private_hwdata*) SDL_malloc (sizeof (private_hwdata));
		if (current ->hwdata == NULL) {
			SDL_OutOfMemory ();
	  		return NULL;		  
		}
		current->hwdata->offscreen = dsp_back_buffer;
	    current->flags   |= SDL_HWSURFACE;
	    this->UpdateRects = DSp_DirectUpdate;
	} 
	else if ( use_dsp_back_buffer ) {
	
		DSpContext_GetBackBuffer  (dsp_context, kDSpBufferKind_Normal, &dsp_back_buffer);
		
		current->flags   |= SDL_DOUBLEBUF | SDL_SWSURFACE; /* only front buffer is in VRAM */                                     
	    this->UpdateRects = DSp_DSpUpdate;	
	} 
	else if ( DSp_NewHWSurface(this, &dsp_back_buffer, bpp, width-1, height-1) == 0 ) {
      
      current->hwdata = (private_hwdata*) SDL_malloc (sizeof (private_hwdata));
      if (current ->hwdata == NULL) {
      	SDL_OutOfMemory ();
      	return NULL;		  
      }
      
      SDL_memset (current->hwdata, 0, sizeof (private_hwdata));
      current->hwdata->offscreen = dsp_back_buffer;
      current->flags |= SDL_DOUBLEBUF | SDL_HWSURFACE; 
      this->UpdateRects = DSp_DirectUpdate; /* hardware doesn't do update rects, must be page-flipped */	   
   }  	
   else {

	   DSpContext_Release (dsp_context);	
	   use_dsp_back_buffer = SDL_TRUE;
	   goto  rebuild;
    }
	   	
    current->pitch  = GetPortPixRowBytes(dsp_back_buffer) & 0x3FFF;
	current->pixels = GetPixBaseAddr(GetPortPixMap(dsp_back_buffer));
	
	current->w = width;
	current->h = height;
	
    #if ! TARGET_API_MAC_OSX
        
	if (use_dsp_back_buffer) {
	   
	   DSpContext_GetMonitorFrequency (dsp_context, &freq);
	   DSpContext_SetMaxFrameRate     (dsp_context, freq >> 16);
	}
	
    
	if ( (current->flags & SDL_HWSURFACE) || (current->flags & SDL_OPENGL) )
		DSpContext_SetVBLProc (dsp_context, DSp_VBLProc, NULL);
    #endif
	
	if (bpp == 8)	
	   current->flags |= SDL_HWPALETTE;
	
	if (flags & SDL_OPENGL) {
		   
	   Rect rect;
	   RGBColor rgb = { 0.0, 0.0, 0.0 };
	   GrafPtr save_port;
	   
	   SetRect (&rect, 0, 0, width, height);
	   SDL_Window = NewCWindow(nil, &( (**SDL_Display).gdRect), "\p", SDL_TRUE, plainDBox, (WindowPtr)-1, SDL_FALSE, 0);
		   
	   if (SDL_Window == NULL) {
		 
		   SDL_SetError ("DSp_SetVideoMode : OpenGL window could not be created.");
		   return NULL;  			   	
	   }
	   
	   /* Set window color to black to avoid white flash*/
	   GetPort (&save_port);
#if TARGET_API_MAC_CARBON
		SetPort (GetWindowPort(SDL_Window));
#else
	   SetPort (SDL_Window);
#endif
	      RGBForeColor (&rgb);
	      PaintRect    (&rect);	
	   SetPort (save_port);
	   
	   SetPortWindowPort (SDL_Window);
	   SelectWindow  (SDL_Window);
	     
	   if ( Mac_GL_Init (this) < 0 ) {
	   
	      SDL_SetError ("DSp_SetVideoMode : could not create OpenGL context.");
	      return NULL;
	   }
	   	   	   	   
	   current->flags |= SDL_OPENGL;	
	}
	
	return current; 
}

#ifdef DSP_TRY_CC_AND_AA

static int DSp_MakeHWMask (_THIS, SDL_Surface *surface)
{
    GDHandle save_device;
    CGrafPtr save_port;
    GWorldPtr temp;
    RGBColor black = { 0, 0, 0 };
    RGBColor white = { 0xFFFF, 0xFFFF, 0xFFFF };
    Rect     rect;
    
    Uint32 depth = GetPixDepth ( GetGDevPixMap (SDL_Display) );
    
    SetRect (&rect, 0, 0, surface->w, surface->h);
    
    if ( noErr != NewGWorld (&(surface->hwdata->mask), depth, &rect, 0, SDL_Display, 0 ) < 0 ) {
    
        SDL_OutOfMemory ();
        return (-1);
    }   
    
    if ( noErr != NewGWorld (&temp, depth, &rect, 0 , SDL_Display, 0 ) ) {
    
        SDL_OutOfMemory ();
        return (-1);
    }                         

            
    GetGWorld (&save_port, &save_device);
    SetGWorld (surface->hwdata->mask, SDL_Display);
    
    RGBForeColor (&white);
    PaintRect    (&rect);
                 
    RGBBackColor (&(surface->hwdata->trans));
    
    CopyBits ( GetPortBitMapForCopyBits(surface->hwdata->offscreen),
                 GetPortBitMapForCopyBits(surface->hwdata->mask),
    	       &rect, &rect, transparent, NULL );
        
    SetGWorld (surface->hwdata->mask, SDL_Display);    
    SetGWorld (save_port, save_device);     
    return (0);
}

static int DSp_SetHWAlpha(_THIS, SDL_Surface *surface, UInt8 alpha)
{
    surface->hwdata->alpha.red   = (alpha / 255.0) * 65535;
    surface->hwdata->alpha.blue  = (alpha / 255.0) * 65535;
    surface->hwdata->alpha.green = (alpha / 255.0) * 65535;

    surface->flags |= SDL_SRCALPHA;

    if (surface->flags & SDL_SRCCOLORKEY) {
        return(DSp_MakeHWMask (this, surface));
    }
    return(0);
}

static int DSp_SetHWColorKey(_THIS, SDL_Surface *surface, Uint32 key)
{
    CGrafPtr save_port;
    GDHandle save_device;
    
    GetGWorld (&save_port, &save_device);
    SetGWorld (surface->hwdata->offscreen, NULL);
    
    Index2Color (key, &(surface->hwdata->trans));
    surface->flags |= SDL_SRCCOLORKEY;    
    
    SetGWorld (save_port, save_device);
    
    if ( surface->flags & SDL_SRCALPHA ) {
        return(DSp_MakeHWMask (this, surface));    
    } 
    return(0);
}

#endif /* DSP_TRY_CC_AND_AA */

static int DSp_NewHWSurface(_THIS, CGrafPtr *port, int depth, int width, int height) {
   
   OSStatus err;
   Rect     bounds;
		
	SetRect (&bounds, 0, 0, width, height);
   
 #if useDistantHdwrMem && useLocalHdwrMem
    if (dsp_vram_available) {
	   /* try VRAM */
   	  err = NewGWorld (port, depth, &bounds, 0 , SDL_Display, useDistantHdwrMem | noNewDevice );
      if (err != noErr)
         DSp_SetHWError (err, SDL_FALSE);        
      else
         return (0);      
    }
    
    if (dsp_agp_available) {
      /* try AGP */
      err = NewGWorld (port, depth, &bounds, 0 , SDL_Display, useLocalHdwrMem | noNewDevice );
                                            
      if (err != noErr)
         DSp_SetHWError (err, SDL_TRUE);
      else   
         return (0);     
     }  
#endif
                  
   return (-1);  
}

static int DSp_AllocHWSurface(_THIS, SDL_Surface *surface)
{
	GWorldPtr temp;
	 	 
	if ( DSp_NewHWSurface (this, &temp, surface->format->BitsPerPixel, surface->w, surface->h) < 0 )
	   return (-1);
			
	surface->hwdata = (private_hwdata*) SDL_malloc (sizeof (private_hwdata));
	if (surface->hwdata == NULL) {
		SDL_OutOfMemory ();
		return -1;
	}
	
	SDL_memset (surface->hwdata, 0, sizeof(private_hwdata));
	surface->hwdata->offscreen = temp;
	surface->pitch	 = GetPixRowBytes (GetPortPixMap (temp)) & 0x3FFF;
	surface->pixels  = GetPixBaseAddr (GetPortPixMap (temp));
	surface->flags	|= SDL_HWSURFACE;
#ifdef DSP_TRY_CC_AND_AA	
	surface->flags  |= SDL_HWACCEL;
#endif	
	return 0;
}

static void DSp_FreeHWSurface(_THIS, SDL_Surface *surface)
{	
	if (surface->hwdata->offscreen != NULL)
		DisposeGWorld (surface->hwdata->offscreen);
	SDL_free(surface->hwdata);

    surface->pixels = NULL;
}

static int DSp_CheckHWBlit(_THIS, SDL_Surface *src, SDL_Surface *dest)
{
	int accelerated;

	/* Set initial acceleration on */
	src->flags |= SDL_HWACCEL;

	/* Set the surface attributes */
	if ( (src->flags & SDL_SRCALPHA) == SDL_SRCALPHA ) {
		if ( ! this->info.blit_hw_A ) {
			src->flags &= ~SDL_HWACCEL;
		}
	}
	if ( (src->flags & SDL_SRCCOLORKEY) == SDL_SRCCOLORKEY ) {
		if ( ! this->info.blit_hw_CC ) {
			src->flags &= ~SDL_HWACCEL;
		}
	}

	/* Check to see if final surface blit is accelerated */
	accelerated = !!(src->flags & SDL_HWACCEL);
	if ( accelerated ) {
		src->map->hw_blit = DSp_HWAccelBlit;
	}
	return(accelerated);
}

static int DSp_HWAccelBlit(SDL_Surface *src, SDL_Rect *srcrect,
                           SDL_Surface *dst, SDL_Rect *dstrect)
{
	CGrafPtr save_port;
	GDHandle save_device;
	Rect src_rect, dst_rect;
    RGBColor black = { 0, 0, 0 };
    RGBColor white = { 0xFFFF, 0xFFFF, 0xFFFF };

#ifdef DSP_TRY_CC_AND_AA		
	UInt32 mode;	
#endif
	
	SetRect (&src_rect, srcrect->x, srcrect->y, srcrect->x + srcrect->w, srcrect->y + srcrect->h);
	SetRect (&dst_rect, dstrect->x, dstrect->y, dstrect->x + dstrect->w, dstrect->y + dstrect->h);
	
	GetGWorld (&save_port, &save_device);
	SetGWorld (dst->hwdata->offscreen, NULL);		
		
	RGBForeColor (&black);
	RGBBackColor (&white);
		
#ifdef DSP_TRY_CC_AND_AA
	
	if ( (src->flags & SDL_SRCCOLORKEY) &&
	     (src->flags & SDL_SRCALPHA)  ) {
	     
	     OpColor (&(src->hwdata->alpha));	           
    
         CopyDeepMask ( GetPortBitMapForCopyBits(src->hwdata->offscreen),
                        GetPortBitMapForCopyBits(src->hwdata->mask),
                        GetPortBitMapForCopyBits(dst->hwdata->offscreen),
	                    &src_rect, &src_rect, &dst_rect,
	                    blend,
	                    NULL );                	                    
	}
	else {
    	
    	if ( src->flags & SDL_SRCCOLORKEY) {		    	    	    
    	    RGBBackColor (&(src->hwdata->trans) );	    
    	    mode = transparent;
    	}
    	else if (src->flags & SDL_SRCALPHA) {
    	
    	    OpColor (&(src->hwdata->alpha));
    	    mode = blend;
    	}    	
    	else {
    	
    	    mode = srcCopy;   	    
    	}            
    	
        CopyBits ( GetPortBitMapForCopyBits(src->hwdata->offscreen),
                   GetPortBitMapForCopyBits(dst->hwdata->offscreen),
    	           &src_rect, &dst_rect, mode, NULL );
    }	
#else
    
    CopyBits ( &(((GrafPtr)(src->hwdata->offscreen))->portBits),
    	           &(((GrafPtr)(dst->hwdata->offscreen))->portBits),
    	           &src_rect, &dst_rect, srcCopy, NULL );

#endif /* DSP_TRY_CC_AND_AA */           
	                             
	SetGWorld (save_port, save_device);

	return(0);
}

static int DSp_FillHWRect(_THIS, SDL_Surface *dst, SDL_Rect *rect, Uint32 color)
{
	CGrafPtr save_port;
	GDHandle save_device;
	Rect     fill_rect;
	RGBColor rgb;
		
	SetRect (&fill_rect, rect->x, rect->y, rect->x + rect->w, rect->y + rect->h);
	
	GetGWorld (&save_port, &save_device);
	SetGWorld (dst->hwdata->offscreen, NULL);

    Index2Color (color, &rgb);
    
	RGBForeColor (&rgb);
	PaintRect (&fill_rect);

	SetGWorld (save_port, save_device);    

	return(0);
}

static int DSp_FlipHWSurface(_THIS, SDL_Surface *surface)
{
	  if ( (surface->flags & SDL_HWSURFACE) ) {
		CGrafPtr dsp_front_buffer, save_port;
		Rect rect;
		
    #if ! TARGET_API_MAC_OSX
		unsigned int old_count;
	#endif
    	
		/* pseudo page flipping for VRAM back buffer*/ 
		DSpContext_GetFrontBuffer (dsp_context, &dsp_front_buffer);
		SetRect (&rect, 0, 0, surface->w-1, surface->h-1);  	
  		
  		GetPort ((GrafPtr *)&save_port);
  		SetPort ((GrafPtr)dsp_front_buffer);
  		
  		/* wait for retrace */
  		/* I have tried doing the swap in interrupt routine (VBL Proc) to do */
  		/* it asynchronously, but apparently CopyBits isn't interrupt safe  */		   
        
            #if ! TARGET_API_MAC_OSX
		#ifndef DSP_NO_SYNC_VBL
    		old_count = retrace_count;
    		while (old_count == retrace_count)
    			  ;
		#endif				  
            #endif
  		
          CopyBits ( GetPortBitMapForCopyBits(dsp_back_buffer),
                      GetPortBitMapForCopyBits(dsp_front_buffer),
  			   &rect, &rect, srcCopy, NULL );
  	
  		SetPort ((GrafPtr)save_port);
  		
	} else {
		/* not really page flipping at all: DSp just blits the dirty rectangles from DSp_UpdateRects */	    
		Boolean busy_flag;
		DSpContext_SwapBuffers (dsp_context, NULL, &busy_flag); /* this  waits for VBL */
		DSpContext_GetBackBuffer (dsp_context, kDSpBufferKind_Normal, &dsp_back_buffer);
        surface->pixels =  GetPixBaseAddr( GetPortPixMap(dsp_back_buffer) );
	}
	return(0);
}

static int DSp_LockHWSurface(_THIS, SDL_Surface *surface)
{
	if ( LockPixels (GetGWorldPixMap (surface->hwdata->offscreen)) )
		return 0;
	else
		return -1;
}

static void DSp_UnlockHWSurface(_THIS, SDL_Surface *surface)
{
	UnlockPixels (GetGWorldPixMap (surface->hwdata->offscreen));
}

static void DSp_DirectUpdate(_THIS, int numrects, SDL_Rect *sdl_rects)
{
	return;
}

static void DSp_DSpUpdate(_THIS, int numrects, SDL_Rect *sdl_rects)
{
#if ! TARGET_API_MAC_OSX /* Unsupported DSp in here */
	int i;
	Rect rect;
	
	for (i = 0; i < numrects; i++) {
	
		rect.top    = sdl_rects[i].y;
		rect.left   = sdl_rects[i].x;
		rect.bottom = sdl_rects[i].h + sdl_rects[i].y;
		rect.right  = sdl_rects[i].w + sdl_rects[i].x;
		
		DSpContext_InvalBackBufferRect (dsp_context, &rect);		
	}
#endif
}

static int DSp_CreatePalette(_THIS) {


	/* Create our palette */
	SDL_CTab = (CTabHandle)NewHandle(sizeof(ColorSpec)*256 + 8);
	if ( SDL_CTab == nil ) {
		SDL_OutOfMemory();
		return(-1);
	}
	(**SDL_CTab).ctSeed = GetCTSeed();
	(**SDL_CTab).ctFlags = 0;
	(**SDL_CTab).ctSize = 255;
	CTabChanged(SDL_CTab);
	SDL_CPal = NewPalette(256, SDL_CTab, pmExplicit+pmTolerant, 0);
	
	return 0;
}

static int DSp_DestroyPalette(_THIS) {

	/* Free palette and restore original one */
	if ( SDL_CTab != nil ) {
		DisposeHandle((Handle)SDL_CTab);
		SDL_CTab = nil;
	}
	if ( SDL_CPal != nil ) {
		DisposePalette(SDL_CPal);
		SDL_CPal = nil;
	}
	RestoreDeviceClut(SDL_Display);
	
   return (0);
}

static int DSp_SetColors(_THIS, int firstcolor, int ncolors, SDL_Color *colors)
{
	CTabHandle   cTab;
	
	int i;

	cTab = SDL_CTab;
	
	/* Verify the range of colors */
	if ( (firstcolor+ncolors) > ((**cTab).ctSize+1) ) {
		return(0);
	}
	
	/* Set the screen palette and update the display */
	for(i = 0; i < ncolors; i++) {
	        int j = firstcolor + i;
	        (**cTab).ctTable[j].value = j;
		(**cTab).ctTable[j].rgb.red = colors[i].r << 8 | colors[i].r;
		(**cTab).ctTable[j].rgb.green = colors[i].g << 8 | colors[i].g;
		(**cTab).ctTable[j].rgb.blue = colors[i].b << 8 | colors[i].b;
	}
	
	SetGDevice(SDL_Display);
	SetEntries(0, (**cTab).ctSize, (ColorSpec *)&(**cTab).ctTable);

	return(1);
}

void DSp_VideoQuit(_THIS)
{
	int i;
	
	/* Free current video mode */
	DSp_UnsetVideoMode(this, this->screen);

	/* Free Palette and restore original */
	DSp_DestroyPalette (this);

#if SDL_MACCLASSIC_GAMMA_SUPPORT
	Mac_QuitGamma(this);
#endif

	/* Free list of video modes */
	if ( SDL_modelist != NULL ) {
		for ( i=0; SDL_modelist[i]; i++ ) {
			SDL_free(SDL_modelist[i]);
		}
		SDL_free(SDL_modelist);
		SDL_modelist = NULL;
	}
	
	/* Unload DrawSprocket */
	DSpShutdown ();
}

#if SDL_VIDEO_OPENGL

/* swap buffers with v-sync */
static void DSp_GL_SwapBuffers (_THIS) {

   #ifndef DSP_NO_SYNC_OPENGL
   
       unsigned int old_count;
          
       old_count = retrace_count;
       while (old_count == retrace_count)
          ;
   #endif
      
   aglSwapBuffers (glContext);
}

#endif
