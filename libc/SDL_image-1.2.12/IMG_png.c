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

#if !defined(__APPLE__) || defined(SDL_IMAGE_USE_COMMON_BACKEND)

/* This is a PNG image file loading framework */

#include <stdlib.h>
#include <stdio.h>

#include "SDL_image.h"

#ifdef LOAD_PNG

/*=============================================================================
        File: SDL_png.c
     Purpose: A PNG loader and saver for the SDL library      
    Revision: 
  Created by: Philippe Lavoie          (2 November 1998)
              lavoie@zeus.genie.uottawa.ca
 Modified by: 

 Copyright notice:
          Copyright (C) 1998 Philippe Lavoie
 
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
          Software Foundation, Inc., 675 Mass Ave, Cambridge, MA 02139, USA.

    Comments: The load and save routine are basically the ones you can find
             in the example.c file from the libpng distribution.

  Changes:
    5/17/99 - Modified to use the new SDL data sources - Sam Lantinga

=============================================================================*/

#include "SDL_endian.h"

#ifdef macintosh
#define MACOS
#endif
#include <png.h>

/* Check for the older version of libpng */
#if (PNG_LIBPNG_VER_MAJOR == 1) && (PNG_LIBPNG_VER_MINOR < 4)
#define LIBPNG_VERSION_12
#endif

static struct {
	int loaded;
	void *handle;
	png_infop (*png_create_info_struct) (png_structp png_ptr);
	png_structp (*png_create_read_struct) (png_const_charp user_png_ver, png_voidp error_ptr, png_error_ptr error_fn, png_error_ptr warn_fn);
	void (*png_destroy_read_struct) (png_structpp png_ptr_ptr, png_infopp info_ptr_ptr, png_infopp end_info_ptr_ptr);
	png_uint_32 (*png_get_IHDR) (png_structp png_ptr, png_infop info_ptr, png_uint_32 *width, png_uint_32 *height, int *bit_depth, int *color_type, int *interlace_method, int *compression_method, int *filter_method);
	png_voidp (*png_get_io_ptr) (png_structp png_ptr);
	png_byte (*png_get_channels) (png_structp png_ptr, png_infop info_ptr);
	png_uint_32 (*png_get_PLTE) (png_structp png_ptr, png_infop info_ptr, png_colorp *palette, int *num_palette);
	png_uint_32 (*png_get_tRNS) (png_structp png_ptr, png_infop info_ptr, png_bytep *trans, int *num_trans, png_color_16p *trans_values);
	png_uint_32 (*png_get_valid) (png_structp png_ptr, png_infop info_ptr, png_uint_32 flag);
	void (*png_read_image) (png_structp png_ptr, png_bytepp image);
	void (*png_read_info) (png_structp png_ptr, png_infop info_ptr);
	void (*png_read_update_info) (png_structp png_ptr, png_infop info_ptr);
	void (*png_set_expand) (png_structp png_ptr);
	void (*png_set_gray_to_rgb) (png_structp png_ptr);
	void (*png_set_packing) (png_structp png_ptr);
	void (*png_set_read_fn) (png_structp png_ptr, png_voidp io_ptr, png_rw_ptr read_data_fn);
	void (*png_set_strip_16) (png_structp png_ptr);
	int (*png_sig_cmp) (png_bytep sig, png_size_t start, png_size_t num_to_check);
#ifndef LIBPNG_VERSION_12
	jmp_buf* (*png_set_longjmp_fn) (png_structp, png_longjmp_ptr, size_t);
#endif
} lib;

