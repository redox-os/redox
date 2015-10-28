/*
    SDL - Simple DirectMedia Layer
    Copyright (C) 1997-2012 Sam Lantinga

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
    Foundation, Inc., 59 Temple Place, Suite 330, Boston, MA  02111-1307  USA

    Sam Lantinga
    slouken@libsdl.org
*/
#include "SDL_config.h"

#define DEBUG_DYNAMIC_X11 0

#include "SDL_x11dyn.h"

#if DEBUG_DYNAMIC_X11
#include <stdio.h>
#endif

#ifdef SDL_VIDEO_DRIVER_X11_DYNAMIC

#include "SDL_name.h"
#include "SDL_loadso.h"

typedef struct
{
    void *lib;
    const char *libname;
} x11dynlib;

#ifndef SDL_VIDEO_DRIVER_X11_DYNAMIC
#define SDL_VIDEO_DRIVER_X11_DYNAMIC NULL
#endif
#ifndef SDL_VIDEO_DRIVER_X11_DYNAMIC_XEXT
#define SDL_VIDEO_DRIVER_X11_DYNAMIC_XEXT NULL
#endif
#ifndef SDL_VIDEO_DRIVER_X11_DYNAMIC_XRENDER
#define SDL_VIDEO_DRIVER_X11_DYNAMIC_XRENDER NULL
#endif
#ifndef SDL_VIDEO_DRIVER_X11_DYNAMIC_XRANDR
#define SDL_VIDEO_DRIVER_X11_DYNAMIC_XRANDR NULL
#endif

static x11dynlib x11libs[] =
{
    { NULL, SDL_VIDEO_DRIVER_X11_DYNAMIC },
    { NULL, SDL_VIDEO_DRIVER_X11_DYNAMIC_XEXT },
    { NULL, SDL_VIDEO_DRIVER_X11_DYNAMIC_XRENDER },
    { NULL, SDL_VIDEO_DRIVER_X11_DYNAMIC_XRANDR },
};

static void *X11_GetSym(const char *fnname, int *rc)
{
	void *fn = NULL;
	int i;
	for (i = 0; i < SDL_TABLESIZE(x11libs); i++) {
		if (x11libs[i].lib != NULL)
		{
			fn = SDL_LoadFunction(x11libs[i].lib, fnname);
			if (fn != NULL)
				break;
		}
	}

	#if DEBUG_DYNAMIC_X11
	if (fn != NULL)
		printf("X11: Found '%s' in %s (%p)\n", fnname, x11libs[i].libname, *fn);
	else
		printf("X11: Symbol '%s' NOT FOUND!\n", fnname);
	#endif

	if (fn == NULL)
		*rc = 0;  /* kill this module. */

	return fn;
}


/* Define all the function pointers and wrappers... */
#define SDL_X11_MODULE(modname)
#define SDL_X11_SYM(rc,fn,params,args,ret) \
	static rc (*p##fn) params = NULL; \
	rc fn params { ret p##fn args ; }
#include "SDL_x11sym.h"
#undef SDL_X11_MODULE
#undef SDL_X11_SYM
#endif  /* SDL_VIDEO_DRIVER_X11_DYNAMIC */

/* Annoying varargs entry point... */
#ifdef X_HAVE_UTF8_STRING
XIC (*pXCreateIC)(XIM,...) = NULL;
char *(*pXGetICValues)(XIC, ...) = NULL;
#endif

/* These SDL_X11_HAVE_* flags are here whether you have dynamic X11 or not. */
#define SDL_X11_MODULE(modname) int SDL_X11_HAVE_##modname = 1;
#define SDL_X11_SYM(rc,fn,params,args,ret)
#include "SDL_x11sym.h"
#undef SDL_X11_MODULE
#undef SDL_X11_SYM


static void *SDL_XGetRequest_workaround(Display* dpy, CARD8 type, size_t len)
{
	xReq *req;
	WORD64ALIGN
	if (dpy->bufptr + len > dpy->bufmax)
		_XFlush(dpy);
	dpy->last_req = dpy->bufptr;
	req = (xReq*)dpy->bufptr;
	req->reqType = type;
	req->length = len / 4;
	dpy->bufptr += len;
	dpy->request++;
	return req;
}

static int x11_load_refcount = 0;

void SDL_X11_UnloadSymbols(void)
{
	#ifdef SDL_VIDEO_DRIVER_X11_DYNAMIC
	/* Don't actually unload if more than one module is using the libs... */
	if (x11_load_refcount > 0) {
		if (--x11_load_refcount == 0) {
			int i;

			/* set all the function pointers to NULL. */
			#define SDL_X11_MODULE(modname) SDL_X11_HAVE_##modname = 1;
			#define SDL_X11_SYM(rc,fn,params,args,ret) p##fn = NULL;
			#include "SDL_x11sym.h"
			#undef SDL_X11_MODULE
			#undef SDL_X11_SYM

			#ifdef X_HAVE_UTF8_STRING
			pXCreateIC = NULL;
			pXGetICValues = NULL;
			#endif

			for (i = 0; i < SDL_TABLESIZE(x11libs); i++) {
				if (x11libs[i].lib != NULL) {
					SDL_UnloadObject(x11libs[i].lib);
					x11libs[i].lib = NULL;
				}
			}
		}
	}
	#endif
}

/* returns non-zero if all needed symbols were loaded. */
int SDL_X11_LoadSymbols(void)
{
	int rc = 1;  /* always succeed if not using Dynamic X11 stuff. */

	#ifdef SDL_VIDEO_DRIVER_X11_DYNAMIC
	/* deal with multiple modules (dga, x11, etc) needing these symbols... */
	if (x11_load_refcount++ == 0) {
		int i;
		int *thismod = NULL;
		for (i = 0; i < SDL_TABLESIZE(x11libs); i++) {
			if (x11libs[i].libname != NULL) {
				x11libs[i].lib = SDL_LoadObject(x11libs[i].libname);
			}
		}
		#define SDL_X11_MODULE(modname) thismod = &SDL_X11_HAVE_##modname;
		#define SDL_X11_SYM(rc,fn,params,args,ret) \
            p##fn = (rc(*)params) X11_GetSym(#fn, thismod);
		#include "SDL_x11sym.h"
		#undef SDL_X11_MODULE
		#undef SDL_X11_SYM

		#ifdef X_HAVE_UTF8_STRING
		pXCreateIC = (XIC(*)(XIM,...)) X11_GetSym("XCreateIC",
		                                          &SDL_X11_HAVE_UTF8);
		pXGetICValues = (char * (*)(XIC,...)) X11_GetSym("XGetICValues",
		                                                 &SDL_X11_HAVE_UTF8);
		#endif

		/*
		 * In case we're built with newer Xlib headers, we need to make sure
		 *  that _XGetRequest() is available, even on older systems.
		 *  Otherwise, various Xlib macros we use will call a NULL pointer.
		 */
		if (!SDL_X11_HAVE_XGETREQUEST) {
			p_XGetRequest = SDL_XGetRequest_workaround;
		}

		if (SDL_X11_HAVE_BASEXLIB) {  /* all required symbols loaded. */
			SDL_ClearError();
		} else {
			SDL_X11_UnloadSymbols();  /* in case something got loaded... */
			rc = 0;
		}
	}
	#else
		#if DEBUG_DYNAMIC_X11
		printf("X11: No dynamic X11 support in this build of SDL.\n");
		#endif
		#ifdef X_HAVE_UTF8_STRING
		pXCreateIC = XCreateIC;
		pXGetICValues = XGetICValues;
		#endif
	#endif

	return rc;
}

/* end of SDL_x11dyn.c ... */

