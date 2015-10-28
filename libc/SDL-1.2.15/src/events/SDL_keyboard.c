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

/* General keyboard handling code for SDL */

#include "SDL_timer.h"
#include "SDL_events.h"
#include "SDL_events_c.h"
#include "SDL_sysevents.h"


/* Global keystate information */
static Uint8  SDL_KeyState[SDLK_LAST];
static SDLMod SDL_ModState;
int SDL_TranslateUNICODE = 0;

static const char *keynames[SDLK_LAST];	/* Array of keycode names */

/*
 * jk 991215 - added
 */
struct {
	int firsttime;    /* if we check against the delay or repeat value */
	int delay;        /* the delay before we start repeating */
	int interval;     /* the delay between key repeat events */
	Uint32 timestamp; /* the time the first keydown event occurred */

	SDL_Event evt;    /* the event we are supposed to repeat */
} SDL_KeyRepeat;

/* Global no-lock-keys support */
static Uint8 SDL_NoLockKeys;

#define SDL_NLK_CAPS 0x01
#define SDL_NLK_NUM  0x02

/* Public functions */
int SDL_KeyboardInit(void)
{
	const char* env;
	SDL_VideoDevice *video = current_video;
	SDL_VideoDevice *this  = current_video;

	/* Set default mode of UNICODE translation */
	SDL_EnableUNICODE(DEFAULT_UNICODE_TRANSLATION);

	/* Initialize the tables */
	SDL_ModState = KMOD_NONE;
	SDL_memset((void*)keynames, 0, sizeof(keynames));
	SDL_memset(SDL_KeyState, 0, sizeof(SDL_KeyState));
	video->InitOSKeymap(this);

	SDL_EnableKeyRepeat(0, 0);

	/* Allow environment override to disable special lock-key behavior */
	SDL_NoLockKeys = 0;
	env = SDL_getenv("SDL_DISABLE_LOCK_KEYS");
	if (env) {
		switch (SDL_atoi(env)) {
			case 1:
				SDL_NoLockKeys = SDL_NLK_CAPS | SDL_NLK_NUM;
				break;
			case 2:
				SDL_NoLockKeys = SDL_NLK_CAPS;
				break;
			case 3:
				SDL_NoLockKeys = SDL_NLK_NUM;
				break;
			default:
				break;
		}
	}

	/* Fill in the blanks in keynames */
	keynames[SDLK_BACKSPACE] = "backspace";
	keynames[SDLK_TAB] = "tab";
	keynames[SDLK_CLEAR] = "clear";
	keynames[SDLK_RETURN] = "return";
	keynames[SDLK_PAUSE] = "pause";
	keynames[SDLK_ESCAPE] = "escape";
	keynames[SDLK_SPACE] = "space";
	keynames[SDLK_EXCLAIM]  = "!";
	keynames[SDLK_QUOTEDBL]  = "\"";
	keynames[SDLK_HASH]  = "#";
	keynames[SDLK_DOLLAR]  = "$";
	keynames[SDLK_AMPERSAND]  = "&";
	keynames[SDLK_QUOTE] = "'";
	keynames[SDLK_LEFTPAREN] = "(";
	keynames[SDLK_RIGHTPAREN] = ")";
	keynames[SDLK_ASTERISK] = "*";
	keynames[SDLK_PLUS] = "+";
	keynames[SDLK_COMMA] = ",";
	keynames[SDLK_MINUS] = "-";
	keynames[SDLK_PERIOD] = ".";
	keynames[SDLK_SLASH] = "/";
	keynames[SDLK_0] = "0";
	keynames[SDLK_1] = "1";
	keynames[SDLK_2] = "2";
	keynames[SDLK_3] = "3";
	keynames[SDLK_4] = "4";
	keynames[SDLK_5] = "5";
	keynames[SDLK_6] = "6";
	keynames[SDLK_7] = "7";
	keynames[SDLK_8] = "8";
	keynames[SDLK_9] = "9";
	keynames[SDLK_COLON] = ":";
	keynames[SDLK_SEMICOLON] = ";";
	keynames[SDLK_LESS] = "<";
	keynames[SDLK_EQUALS] = "=";
	keynames[SDLK_GREATER] = ">";
	keynames[SDLK_QUESTION] = "?";
	keynames[SDLK_AT] = "@";
	keynames[SDLK_LEFTBRACKET] = "[";
	keynames[SDLK_BACKSLASH] = "\\";
	keynames[SDLK_RIGHTBRACKET] = "]";
	keynames[SDLK_CARET] = "^";
	keynames[SDLK_UNDERSCORE] = "_";
	keynames[SDLK_BACKQUOTE] = "`";
	keynames[SDLK_a] = "a";
	keynames[SDLK_b] = "b";
	keynames[SDLK_c] = "c";
	keynames[SDLK_d] = "d";
	keynames[SDLK_e] = "e";
	keynames[SDLK_f] = "f";
	keynames[SDLK_g] = "g";
	keynames[SDLK_h] = "h";
	keynames[SDLK_i] = "i";
	keynames[SDLK_j] = "j";
	keynames[SDLK_k] = "k";
	keynames[SDLK_l] = "l";
	keynames[SDLK_m] = "m";
	keynames[SDLK_n] = "n";
	keynames[SDLK_o] = "o";
	keynames[SDLK_p] = "p";
	keynames[SDLK_q] = "q";
	keynames[SDLK_r] = "r";
	keynames[SDLK_s] = "s";
	keynames[SDLK_t] = "t";
	keynames[SDLK_u] = "u";
	keynames[SDLK_v] = "v";
	keynames[SDLK_w] = "w";
	keynames[SDLK_x] = "x";
	keynames[SDLK_y] = "y";
	keynames[SDLK_z] = "z";
	keynames[SDLK_DELETE] = "delete";

	keynames[SDLK_WORLD_0] = "world 0";
	keynames[SDLK_WORLD_1] = "world 1";
	keynames[SDLK_WORLD_2] = "world 2";
	keynames[SDLK_WORLD_3] = "world 3";
	keynames[SDLK_WORLD_4] = "world 4";
	keynames[SDLK_WORLD_5] = "world 5";
	keynames[SDLK_WORLD_6] = "world 6";
	keynames[SDLK_WORLD_7] = "world 7";
	keynames[SDLK_WORLD_8] = "world 8";
	keynames[SDLK_WORLD_9] = "world 9";
	keynames[SDLK_WORLD_10] = "world 10";
	keynames[SDLK_WORLD_11] = "world 11";
	keynames[SDLK_WORLD_12] = "world 12";
	keynames[SDLK_WORLD_13] = "world 13";
	keynames[SDLK_WORLD_14] = "world 14";
	keynames[SDLK_WORLD_15] = "world 15";
	keynames[SDLK_WORLD_16] = "world 16";
	keynames[SDLK_WORLD_17] = "world 17";
	keynames[SDLK_WORLD_18] = "world 18";
	keynames[SDLK_WORLD_19] = "world 19";
	keynames[SDLK_WORLD_20] = "world 20";
	keynames[SDLK_WORLD_21] = "world 21";
	keynames[SDLK_WORLD_22] = "world 22";
	keynames[SDLK_WORLD_23] = "world 23";
	keynames[SDLK_WORLD_24] = "world 24";
	keynames[SDLK_WORLD_25] = "world 25";
	keynames[SDLK_WORLD_26] = "world 26";
	keynames[SDLK_WORLD_27] = "world 27";
	keynames[SDLK_WORLD_28] = "world 28";
	keynames[SDLK_WORLD_29] = "world 29";
	keynames[SDLK_WORLD_30] = "world 30";
	keynames[SDLK_WORLD_31] = "world 31";
	keynames[SDLK_WORLD_32] = "world 32";
	keynames[SDLK_WORLD_33] = "world 33";
	keynames[SDLK_WORLD_34] = "world 34";
	keynames[SDLK_WORLD_35] = "world 35";
	keynames[SDLK_WORLD_36] = "world 36";
	keynames[SDLK_WORLD_37] = "world 37";
	keynames[SDLK_WORLD_38] = "world 38";
	keynames[SDLK_WORLD_39] = "world 39";
	keynames[SDLK_WORLD_40] = "world 40";
	keynames[SDLK_WORLD_41] = "world 41";
	keynames[SDLK_WORLD_42] = "world 42";
	keynames[SDLK_WORLD_43] = "world 43";
	keynames[SDLK_WORLD_44] = "world 44";
	keynames[SDLK_WORLD_45] = "world 45";
	keynames[SDLK_WORLD_46] = "world 46";
	keynames[SDLK_WORLD_47] = "world 47";
	keynames[SDLK_WORLD_48] = "world 48";
	keynames[SDLK_WORLD_49] = "world 49";
	keynames[SDLK_WORLD_50] = "world 50";
	keynames[SDLK_WORLD_51] = "world 51";
	keynames[SDLK_WORLD_52] = "world 52";
	keynames[SDLK_WORLD_53] = "world 53";
	keynames[SDLK_WORLD_54] = "world 54";
	keynames[SDLK_WORLD_55] = "world 55";
	keynames[SDLK_WORLD_56] = "world 56";
	keynames[SDLK_WORLD_57] = "world 57";
	keynames[SDLK_WORLD_58] = "world 58";
	keynames[SDLK_WORLD_59] = "world 59";
	keynames[SDLK_WORLD_60] = "world 60";
	keynames[SDLK_WORLD_61] = "world 61";
	keynames[SDLK_WORLD_62] = "world 62";
	keynames[SDLK_WORLD_63] = "world 63";
	keynames[SDLK_WORLD_64] = "world 64";
	keynames[SDLK_WORLD_65] = "world 65";
	keynames[SDLK_WORLD_66] = "world 66";
	keynames[SDLK_WORLD_67] = "world 67";
	keynames[SDLK_WORLD_68] = "world 68";
	keynames[SDLK_WORLD_69] = "world 69";
	keynames[SDLK_WORLD_70] = "world 70";
	keynames[SDLK_WORLD_71] = "world 71";
	keynames[SDLK_WORLD_72] = "world 72";
	keynames[SDLK_WORLD_73] = "world 73";
	keynames[SDLK_WORLD_74] = "world 74";
	keynames[SDLK_WORLD_75] = "world 75";
	keynames[SDLK_WORLD_76] = "world 76";
	keynames[SDLK_WORLD_77] = "world 77";
	keynames[SDLK_WORLD_78] = "world 78";
	keynames[SDLK_WORLD_79] = "world 79";
	keynames[SDLK_WORLD_80] = "world 80";
	keynames[SDLK_WORLD_81] = "world 81";
	keynames[SDLK_WORLD_82] = "world 82";
	keynames[SDLK_WORLD_83] = "world 83";
	keynames[SDLK_WORLD_84] = "world 84";
	keynames[SDLK_WORLD_85] = "world 85";
	keynames[SDLK_WORLD_86] = "world 86";
	keynames[SDLK_WORLD_87] = "world 87";
	keynames[SDLK_WORLD_88] = "world 88";
	keynames[SDLK_WORLD_89] = "world 89";
	keynames[SDLK_WORLD_90] = "world 90";
	keynames[SDLK_WORLD_91] = "world 91";
	keynames[SDLK_WORLD_92] = "world 92";
	keynames[SDLK_WORLD_93] = "world 93";
	keynames[SDLK_WORLD_94] = "world 94";
	keynames[SDLK_WORLD_95] = "world 95";

	keynames[SDLK_KP0] = "[0]";
	keynames[SDLK_KP1] = "[1]";
	keynames[SDLK_KP2] = "[2]";
	keynames[SDLK_KP3] = "[3]";
	keynames[SDLK_KP4] = "[4]";
	keynames[SDLK_KP5] = "[5]";
	keynames[SDLK_KP6] = "[6]";
	keynames[SDLK_KP7] = "[7]";
	keynames[SDLK_KP8] = "[8]";
	keynames[SDLK_KP9] = "[9]";
	keynames[SDLK_KP_PERIOD] = "[.]";
	keynames[SDLK_KP_DIVIDE] = "[/]";
	keynames[SDLK_KP_MULTIPLY] = "[*]";
	keynames[SDLK_KP_MINUS] = "[-]";
	keynames[SDLK_KP_PLUS] = "[+]";
	keynames[SDLK_KP_ENTER] = "enter";
	keynames[SDLK_KP_EQUALS] = "equals";

	keynames[SDLK_UP] = "up";
	keynames[SDLK_DOWN] = "down";
	keynames[SDLK_RIGHT] = "right";
	keynames[SDLK_LEFT] = "left";
	keynames[SDLK_DOWN] = "down";
	keynames[SDLK_INSERT] = "insert";
	keynames[SDLK_HOME] = "home";
	keynames[SDLK_END] = "end";
	keynames[SDLK_PAGEUP] = "page up";
	keynames[SDLK_PAGEDOWN] = "page down";

	keynames[SDLK_F1] = "f1";
	keynames[SDLK_F2] = "f2";
	keynames[SDLK_F3] = "f3";
	keynames[SDLK_F4] = "f4";
	keynames[SDLK_F5] = "f5";
	keynames[SDLK_F6] = "f6";
	keynames[SDLK_F7] = "f7";
	keynames[SDLK_F8] = "f8";
	keynames[SDLK_F9] = "f9";
	keynames[SDLK_F10] = "f10";
	keynames[SDLK_F11] = "f11";
	keynames[SDLK_F12] = "f12";
	keynames[SDLK_F13] = "f13";
	keynames[SDLK_F14] = "f14";
	keynames[SDLK_F15] = "f15";

	keynames[SDLK_NUMLOCK] = "numlock";
	keynames[SDLK_CAPSLOCK] = "caps lock";
	keynames[SDLK_SCROLLOCK] = "scroll lock";
	keynames[SDLK_RSHIFT] = "right shift";
	keynames[SDLK_LSHIFT] = "left shift";
	keynames[SDLK_RCTRL] = "right ctrl";
	keynames[SDLK_LCTRL] = "left ctrl";
	keynames[SDLK_RALT] = "right alt";
	keynames[SDLK_LALT] = "left alt";
	keynames[SDLK_RMETA] = "right meta";
	keynames[SDLK_LMETA] = "left meta";
	keynames[SDLK_LSUPER] = "left super";	/* "Windows" keys */
	keynames[SDLK_RSUPER] = "right super";	
	keynames[SDLK_MODE] = "alt gr";
	keynames[SDLK_COMPOSE] = "compose";

	keynames[SDLK_HELP] = "help";
	keynames[SDLK_PRINT] = "print screen";
	keynames[SDLK_SYSREQ] = "sys req";
	keynames[SDLK_BREAK] = "break";
	keynames[SDLK_MENU] = "menu";
	keynames[SDLK_POWER] = "power";
	keynames[SDLK_EURO] = "euro";
	keynames[SDLK_UNDO] = "undo";

	/* Done.  Whew. */
	return(0);
}
void SDL_KeyboardQuit(void)
{
}

