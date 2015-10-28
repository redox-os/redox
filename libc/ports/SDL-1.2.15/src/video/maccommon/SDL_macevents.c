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

#include <stdio.h>

#if defined(__APPLE__) && defined(__MACH__)
#include <Carbon/Carbon.h>
#elif TARGET_API_MAC_CARBON && (UNIVERSAL_INTERFACES_VERSION > 0x0335)
#include <Carbon.h>
#else
#include <Script.h>
#include <LowMem.h>
#include <Devices.h>
#include <DiskInit.h>
#include <ToolUtils.h>
#endif

#include "SDL_events.h"
#include "SDL_video.h"
#include "SDL_syswm.h"
#include "../../events/SDL_events_c.h"
#include "../../events/SDL_sysevents.h"
#include "../SDL_cursor_c.h"
#include "SDL_macevents_c.h"
#include "SDL_mackeys.h"
#include "SDL_macmouse_c.h"

/* Define this to be able to collapse SDL windows.
#define USE_APPEARANCE_MANAGER
 */

/* Macintosh resource constants */
#define mApple	128			/* Apple menu resource */
#define iAbout	1			/* About menu item */

/* Functions to handle the About menu */
static void Mac_DoAppleMenu(_THIS, long item);

/* The translation table from a macintosh key scancode to a SDL keysym */
static SDLKey MAC_keymap[256];
static SDL_keysym *TranslateKey(int scancode, int modifiers,
                                SDL_keysym *keysym, int pressed);

/* Handle activation and deactivation  -- returns whether an event was posted */
static int Mac_HandleActivate(int activate)
{
	if ( activate ) {
		/* Show the current SDL application cursor */
		SDL_SetCursor(NULL);

		/* put our mask back case it changed during context switch */
		SetEventMask(everyEvent & ~autoKeyMask);
	} else {
#if TARGET_API_MAC_CARBON
		{ Cursor cursor;
			SetCursor(GetQDGlobalsArrow(&cursor));
		}
#else
		SetCursor(&theQD->arrow);
#endif
		if ( ! Mac_cursor_showing ) {
			ShowCursor();
			Mac_cursor_showing = 1;
		}
	}
	return(SDL_PrivateAppActive(activate, SDL_APPINPUTFOCUS));
}

static void myGlobalToLocal(_THIS, Point *pt)
{
	if ( SDL_VideoSurface && !(SDL_VideoSurface->flags&SDL_FULLSCREEN) ) {
		GrafPtr saveport;
		GetPort(&saveport);
#if TARGET_API_MAC_CARBON
		SetPort(GetWindowPort(SDL_Window));
#else
		SetPort(SDL_Window);
#endif
		GlobalToLocal(pt);
		SetPort(saveport);
	}
}

