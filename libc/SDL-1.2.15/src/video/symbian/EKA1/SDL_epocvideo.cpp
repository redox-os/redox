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
    slouken@devolution.com
*/

/*
    SDL_epocvideo.cpp
    Epoc based SDL video driver implementation

    Thanks to Peter van Sebille, the author of EMame. It is a great example of 
    low level graphics coding in Epoc.

    Epoc version by Hannu Viitala (hannu.j.viitala@mbnet.fi)
	Assembler routines by Kimmo Kinnunen
*/



#include <stdlib.h>
#include <stdio.h>
#include <string.h>

extern "C" {
#include "SDL_error.h"
#include "SDL_timer.h"
#include "SDL_video.h"
#undef NULL
#include "SDL_pixels_c.h"
#include "SDL.h"
};

#include "SDL_epocvideo.h"
#include "SDL_epocevents_c.h"

#include "sdl_epocruntime.h"

#include <hal.h>
#include <coedef.h>
#include <flogger.h>

#ifdef SYMBIAN_QUARTZ
SDL_VideoDevice* _thisDevice;
#endif

_LIT(KLibName, "SDL");

/* For debugging */

//if old SOS, from 7.x this is public!
class CLockable : public CFbsBitmap
    {
    public:
        static  CLockable* Lockable(CFbsBitmap* aBmp) {return static_cast<CLockable*>(aBmp);}
        void Lock() {LockHeap();}
        void Unlock() {UnlockHeap();}
    };
#define LockHeap(x) CLockable::Lockable(x)->Lock()
#define UnlockHeap(x) CLockable::Lockable(x)->Unlock()

void RDebug_Print_b(char* error_str, void* param)
    {
    TBuf8<128> error8((TUint8*)error_str);
    TBuf<128> error;
    error.Copy(error8);

#ifndef TRACE_TO_FILE
    if (param) //!! Do not work if the parameter is really 0!!
        RDebug::Print(error, param);
    else 
        RDebug::Print(error);
#else
    if (param) //!! Do not work if the parameter is really 0!!
        RFileLogger::WriteFormat(KLibName, _L("SDL.txt"), EFileLoggingModeAppend, error, param);
    else 
        RFileLogger::Write(KLibName, _L("SDL.txt"), EFileLoggingModeAppend, error);
#endif

    }

extern "C" void RDebug_Print(char* error_str, void* param)
    {
    RDebug_Print_b(error_str, param);
    }


int Debug_AvailMem2()
    {
    //User::CompressAllHeaps();
    TMemoryInfoV1Buf membuf; 
    User::LeaveIfError(UserHal::MemoryInfo(membuf));
    TMemoryInfoV1 minfo = membuf();
	return(minfo.iFreeRamInBytes);
    }

extern "C" int Debug_AvailMem()
    {
    return(Debug_AvailMem2());
    }


extern "C" {

/* Initialization/Query functions */

static int EPOC_VideoInit(_THIS, SDL_PixelFormat *vformat);
static SDL_Rect **EPOC_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags);
static SDL_Surface *EPOC_SetVideoMode(_THIS, SDL_Surface *current, int width, int height, int bpp, Uint32 flags);
static int EPOC_SetColors(_THIS, int firstcolor, int ncolors,
			  SDL_Color *colors);
static void EPOC_VideoQuit(_THIS);

/* Hardware surface functions */

static int EPOC_AllocHWSurface(_THIS, SDL_Surface *surface);
static int EPOC_LockHWSurface(_THIS, SDL_Surface *surface);
static int EPOC_FlipHWSurface(_THIS, SDL_Surface *surface);
static void EPOC_UnlockHWSurface(_THIS, SDL_Surface *surface);
static void EPOC_FreeHWSurface(_THIS, SDL_Surface *surface);
static void EPOC_DirectUpdate(_THIS, int numrects, SDL_Rect *rects);

static int EPOC_Available(void);
static SDL_VideoDevice *EPOC_CreateDevice(int devindex);

void DrawBackground(_THIS);
void DirectDraw(_THIS, int numrects, SDL_Rect *rects, TUint16* screenBuffer);
void DirectDrawRotated(_THIS, int numrects, SDL_Rect *rects, TUint16* screenBuffer);

/* Mouse functions */

static WMcursor *EPOC_CreateWMCursor(_THIS, Uint8 *data, Uint8 *mask, int w, int h, int hot_x, int hot_y);
static void EPOC_FreeWMCursor(_THIS, WMcursor *cursor);
static int EPOC_ShowWMCursor(_THIS, WMcursor *cursor);



/* !!For 12 bit screen HW. Table for fast conversion from 8 bit to 12 bit */
// TUint16 is enough, but using TUint32 so we can use better instruction selection on ARMI
static TUint32 EPOC_HWPalette_256_to_Screen[256];

VideoBootStrap EPOC_bootstrap = {
	"epoc", "EPOC system",
    EPOC_Available, EPOC_CreateDevice
};

const TUint32 WindowClientHandle = 9210; //!! const

/* Epoc video driver bootstrap functions */

static int EPOC_Available(void)
{
    return 1; /* Always available */
}

static void EPOC_DeleteDevice(SDL_VideoDevice *device)
{
	free(device->hidden);
	free(device);
}

static SDL_VideoDevice *EPOC_CreateDevice(int /*devindex*/)
{
	SDL_VideoDevice *device;

	SDL_TRACE("SDL:EPOC_CreateDevice");

	/* Allocate all variables that we free on delete */
	device = (SDL_VideoDevice *)malloc(sizeof(SDL_VideoDevice));
	if ( device ) {
		memset(device, 0, (sizeof *device));
		device->hidden = (struct SDL_PrivateVideoData *)
				malloc((sizeof *device->hidden));
	}
	if ( (device == NULL) || (device->hidden == NULL) ) {
		SDL_OutOfMemory();
		if ( device ) {
			free(device);
		}
		return(0);
	}
	memset(device->hidden, 0, (sizeof *device->hidden));

	/* Set the function pointers */
	device->VideoInit = EPOC_VideoInit;
	device->ListModes = EPOC_ListModes;
	device->SetVideoMode = EPOC_SetVideoMode;
	device->SetColors = EPOC_SetColors;
	device->UpdateRects = NULL;
	device->VideoQuit = EPOC_VideoQuit;
	device->AllocHWSurface = EPOC_AllocHWSurface;
	device->CheckHWBlit = NULL;
	device->FillHWRect = NULL;
	device->SetHWColorKey = NULL;
	device->SetHWAlpha = NULL;
	device->LockHWSurface = EPOC_LockHWSurface;
	device->UnlockHWSurface = EPOC_UnlockHWSurface;
	device->FlipHWSurface = EPOC_FlipHWSurface;
	device->FreeHWSurface = EPOC_FreeHWSurface;
	device->SetIcon = NULL;
	device->SetCaption = NULL;
	device->GetWMInfo = NULL;
	device->FreeWMCursor = EPOC_FreeWMCursor;
	device->CreateWMCursor = EPOC_CreateWMCursor;
	device->ShowWMCursor = EPOC_ShowWMCursor;
	device->WarpWMCursor = NULL;
	device->InitOSKeymap = EPOC_InitOSKeymap;
	device->PumpEvents = EPOC_PumpEvents;
	device->free = EPOC_DeleteDevice;

	return device;
}


int GetBpp(TDisplayMode displaymode)
{
    /*TInt numColors = TDisplayModeUtils::NumDisplayModeColors(displaymode);
    TInt bitsPerPixel = 1;
    for (TInt32 i = 2; i < numColors; i <<= 1, bitsPerPixel++);
    return bitsPerPixel;*/
    return  TDisplayModeUtils::NumDisplayModeBitsPerPixel(displaymode);   
}


