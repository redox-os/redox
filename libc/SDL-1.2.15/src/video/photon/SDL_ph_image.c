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

#include <Ph.h>
#include <photon/Pg.h>

#include "SDL_endian.h"
#include "SDL_video.h"
#include "../SDL_pixels_c.h"
#include "SDL_ph_video.h"
#include "SDL_ph_image_c.h"
#include "SDL_ph_modes_c.h"
#include "SDL_ph_gl.h"

int ph_SetupImage(_THIS, SDL_Surface *screen)
{
    PgColor_t* palette=NULL;
    int type=0;
    int bpp;
    
    bpp=screen->format->BitsPerPixel;

    /* Determine image type */
    switch(bpp)
    {
        case 8:{
            type = Pg_IMAGE_PALETTE_BYTE;
        }
        break;
        case 15:{
            type = Pg_IMAGE_DIRECT_555; 
        }
        break;
        case 16:{
            type = Pg_IMAGE_DIRECT_565; 
        }
        break;
        case 24:{
            type = Pg_IMAGE_DIRECT_888;
        }
        break;
        case 32:{
            type = Pg_IMAGE_DIRECT_8888;
        }
        break;
        default:{
            SDL_SetError("ph_SetupImage(): unsupported bpp=%d !\n", bpp);
            return -1;
        }
        break;
    }

    /* palette emulation code */
    if ((bpp==8) && (desktoppal==SDLPH_PAL_EMULATE))
    {
        /* creating image palette */
        palette=SDL_malloc(_Pg_MAX_PALETTE*sizeof(PgColor_t));
        if (palette==NULL)
        {
            SDL_SetError("ph_SetupImage(): can't allocate memory for palette !\n");
            return -1;
        }
        PgGetPalette(palette);

        /* using shared memory for speed (set last param to 1) */
        if ((SDL_Image = PhCreateImage(NULL, screen->w, screen->h, type, palette, _Pg_MAX_PALETTE, 1)) == NULL)
        {
            SDL_SetError("ph_SetupImage(): PhCreateImage() failed for bpp=8 !\n");
            SDL_free(palette);
            return -1;
        }
    }
    else
    {
        /* using shared memory for speed (set last param to 1) */
        if ((SDL_Image = PhCreateImage(NULL, screen->w, screen->h, type, NULL, 0, 1)) == NULL)
        {
            SDL_SetError("ph_SetupImage(): PhCreateImage() failed for bpp=%d !\n", bpp);
            return -1;
        }
    }

    screen->pixels = SDL_Image->image;
    screen->pitch = SDL_Image->bpl;

    this->UpdateRects = ph_NormalUpdate;

    return 0;
}

int ph_SetupOCImage(_THIS, SDL_Surface *screen)
{
    int type = 0;
    int bpp;

    OCImage.flags = screen->flags;
    
    bpp=screen->format->BitsPerPixel;

    /* Determine image type */
    switch(bpp)
    {
        case 8: {
                    type = Pg_IMAGE_PALETTE_BYTE;
                }
                break;
        case 15:{
                    type = Pg_IMAGE_DIRECT_555; 
		}
		break;
        case 16:{
                    type = Pg_IMAGE_DIRECT_565; 
                }
                break;
        case 24:{
                    type = Pg_IMAGE_DIRECT_888;
                }
                break;
        case 32:{
                    type = Pg_IMAGE_DIRECT_8888;
                }
                break;
        default:{
                    SDL_SetError("ph_SetupOCImage(): unsupported bpp=%d !\n", bpp);
                    return -1;
                }
                break;
    }

    /* Currently offscreen contexts with the same bit depth as display bpp only can be created */
    OCImage.offscreen_context = PdCreateOffscreenContext(0, screen->w, screen->h, Pg_OSC_MEM_PAGE_ALIGN);

    if (OCImage.offscreen_context == NULL)
    {
        SDL_SetError("ph_SetupOCImage(): PdCreateOffscreenContext() function failed !\n");
        return -1;
    }

    screen->pitch = OCImage.offscreen_context->pitch;

    OCImage.dc_ptr = (unsigned char *)PdGetOffscreenContextPtr(OCImage.offscreen_context);

    if (OCImage.dc_ptr == NULL)
    {
        SDL_SetError("ph_SetupOCImage(): PdGetOffscreenContextPtr function failed !\n");
        PhDCRelease(OCImage.offscreen_context);
        return -1;
    }

    OCImage.FrameData0 = OCImage.dc_ptr;
    OCImage.CurrentFrameData = OCImage.FrameData0;
    OCImage.current = 0;

    PhDCSetCurrent(OCImage.offscreen_context);

    screen->pixels = OCImage.CurrentFrameData;

    this->UpdateRects = ph_OCUpdate;

    return 0;
}

