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

/* Utilities for getting and setting the X display mode */

#include <stdio.h>

#include "SDL_timer.h"
#include "SDL_events.h"
#include "../../events/SDL_events_c.h"
#include "SDL_x11video.h"
#include "SDL_x11wm_c.h"
#include "SDL_x11modes_c.h"
#include "SDL_x11image_c.h"

/*#define X11MODES_DEBUG*/

#define MAX(a, b)        (a > b ? a : b)

#if SDL_VIDEO_DRIVER_X11_XRANDR
static int cmpmodelist(const void *va, const void *vb)
{
    const SDL_Rect *a = *(const SDL_Rect **)va;
    const SDL_Rect *b = *(const SDL_Rect **)vb;
    if ( a->w == b->w )
        return b->h - a->h;
    else
        return b->w - a->w;
}
#endif

#if SDL_VIDEO_DRIVER_X11_VIDMODE
Bool SDL_NAME(XF86VidModeGetModeInfo)(Display *dpy, int scr, SDL_NAME(XF86VidModeModeInfo) *info)
{
    Bool retval;
    int dotclock;
    SDL_NAME(XF86VidModeModeLine) l;
    SDL_memset(&l, 0, sizeof(l));
    retval = SDL_NAME(XF86VidModeGetModeLine)(dpy, scr, &dotclock, &l);
    info->dotclock = dotclock;
    info->hdisplay = l.hdisplay;
    info->hsyncstart = l.hsyncstart;
    info->hsyncend = l.hsyncend;
    info->htotal = l.htotal;
    info->hskew = l.hskew;
    info->vdisplay = l.vdisplay;
    info->vsyncstart = l.vsyncstart;
    info->vsyncend = l.vsyncend;
    info->vtotal = l.vtotal;
    info->flags = l.flags;
    info->privsize = l.privsize;
    info->private = l.private;
    return retval;
}
#endif /* SDL_VIDEO_DRIVER_X11_VIDMODE */

#if SDL_VIDEO_DRIVER_X11_VIDMODE
static void save_mode(_THIS)
{
    SDL_memset(&saved_mode, 0, sizeof(saved_mode));
    SDL_NAME(XF86VidModeGetModeInfo)(SDL_Display,SDL_Screen,&saved_mode);
    SDL_NAME(XF86VidModeGetViewPort)(SDL_Display,SDL_Screen,&saved_view.x,&saved_view.y);
}
#endif

#if SDL_VIDEO_DRIVER_X11_VIDMODE
static void restore_mode(_THIS)
{
    SDL_NAME(XF86VidModeModeLine) mode;
    int unused;

    if ( SDL_NAME(XF86VidModeGetModeLine)(SDL_Display, SDL_Screen, &unused, &mode) ) {
        if ( (saved_mode.hdisplay != mode.hdisplay) ||
             (saved_mode.vdisplay != mode.vdisplay) ) {
            SDL_NAME(XF86VidModeSwitchToMode)(SDL_Display, SDL_Screen, &saved_mode);
        }
    }
    if ( (saved_view.x != 0) || (saved_view.y != 0) ) {
        SDL_NAME(XF86VidModeSetViewPort)(SDL_Display, SDL_Screen, saved_view.x, saved_view.y);
    }
}
#endif

#if SDL_VIDEO_DRIVER_X11_VIDMODE
static int cmpmodes(const void *va, const void *vb)
{
    const SDL_NAME(XF86VidModeModeInfo) *a = *(const SDL_NAME(XF86VidModeModeInfo)**)va;
    const SDL_NAME(XF86VidModeModeInfo) *b = *(const SDL_NAME(XF86VidModeModeInfo)**)vb;
    if ( a->hdisplay == b->hdisplay )
        return b->vdisplay - a->vdisplay;
    else
        return b->hdisplay - a->hdisplay;
}
#endif

static void get_real_resolution(_THIS, int* w, int* h);