void DisableKeyBlocking(_THIS)
    {
    // Disable key blocking
    TRawEvent event;
    event.Set((TRawEvent::TType)/*EDisableKeyBlock*/51); // !!EDisableKeyBlock not found in epoc32\include!
    Private->EPOC_WsSession.SimulateRawEvent(event);
    }

void ConstructWindowL(_THIS)
{
	TInt	error;

	SDL_TRACE("SDL:ConstructWindowL");
	error = Private->EPOC_WsSession.Connect();
	User::LeaveIfError(error);
	Private->EPOC_WsScreen=new(ELeave) CWsScreenDevice(Private->EPOC_WsSession);
	User::LeaveIfError(Private->EPOC_WsScreen->Construct());
	User::LeaveIfError(Private->EPOC_WsScreen->CreateContext(Private->EPOC_WindowGc));

	Private->EPOC_WsWindowGroup=RWindowGroup(Private->EPOC_WsSession);
	User::LeaveIfError(Private->EPOC_WsWindowGroup.Construct(WindowClientHandle));
	Private->EPOC_WsWindowGroup.SetOrdinalPosition(0);

	// Set window group name (the same as process name)) !!Gives always "EPOC" in WINS
	RProcess thisProcess;
	TParse exeName;
	exeName.Set(thisProcess.FileName(), NULL, NULL);
    TBuf<32> winGroupName;
    winGroupName.Append(0);
    winGroupName.Append(0);
    winGroupName.Append(0);// uid
    winGroupName.Append(0);
    winGroupName.Append(exeName.Name()); // caption
    winGroupName.Append(0);
    winGroupName.Append(0); //doc name
	Private->EPOC_WsWindowGroup.SetName(winGroupName); 

	Private->EPOC_WsWindow=RWindow(Private->EPOC_WsSession);
  // Markus, it was:
  // User::LeaveIfError(Private->EPOC_WsWindow.Construct(Private->EPOC_WsWindowGroup,WindowClientHandle ));
  // but SOS 7.0s debug does not accept same window handle twice
	User::LeaveIfError(Private->EPOC_WsWindow.Construct(Private->EPOC_WsWindowGroup,WindowClientHandle - 1));
	Private->EPOC_WsWindow.SetBackgroundColor(KRgbWhite);
    Private->EPOC_WsWindow.Activate();
	Private->EPOC_WsWindow.SetSize(Private->EPOC_WsScreen->SizeInPixels()); 
	Private->EPOC_WsWindow.SetVisible(ETrue);

    Private->EPOC_WsWindowGroupID = Private->EPOC_WsWindowGroup.Identifier();
    Private->EPOC_IsWindowFocused = EFalse;

    DisableKeyBlocking(_this); //disable key blocking
}

int EPOC_VideoInit(_THIS, SDL_PixelFormat *vformat)
{
    // !!TODO:handle leave functions!

    int i;

	SDL_TRACE("SDL:EPOC_VideoInit");

	/* Initialize all variables that we clean on shutdown */   

	for ( i=0; i<SDL_NUMMODES; ++i ) {
		Private->SDL_modelist[i] = (SDL_Rect *)malloc(sizeof(SDL_Rect));
		Private->SDL_modelist[i]->x = Private->SDL_modelist[i]->y = 0;
	}

	/* Modes sorted largest to smallest */
	Private->SDL_modelist[0]->w = 800; Private->SDL_modelist[0]->h = 250;
	Private->SDL_modelist[1]->w = 640; Private->SDL_modelist[1]->h = 480;
	Private->SDL_modelist[2]->w = 480; Private->SDL_modelist[2]->h = 600;
	Private->SDL_modelist[3]->w = 640; Private->SDL_modelist[3]->h = 400;
	Private->SDL_modelist[4]->w = 352; Private->SDL_modelist[4]->h = 416;
	Private->SDL_modelist[5]->w = 416; Private->SDL_modelist[5]->h = 352;
	Private->SDL_modelist[6]->w = 416; Private->SDL_modelist[6]->h = 312;
	Private->SDL_modelist[7]->w = 352; Private->SDL_modelist[7]->h = 264;
	Private->SDL_modelist[8]->w = 800; Private->SDL_modelist[8]->h = 240; //for doom all these..
	Private->SDL_modelist[9]->w = 640; Private->SDL_modelist[9]->h = 240; 
	Private->SDL_modelist[10]->w = 480; Private->SDL_modelist[10]->h = 240; 
	Private->SDL_modelist[11]->w = 640; Private->SDL_modelist[11]->h = 240; 
	Private->SDL_modelist[12]->w = 352; Private->SDL_modelist[12]->h = 240; 
	Private->SDL_modelist[13]->w = 416; Private->SDL_modelist[13]->h = 240; 
	Private->SDL_modelist[14]->w = 416; Private->SDL_modelist[14]->h = 240; 
	Private->SDL_modelist[15]->w = 352; Private->SDL_modelist[15]->h = 240; 
    Private->SDL_modelist[16]->w = 640; Private->SDL_modelist[16]->h = 200; 
	Private->SDL_modelist[17]->w = 320; Private->SDL_modelist[17]->h = 240; //...for doom, currently engine renders no-higher windows :-(, propably should get fixed 
	Private->SDL_modelist[18]->w = 320; Private->SDL_modelist[18]->h = 200;
	Private->SDL_modelist[19]->w = 256; Private->SDL_modelist[19]->h = 192;
	Private->SDL_modelist[20]->w = 176; Private->SDL_modelist[20]->h = 208;
	Private->SDL_modelist[21]->w = 208; Private->SDL_modelist[21]->h = 176; // Rotated
	Private->SDL_modelist[22]->w = 160; Private->SDL_modelist[22]->h = 144; 

    Private->SDL_modelist[23]->w = 640; Private->SDL_modelist[2]->h = 200;  //s80 some new modes 
    Private->SDL_modelist[24]->w = 640; Private->SDL_modelist[2]->h = 320;  //s90 modes are added
    Private->SDL_modelist[25]->w = 640; Private->SDL_modelist[2]->h = 240;  //here
	Private->SDL_modelist[26]->w = 640; Private->SDL_modelist[4]->h = 200;  //now

	Private->SDL_modelist[27] = NULL;

    /* Construct Epoc window */

    ConstructWindowL(_this);

    /* Initialise Epoc frame buffer */

    TDisplayMode displayMode = Private->EPOC_WsScreen->DisplayMode();

#if !defined(__WINS__) && !defined(TEST_BM_DRAW)

    TScreenInfoV01 screenInfo;
	TPckg<TScreenInfoV01> sInfo(screenInfo);
	UserSvr::ScreenInfo(sInfo);

	Private->EPOC_ScreenSize		= screenInfo.iScreenSize; 
	Private->EPOC_DisplayMode		= displayMode;
    Private->EPOC_HasFrameBuffer	= screenInfo.iScreenAddressValid;
	Private->EPOC_FrameBuffer		= Private->EPOC_HasFrameBuffer ? (TUint8*) screenInfo.iScreenAddress : NULL;
	Private->EPOC_BytesPerPixel	    = ((GetBpp(displayMode)-1) / 8) + 1;
	
    Private->EPOC_BytesPerScanLine	= screenInfo.iScreenSize.iWidth * Private->EPOC_BytesPerPixel;
	Private->EPOC_BytesPerScreen	= Private->EPOC_BytesPerScanLine * Private->EPOC_ScreenSize.iHeight;

    SDL_TRACE1("Screen width %d", screenInfo.iScreenSize.iWidth);
    SDL_TRACE1("Screen height %d", screenInfo.iScreenSize.iHeight);
    SDL_TRACE1("Screen dmode %d", displayMode);
    SDL_TRACE1("Screen valid %d", screenInfo.iScreenAddressValid);

    SDL_TRACE1("bpp %d", Private->EPOC_BytesPerPixel);
    SDL_TRACE1("bpsl %d", Private->EPOC_BytesPerScanLine);
    SDL_TRACE1("bps %d", Private->EPOC_BytesPerScreen);


    /* It seems that in SA1100 machines for 8bpp displays there is a 512 palette table at the 
     * beginning of the frame buffer. E.g. Series 7 and Netbook.
     * In 12 bpp machines the table has 16 entries.
	 */
	if (Private->EPOC_HasFrameBuffer && GetBpp(displayMode) == 8)
        {
		Private->EPOC_FrameBuffer += 512;
        }
	else
        {
        Private->EPOC_FrameBuffer += 32;
        }
        /*if (Private->EPOC_HasFrameBuffer && GetBpp(displayMode) == 12)
		Private->EPOC_FrameBuffer += 16 * 2;
    if (Private->EPOC_HasFrameBuffer && GetBpp(displayMode) == 16)
		Private->EPOC_FrameBuffer += 16 * 2;
    */
#else /* defined __WINS__ */
    
    /* Create bitmap, device and context for screen drawing */
    Private->EPOC_ScreenSize        = Private->EPOC_WsScreen->SizeInPixels();

	Private->EPOC_Bitmap = new (ELeave) CWsBitmap(Private->EPOC_WsSession);
	Private->EPOC_Bitmap->Create(Private->EPOC_ScreenSize, displayMode);

	Private->EPOC_DisplayMode	    = displayMode;
    Private->EPOC_HasFrameBuffer    = ETrue;
    Private->EPOC_FrameBuffer       = NULL; /* Private->EPOC_Bitmap->DataAddress() can change any time */
	Private->EPOC_BytesPerPixel	    = ((GetBpp(displayMode)-1) / 8) + 1;
	Private->EPOC_BytesPerScanLine  = Private->EPOC_WsScreen->SizeInPixels().iWidth * Private->EPOC_BytesPerPixel;

#endif /* __WINS__ */

#ifndef SYMBIAN_CRYSTAL
	// Get draw device for updating the screen
	TScreenInfoV01 screenInfo2;
    
    Epoc_Runtime::GetScreenInfo(screenInfo2);

	TRAPD(status, Private->EPOC_DrawDevice = CFbsDrawDevice::NewScreenDeviceL(screenInfo2, displayMode));
	User::LeaveIfError(status);
#endif

    /* The "best" video format should be returned to caller. */

    vformat->BitsPerPixel       = /*!!GetBpp(displayMode) */ 8;
    vformat->BytesPerPixel      = /*!!Private->EPOC_BytesPerPixel*/ 1;

    /* Activate events for me */

	Private->EPOC_WsEventStatus = KRequestPending;
	Private->EPOC_WsSession.EventReady(&Private->EPOC_WsEventStatus);

    SDL_TRACE("SDL:WsEventStatus");
    User::WaitForRequest(Private->EPOC_WsEventStatus); //Markus: I added this and ...

	Private->EPOC_RedrawEventStatus = KRequestPending;
	Private->EPOC_WsSession.RedrawReady(&Private->EPOC_RedrawEventStatus);
    
    SDL_TRACE("SDL:RedrawEventStatus");
    User::WaitForRequest(Private->EPOC_RedrawEventStatus); //...this, if not catches a stray event is risen
                                                           //if there are active objects used, or confucing
                                                           //actions with User::WaitForAnyRequest
    Private->EPOC_WsWindow.PointerFilter(EPointerFilterDrag, 0); 

    Private->EPOC_ScreenOffset = TPoint(0, 0);

#if defined(__WINS__) || defined(TEST_BM_DRAW)
	LockHeap(Private->EPOC_Bitmap); // Lock bitmap heap
#endif

    SDL_TRACE("SDL:DrawBackground");
	DrawBackground(_this); // Clear screen

#if defined(__WINS__) || defined(TEST_BM_DRAW)
	UnlockHeap(Private->EPOC_Bitmap); // Unlock bitmap heap
#endif
    //!! TODO: error handling
    //if (ret != KErrNone)
    //    return(-1);
    //else
	    return(0);
}


