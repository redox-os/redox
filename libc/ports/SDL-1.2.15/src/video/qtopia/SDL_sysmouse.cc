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

#include "SDL_QWin.h"

extern "C" {

#include "SDL_sysmouse_c.h"

/* The implementation dependent data for the window manager cursor */
struct WMcursor {
	char *bits;
};
WMcursor *QT_CreateWMCursor(_THIS,
		Uint8 *data, Uint8 *mask, int w, int h, int hot_x, int hot_y)
{
  static WMcursor dummy;
  dummy.bits = 0;
  return &dummy;
}

int QT_ShowWMCursor(_THIS, WMcursor *cursor)
{
  return 1;
}

void QT_FreeWMCursor(_THIS, WMcursor *cursor)
{
}

void QT_WarpWMCursor(_THIS, Uint16 x, Uint16 y)
{
  SDL_Win->setMousePos(QPoint(x, y));
}

}; /* Extern C */