/* The main MacOS event handler */
static int Mac_HandleEvents(_THIS, int wait4it)
{
	static int mouse_button = 1;
	int i;
	EventRecord event;

#if TARGET_API_MAC_CARBON
	/* There's no GetOSEvent() in the Carbon API. *sigh* */
#define cooperative_multitasking 1
#else
	int cooperative_multitasking;
	/* If we're running fullscreen, we can hog the MacOS events,
	   otherwise we had better play nicely with the other apps.
	*/
	if ( this->screen && (this->screen->flags & SDL_FULLSCREEN) ) {
		cooperative_multitasking = 0;
	} else {
		cooperative_multitasking = 1;
	}
#endif

	/* If we call WaitNextEvent(), MacOS will check other processes
	 * and allow them to run, and perform other high-level processing.
	 */
	if ( cooperative_multitasking || wait4it ) {
		UInt32 wait_time;

		/* Are we polling or not? */
		if ( wait4it ) {
			wait_time = 2147483647;
		} else {
			wait_time = 0;
		}
		WaitNextEvent(everyEvent, &event, wait_time, nil);
	} else {
#if ! TARGET_API_MAC_CARBON
		GetOSEvent(everyEvent, &event);
#endif
	}

#if TARGET_API_MAC_CARBON
	/* for some reason, event.where isn't set ? */
	GetGlobalMouse ( &event.where );
#endif

	/* Check for mouse motion */
	if ( (event.where.h != last_where.h) ||
	     (event.where.v != last_where.v) ) {
		Point pt;
		pt = last_where = event.where;
		myGlobalToLocal(this, &pt);
		SDL_PrivateMouseMotion(0, 0, pt.h, pt.v);
	}

	/* Check the current state of the keyboard */
	if ( SDL_GetAppState() & SDL_APPINPUTFOCUS ) {
		KeyMap keys;
		const Uint32 *keysptr = (Uint32 *) &keys;
		const Uint32 *last_keysptr = (Uint32 *) &last_keys;

		/* Check for special non-event keys */
		if ( event.modifiers != last_mods ) {
			static struct {
				EventModifiers mask;
				SDLKey key;
			} mods[] = {
				{ alphaLock,		SDLK_CAPSLOCK },
#if 0 /* These are handled below in the GetKeys() code */
				{ cmdKey,		SDLK_LMETA },
				{ shiftKey,		SDLK_LSHIFT },
				{ rightShiftKey,	SDLK_RSHIFT },
				{ optionKey,		SDLK_LALT },
				{ rightOptionKey,	SDLK_RALT },
				{ controlKey,		SDLK_LCTRL },
				{ rightControlKey,	SDLK_RCTRL },
#endif /* 0 */
				{ 0,			0 }
			};
			SDL_keysym keysym;
			Uint8 mode;
			EventModifiers mod, mask;
		

			/* Set up the keyboard event */
			keysym.scancode = 0;
			keysym.sym = SDLK_UNKNOWN;
			keysym.mod = KMOD_NONE;
			keysym.unicode = 0;

			/* See what has changed, and generate events */
			mod = event.modifiers;
			for ( i=0; mods[i].mask; ++i ) {
				mask = mods[i].mask;
				if ( (mod&mask) != (last_mods&mask) ) {
					keysym.sym = mods[i].key;
					if ( (mod&mask) ||
					     (mods[i].key == SDLK_CAPSLOCK) ) {
						mode = SDL_PRESSED;
					} else {
						mode = SDL_RELEASED;
					}
					SDL_PrivateKeyboard(mode, &keysym);
				}
			}

			/* Save state for next time */
			last_mods = mod;
		}

		/* Check for normal event keys, but we have to scan the
		   actual keyboard state because on Mac OS X a keydown event
		   is immediately followed by a keyup event.
		*/
		GetKeys(keys);
		if ( (keysptr[0] != last_keysptr[0]) ||
		     (keysptr[1] != last_keysptr[1]) ||
		     (keysptr[2] != last_keysptr[2]) ||
		     (keysptr[3] != last_keysptr[3]) ) {
			SDL_keysym keysym;
			int old_bit, new_bit;

#ifdef DEBUG_KEYBOARD
			fprintf(sterr, "New keys: 0x%x 0x%x 0x%x 0x%x\n",
				new_keys[0], new_keys[1],
				new_keys[2], new_keys[3]);
#endif
			for ( i=0; i<128; ++i ) {
				old_bit = (((Uint8 *)last_keys)[i/8]>>(i%8)) & 0x01;
				new_bit = (((Uint8 *)keys)[i/8]>>(i%8)) & 0x01;
				if ( old_bit != new_bit ) {
					/* Post the keyboard event */
#ifdef DEBUG_KEYBOARD
					fprintf(stderr,"Scancode: 0x%2.2X\n",i);
#endif
					SDL_PrivateKeyboard(new_bit,
				            TranslateKey(i, event.modifiers,
				                         &keysym, new_bit));
				}
			}

			/* Save state for next time */
			last_keys[0] = keys[0];
			last_keys[1] = keys[1];
			last_keys[2] = keys[2];
			last_keys[3] = keys[3];
		}
	}

	/* Handle normal events */
	switch (event.what) {
	  case mouseDown: {
		WindowRef win;
		short area;
				
		area = FindWindow(event.where, &win);
		/* Support switching between the SIOUX console
		   and SDL_Window by clicking in the window.
		 */
		if ( win && (win != FrontWindow()) ) {
			SelectWindow(win);
		} 
		switch (area) {
		  case inMenuBar: /* Only the apple menu exists */
			Mac_DoAppleMenu(this, MenuSelect(event.where));
			HiliteMenu(0);
			break;
		  case inDrag:
#if TARGET_API_MAC_CARBON
			DragWindow(win, event.where, NULL);
#else
			DragWindow(win, event.where, &theQD->screenBits.bounds);
#endif
			break;
		  case inGoAway:
			if ( TrackGoAway(win, event.where) ) {
				SDL_PrivateQuit();
			}
			break;
		  case inContent:
			myGlobalToLocal(this, &event.where);
			/* Treat command-click as right mouse button */
			if ( event.modifiers & optionKey ) {
				mouse_button = 2;
			} else if ( event.modifiers & cmdKey ) {
				mouse_button = 3;
			} else {
				mouse_button = 1;
			}
			SDL_PrivateMouseButton(SDL_PRESSED,
				mouse_button, event.where.h, event.where.v);
			break;
		  case inGrow: {
			int newSize;

			/* Don't allow resize if video mode isn't resizable */
			if ( ! SDL_PublicSurface ||
			     ! (SDL_PublicSurface->flags & SDL_RESIZABLE) ) {
				break;
			}
#if TARGET_API_MAC_CARBON
			newSize = GrowWindow(win, event.where, NULL);
#else
			newSize = GrowWindow(win, event.where, &theQD->screenBits.bounds);
#endif
			if ( newSize ) {
#if !TARGET_API_MAC_CARBON
				EraseRect ( &theQD->screenBits.bounds );
#endif
				SizeWindow ( win, LoWord (newSize), HiWord (newSize), 1 );
				SDL_PrivateResize ( LoWord (newSize), HiWord (newSize) );
			}
		  	} break;
		  case inZoomIn:
		  case inZoomOut:
			if ( TrackBox (win, event.where, area )) {
				Rect rect;
#if !TARGET_API_MAC_CARBON
				EraseRect ( &theQD->screenBits.bounds );
#endif
				ZoomWindow ( win, area, 0);
				if ( area == inZoomIn ) {
					GetWindowUserState(SDL_Window, &rect);
				} else {
					GetWindowStandardState(SDL_Window, &rect);
				}
				SDL_PrivateResize (rect.right-rect.left,
				                   rect.bottom-rect.top);
			}
			break;
#if TARGET_API_MAC_CARBON
		  case inCollapseBox:
			if ( TrackBox (win, event.where, area )) {
				if ( IsWindowCollapsable(win) ) {
					CollapseWindow (win, !IsWindowCollapsed(win));
					/* There should be something done like in inGrow case, but... */
				}
			}
			break;
#endif /* TARGET_API_MAC_CARBON */
		  case inSysWindow:
#if TARGET_API_MAC_CARBON
			/* Never happens in Carbon? */
#else
			SystemClick(&event, win);
#endif
			break;
		  default:
			break;
		}
	  }
	  break;
	  case mouseUp: {
		myGlobalToLocal(this, &event.where);
		/* Release the mouse button we simulated in the last press.
		   The drawback of this methos is we cannot press more than
		   one button. However, this doesn't matter, since there is
		   only a single logical mouse button, even if you have a
		   multi-button mouse, this doesn't matter at all.
		 */
		SDL_PrivateMouseButton(SDL_RELEASED,
			mouse_button, event.where.h, event.where.v);
	  }
	  break;
#if 0 /* Handled above the switch statement */
	  case keyDown: {
		SDL_keysym keysym;

		SDL_PrivateKeyboard(SDL_PRESSED,
			TranslateKey((event.message&keyCodeMask)>>8
		                     event.modifiers, &keysym, 1));
	  }
	  break;
	  case keyUp: {
		SDL_keysym keysym;

		SDL_PrivateKeyboard(SDL_RELEASED,
			TranslateKey((event.message&keyCodeMask)>>8
		                     event.modifiers, &keysym, 0));
	  }
	  break;
#endif
	  case updateEvt: {
		BeginUpdate(SDL_Window);
	#if SDL_VIDEO_OPENGL
		if (SDL_VideoSurface->flags & SDL_OPENGL)
			SDL_GL_SwapBuffers();
		else
	#endif
		if ( (SDL_VideoSurface->flags & SDL_HWSURFACE) ==
						SDL_SWSURFACE ) {
			SDL_UpdateRect(SDL_VideoSurface, 0, 0, 0, 0);
		}
		EndUpdate(SDL_Window);
	  }
	  /* If this was an update event for the SIOUX console, we return 0
             in order to stop an endless series of updates being triggered.
	  */
	  if ( (WindowRef) event.message != SDL_Window ) {
		return 0;
	  }
	  break;
	  case activateEvt: {
		Mac_HandleActivate(!!(event.modifiers & activeFlag));
	  }
	  break;
	  case diskEvt: {
#if TARGET_API_MAC_CARBON
		/* What are we supposed to do? */;
#else
		if ( ((event.message>>16)&0xFFFF) != noErr ) {
			Point spot;
			SetPt(&spot, 0x0070, 0x0050);
			DIBadMount(spot, event.message);
		}
#endif
	  }
	  break;
	  case osEvt: {
		switch ((event.message>>24) & 0xFF) {
#if 0 /* Handled above the switch statement */
		  case mouseMovedMessage: {
			myGlobalToLocal(this, &event.where);
			SDL_PrivateMouseMotion(0, 0,
					event.where.h, event.where.v);
		  }
		  break;
#endif /* 0 */
		  case suspendResumeMessage: {
			Mac_HandleActivate(!!(event.message & resumeFlag));
		  }
		  break;
		}
	  }
	  break;
	  default: {
		;
	  }
	  break;
	}
	return (event.what != nullEvent);
}


