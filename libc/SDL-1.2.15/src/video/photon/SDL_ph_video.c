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

#include <unistd.h>
#include <sys/ioctl.h>

#include "SDL_endian.h"
#include "SDL_timer.h"
#include "SDL_thread.h"
#include "SDL_video.h"
#include "SDL_mouse.h"
#include "../SDL_sysvideo.h"
#include "../SDL_pixels_c.h"
#include "../../events/SDL_events_c.h"
#include "SDL_ph_video.h"
#include "SDL_ph_modes_c.h"
#include "SDL_ph_image_c.h"
#include "SDL_ph_events_c.h"
#include "SDL_ph_mouse_c.h"
#include "SDL_ph_wm_c.h"
#include "SDL_ph_gl.h"
#include "SDL_phyuv_c.h"
#include "../blank_cursor.h"

static int  ph_VideoInit(_THIS, SDL_PixelFormat *vformat);
static SDL_Surface *ph_SetVideoMode(_THIS, SDL_Surface *current, int width, int height, int bpp, Uint32 flags);
static int  ph_SetColors(_THIS, int firstcolor, int ncolors, SDL_Color *colors);
static void ph_VideoQuit(_THIS);
static void ph_DeleteDevice(SDL_VideoDevice *device);

static int phstatus=-1;

static int ph_Available(void)
{
    if (phstatus!=0)
    {
        phstatus=PtInit(NULL);
        if (phstatus==0)
        {
           return 1;
        }
        else
        {
           return 0;
        }
    }
    return 1;
}

static SDL_VideoDevice* ph_CreateDevice(int devindex)
{
    SDL_VideoDevice* device;

    /* Initialize all variables that we clean on shutdown */
    device = (SDL_VideoDevice *)SDL_malloc(sizeof(SDL_VideoDevice));
    if (device)
    {
        SDL_memset(device, 0, (sizeof *device));
        device->hidden = (struct SDL_PrivateVideoData*)SDL_malloc((sizeof *device->hidden));
        device->gl_data = NULL;
    }
    if ((device == NULL) || (device->hidden == NULL))
    {
        SDL_OutOfMemory();
        ph_DeleteDevice(device);
        return NULL;
    }
    SDL_memset(device->hidden, 0, (sizeof *device->hidden));

    /* Set the driver flags */
    device->handles_any_size = 1;

    /* Set the function pointers */
    device->CreateYUVOverlay = ph_CreateYUVOverlay;
    device->VideoInit = ph_VideoInit;
    device->ListModes = ph_ListModes;
    device->SetVideoMode = ph_SetVideoMode;
    device->ToggleFullScreen = ph_ToggleFullScreen;
    device->UpdateMouse = ph_UpdateMouse;
    device->SetColors = ph_SetColors;
    device->UpdateRects = NULL;                        /* set up in ph_SetupUpdateFunction */
    device->VideoQuit = ph_VideoQuit;
    device->AllocHWSurface = ph_AllocHWSurface;
    device->CheckHWBlit = ph_CheckHWBlit;
    device->FillHWRect = ph_FillHWRect;
    device->SetHWColorKey = ph_SetHWColorKey;
    device->SetHWAlpha = ph_SetHWAlpha;
    device->LockHWSurface = ph_LockHWSurface;
    device->UnlockHWSurface = ph_UnlockHWSurface;
    device->FlipHWSurface = ph_FlipHWSurface;
    device->FreeHWSurface = ph_FreeHWSurface;
    device->SetCaption = ph_SetCaption;
    device->SetIcon = NULL;
    device->IconifyWindow = ph_IconifyWindow;
    device->GrabInput = ph_GrabInput;
    device->GetWMInfo = ph_GetWMInfo;
    device->FreeWMCursor = ph_FreeWMCursor;
    device->CreateWMCursor = ph_CreateWMCursor;
    device->ShowWMCursor = ph_ShowWMCursor;
    device->WarpWMCursor = ph_WarpWMCursor;
    device->MoveWMCursor = NULL;
    device->CheckMouseMode = ph_CheckMouseMode;
    device->InitOSKeymap = ph_InitOSKeymap;
    device->PumpEvents = ph_PumpEvents;

    /* OpenGL support. */
#if SDL_VIDEO_OPENGL
    device->GL_MakeCurrent = ph_GL_MakeCurrent;
    device->GL_SwapBuffers = ph_GL_SwapBuffers;
    device->GL_GetAttribute = ph_GL_GetAttribute;
    device->GL_LoadLibrary = ph_GL_LoadLibrary;
    device->GL_GetProcAddress = ph_GL_GetProcAddress;
#endif /* SDL_VIDEO_OPENGL */

    device->free = ph_DeleteDevice;
    
    return device;
}

