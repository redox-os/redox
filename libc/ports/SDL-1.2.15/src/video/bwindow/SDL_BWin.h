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

#ifndef _SDL_BWin_h
#define _SDL_BWin_h

#include "SDL_config.h"

#include <stdio.h>
#include <AppKit.h>
#include <InterfaceKit.h>
#include <be/game/DirectWindow.h>
#if SDL_VIDEO_OPENGL
#include "SDL_opengl.h"
#include <be/opengl/GLView.h>
#endif
#include <support/UTF8.h>

#include "../../main/beos/SDL_BeApp.h"
#include "SDL_events.h"
#include "SDL_BView.h"

extern "C" {
#include "../../events/SDL_events_c.h"

extern int mouse_relative;
};

class SDL_BWin : public BDirectWindow
{
public:
	SDL_BWin(BRect bounds) :
			BDirectWindow(bounds, "Untitled", B_TITLED_WINDOW, 0) {
		last_buttons = 0;
		the_view = NULL;
#if SDL_VIDEO_OPENGL
		SDL_GLView = NULL;
#endif
		SDL_View = NULL;
		Unlock();
		shown = false;
		inhibit_resize = false;
	}

	virtual ~SDL_BWin() {
		Lock();
		if ( the_view ) {
#if SDL_VIDEO_OPENGL
			if ( the_view == SDL_GLView ) {
				SDL_GLView->UnlockGL();
			}
#endif
			RemoveChild(the_view);
			the_view = NULL;
		}
		Unlock();
#if SDL_VIDEO_OPENGL
		if ( SDL_GLView ) {
			delete SDL_GLView;
		}
#endif
		if ( SDL_View ) {
			delete SDL_View;
		}
	}
	

	/* Override the Show() method so we can tell when we've been shown */
	virtual void Show(void) {
		BWindow::Show();
		shown = true;
	}
	virtual bool Shown(void) {
		return (shown);
	}
	/* If called, the next resize event will not be forwarded to SDL. */
	virtual void InhibitResize(void) {
		inhibit_resize=true;
	}
	/* Handle resizing of the window */
	virtual void FrameResized(float width, float height) {
		if(inhibit_resize)
			inhibit_resize = false;
		else 
			SDL_PrivateResize((int)width, (int)height);
	}
	virtual int CreateView(Uint32 flags, Uint32 gl_flags) {
		int retval;

		retval = 0;
		Lock();
		if ( flags & SDL_OPENGL ) {
#if SDL_VIDEO_OPENGL
			if ( SDL_GLView == NULL ) {
				SDL_GLView = new BGLView(Bounds(), "SDL GLView",
					 	B_FOLLOW_ALL_SIDES, (B_WILL_DRAW|B_FRAME_EVENTS),
					 	gl_flags|BGL_DOUBLE);
				SDL_GLView->EnableDirectMode(true);
			}
			if ( the_view != SDL_GLView ) {
				if ( the_view ) {
					RemoveChild(the_view);
				}
				AddChild(SDL_GLView);
				SDL_GLView->LockGL();
				the_view = SDL_GLView;
			}
#else
			SDL_SetError("OpenGL support not enabled");
			retval = -1;
#endif
		} else {
			if ( SDL_View == NULL ) {
				SDL_View = new SDL_BView(Bounds());
			}
			if ( the_view != SDL_View ) {
				if ( the_view ) {
					RemoveChild(the_view);
				}
				AddChild(SDL_View);
				the_view = SDL_View;
			}
		}
#if SDL_VIDEO_OPENGL
		if ( the_view == SDL_GLView ) {
			SDL_GLView->UnlockGL();
		}
#endif
		Unlock();
		return(retval);
	}
	virtual void SetBitmap(BBitmap *bitmap) {
		SDL_View->SetBitmap(bitmap);
	}
	virtual void SetXYOffset(int x, int y) {
#if SDL_VIDEO_OPENGL
		if ( the_view == SDL_GLView ) {
			return;
		}
#endif
		SDL_View->SetXYOffset(x, y);
	}
	virtual void GetXYOffset(int &x, int &y) {
#if SDL_VIDEO_OPENGL
		if ( the_view == SDL_GLView ) {
			x = 0;
			y = 0;
			return;
		}
#endif
		SDL_View->GetXYOffset(x, y);
	}
	virtual void GetXYOffset(float &x, float &y) {
#if SDL_VIDEO_OPENGL
		if ( the_view == SDL_GLView ) {
			x = 0.0f;
			y = 0.0f;
			return;
		}
#endif
		SDL_View->GetXYOffset(x, y);
	}
	virtual bool BeginDraw(void) {
		return(Lock());
	}
	virtual void DrawAsync(BRect updateRect) {
		SDL_View->DrawAsync(updateRect);
	}
	virtual void EndDraw(void) {
		SDL_View->Sync();
		Unlock();
	}
#if SDL_VIDEO_OPENGL
	virtual void SwapBuffers(void) {
		SDL_GLView->UnlockGL();
		SDL_GLView->SwapBuffers();
		SDL_GLView->LockGL();
	}
#endif
	virtual BView *View(void) {
		return(the_view);
	}

