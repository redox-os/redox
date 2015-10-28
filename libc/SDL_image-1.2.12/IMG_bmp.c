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

/* This is a BMP image file loading framework */
/* ICO/CUR file support is here as well since it uses similar internal
 * representation */

#include <stdio.h>
#include <string.h>

#include "SDL_image.h"

#ifdef LOAD_BMP

/* See if an image is contained in a data source */
int IMG_isBMP(SDL_RWops *src)
{
	int start;
	int is_BMP;
	char magic[2];

	if ( !src )
		return 0;
	start = SDL_RWtell(src);
	is_BMP = 0;
	if ( SDL_RWread(src, magic, sizeof(magic), 1) ) {
		if ( strncmp(magic, "BM", 2) == 0 ) {
			is_BMP = 1;
		}
	}
	SDL_RWseek(src, start, RW_SEEK_SET);
	return(is_BMP);
}

static int IMG_isICOCUR(SDL_RWops *src, int type)
{
	int start;
	int is_ICOCUR;

	/* The Win32 ICO file header (14 bytes) */
    Uint16 bfReserved;
    Uint16 bfType;
    Uint16 bfCount;

	if ( !src )
		return 0;
	start = SDL_RWtell(src);
	is_ICOCUR = 0;
    bfReserved = SDL_ReadLE16(src);
    bfType = SDL_ReadLE16(src);
    bfCount = SDL_ReadLE16(src);
    if ((bfReserved == 0) && (bfType == type) && (bfCount != 0)) 
    	is_ICOCUR = 1;
	SDL_RWseek(src, start, RW_SEEK_SET);

	return (is_ICOCUR);
}

int IMG_isICO(SDL_RWops *src)
{
	return IMG_isICOCUR(src, 1);
}

int IMG_isCUR(SDL_RWops *src)
{
	return IMG_isICOCUR(src, 2);
}

#include "SDL_error.h"
#include "SDL_video.h"
#include "SDL_endian.h"

/* Compression encodings for BMP files */
#ifndef BI_RGB
#define BI_RGB		0
#define BI_RLE8		1
#define BI_RLE4		2
#define BI_BITFIELDS	3
#endif

static int readRlePixels(SDL_Surface * surface, SDL_RWops * src, int isRle8)
{
	/*
	| Sets the surface pixels from src.  A bmp image is upside down.
	*/
	int pitch = surface->pitch;
	int height = surface->h;
	Uint8 *start = (Uint8 *)surface->pixels;
	Uint8 *end = start + (height*pitch);
	Uint8 *bits = end-pitch, *spot;
	int ofs = 0;
	Uint8 ch;
	Uint8 needsPad;

#define COPY_PIXEL(x)	spot = &bits[ofs++]; if(spot >= start && spot < end) *spot = (x)

	for (;;) {
		if ( !SDL_RWread(src, &ch, 1, 1) ) return 1;
		/*
		| encoded mode starts with a run length, and then a byte
		| with two colour indexes to alternate between for the run
		*/
		if ( ch ) {
			Uint8 pixel;
			if ( !SDL_RWread(src, &pixel, 1, 1) ) return 1;
			if ( isRle8 ) {                 /* 256-color bitmap, compressed */
				do {
					COPY_PIXEL(pixel);
				} while (--ch);
			} else {                         /* 16-color bitmap, compressed */
				Uint8 pixel0 = pixel >> 4;
				Uint8 pixel1 = pixel & 0x0F;
				for (;;) {
					COPY_PIXEL(pixel0);	/* even count, high nibble */
					if (!--ch) break;
					COPY_PIXEL(pixel1);	/* odd count, low nibble */
					if (!--ch) break;
				}
			}
		} else {
			/*
			| A leading zero is an escape; it may signal the end of the bitmap,
			| a cursor move, or some absolute data.
			| zero tag may be absolute mode or an escape
			*/
			if ( !SDL_RWread(src, &ch, 1, 1) ) return 1;
			switch (ch) {
			case 0:                         /* end of line */
				ofs = 0;
				bits -= pitch;               /* go to previous */
				break;
			case 1:                         /* end of bitmap */
				return 0;                    /* success! */
			case 2:                         /* delta */
				if ( !SDL_RWread(src, &ch, 1, 1) ) return 1;
				ofs += ch;
				if ( !SDL_RWread(src, &ch, 1, 1) ) return 1;
				bits -= (ch * pitch);
				break;
			default:                        /* no compression */
				if (isRle8) {
					needsPad = ( ch & 1 );
					do {
						Uint8 pixel;
						if ( !SDL_RWread(src, &pixel, 1, 1) ) return 1;
						COPY_PIXEL(pixel);
					} while (--ch);
				} else {
					needsPad = ( ((ch+1)>>1) & 1 ); /* (ch+1)>>1: bytes size */
					for (;;) {
						Uint8 pixel;
						if ( !SDL_RWread(src, &pixel, 1, 1) ) return 1;
						COPY_PIXEL(pixel >> 4);
						if (!--ch) break;
						COPY_PIXEL(pixel & 0x0F);
						if (!--ch) break;
					}
				}
				/* pad at even boundary */
				if ( needsPad && !SDL_RWread(src, &ch, 1, 1) ) return 1;
				break;
			}
		}
	}
}

