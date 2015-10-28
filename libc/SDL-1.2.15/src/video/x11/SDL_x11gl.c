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

#include "SDL_x11video.h"
#include "../../events/SDL_events_c.h"
#include "SDL_x11dga_c.h"
#include "SDL_x11gl_c.h"

#if defined(__IRIX__)
/* IRIX doesn't have a GL library versioning system */
#define DEFAULT_OPENGL	"libGL.so"
#elif defined(__MACOSX__)
#define DEFAULT_OPENGL	"/usr/X11R6/lib/libGL.1.dylib"
#elif defined(__QNXNTO__)
#define DEFAULT_OPENGL	"libGL.so.3"
#elif defined(__OpenBSD__)
#define DEFAULT_OPENGL	"libGL.so.4.0"
#else
#define DEFAULT_OPENGL	"libGL.so.1"
#endif

#ifndef GLX_ARB_multisample
#define GLX_ARB_multisample
#define GLX_SAMPLE_BUFFERS_ARB             100000
#define GLX_SAMPLES_ARB                    100001
#endif

/* GLX_EXT_visual_rating stuff that might not be in the system headers... */
#ifndef GLX_VISUAL_CAVEAT_EXT
#define GLX_VISUAL_CAVEAT_EXT              0x20
#endif
#ifndef GLX_NONE_EXT
#define GLX_NONE_EXT                       0x8000
#endif
#ifndef GLX_SLOW_VISUAL_EXT
#define GLX_SLOW_VISUAL_EXT                0x8001
#endif
#ifndef GLX_NON_CONFORMANT_VISUAL_EXT
#define GLX_NON_CONFORMANT_VISUAL_EXT      0x800D
#endif


#if SDL_VIDEO_OPENGL_GLX
static int glXExtensionSupported(_THIS, const char *extension)
{
	const char *extensions;
	const char *start;
	const char *where, *terminator;

	/* Extension names should not have spaces. */
	where = SDL_strchr(extension, ' ');
	if ( where || *extension == '\0' ) {
	      return 0;
	}

	extensions = this->gl_data->glXQueryExtensionsString(GFX_Display,SDL_Screen);
	/* It takes a bit of care to be fool-proof about parsing the
	 * OpenGL extensions string. Don't be fooled by sub-strings, etc.
	 */

	/* http://bugs.debian.org/537487 */
	if (extensions == NULL) {
	      return 0;
	}
	
	start = extensions;
	
	for (;;) {
		where = SDL_strstr(start, extension);
		if (!where) break;
		
		terminator = where + strlen(extension);
		if (where == start || *(where - 1) == ' ')
	        if (*terminator == ' ' || *terminator == '\0') return 1;
						  
		start = terminator;
	}
	return 0;
}
#endif /* SDL_VIDEO_OPENGL_GLX */

