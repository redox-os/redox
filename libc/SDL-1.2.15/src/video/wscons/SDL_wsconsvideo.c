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

#include <sys/time.h>
#include <sys/mman.h>
#include <sys/ioctl.h>
#include <dev/wscons/wsdisplay_usl_io.h>
#include <fcntl.h>
#include <unistd.h>
#include <errno.h>

#include "SDL_video.h"
#include "SDL_mouse.h"
#include "../SDL_sysvideo.h"
#include "../SDL_pixels_c.h"
#include "../../events/SDL_events_c.h"

#include "SDL_wsconsvideo.h"
#include "SDL_wsconsevents_c.h"
#include "SDL_wsconsmouse_c.h"

#define WSCONSVID_DRIVER_NAME "wscons"
enum {
  WSCONS_ROTATE_NONE = 0,
  WSCONS_ROTATE_CCW = 90,
  WSCONS_ROTATE_UD = 180,
  WSCONS_ROTATE_CW = 270
};

#define min(a,b) ((a)<(b)?(a):(b))

/* Initialization/Query functions */
static int WSCONS_VideoInit(_THIS, SDL_PixelFormat *vformat);
static SDL_Rect **WSCONS_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags);
static SDL_Surface *WSCONS_SetVideoMode(_THIS, SDL_Surface *current, int width, int height, int bpp, Uint32 flags);
static int WSCONS_SetColors(_THIS, int firstcolor, int ncolors, SDL_Color *colors);
static void WSCONS_VideoQuit(_THIS);

/* Hardware surface functions */
static int WSCONS_AllocHWSurface(_THIS, SDL_Surface *surface);
static int WSCONS_LockHWSurface(_THIS, SDL_Surface *surface);
static void WSCONS_UnlockHWSurface(_THIS, SDL_Surface *surface);
static void WSCONS_FreeHWSurface(_THIS, SDL_Surface *surface);

/* etc. */
static WSCONS_bitBlit WSCONS_blit16;
static WSCONS_bitBlit WSCONS_blit16blocked;
static void WSCONS_UpdateRects(_THIS, int numrects, SDL_Rect *rects);

void WSCONS_ReportError(char *fmt, ...)
{
  char message[200];
  va_list vaArgs;
  
  message[199] = '\0';
  
  va_start(vaArgs, fmt);
  vsnprintf(message, 199, fmt, vaArgs);
  va_end(vaArgs);

  SDL_SetError(message); 
  fprintf(stderr, "WSCONS error: %s\n", message);
}

/* WSCONS driver bootstrap functions */

static int WSCONS_Available(void)
{
  return 1;
}

static void WSCONS_DeleteDevice(SDL_VideoDevice *device)
{
  SDL_free(device->hidden);
  SDL_free(device);
}

static SDL_VideoDevice *WSCONS_CreateDevice(int devindex)
{
  SDL_VideoDevice *device;
  
  /* Initialize all variables that we clean on shutdown */
  device = (SDL_VideoDevice *)SDL_malloc(sizeof(SDL_VideoDevice));
  if (device == NULL) {
    SDL_OutOfMemory();
    return 0;
  }
  SDL_memset(device, 0, (sizeof *device));
  device->hidden = 
    (struct SDL_PrivateVideoData *)SDL_malloc((sizeof *device->hidden));
  if (device->hidden == NULL) {
    SDL_OutOfMemory();
    SDL_free(device);
    return(0);
  }
  SDL_memset(device->hidden, 0, (sizeof *device->hidden));
  device->hidden->fd = -1;
  
  /* Set the function pointers */
  device->VideoInit = WSCONS_VideoInit;
  device->ListModes = WSCONS_ListModes;
  device->SetVideoMode = WSCONS_SetVideoMode;
  device->SetColors = WSCONS_SetColors;
  device->UpdateRects = WSCONS_UpdateRects;
  device->VideoQuit = WSCONS_VideoQuit;
  device->AllocHWSurface = WSCONS_AllocHWSurface;
  device->LockHWSurface = WSCONS_LockHWSurface;
  device->UnlockHWSurface = WSCONS_UnlockHWSurface;
  device->FreeHWSurface = WSCONS_FreeHWSurface;
  device->InitOSKeymap = WSCONS_InitOSKeymap;
  device->PumpEvents = WSCONS_PumpEvents;
  device->free = WSCONS_DeleteDevice;
  
  return device;
}

