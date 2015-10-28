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

/* Pocket PC GAPI SDL video driver implementation;
Implemented by Dmitry Yakimov - support@activekitten.com
Inspired by http://arisme.free.fr/ports/SDL.php
*/

// TODO: copy surface on window when lost focus
// TODO: test buttons rotation
// TODO: test on be300 and HPC ( check WinDib fullscreen keys catching )
// TODO: test on smartphones
// TODO: windib on SH3 PPC2000 landscape test
// TODO: optimize 8bpp landscape mode

// there is some problems in runnings apps from a device landscape mode
// due to WinCE bugs. Some works and some - does not.
// TODO: finish Axim Dell X30 and user landscape mode, device landscape mode
// TODO: finish Axim Dell X30 user portrait, device landscape stylus conversion
// TODO: fix running GAPI apps from landscape mode - 
//       wince goes to portrait mode, but does not update video memory


#include "SDL.h"
#include "SDL_error.h"
#include "SDL_video.h"
#include "SDL_mouse.h"
#include "../SDL_sysvideo.h"
#include "../SDL_pixels_c.h"
#include "../../events/SDL_events_c.h"
#include "../wincommon/SDL_syswm_c.h"
#include "../wincommon/SDL_sysmouse_c.h"
#include "../windib/SDL_dibevents_c.h" 

#include "../windib/SDL_gapidibvideo.h"
#include "SDL_gapivideo.h"

#define gapi this->hidden->gapiInfo

#define GAPIVID_DRIVER_NAME "gapi"

#if defined(DEBUG) || defined (_DEBUG) || defined(NDEBUG)
#define REPORT_VIDEO_INFO 1
#else
#define REPORT_VIDEO_INFO 0
#endif

// for testing with GapiEmu
#define USE_GAPI_EMU 0
#define EMULATE_AXIM_X30 0
#define WITHOUT_GAPI 0

#if USE_GAPI_EMU && !REPORT_VIDEO_INFO
#pragma message("Warning: Using GapiEmu in release build. I assume you'd like to set USE_GAPI_EMU to zero.")
#endif

#ifndef _T
#define _T(x) L##x
#endif

#ifndef ASSERT
#define ASSERT(x)
#endif

// defined and used in SDL_sysevents.c
extern HINSTANCE aygshell;
extern void SDL_UnregisterApp();
extern int DIB_AddMode(_THIS, int bpp, int w, int h);

/* Initialization/Query functions */
static int GAPI_VideoInit(_THIS, SDL_PixelFormat *vformat);
static SDL_Rect **GAPI_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags);
static SDL_Surface *GAPI_SetVideoMode(_THIS, SDL_Surface *current, int width, int height, int bpp, Uint32 flags);
static int GAPI_SetColors(_THIS, int firstcolor, int ncolors, SDL_Color *colors);
static void GAPI_VideoQuit(_THIS);

/* Hardware surface functions */
static int GAPI_AllocHWSurface(_THIS, SDL_Surface *surface);
static int GAPI_LockHWSurface(_THIS, SDL_Surface *surface);
static void GAPI_UnlockHWSurface(_THIS, SDL_Surface *surface);
static void GAPI_FreeHWSurface(_THIS, SDL_Surface *surface);

/* Windows message handling functions, will not be processed */
static void GAPI_Activate(_THIS, BOOL active, BOOL minimized);
static void GAPI_RealizePalette(_THIS);
static void GAPI_PaletteChanged(_THIS, HWND window);
static void GAPI_WinPAINT(_THIS, HDC hdc); 

/* etc. */
static void GAPI_UpdateRects(_THIS, int numrects, SDL_Rect *rects);

