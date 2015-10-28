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

#include <stdio.h>
#include <unistd.h>

#include "SDL_endian.h"
#include "../../events/SDL_events_c.h"
#include "SDL_x11image_c.h"

#ifndef NO_SHARED_MEMORY

/* Shared memory error handler routine */
static int shm_error;
static int (*X_handler)(Display *, XErrorEvent *) = NULL;
static int shm_errhandler(Display *d, XErrorEvent *e)
{
        if ( e->error_code == BadAccess ) {
        	shm_error = True;
        	return(0);
        } else
		return(X_handler(d,e));
}

static void try_mitshm(_THIS, SDL_Surface *screen)
{
	/* Dynamic X11 may not have SHM entry points on this box. */
	if ((use_mitshm) && (!SDL_X11_HAVE_SHM))
		use_mitshm = 0;

	if(!use_mitshm)
		return;
	shminfo.shmid = shmget(IPC_PRIVATE, screen->h*screen->pitch,
			       IPC_CREAT | 0777);
	if ( shminfo.shmid >= 0 ) {
		shminfo.shmaddr = (char *)shmat(shminfo.shmid, 0, 0);
		shminfo.readOnly = False;
		if ( shminfo.shmaddr != (char *)-1 ) {
			shm_error = False;
			X_handler = XSetErrorHandler(shm_errhandler);
			XShmAttach(SDL_Display, &shminfo);
			XSync(SDL_Display, True);
			XSetErrorHandler(X_handler);
			if ( shm_error )
				shmdt(shminfo.shmaddr);
		} else {
			shm_error = True;
		}
		shmctl(shminfo.shmid, IPC_RMID, NULL);
	} else {
		shm_error = True;
	}
	if ( shm_error )
		use_mitshm = 0;
	if ( use_mitshm )
		screen->pixels = shminfo.shmaddr;
}
#endif /* ! NO_SHARED_MEMORY */

/* Various screen update functions available */
static void X11_NormalUpdate(_THIS, int numrects, SDL_Rect *rects);
static void X11_MITSHMUpdate(_THIS, int numrects, SDL_Rect *rects);

int X11_SetupImage(_THIS, SDL_Surface *screen)
{
#ifndef NO_SHARED_MEMORY
	try_mitshm(this, screen);
	if(use_mitshm) {
		SDL_Ximage = XShmCreateImage(SDL_Display, SDL_Visual,
					     this->hidden->depth, ZPixmap,
					     shminfo.shmaddr, &shminfo, 
					     screen->w, screen->h);
		if(!SDL_Ximage) {
			XShmDetach(SDL_Display, &shminfo);
			XSync(SDL_Display, False);
			shmdt(shminfo.shmaddr);
			screen->pixels = NULL;
			goto error;
		}
		this->UpdateRects = X11_MITSHMUpdate;
	}
	if(!use_mitshm)
#endif /* not NO_SHARED_MEMORY */
	{
		screen->pixels = SDL_malloc(screen->h*screen->pitch);
		if ( screen->pixels == NULL ) {
			SDL_OutOfMemory();
			return -1;
		}
		SDL_Ximage = XCreateImage(SDL_Display, SDL_Visual,
					  this->hidden->depth, ZPixmap, 0,
					  (char *)screen->pixels, 
					  screen->w, screen->h,
					  32, 0);
		if ( SDL_Ximage == NULL )
			goto error;
		/* XPutImage will convert byte sex automatically */
		SDL_Ximage->byte_order = (SDL_BYTEORDER == SDL_BIG_ENDIAN)
			                 ? MSBFirst : LSBFirst;
		this->UpdateRects = X11_NormalUpdate;
	}
	screen->pitch = SDL_Ximage->bytes_per_line;
	return(0);

error:
	SDL_SetError("Couldn't create XImage");
	return 1;
}

void X11_DestroyImage(_THIS, SDL_Surface *screen)
{
	if ( SDL_Ximage ) {
		XDestroyImage(SDL_Ximage);
#ifndef NO_SHARED_MEMORY
		if ( use_mitshm ) {
			XShmDetach(SDL_Display, &shminfo);
			XSync(SDL_Display, False);
			shmdt(shminfo.shmaddr);
		}
#endif /* ! NO_SHARED_MEMORY */
		SDL_Ximage = NULL;
	}
	if ( screen ) {
		screen->pixels = NULL;
	}
}

/* Determine the number of CPUs in the system */
static int num_CPU(void)
{
       static int num_cpus = 0;

       if(!num_cpus) {
#if defined(__LINUX__)
           char line[BUFSIZ];
           FILE *pstat = fopen("/proc/stat", "r");
           if ( pstat ) {
               while ( fgets(line, sizeof(line), pstat) ) {
                   if (SDL_memcmp(line, "cpu", 3) == 0 && line[3] != ' ') {
                       ++num_cpus;
                   }
               }
               fclose(pstat);
           }
#elif defined(__IRIX__)
	   num_cpus = sysconf(_SC_NPROC_ONLN);
#elif defined(_SC_NPROCESSORS_ONLN)
	   /* number of processors online (SVR4.0MP compliant machines) */
           num_cpus = sysconf(_SC_NPROCESSORS_ONLN);
#elif defined(_SC_NPROCESSORS_CONF)
	   /* number of processors configured (SVR4.0MP compliant machines) */
           num_cpus = sysconf(_SC_NPROCESSORS_CONF);
#endif
           if ( num_cpus <= 0 ) {
               num_cpus = 1;
           }
       }
       return num_cpus;
}

