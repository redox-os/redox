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

#include <dlfcn.h>
#include "SDL.h"
#include "SDL_ph_gl.h"

#if SDL_VIDEO_OPENGL

#if (_NTO_VERSION >= 630)
    /* PhotonGL functions */
    GLPH_DECLARE_FUNCS;
#endif /* 6.3.0 */

#if (_NTO_VERSION < 630)
void ph_GL_SwapBuffers(_THIS)
{
    PgSetRegion(PtWidgetRid(window));
    PdOpenGLContextSwapBuffers(oglctx);
}
#else
void ph_GL_SwapBuffers(_THIS)
{
    qnxgl_swap_buffers(oglbuffers);
}
#endif /* 6.3.0 */

int ph_GL_GetAttribute(_THIS, SDL_GLattr attrib, int* value)
{
    switch (attrib)
    {
        case SDL_GL_DOUBLEBUFFER:
             *value=this->gl_config.double_buffer;
             break;
        case SDL_GL_STENCIL_SIZE:
             *value=this->gl_config.stencil_size;
             break;
        case SDL_GL_DEPTH_SIZE:
             *value=this->gl_config.depth_size;
             break;
#if (_NTO_VERSION >= 630)
        case SDL_GL_RED_SIZE:
             *value=this->gl_config.red_size;
             break;
        case SDL_GL_GREEN_SIZE:
             *value=this->gl_config.green_size;
             break;
        case SDL_GL_BLUE_SIZE:
             *value=this->gl_config.blue_size;
             break;
        case SDL_GL_ALPHA_SIZE:
             *value=this->gl_config.alpha_size;
             break;
        case SDL_GL_ACCUM_RED_SIZE:
             *value=this->gl_config.accum_red_size;
             break;
        case SDL_GL_ACCUM_GREEN_SIZE:
             *value=this->gl_config.accum_green_size;
             break;
        case SDL_GL_ACCUM_BLUE_SIZE:
             *value=this->gl_config.accum_blue_size;
             break;
        case SDL_GL_ACCUM_ALPHA_SIZE:
             *value=this->gl_config.accum_alpha_size;
             break;
        case SDL_GL_STEREO:
             *value=this->gl_config.stereo;
             break;
#endif /* 6.3.0 */
        default:
             *value=0;
             return(-1);
    }
    return 0;
}

#if (_NTO_VERSION < 630)
int ph_GL_LoadLibrary(_THIS, const char* path)
{
    /* if code compiled with SDL_VIDEO_OPENGL, that mean that library already linked */
    this->gl_config.driver_loaded = 1;

    return 0;
}
#else
int ph_GL_LoadLibrary(_THIS, const char* path)
{
    void* handle;
    int dlopen_flags=RTLD_WORLD | RTLD_GROUP;

    if (this->gl_config.dll_handle!=NULL)
    {
        return 0;
    }

    handle = dlopen(path, dlopen_flags);

    if (handle==NULL)
    {
        SDL_SetError("ph_GL_LoadLibrary(): Could not load OpenGL library");
        return -1;
    }

    this->gl_config.dll_handle = handle;
    this->gl_config.driver_loaded = 1;

    SDL_strlcpy(this->gl_config.driver_path, path, SDL_arraysize(this->gl_config.driver_path));

    return 0;
}
#endif /* 6.3.0 */

#if (_NTO_VERSION < 630)
void* ph_GL_GetProcAddress(_THIS, const char* proc)
{
    return NULL;
}
#else
void* ph_GL_GetProcAddress(_THIS, const char* proc)
{
    void* function;

    if (this->gl_config.dll_handle==NULL)
    {
        ph_GL_LoadLibrary(this, DEFAULT_OPENGL);
        if (this->gl_config.dll_handle==NULL)
        {
            return NULL;
        }
    }
   
    function=qnxgl_get_func(proc, oglctx, 0);
    if (function==NULL)
    {
        function=dlsym(this->gl_config.dll_handle, proc);
    }

    return function;
}
#endif /* 6.3.0 */