VideoBootStrap WSCONS_bootstrap = {
  WSCONSVID_DRIVER_NAME,
  "SDL wscons video driver",
  WSCONS_Available,
  WSCONS_CreateDevice
};

#define WSCONSDEV_FORMAT "/dev/ttyC%01x"

int WSCONS_VideoInit(_THIS, SDL_PixelFormat *vformat)
{
  char devnamebuf[30];
  char *devname;
  char *rotation;
  int wstype;
  int wsmode = WSDISPLAYIO_MODE_DUMBFB;
  size_t len, mapsize;
  int pagemask;
  int width, height;
  
  devname = SDL_getenv("SDL_WSCONSDEV");
  if (devname == NULL) {
    int activeVT;
    if (ioctl(STDIN_FILENO, VT_GETACTIVE, &activeVT) == -1) {
      WSCONS_ReportError("Unable to determine active terminal: %s", 
			 strerror(errno));
      return -1;
    }
    SDL_snprintf(devnamebuf, sizeof(devnamebuf), WSCONSDEV_FORMAT, activeVT - 1);
    devname = devnamebuf;
  }

  private->fd = open(devname, O_RDWR | O_NONBLOCK, 0);
  if (private->fd == -1) {
    WSCONS_ReportError("open %s: %s", devname, strerror(errno));
    return -1;
  }
  if (ioctl(private->fd, WSDISPLAYIO_GINFO, &private->info) == -1) {
    WSCONS_ReportError("ioctl WSDISPLAY_GINFO: %s", strerror(errno));
    return -1;
  }
  if (ioctl(private->fd, WSDISPLAYIO_GTYPE, &wstype) == -1) {
    WSCONS_ReportError("ioctl WSDISPLAY_GTYPE: %s", strerror(errno));
    return -1;
  }
  if (ioctl(private->fd, WSDISPLAYIO_LINEBYTES, &private->physlinebytes) == -1) {
    WSCONS_ReportError("ioctl WSDISPLAYIO_LINEBYTES: %s", strerror(errno));
    return -1;
  }
  if (private->info.depth > 8) {
    if (wstype == WSDISPLAY_TYPE_SUN24 ||
	wstype == WSDISPLAY_TYPE_SUNCG12 ||
	wstype == WSDISPLAY_TYPE_SUNCG14 ||
	wstype == WSDISPLAY_TYPE_SUNTCX ||
	wstype == WSDISPLAY_TYPE_SUNFFB) {
      private->redMask = 0x0000ff;
      private->greenMask = 0x00ff00;
      private->blueMask = 0xff0000;
#ifdef WSDISPLAY_TYPE_PXALCD
    } else if (wstype == WSDISPLAY_TYPE_PXALCD) {
      private->redMask = 0x1f << 11;
      private->greenMask = 0x3f << 5;
      private->blueMask = 0x1f;
#endif
    } else {
      WSCONS_ReportError("Unknown video hardware");
      return -1;
    }
  } else {
    WSCONS_ReportError("Displays with 8 bpp or less are not supported");
    return -1;
  }
  
  private->rotate = WSCONS_ROTATE_NONE;
  rotation = SDL_getenv("SDL_VIDEO_WSCONS_ROTATION");
  if (rotation != NULL) {
    if (SDL_strlen(rotation) == 0) {
      private->shadowFB = 0;
      private->rotate = WSCONS_ROTATE_NONE;
      printf("Not rotating, no shadow\n");
    } else if (!SDL_strcmp(rotation, "NONE")) {
      private->shadowFB = 1;
      private->rotate = WSCONS_ROTATE_NONE;
      printf("Not rotating, but still using shadow\n");
    } else if (!SDL_strcmp(rotation, "CW")) {
      private->shadowFB = 1;
      private->rotate = WSCONS_ROTATE_CW;
      printf("Rotating screen clockwise\n");
    } else if (!SDL_strcmp(rotation, "CCW")) {
      private->shadowFB = 1;
      private->rotate = WSCONS_ROTATE_CCW;
      printf("Rotating screen counter clockwise\n");
    } else if (!SDL_strcmp(rotation, "UD")) {
      private->shadowFB = 1;
      private->rotate = WSCONS_ROTATE_UD;
      printf("Rotating screen upside down\n");
    } else {
      WSCONS_ReportError("\"%s\" is not a valid value for "
			 "SDL_VIDEO_WSCONS_ROTATION", rotation);
      return -1;
    }
  }

  switch (private->info.depth) {
    case 1:
    case 4:
    case 8:
      len = private->physlinebytes * private->info.height;
      break;
    case 16:
      if (private->physlinebytes == private->info.width) {
	len = private->info.width * private->info.height * sizeof(short);
      } else {
	len = private->physlinebytes * private->info.height;
      }
      if (private->rotate == WSCONS_ROTATE_NONE ||
	  private->rotate == WSCONS_ROTATE_UD) {
	private->blitFunc = WSCONS_blit16;
      } else {
	private->blitFunc = WSCONS_blit16blocked;
      }
      break;
    case 32:
      if (private->physlinebytes == private->info.width) {
	len = private->info.width * private->info.height * sizeof(int);
      } else {
	len = private->physlinebytes * private->info.height;
      }
      break;
    default:
      WSCONS_ReportError("unsupported depth %d", private->info.depth);
      return -1;
  }

  if (private->shadowFB && private->blitFunc == NULL) {
    WSCONS_ReportError("Using software buffer, but no blitter function is "
		       "available for this %d bpp.", private->info.depth);
    return -1;
  }

  if (ioctl(private->fd, WSDISPLAYIO_SMODE, &wsmode) == -1) {
    WSCONS_ReportError("ioctl SMODE");
    return -1;
  }

  pagemask = getpagesize() - 1;
  mapsize = ((int)len + pagemask) & ~pagemask;
  private->physmem = (Uint8 *)mmap(NULL, mapsize,
				   PROT_READ | PROT_WRITE, MAP_SHARED,
				   private->fd, (off_t)0);
  if (private->physmem == (Uint8 *)MAP_FAILED) {
    private->physmem = NULL;
    WSCONS_ReportError("mmap: %s", strerror(errno));
    return -1;
  }
  private->fbmem_len = len;

  if (private->rotate == WSCONS_ROTATE_CW || 
      private->rotate == WSCONS_ROTATE_CCW) {
    width = private->info.height;
    height = private->info.width;
  } else {
    width = private->info.width;
    height = private->info.height;
  }

  this->info.current_w = width;
  this->info.current_h = height;

  if (private->shadowFB) {
    private->shadowmem = (Uint8 *)SDL_malloc(len);
    if (private->shadowmem == NULL) {
      WSCONS_ReportError("No memory for shadow");
      return -1;
    }
    private->fbstart = private->shadowmem;
    private->fblinebytes = width * ((private->info.depth + 7) / 8);
  } else { 
    private->fbstart = private->physmem;
    private->fblinebytes = private->physlinebytes;
  }
  
  private->SDL_modelist[0] = (SDL_Rect *)SDL_malloc(sizeof(SDL_Rect));
  private->SDL_modelist[0]->w = width;
  private->SDL_modelist[0]->h = height;

  vformat->BitsPerPixel = private->info.depth;
  vformat->BytesPerPixel = private->info.depth / 8;
  
  if (WSCONS_InitKeyboard(this) == -1) {
    return -1;
  }
  
  return 0;
}