static void set_best_resolution(_THIS, int width, int height)
{
#if SDL_VIDEO_DRIVER_X11_VIDMODE
    if ( use_vidmode ) {
        SDL_NAME(XF86VidModeModeLine) mode;
        SDL_NAME(XF86VidModeModeInfo) **modes;
        int i;
        int nmodes;
        int best = -1;

        if ( SDL_NAME(XF86VidModeGetModeLine)(SDL_Display, SDL_Screen, &i, &mode) &&
             SDL_NAME(XF86VidModeGetAllModeLines)(SDL_Display,SDL_Screen,&nmodes,&modes) ) {
            for ( i = 0; i < nmodes ; i++ ) {
                if ( (modes[i]->hdisplay == width) &&
                     (modes[i]->vdisplay == height) ) {
                    best = i;
                    break;
                }
                if ( modes[i]->hdisplay >= width &&
                     modes[i]->vdisplay >= height ) {
                    if ( best < 0 ||
                         (modes[i]->hdisplay < modes[best]->hdisplay &&
                          modes[i]->vdisplay <= modes[best]->vdisplay) ||
                         (modes[i]->vdisplay < modes[best]->vdisplay &&
                          modes[i]->hdisplay <= modes[best]->hdisplay) ) {
                        best = i;
                    }
                }
            }
            if ( best >= 0 &&
                 ((modes[best]->hdisplay != mode.hdisplay) ||
                  (modes[best]->vdisplay != mode.vdisplay)) ) {
#ifdef X11MODES_DEBUG
                printf("Best Mode %d: %d x %d @ %d\n", best,
                        modes[best]->hdisplay, modes[best]->vdisplay,
                        (modes[best]->htotal && modes[best]->vtotal) ? (1000 * modes[best]->dotclock / (modes[best]->htotal * modes[best]->vtotal)) : 0 );
#endif
                SDL_NAME(XF86VidModeSwitchToMode)(SDL_Display, SDL_Screen, modes[best]);
            }
            XFree(modes);
        }
    }
#endif /* SDL_VIDEO_DRIVER_X11_VIDMODE */

                                /* XiG */
#if SDL_VIDEO_DRIVER_X11_XME
    if ( use_xme && SDL_modelist ) {
        int i;

#ifdef X11MODES_DEBUG
        fprintf(stderr, "XME: set_best_resolution(): w = %d, h = %d\n",
                width, height);
#endif
        for ( i=0; SDL_modelist[i]; ++i ) {
            if ( (SDL_modelist[i]->w >= width) &&
                 (SDL_modelist[i]->h >= height) ) {
                break;
            }
        }
        
        if ( SDL_modelist[i] ) { /* found one, lets try it */
            int w, h;        

            /* check current mode so we can avoid uneccessary mode changes */
            get_real_resolution(this, &w, &h);

            if ( (SDL_modelist[i]->w != w) || (SDL_modelist[i]->h != h) ) {
#ifdef X11MODES_DEBUG
                fprintf(stderr, "XME: set_best_resolution: "
                        "XiGMiscChangeResolution: %d %d\n",
                        SDL_modelist[i]->w, SDL_modelist[i]->h);
#endif
                XiGMiscChangeResolution(SDL_Display, 
                                        SDL_Screen,
                                        0, /* view */
                                        SDL_modelist[i]->w, 
                                        SDL_modelist[i]->h, 
                                        0);
                XSync(SDL_Display, False);
            }
        }
    }
#endif /* SDL_VIDEO_DRIVER_X11_XME */

#if SDL_VIDEO_DRIVER_X11_XRANDR
    if ( use_xrandr && SDL_modelist ) {
#ifdef X11MODES_DEBUG
        fprintf(stderr, "XRANDR: set_best_resolution(): w = %d, h = %d\n",
                width, height);
#endif
        int i, nsizes;
        XRRScreenSize *sizes;

        /* find the smallest resolution that is at least as big as the user requested */
        sizes = XRRConfigSizes(screen_config, &nsizes);
        for ( i = (nsizes-1); i >= 0; i-- ) {
            if ( (SDL_modelist[i]->w >= width) &&
                 (SDL_modelist[i]->h >= height) ) {
                break;
            }
        }

        if ( i >= 0 && SDL_modelist[i] ) { /* found one, lets try it */
            int w, h;

            /* check current mode so we can avoid uneccessary mode changes */
            get_real_resolution(this, &w, &h);

            if ( (SDL_modelist[i]->w != w) || (SDL_modelist[i]->h != h) ) {
                int size_id;

#ifdef X11MODES_DEBUG
                fprintf(stderr, "XRANDR: set_best_resolution: "
                        "XXRSetScreenConfig: %d %d\n",
                        SDL_modelist[i]->w, SDL_modelist[i]->h);
#endif

                /* find the matching size entry index */
                for ( size_id = 0; size_id < nsizes; ++size_id ) {
                    if ( (sizes[size_id].width == SDL_modelist[i]->w) &&
                         (sizes[size_id].height == SDL_modelist[i]->h) )
                        break;
                }

                XRRSetScreenConfig(SDL_Display, screen_config, SDL_Root,
                                   size_id, saved_rotation, CurrentTime);
            }
        }
    }
#endif /* SDL_VIDEO_DRIVER_X11_XRANDR */
}

