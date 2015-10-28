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

/* This is the QNX Realtime Platform version of SDL YUV video overlays */

#include <errno.h>

#include <Ph.h>
#include <Pt.h>

#include "SDL_video.h"
#include "SDL_phyuv_c.h"
#include "../SDL_yuvfuncs.h"

#define OVERLAY_STATE_UNINIT 0
#define OVERLAY_STATE_ACTIVE 1

/* The functions are used to manipulate software video overlays */
static struct private_yuvhwfuncs ph_yuvfuncs =
{
    ph_LockYUVOverlay,
    ph_UnlockYUVOverlay,
    ph_DisplayYUVOverlay,
    ph_FreeYUVOverlay
};

int grab_ptrs2(PgVideoChannel_t* channel, FRAMEDATA* Frame0, FRAMEDATA* Frame1)
{
    int planes = 0;

    /* Buffers have moved; re-obtain the pointers */
    Frame0->Y = (unsigned char *)PdGetOffscreenContextPtr(channel->yplane1);
    Frame1->Y = (unsigned char *)PdGetOffscreenContextPtr(channel->yplane2);
    Frame0->U = (unsigned char *)PdGetOffscreenContextPtr(channel->vplane1);
    Frame1->U = (unsigned char *)PdGetOffscreenContextPtr(channel->vplane2);
    Frame0->V = (unsigned char *)PdGetOffscreenContextPtr(channel->uplane1);
    Frame1->V = (unsigned char *)PdGetOffscreenContextPtr(channel->uplane2);

    if (Frame0->Y)
        planes++;

    if (Frame0->U)
        planes++;

    if (Frame0->V)
        planes++;

    return planes;
}

