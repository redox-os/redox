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

#include <Ph.h>
#include <photon/PpProto.h>
#include <photon/PhWm.h>
#include <photon/wmapi.h>

#include "SDL_version.h"
#include "SDL_timer.h"
#include "SDL_video.h"
#include "SDL_syswm.h"
#include "../SDL_pixels_c.h"
#include "../../events/SDL_events_c.h"
#include "SDL_ph_modes_c.h"
#include "SDL_ph_wm_c.h"

void ph_SetIcon(_THIS, SDL_Surface *icon, Uint8 *mask)
{
    return;
}

/* Set window caption */
void ph_SetCaption(_THIS, const char *title, const char *icon)
{
    SDL_Lock_EventThread();

    /* sanity check for set caption call before window init */
    if (window!=NULL)
    {
        PtSetResource(window, Pt_ARG_WINDOW_TITLE, title, 0);
    }

    SDL_Unlock_EventThread();
}

/* Iconify current window */
int ph_IconifyWindow(_THIS)
{
    PhWindowEvent_t windowevent;

    SDL_Lock_EventThread();

    SDL_memset(&windowevent, 0, sizeof(windowevent));
    windowevent.event_f = Ph_WM_HIDE;
    windowevent.event_state = Ph_WM_EVSTATE_HIDE;
    windowevent.rid = PtWidgetRid(window);
    PtForwardWindowEvent(&windowevent);

    SDL_Unlock_EventThread();

    return 0;
}

SDL_GrabMode ph_GrabInputNoLock(_THIS, SDL_GrabMode mode)
{
    short abs_x, abs_y;

    if( mode == SDL_GRAB_OFF )
    {
        PtSetResource(window, Pt_ARG_WINDOW_STATE, Pt_FALSE, Ph_WM_STATE_ISALTKEY);
    }
    else
    {
        PtSetResource(window, Pt_ARG_WINDOW_STATE, Pt_TRUE, Ph_WM_STATE_ISALTKEY);

        PtGetAbsPosition(window, &abs_x, &abs_y);
        PhMoveCursorAbs(PhInputGroup(NULL), abs_x + SDL_VideoSurface->w/2, abs_y + SDL_VideoSurface->h/2);
    }

    SDL_Unlock_EventThread();

    return(mode);
}

SDL_GrabMode ph_GrabInput(_THIS, SDL_GrabMode mode)
{
    SDL_Lock_EventThread();
    mode = ph_GrabInputNoLock(this, mode);
    SDL_Unlock_EventThread();

    return(mode);
}


int ph_GetWMInfo(_THIS, SDL_SysWMinfo *info)
{
    if (info->version.major <= SDL_MAJOR_VERSION)
    {
        return 1;
    }
    else
    {
        SDL_SetError("Application not compiled with SDL %d.%d\n",
                      SDL_MAJOR_VERSION, SDL_MINOR_VERSION);
        return -1;
    }
}