static void get_real_resolution(_THIS, int* w, int* h)
{
#if SDL_VIDEO_DRIVER_X11_XME
    if ( use_xme ) {
        int ractive;
        XiGMiscResolutionInfo *modelist;

        XiGMiscQueryResolutions(SDL_Display, SDL_Screen,
                                0, /* view */
                                &ractive, &modelist);
        *w = modelist[ractive].width;
        *h = modelist[ractive].height;
#ifdef X11MODES_DEBUG
        fprintf(stderr, "XME: get_real_resolution: w = %d h = %d\n", *w, *h);
#endif
        XFree(modelist);
        return;
    }
#endif /* SDL_VIDEO_DRIVER_X11_XME */

#if SDL_VIDEO_DRIVER_X11_VIDMODE
    if ( use_vidmode ) {
        SDL_NAME(XF86VidModeModeLine) mode;
        int unused;

        if ( SDL_NAME(XF86VidModeGetModeLine)(SDL_Display, SDL_Screen, &unused, &mode) ) {
            *w = mode.hdisplay;
            *h = mode.vdisplay;
            return;
        }
    }
#endif /* SDL_VIDEO_DRIVER_X11_VIDMODE */

#if SDL_VIDEO_DRIVER_X11_XRANDR
    if ( use_xrandr ) {
        int nsizes;
        XRRScreenSize* sizes;

        sizes = XRRConfigSizes(screen_config, &nsizes);
        if ( nsizes > 0 ) {
            int cur_size;
            Rotation cur_rotation;

            cur_size = XRRConfigCurrentConfiguration(screen_config, &cur_rotation);
            if ( cur_size >= 0 && cur_size < nsizes ) {
                *w = sizes[cur_size].width;
                *h = sizes[cur_size].height;
            }
#ifdef X11MODES_DEBUG
            fprintf(stderr, "XRANDR: get_real_resolution: w = %d h = %d\n", *w, *h);
#endif
            return;
        }
    }
#endif /* SDL_VIDEO_DRIVER_X11_XRANDR */

#if SDL_VIDEO_DRIVER_X11_XINERAMA
    if ( use_xinerama ) {
        *w = xinerama_info.width;
        *h = xinerama_info.height;
        return;
    }
#endif /* SDL_VIDEO_DRIVER_X11_XINERAMA */

    *w = DisplayWidth(SDL_Display, SDL_Screen);
    *h = DisplayHeight(SDL_Display, SDL_Screen);
}

/* Called after mapping a window - waits until the window is mapped */
void X11_WaitMapped(_THIS, Window win)
{
    XEvent event;
    do {
        XMaskEvent(SDL_Display, StructureNotifyMask, &event);
    } while ( (event.type != MapNotify) || (event.xmap.event != win) );
}

/* Called after unmapping a window - waits until the window is unmapped */
void X11_WaitUnmapped(_THIS, Window win)
{
    XEvent event;
    do {
        XMaskEvent(SDL_Display, StructureNotifyMask, &event);
    } while ( (event.type != UnmapNotify) || (event.xunmap.event != win) );
}

static void move_cursor_to(_THIS, int x, int y)
{
    XWarpPointer(SDL_Display, None, SDL_Root, 0, 0, 0, 0, x, y);
}

static int add_default_visual(_THIS)
{
    int i;
    int n = this->hidden->nvisuals;
    for (i=0; i<n; i++) {
        if (this->hidden->visuals[i].visual == DefaultVisual(SDL_Display, SDL_Screen)) return n;
    }
    this->hidden->visuals[n].depth = DefaultDepth(SDL_Display, SDL_Screen);;
    this->hidden->visuals[n].visual = DefaultVisual(SDL_Display, SDL_Screen);;
    this->hidden->nvisuals++;
    return(this->hidden->nvisuals);
}
static int add_visual(_THIS, int depth, int class)
{
    XVisualInfo vi;
    if(XMatchVisualInfo(SDL_Display, SDL_Screen, depth, class, &vi)) {
        int n = this->hidden->nvisuals;
        this->hidden->visuals[n].depth = vi.depth;
        this->hidden->visuals[n].visual = vi.visual;
        this->hidden->nvisuals++;
    }
    return(this->hidden->nvisuals);
}
static int add_visual_byid(_THIS, const char *visual_id)
{
    XVisualInfo *vi, template;
    int nvis;

    if ( visual_id ) {
        SDL_memset(&template, 0, (sizeof template));
        template.visualid = SDL_strtol(visual_id, NULL, 0);
        vi = XGetVisualInfo(SDL_Display, VisualIDMask, &template, &nvis);
        if ( vi ) {
            int n = this->hidden->nvisuals;
            this->hidden->visuals[n].depth = vi->depth;
            this->hidden->visuals[n].visual = vi->visual;
            this->hidden->nvisuals++;
            XFree(vi);
        }
    }
    return(this->hidden->nvisuals);
}

/* Global for the error handler */
int vm_event, vm_error = -1;

#if SDL_VIDEO_DRIVER_X11_XINERAMA
static int CheckXinerama(_THIS, int *major, int *minor)
{
    const char *env;

    /* Default the extension not available */
    *major = *minor = 0;

    /* Allow environment override */
    env = getenv("SDL_VIDEO_X11_XINERAMA");
    if ( env && !SDL_atoi(env) ) {
        return 0;
    }

    /* Query the extension version */
    if ( !SDL_NAME(XineramaQueryExtension)(SDL_Display, major, minor) ||
         !SDL_NAME(XineramaIsActive)(SDL_Display) ) {
        return 0;
    }
    return 1;
}
#endif /* SDL_VIDEO_DRIVER_X11_XINERAMA */

#if SDL_VIDEO_DRIVER_X11_XRANDR
static int CheckXRandR(_THIS, int *major, int *minor)
{
    const char *env;

    /* Default the extension not available */
    *major = *minor = 0;

    /* Allow environment override */
    env = getenv("SDL_VIDEO_X11_XRANDR");
    if ( env && !SDL_atoi(env) ) {
        return 0;
    }

    /* This defaults off now, due to KDE window maximize problems */
    if ( !env ) {
        return 0;
    }

    if ( !SDL_X11_HAVE_XRANDR ) {
        return 0;
    }

    /* Query the extension version */
    if ( !XRRQueryVersion(SDL_Display, major, minor) ) {
        return 0;
    }
    return 1;
}
#endif /* SDL_VIDEO_DRIVER_X11_XRANDR */