static HMODULE g_hGapiLib = 0;
#define LINK(type,name,import) \
	if( g_hGapiLib ) \
		name = (PFN##type)GetProcAddress( g_hGapiLib, _T(import) ); 

static char g_bRawBufferAvailable = 0;

/* GAPI driver bootstrap functions */

/* hi res definitions */
typedef struct _RawFrameBufferInfo
{
   WORD wFormat;
   WORD wBPP;
   VOID *pFramePointer;
   int  cxStride;
   int  cyStride;
   int  cxPixels;
   int  cyPixels;
} RawFrameBufferInfo; 

static struct _RawFrameBufferInfo g_RawFrameBufferInfo = {0};

#define GETRAWFRAMEBUFFER   0x00020001

#define FORMAT_565 1
#define FORMAT_555 2
#define FORMAT_OTHER 3

/* Dell Axim x30 hangs when we use GAPI from landscape,
   so lets avoid using GxOpenDisplay there via GETGXINFO trick 
   It seems that GAPI subsystem use the same ExtEscape code.
*/
#define GETGXINFO 0x00020000

typedef struct GXDeviceInfo
{
long Version; //00 (should filled with 100 before calling ExtEscape)
void * pvFrameBuffer; //04
unsigned long cbStride; //08
unsigned long cxWidth; //0c
unsigned long cyHeight; //10
unsigned long cBPP; //14
unsigned long ffFormat; //18
char Unused[0x84-7*4];
} GXDeviceInfo; 

static int GAPI_Available(void)
{
	// try to use VGA display, even on emulator
	HDC hdc = GetDC(NULL); 
	int result = ExtEscape(hdc, GETRAWFRAMEBUFFER, 0, NULL, sizeof(RawFrameBufferInfo), (char *)&g_RawFrameBufferInfo);
	ReleaseDC(NULL, hdc);
	g_bRawBufferAvailable = result > 0;

	//My Asus MyPAL 696 reports the RAWFRAMEBUFFER as available, but with a size of 0 x 0
	if(g_RawFrameBufferInfo.cxPixels <= 0 || g_RawFrameBufferInfo.cyPixels <= 0){
		g_bRawBufferAvailable = 0;
	}

#if WITHOUT_GAPI
	return g_bRawBufferAvailable;
#endif

#if USE_GAPI_EMU
	g_hGapiLib = LoadLibrary(_T("GAPI_Emu.dll"));
	if( !g_hGapiLib )
	{
		SDL_SetError("Gapi Emu not found!");
	}
	return g_hGapiLib != 0;
#endif

	// try to find gx.dll
	g_hGapiLib = LoadLibrary(_T("\\Windows\\gx.dll"));
	if( !g_hGapiLib )
	{
		g_hGapiLib = LoadLibrary(_T("gx.dll"));
		if( !g_hGapiLib ) return g_bRawBufferAvailable;
	}

	return(1);
}

static int cmpmodes(const void *va, const void *vb)
{
    SDL_Rect *a = *(SDL_Rect **)va;
    SDL_Rect *b = *(SDL_Rect **)vb;
    if ( a->w == b->w )
        return b->h - a->h;
    else
        return b->w - a->w;
}

static int GAPI_AddMode(_THIS, int bpp, int w, int h)
{
	SDL_Rect *mode;
	int i, index;
	int next_mode;

	/* Check to see if we already have this mode */
	if ( bpp < 8 ) {  /* Not supported */
		return(0);
	}
	index = ((bpp+7)/8)-1;
	for ( i=0; i<gapi->SDL_nummodes[index]; ++i ) {
		mode = gapi->SDL_modelist[index][i];
		if ( (mode->w == w) && (mode->h == h) ) {
			return(0);
		}
	}

	/* Set up the new video mode rectangle */
	mode = (SDL_Rect *)SDL_malloc(sizeof *mode);
	if ( mode == NULL ) {
		SDL_OutOfMemory();
		return(-1);
	}
	mode->x = 0;
	mode->y = 0;
	mode->w = w;
	mode->h = h;

	/* Allocate the new list of modes, and fill in the new mode */
	next_mode = gapi->SDL_nummodes[index];
	gapi->SDL_modelist[index] = (SDL_Rect **)
	       SDL_realloc(gapi->SDL_modelist[index], (1+next_mode+1)*sizeof(SDL_Rect *));
	if ( gapi->SDL_modelist[index] == NULL ) {
		SDL_OutOfMemory();
		gapi->SDL_nummodes[index] = 0;
		SDL_free(mode);
		return(-1);
	}
	gapi->SDL_modelist[index][next_mode] = mode;
	gapi->SDL_modelist[index][next_mode+1] = NULL;
	gapi->SDL_nummodes[index]++;

	return(0);
}

static void GAPI_DeleteDevice(SDL_VideoDevice *device)
{
	if( g_hGapiLib )
	{
		FreeLibrary(g_hGapiLib);
		g_hGapiLib = 0;
	}
	SDL_free(device->hidden->gapiInfo);
	SDL_free(device->hidden);
	SDL_free(device);
}

static SDL_VideoDevice *GAPI_CreateDevice(int devindex)
{
	SDL_VideoDevice *device;

	if( !g_hGapiLib && !g_bRawBufferAvailable)
	{
		if( !GAPI_Available() )
		{
			SDL_SetError("GAPI dll is not found and VGA mode is not available!");
			return 0;
		}
	}

	/* Initialize all variables that we clean on shutdown */
	device = (SDL_VideoDevice *)SDL_malloc(sizeof(SDL_VideoDevice));
	if ( device ) {
		SDL_memset(device, 0, (sizeof *device));
		device->hidden = (struct SDL_PrivateVideoData *)
				SDL_malloc((sizeof *device->hidden));
		if(device->hidden){
			SDL_memset(device->hidden, 0, (sizeof *device->hidden));
			device->hidden->gapiInfo = (GapiInfo *)SDL_malloc((sizeof(GapiInfo)));
			if(device->hidden->gapiInfo == NULL)
			{
				SDL_free(device->hidden);
				device->hidden = NULL;
			}
		}
	}
	if ( (device == NULL) || (device->hidden == NULL) ) {
		SDL_OutOfMemory();
		if ( device ) {
			SDL_free(device);
		}
		return(0);
	}
	SDL_memset(device->hidden->gapiInfo, 0, (sizeof *device->hidden->gapiInfo));

	/* Set the function pointers */
	device->VideoInit = GAPI_VideoInit;
	device->ListModes = GAPI_ListModes;
	device->SetVideoMode = GAPI_SetVideoMode;
	device->UpdateMouse = WIN_UpdateMouse; 
	device->CreateYUVOverlay = NULL;
	device->SetColors = GAPI_SetColors;
	device->UpdateRects = GAPI_UpdateRects;
	device->VideoQuit = GAPI_VideoQuit;
	device->AllocHWSurface = GAPI_AllocHWSurface;
	device->CheckHWBlit = NULL;
	device->FillHWRect = NULL;
	device->SetHWColorKey = NULL;
	device->SetHWAlpha = NULL;
	device->LockHWSurface = GAPI_LockHWSurface;
	device->UnlockHWSurface = GAPI_UnlockHWSurface;
	device->FlipHWSurface = NULL;
	device->FreeHWSurface = GAPI_FreeHWSurface;
	device->SetCaption = WIN_SetWMCaption;
	device->SetIcon = WIN_SetWMIcon;
	device->IconifyWindow = WIN_IconifyWindow;
	device->GrabInput = WIN_GrabInput;
	device->GetWMInfo = WIN_GetWMInfo;
	device->FreeWMCursor = WIN_FreeWMCursor;
	device->CreateWMCursor = WIN_CreateWMCursor; 
	device->ShowWMCursor = WIN_ShowWMCursor;	
	device->WarpWMCursor = WIN_WarpWMCursor; 
    device->CheckMouseMode = WIN_CheckMouseMode;
	device->InitOSKeymap = DIB_InitOSKeymap;
	device->PumpEvents = DIB_PumpEvents;

	/* Set up the windows message handling functions */
	WIN_Activate = GAPI_Activate;
	WIN_RealizePalette = GAPI_RealizePalette;
	WIN_PaletteChanged = GAPI_PaletteChanged;
	WIN_WinPAINT = GAPI_WinPAINT;
	HandleMessage = DIB_HandleMessage; 

	device->free = GAPI_DeleteDevice;

	/* Load gapi library */
#define gx device->hidden->gapiInfo->gxFunc

    LINK( GXOpenDisplay, gx.GXOpenDisplay,         "?GXOpenDisplay@@YAHPAUHWND__@@K@Z" )
    LINK( GXCloseDisplay, gx.GXCloseDisplay,        "?GXCloseDisplay@@YAHXZ" )
    LINK( GXBeginDraw, gx.GXBeginDraw,           "?GXBeginDraw@@YAPAXXZ" )
    LINK( GXEndDraw, gx.GXEndDraw,             "?GXEndDraw@@YAHXZ" )
    LINK( GXOpenInput, gx.GXOpenInput,           "?GXOpenInput@@YAHXZ" )
    LINK( GXCloseInput, gx.GXCloseInput,          "?GXCloseInput@@YAHXZ" )
    LINK( GXGetDisplayProperties, gx.GXGetDisplayProperties,"?GXGetDisplayProperties@@YA?AUGXDisplayProperties@@XZ" )
    LINK( GXGetDefaultKeys, gx.GXGetDefaultKeys,      "?GXGetDefaultKeys@@YA?AUGXKeyList@@H@Z" )
    LINK( GXSuspend, gx.GXSuspend,             "?GXSuspend@@YAHXZ" )
    LINK( GXResume, gx.GXResume,              "?GXResume@@YAHXZ" )
    LINK( GXSetViewport, gx.GXSetViewport,         "?GXSetViewport@@YAHKKKK@Z" )
    LINK( GXIsDisplayDRAMBuffer, gx.GXIsDisplayDRAMBuffer, "?GXIsDisplayDRAMBuffer@@YAHXZ" )

	/* wrong gapi.dll */
	if( !gx.GXOpenDisplay )
	{
		if( g_hGapiLib ) 
		{
			FreeLibrary(g_hGapiLib);
			g_hGapiLib = 0;
		}
	}
	
	if( !gx.GXOpenDisplay && !g_bRawBufferAvailable)
	{
		SDL_SetError("Error: damaged or unknown gapi.dll!\n");
		GAPI_DeleteDevice(device);
		return 0;
	}

	return device;
}

VideoBootStrap GAPI_bootstrap = {
	GAPIVID_DRIVER_NAME, "WinCE GAPI video driver",
	GAPI_Available, GAPI_CreateDevice
};

static void FillStructs(_THIS, BOOL useVga)
{
#ifdef _ARM_
	WCHAR oemstr[100];
#endif
	/* fill a device properties */

	if( !useVga )
	{
		gapi->gxProperties = gapi->gxFunc.GXGetDisplayProperties();
		gapi->needUpdate = 1;
		gapi->hiresFix = 0;
		gapi->useVga = 0;
		gapi->useGXOpenDisplay = 1;

#ifdef _ARM_
		/* check some devices and extract addition info */
		SystemParametersInfo( SPI_GETOEMINFO, sizeof( oemstr ), oemstr, 0 );

		// buggy iPaq38xx
		if ((oemstr[12] == 'H') && (oemstr[13] == '3') && (oemstr[14] == '8') && (gapi->gxProperties.cbxPitch > 0)) 
		{
			gapi->videoMem = (PIXEL*)0xac0755a0;
			gapi->gxProperties.cbxPitch = -640;
			gapi->gxProperties.cbyPitch = 2;
			gapi->needUpdate = 0;
		}
#if (EMULATE_AXIM_X30 == 0)
		// buggy Dell Axim X30
		if( _tcsncmp(oemstr, L"Dell Axim X30", 13) == 0 )
#endif
		{
			GXDeviceInfo gxInfo = {0};
			HDC hdc = GetDC(NULL);
			int result;

			gxInfo.Version = 100;
			result = ExtEscape(hdc, GETGXINFO, 0, NULL, sizeof(gxInfo), (char *)&gxInfo);
			if( result > 0 )
			{
				gapi->useGXOpenDisplay = 0;
				gapi->videoMem = gxInfo.pvFrameBuffer;
				gapi->needUpdate = 0;
				gapi->gxProperties.cbxPitch = 2;
				gapi->gxProperties.cbyPitch = 480;
				gapi->gxProperties.cxWidth = gxInfo.cxWidth;
				gapi->gxProperties.cyHeight = gxInfo.cyHeight;
				gapi->gxProperties.ffFormat = gxInfo.ffFormat;
			}
		}
#endif
	} else
	{
		gapi->needUpdate = 0;		
		gapi->hiresFix = 0;
		gapi->gxProperties.cBPP = g_RawFrameBufferInfo.wBPP;
		gapi->gxProperties.cbxPitch = g_RawFrameBufferInfo.cxStride;
		gapi->gxProperties.cbyPitch = g_RawFrameBufferInfo.cyStride;
		gapi->gxProperties.cxWidth = g_RawFrameBufferInfo.cxPixels;
		gapi->gxProperties.cyHeight = g_RawFrameBufferInfo.cyPixels;
		gapi->videoMem = g_RawFrameBufferInfo.pFramePointer;
		gapi->useVga = 1;

		switch( g_RawFrameBufferInfo.wFormat )
		{
		case FORMAT_565:
			gapi->gxProperties.ffFormat = kfDirect565;
			break;
		case FORMAT_555:
			gapi->gxProperties.ffFormat = kfDirect555;
			break;
		default:
			/* unknown pixel format, try define by BPP! */
			switch( g_RawFrameBufferInfo.wBPP )
			{
			case 4:
			case 8:
				gapi->gxProperties.ffFormat = kfDirect;
			case 16:
				gapi->gxProperties.ffFormat = kfDirect565;
			default:
				gapi->gxProperties.ffFormat = kfDirect;
				break;
			}
		}
	}

	if( gapi->gxProperties.cBPP != 16 )
	{
		gapi->gapiOrientation = SDL_ORIENTATION_UP;
	} else
	if( (gapi->gxProperties.cbxPitch > 0) && (gapi->gxProperties.cbyPitch > 0 ))
	{
		gapi->gapiOrientation = SDL_ORIENTATION_UP;
	} else
	if( (gapi->gxProperties.cbxPitch > 0) && (gapi->gxProperties.cbyPitch < 0 ))
	{
		gapi->gapiOrientation = SDL_ORIENTATION_RIGHT; // ipaq 3660
	} else
	if( (gapi->gxProperties.cbxPitch < 0) && (gapi->gxProperties.cbyPitch > 0 ))
	{
		gapi->gapiOrientation = SDL_ORIENTATION_LEFT; // ipaq 3800
	}
}

static void GAPI_CreatePalette(int ncolors, SDL_Color *colors)
{
  // Setup a custom color palette
   BYTE buffer[ sizeof(LOGPALETTE) + 255 * sizeof(PALETTEENTRY) ];
   int i;
   LOGPALETTE*   pLogical = (LOGPALETTE*)buffer;
   PALETTEENTRY* entries  = pLogical->palPalEntry;
   HPALETTE hPalette;
   HDC hdc;

   for (i = 0; i < ncolors; ++i)
   {
      // Find intensity by replicating the bit patterns over a byte
      entries[i].peRed   = colors[i].r;
      entries[i].peGreen = colors[i].g;
      entries[i].peBlue  = colors[i].b;
      entries[i].peFlags = 0;
   }

   // Create the GDI palette object
   pLogical->palVersion    = 0x0300;
   pLogical->palNumEntries = ncolors;

   hPalette = CreatePalette( pLogical );
   ASSERT(hPalette);
	

   // Realize the palette
   hdc = GetDC(0);

   SelectPalette( hdc, hPalette, FALSE );
   RealizePalette( hdc );

   ReleaseDC( 0, hdc );
   DeleteObject( hPalette );
}

int GAPI_VideoInit(_THIS, SDL_PixelFormat *vformat)
{
	int i,bpp;

	/* Create the window */
	if ( DIB_CreateWindow(this) < 0 ) {
		return(-1);
	}

	if( g_hGapiLib )
	{
		FillStructs(this, 0);

		// SDL does not supports 2/4bpp mode, so use 16 bpp
		bpp = gapi->gxProperties.cBPP < 8 ? 16 : gapi->gxProperties.cBPP;
		
		/* set up normal and landscape mode */
		GAPI_AddMode(this, bpp, gapi->gxProperties.cyHeight, gapi->gxProperties.cxWidth);	
		GAPI_AddMode(this, bpp, gapi->gxProperties.cxWidth, gapi->gxProperties.cyHeight);	
	}

	/* add hi-res mode */
	if( g_bRawBufferAvailable && 
		!((gapi->gxProperties.cxWidth == (unsigned)g_RawFrameBufferInfo.cxPixels) && (gapi->gxProperties.cyHeight == (unsigned)g_RawFrameBufferInfo.cyPixels)))
	{
		FillStructs(this, 1);

		// SDL does not supports 2/4bpp mode, so use 16 bpp
		bpp = gapi->gxProperties.cBPP < 8 ? 16 : gapi->gxProperties.cBPP;

		/* set up normal and landscape mode */
		GAPI_AddMode(this, bpp, gapi->gxProperties.cyHeight, gapi->gxProperties.cxWidth);	
		GAPI_AddMode(this, bpp, gapi->gxProperties.cxWidth, gapi->gxProperties.cyHeight);	
	}

	/* Determine the current screen size.
	 * This is NOT necessarily the size of the Framebuffer or GAPI, as they return
	 * the displaysize in ORIENTATION_UP */
	this->info.current_w = GetSystemMetrics(SM_CXSCREEN);
	this->info.current_h = GetSystemMetrics(SM_CYSCREEN);

	/* Sort the mode lists */
	for ( i=0; i<NUM_MODELISTS; ++i ) {
		if ( gapi->SDL_nummodes[i] > 0 ) {
			SDL_qsort(gapi->SDL_modelist[i], gapi->SDL_nummodes[i], sizeof *gapi->SDL_modelist[i], cmpmodes);
		}
	}

	vformat->BitsPerPixel = gapi->gxProperties.cBPP < 8 ? 16 : (unsigned char)gapi->gxProperties.cBPP;

	// Get color mask
	if (gapi->gxProperties.ffFormat & kfDirect565) {
		vformat->BitsPerPixel = 16;
		vformat->Rmask = 0x0000f800;
		vformat->Gmask = 0x000007e0;
		vformat->Bmask = 0x0000001f;
		gapi->videoMode = GAPI_DIRECT_565;
	}
	else
	if (gapi->gxProperties.ffFormat & kfDirect555) {
		vformat->BitsPerPixel = 16;
		vformat->Rmask = 0x00007c00;
		vformat->Gmask = 0x000003e0;
		vformat->Bmask = 0x0000001f;
		gapi->videoMode = GAPI_DIRECT_555;
	}
	else
	if ((gapi->gxProperties.ffFormat & kfDirect) && (gapi->gxProperties.cBPP < 8)) {
		// We'll perform the conversion
		vformat->BitsPerPixel = 16;
		vformat->Rmask = 0x0000f800; // 16 bit 565
		vformat->Gmask = 0x000007e0;
		vformat->Bmask = 0x0000001f;
		if (gapi->gxProperties.ffFormat & kfDirectInverted)
			gapi->invert = (1 << gapi->gxProperties.cBPP) - 1;
		gapi->colorscale = gapi->gxProperties.cBPP < 8 ? 8 - gapi->gxProperties.cBPP : 0;
		gapi->videoMode = GAPI_MONO;
	}
	else
	if (gapi->gxProperties.ffFormat & kfPalette) {
		gapi->videoMode = GAPI_PALETTE;
	} 

	/* We're done! */
	return(0);
}

SDL_Rect **GAPI_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags)
{
	return(gapi->SDL_modelist[((format->BitsPerPixel+7)/8)-1]);
//  	 return (SDL_Rect **) -1;
}

