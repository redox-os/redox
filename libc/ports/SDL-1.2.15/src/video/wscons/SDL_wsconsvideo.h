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

#ifndef _SDL_wsconsvideo_h
#define _SDL_wsconsvideo_h

#include <sys/time.h>
#include <termios.h>
#include <dev/wscons/wsconsio.h>

#include "SDL_mouse.h"
#include "SDL_mutex.h"
#include "../SDL_sysvideo.h"

void WSCONS_ReportError(char *fmt, ...);

/* Hidden "this" pointer for the video functions */
#define _THIS	SDL_VideoDevice *this
#define private	(this->hidden)

/* Private display data */

typedef void WSCONS_bitBlit(Uint8 *src_pos,
			    int srcRightDelta, // pixels, not bytes
			    int srcDownDelta,  // pixels, not bytes
			    Uint8 *dst_pos,
			    int dst_linebytes,
			    int width,
			    int height);

struct SDL_PrivateVideoData {
  int fd;                       /* file descriptor of open device */
  struct wsdisplay_fbinfo info; /* frame buffer characteristics */
  int physlinebytes;            /* number of bytes per row */
  int redMask, greenMask, blueMask;

  Uint8 *fbstart;               /* These refer to the surface used, */
  int fblinebytes;              /* physical frame buffer or shadow. */

  size_t fbmem_len;
  Uint8 *physmem;
  Uint8 *shadowmem;
  int rotate;
  int shadowFB;                 /* Tells whether a shadow is being used. */

  WSCONS_bitBlit *blitFunc;

  SDL_Rect *SDL_modelist[2];

  unsigned int kbdType;
  int did_save_tty;
  struct termios saved_tty;
};


#endif /* _SDL_wsconsvideo_h */