SDL_Rect **EPOC_ListModes(_THIS, SDL_PixelFormat *format, Uint32 /*flags*/)
{
    if (format->BitsPerPixel == 12 || format->BitsPerPixel == 8)
        return Private->SDL_modelist;
    return NULL;
}

int EPOC_SetColors(_THIS, int firstcolor, int ncolors, SDL_Color *colors)
{
	if ((firstcolor+ncolors) > 256)
		return -1;
//    SDL_TRACE1("colors %d", (TDisplayModeUtils::NumDisplayModeColors(Private->EPOC_DisplayMode)));
    if(TDisplayModeUtils::NumDisplayModeColors(Private->EPOC_DisplayMode) == 4096)
        {
	// Set 12 bit palette
        for(int i = firstcolor; i < ncolors; i++)
            {
	        // 4k value: 0000 rrrr gggg bbbb
	        TUint32 color4K	 = (colors[i].r & 0x0000f0) << 4;
	        color4K			|= (colors[i].g & 0x0000f0);      
	        color4K			|= (colors[i].b & 0x0000f0) >> 4;
            EPOC_HWPalette_256_to_Screen[i] = color4K;
            }
        }
    else if(TDisplayModeUtils::NumDisplayModeColors(Private->EPOC_DisplayMode) == 65536)
        {
        for(int i = firstcolor; i < ncolors; i++)
            {
			// 64k-colour displays effectively support RGB values
			// with 5 bits allocated to red, 6 to green and 5 to blue
			// 64k value: rrrr rggg gggb bbbb
	        TUint32 color64K = (colors[i].r & 0x0000f8) << 8;
	        color64K		|= (colors[i].g & 0x0000fc) << 3;
	        color64K		|= (colors[i].b & 0x0000f8) >> 3;
            EPOC_HWPalette_256_to_Screen[i] = color64K;
            }
        }
    else if(TDisplayModeUtils::NumDisplayModeColors(Private->EPOC_DisplayMode) == 16777216)
        {
        for(int i = firstcolor; i < ncolors; i++)
            {
			// 16M-colour
            //0000 0000 rrrr rrrr gggg gggg bbbb bbbb
	        TUint32 color16M = colors[i].r << 16;
	        color16M		|= colors[i].g << 8;
	        color16M		|= colors[i].b;
            EPOC_HWPalette_256_to_Screen[i] = color16M;
            }
        }
    else
        {
        return -2;
        }
	return(0);
}


