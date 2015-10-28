/*
    SDL - Simple DirectMedia Layer
    Copyright (C) 1997-2012  Sam Lantinga

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

/*    
    @file   SDL_QuartzVideo.h
    @author Darrell Walisser, Max Horn, et al.
    
    @abstract SDL video driver for Mac OS X.
    
    @discussion
    
    TODO
        - Hardware Cursor support with NSCursor instead of Carbon
        - Keyboard repeat/mouse speed adjust (if needed)
        - Multiple monitor support (currently only main display)
        - Accelerated blitting support
        - Fix white OpenGL window on minimize (fixed) (update: broken again on 10.2)
        - Find out what events should be sent/ignored if window is minimized
        - Find a way to deal with external resolution/depth switch while app is running
        - Check accuracy of QZ_SetGamma()
    Problems:
        - OGL not working in full screen with software renderer
        - SetColors sets palette correctly but clears framebuffer
        - Crash in CG after several mode switches (I think this has been fixed)
        - Retained windows don't draw their title bar quite right (OS Bug) (not using retained windows)
        - Cursor in 8 bit modes is screwy (might just be Radeon PCI bug) (update: not just Radeon)
        - Warping cursor delays mouse events for a fraction of a second,
          there is a hack around this that helps a bit
*/

/* Needs to be first, so QuickTime.h doesn't include glext.h (10.4) */
#include "SDL_opengl.h"

#include <Cocoa/Cocoa.h>
#include <Carbon/Carbon.h>
#include <OpenGL/OpenGL.h>	/* For CGL functions and types */
#include <IOKit/IOKitLib.h>	/* For powersave handling */
#include <pthread.h>

#include "SDL_thread.h"
#include "SDL_video.h"
#include "SDL_error.h"
#include "SDL_timer.h"
#include "SDL_loadso.h"
#include "SDL_syswm.h"
#include "../SDL_sysvideo.h"
#include "../SDL_pixels_c.h"
#include "../../events/SDL_events_c.h"


#ifdef __powerpc__
/* 
    This is a workaround to directly access NSOpenGLContext's CGL context
    We need this to check for errors NSOpenGLContext doesn't support
    Please note this is only used on PowerPC (Intel Macs are guaranteed to
    have a better API for this, since it showed up in Mac OS X 10.3).
*/
@interface NSOpenGLContext (CGLContextAccess)
- (CGLContextObj) cglContext;
@end
#endif

/* use this to get the CGLContext; it handles Cocoa interface changes. */
CGLContextObj QZ_GetCGLContextObj(NSOpenGLContext *nsctx);


/* Main driver structure to store required state information */
typedef struct SDL_PrivateVideoData {
    BOOL               use_new_mode_apis;  /* 1 == >= 10.6 APIs available */
    BOOL               allow_screensaver;  /* 0 == disable screensaver */
    CGDirectDisplayID  display;            /* 0 == main display (only support single display) */
    const void         *mode;              /* current mode of the display */
    const void         *save_mode;         /* original mode of the display */
    CGDirectPaletteRef palette;            /* palette of an 8-bit display */
    NSOpenGLContext    *gl_context;        /* OpenGL rendering context */
    NSGraphicsContext  *nsgfx_context;     /* Cocoa graphics context */
    Uint32             width, height, bpp; /* frequently used data about the display */
    Uint32             flags;              /* flags for current mode, for teardown purposes */
    Uint32             video_set;          /* boolean; indicates if video was set correctly */
    Uint32             warp_flag;          /* boolean; notify to event loop that a warp just occured */
    Uint32             warp_ticks;         /* timestamp when the warp occured */
    NSWindow           *window;            /* Cocoa window to implement the SDL window */
    NSView             *view;              /* the window's view; draw 2D and OpenGL into this view */
    CGContextRef       cg_context;         /* CoreGraphics rendering context */
    SDL_Surface        *resize_icon;       /* icon for the resize badge, we have to draw it by hand */
    SDL_GrabMode       current_grab_mode;  /* default value is SDL_GRAB_OFF */
    SDL_Rect           **client_mode_list; /* resolution list to pass back to client */
    SDLKey             keymap[256];        /* Mac OS X to SDL key mapping */
    Uint32             current_mods;       /* current keyboard modifiers, to track modifier state */
    NSText             *field_edit;        /* a field editor for keyboard composition processing */
    Uint32             last_virtual_button;/* last virtual mouse button pressed */
    io_connect_t       power_connection;   /* used with IOKit to detect wake from sleep */
    Uint8              expect_mouse_up;    /* used to determine when to send mouse up events */
    Uint8              grab_state;         /* used to manage grab behavior */
    NSPoint            cursor_loc;         /* saved cursor coords, for activate/deactivate when grabbed */
    BOOL               cursor_should_be_visible;     /* tells if cursor is supposed to be visible (SDL_ShowCursor) */
    BOOL               cursor_visible;     /* tells if cursor is *actually* visible or not */
    Uint8*             sw_buffers[2];      /* pointers to the two software buffers for double-buffer emulation */
    SDL_Thread         *thread;            /* thread for async updates to the screen */
    SDL_sem            *sem1, *sem2;       /* synchronization for async screen updates */
    Uint8              *current_buffer;    /* the buffer being copied to the screen */
    BOOL               quit_thread;        /* used to quit the async blitting thread */
    SInt32             system_version;     /* used to dis-/enable workarounds depending on the system version */

    void *opengl_library;    /* dynamically loaded OpenGL library. */
} SDL_PrivateVideoData;

