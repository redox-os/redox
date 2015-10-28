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

#include "SDL_mouse.h"
#include "../../events/SDL_events_c.h"
#include "../SDL_cursor_c.h"
#include "SDL_ph_mouse_c.h"

struct WMcursor
{
    PhCursorDef_t *ph_cursor ;
};

void ph_FreeWMCursor(_THIS, WMcursor *cursor)
{
    if (window != NULL)
    {
        SDL_Lock_EventThread();

        if (PtSetResource(window, Pt_ARG_CURSOR_TYPE, Ph_CURSOR_INHERIT, 0) < 0)
        {
            /* TODO: output error msg */
        }

        SDL_Unlock_EventThread();
    }	

    SDL_free(cursor);
}

WMcursor *ph_CreateWMCursor(_THIS, Uint8 *data, Uint8 *mask, int w, int h, int hot_x, int hot_y)
{
    WMcursor* cursor;
    int clen, i;
    unsigned char bit, databit, maskbit;

    /* Allocate and initialize the cursor memory */
    if ((cursor = (WMcursor*)SDL_malloc(sizeof(WMcursor))) == NULL)
    {
        SDL_OutOfMemory();
        return(NULL);
    }
    SDL_memset(cursor,0,sizeof(WMcursor));

    cursor->ph_cursor = (PhCursorDef_t *) SDL_malloc(sizeof(PhCursorDef_t) + 32*4*2);

    if (cursor->ph_cursor == NULL)
    {
        SDL_SetError("ph_CreateWMCursor(): cursor malloc failed !\n");
        return NULL;
    }

    SDL_memset(cursor->ph_cursor,0,(sizeof(PhCursorDef_t) + 32*4*2));

    cursor->ph_cursor->hdr.type =Ph_RDATA_CURSOR;   
    cursor->ph_cursor->size1.x = (short)w;
    cursor->ph_cursor->size1.y = (short)h;
    cursor->ph_cursor->offset1.x = (short)hot_x;
    cursor->ph_cursor->offset1.y = (short)hot_y;
    cursor->ph_cursor->bytesperline1 = (char)w/8;
    cursor->ph_cursor->color1 = Pg_WHITE;
    cursor->ph_cursor->size2.x = (short)w;
    cursor->ph_cursor->size2.y = (short)h;
    cursor->ph_cursor->offset2.x = (short)hot_x;
    cursor->ph_cursor->offset2.y = (short)hot_y;
    cursor->ph_cursor->bytesperline2 = (char)w/8;
    cursor->ph_cursor->color2 = Pg_BLACK;

    clen = (w/8)*h;

    /* Copy the mask and the data to different bitmap planes */
    for (i=0; i<clen; ++i)
    {
        for (bit = 0; bit < 8; bit++)
        {
            databit = data[i] & (1 << bit);
            maskbit = mask[i] & (1 << bit);

            cursor->ph_cursor->images[i] |= (databit == 0) ? maskbit : 0;
            /* If the databit != 0, treat it as a black pixel and
             * ignore the maskbit (can't do an inverted color) */
            cursor->ph_cursor->images[i+clen] |= databit;
        }
    }

    /* #bytes following the hdr struct */
    cursor->ph_cursor->hdr.len =sizeof(PhCursorDef_t) + clen*2 - sizeof(PhRegionDataHdr_t); 

    return (cursor);
}

PhCursorDef_t ph_GetWMPhCursor(WMcursor *cursor)
{
    return (*cursor->ph_cursor);
}

int ph_ShowWMCursor(_THIS, WMcursor* cursor)
{
    PtArg_t args[3];
    int nargs = 0;

    /* Don't do anything if the display is gone */
    if (window == NULL)
    {
        return (0);
    }

    /* looks like photon can't draw mouse cursor in direct mode */
    if ((this->screen->flags & SDL_FULLSCREEN) == SDL_FULLSCREEN)
    {
         /* disable the fake mouse in the fullscreen OpenGL mode */
         if ((this->screen->flags & SDL_OPENGL) == SDL_OPENGL)
         {
             cursor=NULL;
         }
         else
         {
             return (0);
         }
    }

    /* Set the photon cursor, or blank if cursor is NULL */
    if (cursor!=NULL)
    {
        PtSetArg(&args[0], Pt_ARG_CURSOR_TYPE, Ph_CURSOR_BITMAP, 0);
        /* Could set next to any PgColor_t value */
        PtSetArg(&args[1], Pt_ARG_CURSOR_COLOR, Ph_CURSOR_DEFAULT_COLOR , 0);
        PtSetArg(&args[2], Pt_ARG_BITMAP_CURSOR, cursor->ph_cursor, (cursor->ph_cursor->hdr.len + sizeof(PhRegionDataHdr_t)));
        nargs = 3;
    }
    else /* Ph_CURSOR_NONE */
    {
        PtSetArg(&args[0], Pt_ARG_CURSOR_TYPE, Ph_CURSOR_NONE, 0);
        nargs = 1;
    }

    SDL_Lock_EventThread();

    if (PtSetResources(window, nargs, args) < 0 )
    {
        return (0);
    }	

    SDL_Unlock_EventThread();

    return (1);
}


void ph_WarpWMCursor(_THIS, Uint16 x, Uint16 y)
{
    short abs_x, abs_y;

    SDL_Lock_EventThread();
    PtGetAbsPosition( window, &abs_x, &abs_y );
    PhMoveCursorAbs( PhInputGroup(NULL), x + abs_x, y + abs_y );
    SDL_Unlock_EventThread();
}


void ph_CheckMouseMode(_THIS)
{
    /* If the mouse is hidden and input is grabbed, we use relative mode */
    if ( !(SDL_cursorstate & CURSOR_VISIBLE) && (this->input_grab != SDL_GRAB_OFF))
    {
        mouse_relative = 1;
    }
    else
    {
        mouse_relative = 0;
    }
}


void ph_UpdateMouse(_THIS)
{
    PhCursorInfo_t phcursor;
    short abs_x;
    short abs_y;

    /* Lock the event thread, in multi-threading environments */
    SDL_Lock_EventThread();

    /* synchronizing photon mouse cursor position and SDL mouse position, if cursor appears over window. */
    PtGetAbsPosition(window, &abs_x, &abs_y);
    PhQueryCursor(PhInputGroup(NULL), &phcursor);
    if (((phcursor.pos.x >= abs_x) && (phcursor.pos.x <= abs_x + this->screen->w)) &&
        ((phcursor.pos.y >= abs_y) && (phcursor.pos.y <= abs_y + this->screen->h)))
    {
        SDL_PrivateAppActive(1, SDL_APPMOUSEFOCUS);
        SDL_PrivateMouseMotion(0, 0, phcursor.pos.x-abs_x, phcursor.pos.y-abs_y);
    }
    else
    {
        SDL_PrivateAppActive(0, SDL_APPMOUSEFOCUS);
    }

    /* Unlock the event thread, in multi-threading environments */
    SDL_Unlock_EventThread();
}
