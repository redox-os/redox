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
    slouken@libsdl.org
*/
#include "SDL_config.h"

/*
     File added by Alan Buckley (alan_baa@hotmail.com) for RISC OS compatability
	 27 March 2003

     Implements keyboard setup, event pump and keyboard and mouse polling
*/


#include "SDL.h"
#include "../../timer/SDL_timer_c.h"
#include "../../events/SDL_sysevents.h"
#include "../../events/SDL_events_c.h"
#include "../SDL_cursor_c.h"
#include "SDL_riscosvideo.h"
#include "SDL_riscosevents_c.h"

#include "memory.h"
#include "stdlib.h"
#include "ctype.h"

#include "kernel.h"
#include "swis.h"

/* The translation table from a RISC OS internal key numbers to a SDL keysym */
static SDLKey RO_keymap[SDLK_LAST];

/* RISC OS Key codes */
#define ROKEY_SHIFT 0
#define ROKEY_CTRL  1
#define ROKEY_ALT   2
/* Left shift is first key we will check for */
#define ROKEY_LEFT_SHIFT 3

/* Need to ignore mouse buttons as they are processed separately */
#define ROKEY_LEFT_MOUSE   9
#define ROKEY_CENTRE_MOUSE 10
#define ROKEY_RIGHT_MOUSE  11

/* No key has been pressed return value*/
#define ROKEY_NONE 255

/* Id of last key in keyboard */
#define ROKEY_LAST_KEY  124

/* Size of array for all keys */
#define ROKEYBD_ARRAYSIZE 125

static char RO_pressed[ROKEYBD_ARRAYSIZE];

static SDL_keysym *TranslateKey(int intkey, SDL_keysym *keysym, int pressed);

void RISCOS_PollMouse(_THIS);
void RISCOS_PollKeyboard();

void RISCOS_PollMouseHelper(_THIS, int fullscreen);

#if SDL_THREADS_DISABLED
extern void DRenderer_FillBuffers();

/* Timer running function */
extern void RISCOS_CheckTimer();

#endif

void FULLSCREEN_PumpEvents(_THIS)
{
    /* Current implementation requires keyboard and mouse polling */
	RISCOS_PollKeyboard();
	RISCOS_PollMouse(this);
#if SDL_THREADS_DISABLED
//	DRenderer_FillBuffers();
	if (SDL_timer_running) RISCOS_CheckTimer();
#endif
}


