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

	MGA CRTC2 support by Thomas Jarosch - tomj@simonv.com
	CRTC2 support is inspired by mplayer's dfbmga driver
	written by Ville Syrj��<syrjala@sci.fi>
*/
#include "SDL_config.h"

/* DirectFB video driver implementation.
*/

#include <fcntl.h>
#include <unistd.h>
#include <sys/mman.h>

#include <directfb.h>
#include <directfb_version.h>

#include "SDL_video.h"
#include "SDL_mouse.h"
#include "../SDL_sysvideo.h"
#include "../SDL_pixels_c.h"
#include "../../events/SDL_events_c.h"
#include "SDL_DirectFB_video.h"
#include "SDL_DirectFB_events.h"
#include "SDL_DirectFB_yuv.h"

/* The implementation dependent data for the window manager cursor */
struct WMcursor {
	int unused;
};


/* Initialization/Query functions */
static int DirectFB_VideoInit(_THIS, SDL_PixelFormat *vformat);
static SDL_Rect **DirectFB_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags);
static SDL_Surface *DirectFB_SetVideoMode(_THIS, SDL_Surface *current, int width, int height, int bpp, Uint32 flags);
static int DirectFB_SetColors(_THIS, int firstcolor, int ncolors,
			 SDL_Color *colors);
static void DirectFB_VideoQuit(_THIS);

/* Hardware surface functions */
static int DirectFB_AllocHWSurface(_THIS, SDL_Surface *surface);
static int DirectFB_FillHWRect(_THIS, SDL_Surface *dst, SDL_Rect *dstrect, Uint32 color);
static int DirectFB_LockHWSurface(_THIS, SDL_Surface *surface);
static void DirectFB_UnlockHWSurface(_THIS, SDL_Surface *surface);
static void DirectFB_FreeHWSurface(_THIS, SDL_Surface *surface);
static int DirectFB_CheckHWBlit(_THIS, SDL_Surface *src, SDL_Surface *dst);
static int DirectFB_HWAccelBlit(SDL_Surface *src, SDL_Rect *srcrect,
                                SDL_Surface *dst, SDL_Rect *dstrect);
static int DirectFB_SetHWColorKey(_THIS, SDL_Surface *surface, Uint32 key);
static int DirectFB_SetHWAlpha(_THIS, SDL_Surface *surface, Uint8 alpha);
static int DirectFB_FlipHWSurface(_THIS, SDL_Surface *surface);
static int DirectFB_ShowWMCursor(_THIS, WMcursor *cursor);

/* Various screen update functions available */
static void DirectFB_DirectUpdate(_THIS, int numrects, SDL_Rect *rects);
static void DirectFB_WindowedUpdate(_THIS, int numrects, SDL_Rect *rects);

/* This is the rect EnumModes2 uses */
struct DirectFBEnumRect {
	SDL_Rect r;
	struct DirectFBEnumRect* next;
};

static struct DirectFBEnumRect *enumlist = NULL;


/* DirectFB driver bootstrap functions */

static int DirectFB_Available(void)
{
  return 1;
}

static void DirectFB_DeleteDevice(SDL_VideoDevice *device)
{
  SDL_free(device->hidden);
  SDL_free(device);
}

static SDL_VideoDevice *DirectFB_CreateDevice(int devindex)
{
  SDL_VideoDevice *device;

  /* Initialize all variables that we clean on shutdown */
  device = (SDL_VideoDevice *)SDL_malloc(sizeof(SDL_VideoDevice));
  if (device)
    {
      SDL_memset (device, 0, (sizeof *device));
      device->hidden = (struct SDL_PrivateVideoData *) malloc (sizeof (*device->hidden));
    }
  if (device == NULL  ||  device->hidden == NULL)
    {
      SDL_OutOfMemory();
      if (device)
        {
          free (device);
        }
      return(0);
    }
  SDL_memset (device->hidden, 0, sizeof (*device->hidden));

  /* Set the function pointers */
  device->VideoInit = DirectFB_VideoInit;
  device->ListModes = DirectFB_ListModes;
  device->SetVideoMode = DirectFB_SetVideoMode;
  device->SetColors = DirectFB_SetColors;
  device->UpdateRects = NULL;
  device->CreateYUVOverlay = DirectFB_CreateYUVOverlay;
  device->VideoQuit = DirectFB_VideoQuit;
  device->AllocHWSurface = DirectFB_AllocHWSurface;
  device->CheckHWBlit = DirectFB_CheckHWBlit;
  device->FillHWRect = DirectFB_FillHWRect;
  device->SetHWColorKey = DirectFB_SetHWColorKey;
  device->SetHWAlpha = DirectFB_SetHWAlpha;
  device->LockHWSurface = DirectFB_LockHWSurface;
  device->UnlockHWSurface = DirectFB_UnlockHWSurface;
  device->FlipHWSurface = DirectFB_FlipHWSurface;
  device->FreeHWSurface = DirectFB_FreeHWSurface;
  device->ShowWMCursor = DirectFB_ShowWMCursor;
  device->SetCaption = NULL;
  device->SetIcon = NULL;
  device->IconifyWindow = NULL;
  device->GrabInput = NULL;
  device->GetWMInfo = NULL;
  device->InitOSKeymap = DirectFB_InitOSKeymap;
  device->PumpEvents = DirectFB_PumpEvents;

  device->free = DirectFB_DeleteDevice;

  return device;
}