VideoBootStrap ph_bootstrap = {
    "photon", "QNX Photon video output",
    ph_Available, ph_CreateDevice
};

static void ph_DeleteDevice(SDL_VideoDevice *device)
{
    if (device)
    {
        if (device->hidden)
        {
            SDL_free(device->hidden);
            device->hidden = NULL;
        }
        if (device->gl_data)
        {
            SDL_free(device->gl_data);
            device->gl_data = NULL;
        }
        SDL_free(device);
        device = NULL;
    }
}

static PtWidget_t *ph_CreateWindow(_THIS)
{
    PtWidget_t *widget;
    
    widget = PtCreateWidget(PtWindow, NULL, 0, NULL);

    return widget;
}

static int ph_SetupWindow(_THIS, int w, int h, int flags)
{
    PtArg_t     args[32];
    PhPoint_t   pos = {0, 0};
    PhDim_t*    olddim;
    PhDim_t     dim = {w, h};
    PhRect_t    desktopextent;
    int         nargs = 0;
    const char* windowpos;
    const char* iscentered;
    int         x, y;

    /* check if window size has been changed by Window Manager */
    PtGetResource(window, Pt_ARG_DIM, &olddim, 0);
    if ((olddim->w!=w) || (olddim->h!=h))
    {
       PtSetArg(&args[nargs++], Pt_ARG_DIM, &dim, 0);
    }

    if ((flags & SDL_RESIZABLE) == SDL_RESIZABLE)
    {
        PtSetArg(&args[nargs++], Pt_ARG_WINDOW_MANAGED_FLAGS, Pt_FALSE, Ph_WM_CLOSE);
        PtSetArg(&args[nargs++], Pt_ARG_WINDOW_MANAGED_FLAGS, Pt_TRUE, Ph_WM_MAX | Ph_WM_RESTORE | Ph_WM_RESIZE);
        PtSetArg(&args[nargs++], Pt_ARG_WINDOW_NOTIFY_FLAGS, Pt_TRUE, Ph_WM_RESIZE | Ph_WM_MOVE | Ph_WM_CLOSE | Ph_WM_MAX | Ph_WM_RESTORE);
        PtSetArg(&args[nargs++], Pt_ARG_WINDOW_RENDER_FLAGS, Pt_TRUE, Ph_WM_RENDER_RESIZE | Ph_WM_RENDER_MAX | Ph_WM_RENDER_COLLAPSE | Ph_WM_RENDER_RETURN);
        PtSetArg(&args[nargs++], Pt_ARG_RESIZE_FLAGS, Pt_TRUE, Pt_RESIZE_XY_AS_REQUIRED);
    }
    else
    {
        PtSetArg(&args[nargs++], Pt_ARG_WINDOW_MANAGED_FLAGS, Pt_FALSE, Ph_WM_RESIZE | Ph_WM_MAX | Ph_WM_RESTORE | Ph_WM_CLOSE);
        PtSetArg(&args[nargs++], Pt_ARG_WINDOW_NOTIFY_FLAGS, Pt_FALSE, Ph_WM_RESIZE | Ph_WM_MAX | Ph_WM_RESTORE);
        PtSetArg(&args[nargs++], Pt_ARG_WINDOW_NOTIFY_FLAGS, Pt_TRUE, Ph_WM_MOVE | Ph_WM_CLOSE);
        PtSetArg(&args[nargs++], Pt_ARG_WINDOW_RENDER_FLAGS, Pt_FALSE, Ph_WM_RENDER_RESIZE | Ph_WM_RENDER_MAX | Ph_WM_RENDER_COLLAPSE | Ph_WM_RENDER_RETURN);
        PtSetArg(&args[nargs++], Pt_ARG_RESIZE_FLAGS, Pt_FALSE, Pt_RESIZE_XY_AS_REQUIRED);
    }

    if (((flags & SDL_NOFRAME)==SDL_NOFRAME) || ((flags & SDL_FULLSCREEN)==SDL_FULLSCREEN))
    {
       if ((flags & SDL_RESIZABLE) != SDL_RESIZABLE)
       {
           PtSetArg(&args[nargs++], Pt_ARG_WINDOW_RENDER_FLAGS, Pt_FALSE, Pt_TRUE);
       }
       else
       {
           PtSetArg(&args[nargs++], Pt_ARG_WINDOW_RENDER_FLAGS, Pt_FALSE, Pt_TRUE);
           PtSetArg(&args[nargs++], Pt_ARG_WINDOW_RENDER_FLAGS, Pt_TRUE, Ph_WM_RENDER_RESIZE | Ph_WM_RENDER_BORDER);
       }
    }
    else
    {
        PtSetArg(&args[nargs++], Pt_ARG_WINDOW_RENDER_FLAGS, Pt_TRUE, Ph_WM_RENDER_BORDER | Ph_WM_RENDER_TITLE |
                                 Ph_WM_RENDER_CLOSE | Ph_WM_RENDER_MENU | Ph_WM_RENDER_MIN);
    }

    if ((flags & SDL_FULLSCREEN) == SDL_FULLSCREEN)
    {
        PtSetArg(&args[nargs++], Pt_ARG_POS, &pos, 0);
        PtSetArg(&args[nargs++], Pt_ARG_BASIC_FLAGS, Pt_TRUE, Pt_BASIC_PREVENT_FILL);
        PtSetArg(&args[nargs++], Pt_ARG_WINDOW_MANAGED_FLAGS, Pt_TRUE, Ph_WM_FFRONT | Ph_WM_MAX | Ph_WM_TOFRONT | Ph_WM_CONSWITCH);
        PtSetArg(&args[nargs++], Pt_ARG_WINDOW_STATE, Pt_TRUE, Ph_WM_STATE_ISFRONT | Ph_WM_STATE_ISFOCUS | Ph_WM_STATE_ISALTKEY);
    }
    else
    {
        PtSetArg(&args[nargs++], Pt_ARG_WINDOW_MANAGED_FLAGS, Pt_FALSE, Ph_WM_FFRONT | Ph_WM_CONSWITCH);
        PtSetArg(&args[nargs++], Pt_ARG_WINDOW_STATE, Pt_FALSE, Ph_WM_STATE_ISFRONT);
        PtSetArg(&args[nargs++], Pt_ARG_WINDOW_STATE, Pt_TRUE, Ph_WM_STATE_ISALTKEY);

        if ((flags & SDL_HWSURFACE) == SDL_HWSURFACE)
        {
            PtSetArg(&args[nargs++], Pt_ARG_BASIC_FLAGS, Pt_TRUE, Pt_BASIC_PREVENT_FILL);
        }
        else
        {
            PtSetArg(&args[nargs++], Pt_ARG_FILL_COLOR, Pg_BLACK, 0);
        }
        if (!currently_maximized)
        {
            windowpos = SDL_getenv("SDL_VIDEO_WINDOW_POS");
            iscentered = SDL_getenv("SDL_VIDEO_CENTERED");

            if ((iscentered) || ((windowpos) && (SDL_strcmp(windowpos, "center")==0)))
            {
                PhWindowQueryVisible(Ph_QUERY_CONSOLE, 0, 0, &desktopextent);
                if (desktop_mode.width>w)
                {
                    pos.x = (desktop_mode.width - w)/2;
                }
                if (desktop_mode.height>h)
                {
                    pos.y = (desktop_mode.height - h)/2;
                }

                pos.x+=desktopextent.ul.x;
                pos.y+=desktopextent.ul.y;
                PtSetArg(&args[nargs++], Pt_ARG_POS, &pos, 0);
            }
            else
            {
                if (windowpos)
                {
                    if (SDL_sscanf(windowpos, "%d,%d", &x, &y) == 2)
                    {
                        if ((x<desktop_mode.width) && (y<desktop_mode.height))
                        {
                            PhWindowQueryVisible(Ph_QUERY_CONSOLE, 0, 0, &desktopextent);
                            pos.x=x+desktopextent.ul.x;
                            pos.y=y+desktopextent.ul.y;
                        }
                        PtSetArg(&args[nargs++], Pt_ARG_POS, &pos, 0);
                    }
                }
            }
        }

        /* if window is maximized render it as maximized */
        if (currently_maximized)
        {
           PtSetArg(&args[nargs++], Pt_ARG_WINDOW_STATE, Pt_TRUE, Ph_WM_STATE_ISMAX);
        }
        else
        {
           PtSetArg(&args[nargs++], Pt_ARG_WINDOW_STATE, Pt_FALSE, Ph_WM_STATE_ISMAX);
        }

        /* do not grab the keyboard by default */
        PtSetArg(&args[nargs++], Pt_ARG_WINDOW_STATE, Pt_FALSE, Ph_WM_STATE_ISALTKEY);

        /* bring the focus to the window */
        PtSetArg(&args[nargs++], Pt_ARG_WINDOW_STATE, Pt_TRUE, Ph_WM_STATE_ISFOCUS);

        /* allow to catch hide event */
        PtSetArg(&args[nargs++], Pt_ARG_WINDOW_MANAGED_FLAGS, Pt_TRUE, Ph_WM_HIDE);
        PtSetArg(&args[nargs++], Pt_ARG_WINDOW_NOTIFY_FLAGS, Pt_TRUE, Ph_WM_HIDE);
    }

    PtSetResources(window, nargs, args);
    PtRealizeWidget(window);
    PtWindowToFront(window);

#if 0 /* FIXME */
    PtGetResource(window, Pt_ARG_POS, &olddim, 0);
    fprintf(stderr, "POSITION: %d, %d\n", olddim->w, olddim->h);
#endif

    return 0;
}

