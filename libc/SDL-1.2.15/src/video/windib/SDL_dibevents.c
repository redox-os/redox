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

#define WIN32_LEAN_AND_MEAN
#include <windows.h>

#include "SDL_main.h"
#include "SDL_events.h"
#include "SDL_syswm.h"
#include "../../events/SDL_sysevents.h"
#include "../../events/SDL_events_c.h"
#include "../wincommon/SDL_lowvideo.h"
#include "SDL_gapidibvideo.h"
#include "SDL_vkeys.h"

#ifdef SDL_VIDEO_DRIVER_GAPI
#include "../gapi/SDL_gapivideo.h"
#endif

#ifdef SDL_VIDEO_DRIVER_WINDIB
#include "SDL_dibvideo.h"
#endif

#ifndef WM_APP
#define WM_APP	0x8000
#endif

#ifdef _WIN32_WCE
#define NO_GETKEYBOARDSTATE
#endif

/* The translation table from a Microsoft VK keysym to a SDL keysym */
static SDLKey VK_keymap[SDLK_LAST];
static SDL_keysym *TranslateKey(WPARAM vkey, UINT scancode, SDL_keysym *keysym, int pressed);
static SDLKey Arrows_keymap[4];

/* Masks for processing the windows KEYDOWN and KEYUP messages */
#define REPEATED_KEYMASK	(1<<30)
#define EXTENDED_KEYMASK	(1<<24)

/* DJM: If the user setup the window for us, we want to save his window proc,
   and give him a chance to handle some messages. */
#ifdef STRICT
#define WNDPROCTYPE	WNDPROC
#else
#define WNDPROCTYPE	FARPROC
#endif
static WNDPROCTYPE userWindowProc = NULL;


#ifdef SDL_VIDEO_DRIVER_GAPI

WPARAM rotateKey(WPARAM key,int direction) 
{
	if(direction ==0 ) return key;
	
	switch (key) {
		case 0x26: /* up */
			return Arrows_keymap[(2 + direction) % 4];
		case 0x27: /* right */
			return Arrows_keymap[(1 + direction) % 4];
		case 0x28: /* down */
			return Arrows_keymap[direction % 4];
		case 0x25: /* left */
			return Arrows_keymap[(3 + direction) % 4];
	}

	return key;
}

static void GapiTransform(GapiInfo *gapiInfo, LONG *x, LONG *y)
{
    if(gapiInfo->hiresFix)
    {
	*x *= 2;
	*y *= 2;
    }

    // 0 3 0
    if((!gapiInfo->userOrientation && gapiInfo->systemOrientation && !gapiInfo->gapiOrientation) ||
    // 3 0 3
       (gapiInfo->userOrientation && !gapiInfo->systemOrientation && gapiInfo->gapiOrientation) ||
    // 3 0 0
       (gapiInfo->userOrientation && !gapiInfo->systemOrientation && !gapiInfo->gapiOrientation))
    {
	Sint16 temp = *x;
        *x = SDL_VideoSurface->w - *y;
        *y = temp;
    }
    else
    // 0 0 0
    if((!gapiInfo->userOrientation && !gapiInfo->systemOrientation && !gapiInfo->gapiOrientation) ||
    // 0 0 3
      (!gapiInfo->userOrientation && !gapiInfo->systemOrientation && gapiInfo->gapiOrientation))
    {
	// without changes
	// *x = *x;
	// *y = *y;
    }
    // default
    else
    {
	// without changes
	// *x = *x;
	// *y = *y;
    }
}
#endif 