SDL_Surface *GAPI_SetVideoMode(_THIS, SDL_Surface *current,
				int width, int height, int bpp, Uint32 flags)
{
	SDL_Surface *video; 
	Uint32 Rmask, Gmask, Bmask; 
	DWORD style; 
	SDL_Rect allScreen;

	if( bpp < 4 )
	{
		SDL_SetError("1 bpp and 2 bpp modes is not implemented yet!");
		return 0;
	}

	/* Recalculate bitmasks if necessary */
	if (bpp == current->format->BitsPerPixel) {
		video = current;
	}
	else {
		switch(bpp) {
			case 8:
				Rmask = 0;
				Gmask = 0;
				Bmask = 0;
				break;
			case 15:				
			case 16:
				/* Default is 565 unless the display is specifically 555 */
				if (gapi->gxProperties.ffFormat & kfDirect555) {
					Rmask = 0x00007c00;
					Gmask = 0x000003e0;
					Bmask = 0x0000001f;
				}
				else {
					Rmask = 0x0000f800;
					Gmask = 0x000007e0;
					Bmask = 0x0000001f;
				}
				break;
			case 24:
			case 32:
				Rmask = 0x00ff0000;
				Gmask = 0x0000ff00;
				Bmask = 0x000000ff;
				break;
			default:
				SDL_SetError("Unsupported Bits Per Pixel format requested");
				return NULL;
		}
		video = SDL_CreateRGBSurface(SDL_SWSURFACE,
					0, 0, bpp, Rmask, Gmask, Bmask, 0);
		if ( video == NULL ) {
			SDL_OutOfMemory();
			return(NULL);
		}
	}

	gapi->userOrientation = SDL_ORIENTATION_UP;
	gapi->systemOrientation = SDL_ORIENTATION_UP;
	video->flags = SDL_FULLSCREEN;	/* Clear flags, GAPI supports fullscreen only */

	/* GAPI or VGA? */
	if( g_hGapiLib )
	{
		FillStructs(this, 0);
		if( (((unsigned)width != gapi->gxProperties.cxWidth) || ((unsigned)height != gapi->gxProperties.cyHeight))
			&& (((unsigned)width != gapi->gxProperties.cyHeight) || ((unsigned)height != gapi->gxProperties.cxWidth)))
			FillStructs(this, 1); // gapi is found but we use VGA resolution			
	} else
		FillStructs(this, 1);

	if ( !gapi->needUpdate && !gapi->videoMem) {
		SDL_SetError("Couldn't get address of video memory, may be unsupported device or bug");
		return(NULL);
	}

	/* detect user landscape mode */
       if( (width > height) && (gapi->gxProperties.cxWidth < gapi->gxProperties.cyHeight))
		gapi->userOrientation = SDL_ORIENTATION_RIGHT;

       if(GetSystemMetrics(SM_CYSCREEN) < GetSystemMetrics(SM_CXSCREEN))
		gapi->systemOrientation = SDL_ORIENTATION_RIGHT;

	gapi->hiresFix = 0;

	/* check hires */
	if(GetSystemMetrics(SM_CXSCREEN) < width && GetSystemMetrics(SM_CYSCREEN) < height)
	{
	    gapi->hiresFix = 1;
	}

	switch( gapi->userOrientation )
	{
	case SDL_ORIENTATION_UP:
		gapi->startOffset = 0;
		gapi->dstLineStep = gapi->gxProperties.cbyPitch;
		gapi->dstPixelStep = gapi->gxProperties.cbxPitch;
		break;
	case SDL_ORIENTATION_RIGHT:
		switch( gapi->gapiOrientation )
		{
		case SDL_ORIENTATION_UP:
		case SDL_ORIENTATION_RIGHT:
		case SDL_ORIENTATION_LEFT:
			if( (gapi->videoMode == GAPI_MONO) )
				gapi->startOffset = -gapi->gxProperties.cbxPitch + 1; // monochrome mode
			else
				gapi->startOffset = gapi->gxProperties.cbyPitch * (gapi->gxProperties.cyHeight - 1);
				
			gapi->dstLineStep = gapi->gxProperties.cbxPitch;
			gapi->dstPixelStep = -gapi->gxProperties.cbyPitch;
			break;
		}
	}

	video->w = gapi->w = width;
	video->h = gapi->h = height;
	video->pitch = SDL_CalculatePitch(video); 

	/* Small fix for WinCE/Win32 - when activating window
	   SDL_VideoSurface is equal to zero, so activating code
	   is not called properly for fullscreen windows because
	   macros WINDIB_FULLSCREEN uses SDL_VideoSurface
	*/
	SDL_VideoSurface = video;

	/* GAPI is always fullscreen, title bar is useless */
	style = 0;

	if (!SDL_windowid)
		SetWindowLong(SDL_Window, GWL_STYLE, style);

	/* Allocate bitmap */
	if( gapi->buffer ) 
	{
		SDL_free( gapi->buffer );
		gapi->buffer = NULL;
	}
	gapi->buffer = SDL_malloc(video->h * video->pitch);
	video->pixels = gapi->buffer; 

	if ( ! gapi->buffer ) {
		SDL_SetError("Couldn't allocate buffer for requested mode");
		return(NULL);
	}

	SDL_memset(gapi->buffer, 255, video->h * video->pitch);
	MoveWindow(SDL_Window, 0, 0, GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN), FALSE);
	ShowWindow(SDL_Window, SW_SHOW);
	SetForegroundWindow(SDL_Window);

	/* JC 14 Mar 2006
		Flush the message loop or this can cause big problems later
		Especially if the user decides to use dialog boxes or assert()!
	*/
	WIN_FlushMessageQueue();

       /* Open GAPI display */
       if( !gapi->useVga && gapi->useGXOpenDisplay && !gapi->alreadyGXOpened )
       {
#if REPORT_VIDEO_INFO
               printf("system display width  (orig): %d\n", GetSystemMetrics(SM_CXSCREEN));
               printf("system display height (orig): %d\n", GetSystemMetrics(SM_CYSCREEN));
#endif
               gapi->alreadyGXOpened = 1;
		if( !gapi->gxFunc.GXOpenDisplay(SDL_Window, GX_FULLSCREEN) )
		{
			SDL_SetError("Couldn't initialize GAPI");
			return(NULL);
		}
       }

	if(gapi->useVga)
		gapi->coordinateTransform = (4 - gapi->systemOrientation + gapi->userOrientation) % 4;
	else
		gapi->coordinateTransform = gapi->userOrientation;