VideoBootStrap DirectFB_bootstrap = {
  "directfb", "DirectFB",
  DirectFB_Available, DirectFB_CreateDevice
};

static DFBSurfacePixelFormat GetFormatForBpp (int bpp, IDirectFBDisplayLayer *layer)
{
  DFBDisplayLayerConfig dlc;
  int                   bytes = (bpp + 7) / 8;

  layer->GetConfiguration (layer, &dlc);

  if (bytes == DFB_BYTES_PER_PIXEL(dlc.pixelformat) && bytes > 1)
    return dlc.pixelformat;

  switch (bytes)
    {
    case 1:
      return DSPF_LUT8;
    case 2:
      return DSPF_RGB16;
    case 3:
      return DSPF_RGB24;
    case 4:
      return DSPF_RGB32;
    }

  return DSPF_UNKNOWN;
}

static DFBEnumerationResult EnumModesCallback (int  width,
                                               int  height,
                                               int  bpp,
                                               void *data)
{
  SDL_VideoDevice *this = (SDL_VideoDevice *)data;
  struct DirectFBEnumRect *enumrect;

  HIDDEN->nummodes++;

  if (enumlist && enumlist->r.w == width && enumlist->r.h == height)
    return DFENUM_OK;

  enumrect = SDL_calloc(1, sizeof(struct DirectFBEnumRect));
  if (!enumrect)
    {
      SDL_OutOfMemory();
      return DFENUM_CANCEL;
    }

  enumrect->r.w  = (Uint16)width;
  enumrect->r.h  = (Uint16)height;
  enumrect->next = enumlist;

  enumlist = enumrect;

  return DFENUM_OK;
}

struct private_hwdata {
  IDirectFBSurface *surface;
  IDirectFBPalette *palette;
};

void SetDirectFBerror (const char *function, DFBResult code)
{
  const char *error = DirectFBErrorString (code);

  if (error)
    SDL_SetError("%s: %s", function, error);
  else
    SDL_SetError("Unknown error code from %s", function);
}

static DFBSurfacePixelFormat SDLToDFBPixelFormat (SDL_PixelFormat *format)
{
  if (format->Rmask && format->Gmask && format->Bmask)
    {
      switch (format->BitsPerPixel)
        {
        case 8:
          return DSPF_LUT8;
          
        case 16:
          if (format->Rmask == 0xF800 &&
              format->Gmask == 0x07E0 &&
              format->Bmask == 0x001F)
            return DSPF_RGB16;
          /* fall through */
          
        case 15:
          if (format->Rmask == 0x7C00 &&
              format->Gmask == 0x03E0 &&
              format->Bmask == 0x001F)
            return DSPF_ARGB1555;
          break;
          
        case 24:
          if (format->Rmask == 0xFF0000 &&
              format->Gmask == 0x00FF00 &&
              format->Bmask == 0x0000FF)
            return DSPF_RGB24;
          break;

        case 32:
          if (format->Rmask == 0xFF0000 &&
              format->Gmask == 0x00FF00 &&
              format->Bmask == 0x0000FF)
            {
              if (format->Amask == 0xFF000000)
                return DSPF_ARGB;
              else
                return DSPF_RGB32;
            }
          break;
        }
    }
  else
    {
      switch (format->BitsPerPixel)
	{
        case 8:
          return DSPF_LUT8;
	case 15:
	  return DSPF_ARGB1555;
	case 16:
	  return DSPF_RGB16;
	case 24:
	  return DSPF_RGB24;
	case 32:
	  return DSPF_RGB32;
	}
    }

  return DSPF_UNKNOWN;
}

static SDL_Palette *AllocatePalette(int size)
{
  SDL_Palette *palette;
  SDL_Color   *colors;

  palette = SDL_calloc (1, sizeof(SDL_Palette));
  if (!palette)
    {
      SDL_OutOfMemory();
      return NULL;
    }

  colors = SDL_calloc (size, sizeof(SDL_Color));
  if (!colors)
    {
      SDL_OutOfMemory();
      return NULL;
    }

  palette->ncolors = size;
  palette->colors  = colors;

  return palette;
}