/* The main Win32 event handler */
LRESULT DIB_HandleMessage(_THIS, HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam)
{
	extern int posted;

	switch (msg) {
		case WM_SYSKEYDOWN:
		case WM_KEYDOWN: {
			SDL_keysym keysym;

#ifdef SDL_VIDEO_DRIVER_GAPI
			if(this->hidden->gapiInfo)
			{
				// Drop GAPI artefacts
				if (wParam == 0x84 || wParam == 0x5B)
					return 0;

				wParam = rotateKey(wParam, this->hidden->gapiInfo->coordinateTransform);
			}
#endif 
			/* Ignore repeated keys */
			if ( lParam&REPEATED_KEYMASK ) {
				return(0);
			}
			switch (wParam) {
				case VK_CONTROL:
					if ( lParam&EXTENDED_KEYMASK )
						wParam = VK_RCONTROL;
					else
						wParam = VK_LCONTROL;
					break;
				case VK_SHIFT:
					/* EXTENDED trick doesn't work here */
					{
					Uint8 *state = SDL_GetKeyState(NULL);
					if (state[SDLK_LSHIFT] == SDL_RELEASED && (GetKeyState(VK_LSHIFT) & 0x8000)) {
						wParam = VK_LSHIFT;
					} else if (state[SDLK_RSHIFT] == SDL_RELEASED && (GetKeyState(VK_RSHIFT) & 0x8000)) {
						wParam = VK_RSHIFT;
					} else {
						/* Win9x */
						int sc = HIWORD(lParam) & 0xFF;

						if (sc == 0x2A)
							wParam = VK_LSHIFT;
						else
						if (sc == 0x36)
							wParam = VK_RSHIFT;
						else
							wParam = VK_LSHIFT;
					}
					}
					break;
				case VK_MENU:
					if ( lParam&EXTENDED_KEYMASK )
						wParam = VK_RMENU;
					else
						wParam = VK_LMENU;
					break;
			}
#ifdef NO_GETKEYBOARDSTATE
			/* this is the workaround for the missing ToAscii() and ToUnicode() in CE (not necessary at KEYUP!) */
			if ( SDL_TranslateUNICODE ) {
				MSG m;

				m.hwnd = hwnd;
				m.message = msg;
				m.wParam = wParam;
				m.lParam = lParam;
				m.time = 0;
				if ( TranslateMessage(&m) && PeekMessage(&m, hwnd, 0, WM_USER, PM_NOREMOVE) && (m.message == WM_CHAR) ) {
					GetMessage(&m, hwnd, 0, WM_USER);
			    		wParam = m.wParam;
				}
			}
#endif /* NO_GETKEYBOARDSTATE */
			posted = SDL_PrivateKeyboard(SDL_PRESSED,
				TranslateKey(wParam,HIWORD(lParam),&keysym,1));
		}
		return(0);

		case WM_SYSKEYUP:
		case WM_KEYUP: {
			SDL_keysym keysym;

#ifdef SDL_VIDEO_DRIVER_GAPI
			if(this->hidden->gapiInfo)
			{
				// Drop GAPI artifacts
				if (wParam == 0x84 || wParam == 0x5B)
					return 0;
	
				wParam = rotateKey(wParam, this->hidden->gapiInfo->coordinateTransform);
			}
#endif

			switch (wParam) {
				case VK_CONTROL:
					if ( lParam&EXTENDED_KEYMASK )
						wParam = VK_RCONTROL;
					else
						wParam = VK_LCONTROL;
					break;
				case VK_SHIFT:
					/* EXTENDED trick doesn't work here */
					{
					Uint8 *state = SDL_GetKeyState(NULL);
					if (state[SDLK_LSHIFT] == SDL_PRESSED && !(GetKeyState(VK_LSHIFT) & 0x8000)) {
						wParam = VK_LSHIFT;
					} else if (state[SDLK_RSHIFT] == SDL_PRESSED && !(GetKeyState(VK_RSHIFT) & 0x8000)) {
						wParam = VK_RSHIFT;
					} else {
						/* Win9x */
						int sc = HIWORD(lParam) & 0xFF;

						if (sc == 0x2A)
							wParam = VK_LSHIFT;
						else
						if (sc == 0x36)
							wParam = VK_RSHIFT;
						else
							wParam = VK_LSHIFT;
					}
					}
					break;
				case VK_MENU:
					if ( lParam&EXTENDED_KEYMASK )
						wParam = VK_RMENU;
					else
						wParam = VK_LMENU;
					break;
			}
			/* Windows only reports keyup for print screen */
			if ( wParam == VK_SNAPSHOT && SDL_GetKeyState(NULL)[SDLK_PRINT] == SDL_RELEASED ) {
				posted = SDL_PrivateKeyboard(SDL_PRESSED,
					TranslateKey(wParam,HIWORD(lParam),&keysym,1));
			}
			posted = SDL_PrivateKeyboard(SDL_RELEASED,
				TranslateKey(wParam,HIWORD(lParam),&keysym,0));
		}
		return(0);
#if defined(SC_SCREENSAVE) && defined(SC_MONITORPOWER)
		case WM_SYSCOMMAND: {
			const DWORD val = (DWORD) (wParam & 0xFFF0);
			if ((val == SC_SCREENSAVE) || (val == SC_MONITORPOWER)) {
				if (this->hidden->dibInfo && !allow_screensaver) {
					/* Note that this doesn't stop anything on Vista
					   if the screensaver has a password. */
					return(0);
				}
			}
		}
		/* Fall through to default processing */
#endif /* SC_SCREENSAVE && SC_MONITORPOWER */

		default: {
			/* Only post the event if we're watching for it */
			if ( SDL_ProcessEvents[SDL_SYSWMEVENT] == SDL_ENABLE ) {
			        SDL_SysWMmsg wmmsg;

				SDL_VERSION(&wmmsg.version);
				wmmsg.hwnd = hwnd;
				wmmsg.msg = msg;
				wmmsg.wParam = wParam;
				wmmsg.lParam = lParam;
				posted = SDL_PrivateSysWMEvent(&wmmsg);

			/* DJM: If the user isn't watching for private
				messages in her SDL event loop, then pass it
				along to any win32 specific window proc.
			 */
			} else if (userWindowProc) {
				return CallWindowProc(userWindowProc, hwnd, msg, wParam, lParam);
			}
		}
		break;
	}
	return(DefWindowProc(hwnd, msg, wParam, lParam));
}

