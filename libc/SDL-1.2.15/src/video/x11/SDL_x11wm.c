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

#include <X11/Xlib.h>
#include <X11/Xutil.h>

#include "SDL_version.h"
#include "SDL_timer.h"
#include "SDL_video.h"
#include "SDL_syswm.h"
#include "../SDL_pixels_c.h"
#include "../../events/SDL_events_c.h"
#include "SDL_x11modes_c.h"
#include "SDL_x11wm_c.h"

static Uint8 reverse_byte(Uint8 x)
{
	x = (x & 0xaa) >> 1 | (x & 0x55) << 1;
	x = (x & 0xcc) >> 2 | (x & 0x33) << 2;
	x = (x & 0xf0) >> 4 | (x & 0x0f) << 4;
	return x;
}

void X11_SetIcon(_THIS, SDL_Surface *icon, Uint8 *mask)
{
	SDL_Surface *sicon;
	XWMHints *wmhints;
	XImage *icon_image;
	Pixmap icon_pixmap;
	Pixmap mask_pixmap;
	Window icon_window = None;
	GC gc;
	XGCValues GCvalues;
	int i, dbpp;
	SDL_Rect bounds;
	Uint8 *LSBmask;
	Visual *dvis;
	char *p;
	int masksize;

	SDL_Lock_EventThread();

	/* The icon must use the default visual, depth and colormap of the
	   screen, so it might need a conversion */
	dvis = DefaultVisual(SDL_Display, SDL_Screen);
	dbpp = DefaultDepth(SDL_Display, SDL_Screen);
	for(i = 0; i < this->hidden->nvisuals; i++) {
		if(this->hidden->visuals[i].visual == dvis) {
			dbpp = this->hidden->visuals[i].bpp;
			break;
		}
	}

	/* The Visual struct is supposed to be opaque but we cheat a little */
	sicon = SDL_CreateRGBSurface(SDL_SWSURFACE, icon->w, icon->h,
				     dbpp,
				     dvis->red_mask, dvis->green_mask,
				     dvis->blue_mask, 0);
	if ( sicon == NULL )
		goto done;

	if(dbpp == 8) {
		/* Default visual is 8bit; we need to allocate colours from
		   the default colormap */
		SDL_Color want[256], got[256];
		int nwant;
		Colormap dcmap;
		int missing;
		dcmap = DefaultColormap(SDL_Display, SDL_Screen);
		if(icon->format->palette) {
			/* The icon has a palette as well - we just have to
			   find those colours */
			nwant = icon->format->palette->ncolors;
			SDL_memcpy(want, icon->format->palette->colors,
			       nwant * sizeof want[0]);
		} else {
			/* try the standard 6x6x6 cube for lack of better
			   ideas */
			int r, g, b, i;
			for(r = i = 0; r < 256; r += 0x33)
				for(g = 0; g < 256; g += 0x33)
					for(b = 0; b < 256; b += 0x33, i++) {
						want[i].r = r;
						want[i].g = g;
						want[i].b = b;
					}
			nwant = 216;
		}
		if(SDL_iconcolors) {
			/* free already allocated colours first */
			unsigned long freelist[512];
			int nfree = 0;
			for(i = 0; i < 256; i++) {
				while(SDL_iconcolors[i]) {
					freelist[nfree++] = i;
					SDL_iconcolors[i]--;
				}
			}
			XFreeColors(GFX_Display, dcmap, freelist, nfree, 0);
		}
		if(!SDL_iconcolors)
			SDL_iconcolors = SDL_malloc(256 * sizeof *SDL_iconcolors);
		SDL_memset(SDL_iconcolors, 0, 256 * sizeof *SDL_iconcolors);

		/* try to allocate the colours */
		SDL_memset(got, 0, sizeof got);
		missing = 0;
		for(i = 0; i < nwant; i++) {
			XColor c;
			c.red = want[i].r << 8;
			c.green = want[i].g << 8;
			c.blue = want[i].b << 8;
			c.flags = DoRed | DoGreen | DoBlue;
			if(XAllocColor(GFX_Display, dcmap, &c)) {
				/* got the colour */
				SDL_iconcolors[c.pixel]++;
				got[c.pixel] = want[i];
			} else {
				missing = 1;
			}
		}
		if(missing) {
			/* Some colours were apparently missing, so we just
			   allocate all the rest as well */
			XColor cols[256];
			for(i = 0; i < 256; i++)
				cols[i].pixel = i;
			XQueryColors(GFX_Display, dcmap, cols, 256);
			for(i = 0; i < 256; i++) {
				got[i].r = cols[i].red >> 8;
				got[i].g = cols[i].green >> 8;
				got[i].b = cols[i].blue >> 8;
				if(!SDL_iconcolors[i]) {
					if(XAllocColor(GFX_Display, dcmap,
							cols + i)) {
						SDL_iconcolors[i] = 1;
					} else {
						/* index not available */
						got[i].r = 0;
						got[i].g = 0;
						got[i].b = 0;
					}
				}
			}
		}

		SDL_SetColors(sicon, got, 0, 256);
	}

	bounds.x = 0;
	bounds.y = 0;
	bounds.w = icon->w;
	bounds.h = icon->h;
	if ( SDL_LowerBlit(icon, &bounds, sicon, &bounds) < 0 )
		goto done;

	/* We need the mask as given, except in LSBfirst format instead of
	   MSBfirst. Reverse the bits in each byte. */
	masksize = ((sicon->w + 7) >> 3) * sicon->h;
	LSBmask = SDL_malloc(masksize);
	if ( LSBmask == NULL ) {
		goto done;
	}
	SDL_memset(LSBmask, 0, masksize);
	for(i = 0; i < masksize; i++)
		LSBmask[i] = reverse_byte(mask[i]);
	mask_pixmap = XCreatePixmapFromBitmapData(SDL_Display, WMwindow,
						  (char *)LSBmask,
						  sicon->w, sicon->h,
						  1L, 0L, 1);

	/* Transfer the image to an X11 pixmap */
	icon_image = XCreateImage(SDL_Display,
				  DefaultVisual(SDL_Display, SDL_Screen),
				  DefaultDepth(SDL_Display, SDL_Screen),
				  ZPixmap, 0, sicon->pixels,
				  sicon->w, sicon->h,
				  32, 0);
	icon_image->byte_order = (SDL_BYTEORDER == SDL_BIG_ENDIAN)
		                 ? MSBFirst : LSBFirst;
	icon_pixmap = XCreatePixmap(SDL_Display, SDL_Root, sicon->w, sicon->h,
				    DefaultDepth(SDL_Display, SDL_Screen));
	gc = XCreateGC(SDL_Display, icon_pixmap, 0, &GCvalues);
	XPutImage(SDL_Display, icon_pixmap, gc, icon_image,
		  0, 0, 0, 0, sicon->w, sicon->h);
	XFreeGC(SDL_Display, gc);
	XDestroyImage(icon_image);
	SDL_free(LSBmask);
	sicon->pixels = NULL;

	/* Some buggy window managers (some versions of Enlightenment, it
	   seems) need an icon window *and* icon pixmap to work properly, while
	   it screws up others. The default is only to use a pixmap. */
	p = SDL_getenv("SDL_VIDEO_X11_ICONWIN");
	if(p && *p) {
		icon_window = XCreateSimpleWindow(SDL_Display, SDL_Root,
						  0, 0, sicon->w, sicon->h, 0,
						  CopyFromParent,
						  CopyFromParent);
		XSetWindowBackgroundPixmap(SDL_Display, icon_window,
					   icon_pixmap);
		XClearWindow(SDL_Display, icon_window);
	}

	/* Set the window icon to the icon pixmap (and icon window) */
	wmhints = XAllocWMHints();
	wmhints->flags = (IconPixmapHint | IconMaskHint | InputHint);
	wmhints->icon_pixmap = icon_pixmap;
	wmhints->icon_mask = mask_pixmap;
	wmhints->input = True;
	if(icon_window != None) {
		wmhints->flags |= IconWindowHint;
		wmhints->icon_window = icon_window;
	}
	XSetWMHints(SDL_Display, WMwindow, wmhints);
	XFree(wmhints);
	XSync(SDL_Display, False);

  done:
	SDL_Unlock_EventThread();
	SDL_FreeSurface(sicon);
}

