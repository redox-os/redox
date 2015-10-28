/*
  showimage:  A test application for the SDL image loading library.
  Copyright (C) 1997-2012 Sam Lantinga <slouken@libsdl.org>

  This software is provided 'as-is', without any express or implied
  warranty.  In no event will the authors be held liable for any damages
  arising from the use of this software.

  Permission is granted to anyone to use this software for any purpose,
  including commercial applications, and to alter it and redistribute it
  freely, subject to the following restrictions:

  1. The origin of this software must not be misrepresented; you must not
     claim that you wrote the original software. If you use this software
     in a product, an acknowledgment in the product documentation would be
     appreciated but is not required.
  2. Altered source versions must be plainly marked as such, and must not be
     misrepresented as being the original software.
  3. This notice may not be removed or altered from any source distribution.
*/

#include <stdlib.h>
#include <stdio.h>
#include <string.h>

#include "SDL.h"
#include "SDL_image.h"

/* #define XPM_INCLUDED and supply picture.xpm to test the XPM inclusion
   feature */

#ifdef XPM_INCLUDED
#include "picture.xpm"
#endif

/* Draw a Gimpish background pattern to show transparency in the image */
static void draw_background(SDL_Surface *screen)
{
    Uint8 *dst = screen->pixels;
    int x, y;
    int bpp = screen->format->BytesPerPixel;
    Uint32 col[2];
    col[0] = SDL_MapRGB(screen->format, 0x66, 0x66, 0x66);
    col[1] = SDL_MapRGB(screen->format, 0x99, 0x99, 0x99);
    for(y = 0; y < screen->h; y++) {
	for(x = 0; x < screen->w; x++) {
	    /* use an 8x8 checkerboard pattern */
	    Uint32 c = col[((x ^ y) >> 3) & 1];
	    switch(bpp) {
	    case 1:
		dst[x] = (Uint8)c;
		break;
	    case 2:
		((Uint16 *)dst)[x] = (Uint16)c;
		break;
	    case 3:
#if SDL_BYTEORDER == SDL_LIL_ENDIAN
		dst[x * 3]     = (Uint8)(c);
		dst[x * 3 + 1] = (Uint8)(c >> 8);
		dst[x * 3 + 2] = (Uint8)(c >> 16);
#else
		dst[x * 3]     = (Uint8)(c >> 16);
		dst[x * 3 + 1] = (Uint8)(c >> 8);
		dst[x * 3 + 2] = (Uint8)(c);
#endif
		break;
	    case 4:
		((Uint32 *)dst)[x] = c;
		break;
	    }
	}
	dst += screen->pitch;
    }
}

int main(int argc, char *argv[])
{
	Uint32 flags;
	SDL_Surface *screen, *image;
	int i, depth, done;
	SDL_Event event;
#if 0
	SDL_RWops* rw_ops;
#endif

	/* Check command line usage */
	if ( ! argv[1] ) {
		fprintf(stderr, "Usage: %s <image_file>\n", argv[0]);
		return(1);
	}

	/* Initialize the SDL library */
	if ( SDL_Init(SDL_INIT_VIDEO) < 0 ) {
		fprintf(stderr, "Couldn't initialize SDL: %s\n",SDL_GetError());
		return(255);
	}

	flags = SDL_SWSURFACE;
	for ( i=1; argv[i]; ++i ) {
		if ( strcmp(argv[i], "-fullscreen") == 0 ) {
			SDL_ShowCursor(0);
			flags |= SDL_FULLSCREEN;
			continue;
		}
#if 0
		rw_ops = SDL_RWFromFile(argv[1], "r");
		
		fprintf(stderr, "BMP:\t%d\n", IMG_isBMP(rw_ops));
		fprintf(stderr, "GIF:\t%d\n", IMG_isGIF(rw_ops));
		fprintf(stderr, "JPG:\t%d\n", IMG_isJPG(rw_ops));
		fprintf(stderr, "PNG:\t%d\n", IMG_isPNG(rw_ops));
		fprintf(stderr, "TIF:\t%d\n", IMG_isTIF(rw_ops));
		/* fprintf(stderr, "TGA:\t%d\n", IMG_isTGA(rw_ops)); */
		fprintf(stderr, "PCX:\t%d\n", IMG_isPCX(rw_ops));
#endif

		/* Open the image file */
#ifdef XPM_INCLUDED
		image = IMG_ReadXPMFromArray(picture_xpm);
#else
		image = IMG_Load(argv[i]);
#endif
		if ( image == NULL ) {
			fprintf(stderr, "Couldn't load %s: %s\n",
			        argv[i], SDL_GetError());
			continue;
		}
		SDL_WM_SetCaption(argv[i], "showimage");

		/* Create a display for the image */
		depth = SDL_VideoModeOK(image->w, image->h, 32, flags);
		/* Use the deepest native mode, except that we emulate 32bpp
		   for viewing non-indexed images on 8bpp screens */
		if ( depth == 0 ) {
			if ( image->format->BytesPerPixel > 1 ) {
				depth = 32;
			} else {
				depth = 8;
			}
		} else
		if ( (image->format->BytesPerPixel > 1) && (depth == 8) ) {
	    		depth = 32;
		}
		if(depth == 8)
			flags |= SDL_HWPALETTE;
		screen = SDL_SetVideoMode(image->w, image->h, depth, flags);
		if ( screen == NULL ) {
			fprintf(stderr,"Couldn't set %dx%dx%d video mode: %s\n",
				image->w, image->h, depth, SDL_GetError());
			continue;
		}

		/* Set the palette, if one exists */
		if ( image->format->palette ) {
			SDL_SetColors(screen, image->format->palette->colors,
			              0, image->format->palette->ncolors);
		}

		/* Draw a background pattern if the surface has transparency */
		if(image->flags & (SDL_SRCALPHA | SDL_SRCCOLORKEY))
	    		draw_background(screen);

		/* Display the image */
		SDL_BlitSurface(image, NULL, screen, NULL);
		SDL_UpdateRect(screen, 0, 0, 0, 0);

		done = 0;
		while ( ! done ) {
			if ( SDL_PollEvent(&event) ) {
				switch (event.type) {
				    case SDL_KEYUP:
					switch (event.key.keysym.sym) {
					    case SDLK_LEFT:
						if ( i > 1 ) {
							i -= 2;
							done = 1;
						}
						break;
					    case SDLK_RIGHT:
						if ( argv[i+1] ) {
							done = 1;
						}
						break;
					    case SDLK_ESCAPE:
					    case SDLK_q:
						argv[i+1] = NULL;
						/* Drop through to done */
					    case SDLK_SPACE:
					    case SDLK_TAB:
						done = 1;
						break;
					    default:
						break;
					}
					break;
				    case SDL_MOUSEBUTTONDOWN:
					done = 1;
					break;
                                    case SDL_QUIT:
					argv[i+1] = NULL;
					done = 1;
					break;
				    default:
					break;
				}
			} else {
				SDL_Delay(10);
			}
		}
		SDL_FreeSurface(image);
	}

	/* We're done! */
	SDL_Quit();
	return(0);
}
