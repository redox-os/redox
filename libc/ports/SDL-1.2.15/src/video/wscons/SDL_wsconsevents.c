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

#include <sys/types.h>
#include <dev/wscons/wsdisplay_usl_io.h>
#include <sys/ioctl.h>
#include <fcntl.h>    
#include <unistd.h>  
#include <termios.h>
#include <errno.h> 
#include <string.h>

#include "SDL.h"
#include "../../events/SDL_sysevents.h"
#include "../../events/SDL_events_c.h"
#include "SDL_wsconsvideo.h"
#include "SDL_wsconsevents_c.h"

static int posted = 0;

int WSCONS_InitKeyboard(_THIS)
{
  struct termios tty;

  if (ioctl(private->fd, WSKBDIO_GTYPE, &private->kbdType) == -1) {
    WSCONS_ReportError("cannot get keyboard type: %s", strerror(errno));
    return -1;
  }

  if (tcgetattr(private->fd, &private->saved_tty) == -1) {
    WSCONS_ReportError("cannot get terminal attributes: %s", strerror(errno));
    return -1;
  }
  private->did_save_tty = 1;
  tty = private->saved_tty;
  tty.c_iflag = IGNPAR | IGNBRK;
  tty.c_oflag = 0;
  tty.c_cflag = CREAD | CS8;
  tty.c_lflag = 0;
  tty.c_cc[VTIME] = 0;
  tty.c_cc[VMIN] = 1;
  cfsetispeed(&tty, 9600);
  cfsetospeed(&tty, 9600);
  if (tcsetattr(private->fd, TCSANOW, &tty) < 0) {
    WSCONS_ReportError("cannot set terminal attributes: %s", strerror(errno));
    return -1;
  }
  if (ioctl(private->fd, KDSKBMODE, K_RAW) == -1) {
    WSCONS_ReportError("cannot set raw keyboard mode: %s", strerror(errno));
    return -1;
  }

  return 0;
}

void WSCONS_ReleaseKeyboard(_THIS)
{
  if (private->fd != -1) {
    if (ioctl(private->fd, KDSKBMODE, K_XLATE) == -1) {
      WSCONS_ReportError("cannot restore keyboard to translated mode: %s",
			 strerror(errno));
    }
    if (private->did_save_tty) {
      if (tcsetattr(private->fd, TCSANOW, &private->saved_tty) < 0) {
	WSCONS_ReportError("cannot restore keynoard attributes: %s",
			   strerror(errno));
      }
    }
  }
}

static void updateMouse()
{
}

static SDLKey keymap[128];

static SDL_keysym *TranslateKey(int scancode, SDL_keysym *keysym)
{
  keysym->scancode = scancode;
  keysym->sym = SDLK_UNKNOWN;
  keysym->mod = KMOD_NONE;

  if (scancode < SDL_arraysize(keymap))
    keysym->sym = keymap[scancode];

  if (keysym->sym == SDLK_UNKNOWN)
    printf("Unknown mapping for scancode %d\n", scancode);

  return keysym;
}

static void updateKeyboard(_THIS)
{
  unsigned char buf[100];
  SDL_keysym keysym;
  int n, i;

  if ((n = read(private->fd, buf, sizeof(buf))) > 0) {
    for (i = 0; i < n; i++) {
      unsigned char c = buf[i] & 0x7f;
      if (c == 224) // special key prefix -- what should we do with it?
	continue;
      posted += SDL_PrivateKeyboard((buf[i] & 0x80) ? SDL_RELEASED : SDL_PRESSED,
				    TranslateKey(c, &keysym));
    }
  }
}

void WSCONS_PumpEvents(_THIS)
{
  do {
    posted = 0;
    updateMouse();
    updateKeyboard(this);
  } while (posted);
}

void WSCONS_InitOSKeymap(_THIS)
{
  int i;

  /* Make sure unknown keys are mapped correctly */
  for (i=0; i < SDL_arraysize(keymap); i++) {
    keymap[i] = SDLK_UNKNOWN;
  }

  switch (private->kbdType) {
#ifdef WSKBD_TYPE_ZAURUS
  case WSKBD_TYPE_ZAURUS:
    /* top row */
    keymap[2] = SDLK_1;
    keymap[3] = SDLK_2;
    keymap[4] = SDLK_3;
    keymap[5] = SDLK_4;
    keymap[6] = SDLK_5;
    keymap[7] = SDLK_6;
    keymap[8] = SDLK_7;
    keymap[9] = SDLK_8;
    keymap[10] = SDLK_9;
    keymap[11] = SDLK_0;
    keymap[14] = SDLK_BACKSPACE;
    
    /* second row */
    keymap[16] = SDLK_q;
    keymap[17] = SDLK_w;
    keymap[18] = SDLK_e;
    keymap[19] = SDLK_r;
    keymap[20] = SDLK_t;
    keymap[21] = SDLK_y;
    keymap[22] = SDLK_u;
    keymap[23] = SDLK_i;
    keymap[24] = SDLK_o;
    keymap[25] = SDLK_p;

    /* third row */
    keymap[15] = SDLK_TAB;
    keymap[30] = SDLK_a;
    keymap[31] = SDLK_s;
    keymap[32] = SDLK_d;
    keymap[33] = SDLK_f;
    keymap[34] = SDLK_g;
    keymap[35] = SDLK_h;
    keymap[36] = SDLK_j;
    keymap[37] = SDLK_k;
    keymap[38] = SDLK_l;

    /* fourth row */
    keymap[42] = SDLK_LSHIFT;
    keymap[44] = SDLK_z;
    keymap[45] = SDLK_x;
    keymap[46] = SDLK_c;
    keymap[47] = SDLK_v;
    keymap[48] = SDLK_b;
    keymap[49] = SDLK_n;
    keymap[50] = SDLK_m;
    keymap[54] = SDLK_RSHIFT;
    keymap[28] = SDLK_RETURN;

    /* fifth row */
    keymap[56] = SDLK_LALT;
    keymap[29] = SDLK_LCTRL;
    /* keymap[56] = ; */
    keymap[0] = SDLK_LSUPER;
    keymap[12] = SDLK_MINUS;
    keymap[57] = SDLK_SPACE;
    keymap[51] = SDLK_COMMA;
    keymap[52] = SDLK_PERIOD;

    /* misc */
    keymap[59] = SDLK_F1;
    keymap[60] = SDLK_F2;
    keymap[61] = SDLK_F3;
    keymap[62] = SDLK_F4;
    keymap[63] = SDLK_F5;
    keymap[1] = SDLK_ESCAPE;
    /* keymap[28] = SDLK_KP_ENTER; */
    keymap[72] = SDLK_UP;
    keymap[75] = SDLK_LEFT;
    keymap[77] = SDLK_RIGHT;
    keymap[80] = SDLK_DOWN;
    break;
#endif /* WSKBD_TYPE_ZAURUS */

  default:
    WSCONS_ReportError("Unable to map keys for keyboard type %u", 
		       private->kbdType);
    break;
  }
}

/* end of SDL_wsconsevents.c ... */

