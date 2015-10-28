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

#ifndef _SDL_ATARIGL_H_
#define _SDL_ATARIGL_H_

#if SDL_VIDEO_OPENGL
#include <GL/osmesa.h>
#endif

#include "../SDL_sysvideo.h"

/* Hidden "this" pointer for the video functions */
#define _THIS   SDL_VideoDevice *this

struct SDL_PrivateGLData {

    int gl_active; 	/* to stop switching drivers while we have a valid context */
	
	int gl_oldmesa;	/* Old OpenGL support ? */

	int	gl_pixelsize;	/* for CopyShadow functions */

	SDL_bool gl_upsidedown; /* Some implementations draw upside down */

	Uint8 *gl_shadow;	/* Shadow buffer for old implementations */

	/* for unsupported OSMesa buffer formats */
	void (*ConvertSurface)(_THIS, SDL_Surface *surface);	

	/* to convert the shadow buffer to the screen format */
	void (*CopyShadow)(_THIS, SDL_Surface *surface);	

#if SDL_VIDEO_OPENGL
	OSMesaContext	ctx;

	/* OpenGL functions */
	void (*glGetIntegerv)( GLenum pname, GLint *value );
	void (*glFinish)(void);
	void (*glFlush)(void);

	/* osmesa.ldg */
	OSMesaContext (*OSMesaCreateContextExt)( GLenum format, GLint depthBits, GLint stencilBits, GLint accumBits, OSMesaContext sharelist);
	void (*OSMesaDestroyContext)( OSMesaContext ctx );
	GLboolean (*OSMesaMakeCurrent)( OSMesaContext ctx, void *buffer, GLenum type, GLsizei width, GLsizei height );
	void (*OSMesaPixelStore)( GLint pname, GLint value );
	void * (*OSMesaGetProcAddress)( const char *funcName );

	/* mesa_gl.ldg, tiny_gl.ldg */
	void *(*OSMesaCreateLDG)( long format, long type, long width, long height );
	void (*OSMesaDestroyLDG)(void);

	/* Info needed to compare existing context with new asked one */
	int width, height;
	GLenum format;
	GLint depth,stencil,accum;
#endif
};

/* Variable names */
#define gl_active		(this->gl_data->gl_active)
#define gl_ctx			(this->gl_data->ctx)
#define gl_oldmesa		(this->gl_data->gl_oldmesa)
#define gl_pixelsize	(this->gl_data->gl_pixelsize)
#define gl_upsidedown	(this->gl_data->gl_upsidedown)
#define gl_shadow		(this->gl_data->gl_shadow)
#define gl_convert		(this->gl_data->ConvertSurface)
#define gl_copyshadow	(this->gl_data->CopyShadow)
#define gl_curformat	(this->gl_data->format)
#define gl_curdepth		(this->gl_data->depth)
#define gl_curstencil	(this->gl_data->stencil)
#define gl_curaccum		(this->gl_data->accum)
#define gl_curwidth		(this->gl_data->width)
#define gl_curheight	(this->gl_data->height)

/* OpenGL functions */
extern int SDL_AtariGL_Init(_THIS, SDL_Surface *current);
extern void SDL_AtariGL_Quit(_THIS, SDL_bool unload);
extern void SDL_AtariGL_InitPointers(_THIS);

extern int SDL_AtariGL_LoadLibrary(_THIS, const char *path);
extern void *SDL_AtariGL_GetProcAddress(_THIS, const char *proc);
extern int SDL_AtariGL_GetAttribute(_THIS, SDL_GLattr attrib, int* value);
extern int SDL_AtariGL_MakeCurrent(_THIS);
extern void SDL_AtariGL_SwapBuffers(_THIS);

#endif /* _SDL_ATARIGL_H_ */