void RISCOS_InitOSKeymap(_THIS)
{
	int i;

	/* Map the VK keysyms */
	for ( i=0; i<SDL_arraysize(RO_keymap); ++i )
		RO_keymap[i] = SDLK_UNKNOWN;

  RO_keymap[3] = SDLK_LSHIFT;
  RO_keymap[4] = SDLK_LCTRL;
  RO_keymap[5] = SDLK_LALT;
  RO_keymap[6] = SDLK_RSHIFT;
  RO_keymap[7] = SDLK_RCTRL;
  RO_keymap[8] = SDLK_RALT;
  RO_keymap[16] = SDLK_q;
  RO_keymap[17] = SDLK_3;
  RO_keymap[18] = SDLK_4;
  RO_keymap[19] = SDLK_5;
  RO_keymap[20] = SDLK_F4;
  RO_keymap[21] = SDLK_8;
  RO_keymap[22] = SDLK_F7;
  RO_keymap[23] = SDLK_MINUS,
  RO_keymap[25] = SDLK_LEFT;
  RO_keymap[26] = SDLK_KP6;
  RO_keymap[27] = SDLK_KP7;
  RO_keymap[28] = SDLK_F11;
  RO_keymap[29] = SDLK_F12;
  RO_keymap[30] = SDLK_F10;
  RO_keymap[31] = SDLK_SCROLLOCK;
  RO_keymap[32] = SDLK_PRINT;
  RO_keymap[33] = SDLK_w;
  RO_keymap[34] = SDLK_e;
  RO_keymap[35] = SDLK_t;
  RO_keymap[36] = SDLK_7;
  RO_keymap[37] = SDLK_i;
  RO_keymap[38] = SDLK_9;
  RO_keymap[39] = SDLK_0;
  RO_keymap[41] = SDLK_DOWN;
  RO_keymap[42] = SDLK_KP8;
  RO_keymap[43] = SDLK_KP9;
  RO_keymap[44] = SDLK_BREAK;
  RO_keymap[45] = SDLK_BACKQUOTE;
/*  RO_keymap[46] = SDLK_currency; TODO: Figure out if this has a value */
  RO_keymap[47] = SDLK_BACKSPACE;
  RO_keymap[48] = SDLK_1;
  RO_keymap[49] = SDLK_2;
  RO_keymap[50] = SDLK_d;
  RO_keymap[51] = SDLK_r;
  RO_keymap[52] = SDLK_6;
  RO_keymap[53] = SDLK_u;
  RO_keymap[54] = SDLK_o;
  RO_keymap[55] = SDLK_p;
  RO_keymap[56] = SDLK_LEFTBRACKET;
  RO_keymap[57] = SDLK_UP;
  RO_keymap[58] = SDLK_KP_PLUS;
  RO_keymap[59] = SDLK_KP_MINUS;
  RO_keymap[60] = SDLK_KP_ENTER;
  RO_keymap[61] = SDLK_INSERT;
  RO_keymap[62] = SDLK_HOME;
  RO_keymap[63] = SDLK_PAGEUP;
  RO_keymap[64] = SDLK_CAPSLOCK;
  RO_keymap[65] = SDLK_a;
  RO_keymap[66] = SDLK_x;
  RO_keymap[67] = SDLK_f;
  RO_keymap[68] = SDLK_y;
  RO_keymap[69] = SDLK_j;
  RO_keymap[70] = SDLK_k;
  RO_keymap[72] = SDLK_SEMICOLON;
  RO_keymap[73] = SDLK_RETURN;
  RO_keymap[74] = SDLK_KP_DIVIDE;
  RO_keymap[76] = SDLK_KP_PERIOD;
  RO_keymap[77] = SDLK_NUMLOCK;
  RO_keymap[78] = SDLK_PAGEDOWN;
  RO_keymap[79] = SDLK_QUOTE;
  RO_keymap[81] = SDLK_s;
  RO_keymap[82] = SDLK_c;
  RO_keymap[83] = SDLK_g;
  RO_keymap[84] = SDLK_h;
  RO_keymap[85] = SDLK_n;
  RO_keymap[86] = SDLK_l;
  RO_keymap[87] = SDLK_SEMICOLON;
  RO_keymap[88] = SDLK_RIGHTBRACKET;
  RO_keymap[89] = SDLK_DELETE;
  RO_keymap[90] = SDLK_KP_MINUS;
  RO_keymap[91] = SDLK_KP_MULTIPLY;
  RO_keymap[93] = SDLK_EQUALS;
  RO_keymap[94] = SDLK_BACKSLASH;
  RO_keymap[96] = SDLK_TAB;
  RO_keymap[97] = SDLK_z;
  RO_keymap[98] = SDLK_SPACE;
  RO_keymap[99] = SDLK_v;
  RO_keymap[100] = SDLK_b;
  RO_keymap[101] = SDLK_m;
  RO_keymap[102] = SDLK_COMMA;
  RO_keymap[103] = SDLK_PERIOD;
  RO_keymap[104] = SDLK_SLASH;
  RO_keymap[105] = SDLK_END;
  RO_keymap[106] = SDLK_KP0;
  RO_keymap[107] = SDLK_KP1;
  RO_keymap[108] = SDLK_KP3;
  RO_keymap[112] = SDLK_ESCAPE;
  RO_keymap[113] = SDLK_F1;
  RO_keymap[114] = SDLK_F2;
  RO_keymap[115] = SDLK_F3;
  RO_keymap[116] = SDLK_F5;
  RO_keymap[117] = SDLK_F6;
  RO_keymap[118] = SDLK_F8;
  RO_keymap[119] = SDLK_F9;
  RO_keymap[120] = SDLK_HASH;
  RO_keymap[121] = SDLK_RIGHT;
  RO_keymap[122] = SDLK_KP4;
  RO_keymap[123] = SDLK_KP5;
  RO_keymap[124] = SDLK_KP2;

  SDL_memset(RO_pressed, 0, ROKEYBD_ARRAYSIZE);
}


/* Variable for mouse relative processing */
int mouse_relative = 0;

/* Check to see if we need to enter or leave mouse relative mode */

void RISCOS_CheckMouseMode(_THIS)
{
    /* If the mouse is hidden and input is grabbed, we use relative mode */
    if ( !(SDL_cursorstate & CURSOR_VISIBLE) &&
        (this->input_grab != SDL_GRAB_OFF) ) {
            mouse_relative = 1;
     } else {
            mouse_relative = 0;
     }
}


void RISCOS_PollMouse(_THIS)
{
   RISCOS_PollMouseHelper(this, 1);
}

extern int mouseInWindow;

void WIMP_PollMouse(_THIS)
{
   /* Only poll when mouse is over the window */
   if (!mouseInWindow) return;

   RISCOS_PollMouseHelper(this, 0);
}

/* Static variables so only changes are reported */
static Sint16 last_x = -1, last_y = -1;
static int last_buttons = 0;

/* Share routine between WIMP and FULLSCREEN for polling mouse and
   passing on events */