/* We lost the keyboard, so post key up messages for all pressed keys */
void SDL_ResetKeyboard(void)
{
	SDL_keysym keysym;
	SDLKey key;

	SDL_memset(&keysym, 0, (sizeof keysym));
	for ( key=SDLK_FIRST; key<SDLK_LAST; ++key ) {
		if ( SDL_KeyState[key] == SDL_PRESSED ) {
			keysym.sym = key;
			SDL_PrivateKeyboard(SDL_RELEASED, &keysym);
		}
	}
	SDL_KeyRepeat.timestamp = 0;
}

int SDL_EnableUNICODE(int enable)
{
	int old_mode;

	old_mode = SDL_TranslateUNICODE;
	if ( enable >= 0 ) {
		SDL_TranslateUNICODE = enable;
	}
	return(old_mode);
}

Uint8 * SDL_GetKeyState (int *numkeys)
{
	if ( numkeys != (int *)0 )
		*numkeys = SDLK_LAST;
	return(SDL_KeyState);
}
SDLMod SDL_GetModState (void)
{
	return(SDL_ModState);
}
void SDL_SetModState (SDLMod modstate)
{
	SDL_ModState = modstate;
}

char *SDL_GetKeyName(SDLKey key)
{
	const char *keyname;

	keyname = NULL;
	if ( key < SDLK_LAST ) {
		keyname = keynames[key];
	}
	if ( keyname == NULL ) {
		keyname = "unknown key";
	}
	/* FIXME: make this function const in 1.3 */
	return (char *)(keyname);
}