static int DFBToSDLPixelFormat (DFBSurfacePixelFormat pixelformat, SDL_PixelFormat *format)
{
  format->Amask = format->Rmask = format->Gmask = format->Bmask = 0;
  format->BitsPerPixel = format->BytesPerPixel = 0;

  switch (pixelformat)
    {
    case DSPF_A8:
      format->Amask = 0x000000FF;
      break;

    case DSPF_ARGB1555:
      format->Rmask = 0x00007C00;
      format->Gmask = 0x000003E0;
      format->Bmask = 0x0000001F;
      break;

    case DSPF_RGB16:
      format->Rmask = 0x0000F800;
      format->Gmask = 0x000007E0;
      format->Bmask = 0x0000001F;
      break;

    case DSPF_ARGB:
      format->Amask = 0; /* apps don't seem to like that:  0xFF000000; */
      /* fall through */
    case DSPF_RGB24:
    case DSPF_RGB32:
      format->Rmask = 0x00FF0000;
      format->Gmask = 0x0000FF00;
      format->Bmask = 0x000000FF;
      break;

    case DSPF_LUT8:
      format->Rmask = 0x000000FF;
      format->Gmask = 0x000000FF;
      format->Bmask = 0x000000FF;

      if (!format->palette)
        format->palette = AllocatePalette(256);
      break;

    default:
      fprintf (stderr, "SDL_DirectFB: Unsupported pixelformat (0x%08x)!\n", pixelformat);
      return -1;
    }

  format->BitsPerPixel  = DFB_BYTES_PER_PIXEL(pixelformat) * 8;
  format->BytesPerPixel = DFB_BYTES_PER_PIXEL(pixelformat);

  return 0;
}


int DirectFB_VideoInit(_THIS, SDL_PixelFormat *vformat)
{
  int                      i;
  DFBResult                ret;
#if (DIRECTFB_MAJOR_VERSION == 0) && (DIRECTFB_MINOR_VERSION == 9) && (DIRECTFB_MICRO_VERSION < 23)
  DFBCardCapabilities      caps;
#else
  DFBGraphicsDeviceDescription caps;
#endif
  DFBDisplayLayerConfig    dlc;
  struct DirectFBEnumRect *rect;
  IDirectFB               *dfb    = NULL;
  IDirectFBDisplayLayer   *layer  = NULL;
  IDirectFBEventBuffer    *events = NULL;

  HIDDEN->c2layer = NULL, HIDDEN->c2frame = NULL;
  HIDDEN->enable_mga_crtc2 = 0;
  HIDDEN->mga_crtc2_stretch_overscan = 1;

  ret = DirectFBInit (NULL, NULL);
  if (ret)
    {
      SetDirectFBerror ("DirectFBInit", ret);
      goto error;
    }

  ret = DirectFBCreate (&dfb);
  if (ret)
    {
      SetDirectFBerror ("DirectFBCreate", ret);
      goto error;
    }

  ret = dfb->GetDisplayLayer (dfb, DLID_PRIMARY, &layer);
  if (ret)
    {
      SetDirectFBerror ("dfb->GetDisplayLayer", ret);
      goto error;
    }

  ret = dfb->CreateInputEventBuffer (dfb, DICAPS_ALL, DFB_FALSE, &events);
  if (ret)
    {
      SetDirectFBerror ("dfb->CreateEventBuffer", ret);
      goto error;
    }
  
  layer->EnableCursor (layer, 1);

  /* Query layer configuration to determine the current mode and pixelformat */
  layer->GetConfiguration (layer, &dlc);

  /* If current format is not supported use LUT8 as the default */
  if (DFBToSDLPixelFormat (dlc.pixelformat, vformat))
    DFBToSDLPixelFormat (DSPF_LUT8, vformat);

  /* Enumerate the available fullscreen modes */
  ret = dfb->EnumVideoModes (dfb, EnumModesCallback, this);
  if (ret)
    {
      SetDirectFBerror ("dfb->EnumVideoModes", ret);
      goto error;
    }

  HIDDEN->modelist = SDL_calloc (HIDDEN->nummodes + 1, sizeof(SDL_Rect *));
  if (!HIDDEN->modelist)
    {
      SDL_OutOfMemory();
      goto error;
    }

  for (i = 0, rect = enumlist; rect; ++i, rect = rect->next )
    {
      HIDDEN->modelist[i] = &rect->r;
    }

  HIDDEN->modelist[i] = NULL;


  /* Query card capabilities to get the video memory size */
#if (DIRECTFB_MAJOR_VERSION == 0) && (DIRECTFB_MINOR_VERSION == 9) && (DIRECTFB_MICRO_VERSION < 23)
  dfb->GetCardCapabilities (dfb, &caps);
#else
  dfb->GetDeviceDescription (dfb, &caps);
#endif

  this->info.wm_available = 1;
  this->info.hw_available = 1;
  this->info.blit_hw      = 1;
  this->info.blit_hw_CC   = 1;
  this->info.blit_hw_A    = 1;
  this->info.blit_fill    = 1;
  this->info.video_mem    = caps.video_memory / 1024;
  this->info.current_w    = dlc.width;  
  this->info.current_h    = dlc.height;

  HIDDEN->initialized = 1;
  HIDDEN->dfb         = dfb;
  HIDDEN->layer       = layer;
  HIDDEN->eventbuffer = events;

  if (SDL_getenv("SDL_DIRECTFB_MGA_CRTC2") != NULL)
    HIDDEN->enable_mga_crtc2 = 1;
  
  if (HIDDEN->enable_mga_crtc2)
    {
      DFBDisplayLayerConfig      dlc;
      DFBDisplayLayerConfigFlags failed;

      ret = dfb->GetDisplayLayer (dfb, 2, &HIDDEN->c2layer);
      if (ret)
        {
          SetDirectFBerror ("dfb->GetDisplayLayer(CRTC2)", ret);
          goto error;
        }

      ret = HIDDEN->layer->SetCooperativeLevel(HIDDEN->layer, DLSCL_EXCLUSIVE);
      if (ret)
        {
          SetDirectFBerror ("layer->SetCooperativeLevel(CRTC2, EXCLUSIVE)", ret);
          goto error;
        }
 
      ret = HIDDEN->c2layer->SetCooperativeLevel(HIDDEN->c2layer, DLSCL_EXCLUSIVE);
      if (ret)
        {
          SetDirectFBerror ("c2layer->SetCooperativeLevel(CRTC2, EXCLUSIVE)", ret);
          goto error;
        }

      HIDDEN->c2layer->SetOpacity(HIDDEN->c2layer, 0x0);

      /* Init the surface here as it got a fixed size */
      dlc.flags      = DLCONF_PIXELFORMAT | DLCONF_BUFFERMODE;
      dlc.buffermode = DLBM_BACKVIDEO;
      dlc.pixelformat = DSPF_RGB32;
      
      ret = HIDDEN->c2layer->TestConfiguration( HIDDEN->c2layer, &dlc, &failed );
      if (ret)
        {
          SetDirectFBerror ("c2layer->TestConfiguration", ret);
          goto error;
        }
    
      ret = HIDDEN->c2layer->SetConfiguration( HIDDEN->c2layer, &dlc );
      if (ret)
        {
          SetDirectFBerror ("c2layer->SetConfiguration", ret);
          goto error;
        }
    
      ret = HIDDEN->c2layer->GetSurface( HIDDEN->c2layer, &HIDDEN->c2frame );
      if (ret)
        {
          SetDirectFBerror ("c2layer->GetSurface", ret);
          goto error;
        }

      HIDDEN->c2framesize.x = 0;
      HIDDEN->c2framesize.y = 0;
      HIDDEN->c2frame->GetSize( HIDDEN->c2frame, &HIDDEN->c2framesize.w, &HIDDEN->c2framesize.h);

      HIDDEN->c2frame->SetBlittingFlags( HIDDEN->c2frame, DSBLIT_NOFX );
      HIDDEN->c2frame->SetColor( HIDDEN->c2frame, 0, 0, 0, 0xff );
    
      /* Clear CRTC2 */
      HIDDEN->c2frame->Clear(HIDDEN->c2frame, 0, 0, 0, 0xff );
      HIDDEN->c2frame->Flip(HIDDEN->c2frame, NULL, 0 );
      HIDDEN->c2frame->Clear(HIDDEN->c2frame, 0, 0, 0, 0xff );
      HIDDEN->c2frame->Flip(HIDDEN->c2frame, NULL, 0 );
      HIDDEN->c2frame->Clear(HIDDEN->c2frame, 0, 0, 0, 0xff );

      HIDDEN->c2layer->SetOpacity(HIDDEN->c2layer, 0xFF );
    
      /* Check if overscan is possibly set */
      if (SDL_getenv("SDL_DIRECTFB_MGA_OVERSCAN") != NULL)
        {
	    float overscan = 0;
	    if (SDL_sscanf(SDL_getenv("SDL_DIRECTFB_MGA_OVERSCAN"), "%f", &overscan) == 1)
               if (overscan > 0 && overscan < 2)
		  HIDDEN->mga_crtc2_stretch_overscan = overscan;
	}

      #ifdef DIRECTFB_CRTC2_DEBUG
      printf("CRTC2 overscan: %f\n", HIDDEN->mga_crtc2_stretch_overscan);
      #endif
    }

  return 0;

 error:
  if (events)
    events->Release (events);
  
  if (HIDDEN->c2frame)
    HIDDEN->c2frame->Release (HIDDEN->c2frame);

  if (HIDDEN->c2layer)
    HIDDEN->c2layer->Release (HIDDEN->c2layer);

  if (layer)
    layer->Release (layer);

  if (dfb)
    dfb->Release (dfb);

  return -1;
}