void RISCOS_PollMouseHelper(_THIS, int fullscreen)
{
    _kernel_swi_regs regs;
    static int starting = 1;

    if (_kernel_swi(OS_Mouse, &regs, &regs) == NULL)
    {
       Sint16 new_x = regs.r[0]; /* Initialy get as OS units */
       Sint16 new_y = regs.r[1];

       /* Discard mouse events until they let go of the mouse after starting */
       if (starting && regs.r[2] != 0)
         return;
       else
         starting = 0;

       if (new_x != last_x || new_y != last_y || last_buttons != regs.r[2])
       {
          /* Something changed so generate appropriate events */
          int topLeftX, topLeftY;  /* Top left OS units */
          int x, y;                /* Mouse position in SDL pixels */

          if (fullscreen)
          {
             topLeftX = 0;
             topLeftY = (this->hidden->height << this->hidden->yeig) - 1;
          } else
          {
             int window_state[9];

	         /* Get current window state */
		     window_state[0] = this->hidden->window_handle;
		     regs.r[1] = (unsigned int)window_state;
		     _kernel_swi(Wimp_GetWindowState, &regs, &regs);

             topLeftX = window_state[1];
             topLeftY = window_state[4];
          }

		  /* Convert co-ordinates to workspace */
		  x = new_x - topLeftX;
          y = topLeftY - new_y; /* Y goes from top of window/screen */

	 	  /* Convert OS units to pixels */
	      x >>= this->hidden->xeig;
		  y >>= this->hidden->yeig;

          if (last_x != new_x || last_y != new_y)
          {
             if (mouse_relative)
             {
                int centre_x = SDL_VideoSurface->w/2;
                int centre_y = SDL_VideoSurface->h/2;

                if (centre_x != x || centre_y != y)
                {
                   if (SDL_VideoSurface) SDL_PrivateMouseMotion(0,1,x - centre_x, y - centre_y);
                   last_x = topLeftX + (centre_x << this->hidden->xeig);
                   last_y = topLeftY - (centre_y << this->hidden->yeig);

                   /* Re-centre the mouse pointer, so we still get relative
                      movement when the mouse is at the edge of the window
                      or screen.
                   */
                   {
                      unsigned char block[5];

                      block[0] = 3; /* OSWORD move pointer sub-reason code */
                      block[1] = last_x & 0xFF;
                      block[2] = (last_x >> 8) & 0xFF;
                      block[3] = last_y & 0xFF;
                      block[4] = (last_y >> 8) & 0xFF;
                       
                      regs.r[0] = 21; /* OSWORD pointer stuff code */
                      regs.r[1] = (int)block;
                      _kernel_swi(OS_Word, &regs, &regs);
               	   }
                }
             } else
             {
                last_x = new_x;
                last_y = new_y;
                SDL_PrivateMouseMotion(0,0,x,y);
             }
          }

          if (last_buttons != regs.r[2])
          {
             int changed = last_buttons ^ regs.r[2];
             last_buttons = regs.r[2];
             if (changed & 4) SDL_PrivateMouseButton((last_buttons & 4) ? SDL_PRESSED : SDL_RELEASED, SDL_BUTTON_LEFT, 0, 0);
             if (changed & 2) SDL_PrivateMouseButton((last_buttons & 2) ? SDL_PRESSED : SDL_RELEASED, SDL_BUTTON_MIDDLE, 0, 0);
             if (changed & 1) SDL_PrivateMouseButton((last_buttons & 1) ? SDL_PRESSED : SDL_RELEASED, SDL_BUTTON_RIGHT, 0, 0);
          }
       }
    }
}

void RISCOS_PollKeyboard()
{
	int which_key = ROKEY_LEFT_SHIFT;
	int j;
	int min_key, max_key;
	SDL_keysym key;

	/* Scan the keyboard to see what is pressed */
	while (which_key <= ROKEY_LAST_KEY)
	{
		which_key = (_kernel_osbyte(121, which_key, 0) & 0xFF);
	    if (which_key != ROKEY_NONE)
	    {
		    switch(which_key)
		    {
		    /* Skip over mouse keys */
		    case ROKEY_LEFT_MOUSE:
		    case ROKEY_CENTRE_MOUSE:
		    case ROKEY_RIGHT_MOUSE:
				which_key = ROKEY_RIGHT_MOUSE;
				break;

            /* Ignore keys that cause 2 internal number to be generated */
			case 71: case 24: case 87: case 40:
			    break;

            /* Ignore break as it can be latched on */
            case 44:
                break;

			default:
				RO_pressed[which_key] += 2;
				break;
		    }
			which_key++;
		}
	}

	/* Generate key released messages */
	min_key = ROKEY_LAST_KEY+1;
	max_key = ROKEY_LEFT_SHIFT;

	for (j = ROKEY_LEFT_SHIFT; j <= ROKEY_LAST_KEY; j++)
	{
		if (RO_pressed[j])
		{
			if (RO_pressed[j] == 1)
			{
				RO_pressed[j] = 0;
				SDL_PrivateKeyboard(SDL_RELEASED, TranslateKey(j,&key,0));
			} else 
			{
				if (j < min_key) min_key = j;
				if (j > max_key) max_key = j;
			}
		}
	}

	/* Generate key pressed messages */
	for (j = min_key; j <= max_key; j++)
	{
		if (RO_pressed[j])
		{
			if (RO_pressed[j] == 2)
			{
				SDL_PrivateKeyboard(SDL_PRESSED,TranslateKey(j,&key,1));
			}
			RO_pressed[j] = 1;
		}
	}
}

