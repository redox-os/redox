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

/* General cursor handling code for SDL */

#include "SDL_mutex.h"
#include "SDL_video.h"
#include "SDL_mouse.h"
#include "SDL_blit.h"
#include "SDL_sysvideo.h"
#include "SDL_cursor_c.h"
#include "SDL_pixels_c.h"
#include "default_cursor.h"
#include "../events/SDL_sysevents.h"
#include "../events/SDL_events_c.h"

/* These are static for our cursor handling code */
volatile int SDL_cursorstate = CURSOR_VISIBLE;
SDL_Cursor *SDL_cursor = NULL;
static SDL_Cursor *SDL_defcursor = NULL;
SDL_mutex *SDL_cursorlock = NULL;

/* Public functions */
void SDL_CursorQuit(void)
{
	if ( SDL_cursor != NULL ) {
		SDL_Cursor *cursor;

		SDL_cursorstate &= ~CURSOR_VISIBLE;
		if ( SDL_cursor != SDL_defcursor ) {
			SDL_FreeCursor(SDL_cursor);
		}
		SDL_cursor = NULL;
		if ( SDL_defcursor != NULL ) {
			cursor = SDL_defcursor;
			SDL_defcursor = NULL;
			SDL_FreeCursor(cursor);
		}
	}
	if ( SDL_cursorlock != NULL ) {
		SDL_DestroyMutex(SDL_cursorlock);
		SDL_cursorlock = NULL;
	}
}
int SDL_CursorInit(Uint32 multithreaded)
{
	/* We don't have mouse focus, and the cursor isn't drawn yet */
#ifndef IPOD
	SDL_cursorstate = CURSOR_VISIBLE;
#endif

	/* Create the default cursor */
	if ( SDL_defcursor == NULL ) {
		SDL_defcursor = SDL_CreateCursor(default_cdata, default_cmask,
					DEFAULT_CWIDTH, DEFAULT_CHEIGHT,
						DEFAULT_CHOTX, DEFAULT_CHOTY);
		SDL_SetCursor(SDL_defcursor);
	}

	/* Create a lock if necessary */
	if ( multithreaded ) {
		SDL_cursorlock = SDL_CreateMutex();
	}

	/* That's it! */
	return(0);
}

/* Multi-thread support for cursors */
#ifndef SDL_LockCursor
void SDL_LockCursor(void)
{
	if ( SDL_cursorlock ) {
		SDL_mutexP(SDL_cursorlock);
	}
}
#endif
#ifndef SDL_UnlockCursor
void SDL_UnlockCursor(void)
{
	if ( SDL_cursorlock ) {
		SDL_mutexV(SDL_cursorlock);
	}
}
#endif

/* Software cursor drawing support */
SDL_Cursor * SDL_CreateCursor (Uint8 *data, Uint8 *mask, 
					int w, int h, int hot_x, int hot_y)
{
	SDL_VideoDevice *video = current_video;
	int savelen;
	int i;
	SDL_Cursor *cursor;

	/* Make sure the width is a multiple of 8 */
	w = ((w+7)&~7);

	/* Sanity check the hot spot */
	if ( (hot_x < 0) || (hot_y < 0) || (hot_x >= w) || (hot_y >= h) ) {
		SDL_SetError("Cursor hot spot doesn't lie within cursor");
		return(NULL);
	}

	/* Allocate memory for the cursor */
	cursor = (SDL_Cursor *)SDL_malloc(sizeof *cursor);
	if ( cursor == NULL ) {
		SDL_OutOfMemory();
		return(NULL);
	}
	savelen = (w*4)*h;
	cursor->area.x = 0;
	cursor->area.y = 0;
	cursor->area.w = w;
	cursor->area.h = h;
	cursor->hot_x = hot_x;
	cursor->hot_y = hot_y;
	cursor->data = (Uint8 *)SDL_malloc((w/8)*h*2);
	cursor->mask = cursor->data+((w/8)*h);
	cursor->save[0] = (Uint8 *)SDL_malloc(savelen*2);
	cursor->save[1] = cursor->save[0] + savelen;
	cursor->wm_cursor = NULL;
	if ( ! cursor->data || ! cursor->save[0] ) {
		SDL_FreeCursor(cursor);
		SDL_OutOfMemory();
		return(NULL);
	}
	for ( i=((w/8)*h)-1; i>=0; --i ) {
		cursor->data[i] = data[i];
		cursor->mask[i] = mask[i] | data[i];
	}
	SDL_memset(cursor->save[0], 0, savelen*2);

	/* If the window manager gives us a good cursor, we're done! */
	if ( video->CreateWMCursor ) {
		cursor->wm_cursor = video->CreateWMCursor(video, data, mask,
							w, h, hot_x, hot_y);
	} else {
		cursor->wm_cursor = NULL;
	}
	return(cursor);
}

