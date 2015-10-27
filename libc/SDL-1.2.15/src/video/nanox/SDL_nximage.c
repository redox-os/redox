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

#include "SDL_nximage_c.h"

void NX_NormalUpdate (_THIS, int numrects, SDL_Rect * rects)
{
    int           i, j, xinc, yinc, destinc, rowinc ;
    int           x, y, w, h ;
    unsigned char * src = NULL, * dest = NULL ;

    Dprintf ("enter NX_NormalUpdate\n") ;
    
    /* These are the values for the incoming image */
    xinc = this -> screen -> format -> BytesPerPixel ;
    yinc = this -> screen -> pitch ;
        
    for (i = 0; i < numrects; ++ i) {
        x = rects [i].x, y = rects [i].y ;
        w = rects [i].w, h = rects [i].h ;
        src = SDL_Image + y * yinc + x * xinc ;
#ifdef ENABLE_NANOX_DIRECT_FB
        if (Clientfb) {
            if (currently_fullscreen)
                dest = fbinfo.winpixels + (((y+OffsetY) * fbinfo.pitch) +
                    ((x+OffsetX) * fbinfo.bytespp));
            else
                dest = fbinfo.winpixels + ((y * fbinfo.pitch) + (x * fbinfo.bytespp));
            destinc = fbinfo.pitch;
        }
        else
#endif
        {
            dest = Image_buff ;
            destinc = w * xinc ;
        }
        rowinc = w * xinc;

        // apply GammaRamp table
        if ((pixel_type == MWPF_TRUECOLOR0888 || pixel_type == MWPF_TRUECOLOR888)
          && GammaRamp_R && GammaRamp_G && GammaRamp_B) {
            Uint8 * ptrsrc ;
            Uint8 * ptrdst ;
            int   k ;

            for (j = h; j > 0; -- j, src += yinc, dest += destinc) {
                ptrsrc = src ;
                ptrdst = dest ;
                for (k = w; k > 0; -- k) {
                    *ptrdst++ = GammaRamp_B [*ptrsrc++] >> 8;
                    *ptrdst++ = GammaRamp_G [*ptrsrc++] >> 8;
                    *ptrdst++ = GammaRamp_R [*ptrsrc++] >> 8;
                    *ptrdst++ = 0;
                    ++ptrsrc;
                }
            }
        }
#if 1 /* This is needed for microwindows 0.90 or older */
        else if (pixel_type == MWPF_TRUECOLOR0888 || pixel_type == MWPF_TRUECOLOR888) {
            Uint8 * ptrsrc ;
            Uint8 * ptrdst ;
            int   k ;

            for (j = h; j > 0; -- j, src += yinc, dest += destinc) {
                ptrsrc = src ;
                ptrdst = dest ;
                for (k = w; k > 0; -- k) {
                    *ptrdst++ = *ptrsrc++;
                    *ptrdst++ = *ptrsrc++;
                    *ptrdst++ = *ptrsrc++;
                    *ptrdst++ = 0;
                    ++ptrsrc;
                }
            }
        }
#endif
        else
        {
            for (j = h; j > 0; -- j, src += yinc, dest += destinc)
                SDL_memcpy (dest, src, rowinc) ;
        }
        if (!Clientfb) {
            if (currently_fullscreen) {
                GrArea (FSwindow, SDL_GC, x + OffsetX, y + OffsetY, w, h, Image_buff, 
                    pixel_type) ;
            } else {
                GrArea (SDL_Window, SDL_GC, x, y, w, h, Image_buff, pixel_type) ;
            }
        }
    }
    GrFlush();

    Dprintf ("leave NX_NormalUpdate\n") ;
}

int NX_SetupImage (_THIS, SDL_Surface * screen)
{
    int size = screen -> h * screen -> pitch ;
    
    Dprintf ("enter NX_SetupImage\n") ;

    screen -> pixels = (void *) SDL_malloc (size) ;

    if (!Clientfb) {
        Image_buff = (unsigned char *) SDL_malloc (size) ;
        if (screen -> pixels == NULL || Image_buff == NULL) {
            SDL_free (screen -> pixels) ;
            SDL_free (Image_buff) ;
            SDL_OutOfMemory () ;
            return -1 ;
        }
    }

    SDL_Image = (unsigned char *) screen -> pixels ;

    this -> UpdateRects = NX_NormalUpdate ;

    Dprintf ("leave NX_SetupImage\n") ;
    return 0 ;
}

void NX_DestroyImage (_THIS, SDL_Surface * screen)
{
    Dprintf ("enter NX_DestroyImage\n") ;
    
    if (SDL_Image) SDL_free (SDL_Image) ;
    if (Image_buff) SDL_free (Image_buff) ;
    if (screen) screen -> pixels = NULL ;
    
    Dprintf ("leave NX_DestroyImage\n") ;
}

int NX_ResizeImage (_THIS, SDL_Surface * screen, Uint32 flags)
{
    int            retval ;
    GR_SCREEN_INFO si ;

    Dprintf ("enter NX_ResizeImage\n") ;

    NX_DestroyImage (this, screen) ;
    retval = NX_SetupImage (this, screen) ;

    GrGetScreenInfo (& si) ;
    OffsetX = (si.cols - screen -> w) / 2 ;
    OffsetY = (si.rows - screen -> h) / 2 ;

#ifdef ENABLE_NANOX_DIRECT_FB
    if (Clientfb) {
        /* Get current window position and fb pointer*/
        if (currently_fullscreen) 
            GrGetWindowFBInfo(FSwindow, &fbinfo);
        else
            GrGetWindowFBInfo(SDL_Window, &fbinfo);
    }
#endif
    Dprintf ("leave NX_ResizeImage\n") ;
    return retval ;
}

void NX_RefreshDisplay (_THIS)
{
    Dprintf ("enter NX_RefreshDisplay\n") ;

    // Don't refresh a display that doesn't have an image (like GL)
    if (! SDL_Image) {
        return;
    }

#ifdef ENABLE_NANOX_DIRECT_FB
    if (Clientfb) {
        int j;
        char *src, *dest = NULL;
        int xinc, yinc, rowinc;

        GrGetWindowFBInfo(SDL_Window, &fbinfo);

        xinc = this -> screen -> format -> BytesPerPixel ; 
        yinc = this -> screen -> pitch ;           

        src = SDL_Image;
        if (currently_fullscreen)
            dest = fbinfo.winpixels + ((OffsetY * fbinfo.pitch) +
                (OffsetX * fbinfo.bytespp));
        else
            dest = fbinfo.winpixels;
        rowinc = xinc * this -> screen -> w;

        for (j = this -> screen -> h; j > 0; -- j, src += yinc, dest += fbinfo.pitch)
            SDL_memcpy (dest, src, rowinc) ;
    }
    else
#endif
    {
        if (currently_fullscreen) {
            GrArea (FSwindow, SDL_GC, OffsetX, OffsetY, this -> screen -> w, 
                this -> screen -> h, SDL_Image, pixel_type) ;
        } else {
            GrArea (SDL_Window, SDL_GC, 0, 0, this -> screen -> w, 
                this -> screen -> h, SDL_Image, pixel_type) ;
        }
    }
    GrFlush();

    Dprintf ("leave NX_RefreshDisplay\n") ;
}
