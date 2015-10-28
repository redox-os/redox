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

/* This is a generic "format not supported" image framework */

#include <stdio.h>

#include "SDL_image.h"

#ifdef LOAD_XXX

/* See if an image is contained in a data source */
int IMG_isXXX(SDL_RWops *src)
{
	int start;
	int is_XXX;

	if ( !src )
		return 0;
	start = SDL_RWtell(src);
	is_XXX = 0;

	/* Detect the image here */

	SDL_RWseek(src, start, RW_SEEK_SET);
	return(is_XXX);
}

/* Load a XXX type image from an SDL datasource */
SDL_Surface *IMG_LoadXXX_RW(SDL_RWops *src)
{
	int start;
	const char *error = NULL;
	SDL_Surface *surface = NULL;

	if ( !src ) {
		/* The error message has been set in SDL_RWFromFile */
		return NULL;
	}
	start = SDL_RWtell(src);

	/* Load the image here */

	if ( error ) {
		SDL_RWseek(src, start, RW_SEEK_SET);
		if ( surface ) {
			SDL_FreeSurface(surface);
			surface = NULL;
		}
		IMG_SetError(error);
	}
	return surface;
}

#else

/* See if an image is contained in a data source */
int IMG_isXXX(SDL_RWops *src)
{
	return(0);
}

/* Load a XXX type image from an SDL datasource */
SDL_Surface *IMG_LoadXXX_RW(SDL_RWops *src)
{
	return(NULL);
}

#endif /* LOAD_XXX */
