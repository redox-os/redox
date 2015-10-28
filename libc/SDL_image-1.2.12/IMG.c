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

/* A simple library to load images of various formats as SDL surfaces */

#include <stdio.h>
#include <string.h>
#include <ctype.h>

#include "SDL_image.h"

#define ARRAYSIZE(a) (sizeof(a) / sizeof((a)[0]))

/* Table of image detection and loading functions */
static struct {
	char *type;
	int (SDLCALL *is)(SDL_RWops *src);
	SDL_Surface *(SDLCALL *load)(SDL_RWops *src);
} supported[] = {
	/* keep magicless formats first */
	{ "TGA", NULL,      IMG_LoadTGA_RW },
	{ "CUR", IMG_isCUR, IMG_LoadCUR_RW },
	{ "ICO", IMG_isICO, IMG_LoadICO_RW },
	{ "BMP", IMG_isBMP, IMG_LoadBMP_RW },
	{ "GIF", IMG_isGIF, IMG_LoadGIF_RW },
	{ "JPG", IMG_isJPG, IMG_LoadJPG_RW },
	{ "LBM", IMG_isLBM, IMG_LoadLBM_RW },
	{ "PCX", IMG_isPCX, IMG_LoadPCX_RW },
	{ "PNG", IMG_isPNG, IMG_LoadPNG_RW },
	{ "PNM", IMG_isPNM, IMG_LoadPNM_RW }, /* P[BGP]M share code */
	{ "TIF", IMG_isTIF, IMG_LoadTIF_RW },
	{ "XCF", IMG_isXCF, IMG_LoadXCF_RW },
	{ "XPM", IMG_isXPM, IMG_LoadXPM_RW },
	{ "XV",  IMG_isXV,  IMG_LoadXV_RW  },
	{ "WEBP", IMG_isWEBP, IMG_LoadWEBP_RW },
};

const SDL_version *IMG_Linked_Version(void)
{
	static SDL_version linked_version;
	SDL_IMAGE_VERSION(&linked_version);
	return(&linked_version);
}

extern int IMG_InitJPG();
extern void IMG_QuitJPG();
extern int IMG_InitPNG();
extern void IMG_QuitPNG();
extern int IMG_InitTIF();
extern void IMG_QuitTIF();

extern int IMG_InitWEBP();
extern void IMG_QuitWEBP();

static int initialized = 0;

int IMG_Init(int flags)
{
	int result = 0;

	if (flags & IMG_INIT_JPG) {
		if ((initialized & IMG_INIT_JPG) || IMG_InitJPG() == 0) {
			result |= IMG_INIT_JPG;
		}
	}
	if (flags & IMG_INIT_PNG) {
		if ((initialized & IMG_INIT_PNG) || IMG_InitPNG() == 0) {
			result |= IMG_INIT_PNG;
		}
	}
	if (flags & IMG_INIT_TIF) {
		if ((initialized & IMG_INIT_TIF) || IMG_InitTIF() == 0) {
			result |= IMG_INIT_TIF;
		}
	}
	if (flags & IMG_INIT_WEBP) {
		if ((initialized & IMG_INIT_WEBP) || IMG_InitWEBP() == 0) {
			result |= IMG_INIT_WEBP;
		}
	}
	initialized |= result;

	return (initialized);
}

void IMG_Quit()
{
	if (initialized & IMG_INIT_JPG) {
		IMG_QuitJPG();
	}
	if (initialized & IMG_INIT_PNG) {
		IMG_QuitPNG();
	}
	if (initialized & IMG_INIT_TIF) {
		IMG_QuitTIF();
	}
	if (initialized & IMG_INIT_WEBP) {
		IMG_QuitWEBP();
	}
	initialized = 0;
}

#if !defined(__APPLE__) || defined(SDL_IMAGE_USE_COMMON_BACKEND)
/* Load an image from a file */
SDL_Surface *IMG_Load(const char *file)
{
    SDL_RWops *src = SDL_RWFromFile(file, "rb");
    char *ext = strrchr(file, '.');
    if(ext) {
        ext++;
    }
    if(!src) {
        /* The error message has been set in SDL_RWFromFile */
        return NULL;
    }
    return IMG_LoadTyped_RW(src, 1, ext);
}
#endif

/* Load an image from an SDL datasource (for compatibility) */
SDL_Surface *IMG_Load_RW(SDL_RWops *src, int freesrc)
{
    return IMG_LoadTyped_RW(src, freesrc, NULL);
}

/* Portable case-insensitive string compare function */
static int IMG_string_equals(const char *str1, const char *str2)
{
	while ( *str1 && *str2 ) {
		if ( toupper((unsigned char)*str1) !=
		     toupper((unsigned char)*str2) )
			break;
		++str1;
		++str2;
	}
	return (!*str1 && !*str2);
}

/* Load an image from an SDL datasource, optionally specifying the type */
SDL_Surface *IMG_LoadTyped_RW(SDL_RWops *src, int freesrc, char *type)
{
	int i;
	SDL_Surface *image;

	/* Make sure there is something to do.. */
	if ( src == NULL ) {
		IMG_SetError("Passed a NULL data source");
		return(NULL);
	}

	/* See whether or not this data source can handle seeking */
	if ( SDL_RWseek(src, 0, RW_SEEK_CUR) < 0 ) {
		IMG_SetError("Can't seek in this data source");
		if(freesrc)
			SDL_RWclose(src);
		return(NULL);
	}

	/* Detect the type of image being loaded */
	image = NULL;
	for ( i=0; i < ARRAYSIZE(supported); ++i ) {
		if(supported[i].is) {
			if(!supported[i].is(src))
				continue;
		} else {
			/* magicless format */
			if(!type
			   || !IMG_string_equals(type, supported[i].type))
				continue;
		}
#ifdef DEBUG_IMGLIB
		fprintf(stderr, "IMGLIB: Loading image as %s\n",
			supported[i].type);
#endif
		image = supported[i].load(src);
		if(freesrc)
			SDL_RWclose(src);
		return image;
	}

	if ( freesrc ) {
		SDL_RWclose(src);
	}
	IMG_SetError("Unsupported image format");
	return NULL;
}

/* Invert the alpha of a surface for use with OpenGL
   This function is a no-op and only kept for backwards compatibility.
 */
int IMG_InvertAlpha(int on)
{
    return 1;
}