#ifdef LOAD_PNG_DYNAMIC
int IMG_InitPNG()
{
	if ( lib.loaded == 0 ) {
		lib.handle = SDL_LoadObject(LOAD_PNG_DYNAMIC);
		if ( lib.handle == NULL ) {
			return -1;
		}
		lib.png_create_info_struct =
			(png_infop (*) (png_structp))
			SDL_LoadFunction(lib.handle, "png_create_info_struct");
		if ( lib.png_create_info_struct == NULL ) {
			SDL_UnloadObject(lib.handle);
			return -1;
		}
		lib.png_create_read_struct =
			(png_structp (*) (png_const_charp, png_voidp, png_error_ptr, png_error_ptr))
			SDL_LoadFunction(lib.handle, "png_create_read_struct");
		if ( lib.png_create_read_struct == NULL ) {
			SDL_UnloadObject(lib.handle);
			return -1;
		}
		lib.png_destroy_read_struct =
			(void (*) (png_structpp, png_infopp, png_infopp))
			SDL_LoadFunction(lib.handle, "png_destroy_read_struct");
		if ( lib.png_destroy_read_struct == NULL ) {
			SDL_UnloadObject(lib.handle);
			return -1;
		}
		lib.png_get_IHDR =
			(png_uint_32 (*) (png_structp, png_infop, png_uint_32 *, png_uint_32 *, int *, int *, int *, int *, int *))
			SDL_LoadFunction(lib.handle, "png_get_IHDR");
		if ( lib.png_get_IHDR == NULL ) {
			SDL_UnloadObject(lib.handle);
			return -1;
		}
		lib.png_get_channels =
			(png_byte (*) (png_structp, png_infop))
			SDL_LoadFunction(lib.handle, "png_get_channels");
		if ( lib.png_get_channels == NULL ) {
			SDL_UnloadObject(lib.handle);
			return -1;
		}
		lib.png_get_io_ptr =
			(png_voidp (*) (png_structp))
			SDL_LoadFunction(lib.handle, "png_get_io_ptr");
		if ( lib.png_get_io_ptr == NULL ) {
			SDL_UnloadObject(lib.handle);
			return -1;
		}
		lib.png_get_PLTE =
			(png_uint_32 (*) (png_structp, png_infop, png_colorp *, int *))
			SDL_LoadFunction(lib.handle, "png_get_PLTE");
		if ( lib.png_get_PLTE == NULL ) {
			SDL_UnloadObject(lib.handle);
			return -1;
		}
		lib.png_get_tRNS =
			(png_uint_32 (*) (png_structp, png_infop, png_bytep *, int *, png_color_16p *))
			SDL_LoadFunction(lib.handle, "png_get_tRNS");
		if ( lib.png_get_tRNS == NULL ) {
			SDL_UnloadObject(lib.handle);
			return -1;
		}
		lib.png_get_valid =
			(png_uint_32 (*) (png_structp, png_infop, png_uint_32))
			SDL_LoadFunction(lib.handle, "png_get_valid");
		if ( lib.png_get_valid == NULL ) {
			SDL_UnloadObject(lib.handle);
			return -1;
		}
		lib.png_read_image =
			(void (*) (png_structp, png_bytepp))
			SDL_LoadFunction(lib.handle, "png_read_image");
		if ( lib.png_read_image == NULL ) {
			SDL_UnloadObject(lib.handle);
			return -1;
		}
		lib.png_read_info =
			(void (*) (png_structp, png_infop))
			SDL_LoadFunction(lib.handle, "png_read_info");
		if ( lib.png_read_info == NULL ) {
			SDL_UnloadObject(lib.handle);
			return -1;
		}
		lib.png_read_update_info =
			(void (*) (png_structp, png_infop))
			SDL_LoadFunction(lib.handle, "png_read_update_info");
		if ( lib.png_read_update_info == NULL ) {
			SDL_UnloadObject(lib.handle);
			return -1;
		}
		lib.png_set_expand =
			(void (*) (png_structp))
			SDL_LoadFunction(lib.handle, "png_set_expand");
		if ( lib.png_set_expand == NULL ) {
			SDL_UnloadObject(lib.handle);
			return -1;
		}
		lib.png_set_gray_to_rgb =
			(void (*) (png_structp))
			SDL_LoadFunction(lib.handle, "png_set_gray_to_rgb");
		if ( lib.png_set_gray_to_rgb == NULL ) {
			SDL_UnloadObject(lib.handle);
			return -1;
		}
		lib.png_set_packing =
			(void (*) (png_structp))
			SDL_LoadFunction(lib.handle, "png_set_packing");
		if ( lib.png_set_packing == NULL ) {
			SDL_UnloadObject(lib.handle);
			return -1;
		}
		lib.png_set_read_fn =
			(void (*) (png_structp, png_voidp, png_rw_ptr))
			SDL_LoadFunction(lib.handle, "png_set_read_fn");
		if ( lib.png_set_read_fn == NULL ) {
			SDL_UnloadObject(lib.handle);
			return -1;
		}
		lib.png_set_strip_16 =
			(void (*) (png_structp))
			SDL_LoadFunction(lib.handle, "png_set_strip_16");
		if ( lib.png_set_strip_16 == NULL ) {
			SDL_UnloadObject(lib.handle);
			return -1;
		}
		lib.png_sig_cmp =
			(int (*) (png_bytep, png_size_t, png_size_t))
			SDL_LoadFunction(lib.handle, "png_sig_cmp");
		if ( lib.png_sig_cmp == NULL ) {
			SDL_UnloadObject(lib.handle);
			return -1;
		}
#ifndef LIBPNG_VERSION_12
		lib.png_set_longjmp_fn =
			(jmp_buf * (*) (png_structp, png_longjmp_ptr, size_t))
			SDL_LoadFunction(lib.handle, "png_set_longjmp_fn");
		if ( lib.png_set_longjmp_fn == NULL ) {
			SDL_UnloadObject(lib.handle);
			return -1;
		}
#endif
	}
	++lib.loaded;

	return 0;
}
void IMG_QuitPNG()
{
	if ( lib.loaded == 0 ) {
		return;
	}
	if ( lib.loaded == 1 ) {
		SDL_UnloadObject(lib.handle);
	}
	--lib.loaded;
}
#else
int IMG_InitPNG()
{
	if ( lib.loaded == 0 ) {
		lib.png_create_info_struct = png_create_info_struct;
		lib.png_create_read_struct = png_create_read_struct;
		lib.png_destroy_read_struct = png_destroy_read_struct;
		lib.png_get_IHDR = png_get_IHDR;
		lib.png_get_channels = png_get_channels;
		lib.png_get_io_ptr = png_get_io_ptr;
		lib.png_get_PLTE = png_get_PLTE;
		lib.png_get_tRNS = png_get_tRNS;
		lib.png_get_valid = png_get_valid;
		lib.png_read_image = png_read_image;
		lib.png_read_info = png_read_info;
		lib.png_read_update_info = png_read_update_info;
		lib.png_set_expand = png_set_expand;
		lib.png_set_gray_to_rgb = png_set_gray_to_rgb;
		lib.png_set_packing = png_set_packing;
		lib.png_set_read_fn = png_set_read_fn;
		lib.png_set_strip_16 = png_set_strip_16;
		lib.png_sig_cmp = png_sig_cmp;
#ifndef LIBPNG_VERSION_12
		lib.png_set_longjmp_fn = png_set_longjmp_fn;
#endif
	}
	++lib.loaded;

	return 0;
}
void IMG_QuitPNG()
{
	if ( lib.loaded == 0 ) {
		return;
	}
	if ( lib.loaded == 1 ) {
	}
	--lib.loaded;
}
#endif /* LOAD_PNG_DYNAMIC */

