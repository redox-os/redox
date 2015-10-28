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

#ifndef _SDL_aavideo_h
#define _SDL_aavideo_h

#include "SDL_mouse.h"
#include "SDL_mutex.h"
#include "../SDL_sysvideo.h"

#include <aalib.h>

/* Hidden "this" pointer for the video functions */
#define _THIS	SDL_VideoDevice *this

#define SDL_NUMMODES 6

/* Private display data */
struct SDL_PrivateVideoData {
	SDL_Rect *SDL_modelist[SDL_NUMMODES+1];
	aa_context *context;
	aa_palette palette;
	aa_renderparams *rparams;
	double x_ratio, y_ratio;
    int w, h;
    SDL_mutex *mutex;
    int in_x11;
    void *buffer;
};

/* Old variable names */
#define SDL_modelist		(this->hidden->SDL_modelist)
#define AA_context		    (this->hidden->context)
#define AA_palette		    (this->hidden->palette)
#define AA_rparams		    (this->hidden->rparams)
#define AA_buffer		    (this->hidden->buffer)

#define AA_x_ratio		    (this->hidden->x_ratio)
#define AA_y_ratio		    (this->hidden->y_ratio)

#define AA_mutex		    (this->hidden->mutex)
#define AA_in_x11		    (this->hidden->in_x11)
#define AA_w                (this->hidden->w)
#define AA_h                (this->hidden->h)

#endif /* _SDL_aavideo_h */