SDL_Rect **WSCONS_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags)
{
  if (format->BitsPerPixel == private->info.depth) {
    return private->SDL_modelist;
  } else {
    return NULL;
  }
}

SDL_Surface *WSCONS_SetVideoMode(_THIS, SDL_Surface *current,
				 int width, int height, int bpp, Uint32 flags)
{
  if (width != private->SDL_modelist[0]->w || 
      height != private->SDL_modelist[0]->h) {
    WSCONS_ReportError("Requested video mode %dx%d not supported.",
		       width, height);
    return NULL;
  }
  if (bpp != private->info.depth) {
    WSCONS_ReportError("Requested video depth %d bpp not supported.", bpp);
    return NULL;
  }

  if (!SDL_ReallocFormat(current, 
			 bpp, 
			 private->redMask,
			 private->greenMask,
			 private->blueMask,
			 0)) {
    WSCONS_ReportError("Couldn't allocate new pixel format");
    return NULL;
  }

  current->flags &= SDL_FULLSCREEN;
  if (private->shadowFB) {
    current->flags |= SDL_SWSURFACE;
  } else {
    current->flags |= SDL_HWSURFACE;
  }
  current->w = width;
  current->h = height;
  current->pitch = private->fblinebytes;
  current->pixels = private->fbstart;

  SDL_memset(private->fbstart, 0, private->fbmem_len);

  return current;
}

