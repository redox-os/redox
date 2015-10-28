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

/* Atari OSMesa.ldg implementation of SDL OpenGL support */

/*--- Includes ---*/

#if SDL_VIDEO_OPENGL
#include <GL/osmesa.h>
#endif

#include <mint/osbind.h>

#include "SDL_endian.h"
#include "SDL_video.h"
#include "SDL_atarigl_c.h"
#if SDL_VIDEO_OPENGL_OSMESA_DYNAMIC
#include "SDL_loadso.h"
#endif

/*--- Defines ---*/

#define PATH_OSMESA_LDG	"osmesa.ldg"
#define PATH_MESAGL_LDG	"mesa_gl.ldg"
#define PATH_TINYGL_LDG	"tiny_gl.ldg"

#define VDI_RGB	0xf

/*--- Functions prototypes ---*/

#if SDL_VIDEO_OPENGL
static void SDL_AtariGL_UnloadLibrary(_THIS);

static void CopyShadowNull(_THIS, SDL_Surface *surface);
static void CopyShadowDirect(_THIS, SDL_Surface *surface);
static void CopyShadowRGBTo555(_THIS, SDL_Surface *surface);
static void CopyShadowRGBTo565(_THIS, SDL_Surface *surface);
static void CopyShadowRGBSwap(_THIS, SDL_Surface *surface);
static void CopyShadowRGBToARGB(_THIS, SDL_Surface *surface);
static void CopyShadowRGBToABGR(_THIS, SDL_Surface *surface);
static void CopyShadowRGBToBGRA(_THIS, SDL_Surface *surface);
static void CopyShadowRGBToRGBA(_THIS, SDL_Surface *surface);
static void CopyShadow8888To555(_THIS, SDL_Surface *surface);
static void CopyShadow8888To565(_THIS, SDL_Surface *surface);

static void ConvertNull(_THIS, SDL_Surface *surface);
static void Convert565To555be(_THIS, SDL_Surface *surface);
static void Convert565To555le(_THIS, SDL_Surface *surface);
static void Convert565le(_THIS, SDL_Surface *surface);
static void ConvertBGRAToABGR(_THIS, SDL_Surface *surface);

static int InitNew(_THIS, SDL_Surface *current);
static int InitOld(_THIS, SDL_Surface *current);
#endif

/*--- Public functions ---*/

int SDL_AtariGL_Init(_THIS, SDL_Surface *current)
{
#if SDL_VIDEO_OPENGL
	if (gl_oldmesa) {
		gl_active = InitOld(this, current);		
	} else {
		gl_active = InitNew(this, current);		
	}
#endif

	return (gl_active);
}

void SDL_AtariGL_Quit(_THIS, SDL_bool unload)
{
#if SDL_VIDEO_OPENGL
	if (gl_oldmesa) {
		/* Old mesa implementations */
		if (this->gl_data->OSMesaDestroyLDG) {
			this->gl_data->OSMesaDestroyLDG();
		}
		if (gl_shadow) {
			Mfree(gl_shadow);
			gl_shadow = NULL;
		}
	} else {
		/* New mesa implementation */
		if (gl_ctx) {
			if (this->gl_data->OSMesaDestroyContext) {
				this->gl_data->OSMesaDestroyContext(gl_ctx);
			}
			gl_ctx = NULL;
		}
	}

	if (unload) {
		SDL_AtariGL_UnloadLibrary(this);
	}

#endif /* SDL_VIDEO_OPENGL */
	gl_active = 0;
}

