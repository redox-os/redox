/*
  SDL_image:  An example image loading library for use with SDL
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

/*
 * PNM (portable anymap) image loader:
 *
 * Supports: PBM, PGM and PPM, ASCII and binary formats
 * (PBM and PGM are loaded as 8bpp surfaces)
 * Does not support: maximum component value > 255
 */

#include <stdio.h>
#include <stdlib.h>
#include <ctype.h>
#include <string.h>

#include "SDL_image.h"

#ifdef LOAD_PNM

/* See if an image is contained in a data source */
int IMG_isPNM(SDL_RWops *src)
{
	int start;
	int is_PNM;
	char magic[2];

	if ( !src )
		return 0;
	start = SDL_RWtell(src);
	is_PNM = 0;
	if ( SDL_RWread(src, magic, sizeof(magic), 1) ) {
		/*
		 * PNM magic signatures:
		 * P1	PBM, ascii format
		 * P2	PGM, ascii format
		 * P3	PPM, ascii format
		 * P4	PBM, binary format
		 * P5	PGM, binary format
		 * P6	PPM, binary format
		 * P7	PAM, a general wrapper for PNM data
		 */
		if ( magic[0] == 'P' && magic[1] >= '1' && magic[1] <= '6' ) {
			is_PNM = 1;
		}
	}
	SDL_RWseek(src, start, RW_SEEK_SET);
	return(is_PNM);
}

/* read a non-negative integer from the source. return -1 upon error */
static int ReadNumber(SDL_RWops *src)
{
	int number;
	unsigned char ch;

	/* Initialize return value */
	number = 0;

	/* Skip leading whitespace */
	do {
		if ( ! SDL_RWread(src, &ch, 1, 1) ) {
			return(0);
		}
		/* Eat comments as whitespace */
		if ( ch == '#' ) {  /* Comment is '#' to end of line */
			do {
				if ( ! SDL_RWread(src, &ch, 1, 1) ) {
					return -1;
				}
			} while ( (ch != '\r') && (ch != '\n') );
		}
	} while ( isspace(ch) );

	/* Add up the number */
	do {
		number *= 10;
		number += ch-'0';

		if ( !SDL_RWread(src, &ch, 1, 1) ) {
			return -1;
		}
	} while ( isdigit(ch) );

	return(number);
}

SDL_Surface *IMG_LoadPNM_RW(SDL_RWops *src)
{
	int start;
	SDL_Surface *surface = NULL;
	int width, height;
	int maxval, y, bpl;
	Uint8 *row;
	Uint8 *buf = NULL;
	char *error = NULL;
	Uint8 magic[2];
	int ascii;
	enum { PBM, PGM, PPM, PAM } kind;

#define ERROR(s) do { error = (s); goto done; } while(0)

	if ( !src ) {
		/* The error message has been set in SDL_RWFromFile */
		return NULL;
	}
	start = SDL_RWtell(src);

	SDL_RWread(src, magic, 2, 1);
	kind = magic[1] - '1';
	ascii = 1;
	if(kind >= 3) {
		ascii = 0;
		kind -= 3;
	}

	width = ReadNumber(src);
	height = ReadNumber(src);
	if(width <= 0 || height <= 0)
		ERROR("Unable to read image width and height");

	if(kind != PBM) {
		maxval = ReadNumber(src);
		if(maxval <= 0 || maxval > 255)
			ERROR("unsupported PNM format");
	} else
		maxval = 255;	/* never scale PBMs */

	/* binary PNM allows just a single character of whitespace after
	   the last parameter, and we've already consumed it */

	if(kind == PPM) {
		/* 24-bit surface in R,G,B byte order */
		surface = SDL_AllocSurface(SDL_SWSURFACE, width, height, 24,
#if SDL_BYTEORDER == SDL_LIL_ENDIAN
					   0x000000ff, 0x0000ff00, 0x00ff0000,
#else
					   0x00ff0000, 0x0000ff00, 0x000000ff,
#endif
					   0);
	} else {
		/* load PBM/PGM as 8-bit indexed images */
		surface = SDL_AllocSurface(SDL_SWSURFACE, width, height, 8,
					   0, 0, 0, 0);
	}
	if ( surface == NULL )
		ERROR("Out of memory");
	bpl = width * surface->format->BytesPerPixel;
	if(kind == PGM) {
		SDL_Color *c = surface->format->palette->colors;
		int i;
		for(i = 0; i < 256; i++)
			c[i].r = c[i].g = c[i].b = i;
		surface->format->palette->ncolors = 256;
	} else if(kind == PBM) {
		/* for some reason PBM has 1=black, 0=white */
		SDL_Color *c = surface->format->palette->colors;
		c[0].r = c[0].g = c[0].b = 255;
		c[1].r = c[1].g = c[1].b = 0;
		surface->format->palette->ncolors = 2;
		bpl = (width + 7) >> 3;
		buf = malloc(bpl);
		if(buf == NULL)
			ERROR("Out of memory");
	}

	/* Read the image into the surface */
	row = surface->pixels;
	for(y = 0; y < height; y++) {
		if(ascii) {
			int i;
			if(kind == PBM) {
				for(i = 0; i < width; i++) {
					Uint8 ch;
					do {
						if(!SDL_RWread(src, &ch,
							       1, 1))
						       ERROR("file truncated");
						ch -= '0';
					} while(ch > 1);
					row[i] = ch;
				}
			} else {
				for(i = 0; i < bpl; i++) {
					int c;
					c = ReadNumber(src);
					if(c < 0)
						ERROR("file truncated");
					row[i] = c;
				}
			}
		} else {
			Uint8 *dst = (kind == PBM) ? buf : row;
			if(!SDL_RWread(src, dst, bpl, 1))
				ERROR("file truncated");
			if(kind == PBM) {
				/* expand bitmap to 8bpp */
				int i;
				for(i = 0; i < width; i++) {
					int bit = 7 - (i & 7);
					row[i] = (buf[i >> 3] >> bit) & 1;
				}
			}
		}
		if(maxval < 255) {
			/* scale up to full dynamic range (slow) */
			int i;
			for(i = 0; i < bpl; i++)
				row[i] = row[i] * 255 / maxval;
		}
		row += surface->pitch;
	}
done:
	free(buf);
	if(error) {
		SDL_RWseek(src, start, RW_SEEK_SET);
		if ( surface ) {
			SDL_FreeSurface(surface);
			surface = NULL;
		}
		IMG_SetError(error);
	}
	return(surface);
}

#else

/* See if an image is contained in a data source */
int IMG_isPNM(SDL_RWops *src)
{
	return(0);
}

/* Load a PNM type image from an SDL datasource */
SDL_Surface *IMG_LoadPNM_RW(SDL_RWops *src)
{
	return(NULL);
}

#endif /* LOAD_PNM */
