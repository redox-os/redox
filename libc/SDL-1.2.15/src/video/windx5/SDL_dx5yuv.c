/*
    SDL - Simple DirectMedia Layer
    Copyright (C) 1997-2012 Sam Lantinga

    This library is free software; you can redistribute it and/or
    modify it under the terms of the GNU Lesser General Public
    License as published by the Free Software Foundation; either
    version 2.1 of the License, or (at your option) any later version.

    This library is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
    Lesser General Public License for more details.

    You should have received a copy of the GNU Lesser General Public
    License along with this library; if not, write to the Free Software
    Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA  02110-1301  USA

    Sam Lantinga
    slouken@libsdl.org
*/
#include "SDL_config.h"

/* This is the DirectDraw implementation of YUV video overlays */
#include "directx.h"
#include "SDL_video.h"
#include "SDL_dx5yuv_c.h"
#include "../SDL_yuvfuncs.h"

//#define USE_DIRECTX_OVERLAY

/* The functions used to manipulate software video overlays */
static struct private_yuvhwfuncs dx5_yuvfuncs = {
	DX5_LockYUVOverlay,
	DX5_UnlockYUVOverlay,
	DX5_DisplayYUVOverlay,
	DX5_FreeYUVOverlay
};

struct private_yuvhwdata {
	LPDIRECTDRAWSURFACE3 surface;

	/* These are just so we don't have to allocate them separately */
	Uint16 pitches[3];
	Uint8 *planes[3];
};


static LPDIRECTDRAWSURFACE3 CreateYUVSurface(_THIS,
                                         int width, int height, Uint32 format)
{
	HRESULT result;
	LPDIRECTDRAWSURFACE  dd_surface1;
	LPDIRECTDRAWSURFACE3 dd_surface3;
	DDSURFACEDESC ddsd;

	/* Set up the surface description */
	SDL_memset(&ddsd, 0, sizeof(ddsd));
	ddsd.dwSize = sizeof(ddsd);
	ddsd.dwFlags = (DDSD_WIDTH|DDSD_HEIGHT|DDSD_CAPS|DDSD_PIXELFORMAT);
	ddsd.dwWidth = width;
	ddsd.dwHeight= height;
#ifdef USE_DIRECTX_OVERLAY
	ddsd.ddsCaps.dwCaps = (DDSCAPS_OVERLAY|DDSCAPS_VIDEOMEMORY);
#else
	ddsd.ddsCaps.dwCaps = (DDSCAPS_OFFSCREENPLAIN|DDSCAPS_VIDEOMEMORY);
#endif
	ddsd.ddpfPixelFormat.dwSize = sizeof(ddsd.ddpfPixelFormat);
	ddsd.ddpfPixelFormat.dwFlags = DDPF_FOURCC;
	ddsd.ddpfPixelFormat.dwFourCC = format;

	/* Create the DirectDraw video surface */
	result = IDirectDraw2_CreateSurface(ddraw2, &ddsd, &dd_surface1, NULL); 
	if ( result != DD_OK ) {
		SetDDerror("DirectDraw2::CreateSurface", result);
		return(NULL);
	}
	result = IDirectDrawSurface_QueryInterface(dd_surface1,
			&IID_IDirectDrawSurface3, (LPVOID *)&dd_surface3);
	IDirectDrawSurface_Release(dd_surface1);
	if ( result != DD_OK ) {
		SetDDerror("DirectDrawSurface::QueryInterface", result);
		return(NULL);
	}

	/* Make sure the surface format was set properly */
	SDL_memset(&ddsd, 0, sizeof(ddsd));
	ddsd.dwSize = sizeof(ddsd);
	result = IDirectDrawSurface3_Lock(dd_surface3, NULL,
					  &ddsd, DDLOCK_NOSYSLOCK, NULL);
	if ( result != DD_OK ) {
		SetDDerror("DirectDrawSurface3::Lock", result);
		IDirectDrawSurface_Release(dd_surface3);
		return(NULL);
	}
	IDirectDrawSurface3_Unlock(dd_surface3, NULL);

	if ( !(ddsd.ddpfPixelFormat.dwFlags & DDPF_FOURCC) ||
	      (ddsd.ddpfPixelFormat.dwFourCC != format) ) {
		SDL_SetError("DDraw didn't use requested FourCC format");
		IDirectDrawSurface_Release(dd_surface3);
		return(NULL);
	}

	/* We're ready to go! */
	return(dd_surface3);
}

#ifdef DEBUG_YUV
static char *PrintFOURCC(Uint32 code)
{
	static char buf[5];

	buf[3] = code >> 24;
	buf[2] = (code >> 16) & 0xFF;
	buf[1] = (code >> 8) & 0xFF;
	buf[0] = (code & 0xFF);
	return(buf);
}
#endif