#if REPORT_VIDEO_INFO
	printf("Video properties:\n");
	printf("display bpp: %d\n", gapi->gxProperties.cBPP);
	printf("display width: %d\n", gapi->gxProperties.cxWidth);
	printf("display height: %d\n", gapi->gxProperties.cyHeight);
       printf("system display width: %d\n", GetSystemMetrics(SM_CXSCREEN));
       printf("system display height: %d\n", GetSystemMetrics(SM_CYSCREEN));
	printf("x pitch: %d\n", gapi->gxProperties.cbxPitch);
	printf("y pitch: %d\n", gapi->gxProperties.cbyPitch);
	printf("gapi flags: 0x%x\n", gapi->gxProperties.ffFormat);
       printf("user orientation: %d\n", gapi->userOrientation);
	printf("system orientation: %d\n", gapi->systemOrientation);
       printf("gapi orientation: %d\n", gapi->gapiOrientation);


	if( !gapi->useVga && gapi->useGXOpenDisplay && gapi->needUpdate)
	{
		gapi->videoMem = gapi->gxFunc.GXBeginDraw(); 
		gapi->gxFunc.GXEndDraw();
	}

	printf("video memory: 0x%x\n", gapi->videoMem);
	printf("need update: %d\n", gapi->needUpdate);
	printf("hi-res fix: %d\n", gapi->hiresFix);
	printf("VGA is available on the device: %d\n", g_bRawBufferAvailable);
	printf("use raw framebuffer: %d\n", gapi->useVga);
	printf("video surface bpp: %d\n", video->format->BitsPerPixel);
	printf("video surface width: %d\n", video->w);
	printf("video surface height: %d\n", video->h);
	printf("mouse/arrows transformation angle: %d\n", gapi->coordinateTransform);
