/*
    SDL - Simple DirectMedia Layer
    Copyright (C) 1997-2012 Sam Lantinga
    Copyright (C) 2001  Hsieh-Fu Tsai
    Copyright (C) 2002  Greg Haerr <greg@censoft.com>

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

#include "SDL_thread.h"
#include "SDL_video.h"
#include "../SDL_pixels_c.h"
#include "../../events/SDL_events_c.h"

#define MWINCLUDECOLORS
#include "SDL_nxvideo.h"
#include "SDL_nxmodes_c.h"
#include "SDL_nxwm_c.h"
#include "SDL_nxmouse_c.h"
#include "SDL_nximage_c.h"
#include "SDL_nxevents_c.h"

// Initialization/Query functions
static int NX_VideoInit (_THIS, SDL_PixelFormat * vformat) ;
static SDL_Surface * NX_SetVideoMode (_THIS, SDL_Surface * current, int width, int height, int bpp, Uint32 flags) ;
static int NX_SetColors (_THIS, int firstcolor, int ncolors, SDL_Color * colors) ;
static void NX_VideoQuit (_THIS) ;
static void NX_DestroyWindow (_THIS, SDL_Surface * screen) ;
static int NX_ToggleFullScreen (_THIS, int on) ;
static void NX_UpdateMouse (_THIS) ;
static int NX_SetGammaRamp (_THIS, Uint16 * ramp) ;
static int NX_GetGammaRamp (_THIS, Uint16 * ramp) ;

// Microwin driver bootstrap functions
static int NX_Available ()
{
    Dprintf ("enter NX_Available\n") ;

    if (GrOpen () < 0) return 0 ;
        GrClose () ;
    
    Dprintf ("leave NX_Available\n") ;
    return 1 ;
}

static void NX_DeleteDevice (SDL_VideoDevice * device)
{
    Dprintf ("enter NX_DeleteDevice\n") ;

    if (device) {
        if (device -> hidden) SDL_free (device -> hidden) ;
        if (device -> gl_data) SDL_free (device -> gl_data) ;
            SDL_free (device) ;
    }

    Dprintf ("leave NX_DeleteDevice\n") ;
}
    
static SDL_VideoDevice * NX_CreateDevice (int devindex)
{
    SDL_VideoDevice * device ;

    Dprintf ("enter NX_CreateDevice\n") ;

    // Initialize all variables that we clean on shutdown
    device = (SDL_VideoDevice *) SDL_malloc (sizeof (SDL_VideoDevice)) ;
    if (device) {
        SDL_memset (device, 0, (sizeof * device)) ;
        device -> hidden = (struct SDL_PrivateVideoData *)
                SDL_malloc ((sizeof * device -> hidden)) ;
        device -> gl_data = NULL ;
    }
    if ((device == NULL) || (device -> hidden == NULL)) {
        SDL_OutOfMemory () ;
        NX_DeleteDevice (device) ;
        return 0 ;
    }
    SDL_memset (device -> hidden, 0, (sizeof * device -> hidden)) ;

    // Set the function pointers
    device -> VideoInit = NX_VideoInit ;
    device -> ListModes = NX_ListModes ;
    device -> SetVideoMode = NX_SetVideoMode ;
    device -> ToggleFullScreen = NX_ToggleFullScreen ;
    device -> UpdateMouse = NX_UpdateMouse ;
    device -> CreateYUVOverlay = NULL ;
    device -> SetColors = NX_SetColors ;
    device -> UpdateRects = NULL ;
    device -> VideoQuit = NX_VideoQuit;
    device -> AllocHWSurface = NULL ;
    device -> CheckHWBlit = NULL ;
    device -> FillHWRect = NULL ;
    device -> SetHWColorKey = NULL ;
    device -> SetHWAlpha = NULL ;
    device -> LockHWSurface = NULL ;
    device -> UnlockHWSurface = NULL ;
    device -> FlipHWSurface = NULL ;
    device -> FreeHWSurface = NULL ;
    device -> SetGamma = NULL ;
    device -> GetGamma = NULL ;
    device -> SetGammaRamp = NX_SetGammaRamp ;
    device -> GetGammaRamp = NX_GetGammaRamp ;

#if SDL_VIDEO_OPENGL
    device -> GL_LoadLibrary = NULL ;
    device -> GL_GetProcAddress = NULL ;
    device -> GL_GetAttribute = NULL ;
    device -> GL_MakeCurrent = NULL ;
    device -> GL_SwapBuffers = NULL ;
#endif

    device -> SetIcon = NULL ;
    device -> SetCaption = NX_SetCaption;
    device -> IconifyWindow = NULL ;
    device -> GrabInput = NULL ;
    device -> GetWMInfo = NX_GetWMInfo ;
    device -> FreeWMCursor =  NX_FreeWMCursor ;
    device -> CreateWMCursor = NX_CreateWMCursor ;
    device -> ShowWMCursor = NX_ShowWMCursor ;
    device -> WarpWMCursor = NX_WarpWMCursor ;
    device -> CheckMouseMode = NULL ;
    device -> InitOSKeymap = NX_InitOSKeymap ;
    device -> PumpEvents = NX_PumpEvents ;

    device -> free = NX_DeleteDevice ;

    Dprintf ("leave NX_CreateDevice\n") ;
    return device ;
}

VideoBootStrap NX_bootstrap = {
    "nanox", "nanox", NX_Available, NX_CreateDevice
} ;

static void create_aux_windows (_THIS)
{
    GR_WM_PROPERTIES props ;

    Dprintf ("enter create_aux_windows\n") ;

    // Don't create any extra windows if we are being managed
    if (SDL_windowid) {
        FSwindow = 0 ;
        return ;
    }
    
    if (FSwindow && FSwindow != GR_ROOT_WINDOW_ID) {
        GrDestroyWindow (FSwindow) ;
    }
    
    FSwindow = GrNewWindow (GR_ROOT_WINDOW_ID, 0, 0, 1, 1, 0, BLACK, BLACK) ;
    props.flags = GR_WM_FLAGS_PROPS ;
    props.props = GR_WM_PROPS_NODECORATE ;
    GrSetWMProperties (FSwindow, & props) ;

    GrSelectEvents (FSwindow, (GR_EVENT_MASK_EXPOSURE         |
        GR_EVENT_MASK_BUTTON_DOWN  | GR_EVENT_MASK_BUTTON_UP  |
        GR_EVENT_MASK_FOCUS_IN     | GR_EVENT_MASK_FOCUS_OUT  |
        GR_EVENT_MASK_KEY_DOWN     | GR_EVENT_MASK_KEY_UP     |
        GR_EVENT_MASK_MOUSE_ENTER  | GR_EVENT_MASK_MOUSE_EXIT |
        GR_EVENT_MASK_MOUSE_MOTION | GR_EVENT_MASK_UPDATE     |
        GR_EVENT_MASK_CLOSE_REQ)) ;

    Dprintf ("leave create_aux_windows\n") ;
}

int NX_VideoInit (_THIS, SDL_PixelFormat * vformat)
{
    GR_SCREEN_INFO si ;

    Dprintf ("enter NX_VideoInit\n") ;
    
    if (GrOpen () < 0) {
        SDL_SetError ("GrOpen() fail") ;
        return -1 ;
    }

    // use share memory to speed up
#ifdef NANOX_SHARE_MEMORY
    GrReqShmCmds (0xFFFF);
#endif

    SDL_Window = 0 ;
    FSwindow = 0 ;

    GammaRamp_R = NULL ;
    GammaRamp_G = NULL ;
    GammaRamp_B = NULL ;    

    GrGetScreenInfo (& si) ;
    SDL_Visual.bpp = si.bpp ;

    /* Determine the current screen size */
    this->info.current_w = si.cols ;
    this->info.current_h = si.rows ;

    // GetVideoMode
    SDL_modelist = (SDL_Rect **) SDL_malloc (sizeof (SDL_Rect *) * 2) ;
    if (SDL_modelist) {
        SDL_modelist [0] = (SDL_Rect *) SDL_malloc (sizeof(SDL_Rect)) ;
        if (SDL_modelist [0]) {
            SDL_modelist [0] -> x = 0 ;
            SDL_modelist [0] -> y = 0 ;
            SDL_modelist [0] -> w = si.cols ;
            SDL_modelist [0] -> h = si.rows ;
        }
        SDL_modelist [1] = NULL ;
    }

    pixel_type = si.pixtype;
    SDL_Visual.red_mask = si.rmask;
    SDL_Visual.green_mask = si.gmask;
    SDL_Visual.blue_mask = si.bmask;

    vformat -> BitsPerPixel = SDL_Visual.bpp ;
    if (vformat -> BitsPerPixel > 8) {
        vformat -> Rmask = SDL_Visual.red_mask ;
        vformat -> Gmask = SDL_Visual.green_mask ;
        vformat -> Bmask = SDL_Visual.blue_mask ;
    }

    // See if we have been passed a window to use
    SDL_windowid = getenv ("SDL_WINDOWID") ;
    
    // Create the fullscreen (and managed windows : no implement)
    create_aux_windows (this) ;

    Dprintf ("leave NX_VideoInit\n") ;
    return 0 ;
}