void X11_SetCaptionNoLock(_THIS, const char *title, const char *icon)
{
	XTextProperty titleprop, iconprop;
	Status status;

#ifdef X_HAVE_UTF8_STRING
	Atom _NET_WM_NAME = 0;
	Atom _NET_WM_ICON_NAME = 0;

	/* Look up some useful Atoms */
	if (SDL_X11_HAVE_UTF8) {
		_NET_WM_NAME = XInternAtom(SDL_Display, "_NET_WM_NAME", False);
		_NET_WM_ICON_NAME = XInternAtom(SDL_Display, "_NET_WM_ICON_NAME", False);
	}
#endif

	if ( title != NULL ) {
		char *title_locale = SDL_iconv_utf8_locale(title);
		if ( !title_locale ) {
			SDL_OutOfMemory();
			return;
		}
		status = XStringListToTextProperty(&title_locale, 1, &titleprop);
		SDL_free(title_locale);
		if ( status ) {
			XSetTextProperty(SDL_Display, WMwindow, &titleprop, XA_WM_NAME);
			XFree(titleprop.value);
		}
#ifdef X_HAVE_UTF8_STRING
		if (SDL_X11_HAVE_UTF8) {
			status = Xutf8TextListToTextProperty(SDL_Display,
					(char **)&title, 1, XUTF8StringStyle, &titleprop);
			if ( status == Success ) {
				XSetTextProperty(SDL_Display, WMwindow, &titleprop, _NET_WM_NAME);
				XFree(titleprop.value);
			}
		}
#endif
	}
	if ( icon != NULL ) {
		char *icon_locale = SDL_iconv_utf8_locale(icon);
		if ( !icon_locale ) {
			SDL_OutOfMemory();
			return;
		}
		status = XStringListToTextProperty(&icon_locale, 1, &iconprop);
		SDL_free(icon_locale);
		if ( status ) {
			XSetTextProperty(SDL_Display, WMwindow, &iconprop, XA_WM_ICON_NAME);
			XFree(iconprop.value);
		}
#ifdef X_HAVE_UTF8_STRING
		if (SDL_X11_HAVE_UTF8) {
			status = Xutf8TextListToTextProperty(SDL_Display,
					(char **)&icon, 1, XUTF8StringStyle, &iconprop);
			if ( status == Success ) {
				XSetTextProperty(SDL_Display, WMwindow, &iconprop, _NET_WM_ICON_NAME);
				XFree(iconprop.value);
			}
		}
#endif
	}
	XSync(SDL_Display, False);
}