#if SDL_VIDEO_DRIVER_X11_VIDMODE
static int CheckVidMode(_THIS, int *major, int *minor)
{
    const char *env;

    /* Default the extension not available */
    *major = *minor = 0;

    /* Allow environment override */
    env = getenv("SDL_VIDEO_X11_VIDMODE");
    if ( env && !SDL_atoi(env) ) {
        return 0;
    }
    
    /* Metro-X 4.3.0 and earlier has a broken implementation of
       XF86VidModeGetAllModeLines() - it hangs the client.
     */
    if ( SDL_strcmp(ServerVendor(SDL_Display), "Metro Link Incorporated") == 0 ) {
        FILE *metro_fp;

        metro_fp = fopen("/usr/X11R6/lib/X11/Metro/.version", "r");
        if ( metro_fp != NULL ) {
            int major, minor, patch, version, scannum;
            major = 0; minor = 0; patch = 0;
            scannum = fscanf(metro_fp, "%d.%d.%d", &major, &minor, &patch);
            fclose(metro_fp);
            if ( (scannum < 0) || (scannum > 3) ) {
                return 0;  /* we need _something_ useful from fscanf(). */
            }
            version = major*100+minor*10+patch;
            if ( version < 431 ) {
                return 0;
            }
        }
    }

    /* Query the extension version */
    vm_error = -1;
    if ( !SDL_NAME(XF86VidModeQueryExtension)(SDL_Display, &vm_event, &vm_error) ||
         !SDL_NAME(XF86VidModeQueryVersion)(SDL_Display, major, minor) ) {
        return 0;
    }
    return 1;
}
#endif /* SDL_VIDEO_DRIVER_X11_VIDMODE */

#if SDL_VIDEO_DRIVER_X11_XME
static int CheckXME(_THIS, int *major, int *minor)
{
    const char *env;

    /* Default the extension not available */
    *major = *minor = 0;

    /* Allow environment override */
    env = getenv("SDL_VIDEO_X11_VIDMODE");
    if ( env && !SDL_atoi(env) ) {
        return 0;
    }
    
    /* Query the extension version */
    if ( !XiGMiscQueryVersion(SDL_Display, major, minor) ) {
        return 0;
    }
    return 1;
}
#endif /* SDL_VIDEO_DRIVER_X11_XME */