int SDL_AtariGL_LoadLibrary(_THIS, const char *path)
{
#if SDL_VIDEO_OPENGL

#if SDL_VIDEO_OPENGL_OSMESA_DYNAMIC
	void *handle;
	SDL_bool cancel_load;

	if (gl_active) {
		SDL_SetError("OpenGL context already created");
		return -1;
	}

	/* Unload previous driver */
	SDL_AtariGL_UnloadLibrary(this);

	/* Load library given by path */
	handle = SDL_LoadObject(path);
	if (handle == NULL) {
		/* Try to load another one */
		path = SDL_getenv("SDL_VIDEO_GL_DRIVER");
		if ( path != NULL ) {
			handle = SDL_LoadObject(path);
		}

		/* If it does not work, try some other */
		if (handle == NULL) {
			path = PATH_OSMESA_LDG;
			handle = SDL_LoadObject(path);
		}

		if (handle == NULL) {
			path = PATH_MESAGL_LDG;
			handle = SDL_LoadObject(path);
		}

		if (handle == NULL) {
			path = PATH_TINYGL_LDG;
			handle = SDL_LoadObject(path);
		}
	}

	if (handle == NULL) {
		SDL_SetError("Could not load OpenGL library");
		return -1;
	}

	this->gl_data->glGetIntegerv = SDL_LoadFunction(handle, "glGetIntegerv");
	this->gl_data->glFinish = SDL_LoadFunction(handle, "glFinish");
	this->gl_data->glFlush = SDL_LoadFunction(handle, "glFlush");

	cancel_load = SDL_FALSE;
	if (this->gl_data->glGetIntegerv == NULL) {
		cancel_load = SDL_TRUE;
	} else {
		/* We need either glFinish (OSMesa) or glFlush (TinyGL) */
		if ((this->gl_data->glFinish == NULL) &&
			(this->gl_data->glFlush == NULL)) {
				cancel_load = SDL_TRUE;
		}
	}
	if (cancel_load) {
		SDL_SetError("Could not retrieve OpenGL functions");
		SDL_UnloadObject(handle);
		/* Restore pointers to static library */
		SDL_AtariGL_InitPointers(this);
		return -1;
	}

	/* Load functions pointers (osmesa.ldg) */
	this->gl_data->OSMesaCreateContextExt = SDL_LoadFunction(handle, "OSMesaCreateContextExt");
	this->gl_data->OSMesaDestroyContext = SDL_LoadFunction(handle, "OSMesaDestroyContext");
	this->gl_data->OSMesaMakeCurrent = SDL_LoadFunction(handle, "OSMesaMakeCurrent");
	this->gl_data->OSMesaPixelStore = SDL_LoadFunction(handle, "OSMesaPixelStore");
	this->gl_data->OSMesaGetProcAddress = SDL_LoadFunction(handle, "OSMesaGetProcAddress");

	/* Load old functions pointers (mesa_gl.ldg, tiny_gl.ldg) */
	this->gl_data->OSMesaCreateLDG = SDL_LoadFunction(handle, "OSMesaCreateLDG");
	this->gl_data->OSMesaDestroyLDG = SDL_LoadFunction(handle, "OSMesaDestroyLDG");

	gl_oldmesa = 0;

	if ( (this->gl_data->OSMesaCreateContextExt == NULL) || 
	     (this->gl_data->OSMesaDestroyContext == NULL) ||
	     (this->gl_data->OSMesaMakeCurrent == NULL) ||
	     (this->gl_data->OSMesaPixelStore == NULL) ||
	     (this->gl_data->OSMesaGetProcAddress == NULL)) {
		/* Hum, maybe old library ? */
		if ( (this->gl_data->OSMesaCreateLDG == NULL) || 
		     (this->gl_data->OSMesaDestroyLDG == NULL)) {
			SDL_SetError("Could not retrieve OSMesa functions");
			SDL_UnloadObject(handle);
			/* Restore pointers to static library */
			SDL_AtariGL_InitPointers(this);
			return -1;
		} else {
			gl_oldmesa = 1;
		}
	}

	this->gl_config.dll_handle = handle;
	if ( path ) {
		SDL_strlcpy(this->gl_config.driver_path, path,
			SDL_arraysize(this->gl_config.driver_path));
	} else {
		*this->gl_config.driver_path = '\0';
	}

#endif
	this->gl_config.driver_loaded = 1;

	return 0;
#else
	return -1;
#endif
}

void *SDL_AtariGL_GetProcAddress(_THIS, const char *proc)
{
	void *func = NULL;
#if SDL_VIDEO_OPENGL

	if (this->gl_config.dll_handle) {
		func = SDL_LoadFunction(this->gl_config.dll_handle, (void *)proc);
	} else if (this->gl_data->OSMesaGetProcAddress) {
		func = this->gl_data->OSMesaGetProcAddress(proc);
	}

#endif
	return func;
}

