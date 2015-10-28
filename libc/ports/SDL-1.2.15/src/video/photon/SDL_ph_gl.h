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

#ifndef __SDL_PH_GL_H__
#define __SDL_PH_GL_H__

#include "SDL_ph_video.h"

#define DEFAULT_OPENGL "/usr/lib/libGL.so"

#if SDL_VIDEO_OPENGL
    void  ph_GL_SwapBuffers(_THIS);
    int   ph_GL_GetAttribute(_THIS, SDL_GLattr attrib, int* value);
    int   ph_GL_LoadLibrary(_THIS, const char* path);
    void* ph_GL_GetProcAddress(_THIS, const char* proc);
    int   ph_GL_MakeCurrent(_THIS);

    int   ph_SetupOpenGLContext(_THIS, int width, int height, int bpp, Uint32 flags);
#endif /* SDL_VIDEO_OPENGL */

#endif /* __SDL_PH_GL_H__ */
