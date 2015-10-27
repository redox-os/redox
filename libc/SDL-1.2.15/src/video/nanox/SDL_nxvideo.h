/*
    SDL - Simple DirectMedia Layer
    Copyright (C) 1997-2012 Sam Lantinga
    Copyright (C) 2001  Hsieh-Fu Tsai

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
    
    Hsieh-Fu Tsai
    clare@setabox.com
*/
#include "SDL_config.h"

#ifndef _SDL_nxvideo_h
#define _SDL_nxvideo_h

#include <microwin/nano-X.h>

#include "../SDL_sysvideo.h"

#ifdef ENABLE_NANOX_DEBUG
#define Dprintf printf
#else
#define Dprintf(ignore...)
#endif

// Hidden "this" pointer for the video functions
#define _THIS   SDL_VideoDevice * this

// Private display data
typedef struct NX_SDL_VISUAL {
    int    bpp ;
    Uint32 red_mask ;
    Uint32 green_mask ;
    Uint32 blue_mask ;
} nx_sdl_visual_t ;

struct SDL_PrivateVideoData {
    GR_WINDOW_ID    SDL_Window ;
    GR_WINDOW_ID    FSwindow ;
    // Flag: true if we have been passed a window
    char            * SDL_windowid ;
    GR_GC_ID        GC ;
    unsigned char   * Image ;
    unsigned char   * Image_buff ;	/* for GrArea*/
    unsigned char   * Clientfb;		/* for DirectFB*/
    nx_sdl_visual_t SDL_Visual ;
    // The current list of available video modes
    SDL_Rect        ** modelist ;
    int             currently_fullscreen ;
    // for fullscreen
    int             OffsetX, OffsetY ;
    // for GammaRamp
    Uint16          * GammaRamp_R, * GammaRamp_G, * GammaRamp_B ;
    // for GrArea, r_mask, g_mask, b_mask
    int             pixel_type ;
#ifdef ENABLE_NANOX_DIRECT_FB
    GR_WINDOW_FB_INFO fbinfo;
#endif
} ;

#define SDL_Window           (this -> hidden -> SDL_Window)
#define FSwindow             (this -> hidden -> FSwindow)
#define SDL_windowid         (this -> hidden -> SDL_windowid)
#define SDL_GC               (this -> hidden -> GC)
#define SDL_Image            (this -> hidden -> Image)
#define Image_buff           (this -> hidden -> Image_buff)
#define Clientfb             (this -> hidden -> Clientfb)
#define SDL_Visual           (this -> hidden -> SDL_Visual)
#define SDL_modelist         (this -> hidden -> modelist)
#define currently_fullscreen (this -> hidden -> currently_fullscreen)
#define OffsetX              (this -> hidden -> OffsetX)
#define OffsetY              (this -> hidden -> OffsetY)
#define GammaRamp_R          (this -> hidden -> GammaRamp_R)
#define GammaRamp_G          (this -> hidden -> GammaRamp_G)
#define GammaRamp_B          (this -> hidden -> GammaRamp_B)
#define pixel_type           (this -> hidden -> pixel_type)
#define fbinfo               (this -> hidden -> fbinfo)

#define CI_SIZE 256   // color index size

#endif  // _SDL_nxvideo_h