/* See if an image is contained in a data source */
int IMG_isPNG(SDL_RWops *src)
{
	int start;
	int is_PNG;
	Uint8 magic[4];

	if ( !src )
		return 0;
	start = SDL_RWtell(src);
	is_PNG = 0;
	if ( SDL_RWread(src, magic, 1, sizeof(magic)) == sizeof(magic) ) {
                if ( magic[0] == 0x89 &&
                     magic[1] == 'P' &&
                     magic[2] == 'N' &&
                     magic[3] == 'G' ) {
			is_PNG = 1;
		}
	}
	SDL_RWseek(src, start, RW_SEEK_SET);
	return(is_PNG);
}

/* Load a PNG type image from an SDL datasource */
static void png_read_data(png_structp ctx, png_bytep area, png_size_t size)
{
	SDL_RWops *src;

	src = (SDL_RWops *)lib.png_get_io_ptr(ctx);
	SDL_RWread(src, area, size, 1);
}
SDL_Surface *IMG_LoadPNG_RW(SDL_RWops *src)
{
	int start;
	const char *error;
	SDL_Surface *volatile surface;
	png_structp png_ptr;
	png_infop info_ptr;
	png_uint_32 width, height;
	int bit_depth, color_type, interlace_type, num_channels;
	Uint32 Rmask;
	Uint32 Gmask;
	Uint32 Bmask;
	Uint32 Amask;
	SDL_Palette *palette;
	png_bytep *volatile row_pointers;
	int row, i;
	volatile int ckey = -1;
	png_color_16 *transv;

	if ( !src ) {
		/* The error message has been set in SDL_RWFromFile */
		return NULL;
	}
	start = SDL_RWtell(src);

	if ( !IMG_Init(IMG_INIT_PNG) ) {
		return NULL;
	}

	/* Initialize the data we will clean up when we're done */
	error = NULL;
	png_ptr = NULL; info_ptr = NULL; row_pointers = NULL; surface = NULL;

	/* Create the PNG loading context structure */
	png_ptr = lib.png_create_read_struct(PNG_LIBPNG_VER_STRING,
					  NULL,NULL,NULL);
	if (png_ptr == NULL){
		error = "Couldn't allocate memory for PNG file or incompatible PNG dll";
		goto done;
	}

	 /* Allocate/initialize the memory for image information.  REQUIRED. */
	info_ptr = lib.png_create_info_struct(png_ptr);
	if (info_ptr == NULL) {
		error = "Couldn't create image information for PNG file";
		goto done;
	}

	/* Set error handling if you are using setjmp/longjmp method (this is
	 * the normal method of doing things with libpng).  REQUIRED unless you
	 * set up your own error handlers in png_create_read_struct() earlier.
	 */
#ifndef LIBPNG_VERSION_12
	if ( setjmp(*lib.png_set_longjmp_fn(png_ptr, longjmp, sizeof (jmp_buf))) )
#else
	if ( setjmp(png_ptr->jmpbuf) )
#endif
	{
		error = "Error reading the PNG file.";
		goto done;
	}

	/* Set up the input control */
	lib.png_set_read_fn(png_ptr, src, png_read_data);

	/* Read PNG header info */
	lib.png_read_info(png_ptr, info_ptr);
	lib.png_get_IHDR(png_ptr, info_ptr, &width, &height, &bit_depth,
			&color_type, &interlace_type, NULL, NULL);

	/* tell libpng to strip 16 bit/color files down to 8 bits/color */
	lib.png_set_strip_16(png_ptr) ;

	/* Extract multiple pixels with bit depths of 1, 2, and 4 from a single
	 * byte into separate bytes (useful for paletted and grayscale images).
	 */
	lib.png_set_packing(png_ptr);

	/* scale greyscale values to the range 0..255 */
	if(color_type == PNG_COLOR_TYPE_GRAY)
		lib.png_set_expand(png_ptr);

	/* For images with a single "transparent colour", set colour key;
	   if more than one index has transparency, or if partially transparent
	   entries exist, use full alpha channel */
	if (lib.png_get_valid(png_ptr, info_ptr, PNG_INFO_tRNS)) {
	        int num_trans;
		Uint8 *trans;
		lib.png_get_tRNS(png_ptr, info_ptr, &trans, &num_trans,
			     &transv);
		if(color_type == PNG_COLOR_TYPE_PALETTE) {
		    /* Check if all tRNS entries are opaque except one */
		    int i, t = -1;
		    for(i = 0; i < num_trans; i++)
			if(trans[i] == 0) {
			    if(t >= 0)
				break;
			    t = i;
			} else if(trans[i] != 255)
			    break;
		    if(i == num_trans) {
			/* exactly one transparent index */
			ckey = t;
		    } else {
			/* more than one transparent index, or translucency */
			lib.png_set_expand(png_ptr);
		    }
		} else
		    ckey = 0; /* actual value will be set later */
	}

	if ( color_type == PNG_COLOR_TYPE_GRAY_ALPHA )
		lib.png_set_gray_to_rgb(png_ptr);

	lib.png_read_update_info(png_ptr, info_ptr);

	lib.png_get_IHDR(png_ptr, info_ptr, &width, &height, &bit_depth,
			&color_type, &interlace_type, NULL, NULL);

	/* Allocate the SDL surface to hold the image */
	Rmask = Gmask = Bmask = Amask = 0 ;
	num_channels = lib.png_get_channels(png_ptr, info_ptr);
	if ( color_type != PNG_COLOR_TYPE_PALETTE ) {
		if ( SDL_BYTEORDER == SDL_LIL_ENDIAN ) {
			Rmask = 0x000000FF;
			Gmask = 0x0000FF00;
			Bmask = 0x00FF0000;
			Amask = (num_channels == 4) ? 0xFF000000 : 0;
		} else {
			int s = (num_channels == 4) ? 0 : 8;
			Rmask = 0xFF000000 >> s;
			Gmask = 0x00FF0000 >> s;
			Bmask = 0x0000FF00 >> s;
			Amask = 0x000000FF >> s;
		}
	}
	surface = SDL_AllocSurface(SDL_SWSURFACE, width, height,
			bit_depth*num_channels, Rmask,Gmask,Bmask,Amask);
	if ( surface == NULL ) {
		error = "Out of memory";
		goto done;
	}

	if(ckey != -1) {
	        if(color_type != PNG_COLOR_TYPE_PALETTE)
			/* FIXME: Should these be truncated or shifted down? */
		        ckey = SDL_MapRGB(surface->format,
			                 (Uint8)transv->red,
			                 (Uint8)transv->green,
			                 (Uint8)transv->blue);
	        SDL_SetColorKey(surface, SDL_SRCCOLORKEY, ckey);
	}

	/* Create the array of pointers to image data */
	row_pointers = (png_bytep*) malloc(sizeof(png_bytep)*height);
	if ( (row_pointers == NULL) ) {
		error = "Out of memory";
		goto done;
	}
	for (row = 0; row < (int)height; row++) {
		row_pointers[row] = (png_bytep)
				(Uint8 *)surface->pixels + row*surface->pitch;
	}

	/* Read the entire image in one go */
	lib.png_read_image(png_ptr, row_pointers);

	/* and we're done!  (png_read_end() can be omitted if no processing of
	 * post-IDAT text/time/etc. is desired)
	 * In some cases it can't read PNG's created by some popular programs (ACDSEE),
	 * we do not want to process comments, so we omit png_read_end

	lib.png_read_end(png_ptr, info_ptr);
	*/

	/* Load the palette, if any */
	palette = surface->format->palette;
	if ( palette ) {
	    int png_num_palette;
	    png_colorp png_palette;
	    lib.png_get_PLTE(png_ptr, info_ptr, &png_palette, &png_num_palette);
	    if(color_type == PNG_COLOR_TYPE_GRAY) {
		palette->ncolors = 256;
		for(i = 0; i < 256; i++) {
		    palette->colors[i].r = i;
		    palette->colors[i].g = i;
		    palette->colors[i].b = i;
		}
	    } else if (png_num_palette > 0 ) {
		palette->ncolors = png_num_palette; 
		for( i=0; i<png_num_palette; ++i ) {
		    palette->colors[i].b = png_palette[i].blue;
		    palette->colors[i].g = png_palette[i].green;
		    palette->colors[i].r = png_palette[i].red;
		}
	    }
	}

done:	/* Clean up and return */
	if ( png_ptr ) {
		lib.png_destroy_read_struct(&png_ptr,
		                        info_ptr ? &info_ptr : (png_infopp)0,
								(png_infopp)0);
	}
	if ( row_pointers ) {
		free(row_pointers);
	}
	if ( error ) {
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

int IMG_InitPNG()
{
	IMG_SetError("PNG images are not supported");
	return(-1);
}

void IMG_QuitPNG()
{
}

/* See if an image is contained in a data source */
int IMG_isPNG(SDL_RWops *src)
{
	return(0);
}

/* Load a PNG type image from an SDL datasource */
SDL_Surface *IMG_LoadPNG_RW(SDL_RWops *src)
{
	return(NULL);
}

#endif /* LOAD_PNG */

#endif /* !defined(__APPLE__) || defined(SDL_IMAGE_USE_COMMON_BACKEND) */
