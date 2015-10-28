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

     Implements Sprite plotting code for wimp display.window
*/

#include "kernel.h"
#include "swis.h"

#include "SDL_stdinc.h"
#include "SDL_riscosvideo.h"

extern void WIMP_ReadModeInfo(_THIS);

void WIMP_PaletteChanged(_THIS);


/* Create sprite buffer for screen */

unsigned char *WIMP_CreateBuffer(int width, int height, int bpp)
{
	int size;
	char sprite_name[12] = "display";
	unsigned char *buffer;
	_kernel_swi_regs regs;
	int bytesPerPixel;
	int bytesPerRow;
	int offsetToSpriteData = 60;

	switch(bpp)
	{
	case 32: bytesPerPixel = 4; break;
	case 16: bytesPerPixel = 2; break;
	case 8:
	    bytesPerPixel = 1;
	    offsetToSpriteData += 2048; /* Add in size of palette */
	    break;
	default:
		return NULL;
		break;
	}

	bytesPerRow = bytesPerPixel * width;

	if ((bytesPerRow & 3) != 0)
	{
		bytesPerRow += 4 - (bytesPerRow & 3);
	}
	size = bytesPerRow * height;

	buffer = SDL_malloc( (size_t) size + offsetToSpriteData );
	if (!buffer) return NULL;

   /* Initialise a sprite area */

	*(unsigned int *)buffer		= size + offsetToSpriteData;
	*(unsigned int *)(buffer + 8)	= 16;

	regs.r[0] = 256+9;
	regs.r[1] = (unsigned int)buffer;
   _kernel_swi(OS_SpriteOp, &regs, &regs);

	regs.r[0] = 256+15;
	regs.r[1] = (unsigned int)buffer;
	regs.r[2] = (unsigned int)&sprite_name;
	regs.r[3] = 0; /* Palette flag: 0 = no palette */
	regs.r[4] = width;
	regs.r[5] = height;
	if (bpp == 8)
	{
		/* Use old style mode number */
		regs.r[6] = 28; /* 8bpp 90x90dpi */
	} else
	{
		regs.r[6] = (((bpp == 16) ? 5 : 6) << 27) /* Type 6 = 32bpp sprite, 5 = 16bpp sprite */
					| (90 << 14) /* Vertical dpi */
					| (90 << 1)  /* Horizontal dpi */
					| 1; /* Marker to distinguish between mode selectors and sprite modes */
	}
   if (_kernel_swi(OS_SpriteOp, &regs, &regs) == NULL)
   {
       if (bpp == 8)
       {
          /* Modify sprite to take into account 256 colour palette */
          int *sprite = (int *)(buffer + 16);
          /* Adjust sprite offsets */
          sprite[0] += 2048;
          sprite[8] += 2048;
          sprite[9] += 2048;
          /* Adjust sprite area next free pointer */
          (*(int *)(buffer+12)) += 2048;

          /* Don't need to set up palette as SDL sets up the default
             256 colour palette */
/*          {
             int *pal = sprite + 11;
             unsigned int j;
             unsigned int entry;
             for (j = 0; j < 255; j++)
             {
                entry = (j << 24) | (j << 16) | (j << 8);
                *pal++ = entry;
                *pal++ = entry;
             }
          }
*/
       }
   } else
   {
      SDL_free(buffer);
      buffer = NULL;
   }

   return buffer;
}


/* Setup translation buffers for the sprite plotting */

void WIMP_SetupPlotInfo(_THIS)
{
   _kernel_swi_regs regs;
   int *sprite = ((int *)this->hidden->bank[1])+4;

   regs.r[0] = (unsigned int)this->hidden->bank[1];
   regs.r[1] = (unsigned int)sprite;
   regs.r[2] = -1; /* Current mode */
   regs.r[3] = -1; /* Current palette */
   regs.r[4] = 0; /* Get size of buffer */
   regs.r[5] = 1|2|16; /* R1 - pointer to sprite and can use full palette words */
   regs.r[6] = 0;
   regs.r[7] = 0;

   if (this->hidden->pixtrans) SDL_free(this->hidden->pixtrans);
   this->hidden->pixtrans = 0;

   /* Get the size required for the buffer */
   _kernel_swi(ColourTrans_GenerateTable, &regs, &regs);
   if (regs.r[4])
   {
      this->hidden->pixtrans = SDL_malloc(regs.r[4]);
    
      regs.r[4] = (unsigned int)this->hidden->pixtrans;
      /* Actually read the buffer */
      _kernel_swi(ColourTrans_GenerateTable, &regs, &regs);
   }
}

/* Plot the sprite in the given context */
void WIMP_PlotSprite(_THIS, int x, int y)
{
   _kernel_swi_regs regs;
   _kernel_oserror *err;

   regs.r[0] =  52 + 512;
   regs.r[1] = (unsigned int)this->hidden->bank[1];
   regs.r[2] = (unsigned int)this->hidden->bank[1]+16;
   regs.r[3] = x;
   regs.r[4] = y;
   regs.r[5] = 0|32; /* Overwrite screen and pixtrans contains wide colour entries */
   regs.r[6] = 0; /* No scale factors i.e. 1:1 */
   regs.r[7] = (int)this->hidden->pixtrans;

   if ((err = _kernel_swi(OS_SpriteOp, &regs, &regs)) != 0)
   {
      int *p = (int *)this->hidden->pixtrans;
      printf("OS_SpriteOp failed \n%s\n",err->errmess);
      printf("pixtrans %d\n", (int)this->hidden->pixtrans);
      printf("%x %x %x\n", p[0], p[1], p[2]);
   }
}


/* Wimp mode has changes so update colour mapping and pixel sizes 
   of windows and the sprites they plot */

void WIMP_ModeChanged(_THIS)
{
	int oldXeig = this->hidden->xeig;
	int oldYeig = this->hidden->yeig;

	WIMP_ReadModeInfo(this);

	if (oldXeig == this->hidden->xeig && oldYeig == this->hidden->yeig)
	{
		/* Only need to update the palette */
		WIMP_PaletteChanged(this);
	} else
	{
		_kernel_swi_regs regs;
		int window_state[9];
		int extent[4];
		int currWidth, currHeight;
		int newWidth, newHeight;
		
		/* Need to resize windows and update the palette */
		WIMP_SetupPlotInfo(this);


		window_state[0] = this->hidden->window_handle;
		regs.r[1] = (unsigned int)window_state;
		_kernel_swi(Wimp_GetWindowState, &regs, &regs);
						
		currWidth = window_state[3] - window_state[1];
		currHeight = window_state[4] - window_state[2];
		
		newWidth = (currWidth >> oldXeig) << this->hidden->xeig;
		newHeight = (currHeight >> oldYeig) << this->hidden->yeig;
		/* Need to avoid extent getting too small for visible part
		of window */
		extent[0] = 0;
		if (currHeight <= newHeight)
		{
			extent[1] = -newHeight;
		} else
		{
			extent[1] = -currHeight;
		}
		if (currWidth <= newWidth)
		{
			extent[2] = newWidth;
		} else
		{
			extent[2] = currWidth;
		}
		extent[3] = 0;
		
		regs.r[0] = this->hidden->window_handle;
		regs.r[1] = (int)extent;
		_kernel_swi(Wimp_SetExtent, &regs, &regs);

		/*TODO: May need to set flag to resize window on next open */
	}
}

/* Palette has changed so update palettes used for windows sprites */

void WIMP_PaletteChanged(_THIS)
{
	WIMP_SetupPlotInfo(this);
}