int SDL_AtariGL_GetAttribute(_THIS, SDL_GLattr attrib, int* value)
{
#if SDL_VIDEO_OPENGL
	GLenum mesa_attrib;
	SDL_Surface *surface;

	if (!gl_active) {
		return -1;
	}

	switch(attrib) {
		case SDL_GL_RED_SIZE:
			mesa_attrib = GL_RED_BITS;
			break;
		case SDL_GL_GREEN_SIZE:
			mesa_attrib = GL_GREEN_BITS;
			break;
		case SDL_GL_BLUE_SIZE:
			mesa_attrib = GL_BLUE_BITS;
			break;
		case SDL_GL_ALPHA_SIZE:
			mesa_attrib = GL_ALPHA_BITS;
			break;
		case SDL_GL_DOUBLEBUFFER:
			surface = this->screen;
			*value = ((surface->flags & SDL_DOUBLEBUF)==SDL_DOUBLEBUF);
			return 0;
		case SDL_GL_DEPTH_SIZE:
			mesa_attrib = GL_DEPTH_BITS;
			break;
		case SDL_GL_STENCIL_SIZE:
			mesa_attrib = GL_STENCIL_BITS;
			break;
		case SDL_GL_ACCUM_RED_SIZE:
			mesa_attrib = GL_ACCUM_RED_BITS;
			break;
		case SDL_GL_ACCUM_GREEN_SIZE:
			mesa_attrib = GL_ACCUM_GREEN_BITS;
			break;
		case SDL_GL_ACCUM_BLUE_SIZE:
			mesa_attrib = GL_ACCUM_BLUE_BITS;
			break;
		case SDL_GL_ACCUM_ALPHA_SIZE:
			mesa_attrib = GL_ACCUM_ALPHA_BITS;
			break;
		default :
			return -1;
	}

	this->gl_data->glGetIntegerv(mesa_attrib, value);
	return 0;
#else
	return -1;
#endif
}

int SDL_AtariGL_MakeCurrent(_THIS)
{
#if SDL_VIDEO_OPENGL
	SDL_Surface *surface;
	GLenum type;

	if (gl_oldmesa && gl_active) {
		return 0;
	}

	if (this->gl_config.dll_handle) {
		if ((this->gl_data->OSMesaMakeCurrent == NULL) ||
			(this->gl_data->OSMesaPixelStore == NULL)) {
			return -1;
		}
	}

	if (!gl_active) {
		SDL_SetError("Invalid OpenGL context");
		return -1;
	}

	surface = this->screen;
	
	if ((surface->format->BitsPerPixel == 15) || (surface->format->BitsPerPixel == 16)) {
		type = GL_UNSIGNED_SHORT_5_6_5;
	} else {
		type = GL_UNSIGNED_BYTE;
	}

	if (!(this->gl_data->OSMesaMakeCurrent(gl_ctx, surface->pixels, type, surface->w, surface->h))) {
		SDL_SetError("Can not make OpenGL context current");
		return -1;
	}

	/* OSMesa draws upside down */
	this->gl_data->OSMesaPixelStore(OSMESA_Y_UP, 0);

	return 0;
#else
	return -1;
#endif
}

void SDL_AtariGL_SwapBuffers(_THIS)
{
#if SDL_VIDEO_OPENGL
	if (gl_active) {
		if (this->gl_config.dll_handle) {
			if (this->gl_data->glFinish) {
				this->gl_data->glFinish();
			} else if (this->gl_data->glFlush) {
				this->gl_data->glFlush();
			}
		} else {
			this->gl_data->glFinish();
		}
		gl_copyshadow(this, this->screen);
		gl_convert(this, this->screen);
	}
#endif
}

void SDL_AtariGL_InitPointers(_THIS)
{
#if SDL_VIDEO_OPENGL
	this->gl_data->OSMesaCreateContextExt = OSMesaCreateContextExt;
	this->gl_data->OSMesaDestroyContext = OSMesaDestroyContext;
	this->gl_data->OSMesaMakeCurrent = OSMesaMakeCurrent;
	this->gl_data->OSMesaPixelStore = OSMesaPixelStore;
	this->gl_data->OSMesaGetProcAddress = OSMesaGetProcAddress;

	this->gl_data->glGetIntegerv = glGetIntegerv;
	this->gl_data->glFinish = glFinish;
	this->gl_data->glFlush = glFlush;

	this->gl_data->OSMesaCreateLDG = NULL;
	this->gl_data->OSMesaDestroyLDG = NULL;
#endif
}