#ifdef _WIN32_WCE
static BOOL GetLastStylusPos(POINT* ptLast)
{
    BOOL bResult = FALSE;
    UINT nRet;
    GetMouseMovePoints(ptLast, 1, &nRet);
    if ( nRet == 1 ) {
        ptLast->x /= 4;
        ptLast->y /= 4;
        bResult = TRUE;
    }
    return bResult;
}
#endif

static void DIB_GenerateMouseMotionEvent(_THIS)
{
	extern int mouse_relative;
	extern int posted;

	POINT mouse;
#ifdef _WIN32_WCE
	if ( !GetCursorPos(&mouse) && !GetLastStylusPos(&mouse) ) return;
#else
	if ( !GetCursorPos(&mouse) ) return;
#endif

	if ( mouse_relative ) {
		POINT center;
		center.x = (SDL_VideoSurface->w/2);
		center.y = (SDL_VideoSurface->h/2);
		ClientToScreen(SDL_Window, &center);

		mouse.x -= center.x;
		mouse.y -= center.y;
		if ( mouse.x || mouse.y ) {
			SetCursorPos(center.x, center.y);
			posted = SDL_PrivateMouseMotion(0, 1, (Sint16)mouse.x, (Sint16)mouse.y);
		}
	} else {
		ScreenToClient(SDL_Window, &mouse);
#ifdef SDL_VIDEO_DRIVER_GAPI
       if (SDL_VideoSurface && this->hidden->gapiInfo)
			GapiTransform(this->hidden->gapiInfo, &mouse.x, &mouse.y);
#endif
		posted = SDL_PrivateMouseMotion(0, 0, (Sint16)mouse.x, (Sint16)mouse.y);
	}
}

void DIB_PumpEvents(_THIS)
{
	MSG msg;

	while ( PeekMessage(&msg, NULL, 0, 0, PM_NOREMOVE) ) {
		if ( GetMessage(&msg, NULL, 0, 0) > 0 ) {
			DispatchMessage(&msg);
		}
	}

	if ( SDL_GetAppState() & SDL_APPMOUSEFOCUS ) {
		DIB_GenerateMouseMotionEvent( this );
	}
}

