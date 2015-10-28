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

    Markus Mertama
*/



#include "epoc_sdl.h"

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
#include "SDL_mouse.h"
}

#include "SDL_epocvideo.h"
#include "SDL_epocevents_c.h"



#include <coedef.h>
#include <flogger.h>

#include <eikenv.h>
#include <eikappui.h>
#include <eikapp.h>
#include "sdlepocapi.h"


////////////////////////////////////////////////////////////////




_LIT(KLibName, "SDL");

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

/*
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
    
*/

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
}


extern "C"
	{
	struct WMcursor
		{
		};
	}

/* Epoc video driver bootstrap functions */


static int EPOC_Available(void)
    {
    return 1; /* Always available */
    }

static void EPOC_DeleteDevice(SDL_VideoDevice *device)
    {
	User::Free(device->hidden);
	User::Free(device);
    }

static SDL_VideoDevice *EPOC_CreateDevice(int /*devindex*/)
    {
	SDL_VideoDevice *device;

	SDL_TRACE("SDL:EPOC_CreateDevice");

	/* Allocate all variables that we free on delete */
	device = static_cast<SDL_VideoDevice*>(User::Alloc(sizeof(SDL_VideoDevice)));
	if ( device ) 
	    {
		Mem::FillZ(device, (sizeof *device));
		device->hidden = static_cast<struct SDL_PrivateVideoData*>
				(User::Alloc((sizeof *device->hidden)));
	    }
	if ( (device == NULL) || (device->hidden == NULL) )
	    {
		SDL_OutOfMemory();
		if ( device ) {
		User::Free(device);
		}
		return(0);
	}
	Mem::FillZ(device->hidden, (sizeof *device->hidden));

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


VideoBootStrap EPOC_bootstrap = {
	"epoc\0\0\0", "EPOC system",
    EPOC_Available, EPOC_CreateDevice
};



void DisableKeyBlocking(_THIS)
    {
    EpocSdlEnv::Request(EpocSdlEnv::EDisableKeyBlocking);
    }

void ConstructWindowL(_THIS)
	{
	SDL_TRACE("SDL:ConstructWindowL");
	DisableKeyBlocking(_this); //disable key blocking
	}
		

int EPOC_VideoInit(_THIS, SDL_PixelFormat *vformat)
	{
    /* Construct Epoc window */

    ConstructWindowL(_this);

    /* Initialise Epoc frame buffer */

  
    const TDisplayMode displayMode = EpocSdlEnv::DisplayMode();

    /* The "best" video format should be returned to caller. */

    vformat->BitsPerPixel 	= TDisplayModeUtils::NumDisplayModeBitsPerPixel(displayMode);
    vformat->BytesPerPixel  = TDisplayModeUtils::NumDisplayModeBitsPerPixel(displayMode) / 8;

    
 //??   Private->iWindow->PointerFilter(EPointerFilterDrag, 0); 

    Private->iScreenPos = TPoint(0, 0);
    
    Private->iRect.x = Private->iScreenPos.iX;
    Private->iRect.y = Private->iScreenPos.iY;
    
    const TSize sz = EpocSdlEnv::WindowSize();
    
    Private->iRect.w = sz.iWidth;
    Private->iRect.h = sz.iHeight;
	Private->iRectPtr = &Private->iRect;

	return(0);
	}


SDL_Rect **EPOC_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags)
	{
	if(flags & SDL_HWSURFACE)
		{
		if(format->BytesPerPixel != 4) //in HW only full color is supported
			return NULL;
		}
	if(flags & SDL_FULLSCREEN)
		{
		return &Private->iRectPtr;
		}
    return (SDL_Rect **)(-1); //everythingisok, unless too small shoes
	}


int EPOC_SetColors(_THIS, int firstcolor, int ncolors, SDL_Color *colors)
	{
	if ((firstcolor+ncolors) > 256)
		return -1;
	TUint32 palette[256];
	const TDisplayMode mode = EpocSdlEnv::DisplayMode();
    if(TDisplayModeUtils::NumDisplayModeColors(mode) == 4096)
        {
	// Set 12 bit palette
        for(int i = firstcolor; i < ncolors; i++)
            {
	        // 4k value: 0000 rrrr gggg bbbb
	        TUint32 color4K	 = (colors[i].r & 0x0000f0) << 4;
	        color4K			|= (colors[i].g & 0x0000f0);      
	        color4K			|= (colors[i].b & 0x0000f0) >> 4;
            palette[i] = color4K;
            }
        }
    else if(TDisplayModeUtils::NumDisplayModeColors(mode) == 65536)
        {
        for(int i = firstcolor; i < ncolors; i++)
            {
			// 64k-colour displays effectively support RGB values
			// with 5 bits allocated to red, 6 to green and 5 to blue
			// 64k value: rrrr rggg gggb bbbb
	        TUint32 color64K = (colors[i].r & 0x0000f8) << 8;
	        color64K		|= (colors[i].g & 0x0000fc) << 3;
	        color64K		|= (colors[i].b & 0x0000f8) >> 3;
            palette[i] = color64K;
            }
        }
    else if(TDisplayModeUtils::NumDisplayModeColors(mode) == 16777216)
        {
        for(int i = firstcolor; i < ncolors; i++)
            {
			// 16M-colour
            //0000 0000 rrrr rrrr gggg gggg bbbb bbbb
	        TUint32 color16M = colors[i].r << 16;
	        color16M		|= colors[i].g << 8;
	        color16M		|= colors[i].b;
            palette[i] = color16M;
            }
        }
    else
        {
        return -2;
        }
    if(EpocSdlEnv::SetPalette(firstcolor, ncolors, palette) == KErrNone)
    	return 0;
	return -1;
	}