int X11_GetVideoModes(_THIS)
{
#if SDL_VIDEO_DRIVER_X11_XINERAMA
    int xinerama_major, xinerama_minor;
#endif
#if SDL_VIDEO_DRIVER_X11_XRANDR
    int xrandr_major, xrandr_minor;
    int nsizes;
    XRRScreenSize *sizes;
#endif
#if SDL_VIDEO_DRIVER_X11_VIDMODE
    int vm_major, vm_minor;
    int nmodes;
    SDL_NAME(XF86VidModeModeInfo) **modes;
#endif
#if SDL_VIDEO_DRIVER_X11_XME
    int xme_major, xme_minor;
    int ractive, nummodes;
    XiGMiscResolutionInfo *modelist;
#endif
    int i, n;
    int screen_w;
    int screen_h;

    use_xinerama = 0;
    use_xrandr = 0;
    use_vidmode = 0;
    use_xme = 0;
    screen_w = DisplayWidth(SDL_Display, SDL_Screen);
    screen_h = DisplayHeight(SDL_Display, SDL_Screen);

#if SDL_VIDEO_DRIVER_X11_XINERAMA
    /* Query Xinerama extention */
    if ( CheckXinerama(this, &xinerama_major, &xinerama_minor) ) {
        /* Find out which screen is the desired one */
        int desired = -1;
        int screens;
        int w, h;
        SDL_NAME(XineramaScreenInfo) *xinerama;

        const char *variable = SDL_getenv("SDL_VIDEO_FULLSCREEN_DISPLAY");
	if ( !variable ) {
        	variable = SDL_getenv("SDL_VIDEO_FULLSCREEN_HEAD");
	}
        if ( variable ) {
                desired = SDL_atoi(variable);
        }
#ifdef X11MODES_DEBUG
        printf("X11 detected Xinerama:\n");
#endif
        xinerama = SDL_NAME(XineramaQueryScreens)(SDL_Display, &screens);
        for ( i = 0; i < screens; i++ ) {
#ifdef X11MODES_DEBUG
            printf("xinerama %d: %dx%d+%d+%d\n",
                xinerama[i].screen_number,
                xinerama[i].width, xinerama[i].height,
                xinerama[i].x_org, xinerama[i].y_org);
#endif
            if ( xinerama[i].screen_number == desired ) {
                use_xinerama = 1;
                xinerama_info = xinerama[i];
            }
        }
        XFree(xinerama);

        if ( use_xinerama ) {
            SDL_modelist = (SDL_Rect **)SDL_malloc(3*sizeof(SDL_Rect *));
            if ( !SDL_modelist ) {
                SDL_OutOfMemory();
                return -1;
            }

            /* Add the full xinerama mode */
            n = 0;
            w = xinerama_info.width;
            h = xinerama_info.height;
            if ( screen_w > w || screen_h > h) {
                SDL_modelist[n] = (SDL_Rect *)SDL_malloc(sizeof(SDL_Rect));
                if ( SDL_modelist[n] ) {
                    SDL_modelist[n]->x = 0;
                    SDL_modelist[n]->y = 0;
                    SDL_modelist[n]->w = screen_w;
                    SDL_modelist[n]->h = screen_h;
                    ++n;
                }
            }

            /* Add the head xinerama mode */
            SDL_modelist[n] = (SDL_Rect *)SDL_malloc(sizeof(SDL_Rect));
            if ( SDL_modelist[n] ) {
                SDL_modelist[n]->x = 0;
                SDL_modelist[n]->y = 0;
                SDL_modelist[n]->w = w;
                SDL_modelist[n]->h = h;
                ++n;
            }
            SDL_modelist[n] = NULL;
        }
    }
#endif /* SDL_VIDEO_DRIVER_X11_XINERAMA */

#if SDL_VIDEO_DRIVER_X11_XRANDR
    /* XRandR */
    /* require at least XRandR v1.0 (arbitrary) */
    if ( CheckXRandR(this, &xrandr_major, &xrandr_minor) && (xrandr_major >= 1) )
    {
#ifdef X11MODES_DEBUG
        fprintf(stderr, "XRANDR: XRRQueryVersion: V%d.%d\n",
                xrandr_major, xrandr_minor);
#endif

        /* save the screen configuration since we must reference it
           each time we toggle modes.
        */
        screen_config = XRRGetScreenInfo(SDL_Display, SDL_Root);

        /* retrieve the list of resolution */
        sizes = XRRConfigSizes(screen_config, &nsizes);
        if (nsizes > 0) {
            if ( SDL_modelist ) {
                for ( i = 0; SDL_modelist[i]; ++i ) {
                    SDL_free(SDL_modelist[i]);
                }
                SDL_free(SDL_modelist);
            }
            SDL_modelist = (SDL_Rect **)malloc((nsizes+1)*sizeof(SDL_Rect *));
            if ( !SDL_modelist ) {
                SDL_OutOfMemory();
                return -1;
            }
            for ( i=0; i < nsizes; i++ ) {
                if ((SDL_modelist[i] =
                     (SDL_Rect *)malloc(sizeof(SDL_Rect))) == NULL)
                    break;
#ifdef X11MODES_DEBUG
                fprintf(stderr, "XRANDR: mode = %4d, w = %4d, h = %4d\n",
                        i, sizes[i].width, sizes[i].height);
#endif

                SDL_modelist[i]->x = 0;
                SDL_modelist[i]->y = 0;
                SDL_modelist[i]->w = sizes[i].width;
                SDL_modelist[i]->h = sizes[i].height;

            }
            /* sort the mode list descending as SDL expects */
            SDL_qsort(SDL_modelist, nsizes, sizeof *SDL_modelist, cmpmodelist);
            SDL_modelist[i] = NULL; /* terminator */

            use_xrandr = xrandr_major * 100 + xrandr_minor;
            saved_size_id = XRRConfigCurrentConfiguration(screen_config, &saved_rotation);
        }
    }
#endif /* SDL_VIDEO_DRIVER_X11_XRANDR */

#if SDL_VIDEO_DRIVER_X11_VIDMODE
    /* XVidMode */
    if ( !use_xrandr &&
#if SDL_VIDEO_DRIVER_X11_XINERAMA
         (!use_xinerama || xinerama_info.screen_number == -1) &&
#endif
         CheckVidMode(this, &vm_major, &vm_minor) &&
         SDL_NAME(XF86VidModeGetAllModeLines)(SDL_Display, SDL_Screen,&nmodes,&modes) )
    {
#ifdef X11MODES_DEBUG
        printf("VidMode modes: (unsorted)\n");
        for ( i = 0; i < nmodes; ++i ) {
            printf("Mode %d: %d x %d @ %d\n", i,
                    modes[i]->hdisplay, modes[i]->vdisplay,
                    (modes[i]->htotal && modes[i]->vtotal) ? (1000 * modes[i]->dotclock / (modes[i]->htotal * modes[i]->vtotal)) : 0 );
        }
#endif
        if ( SDL_modelist ) {
            for ( i = 0; SDL_modelist[i]; ++i ) {
                SDL_free(SDL_modelist[i]);
            }
            SDL_free(SDL_modelist);
        }
        SDL_modelist = (SDL_Rect **)SDL_malloc((nmodes+2)*sizeof(SDL_Rect *));
        if ( !SDL_modelist ) {
            SDL_OutOfMemory();
            return -1;
        }
        SDL_qsort(modes, nmodes, sizeof *modes, cmpmodes);
        n = 0;
        for ( i=0; i<nmodes; ++i ) {
            int w, h;

            /* Eliminate duplicate modes with different refresh rates */
            if ( i > 0 &&
                 modes[i]->hdisplay == modes[i-1]->hdisplay &&
                 modes[i]->vdisplay == modes[i-1]->vdisplay ) {
                    continue;
            }

            /* Check to see if we should add the screen size (Xinerama) */
            w = modes[i]->hdisplay;
            h = modes[i]->vdisplay;
            if ( (screen_w * screen_h) >= (w * h) ) {
                if ( (screen_w != w) || (screen_h != h) ) {
                    SDL_modelist[n] = (SDL_Rect *)SDL_malloc(sizeof(SDL_Rect));
                    if ( SDL_modelist[n] ) {
                        SDL_modelist[n]->x = 0;
                        SDL_modelist[n]->y = 0;
                        SDL_modelist[n]->w = screen_w;
                        SDL_modelist[n]->h = screen_h;
                        ++n;
                    }
                }
                screen_w = 0;
                screen_h = 0;
            }

            /* Add the size from the video mode list */
            SDL_modelist[n] = (SDL_Rect *)SDL_malloc(sizeof(SDL_Rect));
            if ( SDL_modelist[n] == NULL ) {
                break;
            }
            SDL_modelist[n]->x = 0;
            SDL_modelist[n]->y = 0;
            SDL_modelist[n]->w = w;
            SDL_modelist[n]->h = h;
            ++n;
        }
        SDL_modelist[n] = NULL;
        XFree(modes);

        use_vidmode = vm_major * 100 + vm_minor;
        save_mode(this);
    }
#endif /* SDL_VIDEO_DRIVER_X11_VIDMODE */

#if SDL_VIDEO_DRIVER_X11_XME
    /* XiG */
    modelist = NULL;
    /* first lets make sure we have the extension, and it's at least v2.0 */
    if ( CheckXME(this, &xme_major, &xme_minor) && xme_major >= 2 &&
         (nummodes = XiGMiscQueryResolutions(SDL_Display, SDL_Screen,
                                             0, /* view */
                                             &ractive, &modelist)) > 1 )
    {                                /* then we actually have some */
        int j;

        /* We get the list already sorted in descending order.
           We'll copy it in reverse order so SDL is happy */
#ifdef X11MODES_DEBUG
        fprintf(stderr, "XME: nummodes = %d, active mode = %d\n",
                nummodes, ractive);
#endif
        if ( SDL_modelist ) {
            for ( i = 0; SDL_modelist[i]; ++i ) {
                SDL_free(SDL_modelist[i]);
            }
            SDL_free(SDL_modelist);
        }
        SDL_modelist = (SDL_Rect **)SDL_malloc((nummodes+1)*sizeof(SDL_Rect *));
        if ( !SDL_modelist ) {
            SDL_OutOfMemory();
            return -1;
        }
        for ( i=0, j=nummodes-1; j>=0; i++, j-- ) {
            if ((SDL_modelist[i] = 
                 (SDL_Rect *)SDL_malloc(sizeof(SDL_Rect))) == NULL)
              break;
#ifdef X11MODES_DEBUG
            fprintf(stderr, "XME: mode = %4d, w = %4d, h = %4d\n",
                   i, modelist[i].width, modelist[i].height);
#endif
            
            SDL_modelist[i]->x = 0;
            SDL_modelist[i]->y = 0;
            SDL_modelist[i]->w = modelist[j].width;
            SDL_modelist[i]->h = modelist[j].height;
            
        }
        SDL_modelist[i] = NULL; /* terminator */

        use_xme = xme_major * 100 + xme_minor;
        saved_res = modelist[ractive]; /* save the current resolution */
    }
    if ( modelist ) {
        XFree(modelist);
    }
#endif /* SDL_VIDEO_DRIVER_X11_XME */

    {
	/* It's interesting to note that if we allow 32 bit depths,
	   we get a visual with an alpha mask on composite servers.
        static int depth_list[] = { 32, 24, 16, 15, 8 };
	*/
        static int depth_list[] = { 24, 16, 15, 8 };
        int j, np;
        int use_directcolor = 1;
        XPixmapFormatValues *pf;

        /* Search for the visuals in deepest-first order, so that the first
           will be the richest one */
        if ( SDL_getenv("SDL_VIDEO_X11_NODIRECTCOLOR") ) {
                use_directcolor = 0;
        }
        this->hidden->nvisuals = 0;
        if ( ! add_visual_byid(this, SDL_getenv("SDL_VIDEO_X11_VISUALID")) ) {
                for ( i=0; i<SDL_arraysize(depth_list); ++i ) {
                        if ( depth_list[i] > 8 ) {
                                if ( use_directcolor ) {
                                        add_visual(this, depth_list[i], DirectColor);
                                }
                                add_visual(this, depth_list[i], TrueColor);
                        } else {
                                add_visual(this, depth_list[i], PseudoColor);
                                add_visual(this, depth_list[i], StaticColor);
                        }
                }
                add_default_visual(this);
        }
        if ( this->hidden->nvisuals == 0 ) {
            SDL_SetError("Found no sufficiently capable X11 visuals");
            return -1;
        }
            
        /* look up the pixel quantum for each depth */
        pf = XListPixmapFormats(SDL_Display, &np);
        for(i = 0; i < this->hidden->nvisuals; i++) {
            int d = this->hidden->visuals[i].depth;
            for(j = 0; j < np; j++)
                if(pf[j].depth == d)
                    break;
            this->hidden->visuals[i].bpp = j < np ? pf[j].bits_per_pixel : d;
        }

        XFree(pf);
    }

    if ( SDL_modelist == NULL ) {
        SDL_modelist = (SDL_Rect **)SDL_malloc((1+1)*sizeof(SDL_Rect *));
        if ( !SDL_modelist ) {
            SDL_OutOfMemory();
            return -1;
        }
        n = 0;
        SDL_modelist[n] = (SDL_Rect *)SDL_malloc(sizeof(SDL_Rect));
        if ( SDL_modelist[n] ) {
            SDL_modelist[n]->x = 0;
            SDL_modelist[n]->y = 0;
            SDL_modelist[n]->w = screen_w;
            SDL_modelist[n]->h = screen_h;
            ++n;
        }
        SDL_modelist[n] = NULL;
    }

#ifdef X11MODES_DEBUG
    if ( use_xinerama ) {
        printf("Xinerama is enabled\n");
    }

    if ( use_xrandr ) {
        printf("XRandR is enabled\n");
    }

    if ( use_vidmode ) {
        printf("VidMode is enabled\n");
    }

    if ( use_xme ) {
        printf("Xi Graphics XME fullscreen is enabled\n");
    }

    if ( SDL_modelist ) {
        printf("X11 video mode list:\n");
        for ( i=0; SDL_modelist[i]; ++i ) {
            printf("\t%dx%d\n", SDL_modelist[i]->w, SDL_modelist[i]->h);
        }
    }
#endif /* X11MODES_DEBUG */

    return 0;
}