static SDL_Rect **DirectFB_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags)
{
  if (flags & SDL_FULLSCREEN)
    return HIDDEN->modelist;
  else
    if (SDLToDFBPixelFormat (format) != DSPF_UNKNOWN)
      return (SDL_Rect**) -1;

  return NULL;
}

static SDL_Surface *DirectFB_SetVideoMode(_THIS, SDL_Surface *current, int width, int height, int bpp, Uint32 flags)
{
  DFBResult              ret;
  DFBSurfaceDescription  dsc;
  DFBSurfacePixelFormat  pixelformat;
  IDirectFBSurface      *surface;

  fprintf (stderr, "SDL DirectFB_SetVideoMode: %dx%d@%d, flags: 0x%08x\n",
           width, height, bpp, flags);

  flags |= SDL_FULLSCREEN;

  /* Release previous primary surface */
  if (current->hwdata && current->hwdata->surface)
    {
      current->hwdata->surface->Release (current->hwdata->surface);
      current->hwdata->surface = NULL;

      /* And its palette if present */
      if (current->hwdata->palette)
        {
          current->hwdata->palette->Release (current->hwdata->palette);
          current->hwdata->palette = NULL;
        }
    }
  else if (!current->hwdata)
    {
      /* Allocate the hardware acceleration data */
      current->hwdata = (struct private_hwdata *) SDL_calloc (1, sizeof(*current->hwdata));
      if (!current->hwdata)
        {
          SDL_OutOfMemory();
          return NULL;
	}
    }

  /* Set cooperative level depending on flag SDL_FULLSCREEN */
  if (flags & SDL_FULLSCREEN)
    {
      ret = HIDDEN->dfb->SetCooperativeLevel (HIDDEN->dfb, DFSCL_FULLSCREEN);
      if (ret && !HIDDEN->enable_mga_crtc2)
        {
          DirectFBError ("dfb->SetCooperativeLevel", ret);
          flags &= ~SDL_FULLSCREEN;
        }
    }
  else
    HIDDEN->dfb->SetCooperativeLevel (HIDDEN->dfb, DFSCL_NORMAL);

  /* Set video mode */
  ret = HIDDEN->dfb->SetVideoMode (HIDDEN->dfb, width, height, bpp);
  if (ret)
    {
      if (flags & SDL_FULLSCREEN)
        {
          flags &= ~SDL_FULLSCREEN;
          HIDDEN->dfb->SetCooperativeLevel (HIDDEN->dfb, DFSCL_NORMAL);
          ret = HIDDEN->dfb->SetVideoMode (HIDDEN->dfb, width, height, bpp);
        }

      if (ret)
        {
          SetDirectFBerror ("dfb->SetVideoMode", ret);
          return NULL;
        }
    }

  /* Create primary surface */
  dsc.flags       = DSDESC_CAPS | DSDESC_PIXELFORMAT;
  dsc.caps        = DSCAPS_PRIMARY | ((flags & SDL_DOUBLEBUF) ? DSCAPS_FLIPPING : 0);
  dsc.pixelformat = GetFormatForBpp (bpp, HIDDEN->layer);

  ret = HIDDEN->dfb->CreateSurface (HIDDEN->dfb, &dsc, &surface);
  if (ret && (flags & SDL_DOUBLEBUF))
    {
      /* Try without double buffering */
      dsc.caps &= ~DSCAPS_FLIPPING;
      ret = HIDDEN->dfb->CreateSurface (HIDDEN->dfb, &dsc, &surface);
    }
  if (ret)
    {
      SetDirectFBerror ("dfb->CreateSurface", ret);
      return NULL;
    }

  current->w     = width;
  current->h     = height;
  current->flags = SDL_HWSURFACE | SDL_PREALLOC;

  if (flags & SDL_FULLSCREEN)
    {
      current->flags |= SDL_FULLSCREEN;
      this->UpdateRects = DirectFB_DirectUpdate;
    }
  else
    this->UpdateRects = DirectFB_WindowedUpdate;

  if (dsc.caps & DSCAPS_FLIPPING)
    current->flags |= SDL_DOUBLEBUF;

  surface->GetPixelFormat (surface, &pixelformat);

  DFBToSDLPixelFormat (pixelformat, current->format);

  /* Get the surface palette (if supported) */
  if (DFB_PIXELFORMAT_IS_INDEXED( pixelformat ))
    {
      surface->GetPalette (surface, &current->hwdata->palette);

      current->flags |= SDL_HWPALETTE;
    }

  current->hwdata->surface = surface;

  /* MGA CRTC2 stuff */
  if (HIDDEN->enable_mga_crtc2)
    {
      /* no stretching if c2ssize == c2framesize */
      HIDDEN->c2ssize.x = 0, HIDDEN->c2ssize.y = 0;
      HIDDEN->c2ssize.w = width;
      HIDDEN->c2ssize.h = height;

      HIDDEN->c2dsize.x = 0, HIDDEN->c2dsize.y = 0;
      HIDDEN->c2dsize.w = width;
      HIDDEN->c2dsize.h = height;

      HIDDEN->mga_crtc2_stretch = 0;

      if (SDL_getenv("SDL_DIRECTFB_MGA_STRETCH") != NULL)
        {
	    /* Normally assume a picture aspect ratio of 4:3 */
	    int zoom_aspect_x = 4, zoom_aspect_y = 3, i, j;

	    for (i = 1; i < 20; i++)
	      {
		for (j = 1; j < 10; j++)
		  {
		    if ((float)width/(float)i*(float)j == height) 
		      {
			zoom_aspect_x = i;
			zoom_aspect_y = j;
			
			/* break the loop */
			i = 21;
			break;
		      }
		  }
	      }
	
            #ifdef DIRECTFB_CRTC2_DEBUG
            printf("Source resolution: X: %d, Y: %d, Aspect ratio: %d:%d\n", width, height, zoom_aspect_x, zoom_aspect_y);
            printf("CRTC2 resolution: X: %d, Y: %d\n", HIDDEN->c2framesize.w, HIDDEN->c2framesize.h);
            #endif
	
          /* don't stretch only slightly smaller/larger images */
          if ((float)width < (float)HIDDEN->c2framesize.w*0.95 || (float)height < (float)HIDDEN->c2framesize.h*0.95)
            {
              while ((float)HIDDEN->c2dsize.w < (float)HIDDEN->c2framesize.w*HIDDEN->mga_crtc2_stretch_overscan && (float)HIDDEN->c2dsize.h < (float)HIDDEN->c2framesize.h*HIDDEN->mga_crtc2_stretch_overscan)
                {
                   HIDDEN->c2dsize.w+=zoom_aspect_x;
                   HIDDEN->c2dsize.h+=zoom_aspect_y;
                }

              /* one step down */
                HIDDEN->c2dsize.w-=zoom_aspect_x;
                HIDDEN->c2dsize.h-=zoom_aspect_y;

              #ifdef DIRECTFB_CRTC2_DEBUG
              printf("Stretched resolution: X: %d, Y: %d\n", HIDDEN->c2dsize.w, HIDDEN->c2dsize.h);
              #endif

              HIDDEN->mga_crtc2_stretch = 1;
            } 
          else if ((float)width > (float)HIDDEN->c2framesize.w*0.95 || (float)height > (float)HIDDEN->c2framesize.h*0.95)
            {
               while ((float)HIDDEN->c2dsize.w > (float)HIDDEN->c2framesize.w*HIDDEN->mga_crtc2_stretch_overscan || (float)HIDDEN->c2dsize.h > (float)HIDDEN->c2framesize.h*HIDDEN->mga_crtc2_stretch_overscan)
                {
              HIDDEN->c2dsize.w-=zoom_aspect_x;
              HIDDEN->c2dsize.h-=zoom_aspect_y;
                }
              
              #ifdef DIRECTFB_CRTC2_DEBUG
              printf("Down-Stretched resolution: X: %d, Y: %d\n", HIDDEN->c2dsize.w, HIDDEN->c2dsize.h);
              #endif

              HIDDEN->mga_crtc2_stretch = 1;
             } else {
          #ifdef DIRECTFB_CRTC2_DEBUG
          printf("Not stretching image\n");
          #endif
        }

      /* Panning */
      if (HIDDEN->c2framesize.w > HIDDEN->c2dsize.w)
        HIDDEN->c2dsize.x = (HIDDEN->c2framesize.w - HIDDEN->c2dsize.w)  / 2;
      else
        HIDDEN->c2dsize.x = (HIDDEN->c2dsize.w - HIDDEN->c2framesize.w)  / 2;

      if (HIDDEN->c2framesize.h > HIDDEN->c2dsize.h)
        HIDDEN->c2dsize.y = (HIDDEN->c2framesize.h - HIDDEN->c2dsize.h)  / 2;
      else
        HIDDEN->c2dsize.y = (HIDDEN->c2dsize.h - HIDDEN->c2framesize.h)  / 2;

      #ifdef DIRECTFB_CRTC2_DEBUG
    printf("CRTC2 position X: %d, Y: %d\n", HIDDEN->c2dsize.x, HIDDEN->c2dsize.y);
      #endif
   }
  }

  return current;
}