static int WSCONS_AllocHWSurface(_THIS, SDL_Surface *surface)
{
  return -1;
}
static void WSCONS_FreeHWSurface(_THIS, SDL_Surface *surface)
{
}

static int WSCONS_LockHWSurface(_THIS, SDL_Surface *surface)
{
  return 0;
}

static void WSCONS_UnlockHWSurface(_THIS, SDL_Surface *surface)
{
}

static void WSCONS_blit16(Uint8 *byte_src_pos,
			  int srcRightDelta, 
			  int srcDownDelta, 
			  Uint8 *byte_dst_pos,
			  int dst_linebytes,
			  int width,
			  int height)
{
  int w;
  Uint16 *src_pos = (Uint16 *)byte_src_pos;
  Uint16 *dst_pos = (Uint16 *)byte_dst_pos;

  while (height) {
    Uint16 *src = src_pos;
    Uint16 *dst = dst_pos;
    for (w = width; w != 0; w--) {
      *dst = *src;
      src += srcRightDelta;
      dst++;
    }
    dst_pos = (Uint16 *)((Uint8 *)dst_pos + dst_linebytes);
    src_pos += srcDownDelta;
    height--;
  }
}

#define BLOCKSIZE_W 32
#define BLOCKSIZE_H 32

static void WSCONS_blit16blocked(Uint8 *byte_src_pos,
				 int srcRightDelta, 
				 int srcDownDelta, 
				 Uint8 *byte_dst_pos,
				 int dst_linebytes,
				 int width,
				 int height)
{
  int w;
  Uint16 *src_pos = (Uint16 *)byte_src_pos;
  Uint16 *dst_pos = (Uint16 *)byte_dst_pos;

  while (height > 0) {
    Uint16 *src = src_pos;
    Uint16 *dst = dst_pos;
    for (w = width; w > 0; w -= BLOCKSIZE_W) {
      WSCONS_blit16((Uint8 *)src,
		    srcRightDelta,
		    srcDownDelta,
		    (Uint8 *)dst,
		    dst_linebytes,
		    min(w, BLOCKSIZE_W),
		    min(height, BLOCKSIZE_H));
      src += srcRightDelta * BLOCKSIZE_W;
      dst += BLOCKSIZE_W;
    }
    dst_pos = (Uint16 *)((Uint8 *)dst_pos + dst_linebytes * BLOCKSIZE_H);
    src_pos += srcDownDelta * BLOCKSIZE_H;
    height -= BLOCKSIZE_H;
  }
}