XVisualInfo *X11_GL_GetVisual(_THIS)
{
#if SDL_VIDEO_OPENGL_GLX
	/* 64 seems nice. */
	int attribs[64];
	int i;

	/* load the gl driver from a default path */
	if ( ! this->gl_config.driver_loaded ) {
	        /* no driver has been loaded, use default (ourselves) */
	        if ( X11_GL_LoadLibrary(this, NULL) < 0 ) {
		        return NULL;
		}
	}

	/* See if we already have a window which we must use */
	if ( SDL_windowid ) {
		XWindowAttributes a;
		XVisualInfo vi_in;
		int out_count;

		XGetWindowAttributes(SDL_Display, SDL_Window, &a);
		vi_in.screen = SDL_Screen;
		vi_in.visualid = XVisualIDFromVisual(a.visual);
		glx_visualinfo = XGetVisualInfo(SDL_Display,
	                     VisualScreenMask|VisualIDMask, &vi_in, &out_count);
		return glx_visualinfo;
	}

        /* Setup our GLX attributes according to the gl_config. */
	i = 0;
	attribs[i++] = GLX_RGBA;
	attribs[i++] = GLX_RED_SIZE;
	attribs[i++] = this->gl_config.red_size;
	attribs[i++] = GLX_GREEN_SIZE;
	attribs[i++] = this->gl_config.green_size;
	attribs[i++] = GLX_BLUE_SIZE;
	attribs[i++] = this->gl_config.blue_size;

	if( this->gl_config.alpha_size ) {
		attribs[i++] = GLX_ALPHA_SIZE;
		attribs[i++] = this->gl_config.alpha_size;
	}

	if( this->gl_config.double_buffer ) {
		attribs[i++] = GLX_DOUBLEBUFFER;
	}

	attribs[i++] = GLX_DEPTH_SIZE;
	attribs[i++] = this->gl_config.depth_size;

	if( this->gl_config.stencil_size ) {
		attribs[i++] = GLX_STENCIL_SIZE;
		attribs[i++] = this->gl_config.stencil_size;
	}

	if( this->gl_config.accum_red_size ) {
		attribs[i++] = GLX_ACCUM_RED_SIZE;
		attribs[i++] = this->gl_config.accum_red_size;
	}

	if( this->gl_config.accum_green_size ) {
		attribs[i++] = GLX_ACCUM_GREEN_SIZE;
		attribs[i++] = this->gl_config.accum_green_size;
	}

	if( this->gl_config.accum_blue_size ) {
		attribs[i++] = GLX_ACCUM_BLUE_SIZE;
		attribs[i++] = this->gl_config.accum_blue_size;
	}

	if( this->gl_config.accum_alpha_size ) {
		attribs[i++] = GLX_ACCUM_ALPHA_SIZE;
		attribs[i++] = this->gl_config.accum_alpha_size;
	}

	if( this->gl_config.stereo ) {
		attribs[i++] = GLX_STEREO;
	}
	
	if( this->gl_config.multisamplebuffers ) {
		attribs[i++] = GLX_SAMPLE_BUFFERS_ARB;
		attribs[i++] = this->gl_config.multisamplebuffers;
	}
	
	if( this->gl_config.multisamplesamples ) {
		attribs[i++] = GLX_SAMPLES_ARB;
		attribs[i++] = this->gl_config.multisamplesamples;
	}

	if( this->gl_config.accelerated >= 0 &&
	    glXExtensionSupported(this, "GLX_EXT_visual_rating") ) {
		attribs[i++] = GLX_VISUAL_CAVEAT_EXT;
		attribs[i++] = GLX_NONE_EXT;
	}

#ifdef GLX_DIRECT_COLOR /* Try for a DirectColor visual for gamma support */
	if ( !SDL_getenv("SDL_VIDEO_X11_NODIRECTCOLOR") ) {
		attribs[i++] = GLX_X_VISUAL_TYPE;
		attribs[i++] = GLX_DIRECT_COLOR;
	}
#endif
	attribs[i++] = None;

 	glx_visualinfo = this->gl_data->glXChooseVisual(GFX_Display, 
						  SDL_Screen, attribs);
#ifdef GLX_DIRECT_COLOR
	if( !glx_visualinfo && !SDL_getenv("SDL_VIDEO_X11_NODIRECTCOLOR") ) { /* No DirectColor visual?  Try again.. */
		attribs[i-3] = None;
 		glx_visualinfo = this->gl_data->glXChooseVisual(GFX_Display, 
						  SDL_Screen, attribs);
	}
#endif
	if( !glx_visualinfo ) {
		SDL_SetError( "Couldn't find matching GLX visual");
		return NULL;
	}
/*
	printf("Found GLX visual 0x%x\n", glx_visualinfo->visualid);
*/
	return glx_visualinfo;
#else
	SDL_SetError("X11 driver not configured with OpenGL");
	return NULL;
#endif
}

int X11_GL_CreateWindow(_THIS, int w, int h)
{
	int retval;
#if SDL_VIDEO_OPENGL_GLX
	XSetWindowAttributes attributes;
	unsigned long mask;
	unsigned long black;

	black = (glx_visualinfo->visual == DefaultVisual(SDL_Display,
						 	SDL_Screen))
	       	? BlackPixel(SDL_Display, SDL_Screen) : 0;
	attributes.background_pixel = black;
	attributes.border_pixel = black;
	attributes.colormap = SDL_XColorMap;
	mask = CWBackPixel | CWBorderPixel | CWColormap;

	SDL_Window = XCreateWindow(SDL_Display, WMwindow,
			0, 0, w, h, 0, glx_visualinfo->depth,
			InputOutput, glx_visualinfo->visual,
			mask, &attributes);
	if ( !SDL_Window ) {
		SDL_SetError("Could not create window");
		return -1;
	}
	retval = 0;
#else
	SDL_SetError("X11 driver not configured with OpenGL");
	retval = -1;
#endif
	return(retval);
}

