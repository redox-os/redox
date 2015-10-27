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

/* CAUTION!!!!  If you modify this file, check ../windib/SDL_sysevents.c */

#include "directx.h"

#include "SDL_main.h"
#include "SDL_events.h"
#include "SDL_video.h"
#include "SDL_syswm.h"
#include "../../events/SDL_sysevents.h"
#include "../../events/SDL_events_c.h"
#include "../wincommon/SDL_lowvideo.h"
#include "SDL_dx5video.h"

#ifndef WM_APP
#define WM_APP	0x8000
#endif

#ifdef _WIN32_WCE
#define NO_GETKEYBOARDSTATE
#endif

/* The keyboard and mouse device input */
#define MAX_INPUTS	2
#define INPUT_QSIZE	512		/* Buffer up to 512 input messages */

static LPDIRECTINPUT dinput = NULL;
static LPDIRECTINPUTDEVICE2 SDL_DIdev[MAX_INPUTS];
static HANDLE               SDL_DIevt[MAX_INPUTS];
static void (*SDL_DIfun[MAX_INPUTS])(const int, DIDEVICEOBJECTDATA *);
static int SDL_DIndev = 0;
static int mouse_lost;
static int mouse_pressed;
static int mouse_buttons_swapped = 0;

/* The translation table from a DirectInput scancode to an SDL keysym */
static SDLKey DIK_keymap[256];
static SDL_keysym *TranslateKey(UINT scancode, SDL_keysym *keysym, int pressed);

/* DJM: If the user setup the window for us, we want to save his window proc,
   and give him a chance to handle some messages. */
#ifdef STRICT
#define WNDPROCTYPE	WNDPROC
#else
#define WNDPROCTYPE	FARPROC
#endif
static WNDPROCTYPE userWindowProc = NULL;

static HWND GetTopLevelParent(HWND hWnd)
{
    HWND hParentWnd;
    while (1)
    {
        hParentWnd = GetParent(hWnd);
        if (hParentWnd == NULL)
            break;
        hWnd = hParentWnd;
    }
    return hWnd;
}

/* Convert a DirectInput return code to a text message */
static void SetDIerror(char *function, int code)
{
	static char *error;
	static char  errbuf[1024];

	errbuf[0] = 0;
	switch (code) {
                case DIERR_GENERIC:
                        error = "Undefined error!";
                        break;
		case DIERR_OLDDIRECTINPUTVERSION:
			error = "Your version of DirectInput needs upgrading";
			break;
		case DIERR_INVALIDPARAM:
                        error = "Invalid parameters";
                        break;
                case DIERR_OUTOFMEMORY:
                        error = "Out of memory";
                        break;
		case DIERR_DEVICENOTREG:
			error = "Device not registered";
			break;
		case DIERR_NOINTERFACE:
			error = "Interface not supported";
			break;
		case DIERR_NOTINITIALIZED:
			error = "Device not initialized";
			break;
		default:
			SDL_snprintf(errbuf, SDL_arraysize(errbuf),
			         "%s: Unknown DirectInput error: 0x%x",
								function, code);
			break;
	}
	if ( ! errbuf[0] ) {
		SDL_snprintf(errbuf, SDL_arraysize(errbuf), "%s: %s", function, error);
	}
	SDL_SetError("%s", errbuf);
	return;
}

/* Initialize DirectInput
   Note:  If NONEXCLUSIVE access is requested for the devices, normal 
          windows input messages will continue to be generated for that
          input device, in addition to DirectInput messages.
 */
static void handle_keyboard(const int numevents, DIDEVICEOBJECTDATA *events);
static void handle_mouse(const int numevents, DIDEVICEOBJECTDATA *events);
struct {
	char *name;
	REFGUID guid;
	LPCDIDATAFORMAT format;
	DWORD win_level;
	DWORD raw_level;
	void (*fun)(const int numevents, DIDEVICEOBJECTDATA *events);
} inputs[] = {
	{ "keyboard",
		&GUID_SysKeyboard, &c_dfDIKeyboard,
		(DISCL_FOREGROUND|DISCL_NONEXCLUSIVE),
		(DISCL_FOREGROUND|DISCL_NONEXCLUSIVE), handle_keyboard },
	{ "mouse",
		&GUID_SysMouse,
#if DIRECTINPUT_VERSION >= 0x700
		&c_dfDIMouse2,
#else
		&c_dfDIMouse,
#endif
		(DISCL_BACKGROUND|DISCL_NONEXCLUSIVE),
		(DISCL_BACKGROUND|DISCL_NONEXCLUSIVE), handle_mouse },
	{ NULL, NULL, NULL, 0, 0, NULL }
};
	