int X11_SupportedVisual(_THIS, SDL_PixelFormat *format)
{
    int i;
    for(i = 0; i < this->hidden->nvisuals; i++)
        if(this->hidden->visuals[i].bpp == format->BitsPerPixel)
            return 1;
    return 0;
}

SDL_Rect **X11_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags)
{
    if ( X11_SupportedVisual(this, format) ) {
        if ( flags & SDL_FULLSCREEN ) {
            return(SDL_modelist);
        } else {
            return((SDL_Rect **)-1);
        }
    } else {
        return((SDL_Rect **)0);
    }
}

void X11_FreeVideoModes(_THIS)
{
    int i;

    if ( SDL_modelist ) {
        for ( i=0; SDL_modelist[i]; ++i ) {
            SDL_free(SDL_modelist[i]);
        }
        SDL_free(SDL_modelist);
        SDL_modelist = NULL;
    }

#if SDL_VIDEO_DRIVER_X11_XRANDR
    /* Free the Xrandr screen configuration */
    if ( screen_config ) {
        XRRFreeScreenConfigInfo(screen_config);
        screen_config = NULL;
    }
#endif /* SDL_VIDEO_DRIVER_X11_XRANDR */
}

int X11_ResizeFullScreen(_THIS)
{
    int x = 0, y = 0;
    int real_w, real_h;
    int screen_w;
    int screen_h;

    screen_w = DisplayWidth(SDL_Display, SDL_Screen);
    screen_h = DisplayHeight(SDL_Display, SDL_Screen);

#if SDL_VIDEO_DRIVER_X11_XINERAMA
    if ( use_xinerama &&
         window_w <= xinerama_info.width &&
         window_h <= xinerama_info.height ) {
        x = xinerama_info.x_org;
        y = xinerama_info.y_org;
    }
#endif /* SDL_VIDEO_DRIVER_X11_XINERAMA */

    if ( currently_fullscreen ) {
        /* Switch resolution and cover it with the FSwindow */
        move_cursor_to(this, x, y);
        set_best_resolution(this, window_w, window_h);
        move_cursor_to(this, x, y);
        get_real_resolution(this, &real_w, &real_h);
        if ( window_w > real_w ) {
            real_w = MAX(real_w, screen_w);
        }
        if ( window_h > real_h ) {
            real_h = MAX(real_h, screen_h);
        }
        XMoveResizeWindow(SDL_Display, FSwindow, x, y, real_w, real_h);
        move_cursor_to(this, real_w/2, real_h/2);

        /* Center and reparent the drawing window */
        x = (real_w - window_w)/2;
        y = (real_h - window_h)/2;
        XReparentWindow(SDL_Display, SDL_Window, FSwindow, x, y);
        /* FIXME: move the mouse to the old relative location */
        XSync(SDL_Display, True);   /* Flush spurious mode change events */
    }
    return(1);
}