/* These are global for SDL_eventloop.c */
int SDL_PrivateKeyboard(Uint8 state, SDL_keysym *keysym)
{
	SDL_Event event;
	int posted, repeatable;
	Uint16 modstate;

	SDL_memset(&event, 0, sizeof(event));

#if 0
printf("The '%s' key has been %s\n", SDL_GetKeyName(keysym->sym), 
				state == SDL_PRESSED ? "pressed" : "released");
#endif
	/* Set up the keysym */
	modstate = (Uint16)SDL_ModState;

	repeatable = 0;

	if ( state == SDL_PRESSED ) {
		keysym->mod = (SDLMod)modstate;
		switch (keysym->sym) {
			case SDLK_UNKNOWN:
				break;
			case SDLK_NUMLOCK:
				modstate ^= KMOD_NUM;
				if ( SDL_NoLockKeys & SDL_NLK_NUM )
					break;
				if ( ! (modstate&KMOD_NUM) )
					state = SDL_RELEASED;
				keysym->mod = (SDLMod)modstate;
				break;
			case SDLK_CAPSLOCK:
				modstate ^= KMOD_CAPS;
				if ( SDL_NoLockKeys & SDL_NLK_CAPS )
					break;
				if ( ! (modstate&KMOD_CAPS) )
					state = SDL_RELEASED;
				keysym->mod = (SDLMod)modstate;
				break;
			case SDLK_LCTRL:
				modstate |= KMOD_LCTRL;
				break;
			case SDLK_RCTRL:
				modstate |= KMOD_RCTRL;
				break;
			case SDLK_LSHIFT:
				modstate |= KMOD_LSHIFT;
				break;
			case SDLK_RSHIFT:
				modstate |= KMOD_RSHIFT;
				break;
			case SDLK_LALT:
				modstate |= KMOD_LALT;
				break;
			case SDLK_RALT:
				modstate |= KMOD_RALT;
				break;
			case SDLK_LMETA:
				modstate |= KMOD_LMETA;
				break;
			case SDLK_RMETA:
				modstate |= KMOD_RMETA;
				break;
			case SDLK_MODE:
				modstate |= KMOD_MODE;
				break;
			default:
				repeatable = 1;
				break;
		}
	} else {
		switch (keysym->sym) {
			case SDLK_UNKNOWN:
				break;
			case SDLK_NUMLOCK:
				if ( SDL_NoLockKeys & SDL_NLK_NUM )
					break;
				/* Only send keydown events */
				return(0);
			case SDLK_CAPSLOCK:
				if ( SDL_NoLockKeys & SDL_NLK_CAPS )
					break;
				/* Only send keydown events */
				return(0);
			case SDLK_LCTRL:
				modstate &= ~KMOD_LCTRL;
				break;
			case SDLK_RCTRL:
				modstate &= ~KMOD_RCTRL;
				break;
			case SDLK_LSHIFT:
				modstate &= ~KMOD_LSHIFT;
				break;
			case SDLK_RSHIFT:
				modstate &= ~KMOD_RSHIFT;
				break;
			case SDLK_LALT:
				modstate &= ~KMOD_LALT;
				break;
			case SDLK_RALT:
				modstate &= ~KMOD_RALT;
				break;
			case SDLK_LMETA:
				modstate &= ~KMOD_LMETA;
				break;
			case SDLK_RMETA:
				modstate &= ~KMOD_RMETA;
				break;
			case SDLK_MODE:
				modstate &= ~KMOD_MODE;
				break;
			default:
				break;
		}
		keysym->mod = (SDLMod)modstate;
	}

	/* Figure out what type of event this is */
	switch (state) {
		case SDL_PRESSED:
			event.type = SDL_KEYDOWN;
			break;
		case SDL_RELEASED:
			event.type = SDL_KEYUP;
			/*
			 * jk 991215 - Added
			 */
			if ( SDL_KeyRepeat.timestamp &&
			     SDL_KeyRepeat.evt.key.keysym.sym == keysym->sym ) {
				SDL_KeyRepeat.timestamp = 0;
			}
			break;
		default:
			/* Invalid state -- bail */
			return(0);
	}

	if ( keysym->sym != SDLK_UNKNOWN ) {
		/* Drop events that don't change state */
		if ( SDL_KeyState[keysym->sym] == state ) {
#if 0
printf("Keyboard event didn't change state - dropped!\n");
#endif
			return(0);
		}

		/* Update internal keyboard state */
		SDL_ModState = (SDLMod)modstate;
		SDL_KeyState[keysym->sym] = state;
	}

	/* Post the event, if desired */
	posted = 0;
	if ( SDL_ProcessEvents[event.type] == SDL_ENABLE ) {
		event.key.state = state;
		event.key.keysym = *keysym;
		/*
		 * jk 991215 - Added
		 */
		if (repeatable && (SDL_KeyRepeat.delay != 0)) {
			SDL_KeyRepeat.evt = event;
			SDL_KeyRepeat.firsttime = 1;
			SDL_KeyRepeat.timestamp=SDL_GetTicks();
		}
		if ( (SDL_EventOK == NULL) || SDL_EventOK(&event) ) {
			posted = 1;
			SDL_PushEvent(&event);
		}
	}
	return(posted);
}