#endif


	/* Blank screen */
	allScreen.x = allScreen.y = 0;
	allScreen.w = video->w - 1;
	allScreen.h = video->h - 1;
	GAPI_UpdateRects(this, 1, &allScreen);

	/* We're done */
	return(video);
}

/* We don't actually allow hardware surfaces other than the main one */
static int GAPI_AllocHWSurface(_THIS, SDL_Surface *surface)
{
	return(-1);
}
static void GAPI_FreeHWSurface(_THIS, SDL_Surface *surface)
{
	return;
}

/* We need to wait for vertical retrace on page flipped displays */
static int GAPI_LockHWSurface(_THIS, SDL_Surface *surface)
{
	return(0);
}

static void GAPI_UnlockHWSurface(_THIS, SDL_Surface *surface)
{
	return;
}

static int updateLine8to8(_THIS, unsigned char *srcPointer, unsigned char *destPointer, int width, int height, int lines)
{
	if( gapi->dstPixelStep == 1) /* optimized blitting on most devices */
	{
		SDL_memcpy(destPointer, srcPointer, width);
		return 1;
	} else
	{
		// TODO: read 4 pixels, write DWORD
		int step = gapi->dstPixelStep;
		while(width--)
		{
			*destPointer = *srcPointer++;
			destPointer += step;
		}
	}
	return 1;
}

