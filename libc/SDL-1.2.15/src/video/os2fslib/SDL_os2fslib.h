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

#ifndef _SDL_os2fslib_h
#define _SDL_os2fslib_h


// OS2 specific includes
#define INCL_TYPES
#define INCL_DOS
#define INCL_DOSERRORS
#define INCL_DOSPROCESS
#define INCL_WIN
#define INCL_GPI
#include <os2.h>

#include <FSLib.h>

/* Hidden "this" pointer for the video functions */
#define _THIS	SDL_VideoDevice *_this

/* Private display data */
struct SDL_PrivateVideoData
{
  FSLib_VideoMode_p   pAvailableFSLibVideoModes;
  SDL_Rect          **pListModesResult; // Allocated memory to return list of modes for os2fslib_ListModes() API

  FSLib_VideoMode     SrcBufferDesc;    // Description of current source image buffer
  char               *pchSrcBuffer;     // The source image buffer itself
  SDL_Surface        *pSDLSurface;      // The SDL surface describing the buffer
  HMTX                hmtxUseSrcBuffer; // Mutex semaphore to manipulate src buffer
  HWND                hwndFrame, hwndClient;  // Window handle of frame and client
  int                 iPMThreadStatus;  // 0: Not running
                                        // 1: Running
                                        // Other: Not running, had an error
  int                 tidPMThread;      // Thread ID of PM Thread
  int                 fInFocus;         // True if we're in focus!
  int                 iSkipWMMOUSEMOVE; // Number of WM_MOUSEMOVE messages to skip!
  int                 iMouseVisible;    //

  PFNWP               pfnOldFrameProc;  // Old window frame procedure
  int                 bProportionalResize; // 0: No proportional resizing
                                           // 1: Do proportional resizing
  ULONG               ulResizingFlag;   // First resizing flag value
};

/* OS/2 specific backdoor function to be able to set FrameControlFlags of */
/* the SDL window before creating it. */
extern DECLSPEC void SDLCALL SDL_OS2FSLIB_SetFCFToUse(ULONG ulFCF);

#endif /* _SDL_os2fslib_h */
