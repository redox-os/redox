/*
 * SDL - Simple DirectMedia Layer
 * CELL BE Support for PS3 Framebuffer
 * Copyright (C) 2008, 2009 International Business Machines Corporation
 *
 * This library is free software; you can redistribute it and/or modify it
 * under the terms of the GNU Lesser General Public License as published
 * by the Free Software Foundation; either version 2.1 of the License, or
 * (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful, but
 * WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301
 * USA
 *
 *  Martin Lowinski  <lowinski [at] de [dot] ibm [ibm] com>
 *  Dirk Herrendoerfer <d.herrendoerfer [at] de [dot] ibm [dot] com>
 *  SPE code based on research by:
 *  Rene Becker
 *  Thimo Emmerich
 */

#include "SDL_config.h"

#ifndef _SDL_ps3yuv_h
#define _SDL_ps3yuv_h

/* This is the PS3 implementation of YUV video overlays */

#include "SDL_video.h"

extern SDL_Overlay *PS3_CreateYUVOverlay(_THIS, int width, int height, Uint32 format, SDL_Surface *display);
extern int PS3_DisplayYUVOverlay(_THIS, SDL_Overlay *overlay, SDL_Rect *src, SDL_Rect *dst);
extern int PS3_LockYUVOverlay(_THIS, SDL_Overlay *overlay);
extern void PS3_UnlockYUVOverlay(_THIS, SDL_Overlay *overlay);
extern void PS3_FreeYUVOverlay(_THIS, SDL_Overlay *overlay);

#endif /* _SDL_ps3yuv_h */