SDL_Surface *EPOC_SetVideoMode(_THIS, SDL_Surface *current,
				int width, int height, int bpp, Uint32 /*flags*/)
{
	SDL_TRACE("SDL:EPOC_SetVideoMode");
    /* Check parameters */
#ifdef SYMBIAN_CRYSTAL
   if (! (bpp == 8 || bpp == 12 || bpp == 16) &&
		 (
		  (width == 640 && height == 200) ||
          (width == 640 && height == 400) || 
          (width == 640 && height == 480) || 
          (width == 320 && height == 200) || 
          (width == 320 && height == 240)
		 )) {
		SDL_SetError("Requested video mode is not supported");
        return NULL;
    }
#else // SYMBIAN_SERIES60
   if (! (bpp == 8 || bpp == 12 || bpp == 16) &&
		 (
		  (width == 320 && height == 200) || 
          (width == 320 && height == 240) ||
		  (width == 256 && height == 192) ||  
		  (width == 176 && height == 208) || 
		  (width == 208 && height == 176) || // Rotated
		  (width == 160 && height == 144)    
		 )) {
		SDL_SetError("Requested video mode is not supported");
        return NULL;
    }
#endif

    if (current && current->pixels) {
        free(current->pixels);
        current->pixels = NULL;
    }
	if ( ! SDL_ReallocFormat(current, bpp, 0, 0, 0, 0) ) {
		return(NULL);
	}

    /* Set up the new mode framebuffer */
    if (bpp == 8) 
	    current->flags = (SDL_FULLSCREEN|SDL_SWSURFACE|SDL_PREALLOC|SDL_HWPALETTE); 
    else // 12 bpp, 16 bpp
	    current->flags = (SDL_FULLSCREEN|SDL_SWSURFACE|SDL_PREALLOC); 
	current->w = width;
	current->h = height;
    int numBytesPerPixel = ((bpp-1)>>3) + 1;   
	current->pitch = numBytesPerPixel * width; // Number of bytes in scanline 
	current->pixels = malloc(width * height * numBytesPerPixel);
	memset(current->pixels, 0, width * height * numBytesPerPixel);

	/* Set the blit function */
	_this->UpdateRects = EPOC_DirectUpdate;

    /*
     *  Logic for getting suitable screen dimensions, offset, scaling and orientation
     */

	int w = current->w;
	int h = current->h;
	
	// Rotate, if the screen does not fit horizontally and it is landscape screen
/*
	if ((width>Private->EPOC_ScreenSize.iWidth) &&  (width>height)) {
		Private->EPOC_ScreenOrientation = CFbsBitGc::EGraphicsOrientationRotated270;
		w = current->h; 
		h = current->w; 
	}
*/
	// Get nearest stepwise scale values for width and height. The smallest supported scaled screen is 1/2.
	TInt scaleValue = 0;
	Private->EPOC_ScreenXScaleValue = 1;
	Private->EPOC_ScreenYScaleValue = 1;
	if (w > Private->EPOC_ScreenSize.iWidth) {
		// Find the biggest scale value that result the width that fits in the screen HW
		for (scaleValue = 2; scaleValue++;) {
			TInt scaledWidth = (w * (scaleValue-1))/scaleValue;
			if (scaledWidth > Private->EPOC_ScreenSize.iWidth)
				break;
		}
		Private->EPOC_ScreenXScaleValue = Max(2, scaleValue - 1);
		w = (w * (Private->EPOC_ScreenXScaleValue-1))/Private->EPOC_ScreenXScaleValue;
	}
	if (h > Private->EPOC_ScreenSize.iHeight) {
		// Find the biggest scale value that result the height that fits in the screen HW
		for (scaleValue = 2; scaleValue++;) {
			TInt scaledHeight = (h * (scaleValue-1))/scaleValue;
			if (scaledHeight > Private->EPOC_ScreenSize.iHeight)
				break;
		}
		Private->EPOC_ScreenYScaleValue = Max(2, scaleValue - 1);
		h = (h * (Private->EPOC_ScreenYScaleValue-1))/Private->EPOC_ScreenYScaleValue;
	}

    /* Centralize game window on device screen  */
    Private->EPOC_ScreenOffset.iX = (Private->EPOC_ScreenSize.iWidth - w) / 2;
	if (Private->EPOC_ScreenOffset.iX < 0)
		Private->EPOC_ScreenOffset.iX = 0;
    Private->EPOC_ScreenOffset.iY = (Private->EPOC_ScreenSize.iHeight - h) / 2;
	if (Private->EPOC_ScreenOffset.iY < 0)
		Private->EPOC_ScreenOffset.iY = 0;


    SDL_TRACE1("View width %d", w);
    SDL_TRACE1("View height %d", h);
    SDL_TRACE1("View bmode %d", bpp);
    SDL_TRACE1("View s %d", scaleValue);
    SDL_TRACE1("View x %d", Private->EPOC_ScreenOffset.iX);
    SDL_TRACE1("View y %d", Private->EPOC_ScreenOffset.iY);

	/* We're done */
	return(current);
}


void RedrawWindowL(_THIS)
{

#if defined(__WINS__) || defined(TEST_BM_DRAW)
	    LockHeap(Private->EPOC_Bitmap); // Lock bitmap heap
	    Private->EPOC_WindowGc->Activate(Private->EPOC_WsWindow);
#endif

    int w = _this->screen->w;
    int h = _this->screen->h;
	if (Private->EPOC_ScreenOrientation == CFbsBitGc::EGraphicsOrientationRotated270) {
		w = _this->screen->h;
		h = _this->screen->w;
	}
    if ((w < Private->EPOC_ScreenSize.iWidth)
        || (h < Private->EPOC_ScreenSize.iHeight)) {
		DrawBackground(_this);
    }

    /* Tell the system that something has been drawn */
	TRect  rect = TRect(Private->EPOC_WsWindow.Size());
  	Private->EPOC_WsWindow.Invalidate(rect);

#if defined(__WINS__) || defined(TEST_BM_DRAW)
	Private->EPOC_WsWindow.BeginRedraw(rect);
	Private->EPOC_WindowGc->BitBlt(TPoint(), Private->EPOC_Bitmap);
	Private->EPOC_WsWindow.EndRedraw();
	Private->EPOC_WindowGc->Deactivate();
    UnlockHeap(Private->EPOC_Bitmap);; // Unlock bitmap heap
	Private->EPOC_WsSession.Flush();
#endif

    /* Draw current buffer */
    SDL_Rect fullScreen;
    fullScreen.x = 0;
    fullScreen.y = 0;
    fullScreen.w = _this->screen->w;
    fullScreen.h = _this->screen->h;
    EPOC_DirectUpdate(_this, 1, &fullScreen);
}


void DrawBackground(_THIS)
{
	/* Draw background */
#if defined(__WINS__) || defined(TEST_BM_DRAW)
	//warning heap is not locked! - a function calling must ensure that it's ok
    TUint16* screenBuffer = (TUint16*)Private->EPOC_Bitmap->DataAddress();
#else
    TUint16* screenBuffer = (TUint16*)Private->EPOC_FrameBuffer;
#endif
	// Draw black background
	Mem::FillZ(screenBuffer, Private->EPOC_BytesPerScreen);

#if 0
    for (int y = 0; y < Private->EPOC_ScreenSize.iHeight; y++) {
		for (int x = 0; x < Private->EPOC_ScreenSize.iWidth; x++) {
#ifdef SYMBIAN_CRYSTAL
			const TUint16 color = 0; // ((x+y)>>1) & 0xf; /* Draw blue stripes pattern, because in e.g. 320x200 mode there is a big background area*/
#else // SYMBIAN_SERIES60
            const TUint16 color = 0; /* Draw black background */
#endif
            *screenBuffer++ = color;
		}
	}
#endif
}


/* We don't actually allow hardware surfaces other than the main one */
static int EPOC_AllocHWSurface(_THIS, SDL_Surface* /*surface*/)
{
	return(-1);
}
static void EPOC_FreeHWSurface(_THIS, SDL_Surface* /*surface*/)
{
	return;
}

static int EPOC_LockHWSurface(_THIS, SDL_Surface* /*surface*/)
{
	return(0);
}
static void EPOC_UnlockHWSurface(_THIS, SDL_Surface* /*surface*/)
{
	return;
}

static int EPOC_FlipHWSurface(_THIS, SDL_Surface* /*surface*/)
{
	return(0);
}