/*--- Private functions ---*/

#if SDL_VIDEO_OPENGL
static void SDL_AtariGL_UnloadLibrary(_THIS)
{
	if (this->gl_config.dll_handle) {
		SDL_UnloadObject(this->gl_config.dll_handle);
		this->gl_config.dll_handle = NULL;

		/* Restore pointers to static library */
		SDL_AtariGL_InitPointers(this);
	}
}

/*--- Creation of an OpenGL context using new/old functions ---*/

static int InitNew(_THIS, SDL_Surface *current)
{
	GLenum osmesa_format;
	SDL_PixelFormat *pixel_format;
	Uint32	redmask;
	int recreatecontext;
	GLint newaccumsize;

	if (this->gl_config.dll_handle) {
		if (this->gl_data->OSMesaCreateContextExt == NULL) {
			return 0;
		}
	}

	/* Init OpenGL context using OSMesa */
	gl_convert = ConvertNull;
	gl_copyshadow = CopyShadowNull;
	gl_upsidedown = SDL_FALSE;

	pixel_format = current->format;
	redmask = pixel_format->Rmask;
	switch (pixel_format->BitsPerPixel) {
		case 15:
			/* 1555, big and little endian, unsupported */
			gl_pixelsize = 2;
			osmesa_format = OSMESA_RGB_565;
			if (redmask == 31<<10) {
				gl_convert = Convert565To555be;
			} else {
				gl_convert = Convert565To555le;
			}
			break;
		case 16:
			gl_pixelsize = 2;
			if (redmask == 31<<11) {
				osmesa_format = OSMESA_RGB_565;
			} else {
				/* 565, little endian, unsupported */
				osmesa_format = OSMESA_RGB_565;
				gl_convert = Convert565le;
			}
			break;
		case 24:
			gl_pixelsize = 3;
			if (redmask == 255<<16) {
				osmesa_format = OSMESA_RGB;
			} else {
				osmesa_format = OSMESA_BGR;
			}
			break;
		case 32:
			gl_pixelsize = 4;
			if (redmask == 255<<16) {
				osmesa_format = OSMESA_ARGB;
			} else if (redmask == 255<<8) {
				osmesa_format = OSMESA_BGRA;
			} else if (redmask == 255<<24) {
				osmesa_format = OSMESA_RGBA;
			} else {
				/* ABGR format unsupported */
				osmesa_format = OSMESA_BGRA;
				gl_convert = ConvertBGRAToABGR;
			}
			break;
		default:
			gl_pixelsize = 1;
			osmesa_format = OSMESA_COLOR_INDEX;
			break;
	}

	/* Try to keep current context if possible */
	newaccumsize =
		this->gl_config.accum_red_size +
		this->gl_config.accum_green_size +
		this->gl_config.accum_blue_size +
		this->gl_config.accum_alpha_size;
	recreatecontext=1;
	if (gl_ctx &&
		(gl_curformat == osmesa_format) &&
		(gl_curdepth == this->gl_config.depth_size) &&
		(gl_curstencil == this->gl_config.stencil_size) &&
		(gl_curaccum == newaccumsize)) {
		recreatecontext = 0;
	}
	if (recreatecontext) {
		SDL_AtariGL_Quit(this, SDL_FALSE);

		gl_ctx = this->gl_data->OSMesaCreateContextExt(
			osmesa_format, this->gl_config.depth_size,
			this->gl_config.stencil_size, newaccumsize, NULL );

		if (gl_ctx) {
			gl_curformat = osmesa_format;
			gl_curdepth = this->gl_config.depth_size;
			gl_curstencil = this->gl_config.stencil_size;
			gl_curaccum = newaccumsize;
		} else {
			gl_curformat = 0;
			gl_curdepth = 0;
			gl_curstencil = 0;
			gl_curaccum = 0;
		}
	}

	return (gl_ctx != NULL);
}