static SDL_Surface *LoadBMP_RW (SDL_RWops *src, int freesrc)
{
	SDL_bool was_error;
	long fp_offset;
	int bmpPitch;
	int i, pad;
	SDL_Surface *surface;
	Uint32 Rmask;
	Uint32 Gmask;
	Uint32 Bmask;
	Uint32 Amask;
	SDL_Palette *palette;
	Uint8 *bits;
	Uint8 *top, *end;
	SDL_bool topDown;
	int ExpandBMP;

	/* The Win32 BMP file header (14 bytes) */
	char   magic[2];
	Uint32 bfSize;
	Uint16 bfReserved1;
	Uint16 bfReserved2;
	Uint32 bfOffBits;

	/* The Win32 BITMAPINFOHEADER struct (40 bytes) */
	Uint32 biSize;
	Sint32 biWidth;
	Sint32 biHeight;
	Uint16 biPlanes;
	Uint16 biBitCount;
	Uint32 biCompression;
	Uint32 biSizeImage;
	Sint32 biXPelsPerMeter;
	Sint32 biYPelsPerMeter;
	Uint32 biClrUsed;
	Uint32 biClrImportant;

	/* Make sure we are passed a valid data source */
	surface = NULL;
	was_error = SDL_FALSE;
	if ( src == NULL ) {
		was_error = SDL_TRUE;
		goto done;
	}

	/* Read in the BMP file header */
	fp_offset = SDL_RWtell(src);
	SDL_ClearError();
	if ( SDL_RWread(src, magic, 1, 2) != 2 ) {
		SDL_Error(SDL_EFREAD);
		was_error = SDL_TRUE;
		goto done;
	}
	if ( strncmp(magic, "BM", 2) != 0 ) {
		IMG_SetError("File is not a Windows BMP file");
		was_error = SDL_TRUE;
		goto done;
	}
	bfSize		= SDL_ReadLE32(src);
	bfReserved1	= SDL_ReadLE16(src);
	bfReserved2	= SDL_ReadLE16(src);
	bfOffBits	= SDL_ReadLE32(src);

	/* Read the Win32 BITMAPINFOHEADER */
	biSize		= SDL_ReadLE32(src);
	if ( biSize == 12 ) {
		biWidth		= (Uint32)SDL_ReadLE16(src);
		biHeight	= (Uint32)SDL_ReadLE16(src);
		biPlanes	= SDL_ReadLE16(src);
		biBitCount	= SDL_ReadLE16(src);
		biCompression	= BI_RGB;
		biSizeImage	= 0;
		biXPelsPerMeter	= 0;
		biYPelsPerMeter	= 0;
		biClrUsed	= 0;
		biClrImportant	= 0;
	} else {
		biWidth		= SDL_ReadLE32(src);
		biHeight	= SDL_ReadLE32(src);
		biPlanes	= SDL_ReadLE16(src);
		biBitCount	= SDL_ReadLE16(src);
		biCompression	= SDL_ReadLE32(src);
		biSizeImage	= SDL_ReadLE32(src);
		biXPelsPerMeter	= SDL_ReadLE32(src);
		biYPelsPerMeter	= SDL_ReadLE32(src);
		biClrUsed	= SDL_ReadLE32(src);
		biClrImportant	= SDL_ReadLE32(src);
	}
	if (biHeight < 0) {
		topDown = SDL_TRUE;
		biHeight = -biHeight;
	} else {
		topDown = SDL_FALSE;
	}

	/* Check for read error */
	if ( strcmp(SDL_GetError(), "") != 0 ) {
		was_error = SDL_TRUE;
		goto done;
	}

	/* Expand 1 and 4 bit bitmaps to 8 bits per pixel */
	switch (biBitCount) {
		case 1:
		case 4:
			ExpandBMP = biBitCount;
			biBitCount = 8;
			break;
		default:
			ExpandBMP = 0;
			break;
	}

	/* RLE4 and RLE8 BMP compression is supported */
	Rmask = Gmask = Bmask = Amask = 0;
	switch (biCompression) {
		case BI_RGB:
			/* If there are no masks, use the defaults */
			if ( bfOffBits == (14+biSize) ) {
				/* Default values for the BMP format */
				switch (biBitCount) {
					case 15:
					case 16:
						Rmask = 0x7C00;
						Gmask = 0x03E0;
						Bmask = 0x001F;
						break;
					case 24:
#if SDL_BYTEORDER == SDL_BIG_ENDIAN
					        Rmask = 0x000000FF;
					        Gmask = 0x0000FF00;
					        Bmask = 0x00FF0000;
#else
						Rmask = 0x00FF0000;
						Gmask = 0x0000FF00;
						Bmask = 0x000000FF;
#endif
						break;
					case 32:
						Amask = 0xFF000000;
						Rmask = 0x00FF0000;
						Gmask = 0x0000FF00;
						Bmask = 0x000000FF;
						break;
					default:
						break;
				}
				break;
			}
			/* Fall through -- read the RGB masks */

		default:
			switch (biBitCount) {
				case 15:
				case 16:
					Rmask = SDL_ReadLE32(src);
					Gmask = SDL_ReadLE32(src);
					Bmask = SDL_ReadLE32(src);
					break;
				case 32:
					Rmask = SDL_ReadLE32(src);
					Gmask = SDL_ReadLE32(src);
					Bmask = SDL_ReadLE32(src);
					Amask = SDL_ReadLE32(src);
					break;
				default:
					break;
			}
			break;
	}

	/* Create a compatible surface, note that the colors are RGB ordered */
	surface = SDL_CreateRGBSurface(SDL_SWSURFACE,
			biWidth, biHeight, biBitCount, Rmask, Gmask, Bmask, Amask);
	if ( surface == NULL ) {
		was_error = SDL_TRUE;
		goto done;
	}

	/* Load the palette, if any */
	palette = (surface->format)->palette;
	if ( palette ) {
		if ( SDL_RWseek(src, fp_offset+14+biSize, RW_SEEK_SET) < 0 ) {
			SDL_Error(SDL_EFSEEK);
			was_error = SDL_TRUE;
			goto done;
		}

		/*
		| guich: always use 1<<bpp b/c some bitmaps can bring wrong information
		| for colorsUsed
		*/
		/* if ( biClrUsed == 0 ) {  */
		biClrUsed = 1 << biBitCount;
		/* } */
		if ( biSize == 12 ) {
			for ( i = 0; i < (int)biClrUsed; ++i ) {
				SDL_RWread(src, &palette->colors[i].b, 1, 1);
				SDL_RWread(src, &palette->colors[i].g, 1, 1);
				SDL_RWread(src, &palette->colors[i].r, 1, 1);
				palette->colors[i].unused = 0;
			}	
		} else {
			for ( i = 0; i < (int)biClrUsed; ++i ) {
				SDL_RWread(src, &palette->colors[i].b, 1, 1);
				SDL_RWread(src, &palette->colors[i].g, 1, 1);
				SDL_RWread(src, &palette->colors[i].r, 1, 1);
				SDL_RWread(src, &palette->colors[i].unused, 1, 1);
			}	
		}
		palette->ncolors = biClrUsed;
	}

	/* Read the surface pixels.  Note that the bmp image is upside down */
	if ( SDL_RWseek(src, fp_offset+bfOffBits, RW_SEEK_SET) < 0 ) {
		SDL_Error(SDL_EFSEEK);
		was_error = SDL_TRUE;
		goto done;
	}
	if ((biCompression == BI_RLE4) || (biCompression == BI_RLE8)) {
		was_error = readRlePixels(surface, src, biCompression == BI_RLE8);
		if (was_error) IMG_SetError("Error reading from BMP");
		goto done;
	}
	top = (Uint8 *)surface->pixels;
	end = (Uint8 *)surface->pixels+(surface->h*surface->pitch);
	switch (ExpandBMP) {
		case 1:
			bmpPitch = (biWidth + 7) >> 3;
			pad  = (((bmpPitch)%4) ? (4-((bmpPitch)%4)) : 0);
			break;
		case 4:
			bmpPitch = (biWidth + 1) >> 1;
			pad  = (((bmpPitch)%4) ? (4-((bmpPitch)%4)) : 0);
			break;
		default:
			pad  = ((surface->pitch%4) ?
					(4-(surface->pitch%4)) : 0);
			break;
	}
	if ( topDown ) {
		bits = top;
	} else {
		bits = end - surface->pitch;
	}
	while ( bits >= top && bits < end ) {
		switch (ExpandBMP) {
			case 1:
			case 4: {
			Uint8 pixel = 0;
			int   shift = (8-ExpandBMP);
			for ( i=0; i<surface->w; ++i ) {
				if ( i%(8/ExpandBMP) == 0 ) {
					if ( !SDL_RWread(src, &pixel, 1, 1) ) {
						IMG_SetError(
					"Error reading from BMP");
						was_error = SDL_TRUE;
						goto done;
					}
				}
				*(bits+i) = (pixel>>shift);
				pixel <<= ExpandBMP;
			} }
			break;

			default:
			if ( SDL_RWread(src, bits, 1, surface->pitch)
							 != surface->pitch ) {
				SDL_Error(SDL_EFREAD);
				was_error = SDL_TRUE;
				goto done;
			}
#if SDL_BYTEORDER == SDL_BIG_ENDIAN
			/* Byte-swap the pixels if needed. Note that the 24bpp
			   case has already been taken care of above. */
			switch(biBitCount) {
				case 15:
				case 16: {
				        Uint16 *pix = (Uint16 *)bits;
					for(i = 0; i < surface->w; i++)
					        pix[i] = SDL_Swap16(pix[i]);
					break;
				}

				case 32: {
				        Uint32 *pix = (Uint32 *)bits;
					for(i = 0; i < surface->w; i++)
					        pix[i] = SDL_Swap32(pix[i]);
					break;
				}
			}
#endif
			break;
		}
		/* Skip padding bytes, ugh */
		if ( pad ) {
			Uint8 padbyte;
			for ( i=0; i<pad; ++i ) {
				SDL_RWread(src, &padbyte, 1, 1);
			}
		}
		if ( topDown ) {
			bits += surface->pitch;
		} else {
			bits -= surface->pitch;
		}
	}
done:
	if ( was_error ) {
		if ( src ) {
			SDL_RWseek(src, fp_offset, RW_SEEK_SET);
		}
		if ( surface ) {
			SDL_FreeSurface(surface);
		}
		surface = NULL;
	}
	if ( freesrc && src ) {
		SDL_RWclose(src);
	}
	return(surface);
}