static int DirectFB_AllocHWSurface(_THIS, SDL_Surface *surface)
{
  DFBResult             ret;
  DFBSurfaceDescription dsc;

  /*  fprintf(stderr, "SDL: DirectFB_AllocHWSurface (%dx%d@%d, flags: 0x%08x)\n",
      surface->w, surface->h, surface->format->BitsPerPixel, surface->flags);*/

  if (surface->w < 8 || surface->h < 8)
    return -1;

  /* fill surface description */
  dsc.flags  = DSDESC_WIDTH | DSDESC_HEIGHT | DSDESC_PIXELFORMAT | DSDESC_CAPS;
  dsc.width  = surface->w;
  dsc.height = surface->h;
  dsc.caps   = (surface->flags & SDL_DOUBLEBUF) ? DSCAPS_FLIPPING : 0;

  /* find the right pixelformat */
  dsc.pixelformat = SDLToDFBPixelFormat (surface->format);
  if (dsc.pixelformat == DSPF_UNKNOWN)
    return -1;

  /* Allocate the hardware acceleration data */
  surface->hwdata = (struct private_hwdata *) SDL_calloc (1, sizeof(*surface->hwdata));
  if (surface->hwdata == NULL)
    {
      SDL_OutOfMemory();
      return -1;
    }

  /* Create the surface */
  ret = HIDDEN->dfb->CreateSurface (HIDDEN->dfb, &dsc, &surface->hwdata->surface);
  if (ret)
    {
      SetDirectFBerror ("dfb->CreateSurface", ret);
      free (surface->hwdata);
      surface->hwdata = NULL;
      return -1;
    }

  surface->flags |= SDL_HWSURFACE | SDL_PREALLOC;

  return 0;
}