static int InitOld(_THIS, SDL_Surface *current)
{
	GLenum osmesa_format;
	SDL_PixelFormat *pixel_format;
	Uint32	redmask;
	int recreatecontext, tinygl_present;

	if (this->gl_config.dll_handle) {
		if (this->gl_data->OSMesaCreateLDG == NULL) {
			return 0;
		}
	}

	/* TinyGL only supports VDI_RGB (OSMESA_RGB) */
	tinygl_present=0;
	if (this->gl_config.dll_handle) {
		if (this->gl_data->glFinish == NULL) {
			tinygl_present=1;
		}
	}

	/* Init OpenGL context using OSMesa */
	gl_convert = ConvertNull;
	gl_copyshadow = CopyShadowNull;
	gl_upsidedown = SDL_FALSE;

	pixel_format = current->format;
	redmask = pixel_format->Rmask;
	switch (pixel_format->BitsPerPixel) {
		case 15:
			/* 15 bits unsupported */
			if (tinygl_present) {
				gl_pixelsize = 3;
				osmesa_format = VDI_RGB;
				if (redmask == 31<<10) {
					gl_copyshadow = CopyShadowRGBTo555;
				} else {
					gl_copyshadow = CopyShadowRGBTo565;
					gl_convert = Convert565To555le;
				}
			} else {
				gl_pixelsize = 4;
				gl_upsidedown = SDL_TRUE;
				osmesa_format = OSMESA_ARGB;
				if (redmask == 31<<10) {
					gl_copyshadow = CopyShadow8888To555;
				} else {
					gl_copyshadow = CopyShadow8888To565;
					gl_convert = Convert565To555le;
				}
			}
			break;
		case 16:
			/* 16 bits unsupported */
			if (tinygl_present) {
				gl_pixelsize = 3;
				osmesa_format = VDI_RGB;
				gl_copyshadow = CopyShadowRGBTo565;
				if (redmask != 31<<11) {
					/* 565, little endian, unsupported */
					gl_convert = Convert565le;
				}
			} else {
				gl_pixelsize = 4;
				gl_upsidedown = SDL_TRUE;
				osmesa_format = OSMESA_ARGB;
				gl_copyshadow = CopyShadow8888To565;
				if (redmask != 31<<11) {
					/* 565, little endian, unsupported */
					gl_convert = Convert565le;
				}
			}
			break;
		case 24:
			gl_pixelsize = 3;
			if (tinygl_present) {
				osmesa_format = VDI_RGB;
				gl_copyshadow = CopyShadowDirect;
				if (redmask != 255<<16) {
					gl_copyshadow = CopyShadowRGBSwap;
				}
			} else {
				gl_copyshadow = CopyShadowDirect;
				gl_upsidedown = SDL_TRUE;
				if (redmask == 255<<16) {
					osmesa_format = OSMESA_RGB;
				} else {
					osmesa_format = OSMESA_BGR;
				}
			}
			break;
		case 32:
			if (tinygl_present) {
				gl_pixelsize = 3;
				osmesa_format = VDI_RGB;
				gl_copyshadow = CopyShadowRGBToARGB;
				if (redmask == 255) {
					gl_convert = CopyShadowRGBToABGR;
				} else if (redmask == 255<<8) {
					gl_convert = CopyShadowRGBToBGRA;
				} else if (redmask == 255<<24) {
					gl_convert = CopyShadowRGBToRGBA;
				}
			} else {
				gl_pixelsize = 4;
				gl_upsidedown = SDL_TRUE;
				gl_copyshadow = CopyShadowDirect;
				if (redmask == 255<<16) {
					osmesa_format = OSMESA_ARGB;
				} else if (redmask == 255<<8) {
					osmesa_format = OSMESA_BGRA;
				} else if (redmask == 255<<24) {
					osmesa_format = OSMESA_RGBA;
				} else {
					/* ABGR format unsupported */
					osmesa_format = OSMESA_BGRA;
					gl_convert = ConvertBGRAToABGR;
				}
			}
			break;
		default:
			if (tinygl_present) {
				SDL_AtariGL_Quit(this, SDL_FALSE);
				return 0;
			}
			gl_pixelsize = 1;
			gl_copyshadow = CopyShadowDirect;
			osmesa_format = OSMESA_COLOR_INDEX;
			break;
	}

	/* Try to keep current context if possible */
	recreatecontext=1;
	if (gl_shadow &&
		(gl_curformat == osmesa_format) &&
		(gl_curwidth == current->w) &&
		(gl_curheight == current->h)) {
		recreatecontext = 0;
	}
	if (recreatecontext) {
		SDL_AtariGL_Quit(this, SDL_FALSE);

		gl_shadow = this->gl_data->OSMesaCreateLDG(
			osmesa_format, GL_UNSIGNED_BYTE, current->w, current->h
		);

		if (gl_shadow) {
			gl_curformat = osmesa_format;
			gl_curwidth = current->w;
			gl_curheight = current->h;
		} else {
			gl_curformat = 0;
			gl_curwidth = 0;
			gl_curheight = 0;
		}
	}

	return (gl_shadow != NULL);
}

