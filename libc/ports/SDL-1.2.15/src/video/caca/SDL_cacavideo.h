/*
    SDL - Simple DirectMedia Layer
    Copyright (C) 2003  Sam Hocevar

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

    Sam Hocevar
    sam@zoy.org
*/

#ifdef SAVE_RCSID
static char rcsid =
 "@(#) $Id: libsdl-1.2.11-libcaca.patch,v 1.1 2006/09/18 16:06:06 mr_bones_ Exp $";
#endif

#ifndef _SDL_cacavideo_h
#define _SDL_cacavideo_h

#include "SDL_mouse.h"
#include "../SDL_sysvideo.h"
#include "SDL_mutex.h"

#include <sys/time.h>
#include <time.h>

#include <caca.h>
#ifdef CACA_API_VERSION_1
#include <caca0.h>
#endif

/* Hidden "this" pointer for the video functions */
#define _THIS	SDL_VideoDevice *this

#define SDL_NUMMODES 6

/* Private display data */
struct SDL_PrivateVideoData {
	SDL_Rect *SDL_modelist[SDL_NUMMODES+1];
	SDL_mutex *mutex;

	struct caca_bitmap *bitmap;
	void *buffer;
	int w, h;

	int lastkey;
	struct timeval lasttime;
};

/* Old variable names */
#define SDL_modelist		(this->hidden->SDL_modelist)
#define Caca_palette		    (this->hidden->palette)
#define Caca_bitmap		    (this->hidden->bitmap)
#define Caca_buffer		    (this->hidden->buffer)

#define Caca_w		    (this->hidden->w)
#define Caca_h		    (this->hidden->h)

#define Caca_lastkey		    (this->hidden->lastkey)
#define Caca_lasttime		    (this->hidden->lasttime)

#define Caca_mutex		    (this->hidden->mutex)

#endif /* _SDL_cacavideo_h */