int X11_GL_CreateContext(_THIS)
{
	int retval;
#if SDL_VIDEO_OPENGL_GLX

	/* We do this to create a clean separation between X and GLX errors. */
	XSync( SDL_Display, False );
	glx_context = this->gl_data->glXCreateContext(GFX_Display, 
				     glx_visualinfo, NULL, True);
	XSync( GFX_Display, False );

	if ( glx_context == NULL ) {
		SDL_SetError("Could not create GL context");
		return(-1);
	}
	if ( X11_GL_MakeCurrent(this) < 0 ) {
		return(-1);
	}
	gl_active = 1;

	if ( !glXExtensionSupported(this, "GLX_SGI_swap_control") ) {
		this->gl_data->glXSwapIntervalSGI = NULL;
	}
	if ( !glXExtensionSupported(this, "GLX_MESA_swap_control") ) {
		this->gl_data->glXSwapIntervalMESA = NULL;
	}
	if ( !glXExtensionSupported(this, "GLX_EXT_swap_control") ) {
		this->gl_data->glXSwapIntervalEXT = NULL;
	}

	if ( this->gl_config.swap_control >= 0 ) {
		int rc = -1;
		if ( this->gl_data->glXSwapIntervalEXT ) {
			rc = this->gl_data->glXSwapIntervalEXT(GFX_Display, SDL_Window,
					this->gl_config.swap_control);
		} else if ( this->gl_data->glXSwapIntervalMESA ) {
			rc = this->gl_data->glXSwapIntervalMESA(this->gl_config.swap_control);
		} else if ( this->gl_data->glXSwapIntervalSGI ) {
			rc = this->gl_data->glXSwapIntervalSGI(this->gl_config.swap_control);
		}
		if (rc == 0) {
			this->gl_data->swap_interval = this->gl_config.swap_control;
		}
	}
#else
	SDL_SetError("X11 driver not configured with OpenGL");
#endif
	if ( gl_active ) {
		retval = 0;
	} else {
		retval = -1;
	}
	return(retval);
}

void X11_GL_Shutdown(_THIS)
{
#if SDL_VIDEO_OPENGL_GLX
	/* Clean up OpenGL */
	if( glx_context ) {
		this->gl_data->glXMakeCurrent(GFX_Display, None, NULL);

		if (glx_context != NULL)
			this->gl_data->glXDestroyContext(GFX_Display, glx_context);

		glx_context = NULL;
	}
	gl_active = 0;
#endif /* SDL_VIDEO_OPENGL_GLX */
}

#if SDL_VIDEO_OPENGL_GLX

/* Make the current context active */
int X11_GL_MakeCurrent(_THIS)
{
	int retval;
	
	retval = 0;
	if ( ! this->gl_data->glXMakeCurrent(GFX_Display,
	                                     SDL_Window, glx_context) ) {
		SDL_SetError("Unable to make GL context current");
		retval = -1;
	}
	XSync( GFX_Display, False );

	/* More Voodoo X server workarounds... Grr... */
	SDL_Lock_EventThread();
	X11_CheckDGAMouse(this);
	SDL_Unlock_EventThread();

	return(retval);
}