static int DX5_DInputInit(_THIS)
{
	int         i;
	LPDIRECTINPUTDEVICE device;
	HRESULT     result;
	DIPROPDWORD dipdw;
	HWND        topwnd;

	/* Create the DirectInput object */
	result = DInputCreate(SDL_Instance, DIRECTINPUT_VERSION,
							&dinput, NULL);
	if ( result != DI_OK ) {
		SetDIerror("DirectInputCreate", result);
		return(-1);
	}

	/* Create all of our registered input devices */
	SDL_DIndev = 0;
	for ( i=0; inputs[i].name; ++i ) {
		/* Create the DirectInput device */
		result = IDirectInput_CreateDevice(dinput, inputs[i].guid,
								&device, NULL);
		if ( result != DI_OK ) {
			SetDIerror("DirectInput::CreateDevice", result);
			return(-1);
		}
		result = IDirectInputDevice_QueryInterface(device,
			&IID_IDirectInputDevice2, (LPVOID *)&SDL_DIdev[i]);
		IDirectInputDevice_Release(device);
		if ( result != DI_OK ) {
			SetDIerror("DirectInputDevice::QueryInterface", result);
			return(-1);
		}
		topwnd =  GetTopLevelParent(SDL_Window);
		result = IDirectInputDevice2_SetCooperativeLevel(SDL_DIdev[i],
					topwnd, inputs[i].win_level);
		if ( result != DI_OK ) {
			SetDIerror("DirectInputDevice::SetCooperativeLevel",
									result);
			return(-1);
		}
		result = IDirectInputDevice2_SetDataFormat(SDL_DIdev[i],
							inputs[i].format);
		if ( result != DI_OK ) {
			SetDIerror("DirectInputDevice::SetDataFormat", result);
			return(-1);
		}

		/* Set buffered input -- we aren't polling */
		SDL_memset(&dipdw, 0, sizeof(dipdw));
		dipdw.diph.dwSize = sizeof(dipdw);
		dipdw.diph.dwHeaderSize = sizeof(dipdw.diph);
		dipdw.diph.dwObj = 0;
		dipdw.diph.dwHow = DIPH_DEVICE;
		dipdw.dwData = INPUT_QSIZE;
		result = IDirectInputDevice2_SetProperty(SDL_DIdev[i],
						DIPROP_BUFFERSIZE, &dipdw.diph);
		if ( result != DI_OK ) {
			SetDIerror("DirectInputDevice::SetProperty", result);
			return(-1);
		}

		/* Create an event to be signaled when input is ready */
		SDL_DIevt[i] = CreateEvent(NULL, FALSE, FALSE, NULL);
		if ( SDL_DIevt[i] == NULL ) {
			SDL_SetError("Couldn't create DirectInput event");
			return(-1);
		}
		result = IDirectInputDevice2_SetEventNotification(SDL_DIdev[i],
								SDL_DIevt[i]);
		if ( result != DI_OK ) {
			SetDIerror("DirectInputDevice::SetEventNotification",
									result);
			return(-1);
		}
		SDL_DIfun[i] = inputs[i].fun;

		/* Acquire the device for input */
		IDirectInputDevice2_Acquire(SDL_DIdev[i]);

		/* Increment the number of devices we have */
		++SDL_DIndev;
	}
	mouse_pressed = 0;
	mouse_buttons_swapped = GetSystemMetrics(SM_SWAPBUTTON);

	/* DirectInput is ready! */
	return(0);
}

/* Clean up DirectInput */
static void DX5_DInputQuit(_THIS)
{
	int i;

	if ( dinput != NULL ) {
		/* Close and release all DirectInput devices */
		for ( i=0; i<MAX_INPUTS; ++i ) {
			if ( SDL_DIdev[i] != NULL ) {
				IDirectInputDevice2_Unacquire(SDL_DIdev[i]);
				IDirectInputDevice2_SetEventNotification(
							SDL_DIdev[i], NULL);
				if ( SDL_DIevt[i] != NULL ) {
					CloseHandle(SDL_DIevt[i]);
					SDL_DIevt[i] = NULL;
				}
				IDirectInputDevice2_Release(SDL_DIdev[i]);
				SDL_DIdev[i] = NULL;
			}
		}
		SDL_DIndev = 0;

		/* Release DirectInput */
		IDirectInput_Release(dinput);
		dinput = NULL;
	}
}

/* Flag to tell SDL whether or not we queued an event */
static int posted = 0;