int X11_ResizeImage(_THIS, SDL_Surface *screen, Uint32 flags)
{
	int retval;

	X11_DestroyImage(this, screen);
        if ( flags & SDL_OPENGL ) {  /* No image when using GL */
        	retval = 0;
        } else {
		retval = X11_SetupImage(this, screen);
		/* We support asynchronous blitting on the display */
		if ( flags & SDL_ASYNCBLIT ) {
			/* This is actually slower on single-CPU systems,
			   probably because of CPU contention between the
			   X server and the application.
			   Note: Is this still true with XFree86 4.0?
			*/
			if ( num_CPU() > 1 ) {
				screen->flags |= SDL_ASYNCBLIT;
			}
		}
	}
	return(retval);
}

/* We don't actually allow hardware surfaces other than the main one */
int X11_AllocHWSurface(_THIS, SDL_Surface *surface)
{
	return(-1);
}
void X11_FreeHWSurface(_THIS, SDL_Surface *surface)
{
	return;
}

int X11_LockHWSurface(_THIS, SDL_Surface *surface)
{
	if ( (surface == SDL_VideoSurface) && blit_queued ) {
		XSync(GFX_Display, False);
		blit_queued = 0;
	}
	return(0);
}
void X11_UnlockHWSurface(_THIS, SDL_Surface *surface)
{
	return;
}

int X11_FlipHWSurface(_THIS, SDL_Surface *surface)
{
	return(0);
}

static void X11_NormalUpdate(_THIS, int numrects, SDL_Rect *rects)
{
	int i;
	
	for (i = 0; i < numrects; ++i) {
		if ( rects[i].w == 0 || rects[i].h == 0 ) { /* Clipped? */
			continue;
		}
		XPutImage(GFX_Display, SDL_Window, SDL_GC, SDL_Ximage,
			  rects[i].x, rects[i].y,
			  rects[i].x, rects[i].y, rects[i].w, rects[i].h);
	}
	if ( SDL_VideoSurface->flags & SDL_ASYNCBLIT ) {
		XFlush(GFX_Display);
		blit_queued = 1;
	} else {
		XSync(GFX_Display, False);
	}
}

static void X11_MITSHMUpdate(_THIS, int numrects, SDL_Rect *rects)
{
#ifndef NO_SHARED_MEMORY
	int i;

	for ( i=0; i<numrects; ++i ) {
		if ( rects[i].w == 0 || rects[i].h == 0 ) { /* Clipped? */
			continue;
		}
		XShmPutImage(GFX_Display, SDL_Window, SDL_GC, SDL_Ximage,
				rects[i].x, rects[i].y,
				rects[i].x, rects[i].y, rects[i].w, rects[i].h,
									False);
	}
	if ( SDL_VideoSurface->flags & SDL_ASYNCBLIT ) {
		XFlush(GFX_Display);
		blit_queued = 1;
	} else {
		XSync(GFX_Display, False);
	}
#endif /* ! NO_SHARED_MEMORY */
}

/* There's a problem with the automatic refreshing of the display.
   Even though the XVideo code uses the GFX_Display to update the
   video memory, it appears that updating the window asynchronously
   from a different thread will cause "blackouts" of the window.
   This is a sort of a hacked workaround for the problem.
*/
static int enable_autorefresh = 1;

void X11_DisableAutoRefresh(_THIS)
{
	--enable_autorefresh;
}

void X11_EnableAutoRefresh(_THIS)
{
	++enable_autorefresh;
}

void X11_RefreshDisplay(_THIS)
{
	/* Don't refresh a display that doesn't have an image (like GL)
	   Instead, post an expose event so the application can refresh.
	 */
	if ( ! SDL_Ximage || (enable_autorefresh <= 0) ) {
		SDL_PrivateExpose();
		return;
	}
#ifndef NO_SHARED_MEMORY
	if ( this->UpdateRects == X11_MITSHMUpdate ) {
		XShmPutImage(SDL_Display, SDL_Window, SDL_GC, SDL_Ximage,
				0, 0, 0, 0, this->screen->w, this->screen->h,
				False);
	} else
#endif /* ! NO_SHARED_MEMORY */
	{
		XPutImage(SDL_Display, SDL_Window, SDL_GC, SDL_Ximage,
			  0, 0, 0, 0, this->screen->w, this->screen->h);
	}
	XSync(SDL_Display, False);
}

