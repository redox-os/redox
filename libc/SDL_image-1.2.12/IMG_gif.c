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

/* This is a GIF image file loading framework */

#include <stdio.h>
#include <string.h>

#include "SDL_image.h"

#ifdef LOAD_GIF

/* See if an image is contained in a data source */
int IMG_isGIF(SDL_RWops *src)
{
	int start;
	int is_GIF;
	char magic[6];

	if ( !src )
		return 0;
	start = SDL_RWtell(src);
	is_GIF = 0;
	if ( SDL_RWread(src, magic, sizeof(magic), 1) ) {
		if ( (strncmp(magic, "GIF", 3) == 0) &&
		     ((memcmp(magic + 3, "87a", 3) == 0) ||
		      (memcmp(magic + 3, "89a", 3) == 0)) ) {
			is_GIF = 1;
		}
	}
	SDL_RWseek(src, start, RW_SEEK_SET);
	return(is_GIF);
}

/* Code from here to end of file has been adapted from XPaint:           */
/* +-------------------------------------------------------------------+ */
/* | Copyright 1990, 1991, 1993 David Koblas.			       | */
/* | Copyright 1996 Torsten Martinsen.				       | */
/* |   Permission to use, copy, modify, and distribute this software   | */
/* |   and its documentation for any purpose and without fee is hereby | */
/* |   granted, provided that the above copyright notice appear in all | */
/* |   copies and that both that copyright notice and this permission  | */
/* |   notice appear in supporting documentation.  This software is    | */
/* |   provided "as is" without express or implied warranty.	       | */
/* +-------------------------------------------------------------------+ */

/* Adapted for use in SDL by Sam Lantinga -- 7/20/98 */
#define USED_BY_SDL

#include <stdio.h>
#include <string.h>

#ifdef USED_BY_SDL
/* Changes to work with SDL:

   Include SDL header file
   Use SDL_Surface rather than xpaint Image structure
   Define SDL versions of RWSetMsg(), ImageNewCmap() and ImageSetCmap()
*/
#include "SDL.h"

#define Image			SDL_Surface
#define RWSetMsg		IMG_SetError
#define ImageNewCmap(w, h, s)	SDL_AllocSurface(SDL_SWSURFACE,w,h,8,0,0,0,0)
#define ImageSetCmap(s, i, R, G, B) do { \
				s->format->palette->colors[i].r = R; \
				s->format->palette->colors[i].g = G; \
				s->format->palette->colors[i].b = B; \
			} while (0)
/* * * * * */

#else

/* Original XPaint sources */

#include "image.h"
#include "rwTable.h"

#define SDL_RWops	FILE
#define SDL_RWclose	fclose

#endif /* USED_BY_SDL */


#define	MAXCOLORMAPSIZE		256

#define	TRUE	1
#define	FALSE	0

#define CM_RED		0
#define CM_GREEN	1
#define CM_BLUE		2

#define	MAX_LWZ_BITS		12

#define INTERLACE		0x40
#define LOCALCOLORMAP	0x80
#define BitSet(byte, bit)	(((byte) & (bit)) == (bit))

#define	ReadOK(file,buffer,len)	SDL_RWread(file, buffer, len, 1)

#define LM_to_uint(a,b)			(((b)<<8)|(a))

static struct {
    unsigned int Width;
    unsigned int Height;
    unsigned char ColorMap[3][MAXCOLORMAPSIZE];
    unsigned int BitPixel;
    unsigned int ColorResolution;
    unsigned int Background;
    unsigned int AspectRatio;
    int GrayScale;
} GifScreen;

static struct {
    int transparent;
    int delayTime;
    int inputFlag;
    int disposal;
} Gif89;

static int ReadColorMap(SDL_RWops * src, int number,
			unsigned char buffer[3][MAXCOLORMAPSIZE], int *flag);
static int DoExtension(SDL_RWops * src, int label);
static int GetDataBlock(SDL_RWops * src, unsigned char *buf);
static int GetCode(SDL_RWops * src, int code_size, int flag);
static int LWZReadByte(SDL_RWops * src, int flag, int input_code_size);
static Image *ReadImage(SDL_RWops * src, int len, int height, int,
			unsigned char cmap[3][MAXCOLORMAPSIZE],
			int gray, int interlace, int ignore);

