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

/* This is the DirectFB implementation of YUV video overlays */

#include "SDL_video.h"
#include "SDL_DirectFB_yuv.h"
#include "../SDL_yuvfuncs.h"


/* The functions used to manipulate software video overlays */
static struct private_yuvhwfuncs directfb_yuvfuncs = {
  DirectFB_LockYUVOverlay,
  DirectFB_UnlockYUVOverlay,
  DirectFB_DisplayYUVOverlay,
  DirectFB_FreeYUVOverlay
};

struct private_yuvhwdata {
  DFBDisplayLayerID      layer_id;

  IDirectFBDisplayLayer *layer;
  IDirectFBSurface      *surface;

  /* These are just so we don't have to allocate them separately */
  Uint16 pitches[3];
  Uint8 *planes[3];
};

static DFBEnumerationResult
enum_layers_callback( DFBDisplayLayerID            id,
                      DFBDisplayLayerDescription   desc,
                      void                        *data )
{
  struct private_yuvhwdata *hwdata = (struct private_yuvhwdata *) data;

  /* we don't want the primary */
  if (id == DLID_PRIMARY)
    return DFENUM_OK;

  /* take the one with a surface for video */
  if ((desc.caps & DLCAPS_SURFACE) && (desc.type & DLTF_VIDEO))
    {
      hwdata->layer_id = id;

      return DFENUM_CANCEL;
    }

  return DFENUM_OK;
}


static DFBResult CreateYUVSurface(_THIS, struct private_yuvhwdata *hwdata,
                                  int width, int height, Uint32 format)
{
  DFBResult              ret;
  IDirectFB             *dfb = HIDDEN->dfb;
  IDirectFBDisplayLayer *layer;
  DFBDisplayLayerConfig  conf;

  ret = dfb->EnumDisplayLayers (dfb, enum_layers_callback, hwdata);
  if (ret)
    {
      SetDirectFBerror("IDirectFB::EnumDisplayLayers", ret);
      return ret;
    }

  if (!hwdata->layer_id)
    return DFB_UNSUPPORTED;

  ret = dfb->GetDisplayLayer (dfb, hwdata->layer_id, &layer);
  if (ret)
    {
      SetDirectFBerror("IDirectFB::GetDisplayLayer", ret);
      return ret;
    }

  conf.flags = DLCONF_WIDTH | DLCONF_HEIGHT | DLCONF_PIXELFORMAT;
  conf.width = width;
  conf.height = height;

  switch (format)
    {
    case SDL_YV12_OVERLAY:
      conf.pixelformat = DSPF_YV12;
      break;
    case SDL_IYUV_OVERLAY:
      conf.pixelformat = DSPF_I420;
      break;
    case SDL_YUY2_OVERLAY:
      conf.pixelformat = DSPF_YUY2;
      break;
    case SDL_UYVY_OVERLAY:
      conf.pixelformat = DSPF_UYVY;
      break;
    default:
      fprintf (stderr, "SDL_DirectFB: Unsupported YUV format (0x%08x)!\n", format);
      break;
    }

  /* Need to set coop level or newer DirectFB versions will fail here. */
  ret = layer->SetCooperativeLevel (layer, DLSCL_ADMINISTRATIVE);
  if (ret)
    {
      SetDirectFBerror("IDirectFBDisplayLayer::SetCooperativeLevel() failed", ret);
      layer->Release (layer);
      return ret;
    }

  ret = layer->SetConfiguration (layer, &conf);
  if (ret)
    {
      SetDirectFBerror("IDirectFBDisplayLayer::SetConfiguration", ret);
      layer->Release (layer);
      return ret;
    }

  ret = layer->GetSurface (layer, &hwdata->surface);
  if (ret)
    {
      SetDirectFBerror("IDirectFBDisplayLayer::GetSurface", ret);
      layer->Release (layer);
      return ret;
    }

  hwdata->layer = layer;

  return DFB_OK;
}