SDL_Overlay *DX5_CreateYUVOverlay(_THIS, int width, int height, Uint32 format, SDL_Surface *display)
{
	SDL_Overlay *overlay;
	struct private_yuvhwdata *hwdata;

#ifdef DEBUG_YUV
	DWORD numcodes;
	DWORD *codes;

	printf("FOURCC format requested: 0x%x\n", PrintFOURCC(format));
	IDirectDraw2_GetFourCCCodes(ddraw2, &numcodes, NULL);
	if ( numcodes ) {
		DWORD i;
		codes = SDL_malloc(numcodes*sizeof(*codes));
		if ( codes ) {
			IDirectDraw2_GetFourCCCodes(ddraw2, &numcodes, codes);
			for ( i=0; i<numcodes; ++i ) {
				fprintf(stderr, "Code %d: 0x%x\n", i, PrintFOURCC(codes[i]));
			}
			SDL_free(codes);
		}
	} else {
		fprintf(stderr, "No FOURCC codes supported\n");
	}
#endif

	/* Create the overlay structure */
	overlay = (SDL_Overlay *)SDL_malloc(sizeof *overlay);
	if ( overlay == NULL ) {
		SDL_OutOfMemory();
		return(NULL);
	}
	SDL_memset(overlay, 0, (sizeof *overlay));

	/* Fill in the basic members */
	overlay->format = format;
	overlay->w = width;
	overlay->h = height;

	/* Set up the YUV surface function structure */
	overlay->hwfuncs = &dx5_yuvfuncs;

	/* Create the pixel data and lookup tables */
	hwdata = (struct private_yuvhwdata *)SDL_malloc(sizeof *hwdata);
	overlay->hwdata = hwdata;
	if ( hwdata == NULL ) {
		SDL_OutOfMemory();
		SDL_FreeYUVOverlay(overlay);
		return(NULL);
	}
	hwdata->surface = CreateYUVSurface(this, width, height, format);
	if ( hwdata->surface == NULL ) {
		SDL_FreeYUVOverlay(overlay);
		return(NULL);
	}
	overlay->hw_overlay = 1;

	/* Set up the plane pointers */
	overlay->pitches = hwdata->pitches;
	overlay->pixels = hwdata->planes;
	switch (format) {
	    case SDL_YV12_OVERLAY:
	    case SDL_IYUV_OVERLAY:
		overlay->planes = 3;
		break;
	    default:
		overlay->planes = 1;
		break;
	}

	/* We're all done.. */
	return(overlay);
}

int DX5_LockYUVOverlay(_THIS, SDL_Overlay *overlay)
{
	HRESULT result;
	LPDIRECTDRAWSURFACE3 surface;
	DDSURFACEDESC ddsd;

	surface = overlay->hwdata->surface;
	SDL_memset(&ddsd, 0, sizeof(ddsd));
	ddsd.dwSize = sizeof(ddsd);
	result = IDirectDrawSurface3_Lock(surface, NULL,
					  &ddsd, DDLOCK_NOSYSLOCK, NULL);
	if ( result == DDERR_SURFACELOST ) {
		result = IDirectDrawSurface3_Restore(surface);
		result = IDirectDrawSurface3_Lock(surface, NULL, &ddsd, 
					(DDLOCK_NOSYSLOCK|DDLOCK_WAIT), NULL);
	}
	if ( result != DD_OK ) {
		SetDDerror("DirectDrawSurface3::Lock", result);
		return(-1);
	}

	/* Find the pitch and offset values for the overlay */
#if defined(NONAMELESSUNION)
	overlay->pitches[0] = (Uint16)ddsd.u1.lPitch;
#else
	overlay->pitches[0] = (Uint16)ddsd.lPitch;
#endif
	overlay->pixels[0] = (Uint8 *)ddsd.lpSurface;
	switch (overlay->format) {
	    case SDL_YV12_OVERLAY:
	    case SDL_IYUV_OVERLAY:
		/* Add the two extra planes */
		overlay->pitches[1] = overlay->pitches[0] / 2;
		overlay->pitches[2] = overlay->pitches[0] / 2;
	        overlay->pixels[1] = overlay->pixels[0] +
		                     overlay->pitches[0] * overlay->h;
	        overlay->pixels[2] = overlay->pixels[1] +
		                     overlay->pitches[1] * overlay->h / 2;
	        break;
	    default:
		/* Only one plane, no worries */
		break;
	}
	return(0);
}

void DX5_UnlockYUVOverlay(_THIS, SDL_Overlay *overlay)
{
	LPDIRECTDRAWSURFACE3 surface;

	surface = overlay->hwdata->surface;
	IDirectDrawSurface3_Unlock(surface, NULL);
}

int DX5_DisplayYUVOverlay(_THIS, SDL_Overlay *overlay, SDL_Rect *src, SDL_Rect *dst)
{
	HRESULT result;
	LPDIRECTDRAWSURFACE3 surface;
	RECT srcrect, dstrect;

	surface = overlay->hwdata->surface;
	srcrect.top = src->y;
	srcrect.bottom = srcrect.top+src->h;
	srcrect.left = src->x;
	srcrect.right = srcrect.left+src->w;
	dstrect.top = SDL_bounds.top+dst->y;
	dstrect.left = SDL_bounds.left+dst->x;
	dstrect.bottom = dstrect.top+dst->h;
	dstrect.right = dstrect.left+dst->w;
#ifdef USE_DIRECTX_OVERLAY
	result = IDirectDrawSurface3_UpdateOverlay(surface, &srcrect,
				SDL_primary, &dstrect, DDOVER_SHOW, NULL);
	if ( result != DD_OK ) {
		SetDDerror("DirectDrawSurface3::UpdateOverlay", result);
		return(-1);
	}
#else
	result = IDirectDrawSurface3_Blt(SDL_primary, &dstrect, surface, &srcrect,
							DDBLT_WAIT, NULL);
	if ( result != DD_OK ) {
		SetDDerror("DirectDrawSurface3::Blt", result);
		return(-1);
	}
#endif
	return(0);
}

void DX5_FreeYUVOverlay(_THIS, SDL_Overlay *overlay)
{
	struct private_yuvhwdata *hwdata;

	hwdata = overlay->hwdata;
	if ( hwdata ) {
		if ( hwdata->surface ) {
			IDirectDrawSurface_Release(hwdata->surface);
		}
		SDL_free(hwdata);
		overlay->hwdata = NULL;
	}
}