int ph_SetupFullScreenImage(_THIS, SDL_Surface* screen)
{
    OCImage.flags = screen->flags;

    /* Begin direct and fullscreen mode */
    if (!ph_EnterFullScreen(this, screen, PH_ENTER_DIRECTMODE))
    {
        return -1;
    }

    /* store palette for fullscreen */
    if ((screen->format->BitsPerPixel==8) && (desktopbpp!=8))
    {
        PgGetPalette(savedpal);
        PgGetPalette(syspalph);
    }

    OCImage.offscreen_context = PdCreateOffscreenContext(0, 0, 0, Pg_OSC_MAIN_DISPLAY | Pg_OSC_MEM_PAGE_ALIGN | Pg_OSC_CRTC_SAFE);
    if (OCImage.offscreen_context == NULL)
    {
        SDL_SetError("ph_SetupFullScreenImage(): PdCreateOffscreenContext() function failed !\n");
        return -1;
    }
    
    if ((screen->flags & SDL_DOUBLEBUF) == SDL_DOUBLEBUF)
    {
        OCImage.offscreen_backcontext = PdDupOffscreenContext(OCImage.offscreen_context, Pg_OSC_MEM_PAGE_ALIGN | Pg_OSC_CRTC_SAFE);
        if (OCImage.offscreen_backcontext == NULL)
        {
            SDL_SetError("ph_SetupFullScreenImage(): PdCreateOffscreenContext(back) function failed !\n");
            return -1;
        }
    }

    OCImage.FrameData0 = (unsigned char *)PdGetOffscreenContextPtr(OCImage.offscreen_context);
    if (OCImage.FrameData0 == NULL)
    {
        SDL_SetError("ph_SetupFullScreenImage(): PdGetOffscreenContextPtr() function failed !\n");
        ph_DestroyImage(this, screen);
        return -1;
    }

    if ((screen->flags & SDL_DOUBLEBUF) == SDL_DOUBLEBUF)
    {
        OCImage.FrameData1 = (unsigned char *)PdGetOffscreenContextPtr(OCImage.offscreen_backcontext);
        if (OCImage.FrameData1 == NULL)
        {
            SDL_SetError("ph_SetupFullScreenImage(back): PdGetOffscreenContextPtr() function failed !\n");
            ph_DestroyImage(this, screen);
            return -1;
        }
    }

    /* wait for the hardware */
    PgFlush();
    PgWaitHWIdle();

    if ((screen->flags & SDL_DOUBLEBUF) == SDL_DOUBLEBUF)
    {
        OCImage.current = 0;
        PhDCSetCurrent(OCImage.offscreen_context);
        screen->pitch = OCImage.offscreen_context->pitch;
        screen->pixels = OCImage.FrameData0;
        
        /* emulate 640x400 videomode */
        if (videomode_emulatemode==1)
        {
           int i;
           
           for (i=0; i<40; i++)
           {
              SDL_memset(screen->pixels+screen->pitch*i, 0x00, screen->pitch);
           }
           for (i=440; i<480; i++)
           {
              SDL_memset(screen->pixels+screen->pitch*i, 0x00, screen->pitch);
           }
           screen->pixels+=screen->pitch*40;
        }
        PgSwapDisplay(OCImage.offscreen_backcontext, 0);
    }
    else
    {
        OCImage.current = 0;
        PhDCSetCurrent(OCImage.offscreen_context);
        screen->pitch = OCImage.offscreen_context->pitch;
        screen->pixels = OCImage.FrameData0;

        /* emulate 640x400 videomode */
        if (videomode_emulatemode==1)
        {
           int i;
           
           for (i=0; i<40; i++)
           {
              SDL_memset(screen->pixels+screen->pitch*i, 0x00, screen->pitch);
           }
           for (i=440; i<480; i++)
           {
              SDL_memset(screen->pixels+screen->pitch*i, 0x00, screen->pitch);
           }
           screen->pixels+=screen->pitch*40;
        }
    }

    this->UpdateRects = ph_OCDCUpdate;

    /* wait for the hardware */
    PgFlush();
    PgWaitHWIdle();

    return 0;
}