SDL_Overlay* ph_CreateYUVOverlay(_THIS, int width, int height, Uint32 format, SDL_Surface* display)
{
    SDL_Overlay* overlay;
    struct private_yuvhwdata* hwdata;
    int vidport;
    int rtncode;
    int planes;
    int i=0;
    PhPoint_t pos;

    /* Create the overlay structure */
    overlay = SDL_calloc(1, sizeof(SDL_Overlay));

    if (overlay == NULL)
    {
        SDL_OutOfMemory();
        return NULL;
    }

    /* Fill in the basic members */
    overlay->format = format;
    overlay->w = width;
    overlay->h = height;
    overlay->hwdata = NULL;
	
    /* Set up the YUV surface function structure */
    overlay->hwfuncs = &ph_yuvfuncs;

    /* Create the pixel data and lookup tables */
    hwdata = SDL_calloc(1, sizeof(struct private_yuvhwdata));

    if (hwdata == NULL)
    {
        SDL_OutOfMemory();
        SDL_FreeYUVOverlay(overlay);
        return NULL;
    }

    overlay->hwdata = hwdata;

    PhDCSetCurrent(0);
    if (overlay->hwdata->channel == NULL)
    {
        if ((overlay->hwdata->channel = PgCreateVideoChannel(Pg_VIDEO_CHANNEL_SCALER, 0)) == NULL)
        {
            SDL_SetError("ph_CreateYUVOverlay(): Create channel failed: %s\n", strerror(errno));
            SDL_FreeYUVOverlay(overlay);
            return NULL;

        }
    }

    overlay->hwdata->forcedredraw=0;

    PtGetAbsPosition(window, &pos.x, &pos.y);
    overlay->hwdata->CurrentWindowPos.x = pos.x;
    overlay->hwdata->CurrentWindowPos.y = pos.y;
    overlay->hwdata->CurrentViewPort.pos.x = 0;
    overlay->hwdata->CurrentViewPort.pos.y = 0;
    overlay->hwdata->CurrentViewPort.size.w = width;
    overlay->hwdata->CurrentViewPort.size.h = height;
    overlay->hwdata->State = OVERLAY_STATE_UNINIT;
    overlay->hwdata->FrameData0 = (FRAMEDATA *) SDL_calloc(1, sizeof(FRAMEDATA));
    overlay->hwdata->FrameData1 = (FRAMEDATA *) SDL_calloc(1, sizeof(FRAMEDATA));

    vidport = -1;
    i=0;
    
    overlay->hwdata->ischromakey=0;

    do {
        SDL_memset(&overlay->hwdata->caps, 0x00, sizeof(PgScalerCaps_t));
        overlay->hwdata->caps.size = sizeof(PgScalerCaps_t);
        rtncode = PgGetScalerCapabilities(overlay->hwdata->channel, i, &overlay->hwdata->caps);
        if (rtncode==0)
        { 
            if (overlay->hwdata->caps.format==format)
            {
               if ((overlay->hwdata->caps.flags & Pg_SCALER_CAP_DST_CHROMA_KEY) == Pg_SCALER_CAP_DST_CHROMA_KEY)
               {
                   overlay->hwdata->ischromakey=1;
               }
               vidport=1;
               break;
            }
        }
        else
        {
           break;
        }
        i++;
    } while(1);


    if (vidport == -1)
    {
        SDL_SetError("No available video ports for requested format\n");
        SDL_FreeYUVOverlay(overlay);
        return NULL;
    }

    overlay->hwdata->format = format;
    overlay->hwdata->props.format = format;
    overlay->hwdata->props.size = sizeof(PgScalerProps_t);
    overlay->hwdata->props.src_dim.w = width;
    overlay->hwdata->props.src_dim.h = height;

    /* overlay->hwdata->chromakey = PgGetOverlayChromaColor(); */
    overlay->hwdata->chromakey = PgRGB(12, 6, 12); /* very dark pink color */
    overlay->hwdata->props.color_key = overlay->hwdata->chromakey;

    PhAreaToRect(&overlay->hwdata->CurrentViewPort, &overlay->hwdata->props.viewport);

    overlay->hwdata->props.flags = Pg_SCALER_PROP_DOUBLE_BUFFER;

    if ((overlay->hwdata->ischromakey)&&(overlay->hwdata->chromakey))
    {
        overlay->hwdata->props.flags |= Pg_SCALER_PROP_CHROMA_ENABLE;
        overlay->hwdata->props.flags |= Pg_SCALER_PROP_CHROMA_SPECIFY_KEY_MASK;
    } 
    else
    {
        overlay->hwdata->props.flags &= ~Pg_SCALER_PROP_CHROMA_ENABLE;
    }

    rtncode = PgConfigScalerChannel(overlay->hwdata->channel, &overlay->hwdata->props);

    switch(rtncode)
    {
        case -1: SDL_SetError("PgConfigScalerChannel failed\n");
                 SDL_FreeYUVOverlay(overlay);
                 return NULL;
        case 1:
        case 0:
        default:
                 break;
    }

    planes = grab_ptrs2(overlay->hwdata->channel, overlay->hwdata->FrameData0, overlay->hwdata->FrameData1);

    if(overlay->hwdata->channel->yplane1 != NULL)
        overlay->hwdata->YStride = overlay->hwdata->channel->yplane1->pitch;
    if(overlay->hwdata->channel->vplane1 != NULL)
        overlay->hwdata->UStride = overlay->hwdata->channel->vplane1->pitch;
    if(overlay->hwdata->channel->uplane1 != NULL)
        overlay->hwdata->VStride = overlay->hwdata->channel->uplane1->pitch;

    /* check for the validness of all planes */
    if ((overlay->hwdata->channel->yplane1 == NULL) &&
        (overlay->hwdata->channel->uplane1 == NULL) &&
        (overlay->hwdata->channel->vplane1 == NULL))
    {
       SDL_FreeYUVOverlay(overlay);
       SDL_SetError("PgConfigScaler() returns all planes equal NULL\n");
       return NULL;
    }
/*
    overlay->hwdata->current = PgNextVideoFrame(overlay->hwdata->channel);

    if (overlay->hwdata->current==0)
    {
        overlay->hwdata->CurrentFrameData = overlay->hwdata->FrameData0;
    }
    else
    {
        overlay->hwdata->CurrentFrameData = overlay->hwdata->FrameData1;
    }
*/
    overlay->hwdata->CurrentFrameData = overlay->hwdata->FrameData0;

/*
    overlay->hwdata->locked = 1;
*/

    /* Find the pitch and offset values for the overlay */
    overlay->planes = planes;
    overlay->pitches = SDL_calloc(overlay->planes, sizeof(Uint16));
    overlay->pixels  = SDL_calloc(overlay->planes, sizeof(Uint8*));
    if (!overlay->pitches || !overlay->pixels)
    {
        SDL_OutOfMemory();
        SDL_FreeYUVOverlay(overlay);
        return(NULL);
    }

    if (overlay->planes > 0)
    {
        overlay->pitches[0] = overlay->hwdata->channel->yplane1->pitch;
        overlay->pixels[0]  = overlay->hwdata->CurrentFrameData->Y;
    }
    if (overlay->planes > 1)
    {
        overlay->pitches[1] = overlay->hwdata->channel->vplane1->pitch;
        overlay->pixels[1]  = overlay->hwdata->CurrentFrameData->U;
    }
    if (overlay->planes > 2)
    {
        overlay->pitches[2] = overlay->hwdata->channel->uplane1->pitch;
        overlay->pixels[2]  = overlay->hwdata->CurrentFrameData->V;
    }

    overlay->hwdata->State = OVERLAY_STATE_ACTIVE;
    overlay->hwdata->scaler_on = 0;
    overlay->hw_overlay = 1;

    current_overlay=overlay;

    return overlay;
}