static HKL hLayoutUS = NULL;

void DIB_InitOSKeymap(_THIS)
{
	int	i;
#ifndef _WIN32_WCE
	char	current_layout[KL_NAMELENGTH];

	GetKeyboardLayoutName(current_layout);
	//printf("Initial Keyboard Layout Name: '%s'\n", current_layout);

	hLayoutUS = LoadKeyboardLayout("00000409", KLF_NOTELLSHELL);

	if (!hLayoutUS) {
		//printf("Failed to load US keyboard layout. Using current.\n");
		hLayoutUS = GetKeyboardLayout(0);
	}
	LoadKeyboardLayout(current_layout, KLF_ACTIVATE);
#else
#if _WIN32_WCE >=420
	TCHAR	current_layout[KL_NAMELENGTH];

	GetKeyboardLayoutName(current_layout);
	//printf("Initial Keyboard Layout Name: '%s'\n", current_layout);

	hLayoutUS = LoadKeyboardLayout(L"00000409", 0);

	if (!hLayoutUS) {
		//printf("Failed to load US keyboard layout. Using current.\n");
		hLayoutUS = GetKeyboardLayout(0);
	}
	LoadKeyboardLayout(current_layout, 0);
#endif // _WIN32_WCE >=420
#endif
	/* Map the VK keysyms */
	for ( i=0; i<SDL_arraysize(VK_keymap); ++i )
		VK_keymap[i] = SDLK_UNKNOWN;

	VK_keymap[VK_BACK] = SDLK_BACKSPACE;
	VK_keymap[VK_TAB] = SDLK_TAB;
	VK_keymap[VK_CLEAR] = SDLK_CLEAR;
	VK_keymap[VK_RETURN] = SDLK_RETURN;
	VK_keymap[VK_PAUSE] = SDLK_PAUSE;
	VK_keymap[VK_ESCAPE] = SDLK_ESCAPE;
	VK_keymap[VK_SPACE] = SDLK_SPACE;
	VK_keymap[VK_APOSTROPHE] = SDLK_QUOTE;
	VK_keymap[VK_COMMA] = SDLK_COMMA;
	VK_keymap[VK_MINUS] = SDLK_MINUS;
	VK_keymap[VK_PERIOD] = SDLK_PERIOD;
	VK_keymap[VK_SLASH] = SDLK_SLASH;
	VK_keymap[VK_0] = SDLK_0;
	VK_keymap[VK_1] = SDLK_1;
	VK_keymap[VK_2] = SDLK_2;
	VK_keymap[VK_3] = SDLK_3;
	VK_keymap[VK_4] = SDLK_4;
	VK_keymap[VK_5] = SDLK_5;
	VK_keymap[VK_6] = SDLK_6;
	VK_keymap[VK_7] = SDLK_7;
	VK_keymap[VK_8] = SDLK_8;
	VK_keymap[VK_9] = SDLK_9;
	VK_keymap[VK_SEMICOLON] = SDLK_SEMICOLON;
	VK_keymap[VK_EQUALS] = SDLK_EQUALS;
	VK_keymap[VK_LBRACKET] = SDLK_LEFTBRACKET;
	VK_keymap[VK_BACKSLASH] = SDLK_BACKSLASH;
	VK_keymap[VK_OEM_102] = SDLK_LESS;
	VK_keymap[VK_RBRACKET] = SDLK_RIGHTBRACKET;
	VK_keymap[VK_GRAVE] = SDLK_BACKQUOTE;
	VK_keymap[VK_BACKTICK] = SDLK_BACKQUOTE;
	VK_keymap[VK_A] = SDLK_a;
	VK_keymap[VK_B] = SDLK_b;
	VK_keymap[VK_C] = SDLK_c;
	VK_keymap[VK_D] = SDLK_d;
	VK_keymap[VK_E] = SDLK_e;
	VK_keymap[VK_F] = SDLK_f;
	VK_keymap[VK_G] = SDLK_g;
	VK_keymap[VK_H] = SDLK_h;
	VK_keymap[VK_I] = SDLK_i;
	VK_keymap[VK_J] = SDLK_j;
	VK_keymap[VK_K] = SDLK_k;
	VK_keymap[VK_L] = SDLK_l;
	VK_keymap[VK_M] = SDLK_m;
	VK_keymap[VK_N] = SDLK_n;
	VK_keymap[VK_O] = SDLK_o;
	VK_keymap[VK_P] = SDLK_p;
	VK_keymap[VK_Q] = SDLK_q;
	VK_keymap[VK_R] = SDLK_r;
	VK_keymap[VK_S] = SDLK_s;
	VK_keymap[VK_T] = SDLK_t;
	VK_keymap[VK_U] = SDLK_u;
	VK_keymap[VK_V] = SDLK_v;
	VK_keymap[VK_W] = SDLK_w;
	VK_keymap[VK_X] = SDLK_x;
	VK_keymap[VK_Y] = SDLK_y;
	VK_keymap[VK_Z] = SDLK_z;
	VK_keymap[VK_DELETE] = SDLK_DELETE;

	VK_keymap[VK_NUMPAD0] = SDLK_KP0;
	VK_keymap[VK_NUMPAD1] = SDLK_KP1;
	VK_keymap[VK_NUMPAD2] = SDLK_KP2;
	VK_keymap[VK_NUMPAD3] = SDLK_KP3;
	VK_keymap[VK_NUMPAD4] = SDLK_KP4;
	VK_keymap[VK_NUMPAD5] = SDLK_KP5;
	VK_keymap[VK_NUMPAD6] = SDLK_KP6;
	VK_keymap[VK_NUMPAD7] = SDLK_KP7;
	VK_keymap[VK_NUMPAD8] = SDLK_KP8;
	VK_keymap[VK_NUMPAD9] = SDLK_KP9;
	VK_keymap[VK_DECIMAL] = SDLK_KP_PERIOD;
	VK_keymap[VK_DIVIDE] = SDLK_KP_DIVIDE;
	VK_keymap[VK_MULTIPLY] = SDLK_KP_MULTIPLY;
	VK_keymap[VK_SUBTRACT] = SDLK_KP_MINUS;
	VK_keymap[VK_ADD] = SDLK_KP_PLUS;

	VK_keymap[VK_UP] = SDLK_UP;
	VK_keymap[VK_DOWN] = SDLK_DOWN;
	VK_keymap[VK_RIGHT] = SDLK_RIGHT;
	VK_keymap[VK_LEFT] = SDLK_LEFT;
	VK_keymap[VK_INSERT] = SDLK_INSERT;
	VK_keymap[VK_HOME] = SDLK_HOME;
	VK_keymap[VK_END] = SDLK_END;
	VK_keymap[VK_PRIOR] = SDLK_PAGEUP;
	VK_keymap[VK_NEXT] = SDLK_PAGEDOWN;

	VK_keymap[VK_F1] = SDLK_F1;
	VK_keymap[VK_F2] = SDLK_F2;
	VK_keymap[VK_F3] = SDLK_F3;
	VK_keymap[VK_F4] = SDLK_F4;
	VK_keymap[VK_F5] = SDLK_F5;
	VK_keymap[VK_F6] = SDLK_F6;
	VK_keymap[VK_F7] = SDLK_F7;
	VK_keymap[VK_F8] = SDLK_F8;
	VK_keymap[VK_F9] = SDLK_F9;
	VK_keymap[VK_F10] = SDLK_F10;
	VK_keymap[VK_F11] = SDLK_F11;
	VK_keymap[VK_F12] = SDLK_F12;
	VK_keymap[VK_F13] = SDLK_F13;
	VK_keymap[VK_F14] = SDLK_F14;
	VK_keymap[VK_F15] = SDLK_F15;

	VK_keymap[VK_NUMLOCK] = SDLK_NUMLOCK;
	VK_keymap[VK_CAPITAL] = SDLK_CAPSLOCK;
	VK_keymap[VK_SCROLL] = SDLK_SCROLLOCK;
	VK_keymap[VK_RSHIFT] = SDLK_RSHIFT;
	VK_keymap[VK_LSHIFT] = SDLK_LSHIFT;
	VK_keymap[VK_RCONTROL] = SDLK_RCTRL;
	VK_keymap[VK_LCONTROL] = SDLK_LCTRL;
	VK_keymap[VK_RMENU] = SDLK_RALT;
	VK_keymap[VK_LMENU] = SDLK_LALT;
	VK_keymap[VK_RWIN] = SDLK_RSUPER;
	VK_keymap[VK_LWIN] = SDLK_LSUPER;

	VK_keymap[VK_HELP] = SDLK_HELP;
#ifdef VK_PRINT
	VK_keymap[VK_PRINT] = SDLK_PRINT;
#endif
	VK_keymap[VK_SNAPSHOT] = SDLK_PRINT;
	VK_keymap[VK_CANCEL] = SDLK_BREAK;
	VK_keymap[VK_APPS] = SDLK_MENU;

	Arrows_keymap[3] = 0x25;
	Arrows_keymap[2] = 0x26;
	Arrows_keymap[1] = 0x27;
	Arrows_keymap[0] = 0x28;
}