#if SDL_VIDEO_OPENGL

int ph_SetupOpenGLImage(_THIS, SDL_Surface* screen)
{
    this->UpdateRects = ph_OpenGLUpdate;
    screen->pixels=NULL;
    screen->pitch=NULL;

    #if (_NTO_VERSION >= 630)
        if ((screen->flags & SDL_FULLSCREEN) == SDL_FULLSCREEN)
        {
            if (!ph_EnterFullScreen(this, screen, PH_IGNORE_DIRECTMODE))
            {
                screen->flags &= ~SDL_FULLSCREEN;
                return -1;
            }
        }
    #endif /* 6.3.0 */

    if (ph_SetupOpenGLContext(this, screen->w, screen->h, screen->format->BitsPerPixel, screen->flags)!=0)
    {
        screen->flags &= ~SDL_OPENGL;
        return -1;
    }
   
    return 0;
}

#endif /* SDL_VIDEO_OPENGL */

void ph_DestroyImage(_THIS, SDL_Surface* screen)
{

#if SDL_VIDEO_OPENGL
    if ((screen->flags & SDL_OPENGL)==SDL_OPENGL)
    {
        if (oglctx)
        {
            #if (_NTO_VERSION < 630)
                PhDCSetCurrent(NULL);
                PhDCRelease(oglctx);
            #else
                qnxgl_context_destroy(oglctx);
                qnxgl_buffers_destroy(oglbuffers);
                qnxgl_finish();
            #endif /* 6.3.0 */
            oglctx=NULL;
            oglbuffers=NULL;
            oglflags=0;
            oglbpp=0;
        }

        #if (_NTO_VERSION >= 630)
            if (currently_fullscreen)
            {
                ph_LeaveFullScreen(this);
            }
        #endif /* 6.3.0 */

        return;
    }
#endif /* SDL_VIDEO_OPENGL */

    if (currently_fullscreen)
    {
        /* if we right now in 8bpp fullscreen we must release palette */
        if ((screen->format->BitsPerPixel==8) && (desktopbpp!=8))
        {
            PgSetPalette(syspalph, 0, -1, 0, 0, 0);
            PgSetPalette(savedpal, 0, 0, _Pg_MAX_PALETTE, Pg_PALSET_GLOBAL | Pg_PALSET_FORCE_EXPOSE, 0);
            PgFlush();
        }
        ph_LeaveFullScreen(this);
    }

    if (OCImage.offscreen_context != NULL)
    {
        PhDCRelease(OCImage.offscreen_context);
        OCImage.offscreen_context = NULL;
        OCImage.FrameData0 = NULL;
    }
    if (OCImage.offscreen_backcontext != NULL)
    {
        PhDCRelease(OCImage.offscreen_backcontext);
        OCImage.offscreen_backcontext = NULL;
        OCImage.FrameData1 = NULL;
    }
    OCImage.CurrentFrameData = NULL;

    if (SDL_Image)
    {
        /* if palette allocated, free it */
        if (SDL_Image->palette)
        {
            SDL_free(SDL_Image->palette);
        }
        PgShmemDestroy(SDL_Image->image);
        SDL_free(SDL_Image);
    }

    /* Must be zeroed everytime */
    SDL_Image = NULL;

    if (screen)
    {
        screen->pixels = NULL;
    }
}