void NX_VideoQuit (_THIS)
{
    Dprintf ("enter NX_VideoQuit\n") ;

    // Start shutting down the windows
    NX_DestroyImage (this, this -> screen) ;
    NX_DestroyWindow (this, this -> screen) ;
    if (FSwindow && FSwindow != GR_ROOT_WINDOW_ID) {
        GrDestroyWindow (FSwindow) ;
    }
    NX_FreeVideoModes (this) ;
    SDL_free (GammaRamp_R) ;
    SDL_free (GammaRamp_G) ;
    SDL_free (GammaRamp_B) ;

#ifdef ENABLE_NANOX_DIRECT_FB
    if (Clientfb)
        GrCloseClientFramebuffer();
#endif
    GrClose () ;

    Dprintf ("leave NX_VideoQuit\n") ;
}

static void NX_DestroyWindow (_THIS, SDL_Surface * screen)
{
    Dprintf ("enter NX_DestroyWindow\n") ;

    if (! SDL_windowid) {
        if (screen && (screen -> flags & SDL_FULLSCREEN)) {
            screen -> flags &= ~ SDL_FULLSCREEN ;
            NX_LeaveFullScreen (this) ;
        }

        // Destroy the output window
        if (SDL_Window && SDL_Window != GR_ROOT_WINDOW_ID) {
            GrDestroyWindow (SDL_Window) ;
        }
    }
    
    // Free the graphics context
    if (! SDL_GC) {
        GrDestroyGC (SDL_GC) ;
        SDL_GC = 0;
    }

    Dprintf ("leave NX_DestroyWindow\n") ;
}