static void EPOC_DirectUpdate(_THIS, int numrects, SDL_Rect *rects)
{
    //TInt focusWindowGroupId = Private->EPOC_WsSession.GetFocusWindowGroup();//these are async services
   // if (focusWindowGroupId != Private->EPOC_WsWindowGroupID) {              //for that cannot be called from
                                                                            //SDL threads ???
    if (!Private->EPOC_IsWindowFocused)
        {
        /* Force focus window to redraw again for cleaning away SDL screen graphics */
/*
        TInt pos = Private->EPOC_WsWindowGroup.OrdinalPosition();
        Private->EPOC_WsWindowGroup.SetOrdinalPosition(0, KMaxTInt);
       	TRect  rect = TRect(Private->EPOC_WsWindow.Size());
        Private->EPOC_WsWindow.Invalidate(rect);
        Private->EPOC_WsWindowGroup.SetOrdinalPosition(pos, ECoeWinPriorityNormal);
       */ /* If this is not the topmost window, wait here! Sleep for 1 second to give cpu time to 
           multitasking and poll for being the topmost window.
        */
    //    if (Private->EPOC_WsSession.GetFocusWindowGroup() != Private->EPOC_WsWindowGroupID) {
           
		   /* !!TODO: Could call GetRedraw() etc. for WsSession and redraw the screen if needed. That might be 
		      needed if a small dialog comes in front of Game screen.
		   */
           // while (Private->EPOC_WsSession.GetFocusWindowGroup() != Private->EPOC_WsWindowGroupID)
        
        SDL_PauseAudio(1);
        SDL_Delay(1000);
        return;
      //  }

    //    RedrawWindowL(_this);  
    }

    SDL_PauseAudio(0);

	// if we are not focused, do not draw
//	if (!Private->EPOC_IsWindowFocused)
//		return;
#if defined(__WINS__) || defined(TEST_BM_DRAW)
	TBitmapUtil lock(Private->EPOC_Bitmap);	
    lock.Begin(TPoint(0,0)); // Lock bitmap heap
	Private->EPOC_WindowGc->Activate(Private->EPOC_WsWindow);
    TUint16* screenBuffer = (TUint16*)Private->EPOC_Bitmap->DataAddress();
#else
    TUint16* screenBuffer = (TUint16*)Private->EPOC_FrameBuffer;
#endif

	if (Private->EPOC_ScreenOrientation == CFbsBitGc::EGraphicsOrientationRotated270)
		DirectDrawRotated(_this, numrects, rects, screenBuffer);
	else
		DirectDraw(_this, numrects, rects, screenBuffer);
    
   
#if defined(__WINS__) || defined(TEST_BM_DRAW)

	TRect  rect = TRect(Private->EPOC_WsWindow.Size());
	Private->EPOC_WsWindow.Invalidate(rect);
	Private->EPOC_WsWindow.BeginRedraw(rect);
	Private->EPOC_WindowGc->BitBlt(TPoint(), Private->EPOC_Bitmap);
	Private->EPOC_WsWindow.EndRedraw();
	Private->EPOC_WindowGc->Deactivate();
    lock.End(); // Unlock bitmap heap
	Private->EPOC_WsSession.Flush();
#else
#ifndef SYMBIAN_CRYSTAL
	// This is not needed in Crystal. What is the performance penalty in SERIES60?
    TRect  rect2 = TRect(Private->EPOC_WsWindow.Size());
    
    Private->EPOC_DrawDevice->UpdateRegion(rect2); // Should we update rects parameter area only??
    Private->EPOC_DrawDevice->Update();
#endif
#endif

    /* Update virtual cursor. !!Do not yet work properly
    Private->EPOC_WsSession.SetPointerCursorPosition(Private->EPOC_WsSession.PointerCursorPosition());
    */

    /*static int foo = 1;

    	for ( int i=0; i < numrects; ++i ) {
        const SDL_Rect& currentRect = rects[i];
        SDL_Rect rect2;
        rect2.x = currentRect.x;
        rect2.y = currentRect.y;
        rect2.w = currentRect.w;
        rect2.h = currentRect.h;

        if (rect2.w <= 0 || rect2.h <= 0) 
            continue;

    
    foo++;
    if((foo % 200) == 0)
        {
        SDL_TRACE1("foo %d", foo);
        CFbsBitmap* b = new (ELeave) CFbsBitmap;
        SDL_TRACE1("bee %d", (int)b);
        int e = b->Create(TSize(currentRect.w, currentRect.h), Private->EPOC_DisplayMode);
        
        SDL_TRACE1("err %d", e);
        if(e != KErrNone)
            User::Panic(_L("damn"), e);

        TBitmapUtil u(b);
        u.Begin(TPoint(0, 0));
        TUint32* d = b->DataAddress();
        
        SDL_TRACE1("addr %d", (int)d);

        for(TInt o = 0; o < currentRect.h; o++)
            for(TInt p = 0; p < currentRect.w; p++)
                {
                u.SetPos(TPoint(p, o));
                u.SetPixel(0xFFFF);
                }

        SDL_TRACE1("w %d", (int)currentRect.w);
        SDL_TRACE1("h %d", (int)currentRect.h);

        SDL_TRACE1("addr %d", (int)Private->EPOC_DisplayMode);

        
        const TUint f = (TUint)Private->EPOC_FrameBuffer;
        const TUint y = (TUint)Private->EPOC_BytesPerScreen;

        
        SDL_TRACE1("frame %u", f);
        SDL_TRACE1("bytes %u", y);

        Mem::Copy(d, Private->EPOC_FrameBuffer, Private->EPOC_BytesPerScreen);

        SDL_TRACE("kopied");

        u.End();
        TBuf<32> name;
        name.Format(_L("C:\\nokia\\images\\doom%d.mbm"), (foo / 200));
        e= b->Save(name);
        if(e != KErrNone)
            User::Panic(_L("damned"), e);
        delete b;
            }}*/
}