void Mac_PumpEvents(_THIS)
{
	/* Process pending MacOS events */
	while ( Mac_HandleEvents(this, 0) ) {
		/* Loop and check again */;
	}
}

void Mac_InitOSKeymap(_THIS)
{
	const void *KCHRPtr;
	UInt32 state;
	UInt32 value;
	int i;
	int world = SDLK_WORLD_0;

	/* Map the MAC keysyms */
	for ( i=0; i<SDL_arraysize(MAC_keymap); ++i )
		MAC_keymap[i] = SDLK_UNKNOWN;

	/* Defined MAC_* constants */
	MAC_keymap[MK_ESCAPE] = SDLK_ESCAPE;
	MAC_keymap[MK_F1] = SDLK_F1;
	MAC_keymap[MK_F2] = SDLK_F2;
	MAC_keymap[MK_F3] = SDLK_F3;
	MAC_keymap[MK_F4] = SDLK_F4;
	MAC_keymap[MK_F5] = SDLK_F5;
	MAC_keymap[MK_F6] = SDLK_F6;
	MAC_keymap[MK_F7] = SDLK_F7;
	MAC_keymap[MK_F8] = SDLK_F8;
	MAC_keymap[MK_F9] = SDLK_F9;
	MAC_keymap[MK_F10] = SDLK_F10;
	MAC_keymap[MK_F11] = SDLK_F11;
	MAC_keymap[MK_F12] = SDLK_F12;
	MAC_keymap[MK_PRINT] = SDLK_PRINT;
	MAC_keymap[MK_SCROLLOCK] = SDLK_SCROLLOCK;
	MAC_keymap[MK_PAUSE] = SDLK_PAUSE;
	MAC_keymap[MK_POWER] = SDLK_POWER;
	MAC_keymap[MK_BACKQUOTE] = SDLK_BACKQUOTE;
	MAC_keymap[MK_1] = SDLK_1;
	MAC_keymap[MK_2] = SDLK_2;
	MAC_keymap[MK_3] = SDLK_3;
	MAC_keymap[MK_4] = SDLK_4;
	MAC_keymap[MK_5] = SDLK_5;
	MAC_keymap[MK_6] = SDLK_6;
	MAC_keymap[MK_7] = SDLK_7;
	MAC_keymap[MK_8] = SDLK_8;
	MAC_keymap[MK_9] = SDLK_9;
	MAC_keymap[MK_0] = SDLK_0;
	MAC_keymap[MK_MINUS] = SDLK_MINUS;
	MAC_keymap[MK_EQUALS] = SDLK_EQUALS;
	MAC_keymap[MK_BACKSPACE] = SDLK_BACKSPACE;
	MAC_keymap[MK_INSERT] = SDLK_INSERT;
	MAC_keymap[MK_HOME] = SDLK_HOME;
	MAC_keymap[MK_PAGEUP] = SDLK_PAGEUP;
	MAC_keymap[MK_NUMLOCK] = SDLK_NUMLOCK;
	MAC_keymap[MK_KP_EQUALS] = SDLK_KP_EQUALS;
	MAC_keymap[MK_KP_DIVIDE] = SDLK_KP_DIVIDE;
	MAC_keymap[MK_KP_MULTIPLY] = SDLK_KP_MULTIPLY;
	MAC_keymap[MK_TAB] = SDLK_TAB;
	MAC_keymap[MK_q] = SDLK_q;
	MAC_keymap[MK_w] = SDLK_w;
	MAC_keymap[MK_e] = SDLK_e;
	MAC_keymap[MK_r] = SDLK_r;
	MAC_keymap[MK_t] = SDLK_t;
	MAC_keymap[MK_y] = SDLK_y;
	MAC_keymap[MK_u] = SDLK_u;
	MAC_keymap[MK_i] = SDLK_i;
	MAC_keymap[MK_o] = SDLK_o;
	MAC_keymap[MK_p] = SDLK_p;
	MAC_keymap[MK_LEFTBRACKET] = SDLK_LEFTBRACKET;
	MAC_keymap[MK_RIGHTBRACKET] = SDLK_RIGHTBRACKET;
	MAC_keymap[MK_BACKSLASH] = SDLK_BACKSLASH;
	MAC_keymap[MK_DELETE] = SDLK_DELETE;
	MAC_keymap[MK_END] = SDLK_END;
	MAC_keymap[MK_PAGEDOWN] = SDLK_PAGEDOWN;
	MAC_keymap[MK_KP7] = SDLK_KP7;
	MAC_keymap[MK_KP8] = SDLK_KP8;
	MAC_keymap[MK_KP9] = SDLK_KP9;
	MAC_keymap[MK_KP_MINUS] = SDLK_KP_MINUS;
	MAC_keymap[MK_CAPSLOCK] = SDLK_CAPSLOCK;
	MAC_keymap[MK_a] = SDLK_a;
	MAC_keymap[MK_s] = SDLK_s;
	MAC_keymap[MK_d] = SDLK_d;
	MAC_keymap[MK_f] = SDLK_f;
	MAC_keymap[MK_g] = SDLK_g;
	MAC_keymap[MK_h] = SDLK_h;
	MAC_keymap[MK_j] = SDLK_j;
	MAC_keymap[MK_k] = SDLK_k;
	MAC_keymap[MK_l] = SDLK_l;
	MAC_keymap[MK_SEMICOLON] = SDLK_SEMICOLON;
	MAC_keymap[MK_QUOTE] = SDLK_QUOTE;
	MAC_keymap[MK_RETURN] = SDLK_RETURN;
	MAC_keymap[MK_KP4] = SDLK_KP4;
	MAC_keymap[MK_KP5] = SDLK_KP5;
	MAC_keymap[MK_KP6] = SDLK_KP6;
	MAC_keymap[MK_KP_PLUS] = SDLK_KP_PLUS;
	MAC_keymap[MK_LSHIFT] = SDLK_LSHIFT;
	MAC_keymap[MK_z] = SDLK_z;
	MAC_keymap[MK_x] = SDLK_x;
	MAC_keymap[MK_c] = SDLK_c;
	MAC_keymap[MK_v] = SDLK_v;
	MAC_keymap[MK_b] = SDLK_b;
	MAC_keymap[MK_n] = SDLK_n;
	MAC_keymap[MK_m] = SDLK_m;
	MAC_keymap[MK_COMMA] = SDLK_COMMA;
	MAC_keymap[MK_PERIOD] = SDLK_PERIOD;
	MAC_keymap[MK_SLASH] = SDLK_SLASH;
#if 0	/* These are the same as the left versions - use left by default */
	MAC_keymap[MK_RSHIFT] = SDLK_RSHIFT;
#endif
	MAC_keymap[MK_UP] = SDLK_UP;
	MAC_keymap[MK_KP1] = SDLK_KP1;
	MAC_keymap[MK_KP2] = SDLK_KP2;
	MAC_keymap[MK_KP3] = SDLK_KP3;
	MAC_keymap[MK_KP_ENTER] = SDLK_KP_ENTER;
	MAC_keymap[MK_LCTRL] = SDLK_LCTRL;
	MAC_keymap[MK_LALT] = SDLK_LALT;
	MAC_keymap[MK_LMETA] = SDLK_LMETA;
	MAC_keymap[MK_SPACE] = SDLK_SPACE;
#if 0	/* These are the same as the left versions - use left by default */
	MAC_keymap[MK_RMETA] = SDLK_RMETA;
	MAC_keymap[MK_RALT] = SDLK_RALT;
	MAC_keymap[MK_RCTRL] = SDLK_RCTRL;
#endif
	MAC_keymap[MK_LEFT] = SDLK_LEFT;
	MAC_keymap[MK_DOWN] = SDLK_DOWN;
	MAC_keymap[MK_RIGHT] = SDLK_RIGHT;
	MAC_keymap[MK_KP0] = SDLK_KP0;
	MAC_keymap[MK_KP_PERIOD] = SDLK_KP_PERIOD;

#if defined(__APPLE__) && defined(__MACH__)
	/* Wierd, these keys are on my iBook under Mac OS X
	   Note that the left arrow keysym is the same as left ctrl!?
	 */
	MAC_keymap[MK_IBOOK_ENTER] = SDLK_KP_ENTER;
	MAC_keymap[MK_IBOOK_RIGHT] = SDLK_RIGHT;
	MAC_keymap[MK_IBOOK_DOWN] = SDLK_DOWN;
	MAC_keymap[MK_IBOOK_UP] = SDLK_UP;
	MAC_keymap[MK_IBOOK_LEFT] = SDLK_LEFT;
#endif /* Mac OS X */

	/* Up there we setup a static scancode->keysym map. However, it will not
	 * work very well on international keyboard. Hence we now query MacOS
	 * for its own keymap to adjust our own mapping table. However, this is
	 * bascially only useful for ascii char keys. This is also the reason
	 * why we keep the static table, too.
	 */
	
	/* Get a pointer to the systems cached KCHR */
	KCHRPtr = (void *)GetScriptManagerVariable(smKCHRCache);
	if (KCHRPtr)
	{
		/* Loop over all 127 possible scan codes */
		for (i = 0; i < 0x7F; i++)
		{
			/* We pretend a clean start to begin with (i.e. no dead keys active */
			state = 0;
			
			/* Now translate the key code to a key value */
			value = KeyTranslate(KCHRPtr, i, &state) & 0xff;
			
			/* If the state become 0, it was a dead key. We need to translate again,
			passing in the new state, to get the actual key value */
			if (state != 0)
				value = KeyTranslate(KCHRPtr, i, &state) & 0xff;
			
			/* Now we should have an ascii value, or 0. Try to figure out to which SDL symbol it maps */
			if (value >= 128)	 /* Some non-ASCII char, map it to SDLK_WORLD_* */
				MAC_keymap[i] = world++;
			else if (value >= 32)	 /* non-control ASCII char */
				MAC_keymap[i] = value;
		}
	}
	
	/* The keypad codes are re-setup here, because the loop above cannot
	 * distinguish between a key on the keypad and a regular key. We maybe
	 * could get around this problem in another fashion: NSEvent's flags
	 * include a "NSNumericPadKeyMask" bit; we could check that and modify
	 * the symbol we return on the fly. However, this flag seems to exhibit
	 * some weird behaviour related to the num lock key
	 */
	MAC_keymap[MK_KP0] = SDLK_KP0;
	MAC_keymap[MK_KP1] = SDLK_KP1;
	MAC_keymap[MK_KP2] = SDLK_KP2;
	MAC_keymap[MK_KP3] = SDLK_KP3;
	MAC_keymap[MK_KP4] = SDLK_KP4;
	MAC_keymap[MK_KP5] = SDLK_KP5;
	MAC_keymap[MK_KP6] = SDLK_KP6;
	MAC_keymap[MK_KP7] = SDLK_KP7;
	MAC_keymap[MK_KP8] = SDLK_KP8;
	MAC_keymap[MK_KP9] = SDLK_KP9;
	MAC_keymap[MK_KP_MINUS] = SDLK_KP_MINUS;
	MAC_keymap[MK_KP_PLUS] = SDLK_KP_PLUS;
	MAC_keymap[MK_KP_PERIOD] = SDLK_KP_PERIOD;
	MAC_keymap[MK_KP_EQUALS] = SDLK_KP_EQUALS;
	MAC_keymap[MK_KP_DIVIDE] = SDLK_KP_DIVIDE;
	MAC_keymap[MK_KP_MULTIPLY] = SDLK_KP_MULTIPLY;
	MAC_keymap[MK_KP_ENTER] = SDLK_KP_ENTER;
}