Image *
IMG_LoadGIF_RW(SDL_RWops *src)
{
    int start;
    unsigned char buf[16];
    unsigned char c;
    unsigned char localColorMap[3][MAXCOLORMAPSIZE];
    int grayScale;
    int useGlobalColormap;
    int bitPixel;
    int imageCount = 0;
    char version[4];
    int imageNumber = 1;
    Image *image = NULL;

    if ( src == NULL ) {
	return NULL;
    }
    start = SDL_RWtell(src);

    if (!ReadOK(src, buf, 6)) {
	RWSetMsg("error reading magic number");
        goto done;
    }
    if (strncmp((char *) buf, "GIF", 3) != 0) {
	RWSetMsg("not a GIF file");
        goto done;
    }
    memcpy(version, (char *) buf + 3, 3);
    version[3] = '\0';

    if ((strcmp(version, "87a") != 0) && (strcmp(version, "89a") != 0)) {
	RWSetMsg("bad version number, not '87a' or '89a'");
        goto done;
    }
    Gif89.transparent = -1;
    Gif89.delayTime = -1;
    Gif89.inputFlag = -1;
    Gif89.disposal = 0;

    if (!ReadOK(src, buf, 7)) {
	RWSetMsg("failed to read screen descriptor");
        goto done;
    }
    GifScreen.Width = LM_to_uint(buf[0], buf[1]);
    GifScreen.Height = LM_to_uint(buf[2], buf[3]);
    GifScreen.BitPixel = 2 << (buf[4] & 0x07);
    GifScreen.ColorResolution = (((buf[4] & 0x70) >> 3) + 1);
    GifScreen.Background = buf[5];
    GifScreen.AspectRatio = buf[6];

    if (BitSet(buf[4], LOCALCOLORMAP)) {	/* Global Colormap */
	if (ReadColorMap(src, GifScreen.BitPixel, GifScreen.ColorMap,
			 &GifScreen.GrayScale)) {
	    RWSetMsg("error reading global colormap");
            goto done;
	}
    }
    do {
	if (!ReadOK(src, &c, 1)) {
	    RWSetMsg("EOF / read error on image data");
            goto done;
	}
	if (c == ';') {		/* GIF terminator */
	    if (imageCount < imageNumber) {
		RWSetMsg("only %d image%s found in file",
			 imageCount, imageCount > 1 ? "s" : "");
                goto done;
	    }
	}
	if (c == '!') {		/* Extension */
	    if (!ReadOK(src, &c, 1)) {
		RWSetMsg("EOF / read error on extention function code");
                goto done;
	    }
	    DoExtension(src, c);
	    continue;
	}
	if (c != ',') {		/* Not a valid start character */
	    continue;
	}
	++imageCount;

	if (!ReadOK(src, buf, 9)) {
	    RWSetMsg("couldn't read left/top/width/height");
            goto done;
	}
	useGlobalColormap = !BitSet(buf[8], LOCALCOLORMAP);

	bitPixel = 1 << ((buf[8] & 0x07) + 1);

	if (!useGlobalColormap) {
	    if (ReadColorMap(src, bitPixel, localColorMap, &grayScale)) {
		RWSetMsg("error reading local colormap");
                goto done;
	    }
	    image = ReadImage(src, LM_to_uint(buf[4], buf[5]),
			      LM_to_uint(buf[6], buf[7]),
			      bitPixel, localColorMap, grayScale,
			      BitSet(buf[8], INTERLACE),
			      imageCount != imageNumber);
	} else {
	    image = ReadImage(src, LM_to_uint(buf[4], buf[5]),
			      LM_to_uint(buf[6], buf[7]),
			      GifScreen.BitPixel, GifScreen.ColorMap,
			      GifScreen.GrayScale, BitSet(buf[8], INTERLACE),
			      imageCount != imageNumber);
	}
    } while (image == NULL);

#ifdef USED_BY_SDL
    if ( Gif89.transparent >= 0 ) {
        SDL_SetColorKey(image, SDL_SRCCOLORKEY, Gif89.transparent);
    }
#endif

done:
    if ( image == NULL ) {
        SDL_RWseek(src, start, RW_SEEK_SET);
    }
    return image;
}

static int
ReadColorMap(SDL_RWops *src, int number,
             unsigned char buffer[3][MAXCOLORMAPSIZE], int *gray)
{
    int i;
    unsigned char rgb[3];
    int flag;

    flag = TRUE;

    for (i = 0; i < number; ++i) {
	if (!ReadOK(src, rgb, sizeof(rgb))) {
	    RWSetMsg("bad colormap");
	    return 1;
	}
	buffer[CM_RED][i] = rgb[0];
	buffer[CM_GREEN][i] = rgb[1];
	buffer[CM_BLUE][i] = rgb[2];
	flag &= (rgb[0] == rgb[1] && rgb[1] == rgb[2]);
    }

#if 0
    if (flag)
	*gray = (number == 2) ? PBM_TYPE : PGM_TYPE;
    else
	*gray = PPM_TYPE;
#else
    *gray = 0;
#endif

    return FALSE;
}