static const struct ColourMasks* ph_GetColourMasks(int bpp)
{
    /* The alpha mask doesn't appears to be needed */
    static const struct ColourMasks phColorMasks[5] = {
        /*  8 bit      */  {0, 0, 0, 0, 8},
        /* 15 bit ARGB */  {0x7C00, 0x03E0, 0x001F, 0x8000, 15},
        /* 16 bit  RGB */  {0xF800, 0x07E0, 0x001F, 0x0000, 16},
        /* 24 bit  RGB */  {0xFF0000, 0x00FF00, 0x0000FF, 0x000000, 24},
        /* 32 bit ARGB */  {0x00FF0000, 0x0000FF00, 0x000000FF, 0xFF000000, 32},
    };

    switch (bpp)
    {
        case 8:
             return &phColorMasks[0];
        case 15:
             return &phColorMasks[1];
        case 16:
             return &phColorMasks[2];
        case 24:
             return &phColorMasks[3];
        case 32:
             return &phColorMasks[4];
    }
    return NULL;
}

static int ph_VideoInit(_THIS, SDL_PixelFormat* vformat)
{
    PgHWCaps_t hwcaps;
    int i;

    window=NULL;
    desktoppal=SDLPH_PAL_NONE;

#if SDL_VIDEO_OPENGL
    oglctx=NULL;
    oglbuffers=NULL;
    oglflags=0;
    oglbpp=0;
#endif
    
    old_video_mode=-1;
    old_refresh_rate=-1;
	
    if (NULL == (phevent = SDL_malloc(EVENT_SIZE)))
    {
        SDL_OutOfMemory();
        return -1;
    }
    SDL_memset(phevent, 0x00, EVENT_SIZE);

    window = ph_CreateWindow(this);
    if (window == NULL)
    {
        SDL_SetError("ph_VideoInit(): Couldn't create video window !\n");
        return -1;
    }

    /* Create the blank cursor */
    SDL_BlankCursor = this->CreateWMCursor(this, blank_cdata, blank_cmask,
                                          (int)BLANK_CWIDTH, (int)BLANK_CHEIGHT,
                                          (int)BLANK_CHOTX, (int)BLANK_CHOTY);

    if (SDL_BlankCursor == NULL)
    {
        return -1;
    }

    if (PgGetGraphicsHWCaps(&hwcaps) < 0)
    {
        SDL_SetError("ph_VideoInit(): GetGraphicsHWCaps function failed !\n");
        this->FreeWMCursor(this, SDL_BlankCursor);
        return -1;
    }

    if (PgGetVideoModeInfo(hwcaps.current_video_mode, &desktop_mode) < 0)
    {
        SDL_SetError("ph_VideoInit(): PgGetVideoModeInfo function failed !\n");
        this->FreeWMCursor(this, SDL_BlankCursor);
        return -1;
    }

   /* Determine the current screen size */
   this->info.current_w = desktop_mode.width;
   this->info.current_h = desktop_mode.height;

    /* We need to return BytesPerPixel as it in used by CreateRGBsurface */
    vformat->BitsPerPixel = desktop_mode.bits_per_pixel;
    vformat->BytesPerPixel = desktop_mode.bytes_per_scanline/desktop_mode.width;
    desktopbpp = desktop_mode.bits_per_pixel;
    
    /* save current palette */
    if (desktopbpp==8)
    {
        PgGetPalette(savedpal);
        PgGetPalette(syspalph);
    }
    else
    {
        for(i=0; i<_Pg_MAX_PALETTE; i++)
        {
            savedpal[i]=PgRGB(0, 0, 0);
            syspalph[i]=PgRGB(0, 0, 0);
        }
    }
         
    currently_fullscreen = 0;
    currently_hided = 0;
    currently_maximized = 0;
    current_overlay = NULL;

    OCImage.direct_context = NULL;
    OCImage.offscreen_context = NULL;
    OCImage.offscreen_backcontext = NULL;
    OCImage.oldDC = NULL;
    OCImage.CurrentFrameData = NULL;
    OCImage.FrameData0 = NULL;
    OCImage.FrameData1 = NULL;
    videomode_emulatemode = 0;
    
    this->info.wm_available = 1;

    ph_UpdateHWInfo(this);
    
    return 0;
}