/* Input event handler functions */
static void handle_keyboard(const int numevents, DIDEVICEOBJECTDATA *keybuf)
{
	int i;
	SDL_keysym keysym;

	/* Translate keyboard messages */
	for ( i=0; i<numevents; ++i ) {
		if ( keybuf[i].dwData & 0x80 ) {
			posted = SDL_PrivateKeyboard(SDL_PRESSED,
				    TranslateKey(keybuf[i].dwOfs, &keysym, 1));
		} else {
			posted = SDL_PrivateKeyboard(SDL_RELEASED,
				    TranslateKey(keybuf[i].dwOfs, &keysym, 0));
		}
	}
}

static void post_mouse_motion(int relative, Sint16 x, Sint16 y)
{
	extern int mouse_relative;

	if ( SDL_GetAppState() & SDL_APPMOUSEFOCUS ) {
		posted = SDL_PrivateMouseMotion(
			0, relative, x, y);

		if ( !mouse_relative ) {
			/* As DirectInput reads raw device coordinates, it has no notion of
			 * cursors or absolute position. We must assume responsibility for
			 * keeping track of this. */
			int current_x, current_y;
			POINT cursor;
			RECT trap;
			RECT window;
			int at_edge;

			/* Get the current cursor position */
			SDL_GetMouseState(&current_x, &current_y);
			cursor.x = current_x;
			cursor.y = current_y;
			ClientToScreen(SDL_Window, &cursor);

			/* Construct a 1 pixel square RECT that is used to confine the cursor
			 * pointer to a specific pixel using ClipCursor. This is used in
			 * preference to SetCursorPos as it avoids the cursor jumping around as
			 * both the OS and SDL attempt to move it simultaneously. */
			trap.left = cursor.x;
			trap.top = cursor.y;
			trap.right = cursor.x + 1;
			trap.bottom = cursor.y + 1;

			GetClientRect(SDL_Window, &window);
			window.right -= window.left; window.left = 0;
			window.bottom -= window.top; window.top = 0;

			/* As we're assuming control over the cursor, we need to know when to
			 * relinquish control of it back to the operating system. This is when
			 * the cursor reaches the edge of the window. */
			at_edge = (current_x == window.left) ||
				(current_x == (window.right - 1)) ||
				(current_y == window.top) ||
				(current_y == (window.bottom - 1));

			if ( at_edge ) {
				ClipCursor(NULL);
			} else {
				ClipCursor(&trap);
			}
		} else {
			/* When in relative mode, warp the OS's idea of where the cursor is to
			 * the center of the screen. This isn't really necessary as DirectInput
			 * reads from the hardware itself, but in case things go wrong, the
			 * cursor will be left in a sensible place. */
			POINT center;
			center.x = (SDL_VideoSurface->w/2);
			center.y = (SDL_VideoSurface->h/2);
			ClientToScreen(SDL_Window, &center);
			SetCursorPos(center.x, center.y);
		}
	}
}

