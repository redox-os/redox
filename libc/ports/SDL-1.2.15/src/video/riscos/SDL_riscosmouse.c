/*
    SDL - Simple DirectMedia Layer
    Copyright (C) 1997-2012 Sam Lantinga

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

    Sam Lantinga
    slouken@libsdl.org
*/
#include "SDL_config.h"

/*
     File added by Alan Buckley (alan_baa@hotmail.com) for RISC OS compatability
	 27 March 2003

     Implements mouse cursor shape definitions and positioning
*/

#include "SDL_mouse.h"
#include "../../events/SDL_events_c.h"

#include "SDL_riscosmouse_c.h"

#include "kernel.h"
#include "swis.h"

static WMcursor *current_cursor = NULL;
static WMcursor *defined_cursor = NULL;

extern int mouseInWindow;

/* Area to save cursor palette colours changed by SDL.
   Actual values will be read before we change to the SDL cursor */
static Uint8 wimp_cursor_palette[2][5] = {
  {1, 25, 255, 255, 255},
  {3, 25, 255, 255, 255}
};

static int cursor_palette_saved = 0;

void WIMP_SaveCursorPalette();
void WIMP_RestoreWimpCursor();
void WIMP_SetSDLCursorPalette();


void RISCOS_FreeWMCursor(_THIS, WMcursor *cursor)
{
    SDL_free(cursor->data);
	SDL_free(cursor);
}

WMcursor *RISCOS_CreateWMCursor(_THIS,
		Uint8 *data, Uint8 *mask, int w, int h, int hot_x, int hot_y)
{
	WMcursor *cursor;
	Uint8 *cursor_data;
	Uint8 *ptr;
	int i,j,k;
	int data_byte, mask_byte;

	/* Check to make sure the cursor size is okay */
	if ( (w > 32) || (h > 32) ) {
		SDL_SetError("Only with width and height <= 32 pixels are allowed");
		return(NULL);
	}

	/* Allocate the cursor */
	cursor = (WMcursor *)SDL_malloc(sizeof(*cursor));
	if ( cursor == NULL ) {
		SDL_SetError("Out of memory");
		return(NULL);
	}

	/* Note: SDL says width must be a multiple of 8 */
	cursor_data = SDL_malloc(w/4 * h);
	if (cursor_data == NULL)
	{
		SDL_free(cursor);
		SDL_SetError("Out of memory");
		return(NULL);
	}

	cursor->w = w;
	cursor->h = h;
	cursor->hot_x = hot_x;
	cursor->hot_y = hot_y;
	cursor->data = cursor_data;


/* Data / Mask Resulting pixel on screen 
   0 / 1 White 
   1 / 1 Black 
   0 / 0 Transparent 
   1 / 0 Inverted color if possible, black if not. 
*/
	ptr = cursor_data;

	for ( i=0; i<h; ++i )
	{
		for (j = 0; j < w/8; ++j)
		{
			data_byte = *data;
			mask_byte = *mask;
			*ptr++ = 0; /* Sets whole byte transparent */
			*ptr = 0;
			for (k = 0; k < 8; k++)
			{
				(*ptr) <<= 2;
				if (data_byte & 1) *ptr |= 3; /* Black or inverted */
				else if(mask_byte & 1) *ptr |= 1; /* White */
				if ((k&3) == 3) ptr--;
				data_byte >>= 1;
				mask_byte >>= 1;
			}

            ptr+=3;
		    data++;
		    mask++;
		}
	}

	return(cursor);
}

int RISCOS_ShowWMCursor(_THIS, WMcursor *cursor)
{
	current_cursor = cursor;

	if (cursor == NULL)
	{
		_kernel_osbyte(106,0,0);
		defined_cursor = NULL;
	} else
	{
        WMcursor *old_cursor = defined_cursor;

		if (cursor != defined_cursor)
		{
			Uint8 cursor_def[10];

			cursor_def[0] = 0;
			cursor_def[1] = 2; /* Use shape number 2 */
			cursor_def[2] = cursor->w/4; /* Width in bytes */
			cursor_def[3] = cursor->h; /* Height (h) in pixels */
			cursor_def[4] = cursor->hot_x; /* ActiveX in pixels from left */
			cursor_def[5] = cursor->hot_y; /* ActiveY in pixels from top */
			cursor_def[6] = ((int)(cursor->data) & 0xFF);       /* Least significant byte of pointer to data */
			cursor_def[7] = ((int)(cursor->data) >> 8) & 0xFF;  /* ... */
			cursor_def[8] = ((int)(cursor->data) >> 16) & 0xFF; /* ... */
			cursor_def[9] = ((int)(cursor->data) >> 24) & 0xFF; /* Most significant byte of pointer to data */

			if (_kernel_osword(21, (int *)cursor_def) != 0)
			{
				SDL_SetError("RISCOS couldn't create the cursor to show");
				return(0);
			}
			defined_cursor = cursor;
		}

        if (old_cursor == NULL)
        {
            /* First time or reshow in window, so save/setup palette */
            if (!cursor_palette_saved)
            {
                WIMP_SaveCursorPalette();
            }
            WIMP_SetSDLCursorPalette();
        }

        _kernel_osbyte(106, 2, 0);        
	}
	
	return(1);
}