/* Get attribute data from glX. */
int X11_GL_GetAttribute(_THIS, SDL_GLattr attrib, int* value)
{
	int retval = -1;
	int unsupported = 0;
	int glx_attrib = None;

	switch( attrib ) {
	    case SDL_GL_RED_SIZE:
		glx_attrib = GLX_RED_SIZE;
		break;
	    case SDL_GL_GREEN_SIZE:
		glx_attrib = GLX_GREEN_SIZE;
		break;
	    case SDL_GL_BLUE_SIZE:
		glx_attrib = GLX_BLUE_SIZE;
		break;
	    case SDL_GL_ALPHA_SIZE:
		glx_attrib = GLX_ALPHA_SIZE;
		break;
	    case SDL_GL_DOUBLEBUFFER:
		glx_attrib = GLX_DOUBLEBUFFER;
		break;
	    case SDL_GL_BUFFER_SIZE:
		glx_attrib = GLX_BUFFER_SIZE;
		break;
	    case SDL_GL_DEPTH_SIZE:
		glx_attrib = GLX_DEPTH_SIZE;
		break;
	    case SDL_GL_STENCIL_SIZE:
		glx_attrib = GLX_STENCIL_SIZE;
		break;
	    case SDL_GL_ACCUM_RED_SIZE:
		glx_attrib = GLX_ACCUM_RED_SIZE;
		break;
	    case SDL_GL_ACCUM_GREEN_SIZE:
		glx_attrib = GLX_ACCUM_GREEN_SIZE;
		break;
	    case SDL_GL_ACCUM_BLUE_SIZE:
		glx_attrib = GLX_ACCUM_BLUE_SIZE;
		break;
	    case SDL_GL_ACCUM_ALPHA_SIZE:
		glx_attrib = GLX_ACCUM_ALPHA_SIZE;
		break;
	    case SDL_GL_STEREO:
		glx_attrib = GLX_STEREO;
		break;
 	    case SDL_GL_MULTISAMPLEBUFFERS:
 		glx_attrib = GLX_SAMPLE_BUFFERS_ARB;
 		break;
 	    case SDL_GL_MULTISAMPLESAMPLES:
 		glx_attrib = GLX_SAMPLES_ARB;
 		break;
 	    case SDL_GL_ACCELERATED_VISUAL:
		if ( glXExtensionSupported(this, "GLX_EXT_visual_rating") ) {
			glx_attrib = GLX_VISUAL_CAVEAT_EXT;
			retval = this->gl_data->glXGetConfig(GFX_Display, glx_visualinfo, glx_attrib, value);
			if ( *value == GLX_SLOW_VISUAL_EXT ) {
				*value = SDL_FALSE;
			} else {
				*value = SDL_TRUE;
			}
			return retval;
		} else {
			unsupported = 1;
		}
		break;
	    case SDL_GL_SWAP_CONTROL:
		if ( ( this->gl_data->glXSwapIntervalEXT ) ||
		     ( this->gl_data->glXSwapIntervalMESA ) ||
		     ( this->gl_data->glXSwapIntervalSGI ) ) {
			*value = this->gl_data->swap_interval;
			return 0;
		} else {
			unsupported = 1;
		}
		break;
	    default:
			unsupported = 1;
			break;
	}

	if (unsupported) {
		SDL_SetError("OpenGL attribute is unsupported on this system");
	} else {
		retval = this->gl_data->glXGetConfig(GFX_Display, glx_visualinfo, glx_attrib, value);
	}
	return retval;
}

void X11_GL_SwapBuffers(_THIS)
{
	this->gl_data->glXSwapBuffers(GFX_Display, SDL_Window);
}

#endif /* SDL_VIDEO_OPENGL_GLX */

#define OPENGL_REQUIRS_DLOPEN
#if defined(OPENGL_REQUIRS_DLOPEN) && defined(SDL_LOADSO_DLOPEN)
#include <dlfcn.h>
#define GL_LoadObject(X)	dlopen(X, (RTLD_NOW|RTLD_GLOBAL))
#define GL_LoadFunction		dlsym
#define GL_UnloadObject		dlclose
#else
#define GL_LoadObject	SDL_LoadObject
#define GL_LoadFunction	SDL_LoadFunction
#define GL_UnloadObject	SDL_UnloadObject
#endif

void X11_GL_UnloadLibrary(_THIS)
{
#if SDL_VIDEO_OPENGL_GLX
	if ( this->gl_config.driver_loaded ) {

		GL_UnloadObject(this->gl_config.dll_handle);

		this->gl_data->glXGetProcAddress = NULL;
		this->gl_data->glXChooseVisual = NULL;
		this->gl_data->glXCreateContext = NULL;
		this->gl_data->glXDestroyContext = NULL;
		this->gl_data->glXMakeCurrent = NULL;
		this->gl_data->glXSwapBuffers = NULL;
		this->gl_data->glXSwapIntervalSGI = NULL;
		this->gl_data->glXSwapIntervalMESA = NULL;
		this->gl_data->glXSwapIntervalEXT = NULL;

		this->gl_config.dll_handle = NULL;
		this->gl_config.driver_loaded = 0;
	}
#endif
}

#if SDL_VIDEO_OPENGL_GLX