static SDL_keysym *TranslateKey(int intkey, SDL_keysym *keysym, int pressed)
{
	/* Set the keysym information */
	keysym->scancode = (unsigned char) intkey;
	keysym->sym = RO_keymap[intkey];
	keysym->mod = KMOD_NONE;
	keysym->unicode = 0;
	if ( pressed && SDL_TranslateUNICODE )
	{
		int state;
		int ch;

		state = (_kernel_osbyte(202, 0, 255) & 0xFF);

		/*TODO: Take into account other keyboard layouts */

		ch = keysym->sym; /* This should handle most unshifted keys */

        if (intkey < 9 || ch == SDLK_UNKNOWN)
        {
           ch = 0;

        } else if (state & 64) /* Control on */
		{
			ch = ch & 31;

		} else 
		{
			int topOfKey = 0;
            if (state & 8) /* Shift on */
			{
				topOfKey = 1;
			}

			if ((state & 16) == 0) /* Caps lock is on */
			{
			   if (ch >= SDLK_a && ch <= SDLK_z)
			   {
 				  if ((state & 128) == 0) /* Shift Enable off */
				  {
				     /* All letter become upper case */
				 	 topOfKey = 1;
				  } else
				  {
				     /* Shift+Letters gives lower case */
				     topOfKey = 1 - topOfKey;
				  }
		       }
			}

			if (topOfKey)
			{
				/* Key produced with shift held down */

				/* Letters just give upper case version */
				if (ch >= SDLK_a && ch <= SDLK_z) ch = toupper(ch);
				else
				{
					switch(ch)
					{
					case SDLK_HASH:   ch = '~'; break;
					case SDLK_QUOTE:  ch = '@'; break;
					case SDLK_COMMA:  ch = '<'; break;
					case SDLK_MINUS:  ch = '_'; break;
					case SDLK_PERIOD: ch = '>'; break;
					case SDLK_SLASH:  ch = '?'; break;

					case SDLK_0: ch = ')'; break;
					case SDLK_1: ch = '!'; break;
					case SDLK_2: ch = '"'; break;
					case SDLK_3: ch = '£'; break;
					case SDLK_4: ch = '$'; break;
					case SDLK_5: ch = '%'; break;
					case SDLK_6: ch = '^'; break;
					case SDLK_7: ch = '&'; break;
					case SDLK_8: ch = '*'; break;
					case SDLK_9: ch = '('; break;

					case SDLK_SEMICOLON:    ch = ':'; break;
					case SDLK_EQUALS:       ch = '+'; break;
					case SDLK_LEFTBRACKET:  ch = '{'; break;
					case SDLK_BACKSLASH:    ch = '|'; break;
					case SDLK_RIGHTBRACKET: ch = '}'; break;
					case SDLK_BACKQUOTE:    ch = '¬'; break;

					default:
						ch = 0; /* Map to zero character if we don't understand it */
						break;
					}
				}

			} else if (ch > 126)
			{
				/* SDL key code < 126 map directly onto their Unicode equivalents */
				/* Keypad 0 to 9 maps to numeric equivalent */
				if (ch >= SDLK_KP0 && ch <= SDLK_KP9) ch = ch - SDLK_KP0 + '0';
				else
				{
					/* Following switch maps other keys that produce an Ascii value */
					switch(ch)
					{
					case SDLK_KP_PERIOD:   ch = '.'; break;
					case SDLK_KP_DIVIDE:   ch = '/'; break;
					case SDLK_KP_MULTIPLY: ch = '*'; break;
					case SDLK_KP_MINUS:    ch = '-'; break;
					case SDLK_KP_PLUS:     ch = '+'; break;
					case SDLK_KP_EQUALS:   ch = '='; break;

					default:
						/* If we don't know what it is set the Unicode to 0 */
						ch = 0;
						break;
					}
				}
			}			
		}
				
		keysym->unicode = ch;
	}
	return(keysym);
}

/* end of SDL_riscosevents.c ... */