/*--- Conversions routines from shadow buffer to the screen ---*/

static void CopyShadowNull(_THIS, SDL_Surface *surface)
{
}

static void CopyShadowDirect(_THIS, SDL_Surface *surface)
{
	int y, srcpitch, dstpitch;
	Uint8 *srcline, *dstline;

	srcline = gl_shadow;
	srcpitch = surface->w * gl_pixelsize;
	dstline = surface->pixels;
	dstpitch = surface->pitch;
	if (gl_upsidedown) {
		srcline += (surface->h-1)*srcpitch;
		srcpitch = -srcpitch;
	}

	for (y=0; y<surface->h; y++) {
		SDL_memcpy(dstline, srcline, srcpitch);

		srcline += srcpitch;
		dstline += dstpitch;
	}
}

static void CopyShadowRGBTo555(_THIS, SDL_Surface *surface)
{
	int x,y, srcpitch, dstpitch;
	Uint16 *dstline, *dstcol;
	Uint8 *srcline, *srccol;

	srcline = (Uint8 *)gl_shadow;
	srcpitch = surface->w * gl_pixelsize;
	dstline = surface->pixels;
	dstpitch = surface->pitch >>1;
	if (gl_upsidedown) {
		srcline += (surface->h-1)*srcpitch;
		srcpitch = -srcpitch;
	}

	for (y=0; y<surface->h; y++) {
		srccol = srcline;
		dstcol = dstline;
		for (x=0; x<surface->w; x++) {
			Uint16 dstcolor;
			
			dstcolor = ((*srccol++)<<7) & (31<<10);
			dstcolor |= ((*srccol++)<<2) & (31<<5);
			dstcolor |= ((*srccol++)>>3) & 31;
			*dstcol++ = dstcolor;
		}

		srcline += srcpitch;
		dstline += dstpitch;
	}
}

static void CopyShadowRGBTo565(_THIS, SDL_Surface *surface)
{
	int x,y, srcpitch, dstpitch;
	Uint16 *dstline, *dstcol;
	Uint8 *srcline, *srccol;

	srcline = (Uint8 *)gl_shadow;
	srcpitch = surface->w * gl_pixelsize;
	dstline = surface->pixels;
	dstpitch = surface->pitch >>1;
	if (gl_upsidedown) {
		srcline += (surface->h-1)*srcpitch;
		srcpitch = -srcpitch;
	}

	for (y=0; y<surface->h; y++) {
		srccol = srcline;
		dstcol = dstline;

		for (x=0; x<surface->w; x++) {
			Uint16 dstcolor;
			
			dstcolor = ((*srccol++)<<8) & (31<<11);
			dstcolor |= ((*srccol++)<<3) & (63<<5);
			dstcolor |= ((*srccol++)>>3) & 31;
			*dstcol++ = dstcolor;
		}

		srcline += srcpitch;
		dstline += dstpitch;
	}
}

static void CopyShadowRGBSwap(_THIS, SDL_Surface *surface)
{
	int x,y, srcpitch, dstpitch;
	Uint8 *dstline, *dstcol;
	Uint8 *srcline, *srccol;

	srcline = (Uint8 *)gl_shadow;
	srcpitch = surface->w * gl_pixelsize;
	dstline = surface->pixels;
	dstpitch = surface->pitch;
	if (gl_upsidedown) {
		srcline += (surface->h-1)*srcpitch;
		srcpitch = -srcpitch;
	}

	for (y=0; y<surface->h; y++) {
		srccol = srcline;
		dstcol = dstline;

		for (x=0; x<surface->w; x++) {
			*dstcol++ = srccol[2];
			*dstcol++ = srccol[1];
			*dstcol++ = srccol[0];
			srccol += 3;
		}

		srcline += srcpitch;
		dstline += dstpitch;
	}
}