/* Passing a NULL path means load pointers from the application */
int X11_GL_LoadLibrary(_THIS, const char* path) 
{
	void* handle = NULL;

	if ( gl_active ) {
		SDL_SetError("OpenGL context already created");
		return -1;
	}

	if ( path == NULL ) {
		path = SDL_getenv("SDL_VIDEO_GL_DRIVER");
		if ( path == NULL ) {
			path = DEFAULT_OPENGL;
		}
	}

	handle = GL_LoadObject(path);
	if ( handle == NULL ) {
#if defined(OPENGL_REQUIRS_DLOPEN) && defined(SDL_LOADSO_DLOPEN)
		SDL_SetError("Failed loading %s", path);
#else
		/* SDL_LoadObject() will call SDL_SetError() for us. */
#endif
		return -1;
	}

	/* Unload the old driver and reset the pointers */
	X11_GL_UnloadLibrary(this);

	/* Save the handle for X11_GL_GetProcAddress() */
	this->gl_config.dll_handle = handle;

	/* Load new function pointers */
	this->gl_data->glXGetProcAddress =
		(void *(*)(const GLubyte *)) GL_LoadFunction(handle, "glXGetProcAddressARB");
	this->gl_data->glXChooseVisual =
		(XVisualInfo *(*)(Display *, int, int *)) X11_GL_GetProcAddress(this, "glXChooseVisual");
	this->gl_data->glXCreateContext =
		(GLXContext (*)(Display *, XVisualInfo *, GLXContext, int)) X11_GL_GetProcAddress(this, "glXCreateContext");
	this->gl_data->glXDestroyContext =
		(void (*)(Display *, GLXContext)) X11_GL_GetProcAddress(this, "glXDestroyContext");
	this->gl_data->glXMakeCurrent =
		(int (*)(Display *, GLXDrawable, GLXContext)) X11_GL_GetProcAddress(this, "glXMakeCurrent");
	this->gl_data->glXSwapBuffers =
		(void (*)(Display *, GLXDrawable)) X11_GL_GetProcAddress(this, "glXSwapBuffers");
	this->gl_data->glXGetConfig =
		(int (*)(Display *, XVisualInfo *, int, int *)) X11_GL_GetProcAddress(this, "glXGetConfig");
	this->gl_data->glXQueryExtensionsString =
		(const char *(*)(Display *, int)) X11_GL_GetProcAddress(this, "glXQueryExtensionsString");
	this->gl_data->glXSwapIntervalSGI =
		(int (*)(int)) X11_GL_GetProcAddress(this, "glXSwapIntervalSGI");
	this->gl_data->glXSwapIntervalMESA =
		(GLint (*)(unsigned)) X11_GL_GetProcAddress(this, "glXSwapIntervalMESA");
	this->gl_data->glXSwapIntervalEXT =
		(int (*)(Display*,GLXDrawable,int)) X11_GL_GetProcAddress(this, "glXSwapIntervalEXT");

	if ( (this->gl_data->glXChooseVisual == NULL) || 
	     (this->gl_data->glXCreateContext == NULL) ||
	     (this->gl_data->glXDestroyContext == NULL) ||
	     (this->gl_data->glXMakeCurrent == NULL) ||
	     (this->gl_data->glXSwapBuffers == NULL) ||
	     (this->gl_data->glXGetConfig == NULL) ||
	     (this->gl_data->glXQueryExtensionsString == NULL)) {
		GL_UnloadObject(this->gl_config.dll_handle);
		this->gl_config.dll_handle = NULL;
		SDL_SetError("Could not retrieve OpenGL functions");
		return -1;
	}

	this->gl_config.driver_loaded = 1;
	if ( path ) {
		SDL_strlcpy(this->gl_config.driver_path, path,
			SDL_arraysize(this->gl_config.driver_path));
	} else {
		*this->gl_config.driver_path = '\0';
	}
	return 0;
}

void *X11_GL_GetProcAddress(_THIS, const char* proc)
{
	if ( this->gl_data->glXGetProcAddress ) {
		return this->gl_data->glXGetProcAddress((const GLubyte *)proc);
	}
	return GL_LoadFunction(this->gl_config.dll_handle, proc);
}

#endif /* SDL_VIDEO_OPENGL_GLX */