/* SDL_SetCursor(NULL) can be used to force the cursor redraw,
   if this is desired for any reason.  This is used when setting
   the video mode and when the SDL window gains the mouse focus.
 */
void SDL_SetCursor (SDL_Cursor *cursor)
{
	SDL_VideoDevice *video = current_video;
	SDL_VideoDevice *this  = current_video;

	/* Make sure that the video subsystem has been initialized */
	if ( ! video ) {
		return;
	}

	/* Prevent the event thread from moving the mouse */
	SDL_LockCursor();

	/* Set the new cursor */
	if ( cursor && (cursor != SDL_cursor) ) {
		/* Erase the current mouse position */
		if ( SHOULD_DRAWCURSOR(SDL_cursorstate) ) {
			SDL_EraseCursor(SDL_VideoSurface);
		} else if ( video->MoveWMCursor ) {
			/* If the video driver is moving the cursor directly,
			   it needs to hide the old cursor before (possibly)
			   showing the new one.  (But don't erase NULL cursor)
			 */
			if ( SDL_cursor && video->ShowWMCursor ) {
				video->ShowWMCursor(this, NULL);
			}
		}
		SDL_cursor = cursor;
	}

	/* Draw the new mouse cursor */
	if ( SDL_cursor && (SDL_cursorstate&CURSOR_VISIBLE) ) {
		/* Use window manager cursor if possible */
		int show_wm_cursor = 0;
		if ( SDL_cursor->wm_cursor && video->ShowWMCursor ) {
			show_wm_cursor = video->ShowWMCursor(this, SDL_cursor->wm_cursor);
		}
		if ( show_wm_cursor ) {
			SDL_cursorstate &= ~CURSOR_USINGSW;
		} else {
			SDL_cursorstate |= CURSOR_USINGSW;
			if ( video->ShowWMCursor ) {
				video->ShowWMCursor(this, NULL);
			}
			{ int x, y;
				SDL_GetMouseState(&x, &y);
				SDL_cursor->area.x = (x - SDL_cursor->hot_x);
				SDL_cursor->area.y = (y - SDL_cursor->hot_y);
			}
			SDL_DrawCursor(SDL_VideoSurface);
		}
	} else {
		/* Erase window manager mouse (cursor not visible) */
		if ( SDL_cursor && (SDL_cursorstate & CURSOR_USINGSW) ) {
			SDL_EraseCursor(SDL_VideoSurface);
		} else {
			if ( video ) {
				if ( video->ShowWMCursor ) {
					video->ShowWMCursor(this, NULL);
				}
			}
		}
	}
	SDL_UnlockCursor();
}

SDL_Cursor * SDL_GetCursor (void)
{
	return(SDL_cursor);
}

void SDL_FreeCursor (SDL_Cursor *cursor)
{
	if ( cursor ) {
		if ( cursor == SDL_cursor ) {
			SDL_SetCursor(SDL_defcursor);
		}
		if ( cursor != SDL_defcursor ) {
			SDL_VideoDevice *video = current_video;
			SDL_VideoDevice *this  = current_video;

			if ( cursor->data ) {
				SDL_free(cursor->data);
			}
			if ( cursor->save[0] ) {
				SDL_free(cursor->save[0]);
			}
			if ( video && cursor->wm_cursor ) {
				if ( video->FreeWMCursor ) {
					video->FreeWMCursor(this, cursor->wm_cursor);
				}
			}
			SDL_free(cursor);
		}
	}
}

