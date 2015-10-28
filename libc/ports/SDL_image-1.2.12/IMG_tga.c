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

/* This is a Targa image file loading framework */

#include <stdlib.h>
#include <stdio.h>
#include <string.h>

#include "SDL_endian.h"

#include "SDL_image.h"

#ifdef LOAD_TGA

/*
 * A TGA loader for the SDL library
 * Supports: Reading 8, 15, 16, 24 and 32bpp images, with alpha or colourkey,
 *           uncompressed or RLE encoded.
 *
 * 2000-06-10 Mattias Engdegård <f91-men@nada.kth.se>: initial version
 * 2000-06-26 Mattias Engdegård <f91-men@nada.kth.se>: read greyscale TGAs
 * 2000-08-09 Mattias Engdegård <f91-men@nada.kth.se>: alpha inversion removed
 */

struct TGAheader {
    Uint8 infolen;		/* length of info field */
    Uint8 has_cmap;		/* 1 if image has colormap, 0 otherwise */
    Uint8 type;

    Uint8 cmap_start[2];	/* index of first colormap entry */
    Uint8 cmap_len[2];		/* number of entries in colormap */
    Uint8 cmap_bits;		/* bits per colormap entry */

    Uint8 yorigin[2];		/* image origin (ignored here) */
    Uint8 xorigin[2];
    Uint8 width[2];		/* image size */
    Uint8 height[2];
    Uint8 pixel_bits;		/* bits/pixel */
    Uint8 flags;
};

enum tga_type {
    TGA_TYPE_INDEXED = 1,
    TGA_TYPE_RGB = 2,
    TGA_TYPE_BW = 3,
    TGA_TYPE_RLE_INDEXED = 9,
    TGA_TYPE_RLE_RGB = 10,
    TGA_TYPE_RLE_BW = 11
};

#define TGA_INTERLEAVE_MASK	0xc0
#define TGA_INTERLEAVE_NONE	0x00
#define TGA_INTERLEAVE_2WAY	0x40
#define TGA_INTERLEAVE_4WAY	0x80

#define TGA_ORIGIN_MASK		0x30
#define TGA_ORIGIN_LEFT		0x00
#define TGA_ORIGIN_RIGHT	0x10
#define TGA_ORIGIN_LOWER	0x00
#define TGA_ORIGIN_UPPER	0x20

/* read/write unaligned little-endian 16-bit ints */
#define LE16(p) ((p)[0] + ((p)[1] << 8))
#define SETLE16(p, v) ((p)[0] = (v), (p)[1] = (v) >> 8)