/* Video memory is very slow so lets optimize as much as possible */
static int updateLine16to16(_THIS, PIXEL *srcPointer, PIXEL *destPointer, int width, int height, int lines)
{
	PIXEL *line1, *line2;
	int step = gapi->dstPixelStep / 2;

	if( step == 1 ) /* optimized blitting on most devices */
	{
		SDL_memcpy(destPointer, srcPointer, width * sizeof(PIXEL));
		return 1;
	}
	else
	{
		if( (gapi->gapiOrientation != SDL_ORIENTATION_UP) &&
			(gapi->userOrientation == SDL_ORIENTATION_UP )) // iPaq 3660/3800 and user orientation up
		{	
			// to prevent data misalignment copy only one line
			if( ((((unsigned)destPointer & 3) != 0) && (gapi->gapiOrientation == SDL_ORIENTATION_LEFT)) 
				|| ((((unsigned)destPointer & 3) == 0) && (gapi->gapiOrientation != SDL_ORIENTATION_LEFT))
				|| (lines == 1) ) 
			{
				while(width--)
				{
					*destPointer = *srcPointer++;
					destPointer += step;
				}
				return 1;
			}

			/* read two lines at the same time, write DWORD */
			line1 = srcPointer;
			line2 = srcPointer + SDL_VideoSurface->pitch / 2;

			if( gapi->gapiOrientation == SDL_ORIENTATION_LEFT )
				while(width--) // iPaq 3800
				{
					*(DWORD*)destPointer =(*line2++ << 16) | *line1++;
					destPointer += step;
				}
			else
			{
				destPointer += gapi->gxProperties.cbyPitch / 2;

				while(width--) // iPaq 3660
				{
					*(DWORD*)destPointer =(*line1++ << 16) | *line2++;
					destPointer += step;
				}
			}
			return 2;
		} else
		{
			// iPaq 3800 and user orientation landscape
			if( gapi->gapiOrientation == SDL_ORIENTATION_LEFT )
			{
				int w1;

				// to prevent data misalignment copy only one pixel
				if( (((unsigned)destPointer & 3) == 0) && (width > 0)) 
				{
					*destPointer-- = *srcPointer++;
					width--;
				}

				destPointer--;

				w1 = width / 2;

				while(w1--)
				{
					DWORD p = *(DWORD*)srcPointer;
					*((DWORD*)destPointer) = (p << 16) | (p >> 16);
					destPointer -= 2;
					srcPointer += 2;
				}

				if( width & 1 ) // copy the last pixel
				{
					destPointer++;
					*destPointer = *srcPointer;
				}

				return 1;
			}

			// modern iPaqs and user orientation landscape
			// read two pixels, write DWORD

			line1 = srcPointer;
			line2 = srcPointer + SDL_VideoSurface->pitch / 2;

			if( (((unsigned)destPointer & 3) != 0) || (lines == 1) ) 
			{
				while(width--)
				{
					*destPointer = *srcPointer++;
					destPointer += step;
				}
				return 1;
			}
			
			while(width--)
			{
				*(DWORD*)destPointer =(*line2++ << 16) | *line1++;
				destPointer -= gapi->gxProperties.cbyPitch / 2;
			}
			return 2;
		}
	}
}