#define EXTKEYPAD(keypad) ((scancode & 0x100)?(mvke):(keypad))

static int SDL_MapVirtualKey(int scancode, int vkey)
{
#ifndef _WIN32_WCE
	int	mvke  = MapVirtualKeyEx(scancode & 0xFF, 1, hLayoutUS);
#else
	int	mvke  = MapVirtualKey(scancode & 0xFF, 1);
#endif

	switch(vkey) {
		/* These are always correct */
		case VK_DIVIDE:
		case VK_MULTIPLY:
		case VK_SUBTRACT:
		case VK_ADD:
		case VK_LWIN:
		case VK_RWIN:
		case VK_APPS:
		/* These are already handled */
		case VK_LCONTROL:
		case VK_RCONTROL:
		case VK_LSHIFT:
		case VK_RSHIFT:
		case VK_LMENU:
		case VK_RMENU:
		case VK_SNAPSHOT:
		case VK_PAUSE:
			return vkey;
	}	
	switch(mvke) {
		/* Distinguish between keypad and extended keys */
		case VK_INSERT: return EXTKEYPAD(VK_NUMPAD0);
		case VK_DELETE: return EXTKEYPAD(VK_DECIMAL);
		case VK_END:    return EXTKEYPAD(VK_NUMPAD1);
		case VK_DOWN:   return EXTKEYPAD(VK_NUMPAD2);
		case VK_NEXT:   return EXTKEYPAD(VK_NUMPAD3);
		case VK_LEFT:   return EXTKEYPAD(VK_NUMPAD4);
		case VK_CLEAR:  return EXTKEYPAD(VK_NUMPAD5);
		case VK_RIGHT:  return EXTKEYPAD(VK_NUMPAD6);
		case VK_HOME:   return EXTKEYPAD(VK_NUMPAD7);
		case VK_UP:     return EXTKEYPAD(VK_NUMPAD8);
		case VK_PRIOR:  return EXTKEYPAD(VK_NUMPAD9);
	}
	return mvke?mvke:vkey;
}