static int
DoExtension(SDL_RWops *src, int label)
{
    static unsigned char buf[256];
    char *str;

    switch (label) {
    case 0x01:			/* Plain Text Extension */
	str = "Plain Text Extension";
	break;
    case 0xff:			/* Application Extension */
	str = "Application Extension";
	break;
    case 0xfe:			/* Comment Extension */
	str = "Comment Extension";
	while (GetDataBlock(src, (unsigned char *) buf) != 0)
	    ;
	return FALSE;
    case 0xf9:			/* Graphic Control Extension */
	str = "Graphic Control Extension";
	(void) GetDataBlock(src, (unsigned char *) buf);
	Gif89.disposal = (buf[0] >> 2) & 0x7;
	Gif89.inputFlag = (buf[0] >> 1) & 0x1;
	Gif89.delayTime = LM_to_uint(buf[1], buf[2]);
	if ((buf[0] & 0x1) != 0)
	    Gif89.transparent = buf[3];

	while (GetDataBlock(src, (unsigned char *) buf) != 0)
	    ;
	return FALSE;
    default:
	str = (char *)buf;
	sprintf(str, "UNKNOWN (0x%02x)", label);
	break;
    }

    while (GetDataBlock(src, (unsigned char *) buf) != 0)
	;

    return FALSE;
}

static int ZeroDataBlock = FALSE;

static int
GetDataBlock(SDL_RWops *src, unsigned char *buf)
{
    unsigned char count;

    if (!ReadOK(src, &count, 1)) {
	/* pm_message("error in getting DataBlock size" ); */
	return -1;
    }
    ZeroDataBlock = count == 0;

    if ((count != 0) && (!ReadOK(src, buf, count))) {
	/* pm_message("error in reading DataBlock" ); */
	return -1;
    }
    return count;
}

static int
GetCode(SDL_RWops *src, int code_size, int flag)
{
    static unsigned char buf[280];
    static int curbit, lastbit, done, last_byte;
    int i, j, ret;
    unsigned char count;

    if (flag) {
	curbit = 0;
	lastbit = 0;
	done = FALSE;
	return 0;
    }
    if ((curbit + code_size) >= lastbit) {
	if (done) {
	    if (curbit >= lastbit)
		RWSetMsg("ran off the end of my bits");
	    return -1;
	}
	buf[0] = buf[last_byte - 2];
	buf[1] = buf[last_byte - 1];

	if ((count = GetDataBlock(src, &buf[2])) == 0)
	    done = TRUE;

	last_byte = 2 + count;
	curbit = (curbit - lastbit) + 16;
	lastbit = (2 + count) * 8;
    }
    ret = 0;
    for (i = curbit, j = 0; j < code_size; ++i, ++j)
	ret |= ((buf[i / 8] & (1 << (i % 8))) != 0) << j;

    curbit += code_size;

    return ret;
}