int SDL_ShowCursor (int toggle)
{
	int showing;

	showing = (SDL_cursorstate & CURSOR_VISIBLE);
	if ( toggle >= 0 ) {
		SDL_LockCursor();
		if ( toggle ) {
			SDL_cursorstate |= CURSOR_VISIBLE;
		} else {
			SDL_cursorstate &= ~CURSOR_VISIBLE;
		}
		SDL_UnlockCursor();
		if ( (SDL_cursorstate & CURSOR_VISIBLE) != showing ) {
			SDL_VideoDevice *video = current_video;
			SDL_VideoDevice *this  = current_video;

			SDL_SetCursor(NULL);
			if ( video && video->CheckMouseMode ) {
				video->CheckMouseMode(this);
			}
		}
	} else {
		/* Query current state */ ;
	}
	return(showing ? 1 : 0);
}

void SDL_WarpMouse (Uint16 x, Uint16 y)
{
	SDL_VideoDevice *video = current_video;
	SDL_VideoDevice *this  = current_video;

	if ( !video || !SDL_PublicSurface ) {
		SDL_SetError("A video mode must be set before warping mouse");
		return;
	}

	/* If we have an offset video mode, offset the mouse coordinates */
	if (this->screen->pitch == 0) {
		x += this->screen->offset / this->screen->format->BytesPerPixel;
		y += this->screen->offset;
	} else {
		x += (this->screen->offset % this->screen->pitch) /
		      this->screen->format->BytesPerPixel;
		y += (this->screen->offset / this->screen->pitch);
	}

	/* This generates a mouse motion event */
	if ( video->WarpWMCursor ) {
		video->WarpWMCursor(this, x, y);
	} else {
		SDL_PrivateMouseMotion(0, 0, x, y);
	}
}

void SDL_MoveCursor(int x, int y)
{
	SDL_VideoDevice *video = current_video;

	/* Erase and update the current mouse position */
	if ( SHOULD_DRAWCURSOR(SDL_cursorstate) ) {
		/* Erase and redraw mouse cursor in new position */
		SDL_LockCursor();
		SDL_EraseCursor(SDL_VideoSurface);
		SDL_cursor->area.x = (x - SDL_cursor->hot_x);
		SDL_cursor->area.y = (y - SDL_cursor->hot_y);
		SDL_DrawCursor(SDL_VideoSurface);
		SDL_UnlockCursor();
	} else if ( video->MoveWMCursor ) {
		video->MoveWMCursor(video, x, y);
	}
}

/* Keep track of the current cursor colors */
static int palette_changed = 1;
static Uint8 pixels8[2];

void SDL_CursorPaletteChanged(void)
{
	palette_changed = 1;
}

void SDL_MouseRect(SDL_Rect *area)
{
	int clip_diff;

	*area = SDL_cursor->area;
	if ( area->x < 0 ) {
		area->w += area->x;
		area->x = 0;
	}
	if ( area->y < 0 ) {
		area->h += area->y;
		area->y = 0;
	}
	clip_diff = (area->x+area->w)-SDL_VideoSurface->w;
	if ( clip_diff > 0 ) {
		area->w = area->w < clip_diff ? 0 : area->w-clip_diff;
	}
	clip_diff = (area->y+area->h)-SDL_VideoSurface->h;
	if ( clip_diff > 0 ) {
		area->h = area->h < clip_diff ? 0 : area->h-clip_diff;
	}
}