void DirectDraw(_THIS, int numrects, SDL_Rect *rects, TUint16* screenBuffer)
{
	TInt i;

    const TInt sourceNumBytesPerPixel = ((_this->screen->format->BitsPerPixel-1)>>3) + 1;   
    const TPoint fixedOffset = Private->EPOC_ScreenOffset;   
    const TInt screenW = _this->screen->w;
    const TInt screenH = _this->screen->h;
    const TInt sourceScanlineLength = screenW;
    const TInt targetScanlineLength = Private->EPOC_ScreenSize.iWidth;

	/* Render the rectangles in the list */

	for ( i=0; i < numrects; ++i ) {
        const SDL_Rect& currentRect = rects[i];
        SDL_Rect rect2;
        rect2.x = currentRect.x;
        rect2.y = currentRect.y;
        rect2.w = currentRect.w;
        rect2.h = currentRect.h;

        if (rect2.w <= 0 || rect2.h <= 0) /* sanity check */
            continue;

        /* All variables are measured in pixels */

        /* Check rects validity, i.e. upper and lower bounds */
        TInt maxX = Min(screenW - 1, rect2.x + rect2.w - 1);
        TInt maxY = Min(screenH - 1, rect2.y + rect2.h - 1);
        if (maxX < 0 || maxY < 0) /* sanity check */
            continue;
		/* Clip from bottom */
        maxY = Min(maxY, Private->EPOC_ScreenSize.iHeight-1); 
		/* TODO: Clip from the right side */

        const TInt sourceRectWidth = maxX - rect2.x + 1;
        const TInt sourceRectWidthInBytes = sourceRectWidth * sourceNumBytesPerPixel;
        const TInt sourceRectHeight = maxY - rect2.y + 1;
        const TInt sourceStartOffset = rect2.x + rect2.y * sourceScanlineLength;
		const TUint skipValue = 1; // no skip

        TInt targetStartOffset = fixedOffset.iX + rect2.x + (fixedOffset.iY +rect2.y) * targetScanlineLength;   
        
        // Nokia7650 native mode: 12 bpp --> 12 bpp
        //

        switch (_this->screen->format->BitsPerPixel)
			{
		case 12:
			{
			TUint16* bitmapLine = (TUint16*)_this->screen->pixels + sourceStartOffset;
			TUint16* screenMemory = screenBuffer + targetStartOffset;
			if (skipValue == 1)
				{
				for(TInt y = 0 ; y < sourceRectHeight ; y++)
					{
					Mem::Copy(screenMemory, bitmapLine, sourceRectWidthInBytes);
					}
					bitmapLine += sourceScanlineLength;
					screenMemory += targetScanlineLength;
				}
			else
				{
				for(TInt y = 0 ; y < sourceRectHeight ; y++)
					{
					//TODO: optimize: separate loops for 1, 2 and n skip. Mem::Copy() can be used in unscaled case.
					TUint16* bitmapPos = bitmapLine; /* 2 bytes per pixel */
					TUint16* screenMemoryLinePos = screenMemory; /* 2 bytes per pixel */
					for(TInt x = 0 ; x < sourceRectWidth ; x++)
						{					
						__ASSERT_DEBUG(screenMemory < (screenBuffer + Private->EPOC_ScreenSize.iWidth * Private->EPOC_ScreenSize.iHeight), User::Panic(_L("SDL"), KErrCorrupt));
						__ASSERT_DEBUG(screenMemory >= screenBuffer, User::Panic(_L("SDL"), KErrCorrupt));
						__ASSERT_DEBUG(bitmapLine < ((TUint16*)_this->screen->pixels + (_this->screen->w * _this->screen->h)), User::Panic(_L("SDL"), KErrCorrupt));
						__ASSERT_DEBUG(bitmapLine >=  (TUint16*)_this->screen->pixels, User::Panic(_L("SDL"), KErrCorrupt));
                    
						*screenMemoryLinePos++ = *bitmapPos;
						bitmapPos+=skipValue;
						}
					bitmapLine += sourceScanlineLength;
					screenMemory += targetScanlineLength;
					}
				}
			}
			break;
        // 256 color paletted mode: 8 bpp  --> 12 bpp
        //
		default:
			{
            if(Private->EPOC_BytesPerPixel <= 2)
                {
			    TUint8* bitmapLine = (TUint8*)_this->screen->pixels + sourceStartOffset;
                TUint16* screenMemory = screenBuffer + targetStartOffset;
				    for(TInt y = 0 ; y < sourceRectHeight ; y++)
					    {
					    TUint8* bitmapPos = bitmapLine; /* 1 byte per pixel */
					    TUint16* screenMemoryLinePos = screenMemory; /* 2 bytes per pixel */
					    /* Convert each pixel from 256 palette to 4k color values */
					    for(TInt x = 0 ; x < sourceRectWidth ; x++)
						    {
						    __ASSERT_DEBUG(screenMemoryLinePos < (screenBuffer + (Private->EPOC_ScreenSize.iWidth * Private->EPOC_ScreenSize.iHeight)), User::Panic(_L("SDL"), KErrCorrupt));
						    __ASSERT_DEBUG(screenMemoryLinePos >= screenBuffer, User::Panic(_L("SDL"), KErrCorrupt));
						    __ASSERT_DEBUG(bitmapPos < ((TUint8*)_this->screen->pixels + (_this->screen->w * _this->screen->h)), User::Panic(_L("SDL"), KErrCorrupt));
						    __ASSERT_DEBUG(bitmapPos >= (TUint8*)_this->screen->pixels, User::Panic(_L("SDL"), KErrCorrupt));               
						    *screenMemoryLinePos++ = EPOC_HWPalette_256_to_Screen[*bitmapPos++];
    //						bitmapPos+=skipValue; //TODO: optimize: separate loops for 1, 2 and n skip
						    }
					    bitmapLine += sourceScanlineLength;
					    screenMemory += targetScanlineLength;
					    }
                }
            else
                {
                TUint8* bitmapLine = (TUint8*)_this->screen->pixels + sourceStartOffset;
                TUint32* screenMemory = reinterpret_cast<TUint32*>(screenBuffer + targetStartOffset);
				    for(TInt y = 0 ; y < sourceRectHeight ; y++)
					    {
					    TUint8* bitmapPos = bitmapLine; /* 1 byte per pixel */
					    TUint32* screenMemoryLinePos = screenMemory; /* 2 bytes per pixel */
					    /* Convert each pixel from 256 palette to 4k color values */
					    for(TInt x = 0 ; x < sourceRectWidth ; x++)
						    {
						    __ASSERT_DEBUG(screenMemoryLinePos < (reinterpret_cast<TUint32*>(screenBuffer) + (Private->EPOC_ScreenSize.iWidth * Private->EPOC_ScreenSize.iHeight)), User::Panic(_L("SDL"), KErrCorrupt));
						    __ASSERT_DEBUG(screenMemoryLinePos >= reinterpret_cast<TUint32*>(screenBuffer), User::Panic(_L("SDL"), KErrCorrupt));
						    __ASSERT_DEBUG(bitmapPos < ((TUint8*)_this->screen->pixels + (_this->screen->w * _this->screen->h)), User::Panic(_L("SDL"), KErrCorrupt));
						    __ASSERT_DEBUG(bitmapPos >= (TUint8*)_this->screen->pixels, User::Panic(_L("SDL"), KErrCorrupt));               
						    *screenMemoryLinePos++ = EPOC_HWPalette_256_to_Screen[*bitmapPos++];
    //						bitmapPos+=skipValue; //TODO: optimize: separate loops for 1, 2 and n skip
						    }
					    bitmapLine += sourceScanlineLength;
					    screenMemory += targetScanlineLength;
					    }
                }
			}
		} // switch
	} // for
}

/*
void DirectDraw(_THIS, int numrects, SDL_Rect *rects, TUint16* screenBuffer)
{
	TInt i;
    const TInt sourceNumBytesPerPixel = ((_this->screen->format->BitsPerPixel-1)>>3) + 1;   
    const TPoint fixedOffset = Private->EPOC_ScreenOffset;   
    const TInt screenW = _this->screen->w;
    const TInt screenH = _this->screen->h;
    const TInt sourceScanlineLength = screenW;
    const TInt targetScanlineLength = Private->EPOC_ScreenSize.iWidth;

	/* Render the rectangles in the list */

/*	for ( i=0; i < numrects; ++i ) {
        const SDL_Rect& currentRect = rects[i];
        SDL_Rect rect2;
        rect2.x = currentRect.x;
        rect2.y = currentRect.y;
        rect2.w = currentRect.w;
        rect2.h = currentRect.h;

        if (rect2.w <= 0 || rect2.h <= 0) /* sanity check */
/*            continue;

        /* All variables are measured in pixels */

        /* Check rects validity, i.e. upper and lower bounds */
/*        TInt maxX = Min(screenW - 1, rect2.x + rect2.w - 1);
        TInt maxY = Min(screenH - 1, rect2.y + rect2.h - 1);
        if (maxX < 0 || maxY < 0) /* sanity check */
/*            continue;
		/* Clip from bottom */
/*        maxY = Min(maxY, Private->EPOC_ScreenSize.iHeight-1); 
		/* TODO: Clip from the right side */

