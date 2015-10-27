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

/*
 * GEM SDL video driver implementation
 * inspired from the Dummy SDL driver
 * 
 * Patrice Mandin
 * and work from
 * Olivier Landemarre, Johan Klockars, Xavier Joubert, Claude Attard
 */

#include <gem.h>

#include "../../events/SDL_sysevents.h"
#include "../../events/SDL_events_c.h"
#include "SDL_gemvideo.h"
#include "SDL_gemevents_c.h"
#include "SDL_gemmouse_c.h"
#include "../ataricommon/SDL_atarikeys.h"	/* for keyboard scancodes */
#include "../ataricommon/SDL_atarievents_c.h"
#include "../ataricommon/SDL_xbiosevents_c.h"
#include "../ataricommon/SDL_ataridevmouse_c.h"

/* Variables */

static unsigned char gem_currentkeyboard[ATARIBIOS_MAXKEYS];
static unsigned char gem_previouskeyboard[ATARIBIOS_MAXKEYS];

/* Functions prototypes */

static int do_messages(_THIS, short *message);
static void do_keyboard(short kc, short ks);
static void do_mouse(_THIS, short mx, short my, short mb, short ks);

/* Functions */

void GEM_InitOSKeymap(_THIS)
{
	SDL_memset(gem_currentkeyboard, 0, sizeof(gem_currentkeyboard));
	SDL_memset(gem_previouskeyboard, 0, sizeof(gem_previouskeyboard));

	/* Mouse init */
	GEM_mouse_relative = SDL_FALSE;

	SDL_Atari_InitInternalKeymap(this);
}

void GEM_PumpEvents(_THIS)
{
	short prevkc, prevks;
	static short maskmouseb=0;
	int i;
	SDL_keysym	keysym;

	SDL_memset(gem_currentkeyboard,0,sizeof(gem_currentkeyboard));
	prevkc = prevks = 0;

	for (;;)
	{
		int quit, resultat, event_mask, mouse_event;
		short buffer[8], kc;
		short x2,y2,w2,h2;
		short mousex, mousey, mouseb, dummy;
		short kstate;

		quit =
			mouse_event =
			x2=y2=w2=h2 = 0;

		event_mask = MU_MESAG|MU_TIMER|MU_KEYBD|MU_BUTTON;
		if (!GEM_fullscreen && (GEM_handle>=0)) {
			wind_get (GEM_handle, WF_WORKXYWH, &x2, &y2, &w2, &h2);
			event_mask |= MU_M1;
			mouse_event = ( (SDL_GetAppState() & SDL_APPMOUSEFOCUS)
				== SDL_APPMOUSEFOCUS) ? MO_LEAVE : MO_ENTER;
		}

		resultat = evnt_multi(
			event_mask,
			0x101,7,maskmouseb,
			mouse_event,x2,y2,w2,h2,
			0,0,0,0,0,
			buffer,
			10,
			&mousex,&mousey,&mouseb,&kstate,&kc,&dummy
		);

		/* Message event ? */
		if (resultat & MU_MESAG)
			quit = do_messages(this, buffer);

		/* Keyboard event ? */
		if (resultat & MU_KEYBD) {
			if ((prevkc != kc) || (prevks != kstate)) {
				do_keyboard(kc,kstate);
			} else {
				/* Avoid looping, if repeating same key */
				break;
			}
		}

		/* Mouse entering/leaving window */
		if (resultat & MU_M1) {
			if (this->input_grab == SDL_GRAB_OFF) {
				/* Switch mouse focus state */
				SDL_PrivateAppActive((mouse_event == MO_ENTER),
					SDL_APPMOUSEFOCUS);
			}
			GEM_CheckMouseMode(this);
		}

		/* Mouse button event ? */
		if (resultat & MU_BUTTON) {
			do_mouse(this, mousex, mousey, mouseb, kstate);
			maskmouseb = mouseb & 7;
		}

		/* Timer event ? */
		if ((resultat & MU_TIMER) || quit)
			break;
	}

	/* Now generate keyboard events */
	for (i=0; i<ATARIBIOS_MAXKEYS; i++) {
		/* Key pressed ? */
		if (gem_currentkeyboard[i] && !gem_previouskeyboard[i])
			SDL_PrivateKeyboard(SDL_PRESSED,
				SDL_Atari_TranslateKey(i, &keysym, SDL_TRUE));
			
		/* Key unpressed ? */
		if (gem_previouskeyboard[i] && !gem_currentkeyboard[i])
			SDL_PrivateKeyboard(SDL_RELEASED,
				SDL_Atari_TranslateKey(i, &keysym, SDL_FALSE));
	}

	SDL_memcpy(gem_previouskeyboard,gem_currentkeyboard,sizeof(gem_previouskeyboard));

	/* Refresh window name ? */
	if (GEM_refresh_name) {
		const char *window_name =
			(SDL_GetAppState() & SDL_APPACTIVE)
			? GEM_title_name : GEM_icon_name;
		if (window_name) {
			wind_set(GEM_handle,WF_NAME,
				(short)(((unsigned long)window_name)>>16),
				(short)(((unsigned long)window_name) & 0xffff),
				0,0);
		}
		GEM_refresh_name = SDL_FALSE;
	}
}