#define _THIS    SDL_VideoDevice *this
#define display_id (this->hidden->display)
#define mode (this->hidden->mode)
#define save_mode (this->hidden->save_mode)
#define use_new_mode_apis (this->hidden->use_new_mode_apis)
#define allow_screensaver (this->hidden->allow_screensaver)
#define palette (this->hidden->palette)
#define gl_context (this->hidden->gl_context)
#define nsgfx_context (this->hidden->nsgfx_context)
#define device_width (this->hidden->width)
#define device_height (this->hidden->height)
#define device_bpp (this->hidden->bpp)
#define mode_flags (this->hidden->flags)
#define qz_window (this->hidden->window)
#define window_view (this->hidden->view)
#define cg_context (this->hidden->cg_context)
#define video_set (this->hidden->video_set)
#define warp_ticks (this->hidden->warp_ticks)
#define warp_flag (this->hidden->warp_flag)
#define resize_icon (this->hidden->resize_icon)
#define current_grab_mode (this->hidden->current_grab_mode)
#define client_mode_list (this->hidden->client_mode_list)
#define keymap (this->hidden->keymap)
#define current_mods (this->hidden->current_mods)
#define field_edit (this->hidden->field_edit)
#define last_virtual_button (this->hidden->last_virtual_button)
#define power_connection (this->hidden->power_connection)
#define expect_mouse_up (this->hidden->expect_mouse_up)
#define grab_state (this->hidden->grab_state)
#define cursor_loc (this->hidden->cursor_loc)
#define cursor_should_be_visible (this->hidden->cursor_should_be_visible)
#define cursor_visible (this->hidden->cursor_visible)
#define sw_buffers (this->hidden->sw_buffers)
#define sw_contexts (this->hidden->sw_contexts)
#define thread (this->hidden->thread)
#define sem1 (this->hidden->sem1)
#define sem2 (this->hidden->sem2)
#define current_buffer (this->hidden->current_buffer)
#define quit_thread (this->hidden->quit_thread)
#define system_version (this->hidden->system_version)
#define opengl_library (this->hidden->opengl_library)

/* grab states - the input is in one of these states */
enum {
    QZ_UNGRABBED = 0,
    QZ_VISIBLE_GRAB,
    QZ_INVISIBLE_GRAB
};

/* grab actions - these can change the grabbed state */
enum {
    QZ_ENABLE_GRAB = 0,
    QZ_DISABLE_GRAB,
    QZ_HIDECURSOR,
    QZ_SHOWCURSOR
};

/* Gamma Functions */
int    QZ_SetGamma          (_THIS, float red, float green, float blue);
int    QZ_GetGamma          (_THIS, float *red, float *green, float *blue);
int    QZ_SetGammaRamp      (_THIS, Uint16 *ramp);
int    QZ_GetGammaRamp      (_THIS, Uint16 *ramp);

/* OpenGL functions */
int    QZ_SetupOpenGL       (_THIS, int bpp, Uint32 flags);
void   QZ_TearDownOpenGL    (_THIS);
void*  QZ_GL_GetProcAddress (_THIS, const char *proc);
int    QZ_GL_GetAttribute   (_THIS, SDL_GLattr attrib, int* value);
int    QZ_GL_MakeCurrent    (_THIS);
void   QZ_GL_SwapBuffers    (_THIS);
int    QZ_GL_LoadLibrary    (_THIS, const char *location);

/* Cursor and Mouse functions */
void         QZ_FreeWMCursor     (_THIS, WMcursor *cursor);
WMcursor*    QZ_CreateWMCursor   (_THIS, Uint8 *data, Uint8 *mask,
                                  int w, int h, int hot_x, int hot_y);
int          QZ_ShowWMCursor     (_THIS, WMcursor *cursor);
void         QZ_WarpWMCursor     (_THIS, Uint16 x, Uint16 y);
void         QZ_MoveWMCursor     (_THIS, int x, int y);
void         QZ_CheckMouseMode   (_THIS);
void         QZ_UpdateMouse      (_THIS);

/* Event functions */
void         QZ_InitOSKeymap     (_THIS);
void         QZ_PumpEvents       (_THIS);

/* Window Manager functions */
void         QZ_SetCaption       (_THIS, const char *title, const char *icon);
void         QZ_SetIcon          (_THIS, SDL_Surface *icon, Uint8 *mask);
int          QZ_IconifyWindow    (_THIS);
SDL_GrabMode QZ_GrabInput        (_THIS, SDL_GrabMode grab_mode);
/*int          QZ_GetWMInfo        (_THIS, SDL_SysWMinfo *info);*/

/* Private functions (used internally) */
void         QZ_PrivateWarpCursor (_THIS, int x, int y);
void         QZ_ChangeGrabState (_THIS, int action);
void         QZ_RegisterForSleepNotifications (_THIS);
void         QZ_PrivateGlobalToLocal (_THIS, NSPoint *p);
void         QZ_PrivateCocoaToSDL (_THIS, NSPoint *p);
BOOL         QZ_IsMouseInWindow (_THIS);
void         QZ_DoActivate (_THIS);
void         QZ_DoDeactivate (_THIS);