void X11_QueueEnterFullScreen(_THIS)
{
    switch_waiting = 0x01 | SDL_FULLSCREEN;
    switch_time = SDL_GetTicks() + 1500;
#if 0 /* This causes a BadMatch error if the window is iconified (not needed) */
    XSetInputFocus(SDL_Display, WMwindow, RevertToNone, CurrentTime);
#endif
}

int X11_EnterFullScreen(_THIS)
{
    int okay;
#if 0
    Window tmpwin, *windows;
    int i, nwindows;
#endif
    int x = 0, y = 0;
    int real_w, real_h;
    int screen_w;
    int screen_h;

    okay = 1;
    if ( currently_fullscreen ) {
        return(okay);
    }

    /* Ungrab the input so that we can move the mouse around */
    X11_GrabInputNoLock(this, SDL_GRAB_OFF);

#if SDL_VIDEO_DRIVER_X11_XINERAMA
    if ( use_xinerama &&
         window_w <= xinerama_info.width &&
         window_h <= xinerama_info.height ) {
        x = xinerama_info.x_org;
        y = xinerama_info.y_org;
    }
#endif /* SDL_VIDEO_DRIVER_X11_XINERAMA */

    /* Map the fullscreen window to blank the screen */
    screen_w = DisplayWidth(SDL_Display, SDL_Screen);
    screen_h = DisplayHeight(SDL_Display, SDL_Screen);
    get_real_resolution(this, &real_w, &real_h);
    real_w = MAX(window_w, MAX(real_w, screen_w));
    real_h = MAX(window_h, MAX(real_h, screen_h));
    XMoveResizeWindow(SDL_Display, FSwindow,
                      x, y, real_w, real_h);
    XMapRaised(SDL_Display, FSwindow);
    X11_WaitMapped(this, FSwindow);

#if 0 /* This seems to break WindowMaker in focus-follows-mouse mode */
    /* Make sure we got to the top of the window stack */
    if ( XQueryTree(SDL_Display, SDL_Root, &tmpwin, &tmpwin,
                            &windows, &nwindows) && windows ) {
        /* If not, try to put us there - if fail... oh well */
        if ( windows[nwindows-1] != FSwindow ) {
            tmpwin = windows[nwindows-1];
            for ( i=0; i<nwindows; ++i ) {
                if ( windows[i] == FSwindow ) {
                    SDL_memcpy(&windows[i], &windows[i+1],
                           (nwindows-i-1)*sizeof(windows[i]));
                    break;
                }
            }
            windows[nwindows-1] = FSwindow;
            XRestackWindows(SDL_Display, windows, nwindows);
            XSync(SDL_Display, False);
        }
        XFree(windows);
    }
#else
    XRaiseWindow(SDL_Display, FSwindow);
#endif

#if SDL_VIDEO_DRIVER_X11_VIDMODE
    /* Save the current video mode */
    if ( use_vidmode ) {
        SDL_NAME(XF86VidModeLockModeSwitch)(SDL_Display, SDL_Screen, True);
        save_mode(this);
    }
#endif
    currently_fullscreen = 1;

    /* Set the new resolution */
    okay = X11_ResizeFullScreen(this);
    if ( ! okay ) {
        X11_LeaveFullScreen(this);
    }
    /* Set the colormap */
    if ( SDL_XColorMap ) {
        XInstallColormap(SDL_Display, SDL_XColorMap);
    }
    if ( okay ) {
        X11_GrabInputNoLock(this, this->input_grab | SDL_GRAB_FULLSCREEN);
    }

    /* We may need to refresh the screen at this point (no backing store)
       We also don't get an event, which is why we explicitly refresh. */
    if ( this->screen ) {
        if ( this->screen->flags & SDL_OPENGL ) {
            SDL_PrivateExpose();
        } else {
            X11_RefreshDisplay(this);
        }
    }

    return(okay);
}