static int do_messages(_THIS, short *message)
{
	int quit, check_mouse_mode;
	short x2,y2,w2,h2;

	quit = check_mouse_mode = 0;
	switch (message[0]) {
		case WM_CLOSED:
		case AP_TERM:    
			SDL_PrivateQuit();
			quit=1;
			break;
		case WM_MOVED:
			wind_set(message[3],WF_CURRXYWH,message[4],message[5],message[6],message[7]);
			break;
		case WM_TOPPED:
			wind_set(message[3],WF_TOP,message[4],0,0,0);
			/* Continue with TOP event processing */
		case WM_ONTOP:
			SDL_PrivateAppActive(1, SDL_APPINPUTFOCUS);
			if (VDI_setpalette) {
				VDI_setpalette(this, VDI_curpalette);
			}
			check_mouse_mode = 1;
			break;
		case WM_REDRAW:
			if (!GEM_lock_redraw) {
				GEM_wind_redraw(this, message[3],&message[4]);
			}
			break;
		case WM_ICONIFY:
		case WM_ALLICONIFY:
			wind_set(message[3],WF_ICONIFY,message[4],message[5],message[6],message[7]);
			/* If we're active, make ourselves inactive */
			if ( SDL_GetAppState() & SDL_APPACTIVE ) {
				/* Send an internal deactivate event */
				SDL_PrivateAppActive(0, SDL_APPACTIVE);
			}
			/* Update window title */
			if (GEM_refresh_name && GEM_icon_name) {
				wind_set(GEM_handle,WF_NAME,(short)(((unsigned long)GEM_icon_name)>>16),(short)(((unsigned long)GEM_icon_name) & 0xffff),0,0);
				GEM_refresh_name = SDL_FALSE;
			}
			check_mouse_mode = 1;
			break;
		case WM_UNICONIFY:
			wind_set(message[3],WF_UNICONIFY,message[4],message[5],message[6],message[7]);
			/* If we're not active, make ourselves active */
			if ( !(SDL_GetAppState() & SDL_APPACTIVE) ) {
				/* Send an internal activate event */
				SDL_PrivateAppActive(1, SDL_APPACTIVE);
			}
			if (GEM_refresh_name && GEM_title_name) {
				wind_set(GEM_handle,WF_NAME,(short)(((unsigned long)GEM_title_name)>>16),(short)(((unsigned long)GEM_title_name) & 0xffff),0,0);
				GEM_refresh_name = SDL_FALSE;
			}
			check_mouse_mode = 1;
			break;
		case WM_SIZED:
			wind_set (message[3], WF_CURRXYWH, message[4], message[5], message[6], message[7]);
			wind_get (message[3], WF_WORKXYWH, &x2, &y2, &w2, &h2);
			GEM_win_fulled = SDL_FALSE;		/* Cancel maximized flag */
			GEM_lock_redraw = SDL_TRUE;		/* Prevent redraw till buffers resized */
			SDL_PrivateResize(w2, h2);
			break;
		case WM_FULLED:
			{
				short x,y,w,h;

				if (GEM_win_fulled) {
					wind_get (message[3], WF_PREVXYWH, &x, &y, &w, &h);
					GEM_win_fulled = SDL_FALSE;
				} else {
					x = GEM_desk_x;
					y = GEM_desk_y;
					w = GEM_desk_w;
					h = GEM_desk_h;
					GEM_win_fulled = SDL_TRUE;
				}
				wind_set (message[3], WF_CURRXYWH, x, y, w, h);
				wind_get (message[3], WF_WORKXYWH, &x2, &y2, &w2, &h2);
				GEM_lock_redraw = SDL_TRUE;		/* Prevent redraw till buffers resized */
				SDL_PrivateResize(w2, h2);
			}
			break;
		case WM_BOTTOMED:
			wind_set(message[3],WF_BOTTOM,0,0,0,0);
			/* Continue with BOTTOM event processing */
		case WM_UNTOPPED:
			SDL_PrivateAppActive(0, SDL_APPINPUTFOCUS);
			if (VDI_setpalette) {
				VDI_setpalette(this, VDI_oldpalette);
			}
			check_mouse_mode = 1;
			break;
	}

	if (check_mouse_mode) {
		GEM_CheckMouseMode(this);
	}
	
	return quit;
}