int ph_UpdateHWInfo(_THIS)
{
    PgVideoModeInfo_t vmode;
    PgHWCaps_t hwcaps;

    /* Update video ram amount */
    if (PgGetGraphicsHWCaps(&hwcaps) < 0)
    {
        SDL_SetError("ph_UpdateHWInfo(): GetGraphicsHWCaps() function failed !\n");
        return -1;
    }
    this->info.video_mem=hwcaps.currently_available_video_ram/1024;

    /* obtain current mode capabilities */
    if (PgGetVideoModeInfo(hwcaps.current_video_mode, &vmode) < 0)
    {
        SDL_SetError("ph_UpdateHWInfo(): GetVideoModeInfo() function failed !\n");
        return -1;
    }

    if ((vmode.mode_capabilities1 & PgVM_MODE_CAP1_OFFSCREEN) == PgVM_MODE_CAP1_OFFSCREEN)
    {
        /* this is a special test for drivers which tries to lie about offscreen capability */
        if (hwcaps.currently_available_video_ram!=0)
        {
           this->info.hw_available = 1;
        }
        else
        {
           this->info.hw_available = 0;
        }
    }
    else
    {
        this->info.hw_available = 0;
    }

    if ((vmode.mode_capabilities2 & PgVM_MODE_CAP2_RECTANGLE) == PgVM_MODE_CAP2_RECTANGLE)
    {
        this->info.blit_fill = 1;
    }
    else
    {
        this->info.blit_fill = 0;
    }

    if ((vmode.mode_capabilities2 & PgVM_MODE_CAP2_BITBLT) == PgVM_MODE_CAP2_BITBLT)
    {
        this->info.blit_hw = 1;
    }
    else
    {
        this->info.blit_hw = 0;
    }

    if ((vmode.mode_capabilities2 & PgVM_MODE_CAP2_ALPHA_BLEND) == PgVM_MODE_CAP2_ALPHA_BLEND)
    {
        this->info.blit_hw_A = 1;
    }
    else
    {
        this->info.blit_hw_A = 0;
    }
    
    if ((vmode.mode_capabilities2 & PgVM_MODE_CAP2_CHROMA) == PgVM_MODE_CAP2_CHROMA)
    {
        this->info.blit_hw_CC = 1;
    }
    else
    {
        this->info.blit_hw_CC = 0;
    }
    
    return 0;
}

int ph_SetupUpdateFunction(_THIS, SDL_Surface* screen, Uint32 flags)
{
    int setupresult=-1;

    ph_DestroyImage(this, screen);
    
#if SDL_VIDEO_OPENGL
    if ((flags & SDL_OPENGL)==SDL_OPENGL)
    {
        setupresult=ph_SetupOpenGLImage(this, screen);
    }
    else
    {
#endif
       if ((flags & SDL_FULLSCREEN)==SDL_FULLSCREEN)
       {
           setupresult=ph_SetupFullScreenImage(this, screen);
       }
       else
       {
          if ((flags & SDL_HWSURFACE)==SDL_HWSURFACE)
          {
              setupresult=ph_SetupOCImage(this, screen);
          }
          else
          {
              setupresult=ph_SetupImage(this, screen);
          }
       }
#if SDL_VIDEO_OPENGL
    }
#endif
    if (setupresult!=-1)
    {
       ph_UpdateHWInfo(this);
    }
    
    return setupresult;
}