int ph_LockYUVOverlay(_THIS, SDL_Overlay* overlay)
{
    if (overlay == NULL)
    {
        return -1;
    }

    overlay->hwdata->locked = 1;

/*  overlay->hwdata->current = PgNextVideoFrame(overlay->hwdata->channel);
    if (overlay->hwdata->current == -1)
    {
        SDL_SetError("ph_LockYUVOverlay: PgNextFrame() failed, bailing out\n");
        SDL_FreeYUVOverlay(overlay);
        return 0;
    }

    if (overlay->hwdata->current == 0)
    {
        overlay->hwdata->CurrentFrameData = overlay->hwdata->FrameData0;
    }
    else
    {
        overlay->hwdata->CurrentFrameData = overlay->hwdata->FrameData1;
    }

    if (overlay->planes > 0)
    {
        overlay->pitches[0] = overlay->hwdata->channel->yplane1->pitch;
        overlay->pixels[0]  = overlay->hwdata->CurrentFrameData->Y;
    }
    if (overlay->planes > 1)
    {
        overlay->pitches[1] = overlay->hwdata->channel->uplane1->pitch;
        overlay->pixels[1]  = overlay->hwdata->CurrentFrameData->U;
    }
    if (overlay->planes > 2)
    {
        overlay->pitches[2] = overlay->hwdata->channel->vplane1->pitch;
        overlay->pixels[2]  = overlay->hwdata->CurrentFrameData->V;
    }
*/

    return(0);
}

void ph_UnlockYUVOverlay(_THIS, SDL_Overlay* overlay)
{
    if (overlay == NULL)
    {
        return;
    }

    overlay->hwdata->locked = 0;
}

