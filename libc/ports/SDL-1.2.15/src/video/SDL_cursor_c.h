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

/* Useful variables and functions from SDL_cursor.c */
#include "SDL_mouse.h"

extern int  SDL_CursorInit(Uint32 flags);
extern void SDL_CursorPaletteChanged(void);
extern void SDL_DrawCursor(SDL_Surface *screen);
extern void SDL_DrawCursorNoLock(SDL_Surface *screen);
extern void SDL_EraseCursor(SDL_Surface *screen);
extern void SDL_EraseCursorNoLock(SDL_Surface *screen);
extern void SDL_UpdateCursor(SDL_Surface *screen);
extern void SDL_ResetCursor(void);
extern void SDL_MoveCursor(int x, int y);
extern void SDL_CursorQuit(void);

#define INLINE_MOUSELOCK
#ifdef INLINE_MOUSELOCK
/* Inline (macro) versions of the mouse lock functions */
#include "SDL_mutex.h"

extern SDL_mutex *SDL_cursorlock;

#define SDL_LockCursor()						\
	do {								\
		if ( SDL_cursorlock ) {					\
			SDL_mutexP(SDL_cursorlock);			\
		}							\
	} while ( 0 )
#define SDL_UnlockCursor()						\
	do {								\
		if ( SDL_cursorlock ) {					\
			SDL_mutexV(SDL_cursorlock);			\
		}							\
	} while ( 0 )
#else
extern void SDL_LockCursor(void);
extern void SDL_UnlockCursor(void);
#endif /* INLINE_MOUSELOCK */

/* Only for low-level mouse cursor drawing */
extern SDL_Cursor *SDL_cursor;
extern void SDL_MouseRect(SDL_Rect *area);

/* State definitions for the SDL cursor */
#define CURSOR_VISIBLE	0x01
#define CURSOR_USINGSW	0x10
#define SHOULD_DRAWCURSOR(X) 						\
			(((X)&(CURSOR_VISIBLE|CURSOR_USINGSW)) ==  	\
					(CURSOR_VISIBLE|CURSOR_USINGSW))

extern volatile int SDL_cursorstate;