/*		TInt sourceRectWidth = maxX - rect2.x + 1;
        const TInt sourceRectWidthInBytes = sourceRectWidth * sourceNumBytesPerPixel;
        const TInt sourceRectHeight = maxY - rect2.y + 1;
        const TInt sourceStartOffset = rect2.x + rect2.y * sourceScanlineLength;
		const TUint skipValue = Private->EPOC_ScreenXScaleValue; //1; // no skip

		const TInt targetStartOffset = // = (fixedOffset.iX + (rect2.x / skipValue) + (fixedOffset.iY + rect2.y) * targetScanlineLength ) ;
			(skipValue > 1 ? 
			(fixedOffset.iX + (rect2.x / skipValue) + (fixedOffset.iY + rect2.y) * targetScanlineLength ) : 
			(fixedOffset.iX +  rect2.x              + (fixedOffset.iY + rect2.y) * targetScanlineLength ));

		__ASSERT_DEBUG(skipValue >= 1, User::Panic(KLibName, KErrArgument));

        // Nokia7650 native mode: 12 bpp --> 12 bpp
        //
        switch (_this->screen->format->BitsPerPixel)
			{
		case 12:
			{
			TUint16* bitmapLine = (TUint16*)_this->screen->pixels + sourceStartOffset;
			TUint16* screenMemory = screenBuffer + targetStartOffset;
			if (skipValue == 1)
				{
				for(TInt y = 0 ; y < sourceRectHeight ; y++)
					{
					Mem::Copy(screenMemory, bitmapLine, sourceRectWidthInBytes);
					}
					bitmapLine += sourceScanlineLength;
					screenMemory += targetScanlineLength;
				}
			else
				{
				for(TInt y = 0 ; y < sourceRectHeight ; y++)
					{
					//TODO: optimize: separate loops for 1, 2 and n skip. Mem::Copy() can be used in unscaled case.
					TUint16* bitmapPos = bitmapLine; /* 2 bytes per pixel */
/*					TUint16* screenMemoryLinePos = screenMemory; /* 2 bytes per pixel */
/*					for(TInt x = 0 ; x < sourceRectWidth ; x++)
						{					
						__ASSERT_DEBUG(screenMemory < (screenBuffer + Private->EPOC_ScreenSize.iWidth * Private->EPOC_ScreenSize.iHeight), User::Panic(KLibName, KErrCorrupt));
						__ASSERT_DEBUG(screenMemory >= screenBuffer, User::Panic(KLibName, KErrCorrupt));
						__ASSERT_DEBUG(bitmapLine < ((TUint16*)_this->screen->pixels + (_this->screen->w * _this->screen->h)), User::Panic(KLibName, KErrCorrupt));
						__ASSERT_DEBUG(bitmapLine >=  (TUint16*)_this->screen->pixels, User::Panic(KLibName, KErrCorrupt));
                    
						*screenMemoryLinePos++ = *bitmapPos;
						bitmapPos+=skipValue;
						}
					bitmapLine += sourceScanlineLength;
					screenMemory += targetScanlineLength;
					}
				}
			}
			break;
        // 256 color paletted mode: 8 bpp  --> 12 bpp
        //
		default:
			{
			TUint8* bitmapLine = (TUint8*)_this->screen->pixels + sourceStartOffset;
            TUint16* screenMemory = screenBuffer + targetStartOffset;
			if (skipValue > 1)
				sourceRectWidth /= skipValue;
#if defined __MARM_ARMI__
			__asm volatile("
				mov		%4, %4, lsl #1	@ targetScanLineLength is in pixels, we need it in bytes
			1:
				mov		r6, %0			@ bitmapLine
				mov		r7, %2			@ screenMemory
				mov		r8, %6			@ sourceRectWidth
			2:
				ldrb	r4, [%0], %7			@ r4 = *bitmapPos; bitmapPos += skipValue
				ldr		r5, [%1, r4, lsl #2]	@ only 16 lower bits actually used
				subs	r8, r8, #1				@ x--
				strh	r5, [%2], #2			@ *screenMemoryLinePos++ = r4
				bne		2b

				add		%0, r6, %3		@ bitmapLine += sourceScanlineLength
				add		%2, r7, %4		@ screenMemory += targetScanlineLength
				subs    %5, %5, #1		@ sourceRectHeight--
				bne		1b
				"
				: // no output
				//		%0								%1							%2						%3							%4						%5							%6					%7
				: "r" (bitmapLine), "r" (&EPOC_HWPalette_256_to_Screen[0]), "r" (screenMemory), "r" (sourceScanlineLength), "r" (targetScanlineLength), "r" (sourceRectHeight), "r" (sourceRectWidth), "r" (skipValue)
				: "r4", "r5", "r6", "r7", "r8"
			);
#else
			for(TInt y = 0 ; y < sourceRectHeight ; y++)
				{
				TUint8* bitmapPos = bitmapLine; /* 1 byte per pixel */
/*				TUint16* screenMemoryLinePos = screenMemory; /* 2 bytes per pixel */
				/* Convert each pixel from 256 palette to 4k color values */
/*				for (TInt x = 0 ; x < sourceRectWidth ; x++)
					{
					//__ASSERT_DEBUG(screenMemoryLinePos < (screenBuffer + (Private->EPOC_ScreenSize.iWidth * Private->EPOC_ScreenSize.iHeight)), User::Panic(KLibName, KErrCorrupt));
					//__ASSERT_DEBUG(screenMemoryLinePos >= screenBuffer, User::Panic(KLibName, KErrCorrupt));
					//__ASSERT_DEBUG(bitmapPos < ((TUint8*)_this->screen->pixels + (_this->screen->w * _this->screen->h)), User::Panic(KLibName, KErrCorrupt));
					//__ASSERT_DEBUG(bitmapPos >= (TUint8*)_this->screen->pixels, User::Panic(KLibName, KErrCorrupt));
            
					*screenMemoryLinePos++ = EPOC_HWPalette_256_to_Screen[*bitmapPos];
					bitmapPos += skipValue;
					}
				bitmapLine += sourceScanlineLength;
				screenMemory += targetScanlineLength;
				}
//#endif
			}
		} // switch
	} // for
}
*/