/*	
void AllocHWSurfaceL(CFbsBitmap*& aBitmap, const TDisplayMode& aMode, const TSize& aSize)
	{
	aBitmap = new (ELeave) CFbsBitmap();	
	if(KErrNone != aBitmap->CreateHardwareBitmap(aSize, aMode,
		EpocSdlEnv::EikonEnv().EikAppUi()->Application()->AppDllUid())) 
	//...if it fails - should we use wsbitmaps???
		{//the good reason to use hw bitmaps is that they wont need lock heap
		PANIC_IF_ERROR(aBitmap->Create(aSize, aMode));
		}
	}
	
int CreateSurfaceL(_THIS, SDL_Surface* surface)
    {
    __ASSERT_ALWAYS(Private->iFrame == NULL, PANIC(KErrAlreadyExists));
;
	TInt dmode = EColorLast;
	
	TDisplayMode displayMode;
	EpocSdlEnv::GetDiplayMode(displayMode);
	
	if(
	TDisplayModeUtils::NumDisplayModeBitsPerPixel(displayMode)
	== surface->format->BitsPerPixel)
		{
		dmode = displayMode;
		}
	else
		{
		--dmode;
		while(TDisplayModeUtils::IsDisplayModeColor(TDisplayMode(dmode)) &&
			TDisplayModeUtils::NumDisplayModeBitsPerPixel(TDisplayMode(dmode)) !=
			surface->format->BitsPerPixel)
			--dmode;
		}

	__ASSERT_ALWAYS(TDisplayModeUtils::IsDisplayModeColor(TDisplayMode(dmode)), PANIC(KErrNotSupported));
	TRAPD(err, AllocHWSurfaceL(Private->iFrame, TDisplayMode(dmode), TSize(surface->w, surface->h)));
	return err == KErrNone ? 0 : -1;
    }
*/

TDisplayMode GetDisplayMode(TInt aBitsPerPixel)
	{
	const TDisplayMode displayMode = EpocSdlEnv::DisplayMode();
	TInt dmode = EColorLast;
	if(
	TDisplayModeUtils::NumDisplayModeBitsPerPixel(displayMode)
	== aBitsPerPixel)
		{
		dmode = displayMode;
		}
	else
		{
		--dmode;
		while(TDisplayModeUtils::IsDisplayModeColor(TDisplayMode(dmode)) &&
			TDisplayModeUtils::NumDisplayModeBitsPerPixel(TDisplayMode(dmode)) !=
			aBitsPerPixel)
			--dmode;
		}
	return TDisplayMode(dmode);
	}