static SDL_Surface* ph_SetVideoMode(_THIS, SDL_Surface *current, int width, int height, int bpp, Uint32 flags)
{
    const struct ColourMasks* mask;

    /* Lock the event thread, in multi-threading environments */
    SDL_Lock_EventThread();

    current->flags = flags;

    /* if we do not have desired fullscreen mode, then fallback into window mode */
    if (((current->flags & SDL_FULLSCREEN) == SDL_FULLSCREEN) && (ph_GetVideoMode(width, height, bpp)==0))
    {
       current->flags &= ~SDL_FULLSCREEN;
       current->flags &= ~SDL_NOFRAME;
       current->flags &= ~SDL_RESIZABLE;
    }

    ph_SetupWindow(this, width, height, current->flags);

    mask = ph_GetColourMasks(bpp);
    if (mask != NULL)
    {
        SDL_ReallocFormat(current, mask->bpp, mask->red, mask->green, mask->blue, 0);
    }
    else
    {
        SDL_SetError("ph_SetVideoMode(): desired bpp is not supported by photon !\n");
        return NULL;
    }

    if ((current->flags & SDL_OPENGL)==SDL_OPENGL)
    {
#if !SDL_VIDEO_OPENGL
        /* if no built-in OpenGL support */
        SDL_SetError("ph_SetVideoMode(): no OpenGL support, you need to recompile SDL.\n");
        current->flags &= ~SDL_OPENGL;
        return NULL;
#endif /* SDL_VIDEO_OPENGL */
    }
    else
    {
        /* Initialize internal variables */
        if ((current->flags & SDL_FULLSCREEN) == SDL_FULLSCREEN)
        {
            if (bpp==8)
            {
               desktoppal=SDLPH_PAL_SYSTEM;
            }

            current->flags &= ~SDL_RESIZABLE; /* no resize for Direct Context */
            current->flags |= SDL_HWSURFACE;
        }
        else
        {
            /* remove this if we'll have support for the non-fullscreen sw/hw+doublebuf one day */
            current->flags &= ~SDL_DOUBLEBUF;

            /* Use offscreen memory if SDL_HWSURFACE flag is set */
            if ((current->flags & SDL_HWSURFACE) == SDL_HWSURFACE)
            {
                if (desktopbpp!=bpp)
                {
                   current->flags &= ~SDL_HWSURFACE;
                }
            }

            /* using palette emulation code in window mode */
            if (bpp==8)
            {
                if (desktopbpp>=15)
                {
                    desktoppal = SDLPH_PAL_EMULATE;
                }
                else
                {
                    desktoppal = SDLPH_PAL_SYSTEM;
                }
            }
            else
            {
               desktoppal = SDLPH_PAL_NONE;
            }
        }
    }

    current->w = width;
    current->h = height;

    if (desktoppal==SDLPH_PAL_SYSTEM)
    {
       current->flags|=SDL_HWPALETTE;
    }

    /* Must call at least once for setup image planes */
    if (ph_SetupUpdateFunction(this, current, current->flags)==-1)
    {
        /* Error string was filled in the ph_SetupUpdateFunction() */
        return NULL;
    }

    /* finish window drawing, if we are not in fullscreen, of course */
    if ((current->flags & SDL_FULLSCREEN) != SDL_FULLSCREEN)
    {
       PtFlush();
    }
    else
    {
       PgFlush();
    }

    visualbpp=bpp;

    ph_UpdateHWInfo(this);

    SDL_Unlock_EventThread();

    /* We've done! */
    return (current);
}