static void CopyShadowRGBToARGB(_THIS, SDL_Surface *surface)
{
	int x,y, srcpitch, dstpitch;
	Uint32 *dstline, *dstcol;
	Uint8 *srcline, *srccol;

	srcline = (Uint8 *)gl_shadow;
	srcpitch = surface->w * gl_pixelsize;
	dstline = surface->pixels;
	dstpitch = surface->pitch >>2;
	if (gl_upsidedown) {
		srcline += (surface->h-1)*srcpitch;
		srcpitch = -srcpitch;
	}

	for (y=0; y<surface->h; y++) {
		srccol = srcline;
		dstcol = dstline;

		for (x=0; x<surface->w; x++) {
			Uint32	dstcolor;

			dstcolor = (*srccol++)<<16;
			dstcolor |= (*srccol++)<<8;
			dstcolor |= *srccol++;

			*dstcol++ = dstcolor;
		}

		srcline += srcpitch;
		dstline += dstpitch;
	}
}

static void CopyShadowRGBToABGR(_THIS, SDL_Surface *surface)
{
	int x,y, srcpitch, dstpitch;
	Uint32 *dstline, *dstcol;
	Uint8 *srcline, *srccol;

	srcline = (Uint8 *)gl_shadow;
	srcpitch = surface->w * gl_pixelsize;
	dstline = surface->pixels;
	dstpitch = surface->pitch >>2;
	if (gl_upsidedown) {
		srcline += (surface->h-1)*srcpitch;
		srcpitch = -srcpitch;
	}

	for (y=0; y<surface->h; y++) {
		srccol = srcline;
		dstcol = dstline;

		for (x=0; x<surface->w; x++) {
			Uint32	dstcolor;

			dstcolor = *srccol++;
			dstcolor |= (*srccol++)<<8;
			dstcolor |= (*srccol++)<<16;

			*dstcol++ = dstcolor;
		}

		srcline += srcpitch;
		dstline += dstpitch;
	}
}

static void CopyShadowRGBToBGRA(_THIS, SDL_Surface *surface)
{
	int x,y, srcpitch, dstpitch;
	Uint32 *dstline, *dstcol;
	Uint8 *srcline, *srccol;

	srcline = (Uint8 *)gl_shadow;
	srcpitch = surface->w * gl_pixelsize;
	dstline = surface->pixels;
	dstpitch = surface->pitch >>2;
	if (gl_upsidedown) {
		srcline += (surface->h-1)*srcpitch;
		srcpitch = -srcpitch;
	}

	for (y=0; y<surface->h; y++) {
		srccol = srcline;
		dstcol = dstline;

		for (x=0; x<surface->w; x++) {
			Uint32	dstcolor;

			dstcolor = (*srccol++)<<8;
			dstcolor |= (*srccol++)<<16;
			dstcolor |= (*srccol++)<<24;

			*dstcol++ = dstcolor;
		}

		srcline += srcpitch;
		dstline += dstpitch;
	}
}

static void CopyShadowRGBToRGBA(_THIS, SDL_Surface *surface)
{
	int x,y, srcpitch, dstpitch;
	Uint32 *dstline, *dstcol;
	Uint8 *srcline, *srccol;

	srcline = (Uint8 *)gl_shadow;
	srcpitch = surface->w * gl_pixelsize;
	dstline = surface->pixels;
	dstpitch = surface->pitch >>2;
	if (gl_upsidedown) {
		srcline += (surface->h-1)*srcpitch;
		srcpitch = -srcpitch;
	}

	for (y=0; y<surface->h; y++) {
		srccol = srcline;
		dstcol = dstline;

		for (x=0; x<surface->w; x++) {
			Uint32	dstcolor;

			dstcolor = (*srccol++)<<24;
			dstcolor |= (*srccol++)<<16;
			dstcolor |= (*srccol++)<<8;

			*dstcol++ = dstcolor;
		}

		srcline += srcpitch;
		dstline += dstpitch;
	}
}