SDL_Surface *EPOC_SetVideoMode(_THIS, SDL_Surface *current,
				int width, int height, int bpp, Uint32 flags)
	{
	const TSize screenSize = EpocSdlEnv::WindowSize(TSize(width, height));
	if(width > screenSize.iWidth || height > screenSize.iHeight)
	    {
	    if(flags & SDL_FULLSCREEN)
	        {
	        width = screenSize.iWidth;
	        height = screenSize.iHeight;
	        }
	    else    
		    return NULL;
	    }

    if(current && current->pixels)
    	{
      //  free(current->pixels);
        current->pixels = NULL;
    	}
    	
	if(!SDL_ReallocFormat(current, bpp, 0, 0, 0, 0))
	 	{
		return(NULL);
	 	}

	current->flags = 0;
	if(width == screenSize.iWidth && height == screenSize.iHeight)
		current->flags |= SDL_FULLSCREEN;
	
	const int numBytesPerPixel = ((bpp-1)>>3) + 1;   
	current->pitch = numBytesPerPixel * width; // Number of bytes in scanline 

    /* Set up the new mode framebuffer */
   	current->flags |= SDL_PREALLOC;
   	
   	if(bpp <= 8)
   		current->flags |= SDL_HWPALETTE;
   	
   	User::Free(Private->iSwSurface);
   	current->pixels = NULL;
   	Private->iSwSurface = NULL;
   	
   	if(flags & SDL_HWSURFACE)
   	    {
   	    current->flags |= SDL_HWSURFACE;
   	   //	current->pixels = NULL;
   	   // 	Private->iSwSurface = NULL;
   	    }
   	else
   	    {
   	    current->flags |= SDL_SWSURFACE;
   	    const TInt surfacesize = width * height * numBytesPerPixel;  
   	   	Private->iSwSurfaceSize = TSize(width, height);
   	   	delete Private->iSwSurface;
   	   	Private->iSwSurface = NULL;
   	  	current->pixels = (TUint8*) User::AllocL(surfacesize);
   	  	Private->iSwSurface = (TUint8*) current->pixels;
   	  	const TInt err = EpocSdlEnv::AllocSwSurface
   	  		(TSize(width, height), GetDisplayMode(current->format->BitsPerPixel));
	    if(err != KErrNone)
	    	return NULL;
	    }
	
	current->w = width;
	current->h = height;
	
  
	
	/* Set the blit function */
	_this->UpdateRects = EPOC_DirectUpdate;

    /*
     *  Logic for getting suitable screen dimensions, offset, scaling and orientation
     */


    /* Centralize game window on device screen  */
   
    
    Private->iScreenPos.iX = Max(0, (screenSize.iWidth  - width)  / 2);
    Private->iScreenPos.iY = Max(0, (screenSize.iHeight - height) / 2);
    
 //   delete (Private->iFrame);
//	Private->iFrame = NULL;
	
  //  TRAPD(err, CreateSurfaceL(_this, current));
  //  PANIC_IF_ERROR(err);
    
    SDL_TRACE1("View width %d", width);
    SDL_TRACE1("View height %d", height);
    SDL_TRACE1("View bmode %d", bpp);
    SDL_TRACE1("View x %d", Private->iScreenPos.iX);
    SDL_TRACE1("View y %d", Private->iScreenPos.iY);

	EpocSdlEnv::LockPalette(EFalse);
	/* We're done */
	return(current);
}



static int EPOC_AllocHWSurface(_THIS, SDL_Surface* surface)
	{
	return KErrNone == EpocSdlEnv::AllocHwSurface(TSize(surface->w, surface->h), GetDisplayMode(surface->format->BitsPerPixel));
	}
	
static void EPOC_FreeHWSurface(_THIS, SDL_Surface* /*surface*/)
	{
	}

static int EPOC_LockHWSurface(_THIS, SDL_Surface* surface)
	{
	if(EpocSdlEnv::IsDsaAvailable())
		{
		TUint8* address = EpocSdlEnv::LockHwSurface();
		if(address != NULL)
			{
			surface->pixels = address;
			return 1;
			}
		}
	return 0;
	}
static void EPOC_UnlockHWSurface(_THIS, SDL_Surface* /*surface*/)
	{
	EpocSdlEnv::UnlockHwSurface();
	}

static int EPOC_FlipHWSurface(_THIS, SDL_Surface* /*surface*/)
	{
	return(0);
	}

static void EPOC_DirectUpdate(_THIS, int numrects, SDL_Rect *rects)
	{
	if(EpocSdlEnv::IsDsaAvailable())
		{
		if(Private->iSwSurface)
		    {
		    const TRect target(Private->iScreenPos, Private->iSwSurfaceSize);
		    for(TInt i = 0; i < numrects ;i++)
		    	{
		    	const TRect rect(TPoint(rects[i].x, rects[i].y),
		    		TSize(rects[i].w, rects[i].h));
		    	if(!EpocSdlEnv::AddUpdateRect(Private->iSwSurface, rect, target))
		    		return; //not succesful
		    	}
		    EpocSdlEnv::UpdateSwSurface();
		    }
		SDL_PauseAudio(0);
		}
    else
    	{                                                      
     	SDL_PauseAudio(1);
    	EpocSdlEnv::WaitDsaAvailable();              
		}
	}


/* Note:  If we are terminated, this could be called in the middle of
   another SDL video routine -- notably UpdateRects.
*/
void EPOC_VideoQuit(_THIS)
	{
//	delete Private->iFrame;
//	Private->iFrame = NULL;
	User::Free(Private->iSwSurface);
	Private->iSwSurface = NULL;
	EpocSdlEnv::FreeSurface();
	}
	
	


WMcursor *EPOC_CreateWMCursor(_THIS, Uint8* /*data*/, Uint8* /*mask*/, int /*w*/, int /*h*/, int /*hot_x*/, int /*hot_y*/)
    {
    return (WMcursor*) 1; //hii! prevents SDL to view a std cursor
    }

void EPOC_FreeWMCursor(_THIS, WMcursor* /*cursor*/)
    {
    }

int EPOC_ShowWMCursor(_THIS, WMcursor *cursor)
    {
    return true;  
    }