int X11_LeaveFullScreen(_THIS)
{
    if ( currently_fullscreen ) {
        XReparentWindow(SDL_Display, SDL_Window, WMwindow, 0, 0);
#if SDL_VIDEO_DRIVER_X11_VIDMODE
        if ( use_vidmode ) {
            restore_mode(this);
            SDL_NAME(XF86VidModeLockModeSwitch)(SDL_Display, SDL_Screen, False);
        }
#endif

#if SDL_VIDEO_DRIVER_X11_XME
        if ( use_xme ) {
            int rw, rh;        
            
            /* check current mode so we can avoid uneccessary mode changes */
            get_real_resolution(this, &rw, &rh);

            if (rw != saved_res.width || rh != saved_res.height) {
                XiGMiscChangeResolution(SDL_Display, 
                                        SDL_Screen,
                                        0, /* view */
                                        saved_res.width, 
                                        saved_res.height,
                                        0);
                XSync(SDL_Display, False);
            }
        }
#endif

#if SDL_VIDEO_DRIVER_X11_XRANDR
        if ( use_xrandr ) {
            XRRSetScreenConfig(SDL_Display, screen_config, SDL_Root,
                               saved_size_id, saved_rotation, CurrentTime);
        }
#endif

        XUnmapWindow(SDL_Display, FSwindow);
        X11_WaitUnmapped(this, FSwindow);
        XSync(SDL_Display, True);   /* Flush spurious mode change events */
        currently_fullscreen = 0;
    }
    /* If we get popped out of fullscreen mode for some reason, input_grab
       will still have the SDL_GRAB_FULLSCREEN flag set, since this is only
       temporary.  In this case, release the grab unless the input has been
       explicitly grabbed.
     */
    X11_GrabInputNoLock(this, this->input_grab & ~SDL_GRAB_FULLSCREEN);

    /* We may need to refresh the screen at this point (no backing store)
       We also don't get an event, which is why we explicitly refresh. */
    if ( this->screen ) {
        if ( this->screen->flags & SDL_OPENGL ) {
            SDL_PrivateExpose();
        } else {
            X11_RefreshDisplay(this);
        }
    }

    return(0);
}
