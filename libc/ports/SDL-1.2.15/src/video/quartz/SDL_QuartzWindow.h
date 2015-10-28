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

#if (MAC_OS_X_VERSION_MAX_ALLOWED < 1050)
typedef unsigned int NSUInteger;
#endif

/* Subclass of NSWindow to fix genie effect and support resize events  */
@interface SDL_QuartzWindow : NSWindow
{
	BOOL watchForMouseUp;
}

- (void)miniaturize:(id)sender;
- (void)display;
- (void)setFrame:(NSRect)frameRect display:(BOOL)flag;
- (void)appDidHide:(NSNotification*)note;
- (void)appWillUnhide:(NSNotification*)note;
- (void)appDidUnhide:(NSNotification*)note;
- (id)initWithContentRect:(NSRect)contentRect styleMask:(NSUInteger)styleMask backing:(NSBackingStoreType)backingType defer:(BOOL)flag;
@end

/* Delegate for our NSWindow to send SDLQuit() on close */
@interface SDL_QuartzWindowDelegate : NSObject
- (BOOL)windowShouldClose:(id)sender;
@end

/* Subclass of NSView to set cursor rectangle */
@interface SDL_QuartzView : NSView
- (void)resetCursorRects;
@end