SDL_Overlay *DirectFB_CreateYUVOverlay(_THIS, int width, int height, Uint32 format, SDL_Surface *display)
{
  SDL_Overlay *overlay;
  struct private_yuvhwdata *hwdata;

  /* Create the overlay structure */
  overlay = SDL_calloc (1, sizeof(SDL_Overlay));
  if (!overlay)
    {
      SDL_OutOfMemory();
      return NULL;
    }
	
  /* Fill in the basic members */
  overlay->format = format;
  overlay->w = width;
  overlay->h = height;

  /* Set up the YUV surface function structure */
  overlay->hwfuncs = &directfb_yuvfuncs;

  /* Create the pixel data and lookup tables */
  hwdata = SDL_calloc(1, sizeof(struct private_yuvhwdata));
  overlay->hwdata = hwdata;
  if (!hwdata)
    {
      SDL_OutOfMemory();
      SDL_FreeYUVOverlay (overlay);
      return NULL;
    }

  if (CreateYUVSurface (this, hwdata, width, height, format))
    {
      SDL_FreeYUVOverlay (overlay);
      return NULL;
    }

  overlay->hw_overlay = 1;

  /* Set up the plane pointers */
  overlay->pitches = hwdata->pitches;
  overlay->pixels = hwdata->planes;
  switch (format)
    {
    case SDL_YV12_OVERLAY:
    case SDL_IYUV_OVERLAY:
      overlay->planes = 3;
      break;
    default:
      overlay->planes = 1;
      break;
    }

  /* We're all done.. */
  return overlay;
}

int DirectFB_LockYUVOverlay(_THIS, SDL_Overlay *overlay)
{
  DFBResult         ret;
  void             *data;
  int               pitch;
  IDirectFBSurface *surface = overlay->hwdata->surface;

  ret = surface->Lock (surface, DSLF_READ | DSLF_WRITE, &data, &pitch);
  if (ret)
    {
      SetDirectFBerror("IDirectFBSurface::Lock", ret);
      return -1;
    }

  /* Find the pitch and offset values for the overlay */
  overlay->pitches[0] = (Uint16) pitch;
  overlay->pixels[0]  = (Uint8*) data;

  switch (overlay->format)
    {
    case SDL_YV12_OVERLAY:
    case SDL_IYUV_OVERLAY:
      /* Add the two extra planes */
      overlay->pitches[1] = overlay->pitches[0] / 2;
      overlay->pitches[2] = overlay->pitches[0] / 2;
      overlay->pixels[1]  = overlay->pixels[0] + overlay->pitches[0] * overlay->h;
      overlay->pixels[2]  = overlay->pixels[1] + overlay->pitches[1] * overlay->h / 2;
      break;
    default:
      /* Only one plane, no worries */
      break;
    }

  return 0;
}

void DirectFB_UnlockYUVOverlay(_THIS, SDL_Overlay *overlay)
{
  IDirectFBSurface *surface = overlay->hwdata->surface;

  overlay->pixels[0] = overlay->pixels[1] = overlay->pixels[2] = NULL;

  surface->Unlock (surface);
}

int DirectFB_DisplayYUVOverlay(_THIS, SDL_Overlay *overlay, SDL_Rect *src, SDL_Rect *dst)
{
  DFBResult              ret;
  DFBDisplayLayerConfig  conf;
  IDirectFBDisplayLayer *primary = HIDDEN->layer;
  IDirectFBDisplayLayer *layer   = overlay->hwdata->layer;

  primary->GetConfiguration (primary, &conf);

  ret = layer->SetScreenLocation (layer,
                                  dst->x / (float) conf.width, dst->y / (float) conf.height,
                                  dst->w / (float) conf.width, dst->h / (float) conf.height );
  if (ret)
    {
      SetDirectFBerror("IDirectFBDisplayLayer::SetScreenLocation", ret);
      return -1;
    }

  return 0;
}

void DirectFB_FreeYUVOverlay(_THIS, SDL_Overlay *overlay)
{
  struct private_yuvhwdata *hwdata;

  hwdata = overlay->hwdata;
  if (hwdata)
    {
      if (hwdata->surface)
        hwdata->surface->Release (hwdata->surface);

      if (hwdata->layer)
        hwdata->layer->Release (hwdata->layer);

      free (hwdata);
    }
}