/*
 * jk 991215 - Added
 */
void SDL_CheckKeyRepeat(void)
{
	if ( SDL_KeyRepeat.timestamp ) {
		Uint32 now, interval;

		now = SDL_GetTicks();
		interval = (now - SDL_KeyRepeat.timestamp);
		if ( SDL_KeyRepeat.firsttime ) {
			if ( interval > (Uint32)SDL_KeyRepeat.delay ) {
				SDL_KeyRepeat.timestamp = now;
				SDL_KeyRepeat.firsttime = 0;
			}
		} else {
			if ( interval > (Uint32)SDL_KeyRepeat.interval ) {
				SDL_KeyRepeat.timestamp = now;
				if ( (SDL_EventOK == NULL) || SDL_EventOK(&SDL_KeyRepeat.evt) ) {
					SDL_PushEvent(&SDL_KeyRepeat.evt);
				}
			}
		}
	}
}

int SDL_EnableKeyRepeat(int delay, int interval)
{
	if ( (delay < 0) || (interval < 0) ) {
		SDL_SetError("keyboard repeat value less than zero");
		return(-1);
	}
	SDL_KeyRepeat.firsttime = 0;
	SDL_KeyRepeat.delay = delay;
	SDL_KeyRepeat.interval = interval;
	SDL_KeyRepeat.timestamp = 0;
	return(0);
}

void SDL_GetKeyRepeat(int *delay, int *interval)
{
	*delay = SDL_KeyRepeat.delay;
	*interval = SDL_KeyRepeat.interval;
}