static void SDL_DrawCursorFast(SDL_Surface *screen, SDL_Rect *area)
{
	const Uint32 pixels[2] = { 0xFFFFFFFF, 0x00000000 };
	int i, w, h;
	Uint8 *data, datab;
	Uint8 *mask, maskb;

	data = SDL_cursor->data + area->y * SDL_cursor->area.w/8;
	mask = SDL_cursor->mask + area->y * SDL_cursor->area.w/8;
	switch (screen->format->BytesPerPixel) {

	    case 1: {
		Uint8 *dst;
		int dstskip;

		if ( palette_changed ) {
			pixels8[0] = (Uint8)SDL_MapRGB(screen->format, 255, 255, 255);
			pixels8[1] = (Uint8)SDL_MapRGB(screen->format, 0, 0, 0);
			palette_changed = 0;
		}
		dst = (Uint8 *)screen->pixels +
                       (SDL_cursor->area.y+area->y)*screen->pitch +
                       SDL_cursor->area.x;
		dstskip = screen->pitch-area->w;

		for ( h=area->h; h; h-- ) {
			for ( w=area->w/8; w; w-- ) {
				maskb = *mask++;
				datab = *data++;
				for ( i=0; i<8; ++i ) {
					if ( maskb & 0x80 ) {
						*dst = pixels8[datab>>7];
					}
					maskb <<= 1;
					datab <<= 1;
					dst++;
				}
			}
			dst += dstskip;
		}
	    }
	    break;

	    case 2: {
		Uint16 *dst;
		int dstskip;

		dst = (Uint16 *)screen->pixels +
                       (SDL_cursor->area.y+area->y)*screen->pitch/2 +
                       SDL_cursor->area.x;
		dstskip = (screen->pitch/2)-area->w;

		for ( h=area->h; h; h-- ) {
			for ( w=area->w/8; w; w-- ) {
				maskb = *mask++;
				datab = *data++;
				for ( i=0; i<8; ++i ) {
					if ( maskb & 0x80 ) {
						*dst = (Uint16)pixels[datab>>7];
					}
					maskb <<= 1;
					datab <<= 1;
					dst++;
				}
			}
			dst += dstskip;
		}
	    }
	    break;

	    case 3: {
		Uint8 *dst;
		int dstskip;

		dst = (Uint8 *)screen->pixels +
                       (SDL_cursor->area.y+area->y)*screen->pitch +
                       SDL_cursor->area.x*3;
		dstskip = screen->pitch-area->w*3;

		for ( h=area->h; h; h-- ) {
			for ( w=area->w/8; w; w-- ) {
				maskb = *mask++;
				datab = *data++;
				for ( i=0; i<8; ++i ) {
					if ( maskb & 0x80 ) {
						SDL_memset(dst,pixels[datab>>7],3);
					}
					maskb <<= 1;
					datab <<= 1;
					dst += 3;
				}
			}
			dst += dstskip;
		}
	    }
	    break;

	    case 4: {
		Uint32 *dst;
		int dstskip;

		dst = (Uint32 *)screen->pixels +
                       (SDL_cursor->area.y+area->y)*screen->pitch/4 +
                       SDL_cursor->area.x;
		dstskip = (screen->pitch/4)-area->w;

		for ( h=area->h; h; h-- ) {
			for ( w=area->w/8; w; w-- ) {
				maskb = *mask++;
				datab = *data++;
				for ( i=0; i<8; ++i ) {
					if ( maskb & 0x80 ) {
						*dst = pixels[datab>>7];
					}
					maskb <<= 1;
					datab <<= 1;
					dst++;
				}
			}
			dst += dstskip;
		}
	    }
	    break;
	}
}

