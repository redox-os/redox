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
    slouken@devolution.com
*/

#ifndef EPOCVIDEO_H
#define EPOCVIDEO_H

#include<w32std.h>

/* Hidden "this" pointer for the video functions */
#define _THIS	SDL_VideoDevice *_this
#define Private	_this->hidden

class CFbsBitmap;

struct SDL_VideoDevice;
void DisableKeyBlocking(SDL_VideoDevice*);

struct SDL_PrivateVideoData 
    {
    TPoint					iScreenPos;
    TBool                   iIsWindowFocused;
    TSize                   iSwSurfaceSize;
    TUint8*                 iSwSurface;
    SDL_Rect				iRect; //same info in SDL format
    SDL_Rect* 				iRectPtr;
    };
    
#endif    



    