int ph_AllocHWSurface(_THIS, SDL_Surface* surface)
{
    PgHWCaps_t hwcaps;

    if (surface->hwdata!=NULL)
    {
       SDL_SetError("ph_AllocHWSurface(): hwdata already exists!\n");
       return -1;
    }
    surface->hwdata=SDL_malloc(sizeof(struct private_hwdata));
    SDL_memset(surface->hwdata, 0x00, sizeof(struct private_hwdata));
    surface->hwdata->offscreenctx=PdCreateOffscreenContext(0, surface->w, surface->h, Pg_OSC_MEM_PAGE_ALIGN);
    if (surface->hwdata->offscreenctx == NULL)
    {
        SDL_SetError("ph_AllocHWSurface(): PdCreateOffscreenContext() function failed !\n");
        return -1;
    }
    surface->pixels=PdGetOffscreenContextPtr(surface->hwdata->offscreenctx);
    if (surface->pixels==NULL)
    {
        PhDCRelease(surface->hwdata->offscreenctx);
        SDL_SetError("ph_AllocHWSurface(): PdGetOffscreenContextPtr() function failed !\n");
        return -1;
    }
    surface->pitch=surface->hwdata->offscreenctx->pitch;
    surface->flags|=SDL_HWSURFACE;
    surface->flags|=SDL_PREALLOC;
    
#if 0 /* FIXME */
    /* create simple offscreen lock */
    surface->hwdata->crlockparam.flags=0;
    if (PdCreateOffscreenLock(surface->hwdata->offscreenctx, &surface->hwdata->crlockparam)!=EOK)
    {
        PhDCRelease(surface->hwdata->offscreenctx);
        SDL_SetError("ph_AllocHWSurface(): Can't create offscreen lock !\n");
        return -1;
    }
#endif /* 0 */

    /* Update video ram amount */
    if (PgGetGraphicsHWCaps(&hwcaps) < 0)
    {
        PdDestroyOffscreenLock(surface->hwdata->offscreenctx);
        PhDCRelease(surface->hwdata->offscreenctx);
        SDL_SetError("ph_AllocHWSurface(): GetGraphicsHWCaps() function failed !\n");
        return -1;
    }
    this->info.video_mem=hwcaps.currently_available_video_ram/1024;

    return 0;
}

void ph_FreeHWSurface(_THIS, SDL_Surface* surface)
{
    PgHWCaps_t hwcaps;

    if (surface->hwdata==NULL)
    {
       SDL_SetError("ph_FreeHWSurface(): no hwdata!\n");
       return;
    }
    if (surface->hwdata->offscreenctx == NULL)
    {
       SDL_SetError("ph_FreeHWSurface(): no offscreen context to delete!\n");
       return;
    }

#if 0 /* FIXME */
    /* unlock the offscreen context if it has been locked before destroy it */
    if (PdIsOffscreenLocked(surface->hwdata->offscreenctx)==Pg_OSC_LOCKED)
    {
       PdUnlockOffscreen(surface->hwdata->offscreenctx);
    }

    PdDestroyOffscreenLock(surface->hwdata->offscreenctx);
#endif /* 0 */

    PhDCRelease(surface->hwdata->offscreenctx);
    
    SDL_free(surface->hwdata);
    surface->hwdata=NULL;

    /* Update video ram amount */
    if (PgGetGraphicsHWCaps(&hwcaps) < 0)
    {
        SDL_SetError("ph_FreeHWSurface(): GetGraphicsHWCaps() function failed !\n");
        return;
    }
    this->info.video_mem=hwcaps.currently_available_video_ram/1024;

    return;
}

int ph_CheckHWBlit(_THIS, SDL_Surface *src, SDL_Surface *dst)
{
   if ((src->hwdata==NULL) && (src != this->screen))
   {
      SDL_SetError("ph_CheckHWBlit(): Source surface haven't hardware specific data.\n");
      src->flags&=~SDL_HWACCEL;
      return -1;
   }
   if ((src->flags & SDL_HWSURFACE) != SDL_HWSURFACE)
   {
      SDL_SetError("ph_CheckHWBlit(): Source surface isn't a hardware surface.\n");
      src->flags&=~SDL_HWACCEL;
      return -1;
   }

   if ((src->flags & SDL_SRCCOLORKEY) == SDL_SRCCOLORKEY)
   {
       if (this->info.blit_hw_CC!=1)
       {
           src->flags&=~SDL_HWACCEL;
           src->map->hw_blit=NULL;
           return -1;
       }
   }

   if ((src->flags & SDL_SRCALPHA) == SDL_SRCALPHA)
   {
       if (this->info.blit_hw_A!=1)
       {
           src->flags&=~SDL_HWACCEL;
           src->map->hw_blit=NULL;
           return -1;
       }
   }

   src->flags|=SDL_HWACCEL;
   src->map->hw_blit = ph_HWAccelBlit;

   return 1;
}

