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

/* Qtopia based framebuffer implementation */

#include <unistd.h>

#include <qapplication.h>
#include <qpe/qpeapplication.h>

#include "SDL_timer.h"

#include "SDL_QWin.h"

extern "C" {

#include "../SDL_sysvideo.h"
#include "../../events/SDL_events_c.h"
#include "SDL_sysevents_c.h"
#include "SDL_sysmouse_c.h"
#include "SDL_syswm_c.h"
#include "SDL_lowvideo.h"

  //#define QTOPIA_DEBUG
#define QT_HIDDEN_SIZE	32	/* starting hidden window size */

  /* Name of the environment variable used to invert the screen rotation or not:
     Possible values:
     !=0 : Screen is 270° rotated
     0: Screen is 90° rotated*/
#define SDL_QT_ROTATION_ENV_NAME "SDL_QT_INVERT_ROTATION"
  
  /* Initialization/Query functions */
  static int QT_VideoInit(_THIS, SDL_PixelFormat *vformat);
  static SDL_Rect **QT_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags);
  static SDL_Surface *QT_SetVideoMode(_THIS, SDL_Surface *current, int width, int height, int bpp, Uint32 flags);
  static void QT_UpdateMouse(_THIS);
  static int QT_SetColors(_THIS, int firstcolor, int ncolors, SDL_Color *colors);
  static void QT_VideoQuit(_THIS);

  /* Hardware surface functions */
  static int QT_AllocHWSurface(_THIS, SDL_Surface *surface);
  static int QT_LockHWSurface(_THIS, SDL_Surface *surface);
  static void QT_UnlockHWSurface(_THIS, SDL_Surface *surface);
  static void QT_FreeHWSurface(_THIS, SDL_Surface *surface);

  static int QT_ToggleFullScreen(_THIS, int fullscreen);

  static int QT_IconifyWindow(_THIS);
  static SDL_GrabMode QT_GrabInput(_THIS, SDL_GrabMode mode);

  /* FB driver bootstrap functions */

  static int QT_Available(void)
  {
    return(1);
  }

  static void QT_DeleteDevice(SDL_VideoDevice *device)
  {
    SDL_free(device->hidden);
    SDL_free(device);
  }

  static SDL_VideoDevice *QT_CreateDevice(int devindex)
  {
    SDL_VideoDevice *device;

    /* Initialize all variables that we clean on shutdown */
    device = (SDL_VideoDevice *)SDL_malloc(sizeof(SDL_VideoDevice));
    if ( device ) {
      SDL_memset(device, 0, (sizeof *device));
      device->hidden = (struct SDL_PrivateVideoData *)
	SDL_malloc((sizeof *device->hidden));
    }
    if ( (device == NULL) || (device->hidden == NULL) ) {
      SDL_OutOfMemory();
      if ( device ) {
	SDL_free(device);
      }
      return(0);
    }
    SDL_memset(device->hidden, 0, (sizeof *device->hidden));

    /* Set the function pointers */
    device->VideoInit = QT_VideoInit;
    device->ListModes = QT_ListModes;
    device->SetVideoMode = QT_SetVideoMode;
    device->UpdateMouse = QT_UpdateMouse;
    device->SetColors = QT_SetColors;
    device->UpdateRects = NULL;
    device->VideoQuit = QT_VideoQuit;
    device->AllocHWSurface = QT_AllocHWSurface;
    device->CheckHWBlit = NULL;
    device->FillHWRect = NULL;
    device->SetHWColorKey = NULL;
    device->SetHWAlpha = NULL;
    device->LockHWSurface = QT_LockHWSurface;
    device->UnlockHWSurface = QT_UnlockHWSurface;
    device->FlipHWSurface = NULL;
    device->FreeHWSurface = QT_FreeHWSurface;
    device->SetIcon = NULL;
    device->SetCaption = QT_SetWMCaption;
    device->IconifyWindow = QT_IconifyWindow;
    device->GrabInput = QT_GrabInput;
    device->GetWMInfo = NULL;
    device->FreeWMCursor = QT_FreeWMCursor;
    device->CreateWMCursor = QT_CreateWMCursor;
    device->ShowWMCursor = QT_ShowWMCursor;
    device->WarpWMCursor = QT_WarpWMCursor;
    device->InitOSKeymap = QT_InitOSKeymap;
    device->PumpEvents = QT_PumpEvents;

    device->free = QT_DeleteDevice;
    device->ToggleFullScreen = QT_ToggleFullScreen;

    /* Set the driver flags */
    device->handles_any_size = 0;
	
    return device;
  }

  VideoBootStrap Qtopia_bootstrap = {
    "qtopia", "Qtopia / QPE graphics",
    QT_Available, QT_CreateDevice
  };

  /* Function to sort the display_list */
  static int CompareModes(const void *A, const void *B)
  {
#if 0
    const display_mode *a = (display_mode *)A;
    const display_mode *b = (display_mode *)B;

    if ( a->space == b->space ) {
      return((b->virtual_width*b->virtual_height)-
	     (a->virtual_width*a->virtual_height));
    } else {
      return(ColorSpaceToBitsPerPixel(b->space)-
	     ColorSpaceToBitsPerPixel(a->space));
    }
#endif
    return 0;
  }

  /* Yes, this isn't the fastest it could be, but it works nicely */
  static int QT_AddMode(_THIS, int index, unsigned int w, unsigned int h)
  {
    SDL_Rect *mode;
    int i;
    int next_mode;

    /* Check to see if we already have this mode */
    if ( SDL_nummodes[index] > 0 ) {
      for ( i=SDL_nummodes[index]-1; i >= 0; --i ) {
	mode = SDL_modelist[index][i];
	if ( (mode->w == w) && (mode->h == h) ) {
	  return(0);
	}
      }
    }

    /* Set up the new video mode rectangle */
    mode = (SDL_Rect *)SDL_malloc(sizeof *mode);
    if ( mode == NULL ) {
      SDL_OutOfMemory();
      return(-1);
    }
    mode->x = 0;
    mode->y = 0;
    mode->w = w;
    mode->h = h;
#ifdef QTOPIA_DEBUG
    fprintf(stderr, "Adding mode %dx%d at %d bytes per pixel\n", w, h, index+1);
#endif

    /* Allocate the new list of modes, and fill in the new mode */
    next_mode = SDL_nummodes[index];
    SDL_modelist[index] = (SDL_Rect **)
      SDL_realloc(SDL_modelist[index], (1+next_mode+1)*sizeof(SDL_Rect *));
    if ( SDL_modelist[index] == NULL ) {
      SDL_OutOfMemory();
      SDL_nummodes[index] = 0;
      SDL_free(mode);
      return(-1);
    }
    SDL_modelist[index][next_mode] = mode;
    SDL_modelist[index][next_mode+1] = NULL;
    SDL_nummodes[index]++;

    return(0);
  }

  int QT_VideoInit(_THIS, SDL_PixelFormat *vformat)
  {
    /* Initialize the QPE Application  */
     /* Determine the screen depth */
    vformat->BitsPerPixel = QPixmap::defaultDepth();

    // For now we hardcode the current depth because anything else
    // might as well be emulated by SDL rather than by Qtopia.
    
    QSize desktop_size = qApp->desktop()->size();
    QT_AddMode(_this, ((vformat->BitsPerPixel+7)/8)-1,
	       desktop_size.width(), desktop_size.height());
    QT_AddMode(_this, ((vformat->BitsPerPixel+7)/8)-1,
	       desktop_size.height(), desktop_size.width());

    /* Determine the current screen size */
    _this->info.current_w = desktop_size.width();
    _this->info.current_h = desktop_size.height();

    /* Create the window / widget */
    SDL_Win = new SDL_QWin(QSize(QT_HIDDEN_SIZE, QT_HIDDEN_SIZE));
    ((QPEApplication*)qApp)->showMainWidget(SDL_Win);
    /* Fill in some window manager capabilities */
    _this->info.wm_available = 0;

    /* We're done! */
    return(0);
  }

  /* We support any dimension at our bit-depth */
  SDL_Rect **QT_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags)
  {
    SDL_Rect **modes;

    modes = ((SDL_Rect **)0);
    if ( (flags & SDL_FULLSCREEN) == SDL_FULLSCREEN ) {
      modes = SDL_modelist[((format->BitsPerPixel+7)/8)-1];
    } else {
      if ( format->BitsPerPixel ==
	   _this->screen->format->BitsPerPixel ) {
	modes = ((SDL_Rect **)-1);
      }
    }
    return(modes);
  }

  /* Various screen update functions available */
  static void QT_NormalUpdate(_THIS, int numrects, SDL_Rect *rects);


  static int QT_SetFullScreen(_THIS, SDL_Surface *screen, int fullscreen)
  {
    return -1;
  }

  static int QT_ToggleFullScreen(_THIS, int fullscreen)
  {
    return -1;
  }

  /* FIXME: check return values and cleanup here */
  SDL_Surface *QT_SetVideoMode(_THIS, SDL_Surface *current,
			       int width, int height, int bpp, Uint32 flags)
  {

    QImage *qimage;
    QSize desktop_size = qApp->desktop()->size();

    
    current->flags = 0; //SDL_FULLSCREEN; // We always run fullscreen.

    if(width <= desktop_size.width()
	      && height <= desktop_size.height()) {
      current->w = desktop_size.width();
      current->h = desktop_size.height();
    } else if(width <= desktop_size.height() && height <= desktop_size.width()) {
      // Landscape mode
      char * envString = SDL_getenv(SDL_QT_ROTATION_ENV_NAME);
      int envValue = envString ? atoi(envString) : 0;
      screenRotation = envValue ? SDL_QT_ROTATION_270 : SDL_QT_ROTATION_90;
      current->h = desktop_size.width();
      current->w = desktop_size.height();
    } else {
      SDL_SetError("Unsupported resolution, %dx%d\n", width, height);
    }
    if ( flags & SDL_OPENGL ) {
      SDL_SetError("OpenGL not supported");
      return(NULL);
    } 
    /* Create the QImage framebuffer */
    qimage = new QImage(current->w, current->h, bpp);
    if (qimage->isNull()) {
      SDL_SetError("Couldn't create screen bitmap");
      delete qimage;
      return(NULL);
    }
    current->pitch = qimage->bytesPerLine();
    current->pixels = (void *)qimage->bits();
    SDL_Win->setImage(qimage);
    _this->UpdateRects = QT_NormalUpdate;
    SDL_Win->setFullscreen(true);
    /* We're done */
    return(current);
  }

  /* Update the current mouse state and position */
  void QT_UpdateMouse(_THIS)
  {
    QPoint point(-1, -1);
    if ( SDL_Win->isActiveWindow() ) {
      point = SDL_Win->mousePos();
    }
    
    if ( (point.x() >= 0) && (point.x() < SDL_VideoSurface->w) &&
	 (point.y() >= 0) && (point.y() < SDL_VideoSurface->h) ) {
      SDL_PrivateAppActive(1, SDL_APPMOUSEFOCUS);
      SDL_PrivateMouseMotion(0, 0,
			     (Sint16)point.x(), (Sint16)point.y());
    } else {
      SDL_PrivateAppActive(0, SDL_APPMOUSEFOCUS);
    }
  }

  /* We don't actually allow hardware surfaces other than the main one */
  static int QT_AllocHWSurface(_THIS, SDL_Surface *surface)
  {
    return(-1);
  }
  static void QT_FreeHWSurface(_THIS, SDL_Surface *surface)
  {
    return;
  }
  static int QT_LockHWSurface(_THIS, SDL_Surface *surface)
  {
    return(0);
  }
  static void QT_UnlockHWSurface(_THIS, SDL_Surface *surface)
  {
    return;
  }

  static void QT_NormalUpdate(_THIS, int numrects, SDL_Rect *rects)
  {
    if(SDL_Win->lockScreen()) {
      for(int i=0; i<numrects; ++i ) {
	QRect rect(rects[i].x, rects[i].y,
		   rects[i].w, rects[i].h);
	SDL_Win->repaintRect(rect);
      }
      SDL_Win->unlockScreen();
    }
  }
  /* Is the system palette settable? */
  int QT_SetColors(_THIS, int firstcolor, int ncolors, SDL_Color *colors)
  {
    return -1;
  }

  void QT_VideoQuit(_THIS)
  {
    // This is dumb, but if I free this, the app doesn't exit correctly.
    // Of course, this will leak memory if init video is done more than once.
    // Sucks but such is life.
    
    //    -- David Hedbor
    //    delete SDL_Win; 
    //    SDL_Win = 0;
    _this->screen->pixels = NULL;
    QT_GrabInput(_this, SDL_GRAB_OFF);
  }

  static int QT_IconifyWindow(_THIS) {
    SDL_Win->hide();
    
    return true;
  }

  static SDL_GrabMode QT_GrabInput(_THIS, SDL_GrabMode mode) {
    if(mode == SDL_GRAB_OFF) {
      QPEApplication::grabKeyboard();
      qApp->processEvents();
      QPEApplication::ungrabKeyboard();
    } else {
      QPEApplication::grabKeyboard();
    }
    qApp->processEvents();
    return mode;
  }
  
}; /* Extern C */