static Uint8
SDL_Read8(SDL_RWops * src)
{
    Uint8 value;

    SDL_RWread(src, &value, 1, 1);
    return (value);
}

static SDL_Surface *
LoadICOCUR_RW(SDL_RWops * src, int type, int freesrc)
{
    SDL_bool was_error;
    long fp_offset;
    int bmpPitch;
    int i, pad;
    SDL_Surface *surface;
    Uint32 Rmask;
    Uint32 Gmask;
    Uint32 Bmask;
    Uint8 *bits;
    int ExpandBMP;
    int maxCol = 0;
    int icoOfs = 0;
    Uint32 palette[256];

    /* The Win32 ICO file header (14 bytes) */
    Uint16 bfReserved;
    Uint16 bfType;
    Uint16 bfCount;

    /* The Win32 BITMAPINFOHEADER struct (40 bytes) */
    Uint32 biSize;
    Sint32 biWidth;
    Sint32 biHeight;
    Uint16 biPlanes;
    Uint16 biBitCount;
    Uint32 biCompression;
    Uint32 biSizeImage;
    Sint32 biXPelsPerMeter;
    Sint32 biYPelsPerMeter;
    Uint32 biClrUsed;
    Uint32 biClrImportant;

    /* Make sure we are passed a valid data source */
    surface = NULL;
    was_error = SDL_FALSE;
    if (src == NULL) {
        was_error = SDL_TRUE;
        goto done;
    }

    /* Read in the ICO file header */
    fp_offset = SDL_RWtell(src);
    SDL_ClearError();

    bfReserved = SDL_ReadLE16(src);
    bfType = SDL_ReadLE16(src);
    bfCount = SDL_ReadLE16(src);
    if ((bfReserved != 0) || (bfType != type) || (bfCount == 0)) {
        IMG_SetError("File is not a Windows %s file", type == 1 ? "ICO" : "CUR");
        was_error = SDL_TRUE;
        goto done;
    }

    /* Read the Win32 Icon Directory */
    for (i = 0; i < bfCount; i++) {
        /* Icon Directory Entries */
        int bWidth = SDL_Read8(src);    /* Uint8, but 0 = 256 ! */
        int bHeight = SDL_Read8(src);   /* Uint8, but 0 = 256 ! */
        int bColorCount = SDL_Read8(src);       /* Uint8, but 0 = 256 ! */
        Uint8 bReserved = SDL_Read8(src);
        Uint16 wPlanes = SDL_ReadLE16(src);
        Uint16 wBitCount = SDL_ReadLE16(src);
        Uint32 dwBytesInRes = SDL_ReadLE32(src);
        Uint32 dwImageOffset = SDL_ReadLE32(src);

        if (!bWidth)
            bWidth = 256;
        if (!bHeight)
            bHeight = 256;
        if (!bColorCount)
            bColorCount = 256;

        //printf("%dx%d@%d - %08x\n", bWidth, bHeight, bColorCount, dwImageOffset);
        if (bColorCount > maxCol) {
            maxCol = bColorCount;
            icoOfs = dwImageOffset;
            //printf("marked\n");
        }
    }

    /* Advance to the DIB Data */
    if (SDL_RWseek(src, icoOfs, RW_SEEK_SET) < 0) {
        SDL_Error(SDL_EFSEEK);
        was_error = SDL_TRUE;
        goto done;
    }

    /* Read the Win32 BITMAPINFOHEADER */
    biSize = SDL_ReadLE32(src);
    if (biSize == 40) {
        biWidth = SDL_ReadLE32(src);
        biHeight = SDL_ReadLE32(src);
        biPlanes = SDL_ReadLE16(src);
        biBitCount = SDL_ReadLE16(src);
        biCompression = SDL_ReadLE32(src);
        biSizeImage = SDL_ReadLE32(src);
        biXPelsPerMeter = SDL_ReadLE32(src);
        biYPelsPerMeter = SDL_ReadLE32(src);
        biClrUsed = SDL_ReadLE32(src);
        biClrImportant = SDL_ReadLE32(src);
    } else {
        IMG_SetError("Unsupported ICO bitmap format");
        was_error = SDL_TRUE;
        goto done;
    }

    /* Check for read error */
    if (SDL_strcmp(SDL_GetError(), "") != 0) {
        was_error = SDL_TRUE;
        goto done;
    }

    /* We don't support any BMP compression right now */
    switch (biCompression) {
    case BI_RGB:
        /* Default values for the BMP format */
        switch (biBitCount) {
        case 1:
        case 4:
            ExpandBMP = biBitCount;
            biBitCount = 8;
            break;
        case 8:
            ExpandBMP = 8;
            break;
        case 32:
            Rmask = 0x00FF0000;
            Gmask = 0x0000FF00;
            Bmask = 0x000000FF;
            ExpandBMP = 0;
            break;
        default:
            IMG_SetError("ICO file with unsupported bit count");
            was_error = SDL_TRUE;
            goto done;
        }
        break;
    default:
        IMG_SetError("Compressed ICO files not supported");
        was_error = SDL_TRUE;
        goto done;
    }

    /* Create a RGBA surface */
    biHeight = biHeight >> 1;
    //printf("%d x %d\n", biWidth, biHeight);
    surface =
        SDL_CreateRGBSurface(0, biWidth, biHeight, 32, 0x00FF0000,
                             0x0000FF00, 0x000000FF, 0xFF000000);
    if (surface == NULL) {
        was_error = SDL_TRUE;
        goto done;
    }

    /* Load the palette, if any */
    //printf("bc %d bused %d\n", biBitCount, biClrUsed);
    if (biBitCount <= 8) {
        if (biClrUsed == 0) {
            biClrUsed = 1 << biBitCount;
        }
        for (i = 0; i < (int) biClrUsed; ++i) {
            SDL_RWread(src, &palette[i], 4, 1);
        }
    }

    /* Read the surface pixels.  Note that the bmp image is upside down */
    bits = (Uint8 *) surface->pixels + (surface->h * surface->pitch);
    switch (ExpandBMP) {
    case 1:
        bmpPitch = (biWidth + 7) >> 3;
        pad = (((bmpPitch) % 4) ? (4 - ((bmpPitch) % 4)) : 0);
        break;
    case 4:
        bmpPitch = (biWidth + 1) >> 1;
        pad = (((bmpPitch) % 4) ? (4 - ((bmpPitch) % 4)) : 0);
        break;
    case 8:
        bmpPitch = biWidth;
        pad = (((bmpPitch) % 4) ? (4 - ((bmpPitch) % 4)) : 0);
        break;
    default:
        bmpPitch = biWidth * 4;
        pad = 0;
        break;
    }
    while (bits > (Uint8 *) surface->pixels) {
        bits -= surface->pitch;
        switch (ExpandBMP) {
        case 1:
        case 4:
        case 8:
            {
                Uint8 pixel = 0;
                int shift = (8 - ExpandBMP);
                for (i = 0; i < surface->w; ++i) {
                    if (i % (8 / ExpandBMP) == 0) {
                        if (!SDL_RWread(src, &pixel, 1, 1)) {
                            IMG_SetError("Error reading from ICO");
                            was_error = SDL_TRUE;
                            goto done;
                        }
                    }
                    *((Uint32 *) bits + i) = (palette[pixel >> shift]);
                    pixel <<= ExpandBMP;
                }
            }
            break;

        default:
            if (SDL_RWread(src, bits, 1, surface->pitch)
                != surface->pitch) {
                SDL_Error(SDL_EFREAD);
                was_error = SDL_TRUE;
                goto done;
            }
            break;
        }
        /* Skip padding bytes, ugh */
        if (pad) {
            Uint8 padbyte;
            for (i = 0; i < pad; ++i) {
                SDL_RWread(src, &padbyte, 1, 1);
            }
        }
    }
    /* Read the mask pixels.  Note that the bmp image is upside down */
    bits = (Uint8 *) surface->pixels + (surface->h * surface->pitch);
    ExpandBMP = 1;
    bmpPitch = (biWidth + 7) >> 3;
    pad = (((bmpPitch) % 4) ? (4 - ((bmpPitch) % 4)) : 0);
    while (bits > (Uint8 *) surface->pixels) {
        Uint8 pixel = 0;
        int shift = (8 - ExpandBMP);

        bits -= surface->pitch;
        for (i = 0; i < surface->w; ++i) {
            if (i % (8 / ExpandBMP) == 0) {
                if (!SDL_RWread(src, &pixel, 1, 1)) {
                    IMG_SetError("Error reading from ICO");
                    was_error = SDL_TRUE;
                    goto done;
                }
            }
            *((Uint32 *) bits + i) |= ((pixel >> shift) ? 0 : 0xFF000000);
            pixel <<= ExpandBMP;
        }
        /* Skip padding bytes, ugh */
        if (pad) {
            Uint8 padbyte;
            for (i = 0; i < pad; ++i) {
                SDL_RWread(src, &padbyte, 1, 1);
            }
        }
    }
  done:
    if (was_error) {
        if (src) {
            SDL_RWseek(src, fp_offset, RW_SEEK_SET);
        }
        if (surface) {
            SDL_FreeSurface(surface);
        }
        surface = NULL;
    }
    if (freesrc && src) {
        SDL_RWclose(src);
    }
    return (surface);
}

