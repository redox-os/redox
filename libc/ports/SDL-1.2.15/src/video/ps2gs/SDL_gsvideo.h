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

#ifndef _SDL_gsvideo_h
#define _SDL_gsvideo_h

#include <sys/types.h>
#include <termios.h>
#include <linux/ps2/dev.h>
#include <linux/ps2/gs.h>

#include "SDL_mouse.h"
#include "SDL_mutex.h"
#include "../SDL_sysvideo.h"

/* Hidden "this" pointer for the video functions */
#define _THIS	SDL_VideoDevice *this


/* Private display data */
struct SDL_PrivateVideoData {
	/* Gotta love that simple PS2 graphics interface. :) */
	int console_fd;
	int memory_fd;
	struct ps2_screeninfo saved_vinfo;

	/* Ye olde linux keyboard code */
	int current_vt;
	int saved_vt;
	int keyboard_fd;
	int saved_kbd_mode;
	struct termios saved_kbd_termios;

	/* Ye olde linux mouse code */
	int mouse_fd;
	int cursor_drawn;

	/* The memory mapped DMA area and associated variables */
	caddr_t mapped_mem;
	int pixels_len;
	int mapped_len;
	struct ps2_image screen_image;
	int screen_image_size;
	unsigned long long *head_tags_mem;
	unsigned long long *image_tags_mem;
	unsigned long long *tex_tags_mem;
	unsigned long long *scale_tags_mem;
	int dma_pending;
};
/* Old variable names */
#define console_fd		(this->hidden->console_fd)
#define memory_fd		(this->hidden->memory_fd)
#define saved_vinfo		(this->hidden->saved_vinfo)
#define current_vt		(this->hidden->current_vt)
#define saved_vt		(this->hidden->saved_vt)
#define keyboard_fd		(this->hidden->keyboard_fd)
#define saved_kbd_mode		(this->hidden->saved_kbd_mode)
#define saved_kbd_termios	(this->hidden->saved_kbd_termios)
#define mouse_fd		(this->hidden->mouse_fd)
#define cursor_drawn		(this->hidden->cursor_drawn)
#define mapped_mem		(this->hidden->mapped_mem)
#define pixels_len		(this->hidden->pixels_len)
#define mapped_len		(this->hidden->mapped_len)
#define screen_image		(this->hidden->screen_image)
#define screen_image_size	(this->hidden->screen_image_size)
#define head_tags_mem		(this->hidden->head_tags_mem)
#define image_tags_mem		(this->hidden->image_tags_mem)
#define tex_tags_mem		(this->hidden->tex_tags_mem)
#define scale_tags_mem		(this->hidden->scale_tags_mem)
#define dma_pending		(this->hidden->dma_pending)

/* Shared between the mouse and video code for screen update scaling */
extern int scaleimage_nonblock(int fd,
                               unsigned long long *tm, unsigned long long *sm);
#endif /* _SDL_gsvideo_h */