static int NX_CreateWindow (_THIS, SDL_Surface * screen,
                int w, int h, int bpp, Uint32 flags)
{
    Dprintf ("enter NX_CreateWindow\n") ;

    // If a window is already present, destroy it and start fresh
    if (SDL_Window && SDL_Window != GR_ROOT_WINDOW_ID) {
        NX_DestroyWindow (this, screen) ;
    }

    // See if we have been given a window id
    if (SDL_windowid) {
        SDL_Window = SDL_strtol (SDL_windowid, NULL, 0) ;
    } else {
        SDL_Window = 0 ;
    }
    
    if ( ! SDL_ReallocFormat (screen, bpp, SDL_Visual.red_mask, 
        SDL_Visual.green_mask, SDL_Visual.blue_mask, 0))
        return -1;

    // Create (or use) the nanox display window
    if (! SDL_windowid) {

        SDL_Window = GrNewWindow (GR_ROOT_WINDOW_ID, 0, 0, w, h, 0, BLACK, WHITE) ;

        GrSelectEvents (SDL_Window, (GR_EVENT_MASK_EXPOSURE       |
            GR_EVENT_MASK_BUTTON_DOWN  | GR_EVENT_MASK_BUTTON_UP  |
            GR_EVENT_MASK_FOCUS_IN     | GR_EVENT_MASK_FOCUS_OUT  |
            GR_EVENT_MASK_KEY_DOWN     | GR_EVENT_MASK_KEY_UP     |
            GR_EVENT_MASK_MOUSE_ENTER  | GR_EVENT_MASK_MOUSE_EXIT |
            GR_EVENT_MASK_MOUSE_MOTION | GR_EVENT_MASK_UPDATE     |
            GR_EVENT_MASK_CLOSE_REQ)) ;
    }
    
    /* Create the graphics context here, once we have a window */
    SDL_GC = GrNewGC () ;
    if (SDL_GC == 0) {
        SDL_SetError("Couldn't create graphics context");
        return(-1);
    }

    // Map them both and go fullscreen, if requested
    if (! SDL_windowid) {
        GrMapWindow (SDL_Window) ;
        if (flags & SDL_FULLSCREEN) {
            screen -> flags |= SDL_FULLSCREEN ;
            NX_EnterFullScreen (this) ;
        } else {
            screen -> flags &= ~ SDL_FULLSCREEN ;
        }
    }

#ifdef ENABLE_NANOX_DIRECT_FB
    /* attempt allocating the client side framebuffer */
    Clientfb = GrOpenClientFramebuffer();
    /* NULL return will default to using GrArea()*/
#endif

    Dprintf ("leave NX_CreateWindow\n") ;
    return 0 ;
}