static void SDL_DrawCursorSlow(SDL_Surface *screen, SDL_Rect *area)
{
	const Uint32 pixels[2] = { 0xFFFFFF, 0x000000 };
	int h;
	int x, minx, maxx;
	Uint8 *data, datab = 0;
	Uint8 *mask, maskb = 0;
	Uint8 *dst;
	int dstbpp, dstskip;

	data = SDL_cursor->data + area->y * SDL_cursor->area.w/8;
	mask = SDL_cursor->mask + area->y * SDL_cursor->area.w/8;
	dstbpp = screen->format->BytesPerPixel;
	dst = (Uint8 *)screen->pixels +
                       (SDL_cursor->area.y+area->y)*screen->pitch +
                       SDL_cursor->area.x*dstbpp;
	dstskip = screen->pitch-SDL_cursor->area.w*dstbpp;

	minx = area->x;
	maxx = area->x+area->w;
	if ( screen->format->BytesPerPixel == 1 ) {
		if ( palette_changed ) {
			pixels8[0] = (Uint8)SDL_MapRGB(screen->format, 255, 255, 255);
			pixels8[1] = (Uint8)SDL_MapRGB(screen->format, 0, 0, 0);
			palette_changed = 0;
		}
		for ( h=area->h; h; h-- ) {
			for ( x=0; x<SDL_cursor->area.w; ++x ) {
				if ( (x%8) == 0 ) {
					maskb = *mask++;
					datab = *data++;
				}
				if ( (x >= minx) && (x < maxx) ) {
					if ( maskb & 0x80 ) {
						SDL_memset(dst, pixels8[datab>>7], dstbpp);
					}
				}
				maskb <<= 1;
				datab <<= 1;
				dst += dstbpp;
			}
			dst += dstskip;
		}
	} else {
		for ( h=area->h; h; h-- ) {
			for ( x=0; x<SDL_cursor->area.w; ++x ) {
				if ( (x%8) == 0 ) {
					maskb = *mask++;
					datab = *data++;
				}
				if ( (x >= minx) && (x < maxx) ) {
					if ( maskb & 0x80 ) {
						SDL_memset(dst, pixels[datab>>7], dstbpp);
					}
				}
				maskb <<= 1;
				datab <<= 1;
				dst += dstbpp;
			}
			dst += dstskip;
		}
	}
}

/* This handles the ugly work of converting the saved cursor background from
   the pixel format of the shadow surface to that of the video surface.
   This is only necessary when blitting from a shadow surface of a different
   pixel format than the video surface, and using a software rendered cursor.
*/
static void SDL_ConvertCursorSave(SDL_Surface *screen, int w, int h)
{
	SDL_BlitInfo info;
	SDL_loblit RunBlit;

	/* Make sure we can steal the blit mapping */
	if ( screen->map->dst != SDL_VideoSurface ) {
		return;
	}

	/* Set up the blit information */
	info.s_pixels = SDL_cursor->save[1];
	info.s_width = w;
	info.s_height = h;
	info.s_skip = 0;
	info.d_pixels = SDL_cursor->save[0];
	info.d_width = w;
	info.d_height = h;
	info.d_skip = 0;
	info.aux_data = screen->map->sw_data->aux_data;
	info.src = screen->format;
	info.table = screen->map->table;
	info.dst = SDL_VideoSurface->format;
	RunBlit = screen->map->sw_data->blit;

	/* Run the actual software blit */
	RunBlit(&info);
}

void SDL_DrawCursorNoLock(SDL_Surface *screen)
{
	SDL_Rect area;

	/* Get the mouse rectangle, clipped to the screen */
	SDL_MouseRect(&area);
	if ( (area.w == 0) || (area.h == 0) ) {
		return;
	}

	/* Copy mouse background */
	{ int w, h, screenbpp;
	  Uint8 *src, *dst;

	  /* Set up the copy pointers */
	  screenbpp = screen->format->BytesPerPixel;
	  if ( (screen == SDL_VideoSurface) ||
	          FORMAT_EQUAL(screen->format, SDL_VideoSurface->format) ) {
		dst = SDL_cursor->save[0];
	  } else {
		dst = SDL_cursor->save[1];
	  }
	  src = (Uint8 *)screen->pixels + area.y * screen->pitch +
                                          area.x * screenbpp;

	  /* Perform the copy */
	  w = area.w*screenbpp;
	  h = area.h;
	  while ( h-- ) {
		  SDL_memcpy(dst, src, w);
		  dst += w;
		  src += screen->pitch;
	  }
	}

	/* Draw the mouse cursor */
	area.x -= SDL_cursor->area.x;
	area.y -= SDL_cursor->area.y;
	if ( (area.x == 0) && (area.w == SDL_cursor->area.w) ) {
		SDL_DrawCursorFast(screen, &area);
	} else {
		SDL_DrawCursorSlow(screen, &area);
	}
}