static void do_keyboard(short kc, short ks)
{
	int scancode;

	if (kc) {
		scancode=(kc>>8) & (ATARIBIOS_MAXKEYS-1);
		gem_currentkeyboard[scancode]=0xFF;
	}

	/* Read special keys */
	if (ks & K_RSHIFT)
		gem_currentkeyboard[SCANCODE_RIGHTSHIFT]=0xFF;
	if (ks & K_LSHIFT)
		gem_currentkeyboard[SCANCODE_LEFTSHIFT]=0xFF;
	if (ks & K_CTRL)
		gem_currentkeyboard[SCANCODE_LEFTCONTROL]=0xFF;
	if (ks & K_ALT)
		gem_currentkeyboard[SCANCODE_LEFTALT]=0xFF;
}

static void do_mouse(_THIS, short mx, short my, short mb, short ks)
{
	static short prevmousex=0, prevmousey=0, prevmouseb=0;
	short x2, y2, w2, h2;

	/* Don't return mouse events if out of window */
	if ((SDL_GetAppState() & SDL_APPMOUSEFOCUS)==0) {
		return;
	}

	/* Retrieve window coords, and generate mouse events accordingly */
	x2 = y2 = 0;
	w2 = VDI_w;
	h2 = VDI_h;
	if ((!GEM_fullscreen) && (GEM_handle>=0)) {
		wind_get (GEM_handle, WF_WORKXYWH, &x2, &y2, &w2, &h2);

		/* Do not generate mouse button event if out of window working area */
		if ((mx<x2) || (mx>=x2+w2) || (my<y2) || (my>=y2+h2)) {
			mb=prevmouseb;
		}
	}

	/* Mouse motion ? */
	if (GEM_mouse_relative) {
		if (GEM_usedevmouse) {
			SDL_AtariDevMouse_PostMouseEvents(this, SDL_FALSE);
		} else {
			SDL_AtariXbios_PostMouseEvents(this, SDL_FALSE);
		}
	} else {
		if ((prevmousex!=mx) || (prevmousey!=my)) {
			int posx, posy;

			/* Give mouse position relative to window position */
			posx = mx - x2;
			if (posx<0) posx = 0;
			if (posx>w2) posx = w2-1;
			posy = my - y2;
			if (posy<0) posy = 0;
			if (posy>h2) posy = h2-1;

			SDL_PrivateMouseMotion(0, 0, posx, posy);
		}
		prevmousex = mx;
		prevmousey = my;
	}

	/* Mouse button ? */
	if (prevmouseb!=mb) {
		int i;

		for (i=0;i<3;i++) {
			int curbutton, prevbutton;
		
			curbutton = mb & (1<<i);
			prevbutton = prevmouseb & (1<<i);
		
			if (curbutton && !prevbutton) {
				SDL_PrivateMouseButton(SDL_PRESSED, i+1, 0, 0);
			}
			if (!curbutton && prevbutton) {
				SDL_PrivateMouseButton(SDL_RELEASED, i+1, 0, 0);
			}
		}
		prevmouseb = mb;
	}

	/* Read special keys */
	if (ks & K_RSHIFT)
		gem_currentkeyboard[SCANCODE_RIGHTSHIFT]=0xFF;
	if (ks & K_LSHIFT)
		gem_currentkeyboard[SCANCODE_LEFTSHIFT]=0xFF;
	if (ks & K_CTRL)
		gem_currentkeyboard[SCANCODE_LEFTCONTROL]=0xFF;
	if (ks & K_ALT)
		gem_currentkeyboard[SCANCODE_LEFTALT]=0xFF;
}