SDL_Surface * NX_SetVideoMode (_THIS, SDL_Surface * current,
                int width, int height, int bpp, Uint32 flags)
{
    Dprintf ("enter NX_SetVideoMode\n") ;

    // Lock the event thread, in multi-threading environments
    SDL_Lock_EventThread () ;

    bpp = SDL_Visual.bpp ;
    if (NX_CreateWindow (this, current, width, height, bpp, flags) < 0) {
        current = NULL;
        goto done;
    }

    if (current -> w != width || current -> h != height) {
        current -> w = width ;
        current -> h = height ;
        current -> pitch = SDL_CalculatePitch (current) ;
        NX_ResizeImage (this, current, flags) ;
    }

    /* Clear these flags and set them only if they are in the new set. */
    current -> flags &= ~(SDL_RESIZABLE|SDL_NOFRAME);
    current -> flags |= (flags & (SDL_RESIZABLE | SDL_NOFRAME)) ;

  done:
    SDL_Unlock_EventThread () ;

    Dprintf ("leave NX_SetVideoMode\n") ;

    // We're done!
    return current ;
}

// ncolors <= 256
int NX_SetColors (_THIS, int firstcolor, int ncolors, SDL_Color * colors)
{
    int        i ;
    GR_PALETTE pal ;

    Dprintf ("enter NX_SetColors\n") ;

    if (ncolors > 256) return 0 ;
    
    pal.count = ncolors ;
    for (i = 0; i < ncolors; ++ i) {
        pal.palette [i].r = colors [i].r ;
        pal.palette [i].g = colors [i].g ;
        pal.palette [i].b = colors [i].b ;
    }
    GrSetSystemPalette (firstcolor, & pal) ;

    Dprintf ("leave NX_SetColors\n") ;
    return 1 ;
}

static int NX_ToggleFullScreen (_THIS, int on)
{
    SDL_Rect rect ;
    Uint32   event_thread ;
    
    Dprintf ("enter NX_ToggleFullScreen\n") ;

    // Don't switch if we don't own the window
    if (SDL_windowid) return 0 ;
    
    // Don't lock if we are the event thread
    event_thread = SDL_EventThreadID () ;
    if (event_thread && (SDL_ThreadID () == event_thread)) {
        event_thread = 0 ;
    }
    if (event_thread) {
        SDL_Lock_EventThread() ;
    }
    
    if (on) {
        NX_EnterFullScreen (this) ;
    } else {
        this -> screen -> flags &= ~ SDL_FULLSCREEN ;
        NX_LeaveFullScreen (this) ;
    }

    rect.x = rect.y = 0 ;
    rect.w = this -> screen -> w, rect.h = this -> screen -> h ;
    NX_NormalUpdate (this, 1, & rect) ;

    if (event_thread) {
        SDL_Unlock_EventThread () ;
    }
    
    Dprintf ("leave NX_ToggleFullScreen\n") ;
    return 1 ;
}