// Color component masks for 565
#define REDMASK (31<<11)
#define GREENMASK (63<<5)
#define BLUEMASK (31)


static int updateLine16to4(_THIS, PIXEL *srcPointer, unsigned char *destPointer, int width, int height, int lines, int yNibble, int xNibble)
{
	PIXEL *line1, *line2;
	int step = gapi->dstPixelStep;

	if( gapi->userOrientation == SDL_ORIENTATION_UP )
	{
		if( yNibble ) // copy bottom half of a line
		{
			while(width--)
			{
				PIXEL c1 = *srcPointer++;
				c1 = ((c1 & REDMASK) >> 11) + ((c1 & GREENMASK) >> 5) + (c1 & BLUEMASK);			
				*destPointer = (*destPointer & 0x0F) | ((~(c1 >> 3) << 4));
				destPointer += step;
			}
			return 1;
		}

		// either 1 pixel picture or tail, anyway this is the last line
		if( lines == 1 )
		{
			while(width--)
			{
				PIXEL c1 = *srcPointer++;
				c1 = ((c1 & REDMASK) >> 11) + ((c1 & GREENMASK) >> 5) + (c1 & BLUEMASK);			
				*destPointer = (*destPointer & 0xF0) | ((~(c1 >> 3) & 0xF));
				destPointer += step;
			}
			return 1;
		}

		line1 = srcPointer;
		line2 = srcPointer + SDL_VideoSurface->pitch / 2;

		while(width--)
		{
			PIXEL c1 = *line1++;
			PIXEL c2 = *line2++;
			c1 = ((c1 & REDMASK) >> 11) + ((c1 & GREENMASK) >> 5) + (c1 & BLUEMASK);
			c2 = ((c2 & REDMASK) >> 11) + ((c2 & GREENMASK) >> 5) + (c2 & BLUEMASK);
			*destPointer = ~((c1 >> 3) + ((c2 >> 3) << 4));
			destPointer += step;
		}
		return 2;
	} else
	{
		int w1;
		w1 = width / 2;

		if( xNibble )
		{
			// copy one pixel
			PIXEL c1 = *srcPointer++;
			c1 = ((c1 & REDMASK) >> 11) + ((c1 & GREENMASK) >> 5) + (c1 & BLUEMASK);			
			*destPointer = (*destPointer & 0xF0) | ((~(c1 >> 3) & 0xF));
			destPointer++;
		}

		while(w1--)
		{
			PIXEL c1 = *srcPointer;
			PIXEL c2 = *(srcPointer + 1);
			c1 = ((c1 & REDMASK) >> 11) + ((c1 & GREENMASK) >> 5) + (c1 & BLUEMASK);
			c2 = ((c2 & REDMASK) >> 11) + ((c2 & GREENMASK) >> 5) + (c2 & BLUEMASK);
			*destPointer++ = ~((c2 >> 3) + ((c1 >> 3) << 4));
			srcPointer += 2;
		}

		// copy tail
		if( (width & 1) && !xNibble )
		{
			PIXEL c1 = *srcPointer;
			c1 = ((c1 & REDMASK) >> 11) + ((c1 & GREENMASK) >> 5) + (c1 & BLUEMASK);			
			*destPointer = (*destPointer & 0x0F) | ((~(c1 >> 3) << 4));
		}

		return 1;
	}
}

static void GAPI_UpdateRectsMono(_THIS, int numrects, SDL_Rect *rects)
{
	int i, height;
	int linesProcessed;
	int xNibble, yNibble;

	for (i=0; i<numrects; i++)
	{
		unsigned char *destPointer;
		unsigned char *srcPointer;

		if( gapi->userOrientation == SDL_ORIENTATION_UP )
			destPointer = (unsigned char*) gapi->videoMem + gapi->startOffset - rects[i].y * gapi->gxProperties.cBPP / 8 + rects[i].x * gapi->dstPixelStep;
		else
			destPointer = (unsigned char*) gapi->videoMem + gapi->startOffset + rects[i].x * gapi->gxProperties.cBPP / 8 + rects[i].y * gapi->dstLineStep;

		srcPointer = ((unsigned char*) SDL_VideoSurface->pixels) + rects[i].y * SDL_VideoSurface->pitch + rects[i].x * 2;
		yNibble = rects[i].y & 1; // TODO: only for 4 bpp
		xNibble = rects[i].x & 1;
		height = rects[i].h;
		while (height > 0)
		{
			switch(gapi->gxProperties.cBPP)
			{
			case 2: // TODO
			case 4:
					linesProcessed = updateLine16to4(this, (PIXEL*) srcPointer, destPointer, rects[i].w, rects[i].h, height, yNibble, xNibble);
					yNibble = 0;
			}
			height -= linesProcessed;
			if( gapi->userOrientation == SDL_ORIENTATION_UP )
				destPointer--; // always fill 1 byte
			else destPointer += gapi->dstLineStep;
			srcPointer += SDL_VideoSurface->pitch * linesProcessed; // pitch in bytes
		}
	}
}