static SDL_keysym *TranslateKey(WPARAM vkey, UINT scancode, SDL_keysym *keysym, int pressed)
{
	/* Set the keysym information */
	keysym->scancode = (unsigned char) scancode;
	keysym->mod = KMOD_NONE;
	keysym->unicode = 0;
	
	if ((vkey == VK_RETURN) && (scancode & 0x100)) {
		/* No VK_ code for the keypad enter key */
		keysym->sym = SDLK_KP_ENTER;
	}
	else {
		keysym->sym = VK_keymap[SDL_MapVirtualKey(scancode, vkey)];
	}

	if ( pressed && SDL_TranslateUNICODE ) {
#ifdef NO_GETKEYBOARDSTATE
		/* Uh oh, better hope the vkey is close enough.. */
		if((keysym->sym == vkey) || (vkey > 0x7f))
		keysym->unicode = vkey;
#else
		BYTE	keystate[256];
		Uint16	wchars[2];

		GetKeyboardState(keystate);
		/* Numlock isn't taken into account in ToUnicode,
		 * so we handle it as a special case here */
		if ((keystate[VK_NUMLOCK] & 1) && vkey >= VK_NUMPAD0 && vkey <= VK_NUMPAD9)
		{
			keysym->unicode = vkey - VK_NUMPAD0 + '0';
		}
		else if (SDL_ToUnicode((UINT)vkey, scancode, keystate, wchars, sizeof(wchars)/sizeof(wchars[0]), 0) > 0)
		{
			keysym->unicode = wchars[0];
		}
#endif /* NO_GETKEYBOARDSTATE */
	}

#if 0
	{
		HKL     hLayoutCurrent = GetKeyboardLayout(0);
		int     sc = scancode & 0xFF;

		printf("SYM:%d, VK:0x%02X, SC:0x%04X, US:(1:0x%02X, 3:0x%02X), "
			"Current:(1:0x%02X, 3:0x%02X)\n",
			keysym->sym, vkey, scancode,
			MapVirtualKeyEx(sc, 1, hLayoutUS),
			MapVirtualKeyEx(sc, 3, hLayoutUS),
			MapVirtualKeyEx(sc, 1, hLayoutCurrent),
			MapVirtualKeyEx(sc, 3, hLayoutCurrent)
		);
	}
#endif
	return(keysym);
}