// Update the current mouse state and position
static void NX_UpdateMouse (_THIS)
{
    int            x, y ;
    GR_WINDOW_INFO info ;
    GR_SCREEN_INFO si ;


    Dprintf ("enter NX_UpdateMouse\n") ;

    // Lock the event thread, in multi-threading environments
    SDL_Lock_EventThread () ;
    
    GrGetScreenInfo (& si) ;
    GrGetWindowInfo (SDL_Window, & info) ;
    x = si.xpos - info.x ;
    y = si.ypos - info.y ;
    if (x >= 0 && x <= info.width && y >= 0 && y <= info.height) {
        SDL_PrivateAppActive (1, SDL_APPMOUSEFOCUS) ;
        SDL_PrivateMouseMotion (0, 0, x, y);
    } else {
        SDL_PrivateAppActive (0, SDL_APPMOUSEFOCUS) ;
    }

    SDL_Unlock_EventThread () ;
    Dprintf ("leave NX_UpdateMouse\n") ;
}

static int NX_SetGammaRamp (_THIS, Uint16 * ramp)
{
    int i ;
    Uint16 * red, * green, * blue ;
    
    Dprintf ("enter NX_SetGammaRamp\n") ;
    
    if (SDL_Visual.bpp != 32 && SDL_Visual.bpp != 24) return -1 ;

    if (! GammaRamp_R) GammaRamp_R = (Uint16 *) SDL_malloc (sizeof (Uint16) * CI_SIZE) ;
    if (! GammaRamp_G) GammaRamp_G = (Uint16 *) SDL_malloc (sizeof (Uint16) * CI_SIZE) ;
    if (! GammaRamp_B) GammaRamp_B = (Uint16 *) SDL_malloc (sizeof (Uint16) * CI_SIZE) ;
    if ((! GammaRamp_R) || (! GammaRamp_G) || (! GammaRamp_B)) {
        SDL_OutOfMemory () ;
        return -1 ;
    }

    for (i = 0; i < CI_SIZE; ++ i)
        GammaRamp_R [i] = GammaRamp_G [i] = GammaRamp_B [i] = i ;

    red   = ramp ;
    green = ramp + CI_SIZE ;
    blue  = green + CI_SIZE ;
        
    for (i = 0; i < CI_SIZE; ++ i) {
        GammaRamp_R [i] = red   [i] ;
        GammaRamp_G [i] = green [i] ;
        GammaRamp_B [i] = blue  [i] ;
    }
    SDL_UpdateRect(this->screen, 0, 0, 0, 0);

    Dprintf ("leave NX_SetGammaRamp\n") ;   
    return 0 ;
}

static int NX_GetGammaRamp (_THIS, Uint16 * ramp)
{
    int i ;
    Uint16 * red, * green, * blue ;

    Dprintf ("enter NX_GetGammaRamp\n") ;   

    if (SDL_Visual.bpp != 32 && SDL_Visual.bpp != 24) return -1 ;
    red   = ramp ;
    green = ramp  + CI_SIZE ;
    blue  = green + CI_SIZE ;
    if (GammaRamp_R && GammaRamp_G && GammaRamp_B) {
        for (i = 0; i < CI_SIZE; ++ i) {
            red   [i] = GammaRamp_R [i] ;
            green [i] = GammaRamp_G [i] ;
            blue  [i] = GammaRamp_B [i] ;
        }
    } else {
        for (i = 0; i < CI_SIZE; ++ i)
            red [i] = green [i] = blue [i] = i ;
    }

    Dprintf ("leave NX_GetGammaRamp\n") ;
    return 0 ;
}