void DirectDrawRotated(_THIS, int numrects, SDL_Rect *rects, TUint16* screenBuffer)
{
	TInt i;
//    TInt sourceNumBytesPerPixel = ((_this->screen->format->BitsPerPixel-1)>>3) + 1;   
    TPoint fixedScreenOffset = Private->EPOC_ScreenOffset;   
    TInt bufferW = _this->screen->w;
    TInt bufferH = _this->screen->h;
    TInt ScreenW = Private->EPOC_ScreenSize.iWidth;
//    TInt ScreenH = Private->EPOC_ScreenSize.iWidth;
    TInt sourceW = bufferW;
    TInt sourceH = bufferH;
    TInt targetW = ScreenW - fixedScreenOffset.iX * 2;
//    TInt targetH = ScreenH - fixedScreenOffset.iY * 2;
	TInt sourceScanlineLength = bufferW;
    TInt targetScanlineLength = Private->EPOC_ScreenSize.iWidth;

	/* Render the rectangles in the list */

	for ( i=0; i < numrects; ++i ) {
        SDL_Rect rect2;
        const SDL_Rect& currentRect = rects[i];
        rect2.x = currentRect.x;
        rect2.y = currentRect.y;
        rect2.w = currentRect.w;
        rect2.h = currentRect.h;

        if (rect2.w <= 0 || rect2.h <= 0) /* sanity check */
            continue;

        /* All variables are measured in pixels */

        /* Check rects validity, i.e. upper and lower bounds */
        TInt maxX = Min(sourceW - 1, rect2.x + rect2.w - 1);
        TInt maxY = Min(sourceH - 1, rect2.y + rect2.h - 1);
        if (maxX < 0 || maxY < 0) /* sanity check */
            continue;
		/* Clip from bottom */
        //maxX = Min(maxX, Private->EPOC_ScreenSize.iHeight-1); 
		/* TODO: Clip from the right side */

        TInt sourceRectWidth = maxX - rect2.x + 1;
//        TInt sourceRectWidthInBytes = sourceRectWidth * sourceNumBytesPerPixel;
        TInt sourceRectHeight = maxY - rect2.y + 1;
        TInt sourceStartOffset = rect2.x + rect2.y * sourceScanlineLength;
        TInt targetStartOffset = fixedScreenOffset.iX + (targetW-1 - rect2.y) + (fixedScreenOffset.iY +rect2.x) * targetScanlineLength;   
        
        // Nokia7650 native mode: 12 bpp --> 12 bpp
        if (_this->screen->format->BitsPerPixel == 12) { 
            
            /* !!TODO: not yet implemented

	        TUint16* bitmapLine = (TUint16*)_this->screen->pixels + sourceStartOffset;
            TUint16* screenMemory = screenBuffer + targetStartOffset;
            for(TInt y = 0 ; y < sourceRectHeight ; y++) {
				//TODO: optimize: separate loops for 1, 2 and n skip
		        //Mem::Copy(screenMemory, bitmapLine, sourceRectWidthInBytes);
                TUint16* bitmapPos = bitmapLine; // 2 bytes per pixel 
                TUint16* screenMemoryLinePos = screenMemory; // 2 bytes per pixel 
				for(TInt x = 0 ; x < sourceRectWidth ; x++) {

					__ASSERT_DEBUG(screenMemory < (screenBuffer + Private->EPOC_ScreenSize.iWidth * Private->EPOC_ScreenSize.iHeight), User::Panic(KLibName, KErrCorrupt));
					__ASSERT_DEBUG(screenMemory >= screenBuffer, User::Panic(KLibName, KErrCorrupt));
					__ASSERT_DEBUG(bitmapLine < ((TUint16*)_this->screen->pixels + (_this->screen->w * _this->screen->h)), User::Panic(KLibName, KErrCorrupt));
					__ASSERT_DEBUG(bitmapLine >=  (TUint16*)_this->screen->pixels, User::Panic(KLibName, KErrCorrupt));
                    
                      *screenMemoryLinePos = *bitmapPos;
                    bitmapPos++;
                    screenMemoryLinePos += targetScanlineLength;
                }
		        bitmapLine += sourceScanlineLength;
		        screenMemory--;
            }

            */
        }
        // 256 color paletted mode: 8 bpp  --> 12 bpp
        else { 
	        TUint8* bitmapLine = (TUint8*)_this->screen->pixels + sourceStartOffset;
            TUint16* screenMemory = screenBuffer + targetStartOffset;
			TInt screenXScaleValue = Private->EPOC_ScreenXScaleValue;
			TInt debug_ycount=0;
            for(TInt y = 0 ; y < sourceRectHeight ; y++) {
				if(--screenXScaleValue) {
					TUint8* bitmapPos = bitmapLine; /* 1 byte per pixel */
					TUint16* screenMemoryLinePos = screenMemory; /* 2 bytes per pixel */
					TInt screenYScaleValue = Private->EPOC_ScreenYScaleValue;
					TInt debug_xcount=0;
					/* Convert each pixel from 256 palette to 4k color values */
					for(TInt x = 0 ; x < sourceRectWidth ; x++) {
						if(--screenYScaleValue) {
							
                            __ASSERT_DEBUG(screenMemoryLinePos < (screenBuffer + (Private->EPOC_ScreenSize.iWidth * Private->EPOC_ScreenSize.iHeight)), User::Panic(KLibName, KErrCorrupt));
							__ASSERT_DEBUG(screenMemoryLinePos >= screenBuffer, User::Panic(KLibName, KErrCorrupt));
							__ASSERT_DEBUG(bitmapPos < ((TUint8*)_this->screen->pixels + (_this->screen->w * _this->screen->h)), User::Panic(KLibName, KErrCorrupt));
							__ASSERT_DEBUG(bitmapPos >= (TUint8*)_this->screen->pixels, User::Panic(KLibName, KErrCorrupt));
							
                            *screenMemoryLinePos = TUint16(EPOC_HWPalette_256_to_Screen[*bitmapPos]);
							screenMemoryLinePos += targetScanlineLength; debug_xcount++;
						}
						else
							screenYScaleValue = Private->EPOC_ScreenYScaleValue;
						bitmapPos++; 
					}
					screenMemory--; debug_ycount++;
				} // endif
				else
					screenXScaleValue = Private->EPOC_ScreenXScaleValue;
				bitmapLine += sourceScanlineLength;
             }
	    }
    }    
}


/* Note:  If we are terminated, this could be called in the middle of
   another SDL video routine -- notably UpdateRects.
*/
void EPOC_VideoQuit(_THIS)
{
	int i;

	/* Free video mode lists */
	for ( i=0; i<SDL_NUMMODES; ++i ) {
		if ( Private->SDL_modelist[i] != NULL ) {
			free(Private->SDL_modelist[i]);
			Private->SDL_modelist[i] = NULL;
		}
	}
	
    if ( _this->screen && (_this->screen->flags & SDL_HWSURFACE) ) {
		/* Direct screen access, no memory buffer */
		_this->screen->pixels = NULL;
	}

    if (_this->screen && _this->screen->pixels) {
        free(_this->screen->pixels);
        _this->screen->pixels = NULL;
    }

    /* Free Epoc resources */

    /* Disable events for me */
	if (Private->EPOC_WsEventStatus != KRequestPending)
		Private->EPOC_WsSession.EventReadyCancel();
	if (Private->EPOC_RedrawEventStatus != KRequestPending)
		Private->EPOC_WsSession.RedrawReadyCancel();

	#if defined(__WINS__) || defined(TEST_BM_DRAW)
	delete Private->EPOC_Bitmap;
	Private->EPOC_Bitmap = NULL;
	#else
    #endif

#ifndef SYMBIAN_CRYSTAL
	free(Private->EPOC_DrawDevice);
#endif

	if (Private->EPOC_WsWindow.WsHandle())
		Private->EPOC_WsWindow.Close();

	if (Private->EPOC_WsWindowGroup.WsHandle())
		Private->EPOC_WsWindowGroup.Close();

	delete Private->EPOC_WindowGc;
	Private->EPOC_WindowGc = NULL;

	delete Private->EPOC_WsScreen;
	Private->EPOC_WsScreen = NULL;

	if (Private->EPOC_WsSession.WsHandle())
		Private->EPOC_WsSession.Close();
}


WMcursor *EPOC_CreateWMCursor(_THIS, Uint8* /*data*/, Uint8* /*mask*/, int /*w*/, int /*h*/, int /*hot_x*/, int /*hot_y*/)
{
	return (WMcursor *) 9210; // it's ok to return something unuseful but true
}

void EPOC_FreeWMCursor(_THIS, WMcursor* /*cursor*/)
{
    /* Disable virtual cursor */
    HAL::Set(HAL::EMouseState, HAL::EMouseState_Invisible);
    Private->EPOC_WsSession.SetPointerCursorMode(EPointerCursorNone);
}

int EPOC_ShowWMCursor(_THIS, WMcursor *cursor)
{

    if (cursor ==  (WMcursor *)9210) {
        /* Enable virtual cursor */
	    Private->EPOC_WsSession.SetPointerCursorMode(EPointerCursorNormal);
        if (isCursorVisible)
	        HAL::Set(HAL::EMouseState, HAL::EMouseState_Visible);
        else
            Private->EPOC_WsSession.SetPointerCursorMode(EPointerCursorNone);
    }
    else {
        /* Disable virtual cursor */
        HAL::Set(HAL::EMouseState, HAL::EMouseState_Invisible);
        Private->EPOC_WsSession.SetPointerCursorMode(EPointerCursorNone);
    }

	return(1);
}

}; // extern "C"