/* Load a BMP type image from an SDL datasource */
SDL_Surface *IMG_LoadBMP_RW(SDL_RWops *src)
{
	return(LoadBMP_RW(src, 0));
}

/* Load a ICO type image from an SDL datasource */
SDL_Surface *IMG_LoadICO_RW(SDL_RWops *src)
{
	return(LoadICOCUR_RW(src, 1, 0));
}

/* Load a CUR type image from an SDL datasource */
SDL_Surface *IMG_LoadCUR_RW(SDL_RWops *src)
{
	return(LoadICOCUR_RW(src, 2, 0));
}

#else

/* See if an image is contained in a data source */
int IMG_isBMP(SDL_RWops *src)
{
	return(0);
}

int IMG_isICO(SDL_RWops *src)
{
	return(0);
}

int IMG_isCUR(SDL_RWops *src)
{
	return(0);
}

/* Load a BMP type image from an SDL datasource */
SDL_Surface *IMG_LoadBMP_RW(SDL_RWops *src)
{
	return(NULL);
}

/* Load a BMP type image from an SDL datasource */
SDL_Surface *IMG_LoadCUR_RW(SDL_RWops *src)
{
	return(NULL);
}

/* Load a BMP type image from an SDL datasource */
SDL_Surface *IMG_LoadICO_RW(SDL_RWops *src)
{
	return(NULL);
}

#endif /* LOAD_BMP */

#endif /* !defined(__APPLE__) || defined(SDL_IMAGE_USE_COMMON_BACKEND) */