int ph_DisplayYUVOverlay(_THIS, SDL_Overlay* overlay, SDL_Rect* src, SDL_Rect* dst)
{
    int rtncode;
    PhPoint_t pos;
    SDL_Rect backrect;
    PhRect_t windowextent;
    int winchanged=0;

    if ((overlay == NULL) || (overlay->hwdata==NULL))
    {
        return -1;
    }

    if (overlay->hwdata->State == OVERLAY_STATE_UNINIT)
    {
        return -1;
    }

    PtGetAbsPosition(window, &pos.x, &pos.y);
    if ((pos.x!=overlay->hwdata->CurrentWindowPos.x) ||
        (pos.y!=overlay->hwdata->CurrentWindowPos.y))
    {
       winchanged=1;
       overlay->hwdata->CurrentWindowPos.x=pos.x;
       overlay->hwdata->CurrentWindowPos.y=pos.y;
    }

    /* If CurrentViewPort position/size has been changed, then move/resize the viewport */
    if ((overlay->hwdata->CurrentViewPort.pos.x != dst->x) ||
        (overlay->hwdata->CurrentViewPort.pos.y != dst->y) ||
        (overlay->hwdata->CurrentViewPort.size.w != dst->w) ||
        (overlay->hwdata->CurrentViewPort.size.h != dst->h) ||
        (overlay->hwdata->scaler_on==0) || (winchanged==1) ||
        (overlay->hwdata->forcedredraw==1))
    {

        if (overlay->hwdata->ischromakey==1)
        {
            /* restore screen behind the overlay/chroma color. */
            backrect.x=overlay->hwdata->CurrentViewPort.pos.x;
            backrect.y=overlay->hwdata->CurrentViewPort.pos.y;
            backrect.w=overlay->hwdata->CurrentViewPort.size.w;
            backrect.h=overlay->hwdata->CurrentViewPort.size.h;
            this->UpdateRects(this, 1, &backrect);

            /* Draw the new rectangle of the chroma color at the viewport position */
            PgSetFillColor(overlay->hwdata->chromakey);
            PgDrawIRect(dst->x, dst->y, dst->x+dst->w-1, dst->y+dst->h-1, Pg_DRAW_FILL);
            PgFlush();
        }

        overlay->hwdata->props.flags |= Pg_SCALER_PROP_SCALER_ENABLE;
        overlay->hwdata->scaler_on = 1;

        PhWindowQueryVisible(Ph_QUERY_CONSOLE, 0, PtWidgetRid(window), &windowextent);
        overlay->hwdata->CurrentViewPort.pos.x = pos.x-windowextent.ul.x+dst->x;
        overlay->hwdata->CurrentViewPort.pos.y = pos.y-windowextent.ul.y+dst->y;
        overlay->hwdata->CurrentViewPort.size.w = dst->w;
        overlay->hwdata->CurrentViewPort.size.h = dst->h;
        PhAreaToRect(&overlay->hwdata->CurrentViewPort, &overlay->hwdata->props.viewport);
        overlay->hwdata->CurrentViewPort.pos.x = dst->x;
        overlay->hwdata->CurrentViewPort.pos.y = dst->y;

        rtncode = PgConfigScalerChannel(overlay->hwdata->channel, &(overlay->hwdata->props));

        switch(rtncode)
        {
            case -1:
                     SDL_SetError("PgConfigScalerChannel() function failed\n");
                     SDL_FreeYUVOverlay(overlay);
                     return -1;
            case 1:
                     grab_ptrs2(overlay->hwdata->channel, overlay->hwdata->FrameData0, overlay->hwdata->FrameData1);
                     break;
            case 0:
            default:
                     break;
        }
    }


/*
    if (overlay->hwdata->locked==0)
    {
        overlay->hwdata->current = PgNextVideoFrame(overlay->hwdata->channel);
        if (overlay->hwdata->current == -1)
        {
            SDL_SetError("ph_LockYUVOverlay: PgNextFrame() failed, bailing out\n");
            SDL_FreeYUVOverlay(overlay);
            return 0;
        }

        if (overlay->hwdata->current == 0)
        {
            overlay->hwdata->CurrentFrameData = overlay->hwdata->FrameData0;
        }
        else
        {
            overlay->hwdata->CurrentFrameData = overlay->hwdata->FrameData1;
        }

        if (overlay->planes > 0)
        {
            overlay->pitches[0] = overlay->hwdata->channel->yplane1->pitch;
            overlay->pixels[0]  = overlay->hwdata->CurrentFrameData->Y;
        }
        if (overlay->planes > 1)
        {
            overlay->pitches[1] = overlay->hwdata->channel->uplane1->pitch;
            overlay->pixels[1]  = overlay->hwdata->CurrentFrameData->U;
        }
        if (overlay->planes > 2)
        {
            overlay->pitches[2] = overlay->hwdata->channel->vplane1->pitch;
            overlay->pixels[2]  = overlay->hwdata->CurrentFrameData->V;
        }
    }
*/
        
    return 0;
}

void ph_FreeYUVOverlay(_THIS, SDL_Overlay *overlay)
{
    SDL_Rect backrect;

    if (overlay == NULL)
    {
        return;
    }

    if (overlay->hwdata == NULL)
    {
        return;
    }

    current_overlay=NULL;

    /* restore screen behind the overlay/chroma color. */
    backrect.x=overlay->hwdata->CurrentViewPort.pos.x;
    backrect.y=overlay->hwdata->CurrentViewPort.pos.y;
    backrect.w=overlay->hwdata->CurrentViewPort.size.w;
    backrect.h=overlay->hwdata->CurrentViewPort.size.h;
    this->UpdateRects(this, 1, &backrect);

    /* it is need for some buggy drivers, that can't hide overlay before */
    /* freeing buffer, so we got trash on the srceen                     */
    overlay->hwdata->props.flags &= ~Pg_SCALER_PROP_SCALER_ENABLE;
    PgConfigScalerChannel(overlay->hwdata->channel, &(overlay->hwdata->props));

    overlay->hwdata->scaler_on = 0;
    overlay->hwdata->State = OVERLAY_STATE_UNINIT;

    if (overlay->hwdata->channel != NULL)
    {
        PgDestroyVideoChannel(overlay->hwdata->channel);
        overlay->hwdata->channel = NULL;
        return;
    }	

    overlay->hwdata->CurrentFrameData = NULL;  
	
    SDL_free(overlay->hwdata->FrameData0);
    SDL_free(overlay->hwdata->FrameData1);
    overlay->hwdata->FrameData0 = NULL;
    overlay->hwdata->FrameData1 = NULL;
    SDL_free(overlay->hwdata);
}