static void handle_mouse(const int numevents, DIDEVICEOBJECTDATA *ptrbuf)
{
	int i;
	Sint16 xrel, yrel;
	Uint8 state;
	Uint8 button;
	DWORD timestamp = 0;

	/* Sanity check. Mailing list reports this being NULL unexpectedly. */
	if (SDL_PublicSurface == NULL) {
		return;
	}

	/* If mouse focus has been lost, make sure we release the cursor. */
	if ( !(SDL_GetAppState() & SDL_APPMOUSEFOCUS) ) {
		mouse_lost = 1;
		ClipCursor(NULL);
	} else {
		/* If the mouse was lost, regain some sense of mouse state */
		if ( mouse_lost ) {
			POINT mouse_pos;
			Uint8 old_state;
			Uint8 new_state;

			/* Set ourselves up with the current cursor position */
			GetCursorPos(&mouse_pos);
			ScreenToClient(SDL_Window, &mouse_pos);
			post_mouse_motion( 0, (Sint16)mouse_pos.x, (Sint16)mouse_pos.y);

			/* Check for mouse button changes */
			old_state = SDL_GetMouseState(NULL, NULL);
			new_state = 0;
			{ /* Get the new DirectInput button state for the mouse */
	#if DIRECTINPUT_VERSION >= 0x700
				DIMOUSESTATE2 distate;
	#else
				DIMOUSESTATE distate;
	#endif
				HRESULT result;

				result=IDirectInputDevice2_GetDeviceState(SDL_DIdev[1],
							sizeof(distate), &distate);
				if ( result != DI_OK ) {
					/* Try again next time */
					SetDIerror(
					"IDirectInputDevice2::GetDeviceState", result);
					return;
				}
				for ( i=3; i>=0; --i ) {
					if ( (distate.rgbButtons[i]&0x80) == 0x80 ) {
						new_state |= 0x01;
					}
					new_state <<= 1;
				}
			}
			for ( i=0; i<8; ++i ) {
				if ( (old_state&0x01) != (new_state&0x01) ) {
					button = (Uint8)(i+1);
					/* Map DI button numbers to SDL */
					switch ( button ) {
						case 2: button = SDL_BUTTON_RIGHT; break;
						case 3: button = SDL_BUTTON_MIDDLE; break;
						case 4: button = SDL_BUTTON_X1; break;
						case 5: button = SDL_BUTTON_X2; break;
						default: break;
					}
					if ( new_state & 0x01 ) {
						/* Grab mouse so we get mouse-up */
						if ( ++mouse_pressed > 0 ) {
							SetCapture(SDL_Window);
						}
						state = SDL_PRESSED;
					} else {
						/* Release mouse after all mouse-ups */
						if ( --mouse_pressed <= 0 ) {
							ReleaseCapture();
							mouse_pressed = 0;
						}
						state = SDL_RELEASED;
					}
					if ( mouse_buttons_swapped ) {
						if ( button == 1 ) button = 3;
						else
						if ( button == 3 ) button = 1;
					}
					posted = SDL_PrivateMouseButton(state, button,
										0, 0);
				}
				old_state >>= 1;
				new_state >>= 1;
			}
			mouse_lost = 0;
			return;
		}

		/* Translate mouse messages */
		xrel = 0;
		yrel = 0;
		for ( i=0; i<(int)numevents; ++i ) {
			switch (ptrbuf[i].dwOfs) {
				case DIMOFS_X:
					if ( timestamp != ptrbuf[i].dwTimeStamp ) {
						if ( xrel || yrel ) {
							post_mouse_motion(1, xrel, yrel);
							xrel = 0;
							yrel = 0;
						}
						timestamp = ptrbuf[i].dwTimeStamp;
					}
					xrel += (Sint16)ptrbuf[i].dwData;
					break;
				case DIMOFS_Y:
					if ( timestamp != ptrbuf[i].dwTimeStamp ) {
						if ( xrel || yrel ) {
							post_mouse_motion(1, xrel, yrel);
							xrel = 0;
							yrel = 0;
						}
						timestamp = ptrbuf[i].dwTimeStamp;
					}
					yrel += (Sint16)ptrbuf[i].dwData;
					break;
				case DIMOFS_Z:
					if ( xrel || yrel ) {
						post_mouse_motion(1, xrel, yrel);
						xrel = 0;
						yrel = 0;
					}
					timestamp = 0;
					if((int)ptrbuf[i].dwData > 0)
						button = SDL_BUTTON_WHEELUP;
					else
						button = SDL_BUTTON_WHEELDOWN;
					posted = SDL_PrivateMouseButton(
							SDL_PRESSED, button, 0, 0);
					posted |= SDL_PrivateMouseButton(
							SDL_RELEASED, button, 0, 0);
					break;
				case DIMOFS_BUTTON0:
				case DIMOFS_BUTTON1:
				case DIMOFS_BUTTON2:
				case DIMOFS_BUTTON3:
	#if DIRECTINPUT_VERSION >= 0x700
				case DIMOFS_BUTTON4:
				case DIMOFS_BUTTON5:
				case DIMOFS_BUTTON6:
				case DIMOFS_BUTTON7:
	#endif
					if ( xrel || yrel ) {
						post_mouse_motion(1, xrel, yrel);
						xrel = 0;
						yrel = 0;
					}
					timestamp = 0;
					button = (Uint8)(ptrbuf[i].dwOfs-DIMOFS_BUTTON0)+1;
					/* Map DI button numbers to SDL */
					switch ( button ) {
						case 2: button = SDL_BUTTON_RIGHT; break;
						case 3: button = SDL_BUTTON_MIDDLE; break;
						case 4: button = SDL_BUTTON_X1; break;
						case 5: button = SDL_BUTTON_X2; break;
						default: break;
					}
					if ( ptrbuf[i].dwData & 0x80 ) {
						/* Grab mouse so we get mouse-up */
						if ( ++mouse_pressed > 0 ) {
							SetCapture(SDL_Window);
						}
						state = SDL_PRESSED;
					} else {
						/* Release mouse after all mouse-ups */
						if ( --mouse_pressed <= 0 ) {
							ReleaseCapture();
							mouse_pressed = 0;
						}
						state = SDL_RELEASED;
					}
					if ( mouse_buttons_swapped ) {
						if ( button == 1 ) button = 3;
						else
						if ( button == 3 ) button = 1;
					}
					posted = SDL_PrivateMouseButton(state, button,
										0, 0);
					break;
			}
		}
		if ( xrel || yrel ) {
			post_mouse_motion(1, xrel, yrel);
		}
	}
}

