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

#define WIN32_LEAN_AND_MEAN
#include <windows.h>

#include "SDL_version.h"
#include "SDL_video.h"
#include "SDL_loadso.h"
#include "SDL_syswm.h"
#include "../SDL_pixels_c.h"
#include "../SDL_cursor_c.h"
#include "SDL_syswm_c.h"
#include "SDL_wingl_c.h"


#ifdef _WIN32_WCE
#define DISABLE_ICON_SUPPORT
#endif

/* The screen icon -- needs to be freed on SDL_VideoQuit() */
HICON   screen_icn = NULL;

/* Win32 icon mask semantics are different from those of SDL:
     SDL applies the mask to the icon and copies result to desktop.
     Win32 applies the mask to the desktop and XORs the icon on.
   This means that the SDL mask needs to be applied to the icon and
   then inverted and passed to Win32.
*/
void WIN_SetWMIcon(_THIS, SDL_Surface *icon, Uint8 *mask)
{
#ifdef DISABLE_ICON_SUPPORT
	return;
#else
	SDL_Palette *pal_256;
	SDL_Surface *icon_256;
	Uint8 *pdata, *pwin32;
	Uint8 *mdata, *mwin32, m = 0;
	int icon_len;
	int icon_plen;
	int icon_mlen;
	int icon_pitch;
	int mask_pitch;
	SDL_Rect bounds;
	int i, skip;
	int row, col;
	struct /* quasi-BMP format */ Win32Icon {
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
		struct /* RGBQUAD -- note it's BGR ordered */ {
			Uint8 rgbBlue;
			Uint8 rgbGreen;
			Uint8 rgbRed;
			Uint8 rgbReserved;
		} biColors[256];
		/* Pixels:
		Uint8 pixels[]
		*/
		/* Mask:
		Uint8 mask[]
		*/
	} *icon_win32;
	
	/* Allocate the win32 bmp icon and set everything to zero */
	icon_pitch = ((icon->w+3)&~3);
	mask_pitch = ((icon->w+7)/8);
	icon_plen = icon->h*icon_pitch;
	icon_mlen = icon->h*mask_pitch;
	icon_len = sizeof(*icon_win32)+icon_plen+icon_mlen;
	icon_win32 = (struct Win32Icon *)SDL_stack_alloc(Uint8, icon_len);
	if ( icon_win32 == NULL ) {
		return;
	}
	SDL_memset(icon_win32, 0, icon_len);

	/* Set the basic BMP parameters */
	icon_win32->biSize = sizeof(*icon_win32)-sizeof(icon_win32->biColors);
	icon_win32->biWidth = icon->w;
	icon_win32->biHeight = icon->h*2;
	icon_win32->biPlanes = 1;
	icon_win32->biBitCount = 8;
	icon_win32->biSizeImage = icon_plen+icon_mlen;

	/* Allocate a standard 256 color icon surface */
	icon_256 = SDL_CreateRGBSurface(SDL_SWSURFACE, icon->w, icon->h,
					 icon_win32->biBitCount, 0, 0, 0, 0);
	if ( icon_256 == NULL ) {
		SDL_stack_free(icon_win32);
		return;
	}
	pal_256 = icon_256->format->palette;
	if (icon->format->palette && 
		(icon->format->BitsPerPixel == icon_256->format->BitsPerPixel)){
		Uint8 black;
		SDL_memcpy(pal_256->colors, icon->format->palette->colors,
					pal_256->ncolors*sizeof(SDL_Color));
		/* Make sure that 0 is black! */
		black = SDL_FindColor(pal_256, 0x00, 0x00, 0x00);
		pal_256->colors[black] = pal_256->colors[0];
		pal_256->colors[0].r = 0x00;
		pal_256->colors[0].g = 0x00;
		pal_256->colors[0].b = 0x00;
	} else {
		SDL_DitherColors(pal_256->colors,
					icon_256->format->BitsPerPixel);
	}

	/* Now copy color data to the icon BMP */
	for ( i=0; i<(1<<icon_win32->biBitCount); ++i ) {
		icon_win32->biColors[i].rgbRed = pal_256->colors[i].r;
		icon_win32->biColors[i].rgbGreen = pal_256->colors[i].g;
		icon_win32->biColors[i].rgbBlue = pal_256->colors[i].b;
	}

	/* Convert icon to a standard surface format.  This may not always
	   be necessary, as Windows supports a variety of BMP formats, but
	   it greatly simplifies our code.
	*/ 
    bounds.x = 0;
    bounds.y = 0;
    bounds.w = icon->w;
    bounds.h = icon->h;
    if ( SDL_LowerBlit(icon, &bounds, icon_256, &bounds) < 0 ) {
	    SDL_stack_free(icon_win32);
		SDL_FreeSurface(icon_256);
        return;
	}

	/* Copy pixels upside-down to icon BMP, masked with the icon mask */
	if ( SDL_MUSTLOCK(icon_256) || (icon_256->pitch != icon_pitch) ) {
		SDL_stack_free(icon_win32);
		SDL_FreeSurface(icon_256);
		SDL_SetError("Warning: Unexpected icon_256 characteristics");
		return;
	}
	pdata = (Uint8 *)icon_256->pixels;
	mdata = mask;
	pwin32 = (Uint8 *)icon_win32+sizeof(*icon_win32)+icon_plen-icon_pitch;
	skip = icon_pitch - icon->w;
	for ( row=0; row<icon->h; ++row ) {
		for ( col=0; col<icon->w; ++col ) {
			if ( (col%8) == 0 ) {
				m = *mdata++;
			}
			if ( (m&0x80) != 0x00 ) {
				*pwin32 = *pdata;
			}
			m <<= 1;
			++pdata;
			++pwin32;
		}
		pdata  += skip;
		pwin32 += skip;
		pwin32 -= 2*icon_pitch;
	}
	SDL_FreeSurface(icon_256);

	/* Copy mask inverted and upside-down to icon BMP */
	mdata = mask;
	mwin32 = (Uint8 *)icon_win32
			+sizeof(*icon_win32)+icon_plen+icon_mlen-mask_pitch;
	for ( row=0; row<icon->h; ++row ) {
		for ( col=0; col<mask_pitch; ++col ) {
			*mwin32++ = ~*mdata++;
		}
		mwin32 -= 2*mask_pitch;
	}

	/* Finally, create the icon handle and set the window icon */
	screen_icn = CreateIconFromResourceEx((Uint8 *)icon_win32, icon_len,
			TRUE, 0x00030000, icon->w, icon->h, LR_DEFAULTCOLOR);
	if ( screen_icn == NULL ) {
		SDL_SetError("Couldn't create Win32 icon handle");
	} else {
		SetClassLongPtr(SDL_Window, GCLP_HICON, (LONG_PTR)screen_icn);
	}
	SDL_stack_free(icon_win32);
#endif /* DISABLE_ICON_SUPPORT */
}