#if (_NTO_VERSION < 630)
int ph_GL_MakeCurrent(_THIS)
{
    PgSetRegion(PtWidgetRid(window));

    if (oglctx!=NULL)
    {
        PhDCSetCurrent(oglctx);
    }

    return 0;
}
#else
int ph_GL_MakeCurrent(_THIS)
{
    PgSetRegion(PtWidgetRid(window));

    if (oglctx!=NULL)
    {
        if (qnxgl_set_current(oglctx) == -1)
        {
           return -1;
        }
    }

    return 0;
}
#endif /* 6.3.0 */

#if (_NTO_VERSION < 630)

/* This code is actual for the Photon3D Runtime which was available prior to 6.3 only */

int ph_SetupOpenGLContext(_THIS, int width, int height, int bpp, Uint32 flags)
{
    PhDim_t dim;
    uint64_t OGLAttrib[PH_OGL_MAX_ATTRIBS];
    int exposepost=0;
    int OGLargc;

    dim.w=width;
    dim.h=height;
    
    if ((oglctx!=NULL) && (oglflags==flags) && (oglbpp==bpp))
    {
       PdOpenGLContextResize(oglctx, &dim);
       PhDCSetCurrent(oglctx);
       return 0;
    }
    else
    {
       if (oglctx!=NULL)
       {
          PhDCSetCurrent(NULL);
          PhDCRelease(oglctx);
          oglctx=NULL;
          exposepost=1;
       }
    }

    OGLargc=0;
    if (this->gl_config.depth_size)
    {
        OGLAttrib[OGLargc++]=PHOGL_ATTRIB_DEPTH_BITS;
        OGLAttrib[OGLargc++]=this->gl_config.depth_size;
    }
    if (this->gl_config.stencil_size)
    {
        OGLAttrib[OGLargc++]=PHOGL_ATTRIB_STENCIL_BITS;
        OGLAttrib[OGLargc++]=this->gl_config.stencil_size;
    }
    OGLAttrib[OGLargc++]=PHOGL_ATTRIB_FORCE_SW;
    if (flags & SDL_FULLSCREEN)
    {
        OGLAttrib[OGLargc++]=PHOGL_ATTRIB_FULLSCREEN;
        OGLAttrib[OGLargc++]=PHOGL_ATTRIB_DIRECT;
        OGLAttrib[OGLargc++]=PHOGL_ATTRIB_FULLSCREEN_BEST;
        OGLAttrib[OGLargc++]=PHOGL_ATTRIB_FULLSCREEN_CENTER;
    }
    OGLAttrib[OGLargc++]=PHOGL_ATTRIB_NONE;

    if (this->gl_config.double_buffer)
    {
        oglctx=PdCreateOpenGLContext(2, &dim, 0, OGLAttrib);
    }
    else
    {
        oglctx=PdCreateOpenGLContext(1, &dim, 0, OGLAttrib);
    }

    if (oglctx==NULL)
    {
        SDL_SetError("ph_SetupOpenGLContext(): cannot create OpenGL context !\n");
        return -1;
    }

    PhDCSetCurrent(oglctx);

    PtFlush();

    oglflags=flags;
    oglbpp=bpp;

    if (exposepost!=0)
    {
        /* OpenGL context has been recreated, so report about this fact */
        SDL_PrivateExpose();
    }

    return 0;
}

#else /* _NTO_VERSION */

/* This code is actual for the built-in PhGL support, which became available since 6.3 */