void FULLSCREEN_WarpWMCursor(_THIS, Uint16 x, Uint16 y)
{
	Uint8 move_block[5];
	int eig_block[3];
	_kernel_swi_regs regs;
	int os_x, os_y;

	eig_block[0] = 4;  /* X eig factor */
	eig_block[1] = 5;  /* Y eig factor */
	eig_block[2] = -1;  /* End of list of variables to request */

    regs.r[0] = (int)eig_block;
    regs.r[1] = (int)eig_block;
    _kernel_swi(OS_ReadVduVariables, &regs, &regs);

	os_x = x << eig_block[0];
	os_y = y << eig_block[1];

	move_block[0] = 3; /* Move cursor */
	move_block[1] = os_x & 0xFF;
	move_block[2] = (os_x >> 8) & 0xFF;
	move_block[3] = os_y & 0xFF;
	move_block[4] = (os_y >> 8) & 0xFF;

	_kernel_osword(21, (int *)move_block);
	SDL_PrivateMouseMotion(0, 0, x, y);
}


/* Reshow cursor when mouse re-enters the window */
void WIMP_ReshowCursor(_THIS)
{
	defined_cursor = NULL;
    cursor_palette_saved = 0;
	RISCOS_ShowWMCursor(this, current_cursor);
}

void WIMP_WarpWMCursor(_THIS, Uint16 x, Uint16 y)
{
	_kernel_swi_regs regs;
	int window_state[9];
	char block[5];
	int osX, osY;

	window_state[0] = this->hidden->window_handle;
	regs.r[1] = (unsigned int)window_state;
	_kernel_swi(Wimp_GetWindowState, &regs, &regs);

	 osX = (x << this->hidden->xeig) + window_state[1];
	 osY = window_state[4] - (y << this->hidden->yeig);

	block[0] = 3;
	block[1] = osX & 0xFF;
	block[2] = (osX >> 8) & 0xFF;
	block[3] = osY & 0xFF;
	block[4] = (osY >> 8) & 0xFF;

	regs.r[0] = 21;
	regs.r[1] = (int)block;
	_kernel_swi(OS_Word, &regs, &regs);
	SDL_PrivateMouseMotion(0, 0, x, y);
}

int WIMP_ShowWMCursor(_THIS, WMcursor *cursor)
{
	if (mouseInWindow) return RISCOS_ShowWMCursor(this, cursor);
	else current_cursor = cursor;

	return 1;
}

SDL_GrabMode RISCOS_GrabInput(_THIS, SDL_GrabMode mode)
{
   /* In fullscreen mode we don't need to do anything */
   if (mode < SDL_GRAB_FULLSCREEN)
   {
      _kernel_swi_regs regs;
      unsigned char block[9];
      block[0] = 1; /* Define mouse cursor bounding block */

      if ( mode == SDL_GRAB_OFF )
      {
         /* Clip to whole screen */

         int r = (this->hidden->screen_width << this->hidden->xeig) - 1;
         int t = (this->hidden->screen_height << this->hidden->yeig) - 1;

	 block[1] = 0; block[2] = 0; /* Left*/
         block[3] = 0; block[4] = 0; /* Bottom */
         block[5] = r & 0xFF; block[6] = (r >> 8) & 0xFF; /* Right */
         block[7] = t & 0xFF; block[8] = (t >> 8) & 0xFF; /* Top */
      } else
      {
        /* Clip to window */
       	unsigned char window_state[36];

	*((int *)window_state) = this->hidden->window_handle;
	regs.r[1] = (unsigned int)window_state;
	_kernel_swi(Wimp_GetWindowState, &regs, &regs);

        block[1] = window_state[4];
        block[2] = window_state[5];
        block[3] = window_state[8];
        block[4] = window_state[9];
        block[5] = window_state[12];
        block[6] = window_state[13];
        block[7] = window_state[16];
        block[8] = window_state[17];

      }

      regs.r[0] = 21; /* OS word code */
      regs.r[1] = (int)block;
      _kernel_swi(OS_Word, &regs, &regs);
   }

   return mode;
}

/* Save mouse cursor palette to be restore when we are no longer
   defining a cursor */

void WIMP_SaveCursorPalette()
{
    _kernel_swi_regs regs;
    int colour;

    for (colour = 0; colour < 2; colour++)
    {
      regs.r[0] = (int)wimp_cursor_palette[colour][0];
      regs.r[1] = 25;
      /* Read settings with OS_ReadPalette */
      if (_kernel_swi(0x2f, &regs, &regs) == NULL)
      {
        wimp_cursor_palette[colour][2] = (unsigned char)((regs.r[2] >> 8) & 0xFF);
        wimp_cursor_palette[colour][3] = (unsigned char)((regs.r[2] >> 16) & 0xFF);
        wimp_cursor_palette[colour][4] = (unsigned char)((regs.r[2] >> 24) & 0xFF);
      }
    }

    cursor_palette_saved = 1;
}

/* Restore the WIMP's cursor when we leave the SDL window */
void WIMP_RestoreWimpCursor()
{
    int colour;

    /* Reset to pointer shape 1 */
    _kernel_osbyte(106, 1, 0);

    /* Reset pointer colours */
    if (cursor_palette_saved)
    {
      for (colour = 0; colour < 2; colour++)
      {
        _kernel_osword(12, (int *)wimp_cursor_palette[colour]);
      }
    }
    cursor_palette_saved = 0;
}

/* Set palette used for SDL mouse cursors */
void WIMP_SetSDLCursorPalette()
{
  /* First time set up the mouse colours */
  Uint8 block[5];

  /* Set up colour 1 as white */
  block[0] = 1;   /* Colour to change 1 - 3 */
  block[1] = 25;  /* Set pointer colour */
  block[2] = 255; /* red component*/
  block[3] = 255; /* green component */
  block[4] = 255; /* blue component*/
 _kernel_osword(12, (int *)block);
		
 /* Set colour 3 to back */
 block[0] = 3;   /* Colour to change 1 - 3 */
 block[1] = 25;  /* Set pointer colour*/
 block[2] = 0; /* red component*/
 block[3] = 0; /* green component */
 block[4] = 0; /* blue component*/
 _kernel_osword(12, (int *)block);
}