static void CopyShadow8888To555(_THIS, SDL_Surface *surface)
{
	int x,y, srcpitch, dstpitch;
	Uint16 *dstline, *dstcol;
	Uint32 *srcline, *srccol;

	srcline = (Uint32 *)gl_shadow;
	srcpitch = (surface->w * gl_pixelsize) >>2;
	dstline = surface->pixels;
	dstpitch = surface->pitch >>1;
	if (gl_upsidedown) {
		srcline += (surface->h-1)*srcpitch;
		srcpitch = -srcpitch;
	}

	for (y=0; y<surface->h; y++) {
		srccol = srcline;
		dstcol = dstline;
		for (x=0; x<surface->w; x++) {
			Uint32 srccolor;
			Uint16 dstcolor;
			
			srccolor = *srccol++;
			dstcolor = (srccolor>>9) & (31<<10);
			dstcolor |= (srccolor>>6) & (31<<5);
			dstcolor |= (srccolor>>3) & 31;
			*dstcol++ = dstcolor;
		}

		srcline += srcpitch;
		dstline += dstpitch;
	}
}

static void CopyShadow8888To565(_THIS, SDL_Surface *surface)
{
	int x,y, srcpitch, dstpitch;
	Uint16 *dstline, *dstcol;
	Uint32 *srcline, *srccol;

	srcline = (Uint32 *)gl_shadow;
	srcpitch = (surface->w * gl_pixelsize) >> 2;
	dstline = surface->pixels;
	dstpitch = surface->pitch >>1;
	if (gl_upsidedown) {
		srcline += (surface->h-1)*srcpitch;
		srcpitch = -srcpitch;
	}

	for (y=0; y<surface->h; y++) {
		srccol = srcline;
		dstcol = dstline;

		for (x=0; x<surface->w; x++) {
			Uint32 srccolor;
			Uint16 dstcolor;
			
			srccolor = *srccol++;
			dstcolor = (srccolor>>8) & (31<<11);
			dstcolor |= (srccolor>>5) & (63<<5);
			dstcolor |= (srccolor>>3) & 31;
			*dstcol++ = dstcolor;
		}

		srcline += srcpitch;
		dstline += dstpitch;
	}
}

/*--- Conversions routines in the screen ---*/

static void ConvertNull(_THIS, SDL_Surface *surface)
{
}

static void Convert565To555be(_THIS, SDL_Surface *surface)
{
	int x,y, pitch;
	unsigned short *line, *pixel;

	line = surface->pixels;
	pitch = surface->pitch >> 1;
	for (y=0; y<surface->h; y++) {
		pixel = line;
		for (x=0; x<surface->w; x++) {
			unsigned short color = *pixel;

			*pixel++ = (color & 0x1f)|((color>>1) & 0xffe0);
		}

		line += pitch;
	}
}

static void Convert565To555le(_THIS, SDL_Surface *surface)
{
	int x,y, pitch;
	unsigned short *line, *pixel;

	line = surface->pixels;
	pitch = surface->pitch >>1;
	for (y=0; y<surface->h; y++) {
		pixel = line;
		for (x=0; x<surface->w; x++) {
			unsigned short color = *pixel;

			color = (color & 0x1f)|((color>>1) & 0xffe0);
			*pixel++ = SDL_Swap16(color);
		}

		line += pitch;
	}
}

static void Convert565le(_THIS, SDL_Surface *surface)
{
	int x,y, pitch;
	unsigned short *line, *pixel;

	line = surface->pixels;
	pitch = surface->pitch >>1;
	for (y=0; y<surface->h; y++) {
		pixel = line;
		for (x=0; x<surface->w; x++) {
			unsigned short color = *pixel;

			*pixel++ = SDL_Swap16(color);
		}

		line += pitch;
	}
}

static void ConvertBGRAToABGR(_THIS, SDL_Surface *surface)
{
	int x,y, pitch;
	unsigned long *line, *pixel;

	line = surface->pixels;
	pitch = surface->pitch >>2;
	for (y=0; y<surface->h; y++) {
		pixel = line;
		for (x=0; x<surface->w; x++) {
			unsigned long color = *pixel;

			*pixel++ = (color<<24)|(color>>8);
		}

		line += pitch;
	}
}

#endif /* SDL_VIDEO_OPENGL */