int ph_SetupOpenGLContext(_THIS, int width, int height, int bpp, Uint32 flags)
{
    qnxgl_buf_attrib_t qnxgl_attribs[PH_OGL_MAX_ATTRIBS];
    qnxgl_buf_attrib_t* qnxgl_attribs_slide;
    int num_interfaces = 0;
    int num_buffers = 0;

    /* Initialize the OpenGL subsystem */

    num_interfaces = qnxgl_init(NULL, NULL, 0);

    if (num_interfaces < 0)
    {
        SDL_SetError("ph_SetupOpenGLContext(): cannot initialize OpenGL subsystem !\n");
        return -1;
    }
    if (num_interfaces == 0)
    {
        SDL_SetError("ph_SetupOpenGLContext(): there are no available OpenGL renderers was found !\n");
        return -1;
    }

    /* Driver is linked */
    this->gl_config.driver_loaded=1;

    /* Initialize the OpenGL context attributes */
    qnxgl_attribs_slide=qnxgl_attribs;

    /* Depth size */
    if (this->gl_config.depth_size)
    {
        fprintf(stderr, "setted depth size %d\n", this->gl_config.depth_size);
        qnxgl_attribs_slide = qnxgl_attrib_set_depth(qnxgl_attribs_slide, this->gl_config.depth_size);
    }

    /* Stencil size */
    if (this->gl_config.stencil_size)
    {
        qnxgl_attribs_slide = qnxgl_attrib_set_stencil(qnxgl_attribs_slide, this->gl_config.stencil_size);
    }

    /* The sum of the accum bits of each channel */
    if ((this->gl_config.accum_red_size != 0) && (this->gl_config.accum_blue_size != 0) &&
        (this->gl_config.accum_green_size != 0))
    {
        qnxgl_attribs_slide = qnxgl_attrib_set_accum(qnxgl_attribs_slide,
           this->gl_config.accum_red_size + this->gl_config.accum_blue_size +
           this->gl_config.accum_green_size + this->gl_config.accum_alpha_size);
    }
    
    /* Stereo mode */
    if (this->gl_config.stereo)
    {
        qnxgl_attribs_slide = qnxgl_attrib_set_stereo(qnxgl_attribs_slide);
    }

    /* Fullscreen mode */
    if ((flags & SDL_FULLSCREEN) == SDL_FULLSCREEN)
    {
        qnxgl_attribs_slide = qnxgl_attrib_set_hint_fullscreen(qnxgl_attribs_slide);
    }
    
    /* Double buffering mode */
    if (this->gl_config.double_buffer)
    {
        num_buffers=2;
    }
    else
    {
        num_buffers=1;
    }

    /* Loading the function pointers so we can use the extensions */
    GLPH_LOAD_FUNCS_GC(oglctx);

    /* Set the buffers region to be that of our window's region */
    qnxgl_attribs_slide = glph_attrib_set_region(qnxgl_attribs_slide, PtWidgetRid(window));

    /* End of the attributes array */
    qnxgl_attribs_slide = qnxgl_attrib_set_end(qnxgl_attribs_slide);
    
    /* Create the buffers with the specified color model */
    fprintf(stderr, "ARGB: %d, %d, %d, %d\n", this->gl_config.alpha_size, this->gl_config.red_size, this->gl_config.green_size, this->gl_config.blue_size);
    oglbuffers = qnxgl_buffers_create(
                   QNXGL_FORMAT_BEST_RGB,
/*                 __QNXGL_BUILD_FORMAT(0, __QNXGL_COLOR_MODEL_RGB, this->gl_config.alpha_size,
                     this->gl_config.red_size, this->gl_config.green_size, this->gl_config.blue_size), */
                 num_buffers, width, height, qnxgl_attribs, -1);


    if (oglbuffers == NULL)
    {
        SDL_SetError("ph_SetupOpenGLContext(): failed to create OpenGL buffers !\n");
        qnxgl_finish();
        return -1;
    }

    /* Create a QNXGL context for the previously created buffer */
    oglctx = qnxgl_context_create(oglbuffers, NULL);

    if (oglctx == NULL)
    {
        SDL_SetError("ph_SetupOpenGLContext(): failed to create OpenGL context !\n");
        qnxgl_buffers_destroy(oglbuffers);
        qnxgl_finish();
        return -1;
    }

    /* Attempt to make the context current so we can use OpenGL commands */
    if (qnxgl_set_current(oglctx) == -1)
    {
        SDL_SetError("ph_SetupOpenGLContext(): failed to make the OpenGL context current !\n");
        qnxgl_context_destroy(oglctx);
        qnxgl_buffers_destroy(oglbuffers);
        qnxgl_finish();
        return -1;
    }

    PtFlush();

    oglflags=flags;
    oglbpp=bpp;

    return 0;
}

#endif /* _NTO_VERSION */

#endif /* SDL_VIDEO_OPENGL */