static void GAPI_UpdateRectsColor(_THIS, int numrects, SDL_Rect *rects)
{
	int i, height;
	int bytesPerPixel = (gapi->gxProperties.cBPP + 1) / 8;
	int linesProcessed;
	for (i=0; i<numrects; i++) {
		unsigned char *destPointer = (unsigned char*) gapi->videoMem + gapi->startOffset + rects[i].y * gapi->dstLineStep + rects[i].x * gapi->dstPixelStep;
		unsigned char *srcPointer = ((unsigned char*) SDL_VideoSurface->pixels) + rects[i].y * SDL_VideoSurface->pitch + rects[i].x * bytesPerPixel;
		height = rects[i].h;

//		fprintf(stderr, "Starting rect %dx%d, dst=0x%x, w = %d, h = %d\n", rects[i].w, rects[i].h,destPointer,rects[i].w,rects[i].h);
//		fflush(stderr);
		linesProcessed = height;

		while (height > 0) {
			switch(bytesPerPixel)
			{
			case 1:
				linesProcessed = updateLine8to8(this, srcPointer, (unsigned char *) destPointer, rects[i].w, rects[i].h, height);
				break;
			case 2:
#pragma warning(disable: 4133)
				linesProcessed = updateLine16to16(this, (PIXEL*) srcPointer, destPointer, rects[i].w, rects[i].h, height);
				break;
			}
			height -= linesProcessed;
			destPointer += gapi->dstLineStep * linesProcessed;
			srcPointer += SDL_VideoSurface->pitch * linesProcessed; // pitch in bytes
		}
//		fprintf(stderr, "End of rect\n");
//		fflush(stderr);
	}
}


static void GAPI_UpdateRects(_THIS, int numrects, SDL_Rect *rects)
{
	// we do not want to corrupt video memory
	if( gapi->suspended ) return;

	if( gapi->needUpdate )
		gapi->videoMem = gapi->gxFunc.GXBeginDraw(); 

	if( gapi->gxProperties.cBPP < 8 )
		GAPI_UpdateRectsMono(this, numrects, rects);
	else
		GAPI_UpdateRectsColor(this, numrects, rects);

	if( gapi->needUpdate )
		gapi->gxFunc.GXEndDraw();
}

/* Note:  If we are terminated, this could be called in the middle of
   another SDL video routine -- notably UpdateRects.
*/
void GAPI_VideoQuit(_THIS)
{
	int i, j;
	/* Destroy the window and everything associated with it */
	if ( SDL_Window ) 
	{
	    if ((g_hGapiLib != 0) && this && gapi && gapi->gxFunc.GXCloseDisplay && !gapi->useVga)
			gapi->gxFunc.GXCloseDisplay(); 

		if (this->screen->pixels != NULL)
		{
			SDL_free(this->screen->pixels);
			this->screen->pixels = NULL;
		}
		if ( screen_icn ) {
			DestroyIcon(screen_icn);
			screen_icn = NULL;
		}

		DIB_DestroyWindow(this);
		SDL_UnregisterApp();

		SDL_Window = NULL;
#if defined(_WIN32_WCE)

// Unload wince aygshell library to prevent leak
		if( aygshell ) 
		{
			FreeLibrary(aygshell);
			aygshell = NULL;
		}
#endif

	/* Free video mode lists */
	for ( i=0; i<NUM_MODELISTS; ++i ) {
		if ( gapi->SDL_modelist[i] != NULL ) {
			for ( j=0; gapi->SDL_modelist[i][j]; ++j )
				SDL_free(gapi->SDL_modelist[i][j]);
			SDL_free(gapi->SDL_modelist[i]);
			gapi->SDL_modelist[i] = NULL;
		}
	}

	}

}

static void GAPI_Activate(_THIS, BOOL active, BOOL minimized)
{
	//Nothing to do here (as far as I know)
}

static void GAPI_RealizePalette(_THIS)
{
	OutputDebugString(TEXT("GAPI_RealizePalette NOT IMPLEMENTED !\r\n"));
}

static void GAPI_PaletteChanged(_THIS, HWND window)
{
	OutputDebugString(TEXT("GAPI_PaletteChanged NOT IMPLEMENTED !\r\n"));
}

static void GAPI_WinPAINT(_THIS, HDC hdc)
{
	// draw current offscreen buffer on hdc

	int bpp = 16; // we always use either 8 or 16 bpp internally
	HGDIOBJ prevObject;
	unsigned short *bitmapData;
	HBITMAP hb;
	HDC srcDC;

    // Create a DIB
    BYTE buffer[sizeof(BITMAPINFOHEADER) + 3 * sizeof(RGBQUAD)] = {0};
    BITMAPINFO*       pBMI    = (BITMAPINFO*)buffer;
    BITMAPINFOHEADER* pHeader = &pBMI->bmiHeader;
    DWORD*            pColors = (DWORD*)&pBMI->bmiColors;   

	// CreateDIBSection does not support 332 pixel format on wce
	if( gapi->gxProperties.cBPP == 8 ) return;

    // DIB Header
    pHeader->biSize            = sizeof(BITMAPINFOHEADER);
    pHeader->biWidth           = gapi->w;
    pHeader->biHeight          = -gapi->h;
    pHeader->biPlanes          = 1;
    pHeader->biBitCount        = bpp;
    pHeader->biCompression     = BI_RGB;
    pHeader->biSizeImage       = (gapi->w * gapi->h * bpp) / 8;
	
    // Color masks
	if( bpp == 16 )
	{
		pColors[0] = REDMASK;
		pColors[1] = GREENMASK;
		pColors[2] = BLUEMASK;
		pHeader->biCompression = BI_BITFIELDS;
	}
    // Create the DIB
    hb =  CreateDIBSection( 0, pBMI, DIB_RGB_COLORS, (void**)&bitmapData, 0, 0 );

	// copy data
	// FIXME: prevent misalignment, but I've never seen non aligned width of screen
	memcpy(bitmapData, gapi->buffer, pHeader->biSizeImage);
	srcDC = CreateCompatibleDC(hdc);
	prevObject = SelectObject(srcDC, hb);

	BitBlt(hdc, 0, 0, gapi->w, gapi->h, srcDC, 0, 0, SRCCOPY);

	SelectObject(srcDC, prevObject);
	DeleteObject(hb);
	DeleteDC(srcDC);
}

int GAPI_SetColors(_THIS, int firstcolor, int ncolors, SDL_Color *colors) 
{
	GAPI_CreatePalette(ncolors, colors);
	return 1;
}