static SDL_keysym *TranslateKey(int scancode, int modifiers,
                                SDL_keysym *keysym, int pressed)
{
	/* Set the keysym information */
	keysym->scancode = scancode;
	keysym->sym = MAC_keymap[keysym->scancode];
	keysym->mod = KMOD_NONE;
	keysym->unicode = 0;
	if ( pressed && SDL_TranslateUNICODE ) {
		static unsigned long state = 0;
		static Ptr keymap = nil;
		Ptr new_keymap;

		/* Get the current keyboard map resource */
		new_keymap = (Ptr)GetScriptManagerVariable(smKCHRCache);
		if ( new_keymap != keymap ) {
			keymap = new_keymap;
			state = 0;
		}
		keysym->unicode = KeyTranslate(keymap,
			keysym->scancode|modifiers, &state) & 0xFFFF;
	}
	return(keysym);
}

void Mac_InitEvents(_THIS)
{
	/* Create apple menu bar */
	apple_menu = GetMenu(mApple);
	if ( apple_menu != nil ) {
		AppendResMenu(apple_menu, 'DRVR');
		InsertMenu(apple_menu, 0);
	}
	DrawMenuBar();

	/* Get rid of spurious events at startup */
	FlushEvents(everyEvent, 0);
	
	/* Allow every event but keyrepeat */
	SetEventMask(everyEvent & ~autoKeyMask);
}