static void ph_VideoQuit(_THIS)
{
    /* restore palette */
    if (desktopbpp==8)
    {
        PgSetPalette(syspalph, 0, -1, 0, 0, 0);
        PgSetPalette(savedpal, 0, 0, _Pg_MAX_PALETTE, Pg_PALSET_GLOBAL | Pg_PALSET_FORCE_EXPOSE, 0);
        PgFlush();
    }

    ph_DestroyImage(this, SDL_VideoSurface); 

    if (window)
    {
        PtUnrealizeWidget(window);
        PtDestroyWidget(window);
        window=NULL;
    }

    if (phevent!=NULL)
    {
        SDL_free(phevent);
        phevent=NULL;
    }
}

static int ph_SetColors(_THIS, int firstcolor, int ncolors, SDL_Color *colors)
{
    int i;
    SDL_Rect updaterect;

    updaterect.x = updaterect.y = 0;
    updaterect.w = this->screen->w;
    updaterect.h = this->screen->h;

    /* palette emulation code, using palette of the PhImage_t struct */
    if (desktoppal==SDLPH_PAL_EMULATE)
    {
        if ((SDL_Image) && (SDL_Image->palette))
        {
            for (i=firstcolor; i<firstcolor+ncolors; i++)
            {
                syspalph[i] = PgRGB(colors[i-firstcolor].r, colors[i-firstcolor].g, colors[i-firstcolor].b);
                SDL_Image->palette[i] = syspalph[i];
            }

            /* image needs to be redrawn */
            this->UpdateRects(this, 1, &updaterect);
        }
    }
    else
    {
        if (desktoppal==SDLPH_PAL_SYSTEM)
        {
            for (i=firstcolor; i<firstcolor+ncolors; i++)
            {
                syspalph[i] = PgRGB(colors[i-firstcolor].r, colors[i-firstcolor].g, colors[i-firstcolor].b);
            }

            if ((this->screen->flags & SDL_FULLSCREEN) != SDL_FULLSCREEN)
            {
                 /* window mode must use soft palette */
                PgSetPalette(&syspalph[firstcolor], 0, firstcolor, ncolors, Pg_PALSET_GLOBAL, 0);
                /* image needs to be redrawn */
                this->UpdateRects(this, 1, &updaterect);
            }
            else
            {
                /* fullscreen mode must use hardware palette */
                PgSetPalette(&syspalph[firstcolor], 0, firstcolor, ncolors, Pg_PALSET_GLOBAL, 0);
            }
        }
        else
        {
            /* SDLPH_PAL_NONE do nothing */
        }
    }
    
    return 1;
}

