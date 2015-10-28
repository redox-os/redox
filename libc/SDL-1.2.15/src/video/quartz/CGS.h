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
    Obscuring code: maximum number of windows above ours (inclusive) 
    
    Note: this doesn't work too well in practice and should be
    phased out when we add OpenGL 2D acceleration. It was never
    enabled in the first place, so this shouldn't be a problem ;-)
*/
#define kMaxWindows 256

/* Some of the Core Graphics Server API for obscuring code */
#define kCGSWindowLevelTop          2147483632
#define kCGSWindowLevelDockIconDrag 500
#define kCGSWindowLevelDockMenu     101
#define kCGSWindowLevelMenuIgnore    21
#define kCGSWindowLevelMenu          20
#define kCGSWindowLevelDockLabel     12
#define kCGSWindowLevelDockIcon      11
#define kCGSWindowLevelDock          10
#define kCGSWindowLevelUtility        3
#define kCGSWindowLevelNormal         0

/* 
    For completeness; We never use these window levels, they are always below us
    #define kCGSWindowLevelMBarShadow -20
    #define kCGSWindowLevelDesktopPicture -2147483647
    #define kCGSWindowLevelDesktop        -2147483648
*/

typedef CGError       CGSError;
typedef long          CGSWindowCount;
typedef void *        CGSConnectionID;
typedef int           CGSWindowID;
typedef CGSWindowID*  CGSWindowIDList;
typedef CGWindowLevel CGSWindowLevel;
typedef NSRect        CGSRect;

extern CGSConnectionID _CGSDefaultConnection ();

extern CGSError CGSGetOnScreenWindowList (CGSConnectionID cid,
                                          CGSConnectionID owner,
                                          CGSWindowCount listCapacity,
                                          CGSWindowIDList list,
                                          CGSWindowCount *listCount);

extern CGSError CGSGetScreenRectForWindow (CGSConnectionID cid,
                                           CGSWindowID wid,
                                           CGSRect *rect);

extern CGWindowLevel CGSGetWindowLevel (CGSConnectionID cid,
                                        CGSWindowID wid,
                                        CGSWindowLevel *level);

extern CGSError CGSDisplayHWFill (CGDirectDisplayID id, unsigned int x, unsigned int y,
                                  unsigned int w, unsigned int h, unsigned int color);

extern CGSError CGSDisplayCanHWFill (CGDirectDisplayID id);

extern CGSError CGSGetMouseEnabledFlags (CGSConnectionID cid, CGSWindowID wid, int *flags);

int CGSDisplayHWSync (CGDirectDisplayID id);