int DIB_CreateWindow(_THIS)
{
	char *windowid;

	SDL_RegisterApp(NULL, 0, 0);

	windowid = SDL_getenv("SDL_WINDOWID");
	SDL_windowid = (windowid != NULL);
	if ( SDL_windowid ) {
#if defined(_WIN32_WCE) && (_WIN32_WCE < 300)
		/* wince 2.1 does not have strtol */
		wchar_t *windowid_t = SDL_malloc((SDL_strlen(windowid) + 1) * sizeof(wchar_t));
		MultiByteToWideChar(CP_ACP, MB_PRECOMPOSED, windowid, -1, windowid_t, SDL_strlen(windowid) + 1);
		SDL_Window = (HWND)wcstol(windowid_t, NULL, 0);
		SDL_free(windowid_t);
#else
		SDL_Window = (HWND)((size_t)SDL_strtoull(windowid, NULL, 0));
#endif
		if ( SDL_Window == NULL ) {
			SDL_SetError("Couldn't get user specified window");
			return(-1);
		}

		/* DJM: we want all event's for the user specified
			window to be handled by SDL.
		 */
		userWindowProc = (WNDPROCTYPE)GetWindowLongPtr(SDL_Window, GWLP_WNDPROC);
		SetWindowLongPtr(SDL_Window, GWLP_WNDPROC, (LONG_PTR)WinMessage);
	} else {
		SDL_Window = CreateWindow(SDL_Appname, SDL_Appname,
                        (WS_OVERLAPPED|WS_CAPTION|WS_SYSMENU|WS_MINIMIZEBOX),
                        CW_USEDEFAULT, CW_USEDEFAULT, 0, 0, NULL, NULL, SDL_Instance, NULL);
		if ( SDL_Window == NULL ) {
			SDL_SetError("Couldn't create window");
			return(-1);
		}
		ShowWindow(SDL_Window, SW_HIDE);
	}

	/* JC 14 Mar 2006
		Flush the message loop or this can cause big problems later
		Especially if the user decides to use dialog boxes or assert()!
	*/
	WIN_FlushMessageQueue();

	return(0);
}

void DIB_DestroyWindow(_THIS)
{
	if ( SDL_windowid ) {
		SetWindowLongPtr(SDL_Window, GWLP_WNDPROC, (LONG_PTR)userWindowProc);
	} else {
		DestroyWindow(SDL_Window);
	}
	SDL_UnregisterApp();

	/* JC 14 Mar 2006
		Flush the message loop or this can cause big problems later
		Especially if the user decides to use dialog boxes or assert()!
	*/
	WIN_FlushMessageQueue();
}