void SDL_DrawCursor(SDL_Surface *screen)
{
	/* Lock the screen if necessary */
	if ( screen == NULL ) {
		return;
	}
	if ( SDL_MUSTLOCK(screen) ) {
		if ( SDL_LockSurface(screen) < 0 ) {
			return;
		}
	}

	SDL_DrawCursorNoLock(screen);

	/* Unlock the screen and update if necessary */
	if ( SDL_MUSTLOCK(screen) ) {
		SDL_UnlockSurface(screen);
	}
	if ( (screen == SDL_VideoSurface) &&
	     ((screen->flags & SDL_HWSURFACE) != SDL_HWSURFACE) ) {
		SDL_VideoDevice *video = current_video;
		SDL_VideoDevice *this  = current_video;
		SDL_Rect area;

		SDL_MouseRect(&area);

		/* This can be called before a video mode is set */
		if ( video->UpdateRects ) {
			video->UpdateRects(this, 1, &area);
		}
	}
}

void SDL_EraseCursorNoLock(SDL_Surface *screen)
{
	SDL_Rect area;

	/* Get the mouse rectangle, clipped to the screen */
	SDL_MouseRect(&area);
	if ( (area.w == 0) || (area.h == 0) ) {
		return;
	}

	/* Copy mouse background */
	{ int w, h, screenbpp;
	  Uint8 *src, *dst;

	  /* Set up the copy pointers */
	  screenbpp = screen->format->BytesPerPixel;
	  if ( (screen == SDL_VideoSurface) ||
	          FORMAT_EQUAL(screen->format, SDL_VideoSurface->format) ) {
		src = SDL_cursor->save[0];
	  } else {
		src = SDL_cursor->save[1];
	  }
	  dst = (Uint8 *)screen->pixels + area.y * screen->pitch +
                                          area.x * screenbpp;

	  /* Perform the copy */
	  w = area.w*screenbpp;
	  h = area.h;
	  while ( h-- ) {
		  SDL_memcpy(dst, src, w);
		  src += w;
		  dst += screen->pitch;
	  }

	  /* Perform pixel conversion on cursor background */
	  if ( src > SDL_cursor->save[1] ) {
		SDL_ConvertCursorSave(screen, area.w, area.h);
	  }
	}
}

void SDL_EraseCursor(SDL_Surface *screen)
{
	/* Lock the screen if necessary */
	if ( screen == NULL ) {
		return;
	}
	if ( SDL_MUSTLOCK(screen) ) {
		if ( SDL_LockSurface(screen) < 0 ) {
			return;
		}
	}

	SDL_EraseCursorNoLock(screen);

	/* Unlock the screen and update if necessary */
	if ( SDL_MUSTLOCK(screen) ) {
		SDL_UnlockSurface(screen);
	}
	if ( (screen == SDL_VideoSurface) &&
	     ((screen->flags & SDL_HWSURFACE) != SDL_HWSURFACE) ) {
		SDL_VideoDevice *video = current_video;
		SDL_VideoDevice *this  = current_video;
		SDL_Rect area;

		SDL_MouseRect(&area);
		if ( video->UpdateRects ) {
			video->UpdateRects(this, 1, &area);
		}
	}
}

/* Reset the cursor on video mode change
   FIXME:  Keep track of all cursors, and reset them all.
 */
void SDL_ResetCursor(void)
{
	int savelen;

	if ( SDL_cursor ) {
		savelen = SDL_cursor->area.w*4*SDL_cursor->area.h;
		SDL_cursor->area.x = 0;
		SDL_cursor->area.y = 0;
		SDL_memset(SDL_cursor->save[0], 0, savelen);
	}
}