void X11_SetCaption(_THIS, const char *title, const char *icon)
{
	SDL_Lock_EventThread();
	X11_SetCaptionNoLock(this, title, icon);
	SDL_Unlock_EventThread();
}

/* Iconify the window */
int X11_IconifyWindow(_THIS)
{
	int result;

	SDL_Lock_EventThread();
	result = XIconifyWindow(SDL_Display, WMwindow, SDL_Screen);
	XSync(SDL_Display, False);
	SDL_Unlock_EventThread();
	return(result);
}

SDL_GrabMode X11_GrabInputNoLock(_THIS, SDL_GrabMode mode)
{
	int result;

	if ( this->screen == NULL || SDL_Display == NULL ) {
		return(SDL_GRAB_OFF);
	}
	if ( ! SDL_Window ) {
		return(mode);	/* Will be set later on mode switch */
	}
	if ( mode == SDL_GRAB_OFF ) {
		XUngrabPointer(SDL_Display, CurrentTime);
		XUngrabKeyboard(SDL_Display, CurrentTime);
	} else {
		if ( this->screen->flags & SDL_FULLSCREEN ) {
			/* Unbind the mouse from the fullscreen window */
			XUngrabPointer(SDL_Display, CurrentTime);
		}
		/* Try to grab the mouse */
#if 0 /* We'll wait here until we actually grab, otherwise behavior undefined */
		for ( numtries = 0; numtries < 10; ++numtries ) {
#else
		for ( ; ; ) {
#endif
			result = XGrabPointer(SDL_Display, SDL_Window, True, 0,
						GrabModeAsync, GrabModeAsync,
						SDL_Window, None, CurrentTime);
			if ( result == GrabSuccess ) {
				break;
			}
			SDL_Delay(100);
		}
		if ( result != GrabSuccess ) {
			/* Uh, oh, what do we do here? */ ;
		}
		/* Now grab the keyboard */
		XGrabKeyboard(SDL_Display, WMwindow, True,
				GrabModeAsync, GrabModeAsync, CurrentTime);

		/* Raise the window if we grab the mouse */
		if ( !(this->screen->flags & SDL_FULLSCREEN) )
			XRaiseWindow(SDL_Display, WMwindow);

		/* Make sure we register input focus */
		SDL_PrivateAppActive(1, SDL_APPINPUTFOCUS);
		/* Since we grabbed the pointer, we have mouse focus, too. */
		SDL_PrivateAppActive(1, SDL_APPMOUSEFOCUS);
	}
	XSync(SDL_Display, False);

	return(mode);
}

SDL_GrabMode X11_GrabInput(_THIS, SDL_GrabMode mode)
{
	SDL_Lock_EventThread();
	mode = X11_GrabInputNoLock(this, mode);
	SDL_Unlock_EventThread();

	return(mode);
}

/* If 'info' is the right version, this function fills it and returns 1.
   Otherwise, in case of a version mismatch, it returns -1.
*/
static void lock_display(void)
{
	SDL_Lock_EventThread();
}
static void unlock_display(void)
{
	/* Make sure any X11 transactions are completed */
	SDL_VideoDevice *this = current_video;
	XSync(SDL_Display, False);
	SDL_Unlock_EventThread();
}

#include <stdio.h>
int X11_GetWMInfo(_THIS, SDL_SysWMinfo *info)
{
	if ( info->version.major <= SDL_MAJOR_VERSION ) {
		info->subsystem = SDL_SYSWM_X11;
		info->info.x11.display = SDL_Display;
		info->info.x11.window = SDL_Window;
		if ( SDL_VERSIONNUM(info->version.major,
		                    info->version.minor,
		                    info->version.patch) >= 1002 ) {
			info->info.x11.fswindow = FSwindow;
			info->info.x11.wmwindow = WMwindow;
		}


		if ( SDL_VERSIONNUM(info->version.major,
		                    info->version.minor,
		                    info->version.patch) >= 1212 ) {
			info->info.x11.gfxdisplay = GFX_Display;
		}

		info->info.x11.lock_func = lock_display;
		info->info.x11.unlock_func = unlock_display;
		return(1);
	} else {
		SDL_SetError("Application not compiled with SDL %d.%d\n",
					SDL_MAJOR_VERSION, SDL_MINOR_VERSION);
		return(-1);
	}
}