static void DirectFB_FreeHWSurface(_THIS, SDL_Surface *surface)
{
  if (surface->hwdata && HIDDEN->initialized)
    {
      surface->hwdata->surface->Release (surface->hwdata->surface);
      free (surface->hwdata);
      surface->hwdata = NULL;
    }
}

static int DirectFB_CheckHWBlit(_THIS, SDL_Surface *src, SDL_Surface *dst)
{
  /*  fprintf(stderr, "SDL: DirectFB_CheckHWBlit (src->hwdata: %p, dst->hwdata: %p)\n",
      src->hwdata, dst->hwdata);*/

  if (!src->hwdata || !dst->hwdata)
    return 0;

  src->flags |= SDL_HWACCEL;
  src->map->hw_blit = DirectFB_HWAccelBlit;

  return 1;
}

static int DirectFB_HWAccelBlit(SDL_Surface *src, SDL_Rect *srcrect,
                                SDL_Surface *dst, SDL_Rect *dstrect)
{
  DFBSurfaceBlittingFlags flags = DSBLIT_NOFX;

  DFBRectangle sr = { srcrect->x, srcrect->y, srcrect->w, srcrect->h };
  DFBRectangle dr = { dstrect->x, dstrect->y, dstrect->w, dstrect->h };

  IDirectFBSurface *surface = dst->hwdata->surface;

  if (src->flags & SDL_SRCCOLORKEY)
    {
      flags |= DSBLIT_SRC_COLORKEY;
      DirectFB_SetHWColorKey (NULL, src, src->format->colorkey);
    }

  if (src->flags & SDL_SRCALPHA)
    {
      flags |= DSBLIT_BLEND_COLORALPHA;
      surface->SetColor (surface, 0xff, 0xff, 0xff, src->format->alpha);
    }

  surface->SetBlittingFlags (surface, flags);

  if (sr.w == dr.w && sr.h == dr.h)
    surface->Blit (surface, src->hwdata->surface, &sr, dr.x, dr.y);
  else
    surface->StretchBlit (surface, src->hwdata->surface, &sr, &dr);

  return 0;
}