PgColor_t ph_ExpandColor(_THIS, SDL_Surface* surface, Uint32 color)
{
    Uint32 truecolor;

    /* Photon API accepts true colors only during hw filling operations */
    switch(surface->format->BitsPerPixel)
    {
       case 8:
            {
                if ((surface->format->palette) && (color<=surface->format->palette->ncolors))
                {
                    truecolor=PgRGB(surface->format->palette->colors[color].r,
                                    surface->format->palette->colors[color].g,
                                    surface->format->palette->colors[color].b);
                }
                else
                {
                    SDL_SetError("ph_ExpandColor(): Color out of range for the 8bpp mode !\n");
                    return 0xFFFFFFFFUL;
                }
            }
            break;
       case 15: 
            {
                truecolor = ((color & 0x00007C00UL) << 9) |   /* R */
                            ((color & 0x000003E0UL) << 6) |   /* G */
                            ((color & 0x0000001FUL) << 3) |   /* B */
                            ((color & 0x00007000UL) << 4) |   /* R compensation */
                            ((color & 0x00000380UL) << 1) |   /* G compensation */
                            ((color & 0x0000001CUL) >> 2);    /* B compensation */
            }
            break;
       case 16: 
            {
                truecolor = ((color & 0x0000F800UL) << 8) |   /* R */
                            ((color & 0x000007E0UL) << 5) |   /* G */
                            ((color & 0x0000001FUL) << 3) |   /* B */
                            ((color & 0x0000E000UL) << 3) |   /* R compensation */
                            ((color & 0x00000600UL) >> 1) |   /* G compensation */
                            ((color & 0x0000001CUL) >> 2);    /* B compensation */

            }
            break;
       case 24: 
            {
                truecolor=color & 0x00FFFFFFUL;
            }
            break;
       case 32: 
            {
                truecolor=color;
            }
            break;
       default:
            {
                SDL_SetError("ph_ExpandColor(): Unsupported depth for the hardware operations !\n");
                return 0xFFFFFFFFUL;
            }
    }

    return truecolor;
}

int ph_FillHWRect(_THIS, SDL_Surface* surface, SDL_Rect* rect, Uint32 color)
{
    PgColor_t oldcolor;
    Uint32 truecolor;
    int ydisp=0;

    if (this->info.blit_fill!=1)
    {
       return -1;
    }

    truecolor=ph_ExpandColor(this, surface, color);
    if (truecolor==0xFFFFFFFFUL)
    {
        return -1;
    }

    oldcolor=PgSetFillColor(truecolor);

    /* 640x400 videomode emulation */
    if (videomode_emulatemode==1)
    {
        ydisp+=40;
    }

    PgDrawIRect(rect->x, rect->y+ydisp, rect->w+rect->x-1, rect->h+rect->y+ydisp-1, Pg_DRAW_FILL);
    PgSetFillColor(oldcolor);
    PgFlush();
    PgWaitHWIdle();

    return 0;
}

int ph_FlipHWSurface(_THIS, SDL_Surface* screen)
{
    PhArea_t farea;

    if ((screen->flags & SDL_FULLSCREEN) == SDL_FULLSCREEN)
    {
        /* flush all drawing ops before blitting */
        PgFlush();
        PgWaitHWIdle();

        farea.pos.x=0;
        farea.pos.y=0;
        farea.size.w=screen->w;
        farea.size.h=screen->h;

        /* emulate 640x400 videomode */
        if (videomode_emulatemode==1)
        {
            farea.pos.y+=40;
        }

        PgContextBlitArea(OCImage.offscreen_context, &farea, OCImage.offscreen_backcontext, &farea);

        /* flush the blitting */
        PgFlush();
        PgWaitHWIdle();
    }
    return 0;
}

