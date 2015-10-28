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

#include <sys/ioctl.h>

#include "SDL_mouse.h"
#include "../../events/SDL_events_c.h"
#include "../SDL_cursor_c.h"
#include "SDL_gsvideo.h"
#include "SDL_gsmouse_c.h"


/* The implementation dependent data for the window manager cursor */
struct WMcursor {
	int unused;
};

/* There isn't any implementation dependent data */
void GS_FreeWMCursor(_THIS, WMcursor *cursor)
{
	return;
}

/* There isn't any implementation dependent data */
WMcursor *GS_CreateWMCursor(_THIS,
		Uint8 *data, Uint8 *mask, int w, int h, int hot_x, int hot_y)
{
	return((WMcursor *)0x01);
}

static void GS_MoveCursor(_THIS, SDL_Cursor *cursor, int x, int y)
{
	SDL_Surface *screen;
	struct ps2_image image;
	SDL_Rect area;
	int mouse_y1, mouse_y2;
	void *saved_pixels;
	int screen_updated;

	/* Lock so we don't interrupt an update with mouse motion */
	SDL_LockCursor();

	/* Make sure any pending DMA has completed */
	if ( dma_pending ) {
		ioctl(console_fd, PS2IOC_SENDQCT, 1);
		dma_pending = 0;
	}

	/* Remove the cursor image from the DMA area */
	screen = this->screen;
	saved_pixels = screen->pixels;
	screen->pixels = mapped_mem + screen->offset;
	screen_updated = 0;
	if ( cursor_drawn ) {
		SDL_EraseCursorNoLock(screen);
		cursor_drawn = 0;
		screen_updated = 1;
	}

	/* Save the current mouse area */
	SDL_MouseRect(&area);
	mouse_y1 = area.y;
	mouse_y2 = area.y+area.h;

	/* Only draw the new cursor if there was one passed in */
	if ( cursor ) {
		/* Set the new location */
		cursor->area.x = (x - cursor->hot_x);
		cursor->area.y = (y - cursor->hot_y);

		/* Draw the cursor at the new location */
		if ( (SDL_cursorstate & CURSOR_VISIBLE) && screen->pixels ) {
			SDL_DrawCursorNoLock(screen);
			cursor_drawn = 1;
			screen_updated = 1;
		}
	}
	screen->pixels = saved_pixels;

	/* Update the affected area of the screen */
	if ( screen_updated ) {
		SDL_MouseRect(&area);
		if ( area.y < mouse_y1 ) {
			mouse_y1 = area.y;
		}
		if ( (area.y+area.h) > mouse_y2 ) {
			mouse_y2 = area.y+area.h;
		}
		image = screen_image;
		image.y += screen->offset / screen->pitch + mouse_y1;
		image.h = mouse_y2 - mouse_y1;
		image.ptr = mapped_mem +
		            (image.y - screen_image.y) * screen->pitch;
		ioctl(console_fd, PS2IOC_LOADIMAGE, &image);

		/* Need to scale offscreen image to TV output */
		if ( image.y > 0 ) {
			scaleimage_nonblock(console_fd,
			                    tex_tags_mem, scale_tags_mem);
		}
	}

	/* We're finished */
	SDL_UnlockCursor();
}

void GS_MoveWMCursor(_THIS, int x, int y)
{
	GS_MoveCursor(this, SDL_cursor, x, y);
}

int GS_ShowWMCursor(_THIS, WMcursor *wmcursor)
{
	SDL_Cursor *cursor;
	int x, y;

	/* Draw the cursor at the appropriate location */
	SDL_GetMouseState(&x, &y);
	if ( wmcursor ) {
		cursor = SDL_cursor;
	} else {
		cursor = NULL;
	}
	GS_MoveCursor(this, cursor, x, y);
	return(1);
}
