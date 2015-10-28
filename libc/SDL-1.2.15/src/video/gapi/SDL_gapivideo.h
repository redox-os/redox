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

#ifndef _SDL_gapivideo_h
#define _SDL_gapivideo_h

#include "SDL_mouse.h"
#include "SDL_mutex.h"
#include "../SDL_sysvideo.h"
#include "../windib/SDL_gapidibvideo.h"

/* From gx.h, since it's not really C compliant */

struct GXDisplayProperties {
    DWORD cxWidth;
    DWORD cyHeight;         // notice lack of 'th' in the word height.
    long cbxPitch;          // number of bytes to move right one x pixel - can be negative.
    long cbyPitch;          // number of bytes to move down one y pixel - can be negative.
    long cBPP;              // # of bits in each pixel
    DWORD ffFormat;         // format flags.
};

struct GXKeyList {
    short vkUp;             // key for up
    POINT ptUp;             // x,y position of key/button.  Not on screen but in screen coordinates.
    short vkDown;
    POINT ptDown;
    short vkLeft;
    POINT ptLeft;
    short vkRight;
    POINT ptRight;
    short vkA;
    POINT ptA;
    short vkB;
    POINT ptB;
    short vkC;
    POINT ptC;
    short vkStart;
    POINT ptStart;
};

typedef int   (*PFNGXOpenDisplay)(HWND hWnd, DWORD dwFlags);
typedef int   (*PFNGXCloseDisplay)();
typedef void* (*PFNGXBeginDraw)();
typedef int   (*PFNGXEndDraw)();
typedef int   (*PFNGXOpenInput)();
typedef int   (*PFNGXCloseInput)();
typedef struct GXDisplayProperties (*PFNGXGetDisplayProperties)();
typedef struct GXKeyList (*PFNGXGetDefaultKeys)(int iOptions);
typedef int   (*PFNGXSuspend)();
typedef int   (*PFNGXResume)();
typedef int   (*PFNGXSetViewport)( DWORD dwTop, DWORD dwHeight, DWORD dwReserved1, DWORD dwReserved2 );
typedef BOOL  (*PFNGXIsDisplayDRAMBuffer)();

struct GapiFunc
{
    PFNGXOpenDisplay          GXOpenDisplay;
    PFNGXCloseDisplay         GXCloseDisplay;
    PFNGXBeginDraw            GXBeginDraw;
    PFNGXEndDraw              GXEndDraw;
    PFNGXOpenInput            GXOpenInput;
    PFNGXCloseInput           GXCloseInput;
    PFNGXGetDisplayProperties GXGetDisplayProperties;
    PFNGXGetDefaultKeys       GXGetDefaultKeys;
    PFNGXSuspend              GXSuspend;
    PFNGXResume               GXResume;
    PFNGXSetViewport          GXSetViewport;
    PFNGXIsDisplayDRAMBuffer  GXIsDisplayDRAMBuffer;
};

#define kfLandscape	0x8			// Screen is rotated 270 degrees
#define kfPalette	0x10		// Pixel values are indexes into a palette
#define kfDirect	0x20		// Pixel values contain actual level information
#define kfDirect555	0x40		// 5 bits each for red, green and blue values in a pixel.
#define kfDirect565	0x80		// 5 red bits, 6 green bits and 5 blue bits per pixel
#define kfDirect888	0x100		// 8 bits each for red, green and blue values in a pixel.
#define kfDirect444	0x200		// 4 red, 4 green, 4 blue
#define kfDirectInverted 0x400

#define GX_FULLSCREEN	0x01		// for OpenDisplay() 
#define GX_NORMALKEYS   0x02
#define GX_LANDSCAPEKEYS        0x03


/* GAPI video mode */
typedef enum {
	GAPI_NONE = 0,
	GAPI_DIRECT_565,
	GAPI_DIRECT_555,
	GAPI_MONO,
	GAPI_PALETTE
} GAPIVideoMode; 

typedef unsigned short PIXEL;

/* Private display data 
   begin with DIB private structure to allow DIB events code sharing
*/
struct GapiInfo {
	/* Rotation which has to be applied to the key (arrow keys) and mouse events measured in quarters of a circle
	 * counter clockwise */
	int coordinateTransform; 
	char hiresFix; /* using hires mode without defining hires resource */
	int invert; //TODO this is only written but never read, so it should be removed

#define NUM_MODELISTS	4		/* 8, 16, 24, and 32 bits-per-pixel */
	int SDL_nummodes[NUM_MODELISTS];
	SDL_Rect **SDL_modelist[NUM_MODELISTS];
	

	// The orientation of the video mode user wants to get
	// Probably restricted to UP and RIGHT
	SDL_ScreenOrientation userOrientation;
	SDL_ScreenOrientation systemOrientation;
// --------------
	int useGXOpenDisplay; /* use GXOpenDispplay */
	int alreadyGXOpened;
	int w, h;
	// The orientation of GAPI framebuffer.
	// Never changes on the same device.
	SDL_ScreenOrientation gapiOrientation;

	void *buffer; // may be 8, 16, 24, 32 bpp
	PIXEL *videoMem;
	BOOL needUpdate;
	struct GXKeyList keyList;
	struct GapiFunc gxFunc;
	struct GXDisplayProperties gxProperties;
	GAPIVideoMode videoMode;
	int colorscale;
	int dstLineStep;  // in bytes
	int dstPixelStep; // in bytes
	int startOffset; // in bytes
	int useVga;
	int suspended; // do not pu anything into video memory
};



#endif /* _SDL_gapivideo_h */