int ph_LockHWSurface(_THIS, SDL_Surface* surface)
{

#if 0 /* FIXME */
    int lockresult;

    if (surface->hwdata == NULL)
    {
        return;
    }

    surface->hwdata->lockparam.flags=0;
    surface->hwdata->lockparam.time_out=NULL;
    lockresult=PdLockOffscreen(surface->hwdata->offscreenctx, &surface->hwdata->lockparam);

    switch (lockresult)
    {
       case EOK:
                 break;
       case Pg_OSC_LOCK_DEADLOCK: 
                 SDL_SetError("ph_LockHWSurface(): Deadlock detected !\n");
                 return -1;
       case Pg_OSC_LOCK_INVALID:
                 SDL_SetError("ph_LockHWSurface(): Lock invalid !\n");
                 return -1;
       default:
                 SDL_SetError("ph_LockHWSurface(): Can't lock the surface !\n");
                 return -1;
    }
#endif /* 0 */

    return 0;
}

void ph_UnlockHWSurface(_THIS, SDL_Surface* surface)
{

#if 0 /* FIXME */
    int unlockresult;

    if ((surface == NULL) || (surface->hwdata == NULL))
    {
        return;
    }

    if (PdIsOffscreenLocked(surface->hwdata->offscreenctx)==Pg_OSC_LOCKED)
    {
        unlockresult=PdUnlockOffscreen(surface->hwdata->offscreenctx);
    }
#endif /* 0 */

    return;
}

int ph_HWAccelBlit(SDL_Surface* src, SDL_Rect* srcrect, SDL_Surface* dst, SDL_Rect* dstrect)
{
    SDL_VideoDevice* this=current_video;
    PhArea_t srcarea;
    PhArea_t dstarea;
    int ydisp=0;

    /* 640x400 videomode emulation */
    if (videomode_emulatemode==1)
    {
       ydisp+=40;
    }

    srcarea.pos.x=srcrect->x;
    srcarea.pos.y=srcrect->y;
    srcarea.size.w=srcrect->w;
    srcarea.size.h=srcrect->h;

    dstarea.pos.x=dstrect->x;
    dstarea.pos.y=dstrect->y;
    dstarea.size.w=dstrect->w;
    dstarea.size.h=dstrect->h;

    if (((src == this->screen) || (src->hwdata!=NULL)) && ((dst == this->screen) || (dst->hwdata!=NULL)))
    {
        if ((src->flags & SDL_SRCCOLORKEY) == SDL_SRCCOLORKEY)
        {
            ph_SetHWColorKey(this, src, src->format->colorkey);
            PgChromaOn();
        }

        if ((src->flags & SDL_SRCALPHA) == SDL_SRCALPHA)
        {
            ph_SetHWAlpha(this, src, src->format->alpha);
            PgAlphaOn();
        }

        if (dst == this->screen)
        {
            if (src == this->screen)
            {
                /* blitting from main screen to main screen */
                dstarea.pos.y+=ydisp;
                srcarea.pos.y+=ydisp;
                PgContextBlitArea(OCImage.offscreen_context, &srcarea, OCImage.offscreen_context, &dstarea);
            }
            else
            {
                /* blitting from offscreen to main screen */
                dstarea.pos.y+=ydisp;
                PgContextBlitArea(src->hwdata->offscreenctx, &srcarea, OCImage.offscreen_context, &dstarea);
            }
        }
        else
        {
            if (src == this->screen)
            {
                /* blitting from main screen to offscreen */
                srcarea.pos.y+=ydisp;
                PgContextBlitArea(OCImage.offscreen_context, &srcarea, dst->hwdata->offscreenctx, &dstarea);
            }
            else
            {
                /* blitting offscreen to offscreen */
                PgContextBlitArea(src->hwdata->offscreenctx, &srcarea, dst->hwdata->offscreenctx, &dstarea);
            }
        }

        if ((src->flags & SDL_SRCALPHA) == SDL_SRCALPHA)
        {
            PgAlphaOff();
        }

        if ((src->flags & SDL_SRCCOLORKEY) == SDL_SRCCOLORKEY)
        {
            PgChromaOff();
        }
    }
    else
    {
        SDL_SetError("ph_HWAccelBlit(): Source or target surface is not a hardware surface !\n");
        return -1;
    }

    PgFlush();
    PgWaitHWIdle();

    return 0;
}