void Mac_QuitEvents(_THIS)
{
	ClearMenuBar();
	if ( apple_menu != nil ) {
		ReleaseResource((char **)apple_menu);
	}

	/* Clean up pending events */
	FlushEvents(everyEvent, 0);
}

static void Mac_DoAppleMenu(_THIS, long choice)
{
#if !TARGET_API_MAC_CARBON  /* No Apple menu in OS X */
	short menu, item;

	item = (choice&0xFFFF);
	choice >>= 16;
	menu = (choice&0xFFFF);
	
	switch (menu) {
		case mApple: {
			switch (item) {
				case iAbout: {
					/* Run the about box */;
				}
				break;
				default: {
					Str255 name;
					
					GetMenuItemText(apple_menu, item, name);
					OpenDeskAcc(name);
				}
				break;
			}
		}
		break;
		default: {
			/* Ignore other menus */;
		}
	}
#endif /* !TARGET_API_MAC_CARBON */
}

#if !TARGET_API_MAC_CARBON
/* Since we don't initialize QuickDraw, we need to get a pointer to qd */
struct QDGlobals *theQD = NULL;
#endif

/* Exported to the macmain code */
void SDL_InitQuickDraw(struct QDGlobals *the_qd)
{
#if !TARGET_API_MAC_CARBON
	theQD = the_qd;
#endif
}