static void WSCONS_UpdateRects(_THIS, int numrects, SDL_Rect *rects)
{
  int width = private->SDL_modelist[0]->w;
  int height = private->SDL_modelist[0]->h;
  int bytesPerPixel = (private->info.depth + 7) / 8;
  int i;

  if (!private->shadowFB) {
    return;
  }

  if (private->info.depth != 16) {
    WSCONS_ReportError("Shadow copy only implemented for 16 bpp");
    return;
  }

  for (i = 0; i < numrects; i++) {
    int x1, y1, x2, y2;
    int scr_x1, scr_y1, scr_x2, scr_y2;
    int sha_x1, sha_y1;
    int shadowRightDelta;  /* Address change when moving right in dest */
    int shadowDownDelta;   /* Address change when moving down in dest */
    Uint8 *src_start;
    Uint8 *dst_start;

    x1 = rects[i].x; 
    y1 = rects[i].y;
    x2 = x1 + rects[i].w; 
    y2 = y1 + rects[i].h;

    if (x1 < 0) {
      x1 = 0;
    } else if (x1 > width) {
      x1 = width;
    }
    if (x2 < 0) {
      x2 = 0;
    } else if (x2 > width) {
      x2 = width;
    }
    if (y1 < 0) {
      y1 = 0;
    } else if (y1 > height) {
      y1 = height;
    }
    if (y2 < 0) {
      y2 = 0;
    } else if (y2 > height) {
      y2 = height;
    }
    if (x2 <= x1 || y2 <= y1) {
      continue;
    }

    switch (private->rotate) {
      case WSCONS_ROTATE_NONE:
	sha_x1 = scr_x1 = x1;
	sha_y1 = scr_y1 = y1;
	scr_x2 = x2;
	scr_y2 = y2;
	shadowRightDelta = 1;
	shadowDownDelta = width;
	break;
      case WSCONS_ROTATE_CCW:
	scr_x1 = y1;
	scr_y1 = width - x2;
	scr_x2 = y2;
	scr_y2 = width - x1;
	sha_x1 = x2 - 1;
	sha_y1 = y1;
	shadowRightDelta = width;
	shadowDownDelta = -1;
	break;
      case WSCONS_ROTATE_UD:
	scr_x1 = width - x2;
	scr_y1 = height - y2;
	scr_x2 = width - x1;
	scr_y2 = height - y1;
	sha_x1 = x2 - 1;
	sha_y1 = y2 - 1;
	shadowRightDelta = -1;
	shadowDownDelta = -width;
	break;
      case WSCONS_ROTATE_CW:
	scr_x1 = height - y2;
	scr_y1 = x1;
	scr_x2 = height - y1;
	scr_y2 = x2;
	sha_x1 = x1;
	sha_y1 = y2 - 1;
	shadowRightDelta = -width;
	shadowDownDelta = 1;
	break;
      default:
	WSCONS_ReportError("Unknown rotation");
	return;
    }

    src_start = private->shadowmem + (sha_y1 * width + sha_x1) * bytesPerPixel;
    dst_start = private->physmem + scr_y1 * private->physlinebytes + 
      scr_x1 * bytesPerPixel;

    private->blitFunc(src_start,
		      shadowRightDelta, 
		      shadowDownDelta, 
		      dst_start,
		      private->physlinebytes,
		      scr_x2 - scr_x1,
		      scr_y2 - scr_y1);
  }
}

int WSCONS_SetColors(_THIS, int firstcolor, int ncolors, SDL_Color *colors)
{
  return 0;
}

/*
 * Note: If we are terminated, this could be called in the middle of
 * another SDL video routine -- notably UpdateRects.
 */
void WSCONS_VideoQuit(_THIS)
{
  int mode = WSDISPLAYIO_MODE_EMUL;

  if (private->shadowmem != NULL) {
    SDL_free(private->shadowmem);
    private->shadowmem = NULL;
  }
  private->fbstart = NULL;
  if (this->screen != NULL) {
    this->screen->pixels = NULL;
  }

  if (private->SDL_modelist[0] != NULL) {
    SDL_free(private->SDL_modelist[0]);
    private->SDL_modelist[0] = NULL;
  }

  if (ioctl(private->fd, WSDISPLAYIO_SMODE, &mode) == -1) {
    WSCONS_ReportError("ioctl SMODE");
  }

  WSCONS_ReleaseKeyboard(this);

  if (private->fd != -1) {
    close(private->fd);
    private->fd = -1;
  }
}