int ph_SetHWColorKey(_THIS, SDL_Surface *surface, Uint32 key)
{
    if (this->info.blit_hw_CC!=1)
    {
       return -1;
    }

    if (surface->hwdata!=NULL)
    {
        surface->hwdata->colorkey=ph_ExpandColor(this, surface, key);
        if (surface->hwdata->colorkey==0xFFFFFFFFUL)
        {
            return -1;
        }
    }
    PgSetChroma(surface->hwdata->colorkey, Pg_CHROMA_SRC_MATCH | Pg_CHROMA_NODRAW);

    return 0;
}

int ph_SetHWAlpha(_THIS, SDL_Surface* surface, Uint8 alpha)
{
    if (this->info.blit_hw_A!=1)
    {
       return -1;
    }

    PgSetAlphaBlend(NULL, alpha);

    return 0;
}

#if SDL_VIDEO_OPENGL
void ph_OpenGLUpdate(_THIS, int numrects, SDL_Rect* rects)
{
   this->GL_SwapBuffers(this);
   
   return;
}
#endif /* SDL_VIDEO_OPENGL */

void ph_NormalUpdate(_THIS, int numrects, SDL_Rect *rects)
{
    PhPoint_t ph_pos;
    PhRect_t ph_rect;
    int i;

    for (i=0; i<numrects; ++i) 
    {
    	if (rects[i].w==0) /* Clipped? dunno why but this occurs sometime. */
        { 
            continue;
        }

    	if (rects[i].h==0) /* Clipped? dunno why but this occurs sometime. */
        { 
            continue;
        }

        ph_pos.x = rects[i].x;
        ph_pos.y = rects[i].y;
        ph_rect.ul.x = rects[i].x;
        ph_rect.ul.y = rects[i].y;
        ph_rect.lr.x = rects[i].x + rects[i].w;
        ph_rect.lr.y = rects[i].y + rects[i].h;

        if (PgDrawPhImageRectmx(&ph_pos, SDL_Image, &ph_rect, 0) < 0)
        {
            SDL_SetError("ph_NormalUpdate(): PgDrawPhImageRectmx failed!\n");
            return;
        }
    }

    if (PgFlush() < 0)
    {
    	SDL_SetError("ph_NormalUpdate(): PgFlush() function failed!\n");
    }
}

void ph_OCUpdate(_THIS, int numrects, SDL_Rect *rects)
{
    int i;

    PhPoint_t zero = {0, 0};
    PhArea_t src_rect;
    PhArea_t dest_rect;

    PgSetTranslation(&zero, 0);
    PgSetRegion(PtWidgetRid(window));
    PgSetClipping(0, NULL);

    PgFlush();
    PgWaitHWIdle();

    for (i=0; i<numrects; ++i)
    {
        if (rects[i].w == 0)  /* Clipped? */
        {
            continue;
        }

        if (rects[i].h == 0)  /* Clipped? */
        {
            continue;
        }
        
        src_rect.pos.x=rects[i].x;
        src_rect.pos.y=rects[i].y;
        dest_rect.pos.x=rects[i].x;
        dest_rect.pos.y=rects[i].y;

        src_rect.size.w=rects[i].w;
        src_rect.size.h=rects[i].h;
        dest_rect.size.w=rects[i].w;
        dest_rect.size.h=rects[i].h;
        
        PgContextBlitArea(OCImage.offscreen_context, &src_rect, NULL, &dest_rect);
    }

    if (PgFlush() < 0)
    {
        SDL_SetError("ph_OCUpdate(): PgFlush failed.\n");
    }
}

void ph_OCDCUpdate(_THIS, int numrects, SDL_Rect *rects)
{
    PgWaitHWIdle();

    if (PgFlush() < 0)
    {
        SDL_SetError("ph_OCDCUpdate(): PgFlush failed.\n");
    }
}
