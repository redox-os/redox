/* 

SDL_gfxBlitFunc.h: custom blitters

Copyright (C) 2001-2012  Andreas Schiffler

This software is provided 'as-is', without any express or implied
warranty. In no event will the authors be held liable for any damages
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

3. This notice may not be removed or altered from any source
distribution.

Andreas Schiffler -- aschiffler at ferzkopp dot net

*/

#ifndef _SDL_gfxBlitFunc_h
#define _SDL_gfxBlitFunc_h

/* Set up for C function definitions, even when using C++ */
#ifdef __cplusplus
extern    "C" {
#endif

#include <stdio.h>
#include <stdlib.h>

#include "SDL.h"
#include "SDL_video.h"


	extern const unsigned int GFX_ALPHA_ADJUST_ARRAY[256];

	/* ---- Function Prototypes */

#ifdef _MSC_VER
#  if defined(DLL_EXPORT) && !defined(LIBSDL_GFX_DLL_IMPORT)
#    define SDL_GFXBLITFUNC_SCOPE __declspec(dllexport)
#  else
#    ifdef LIBSDL_GFX_DLL_IMPORT
#      define SDL_GFXBLITFUNC_SCOPE __declspec(dllimport)
#    endif
#  endif
#endif
#ifndef SDL_GFXBLITFUNC_SCOPE
#  define SDL_GFXBLITFUNC_SCOPE extern
#endif


	SDL_GFXBLITFUNC_SCOPE int SDL_gfxBlitRGBA(SDL_Surface * src, SDL_Rect * srcrect, SDL_Surface * dst, SDL_Rect * dstrect);

	SDL_GFXBLITFUNC_SCOPE int SDL_gfxSetAlpha(SDL_Surface * src, Uint8 a);

	SDL_GFXBLITFUNC_SCOPE int SDL_gfxMultiplyAlpha(SDL_Surface * src, Uint8 a);

	/* -------- Macros */

	/* Define SDL macros locally as a substitute for an #include "SDL_blit.h", */
	/* which doesn't work since the include file doesn't get installed.       */

	/*!
	\brief The structure passed to the low level blit functions.
	*/
	typedef struct {
		Uint8    *s_pixels;
		int       s_width;
		int       s_height;
		int       s_skip;
		Uint8    *d_pixels;
		int       d_width;
		int       d_height;
		int       d_skip;
		void     *aux_data;
		SDL_PixelFormat *src;
		Uint8    *table;
		SDL_PixelFormat *dst;
	} SDL_gfxBlitInfo;

	/*!
	\brief Unwrap RGBA values from a pixel using mask, shift and loss for surface.
	*/
#define GFX_RGBA_FROM_PIXEL(pixel, fmt, r, g, b, a)				\
	{									\
	r = ((pixel&fmt->Rmask)>>fmt->Rshift)<<fmt->Rloss; 		\
	g = ((pixel&fmt->Gmask)>>fmt->Gshift)<<fmt->Gloss; 		\
	b = ((pixel&fmt->Bmask)>>fmt->Bshift)<<fmt->Bloss; 		\
	a = ((pixel&fmt->Amask)>>fmt->Ashift)<<fmt->Aloss;	 	\
	}

	/*!
	\brief Disassemble buffer pointer into a pixel and separate RGBA values.
	*/
#define GFX_DISASSEMBLE_RGBA(buf, bpp, fmt, pixel, r, g, b, a)			   \
	do {									   \
	pixel = *((Uint32 *)(buf));			   		   \
	GFX_RGBA_FROM_PIXEL(pixel, fmt, r, g, b, a);			   \
	pixel &= ~fmt->Amask;						   \
	} while(0)

	/*!
	\brief Wrap a pixel from RGBA values using mask, shift and loss for surface.
	*/
#define GFX_PIXEL_FROM_RGBA(pixel, fmt, r, g, b, a)				\
	{									\
	pixel = ((r>>fmt->Rloss)<<fmt->Rshift)|				\
	((g>>fmt->Gloss)<<fmt->Gshift)|				\
	((b>>fmt->Bloss)<<fmt->Bshift)|				\
	((a<<fmt->Aloss)<<fmt->Ashift);				\
	}

	/*!
	\brief Assemble pixel into buffer pointer from separate RGBA values.
	*/
#define GFX_ASSEMBLE_RGBA(buf, bpp, fmt, r, g, b, a)			\
	{									\
	Uint32 pixel;					\
	\
	GFX_PIXEL_FROM_RGBA(pixel, fmt, r, g, b, a);	\
	*((Uint32 *)(buf)) = pixel;			\
	}

	/*!
	\brief Blend the RGB values of two pixels based on a source alpha value.
	*/
#define GFX_ALPHA_BLEND(sR, sG, sB, A, dR, dG, dB)	\
	do {						\
	dR = (((sR-dR)*(A))/255)+dR;		\
	dG = (((sG-dG)*(A))/255)+dG;		\
	dB = (((sB-dB)*(A))/255)+dB;		\
	} while(0)

	/*!
	\brief 4-times unrolled DUFFs loop.

	This is a very useful loop for optimizing blitters.
	*/
#define GFX_DUFFS_LOOP4(pixel_copy_increment, width)			\
	{ int n = (width+3)/4;							\
	switch (width & 3) {						\
	case 0: do {	pixel_copy_increment;				\
	case 3:		pixel_copy_increment;				\
	case 2:		pixel_copy_increment;				\
	case 1:		pixel_copy_increment;				\
	} while ( --n > 0 );					\
	}								\
	}



	/* Ends C function definitions when using C++ */
#ifdef __cplusplus
}
#endif

#endif /* _SDL_gfxBlitFunc_h */