static int DirectFB_FillHWRect(_THIS, SDL_Surface *dst, SDL_Rect *dstrect, Uint32 color)
{
  SDL_PixelFormat  *fmt     = dst->format;
  IDirectFBSurface *surface = dst->hwdata->surface;

  /* ugly */
  surface->SetColor (surface,
                     (color & fmt->Rmask) >> (fmt->Rshift - fmt->Rloss),
                     (color & fmt->Gmask) >> (fmt->Gshift - fmt->Gloss),
                     (color & fmt->Bmask) << (fmt->Bloss - fmt->Bshift), 0xFF);
  surface->FillRectangle (surface, dstrect->x, dstrect->y, dstrect->w, dstrect->h);

  return 0;
}

static int DirectFB_SetHWColorKey(_THIS, SDL_Surface *src, Uint32 key)
{
  SDL_PixelFormat  *fmt     = src->format;
  IDirectFBSurface *surface = src->hwdata->surface;

  if (fmt->BitsPerPixel == 8)
    surface->SetSrcColorKeyIndex (surface, key);
  else
    /* ugly */
    surface->SetSrcColorKey (surface,
                             (key & fmt->Rmask) >> (fmt->Rshift - fmt->Rloss),
                             (key & fmt->Gmask) >> (fmt->Gshift - fmt->Gloss),
                             (key & fmt->Bmask) << (fmt->Bloss - fmt->Bshift));

  return 0;
}

static int DirectFB_SetHWAlpha(_THIS, SDL_Surface *surface, Uint8 alpha)
{
  return 0;
}

static int DirectFB_FlipHWSurface(_THIS, SDL_Surface *surface)
{
  if (HIDDEN->enable_mga_crtc2)
    {
       int rtn = surface->hwdata->surface->Flip (surface->hwdata->surface, NULL, 0);
       if (HIDDEN->mga_crtc2_stretch)
         HIDDEN->c2frame->StretchBlit(HIDDEN->c2frame, surface->hwdata->surface, &HIDDEN->c2ssize, &HIDDEN->c2dsize);
       else
         HIDDEN->c2frame->Blit(HIDDEN->c2frame, surface->hwdata->surface, NULL, HIDDEN->c2dsize.x, HIDDEN->c2dsize.y);
     
       HIDDEN->c2frame->Flip(HIDDEN->c2frame, NULL, DSFLIP_WAITFORSYNC);
       return rtn;
    } 
  else 
     return surface->hwdata->surface->Flip (surface->hwdata->surface, NULL, DSFLIP_WAITFORSYNC);
}

static int DirectFB_LockHWSurface(_THIS, SDL_Surface *surface)
{
  DFBResult  ret;
  void      *data;
  int        pitch;

  ret = surface->hwdata->surface->Lock (surface->hwdata->surface,
                                        DSLF_WRITE, &data, &pitch);
  if (ret)
    {
      SetDirectFBerror ("surface->Lock", ret);
      return -1;
    }

  surface->pixels = data;
  surface->pitch  = pitch;

  return 0;
}

static void DirectFB_UnlockHWSurface(_THIS, SDL_Surface *surface)
{
  surface->hwdata->surface->Unlock (surface->hwdata->surface);
  surface->pixels = NULL;
}