typedef BOOL (WINAPI *PtrSetWindowTextW)(HWND hWnd, LPCWSTR lpString);

void WIN_SetWMCaption(_THIS, const char *title, const char *icon)
{
#ifdef _WIN32_WCE
	/* WinCE uses the UNICODE version */
	LPWSTR lpszW = SDL_iconv_utf8_ucs2((char *)title);
	SetWindowText(SDL_Window, lpszW);
	SDL_free(lpszW);
#else
	Uint16 *lpsz = SDL_iconv_utf8_ucs2(title);
	size_t len = WideCharToMultiByte(CP_ACP, 0, lpsz, -1, NULL, 0, NULL, NULL);
	char *cvt = SDL_stack_alloc(char, len + 1);
	WideCharToMultiByte(CP_ACP, 0, lpsz, -1, cvt, len, NULL, NULL);
	SetWindowText(SDL_Window, cvt);
	SDL_stack_free(cvt);
	SDL_free(lpsz);
#endif
}

int WIN_IconifyWindow(_THIS)
{
	ShowWindow(SDL_Window, SW_MINIMIZE);
	return(1);
}

SDL_GrabMode WIN_GrabInput(_THIS, SDL_GrabMode mode)
{
	if ( mode == SDL_GRAB_OFF ) {
		ClipCursor(NULL);
		if ( !(SDL_cursorstate & CURSOR_VISIBLE) ) {
		/*	RJR: March 28, 2000
			must be leaving relative mode, move mouse from
			center of window to where it belongs ... */
			POINT pt;
			int x, y;
			SDL_GetMouseState(&x,&y);
			pt.x = x;
			pt.y = y;
			ClientToScreen(SDL_Window, &pt);
			SetCursorPos(pt.x,pt.y);
		}
#ifdef _WIN32_WCE
		AllKeys(0);
#endif
	} else {
		ClipCursor(&SDL_bounds);
		if ( !(SDL_cursorstate & CURSOR_VISIBLE) ) {
		/*	RJR: March 28, 2000
			must be entering relative mode, get ready by
			moving mouse to	center of window ... */
			POINT pt;
			pt.x = (SDL_VideoSurface->w/2);
			pt.y = (SDL_VideoSurface->h/2);
			ClientToScreen(SDL_Window, &pt);
			SetCursorPos(pt.x, pt.y);
		}
#ifdef _WIN32_WCE
		AllKeys(1);
#endif
	}
	return(mode);
}

/* If 'info' is the right version, this function fills it and returns 1.
   Otherwise, in case of a version mismatch, it returns -1.
*/
int WIN_GetWMInfo(_THIS, SDL_SysWMinfo *info)
{
	if ( info->version.major <= SDL_MAJOR_VERSION ) {
		info->window = SDL_Window;
		if ( SDL_VERSIONNUM(info->version.major,
		                    info->version.minor,
		                    info->version.patch) >=
		     SDL_VERSIONNUM(1, 2, 5) ) {
#if SDL_VIDEO_OPENGL
			info->hglrc = GL_hrc;
#else
			info->hglrc = NULL;
#endif
		}
		return(1);
	} else {
		SDL_SetError("Application not compiled with SDL %d.%d\n",
					SDL_MAJOR_VERSION, SDL_MINOR_VERSION);
		return(-1);
	}
}
