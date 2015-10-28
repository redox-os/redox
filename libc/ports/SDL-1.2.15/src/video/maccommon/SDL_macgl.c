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

/* AGL implementation of SDL OpenGL support */

#include "SDL_lowvideo.h"
#include "SDL_macgl_c.h"
#include "SDL_loadso.h"


/* krat: adding OpenGL support */
int Mac_GL_Init(_THIS)
{
#if SDL_VIDEO_OPENGL
	AGLPixelFormat format;
   	int i = 0;
	GLint attributes [ 26 ]; /* 26 is max possible in this setup */
	GLboolean noerr;
   
	/* load the gl driver from a default path */
	if ( ! this->gl_config.driver_loaded ) {
		/* no driver has been loaded, use default (ourselves) */
		if ( Mac_GL_LoadLibrary(this, NULL) < 0 ) {
			return(-1);
		}
	}

	attributes[i++] = AGL_RGBA;
	if ( this->gl_config.red_size   != 0 &&
	     this->gl_config.blue_size  != 0 &&
	     this->gl_config.green_size != 0 ) {
		attributes[i++] = AGL_RED_SIZE;
		attributes[i++] = this->gl_config.red_size;
		attributes[i++] = AGL_GREEN_SIZE;
		attributes[i++] = this->gl_config.green_size;
		attributes[i++] = AGL_BLUE_SIZE;
		attributes[i++] = this->gl_config.blue_size;
		attributes[i++] = AGL_ALPHA_SIZE;
		attributes[i++] = this->gl_config.alpha_size;
	}
	if ( this->gl_config.double_buffer ) {
		attributes[i++] = AGL_DOUBLEBUFFER;
	}
	if ( this->gl_config.depth_size != 0 ) {
		attributes[i++] = AGL_DEPTH_SIZE;
		attributes[i++] = this->gl_config.depth_size;
	}	
	if ( this->gl_config.stencil_size != 0 ) {
		attributes[i++] = AGL_STENCIL_SIZE;
		attributes[i++] = this->gl_config.stencil_size;
	}
	if ( this->gl_config.accum_red_size   != 0 &&
	     this->gl_config.accum_blue_size  != 0 &&
	     this->gl_config.accum_green_size != 0 ) {
		
		attributes[i++] = AGL_ACCUM_RED_SIZE;
		attributes[i++] = this->gl_config.accum_red_size;
		attributes[i++] = AGL_ACCUM_GREEN_SIZE;
		attributes[i++] = this->gl_config.accum_green_size;
		attributes[i++] = AGL_ACCUM_BLUE_SIZE;
		attributes[i++] = this->gl_config.accum_blue_size;
		attributes[i++] = AGL_ACCUM_ALPHA_SIZE;
		attributes[i++] = this->gl_config.accum_alpha_size;
	}
	if ( this->gl_config.stereo ) {
		attributes[i++] = AGL_STEREO;
	}
#if defined(AGL_SAMPLE_BUFFERS_ARB) && defined(AGL_SAMPLES_ARB)
	if ( this->gl_config.multisamplebuffers != 0 ) {
		attributes[i++] = AGL_SAMPLE_BUFFERS_ARB;
		attributes[i++] = this->gl_config.multisamplebuffers;
	}	
	if ( this->gl_config.multisamplesamples != 0 ) {
		attributes[i++] = AGL_SAMPLES_ARB;
		attributes[i++] = this->gl_config.multisamplesamples;
	}	
#endif
	if ( this->gl_config.accelerated > 0 ) {
		attributes[i++] = AGL_ACCELERATED;
		attributes[i++] = AGL_NO_RECOVERY;
	}

	attributes[i++] = AGL_ALL_RENDERERS;
	attributes[i]	= AGL_NONE;

	format = aglChoosePixelFormat(NULL, 0, attributes);
	if ( format == NULL ) {
		SDL_SetError("Couldn't match OpenGL desired format");
		return(-1);
	}

	glContext = aglCreateContext(format, NULL);
	if ( glContext == NULL ) {
		SDL_SetError("Couldn't create OpenGL context");
		return(-1);
	}
	aglDestroyPixelFormat(format);

    #if  TARGET_API_MAC_CARBON
	noerr = aglSetDrawable(glContext, GetWindowPort(SDL_Window));
    #else
	noerr = aglSetDrawable(glContext, (AGLDrawable)SDL_Window);
    #endif
    
	if(!noerr) {
		SDL_SetError("Unable to bind GL context to window");
		return(-1);
	}
	return(0);
#else
	SDL_SetError("OpenGL support not configured");
	return(-1);
#endif
}

void Mac_GL_Quit(_THIS)
{
#if SDL_VIDEO_OPENGL
	if ( glContext != NULL ) {
		aglSetCurrentContext(NULL);
		aglSetDrawable(glContext, NULL);
		aglDestroyContext(glContext);		
		glContext = NULL;
	}
#endif
}

#if SDL_VIDEO_OPENGL

/* Make the current context active */
int Mac_GL_MakeCurrent(_THIS)
{
	int retval;

	retval = 0;
	if( ! aglSetCurrentContext(glContext) ) {
		SDL_SetError("Unable to make GL context current");
		retval = -1;
	}
	return(retval);
}

void Mac_GL_SwapBuffers(_THIS)
{
	aglSwapBuffers(glContext);
}

int Mac_GL_LoadLibrary(_THIS, const char *location)
{
	if (location == NULL)
#if __MACH__
		location = "/System/Library/Frameworks/OpenGL.framework/OpenGL";
#else
		location = "OpenGLLibrary";
#endif

	this->hidden->libraryHandle = SDL_LoadObject(location);

	this->gl_config.driver_loaded = 1;
	return (this->hidden->libraryHandle != NULL) ? 0 : -1;
}

void Mac_GL_UnloadLibrary(_THIS)
{
	SDL_UnloadObject(this->hidden->libraryHandle);

	this->hidden->libraryHandle = NULL;
	this->gl_config.driver_loaded = 0;
}

void* Mac_GL_GetProcAddress(_THIS, const char *proc)
{
	return SDL_LoadFunction( this->hidden->libraryHandle, proc );
}

#endif /* SDL_VIDEO_OPENGL */