static void DirectFB_DirectUpdate(_THIS, int numrects, SDL_Rect *rects)
{
  if (HIDDEN->enable_mga_crtc2)
    {
       if (HIDDEN->mga_crtc2_stretch)
         HIDDEN->c2frame->StretchBlit(HIDDEN->c2frame, this->screen->hwdata->surface, &HIDDEN->c2ssize, &HIDDEN->c2dsize); 
       else
         HIDDEN->c2frame->Blit(HIDDEN->c2frame, this->screen->hwdata->surface, NULL, HIDDEN->c2dsize.x, HIDDEN->c2dsize.y); 

       HIDDEN->c2frame->Flip(HIDDEN->c2frame, NULL, DSFLIP_WAITFORSYNC);
    }
}

static void DirectFB_WindowedUpdate(_THIS, int numrects, SDL_Rect *rects)
{
  DFBRegion         region;
  int               i;
  int               region_valid = 0;
  IDirectFBSurface *surface = this->screen->hwdata->surface;

  for (i=0; i<numrects; ++i)
    {
      int x2, y2;

      if ( ! rects[i].w ) /* Clipped? */
        continue;

      x2 = rects[i].x + rects[i].w - 1;
      y2 = rects[i].y + rects[i].h - 1;

      if (region_valid)
        {
          if (rects[i].x < region.x1)
            region.x1 = rects[i].x;

          if (rects[i].y < region.y1)
            region.y1 = rects[i].y;

          if (x2 > region.x2)
            region.x2 = x2;

          if (y2 > region.y2)
            region.y2 = y2;
        }
      else
        {
            region.x1 = rects[i].x;
            region.y1 = rects[i].y;
            region.x2 = x2;
            region.y2 = y2;

            region_valid = 1;
        }
    }

  if (region_valid)
    {
      if (HIDDEN->enable_mga_crtc2)
        {
          if (HIDDEN->mga_crtc2_stretch)
            HIDDEN->c2frame->StretchBlit(HIDDEN->c2frame, surface, &HIDDEN->c2ssize, &HIDDEN->c2dsize);
          else
            HIDDEN->c2frame->Blit(HIDDEN->c2frame, surface, NULL, HIDDEN->c2dsize.x, HIDDEN->c2dsize.y); 
      
          HIDDEN->c2frame->Flip(HIDDEN->c2frame, NULL, DSFLIP_WAITFORSYNC);
        }
      else 
        surface->Flip (surface, &region, DSFLIP_WAITFORSYNC);
    }
}

int DirectFB_SetColors(_THIS, int firstcolor, int ncolors, SDL_Color *colors)
{
  IDirectFBPalette *palette = this->screen->hwdata->palette;

  if (!palette)
    return 0;

  if (firstcolor > 255)
    return 0;

  if (firstcolor + ncolors > 256)
    ncolors = 256 - firstcolor;

  if (ncolors > 0)
    {
      int      i;
      DFBColor entries[ncolors];

      for (i=0; i<ncolors; i++)
        {
          entries[i].a = 0xff;
          entries[i].r = colors[i].r;
          entries[i].g = colors[i].g;
          entries[i].b = colors[i].b;
        }

      palette->SetEntries (palette, entries, ncolors, firstcolor);
    }

  return 1;
}
	
void DirectFB_VideoQuit(_THIS)
{
  struct DirectFBEnumRect *rect    = enumlist;

  if (this->screen && this->screen->hwdata)
    {
      IDirectFBSurface        *surface = this->screen->hwdata->surface;
      IDirectFBPalette        *palette = this->screen->hwdata->palette;

      if (palette)
        palette->Release (palette);

      if (surface)
        surface->Release (surface);

      this->screen->hwdata->surface = NULL;
      this->screen->hwdata->palette = NULL;
    }

  if (HIDDEN->c2frame)
    {
      HIDDEN->c2frame->Release (HIDDEN->c2frame);
      HIDDEN->c2frame = NULL;
    }

  if (HIDDEN->eventbuffer)
    {
      HIDDEN->eventbuffer->Release (HIDDEN->eventbuffer);
      HIDDEN->eventbuffer = NULL;
    }

  if (HIDDEN->c2layer)
    {
      HIDDEN->c2layer->Release (HIDDEN->c2layer);
      HIDDEN->c2layer = NULL;
    }

  if (HIDDEN->layer)
    {
      HIDDEN->layer->Release (HIDDEN->layer);
      HIDDEN->layer = NULL;
    }

  if (HIDDEN->dfb)
    {
      HIDDEN->dfb->Release (HIDDEN->dfb);
      HIDDEN->dfb = NULL;
    }

  /* Free video mode list */
  if (HIDDEN->modelist)
    {
      free (HIDDEN->modelist);
      HIDDEN->modelist = NULL;
    }

  /* Free mode enumeration list */
  while (rect)
    {
      struct DirectFBEnumRect *next = rect->next;
      free (rect);
      rect = next;
    }
  enumlist = NULL;

  HIDDEN->initialized = 0;
}


int DirectFB_ShowWMCursor(_THIS, WMcursor *cursor)
{
  /* We can only hide or show the default cursor */
  if ( cursor == NULL )
    {
      HIDDEN->layer->SetCursorOpacity(HIDDEN->layer, 0x00);
    }
    else
    {
      HIDDEN->layer->SetCursorOpacity(HIDDEN->layer, 0xFF);
    }
  return 1;
}

void DirectFB_FinalQuit(void) 
{
}
