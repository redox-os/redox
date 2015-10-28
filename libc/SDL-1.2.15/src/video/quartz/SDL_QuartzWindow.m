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

#include "SDL_QuartzVideo.h"
#include "SDL_QuartzWM.h"
#include "SDL_QuartzWindow.h"

/*
    This function makes the *SDL region* of the window 100% opaque. 
    The genie effect uses the alpha component. Otherwise,
    it doesn't seem to matter what value it has.
*/
static void QZ_SetPortAlphaOpaque () {
    
    SDL_Surface *surface = current_video->screen;
    int bpp;
    
    bpp = surface->format->BitsPerPixel;
    
    if (bpp == 32) {
    
        Uint32    *pixels = (Uint32*) surface->pixels;
        Uint32    rowPixels = surface->pitch / 4;
        Uint32    i, j;
        
        for (i = 0; i < surface->h; i++)
            for (j = 0; j < surface->w; j++) {
        
                pixels[ (i * rowPixels) + j ] |= 0xFF000000;
            }
    }
}

@implementation SDL_QuartzWindow

/* we override these methods to fix the miniaturize animation/dock icon bug */
- (void)miniaturize:(id)sender
{
    if (SDL_VideoSurface->flags & SDL_OPENGL) {
    
        /* 
            Future: Grab framebuffer and put into NSImage
            [ qz_window setMiniwindowImage:image ];
        */
    }
    else {
        
        /* make the alpha channel opaque so anim won't have holes in it */
        QZ_SetPortAlphaOpaque ();
    }
    
    /* window is hidden now */
    SDL_PrivateAppActive (0, SDL_APPACTIVE);
    
    [ super miniaturize:sender ];
}

- (void)display
{    
    /* 
        This method fires just before the window deminaturizes from the Dock.
        
        We'll save the current visible surface, let the window manager redraw any
        UI elements, and restore the SDL surface. This way, no expose event 
        is required, and the deminiaturize works perfectly.
    */
     SDL_VideoDevice *this = (SDL_VideoDevice*)current_video;
    
    /* make sure pixels are fully opaque */
    if (! ( SDL_VideoSurface->flags & SDL_OPENGL ) )
        QZ_SetPortAlphaOpaque ();
    
    /* save current visible SDL surface */
    [ self cacheImageInRect:[ window_view frame ] ];
    
    /* let the window manager redraw controls, border, etc */
    [ super display ];
    
    /* restore visible SDL surface */
    [ self restoreCachedImage ];
    
    /* window is visible again */
    SDL_PrivateAppActive (1, SDL_APPACTIVE);
}

- (void)setFrame:(NSRect)frameRect display:(BOOL)flag
{

    /*
        If the video surface is NULL, this originated from QZ_SetVideoMode,
        so don't send the resize event. 
    */
    SDL_VideoDevice *this = (SDL_VideoDevice*)current_video;
    
    if (this && SDL_VideoSurface == NULL) {

        [ super setFrame:frameRect display:flag ];
    }
    else if (this && qz_window) {

        NSRect newViewFrame;
        
        [ super setFrame:frameRect display:flag ];
        
        newViewFrame = [ window_view frame ];
        
        SDL_PrivateResize (newViewFrame.size.width, newViewFrame.size.height);
    }
}

/* QZ_DoActivate() calls a low-level CoreGraphics routine to adjust
   the cursor position, if input is being grabbed. If app activation is
   triggered by a mouse click in the title bar, then the window manager
   gets confused and thinks we're dragging the window. The solution
   below postpones the activate event to avoid this scenario. */
- (void)becomeKeyWindow
{
	NSEvent *event = [self currentEvent];
	if ([event type] == NSLeftMouseDown && [event window] == self)
		watchForMouseUp = YES;
	else
		[super becomeKeyWindow];
}

- (void)sendEvent:(NSEvent *)event
{
	[super sendEvent:event];
	if (watchForMouseUp && [event type] == NSLeftMouseUp)
	{
		watchForMouseUp = NO;
		[super becomeKeyWindow];
	}
}

- (void)appDidHide:(NSNotification*)note
{
    SDL_PrivateAppActive (0, SDL_APPACTIVE);
}

- (void)appWillUnhide:(NSNotification*)note
{
    SDL_VideoDevice *this = (SDL_VideoDevice*)current_video;
    
    if ( this ) {
    
        /* make sure pixels are fully opaque */
        if (! ( SDL_VideoSurface->flags & SDL_OPENGL ) )
            QZ_SetPortAlphaOpaque ();
          
        /* save current visible SDL surface */
        [ self cacheImageInRect:[ window_view frame ] ];
    }
}

- (void)appDidUnhide:(NSNotification*)note
{
    /* restore cached image, since it may not be current, post expose event too */
    [ self restoreCachedImage ];
    
    /*SDL_PrivateExpose ();*/
    
    SDL_PrivateAppActive (1, SDL_APPACTIVE);
}

- (id)initWithContentRect:(NSRect)contentRect styleMask:(NSUInteger)styleMask backing:(NSBackingStoreType)backingType defer:(BOOL)flag
{
    /* Make our window subclass receive these application notifications */
    [ [ NSNotificationCenter defaultCenter ] addObserver:self
        selector:@selector(appDidHide:) name:NSApplicationDidHideNotification object:NSApp ];
    
    [ [ NSNotificationCenter defaultCenter ] addObserver:self
        selector:@selector(appDidUnhide:) name:NSApplicationDidUnhideNotification object:NSApp ];
   
    [ [ NSNotificationCenter defaultCenter ] addObserver:self
        selector:@selector(appWillUnhide:) name:NSApplicationWillUnhideNotification object:NSApp ];
        
    return [ super initWithContentRect:contentRect styleMask:styleMask backing:backingType defer:flag ];
}

@end

@implementation SDL_QuartzWindowDelegate
- (BOOL)windowShouldClose:(id)sender
{
    SDL_PrivateQuit();
    return NO;
}

- (void)windowDidBecomeKey:(NSNotification *)aNotification
{
    QZ_DoActivate (current_video);
}

- (void)windowDidResignKey:(NSNotification *)aNotification
{
    QZ_DoDeactivate (current_video);
}

@end

@implementation SDL_QuartzView

- (void)resetCursorRects
{
    SDL_Cursor *sdlc = SDL_GetCursor();
    if (sdlc != NULL && sdlc->wm_cursor != NULL) {
        [self addCursorRect: [self visibleRect] cursor: sdlc->wm_cursor->nscursor];
    }
}

@end