	/* Hook functions -- overridden */
	virtual void Minimize(bool minimize) {
		/* This is only called when mimimized, not when restored */
		//SDL_PrivateAppActive(minimize, SDL_APPACTIVE);
		BWindow::Minimize(minimize);
	}
	virtual void WindowActivated(bool active) {
		SDL_PrivateAppActive(active, SDL_APPINPUTFOCUS);
	}
	virtual bool QuitRequested(void) {
		if ( SDL_BeAppActive > 0 ) {
			SDL_PrivateQuit();
			/* We don't ever actually close the window here because
			   the application should respond to the quit request,
			   or ignore it as desired.
			 */
#if SDL_VIDEO_OPENGL
			if ( SDL_GLView != NULL ) {
				SDL_GLView->EnableDirectMode(false);
			}
#endif
			return(false);
		}
		return(true);	/* Close the app window */
	}
	virtual void Quit() {
		if (!IsLocked())
			Lock();
		BDirectWindow::Quit();
	}

	virtual int16 Translate2Unicode(const char *buf) {
		int32 state, srclen, dstlen;
		unsigned char destbuf[2];
		Uint16 unicode = 0;

		if ((uchar)buf[0] > 127) {
			state = 0;
			srclen = SDL_strlen(buf);
			dstlen = sizeof(destbuf);
			convert_from_utf8(B_UNICODE_CONVERSION, buf, &srclen, (char *)destbuf, &dstlen, &state);
			unicode = destbuf[0];
			unicode <<= 8;
			unicode |= destbuf[1];
		} else
			unicode = buf[0];

		/* For some reason function keys map to control characters */
# define CTRL(X)	((X)-'@')
		switch (unicode) {
		    case CTRL('A'):
		    case CTRL('B'):
		    case CTRL('C'):
		    case CTRL('D'):
		    case CTRL('E'):
		    case CTRL('K'):
		    case CTRL('L'):
		    case CTRL('P'):
			if ( ! (SDL_GetModState() & KMOD_CTRL) )
				unicode = 0;
			break;
			/* Keyboard input maps newline to carriage return */
			case '\n':
				unicode = '\r';
			break;
		    default:
			break;
		}

		return unicode;
	}

	virtual void DispatchMessage(BMessage *msg, BHandler *target);
	
	virtual void DirectConnected(direct_buffer_info *info);

private:
#if SDL_VIDEO_OPENGL
	BGLView *SDL_GLView;
#endif
	SDL_BView *SDL_View;
	BView *the_view;
	bool shown;
	bool inhibit_resize;
	int32 last_buttons;
};

#endif /* _SDL_BWin_h */