/* The main Win32 event handler */
LRESULT DX5_HandleMessage(_THIS, HWND hwnd, UINT msg, WPARAM wParam, LPARAM lParam)
{
	switch (msg) {
#ifdef WM_ACTIVATEAPP
		case WM_ACTIVATEAPP: {
			int i, active;

			active = (wParam && (GetForegroundWindow() == hwnd));
			if ( active ) {
				for ( i=0; i<MAX_INPUTS; ++i ) {
					if (SDL_DIdev[i] != NULL)
						IDirectInputDevice2_Acquire(
								SDL_DIdev[i]);
				}
			} else {
				for ( i=0; i<MAX_INPUTS; ++i ) {
					if (SDL_DIdev[i] != NULL) 
						IDirectInputDevice2_Unacquire(
								SDL_DIdev[i]);
				}
				mouse_lost = 1;
			}
		}
		break;
#endif /* WM_ACTIVATEAPP */

#ifdef WM_DISPLAYCHANGE
		case WM_DISPLAYCHANGE: {
			WPARAM BitsPerPixel;
			WORD SizeX, SizeY;

			/* Ack!  The display changed size and/or depth! */
			SizeX = LOWORD(lParam);
			SizeY = HIWORD(lParam);
			BitsPerPixel = wParam;
			/* We cause this message when we go fullscreen */
		}
		break;
#endif /* WM_DISPLAYCHANGE */

		/* The keyboard is handled via DirectInput */
		case WM_SYSKEYUP:
		case WM_SYSKEYDOWN:
		case WM_KEYUP:
		case WM_KEYDOWN: {
			/* Ignore windows keyboard messages */;
		}
		return(0);

#if defined(SC_SCREENSAVE) || defined(SC_MONITORPOWER)
		/* Don't allow screen savers or monitor power downs.
		   This is because they quietly clear DirectX surfaces.
		   It would be better to allow the application to
		   decide whether or not to blow these off, but the
		   semantics of SDL_PrivateSysWMEvent() don't allow
		   the application that choice.
		 */
		case WM_SYSCOMMAND: {
			if ((wParam&0xFFF0)==SC_SCREENSAVE || 
			    (wParam&0xFFF0)==SC_MONITORPOWER)
				return(0);
		}
		/* Fall through to default processing */

#endif /* SC_SCREENSAVE || SC_MONITORPOWER */

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

/* This function checks the windows message queue and DirectInput and returns
   1 if there was input, 0 if there was no input, or -1 if the application has
   posted a quit message.
*/
static int DX5_CheckInput(_THIS, int timeout, BOOL processInput)
{
	MSG msg;
	int      i;
	HRESULT  result;
	DWORD    event;

	/* Check the normal windows queue (highest preference) */
	posted = 0;
	while ( ! posted &&
	        PeekMessage(&msg, NULL, 0, 0, PM_NOREMOVE) ) {
		if ( GetMessage(&msg, NULL, 0, 0) > 0 ) {
			DispatchMessage(&msg);
		} else {
			return(-1);
		}
	}
	if ( posted ) {
		return(1);
	}

	/* Pump the DirectInput flow */
	if ( SDL_GetAppState() & SDL_APPMOUSEFOCUS ) {
		for ( i=0; i<MAX_INPUTS; ++i ) {
			if ( SDL_DIdev[i] != NULL ) {
				result = IDirectInputDevice2_Poll(SDL_DIdev[i]);
				if ( (result == DIERR_INPUTLOST) ||
						(result == DIERR_NOTACQUIRED) ) {
					if ( SDL_strcmp(inputs[i].name, "mouse") == 0 ) {
						mouse_lost = 1;
					}
					IDirectInputDevice2_Acquire(SDL_DIdev[i]);
					IDirectInputDevice2_Poll(SDL_DIdev[i]);
				}
			}
		}
	}

	/* Wait for messages and input events */
	event = MsgWaitForMultipleObjects(SDL_DIndev, SDL_DIevt, FALSE,
							timeout, QS_ALLEVENTS);
	if ((event >= WAIT_OBJECT_0) && (event < (WAIT_OBJECT_0+SDL_DIndev))) {
		DWORD numevents;
		static DIDEVICEOBJECTDATA evtbuf[INPUT_QSIZE];

		event -= WAIT_OBJECT_0;
		numevents = INPUT_QSIZE;
		result = IDirectInputDevice2_GetDeviceData(
				SDL_DIdev[event], sizeof(DIDEVICEOBJECTDATA),
							evtbuf, &numevents, 0);
		if ( (result == DIERR_INPUTLOST) ||
					(result == DIERR_NOTACQUIRED) ) {
			if ( SDL_strcmp(inputs[event].name, "mouse") == 0 ) {
				mouse_lost = 1;
			}
			IDirectInputDevice2_Acquire(SDL_DIdev[event]);
			result = IDirectInputDevice2_GetDeviceData(
				SDL_DIdev[event], sizeof(DIDEVICEOBJECTDATA),
							evtbuf, &numevents, 0);
		}
		/* Handle the events */
		if ( result == DI_OK && processInput ) {
			/* Note: This can post multiple events to event queue
			 */
			(*SDL_DIfun[event])((int)numevents, evtbuf);
			return(1);
		}
	}
	if ( event != WAIT_TIMEOUT ) {
		/* Maybe there was a windows message? */
		if ( PeekMessage(&msg, NULL, 0, 0, PM_NOREMOVE) ) {
			if ( GetMessage(&msg, NULL, 0, 0) > 0 ) {
				DispatchMessage(&msg);
			} else {
				return(-1);
			}
			return(1);
		}
	}
	return(0);
}

/* Change cooperative level based on whether or not we are fullscreen */
void DX5_DInputReset(_THIS, int fullscreen)
{
	DWORD level;
	int i;
	HRESULT result;
	HWND topwnd;

	for ( i=0; i<MAX_INPUTS; ++i ) {
		if ( SDL_DIdev[i] != NULL ) {
			if ( fullscreen ) {
				level = inputs[i].raw_level;
			} else {
				level = inputs[i].win_level;
			}
			IDirectInputDevice2_Unacquire(SDL_DIdev[i]);
			topwnd = GetTopLevelParent(SDL_Window);
			result = IDirectInputDevice2_SetCooperativeLevel(
					SDL_DIdev[i], topwnd, level);
			IDirectInputDevice2_Acquire(SDL_DIdev[i]);
			if ( result != DI_OK ) {
				SetDIerror(
			"DirectInputDevice::SetCooperativeLevel", result);
			}
		}
	}
	mouse_lost = 1;

	/* Flush pending input */
	DX5_CheckInput(this, 0, FALSE);
}

void DX5_PumpEvents(_THIS)
{
	/* Wait for messages and DirectInput */
	while ( DX5_CheckInput(this, 0, TRUE) > 0 ) {
		/* Loop and check again */;
	}
}

void DX5_InitOSKeymap(_THIS)
{
#ifndef DIK_PAUSE
#define DIK_PAUSE	0xC5
#endif
#ifndef DIK_OEM_102
#define DIK_OEM_102	0x56	/* < > | on UK/Germany keyboards */
#endif
	int i;

	/* Map the DIK scancodes to SDL keysyms */
	for ( i=0; i<SDL_arraysize(DIK_keymap); ++i )
		DIK_keymap[i] = 0;

	/* Defined DIK_* constants */
	DIK_keymap[DIK_ESCAPE] = SDLK_ESCAPE;
	DIK_keymap[DIK_1] = SDLK_1;
	DIK_keymap[DIK_2] = SDLK_2;
	DIK_keymap[DIK_3] = SDLK_3;
	DIK_keymap[DIK_4] = SDLK_4;
	DIK_keymap[DIK_5] = SDLK_5;
	DIK_keymap[DIK_6] = SDLK_6;
	DIK_keymap[DIK_7] = SDLK_7;
	DIK_keymap[DIK_8] = SDLK_8;
	DIK_keymap[DIK_9] = SDLK_9;
	DIK_keymap[DIK_0] = SDLK_0;
	DIK_keymap[DIK_MINUS] = SDLK_MINUS;
	DIK_keymap[DIK_EQUALS] = SDLK_EQUALS;
	DIK_keymap[DIK_BACK] = SDLK_BACKSPACE;
	DIK_keymap[DIK_TAB] = SDLK_TAB;
	DIK_keymap[DIK_Q] = SDLK_q;
	DIK_keymap[DIK_W] = SDLK_w;
	DIK_keymap[DIK_E] = SDLK_e;
	DIK_keymap[DIK_R] = SDLK_r;
	DIK_keymap[DIK_T] = SDLK_t;
	DIK_keymap[DIK_Y] = SDLK_y;
	DIK_keymap[DIK_U] = SDLK_u;
	DIK_keymap[DIK_I] = SDLK_i;
	DIK_keymap[DIK_O] = SDLK_o;
	DIK_keymap[DIK_P] = SDLK_p;
	DIK_keymap[DIK_LBRACKET] = SDLK_LEFTBRACKET;
	DIK_keymap[DIK_RBRACKET] = SDLK_RIGHTBRACKET;
	DIK_keymap[DIK_RETURN] = SDLK_RETURN;
	DIK_keymap[DIK_LCONTROL] = SDLK_LCTRL;
	DIK_keymap[DIK_A] = SDLK_a;
	DIK_keymap[DIK_S] = SDLK_s;
	DIK_keymap[DIK_D] = SDLK_d;
	DIK_keymap[DIK_F] = SDLK_f;
	DIK_keymap[DIK_G] = SDLK_g;
	DIK_keymap[DIK_H] = SDLK_h;
	DIK_keymap[DIK_J] = SDLK_j;
	DIK_keymap[DIK_K] = SDLK_k;
	DIK_keymap[DIK_L] = SDLK_l;
	DIK_keymap[DIK_SEMICOLON] = SDLK_SEMICOLON;
	DIK_keymap[DIK_APOSTROPHE] = SDLK_QUOTE;
	DIK_keymap[DIK_GRAVE] = SDLK_BACKQUOTE;
	DIK_keymap[DIK_LSHIFT] = SDLK_LSHIFT;
	DIK_keymap[DIK_BACKSLASH] = SDLK_BACKSLASH;
	DIK_keymap[DIK_OEM_102] = SDLK_LESS;
	DIK_keymap[DIK_Z] = SDLK_z;
	DIK_keymap[DIK_X] = SDLK_x;
	DIK_keymap[DIK_C] = SDLK_c;
	DIK_keymap[DIK_V] = SDLK_v;
	DIK_keymap[DIK_B] = SDLK_b;
	DIK_keymap[DIK_N] = SDLK_n;
	DIK_keymap[DIK_M] = SDLK_m;
	DIK_keymap[DIK_COMMA] = SDLK_COMMA;
	DIK_keymap[DIK_PERIOD] = SDLK_PERIOD;
	DIK_keymap[DIK_SLASH] = SDLK_SLASH;
	DIK_keymap[DIK_RSHIFT] = SDLK_RSHIFT;
	DIK_keymap[DIK_MULTIPLY] = SDLK_KP_MULTIPLY;
	DIK_keymap[DIK_LMENU] = SDLK_LALT;
	DIK_keymap[DIK_SPACE] = SDLK_SPACE;
	DIK_keymap[DIK_CAPITAL] = SDLK_CAPSLOCK;
	DIK_keymap[DIK_F1] = SDLK_F1;
	DIK_keymap[DIK_F2] = SDLK_F2;
	DIK_keymap[DIK_F3] = SDLK_F3;
	DIK_keymap[DIK_F4] = SDLK_F4;
	DIK_keymap[DIK_F5] = SDLK_F5;
	DIK_keymap[DIK_F6] = SDLK_F6;
	DIK_keymap[DIK_F7] = SDLK_F7;
	DIK_keymap[DIK_F8] = SDLK_F8;
	DIK_keymap[DIK_F9] = SDLK_F9;
	DIK_keymap[DIK_F10] = SDLK_F10;
	DIK_keymap[DIK_NUMLOCK] = SDLK_NUMLOCK;
	DIK_keymap[DIK_SCROLL] = SDLK_SCROLLOCK;
	DIK_keymap[DIK_NUMPAD7] = SDLK_KP7;
	DIK_keymap[DIK_NUMPAD8] = SDLK_KP8;
	DIK_keymap[DIK_NUMPAD9] = SDLK_KP9;
	DIK_keymap[DIK_SUBTRACT] = SDLK_KP_MINUS;
	DIK_keymap[DIK_NUMPAD4] = SDLK_KP4;
	DIK_keymap[DIK_NUMPAD5] = SDLK_KP5;
	DIK_keymap[DIK_NUMPAD6] = SDLK_KP6;
	DIK_keymap[DIK_ADD] = SDLK_KP_PLUS;
	DIK_keymap[DIK_NUMPAD1] = SDLK_KP1;
	DIK_keymap[DIK_NUMPAD2] = SDLK_KP2;
	DIK_keymap[DIK_NUMPAD3] = SDLK_KP3;
	DIK_keymap[DIK_NUMPAD0] = SDLK_KP0;
	DIK_keymap[DIK_DECIMAL] = SDLK_KP_PERIOD;
	DIK_keymap[DIK_F11] = SDLK_F11;
	DIK_keymap[DIK_F12] = SDLK_F12;

	DIK_keymap[DIK_F13] = SDLK_F13;
	DIK_keymap[DIK_F14] = SDLK_F14;
	DIK_keymap[DIK_F15] = SDLK_F15;

	DIK_keymap[DIK_NUMPADEQUALS] = SDLK_KP_EQUALS;
	DIK_keymap[DIK_NUMPADENTER] = SDLK_KP_ENTER;
	DIK_keymap[DIK_RCONTROL] = SDLK_RCTRL;
	DIK_keymap[DIK_DIVIDE] = SDLK_KP_DIVIDE;
	DIK_keymap[DIK_SYSRQ] = SDLK_PRINT;
	DIK_keymap[DIK_RMENU] = SDLK_RALT;
	DIK_keymap[DIK_PAUSE] = SDLK_PAUSE;
	DIK_keymap[DIK_HOME] = SDLK_HOME;
	DIK_keymap[DIK_UP] = SDLK_UP;
	DIK_keymap[DIK_PRIOR] = SDLK_PAGEUP;
	DIK_keymap[DIK_LEFT] = SDLK_LEFT;
	DIK_keymap[DIK_RIGHT] = SDLK_RIGHT;
	DIK_keymap[DIK_END] = SDLK_END;
	DIK_keymap[DIK_DOWN] = SDLK_DOWN;
	DIK_keymap[DIK_NEXT] = SDLK_PAGEDOWN;
	DIK_keymap[DIK_INSERT] = SDLK_INSERT;
	DIK_keymap[DIK_DELETE] = SDLK_DELETE;
	DIK_keymap[DIK_LWIN] = SDLK_LMETA;
	DIK_keymap[DIK_RWIN] = SDLK_RMETA;
	DIK_keymap[DIK_APPS] = SDLK_MENU;
}

static SDL_keysym *TranslateKey(UINT scancode, SDL_keysym *keysym, int pressed)
{
	/* Set the keysym information */
	keysym->scancode = (unsigned char)scancode;
	keysym->sym = DIK_keymap[scancode];
	keysym->mod = KMOD_NONE;
	keysym->unicode = 0;
	if ( pressed && SDL_TranslateUNICODE ) {
		UINT vkey;
#ifndef NO_GETKEYBOARDSTATE
		BYTE	keystate[256];
		Uint16	wchars[2];
#endif

		vkey = MapVirtualKey(scancode, 1);
#ifdef NO_GETKEYBOARDSTATE
		/* Uh oh, better hope the vkey is close enough.. */
		keysym->unicode = vkey;
#else
		GetKeyboardState(keystate);
		/* Numlock isn't taken into account in ToUnicode,
		 * so we handle it as a special case here */
		if ((keystate[VK_NUMLOCK] & 1) && vkey >= VK_NUMPAD0 && vkey <= VK_NUMPAD9)
		{
			keysym->unicode = vkey - VK_NUMPAD0 + '0';
		}
		else if (SDL_ToUnicode(vkey, scancode, keystate, wchars, sizeof(wchars)/sizeof(wchars[0]), 0) > 0)
		{
			keysym->unicode = wchars[0];
		}
#endif /* NO_GETKEYBOARDSTATE */
	}
	return(keysym);
}

int DX5_CreateWindow(_THIS)
{
	char *windowid = SDL_getenv("SDL_WINDOWID");
	int i;

	/* Clear out DirectInput variables in case we fail */
	for ( i=0; i<MAX_INPUTS; ++i ) {
		SDL_DIdev[i] = NULL;
		SDL_DIevt[i] = NULL;
		SDL_DIfun[i] = NULL;
	}

	SDL_RegisterApp(NULL, 0, 0);

	SDL_windowid = (windowid != NULL);
	if ( SDL_windowid ) {
		SDL_Window = (HWND)((size_t)SDL_strtoull(windowid, NULL, 0));
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

	/* Initialize DirectInput */
	if ( DX5_DInputInit(this) < 0 ) {
		return(-1);
	}

	/* JC 14 Mar 2006
		Flush the message loop or this can cause big problems later
		Especially if the user decides to use dialog boxes or assert()!
	*/
	WIN_FlushMessageQueue();

	/* Ready to roll */
	return(0);
}

void DX5_DestroyWindow(_THIS)
{
	/* Close down DirectInput */
	DX5_DInputQuit(this);

	/* Destroy our window */
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