/* Load a TGA type image from an SDL datasource */
SDL_Surface *IMG_LoadTGA_RW(SDL_RWops *src)
{
    int start;
    const char *error = NULL;
    struct TGAheader hdr;
    int rle = 0;
    int alpha = 0;
    int indexed = 0;
    int grey = 0;
    int ckey = -1;
    int ncols, w, h;
    SDL_Surface *img = NULL;
    Uint32 rmask, gmask, bmask, amask;
    Uint8 *dst;
    int i;
    int bpp;
    int lstep;
    Uint32 pixel;
    int count, rep;

    if ( !src ) {
        /* The error message has been set in SDL_RWFromFile */
        return NULL;
    }
    start = SDL_RWtell(src);

    if(!SDL_RWread(src, &hdr, sizeof(hdr), 1)) {
        error = "Error reading TGA data";
	goto error;
    }
    ncols = LE16(hdr.cmap_len);
    switch(hdr.type) {
    case TGA_TYPE_RLE_INDEXED:
	rle = 1;
	/* fallthrough */
    case TGA_TYPE_INDEXED:
	if(!hdr.has_cmap || hdr.pixel_bits != 8 || ncols > 256)
	    goto unsupported;
	indexed = 1;
	break;

    case TGA_TYPE_RLE_RGB:
	rle = 1;
	/* fallthrough */
    case TGA_TYPE_RGB:
	indexed = 0;
	break;

    case TGA_TYPE_RLE_BW:
	rle = 1;
	/* fallthrough */
    case TGA_TYPE_BW:
	if(hdr.pixel_bits != 8)
	    goto unsupported;
	/* Treat greyscale as 8bpp indexed images */
	indexed = grey = 1;
	break;

    default:
        goto unsupported;
    }

    bpp = (hdr.pixel_bits + 7) >> 3;
    rmask = gmask = bmask = amask = 0;
    switch(hdr.pixel_bits) {
    case 8:
	if(!indexed) {
            goto unsupported;
	}
	break;

    case 15:
    case 16:
	/* 15 and 16bpp both seem to use 5 bits/plane. The extra alpha bit
	   is ignored for now. */
	rmask = 0x7c00;
	gmask = 0x03e0;
	bmask = 0x001f;
	break;

    case 32:
	alpha = 1;
	/* fallthrough */
    case 24:
	if(SDL_BYTEORDER == SDL_BIG_ENDIAN) {
	    int s = alpha ? 0 : 8;
	    amask = 0x000000ff >> s;
	    rmask = 0x0000ff00 >> s;
	    gmask = 0x00ff0000 >> s;
	    bmask = 0xff000000 >> s;
	} else {
	    amask = alpha ? 0xff000000 : 0;
	    rmask = 0x00ff0000;
	    gmask = 0x0000ff00;
	    bmask = 0x000000ff;
	}
	break;

    default:
        goto unsupported;
    }

    if((hdr.flags & TGA_INTERLEAVE_MASK) != TGA_INTERLEAVE_NONE
       || hdr.flags & TGA_ORIGIN_RIGHT) {
        goto unsupported;
    }
    
    SDL_RWseek(src, hdr.infolen, RW_SEEK_CUR); /* skip info field */

    w = LE16(hdr.width);
    h = LE16(hdr.height);
    img = SDL_CreateRGBSurface(SDL_SWSURFACE, w, h,
			       bpp * 8,
			       rmask, gmask, bmask, amask);
    if(img == NULL) {
        error = "Out of memory";
        goto error;
    }

    if(hdr.has_cmap) {
	int palsiz = ncols * ((hdr.cmap_bits + 7) >> 3);
	if(indexed && !grey) {
	    Uint8 *pal = malloc(palsiz), *p = pal;
	    SDL_Color *colors = img->format->palette->colors;
	    img->format->palette->ncolors = ncols;
	    SDL_RWread(src, pal, palsiz, 1);
	    for(i = 0; i < ncols; i++) {
		switch(hdr.cmap_bits) {
		case 15:
		case 16:
		    {
			Uint16 c = p[0] + (p[1] << 8);
			p += 2;
			colors[i].r = (c >> 7) & 0xf8;
			colors[i].g = (c >> 2) & 0xf8;
			colors[i].b = c << 3;
		    }
		    break;
		case 24:
		case 32:
		    colors[i].b = *p++;
		    colors[i].g = *p++;
		    colors[i].r = *p++;
		    if(hdr.cmap_bits == 32 && *p++ < 128)
			ckey = i;
		    break;
		}
	    }
	    free(pal);
	    if(ckey >= 0)
		SDL_SetColorKey(img, SDL_SRCCOLORKEY, ckey);
	} else {
	    /* skip unneeded colormap */
	    SDL_RWseek(src, palsiz, RW_SEEK_CUR);
	}
    }

    if(grey) {
	SDL_Color *colors = img->format->palette->colors;
	for(i = 0; i < 256; i++)
	    colors[i].r = colors[i].g = colors[i].b = i;
	img->format->palette->ncolors = 256;
    }

    if(hdr.flags & TGA_ORIGIN_UPPER) {
	lstep = img->pitch;
	dst = img->pixels;
    } else {
	lstep = -img->pitch;
	dst = (Uint8 *)img->pixels + (h - 1) * img->pitch;
    }

    /* The RLE decoding code is slightly convoluted since we can't rely on
       spans not to wrap across scan lines */
    count = rep = 0;
    for(i = 0; i < h; i++) {
	if(rle) {
	    int x = 0;
	    for(;;) {
		Uint8 c;

		if(count) {
		    int n = count;
		    if(n > w - x)
			n = w - x;
		    SDL_RWread(src, dst + x * bpp, n * bpp, 1);
		    count -= n;
		    x += n;
		    if(x == w)
			break;
		} else if(rep) {
		    int n = rep;
		    if(n > w - x)
			n = w - x;
		    rep -= n;
		    while(n--) {
			memcpy(dst + x * bpp, &pixel, bpp);
			x++;
		    }
		    if(x == w)
			break;
		}

		SDL_RWread(src, &c, 1, 1);
		if(c & 0x80) {
		    SDL_RWread(src, &pixel, bpp, 1);
		    rep = (c & 0x7f) + 1;
		} else {
		    count = c + 1;
		}
	    }

	} else {
	    SDL_RWread(src, dst, w * bpp, 1);
	}
	if(SDL_BYTEORDER == SDL_BIG_ENDIAN && bpp == 2) {
	    /* swap byte order */
	    int x;
	    Uint16 *p = (Uint16 *)dst;
	    for(x = 0; x < w; x++)
		p[x] = SDL_Swap16(p[x]);
	}
	dst += lstep;
    }
    return img;

unsupported:
    error = "Unsupported TGA format";

error:
    SDL_RWseek(src, start, RW_SEEK_SET);
    if ( img ) {
        SDL_FreeSurface(img);
    }
    IMG_SetError(error);
    return NULL;
}

#else

/* dummy TGA load routine */
SDL_Surface *IMG_LoadTGA_RW(SDL_RWops *src)
{
	return(NULL);
}

#endif /* LOAD_TGA */

#endif /* !defined(__APPLE__) || defined(SDL_IMAGE_USE_COMMON_BACKEND) */