static int
LWZReadByte(SDL_RWops *src, int flag, int input_code_size)
{
    static int fresh = FALSE;
    int code, incode;
    static int code_size, set_code_size;
    static int max_code, max_code_size;
    static int firstcode, oldcode;
    static int clear_code, end_code;
    static int table[2][(1 << MAX_LWZ_BITS)];
    static int stack[(1 << (MAX_LWZ_BITS)) * 2], *sp;
    register int i;

    /* Fixed buffer overflow found by Michael Skladnikiewicz */
    if (input_code_size > MAX_LWZ_BITS)
        return -1;

    if (flag) {
	set_code_size = input_code_size;
	code_size = set_code_size + 1;
	clear_code = 1 << set_code_size;
	end_code = clear_code + 1;
	max_code_size = 2 * clear_code;
	max_code = clear_code + 2;

	GetCode(src, 0, TRUE);

	fresh = TRUE;

	for (i = 0; i < clear_code; ++i) {
	    table[0][i] = 0;
	    table[1][i] = i;
	}
	for (; i < (1 << MAX_LWZ_BITS); ++i)
	    table[0][i] = table[1][0] = 0;

	sp = stack;

	return 0;
    } else if (fresh) {
	fresh = FALSE;
	do {
	    firstcode = oldcode = GetCode(src, code_size, FALSE);
	} while (firstcode == clear_code);
	return firstcode;
    }
    if (sp > stack)
	return *--sp;

    while ((code = GetCode(src, code_size, FALSE)) >= 0) {
	if (code == clear_code) {
	    for (i = 0; i < clear_code; ++i) {
		table[0][i] = 0;
		table[1][i] = i;
	    }
	    for (; i < (1 << MAX_LWZ_BITS); ++i)
		table[0][i] = table[1][i] = 0;
	    code_size = set_code_size + 1;
	    max_code_size = 2 * clear_code;
	    max_code = clear_code + 2;
	    sp = stack;
	    firstcode = oldcode = GetCode(src, code_size, FALSE);
	    return firstcode;
	} else if (code == end_code) {
	    int count;
	    unsigned char buf[260];

	    if (ZeroDataBlock)
		return -2;

	    while ((count = GetDataBlock(src, buf)) > 0)
		;

	    if (count != 0) {
		/*
		 * pm_message("missing EOD in data stream (common occurence)");
		 */
	    }
	    return -2;
	}
	incode = code;

	if (code >= max_code) {
	    *sp++ = firstcode;
	    code = oldcode;
	}
	while (code >= clear_code) {
	    *sp++ = table[1][code];
	    if (code == table[0][code])
		RWSetMsg("circular table entry BIG ERROR");
	    code = table[0][code];
	}

	*sp++ = firstcode = table[1][code];

	if ((code = max_code) < (1 << MAX_LWZ_BITS)) {
	    table[0][code] = oldcode;
	    table[1][code] = firstcode;
	    ++max_code;
	    if ((max_code >= max_code_size) &&
		(max_code_size < (1 << MAX_LWZ_BITS))) {
		max_code_size *= 2;
		++code_size;
	    }
	}
	oldcode = incode;

	if (sp > stack)
	    return *--sp;
    }
    return code;
}

static Image *
ReadImage(SDL_RWops * src, int len, int height, int cmapSize,
	  unsigned char cmap[3][MAXCOLORMAPSIZE],
	  int gray, int interlace, int ignore)
{
    Image *image;
    unsigned char c;
    int i, v;
    int xpos = 0, ypos = 0, pass = 0;

    /*
    **	Initialize the compression routines
     */
    if (!ReadOK(src, &c, 1)) {
	RWSetMsg("EOF / read error on image data");
	return NULL;
    }
    if (LWZReadByte(src, TRUE, c) < 0) {
	RWSetMsg("error reading image");
	return NULL;
    }
    /*
    **	If this is an "uninteresting picture" ignore it.
     */
    if (ignore) {
	while (LWZReadByte(src, FALSE, c) >= 0)
	    ;
	return NULL;
    }
    image = ImageNewCmap(len, height, cmapSize);

    for (i = 0; i < cmapSize; i++)
	ImageSetCmap(image, i, cmap[CM_RED][i],
		     cmap[CM_GREEN][i], cmap[CM_BLUE][i]);

    while ((v = LWZReadByte(src, FALSE, c)) >= 0) {
#ifdef USED_BY_SDL
	((Uint8 *)image->pixels)[xpos + ypos * image->pitch] = v;
#else
	image->data[xpos + ypos * len] = v;
#endif
	++xpos;
	if (xpos == len) {
	    xpos = 0;
	    if (interlace) {
		switch (pass) {
		case 0:
		case 1:
		    ypos += 8;
		    break;
		case 2:
		    ypos += 4;
		    break;
		case 3:
		    ypos += 2;
		    break;
		}

		if (ypos >= height) {
		    ++pass;
		    switch (pass) {
		    case 1:
			ypos = 4;
			break;
		    case 2:
			ypos = 2;
			break;
		    case 3:
			ypos = 1;
			break;
		    default:
			goto fini;
		    }
		}
	    } else {
		++ypos;
	    }
	}
	if (ypos >= height)
	    break;
    }

  fini:

    return image;
}

#else

/* See if an image is contained in a data source */
int IMG_isGIF(SDL_RWops *src)
{
	return(0);
}

/* Load a GIF type image from an SDL datasource */
SDL_Surface *IMG_LoadGIF_RW(SDL_RWops *src)
{
	return(NULL);
}

#endif /* LOAD_GIF */

#endif /* !defined(__APPLE__) || defined(SDL_IMAGE_USE_COMMON_BACKEND) */
