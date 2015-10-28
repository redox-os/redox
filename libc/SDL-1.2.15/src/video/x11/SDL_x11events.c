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

/* Handle the event stream, converting X11 events into SDL events */

#include <setjmp.h>
#include <X11/Xlib.h>
#include <X11/Xutil.h>
#include <X11/keysym.h>
#ifdef __SVR4
#include <X11/Sunkeysym.h>
#endif
#include <sys/types.h>
#include <sys/time.h>
#include <unistd.h>

#include "SDL_timer.h"
#include "SDL_syswm.h"
#include "../SDL_sysvideo.h"
#include "../../events/SDL_sysevents.h"
#include "../../events/SDL_events_c.h"
#include "SDL_x11video.h"
#include "SDL_x11dga_c.h"
#include "SDL_x11modes_c.h"
#include "SDL_x11image_c.h"
#include "SDL_x11gamma_c.h"
#include "SDL_x11wm_c.h"
#include "SDL_x11mouse_c.h"
#include "SDL_x11events_c.h"


/* Define this if you want to debug X11 events */
/*#define DEBUG_XEVENTS*/

/* The translation tables from an X11 keysym to a SDL keysym */
static SDLKey ODD_keymap[256];
static SDLKey MISC_keymap[256];
SDLKey X11_TranslateKeycode(Display *display, KeyCode kc);

/*
 Pending resize target for ConfigureNotify (so outdated events don't
 cause inappropriate resize events)
*/
int X11_PendingConfigureNotifyWidth = -1;
int X11_PendingConfigureNotifyHeight = -1;

#ifdef X_HAVE_UTF8_STRING
Uint32 Utf8ToUcs4(const Uint8 *utf8)
{
	Uint32 c;
	int i = 1;
	int noOctets = 0;
	int firstOctetMask = 0;
	unsigned char firstOctet = utf8[0];
	if (firstOctet < 0x80) {
		/*
		  Characters in the range:
		    00000000 to 01111111 (ASCII Range)
		  are stored in one octet:
		    0xxxxxxx (The same as its ASCII representation)
		  The least 6 significant bits of the first octet is the most 6 significant nonzero bits
		  of the UCS4 representation.
		*/
		noOctets = 1;
		firstOctetMask = 0x7F;  /* 0(1111111) - The most significant bit is ignored */
	} else if ((firstOctet & 0xE0) /* get the most 3 significant bits by AND'ing with 11100000 */
	              == 0xC0 ) {  /* see if those 3 bits are 110. If so, the char is in this range */
		/*
		  Characters in the range:
		    00000000 10000000 to 00000111 11111111
		  are stored in two octets:
		    110xxxxx 10xxxxxx
		  The least 5 significant bits of the first octet is the most 5 significant nonzero bits
		  of the UCS4 representation.
		*/
		noOctets = 2;
		firstOctetMask = 0x1F;  /* 000(11111) - The most 3 significant bits are ignored */
	} else if ((firstOctet & 0xF0) /* get the most 4 significant bits by AND'ing with 11110000 */
	              == 0xE0) {  /* see if those 4 bits are 1110. If so, the char is in this range */
		/*
		  Characters in the range:
		    00001000 00000000 to 11111111 11111111
		  are stored in three octets:
		    1110xxxx 10xxxxxx 10xxxxxx
		  The least 4 significant bits of the first octet is the most 4 significant nonzero bits
		  of the UCS4 representation.
		*/
		noOctets = 3;
		firstOctetMask = 0x0F; /* 0000(1111) - The most 4 significant bits are ignored */
	} else if ((firstOctet & 0xF8) /* get the most 5 significant bits by AND'ing with 11111000 */
	              == 0xF0) {  /* see if those 5 bits are 11110. If so, the char is in this range */
		/*
		  Characters in the range:
		    00000001 00000000 00000000 to 00011111 11111111 11111111
		  are stored in four octets:
		    11110xxx 10xxxxxx 10xxxxxx 10xxxxxx
		  The least 3 significant bits of the first octet is the most 3 significant nonzero bits
		  of the UCS4 representation.
		*/
		noOctets = 4;
		firstOctetMask = 0x07; /* 11110(111) - The most 5 significant bits are ignored */
	} else if ((firstOctet & 0xFC) /* get the most 6 significant bits by AND'ing with 11111100 */
	              == 0xF8) { /* see if those 6 bits are 111110. If so, the char is in this range */
		/*
		  Characters in the range:
		    00000000 00100000 00000000 00000000 to
		    00000011 11111111 11111111 11111111
		  are stored in five octets:
		    111110xx 10xxxxxx 10xxxxxx 10xxxxxx 10xxxxxx
		  The least 2 significant bits of the first octet is the most 2 significant nonzero bits
		  of the UCS4 representation.
		*/
		noOctets = 5;
		firstOctetMask = 0x03; /* 111110(11) - The most 6 significant bits are ignored */
	} else if ((firstOctet & 0xFE) /* get the most 7 significant bits by AND'ing with 11111110 */
	              == 0xFC) { /* see if those 7 bits are 1111110. If so, the char is in this range */
		/*
		  Characters in the range:
		    00000100 00000000 00000000 00000000 to
		    01111111 11111111 11111111 11111111
		  are stored in six octets:
		    1111110x 10xxxxxx 10xxxxxx 10xxxxxx 10xxxxxx 10xxxxxx
		  The least significant bit of the first octet is the most significant nonzero bit
		  of the UCS4 representation.
		*/
		noOctets = 6;
		firstOctetMask = 0x01; /* 1111110(1) - The most 7 significant bits are ignored */
	} else
		return 0;  /* The given chunk is not a valid UTF-8 encoded Unicode character */
	
	/*
	  The least noOctets significant bits of the first octet is the most 2 significant nonzero bits
	  of the UCS4 representation.
	  The first 6 bits of the UCS4 representation is the least 8-noOctets-1 significant bits of
	  firstOctet if the character is not ASCII. If so, it's the least 7 significant bits of firstOctet.
	  This done by AND'ing firstOctet with its mask to trim the bits used for identifying the
	  number of continuing octets (if any) and leave only the free bits (the x's)
	  Sample:
	  1-octet:    0xxxxxxx  &  01111111 = 0xxxxxxx
	  2-octets:  110xxxxx  &  00011111 = 000xxxxx
	*/
	c = firstOctet & firstOctetMask;
	
	/* Now, start filling c.ucs4 with the bits from the continuing octets from utf8. */
	for (i = 1; i < noOctets; i++) {
		/* A valid continuing octet is of the form 10xxxxxx */
		if ((utf8[i] & 0xC0) /* get the most 2 significant bits by AND'ing with 11000000 */
		    != 0x80) /* see if those 2 bits are 10. If not, the is a malformed sequence. */
			/*The given chunk is a partial sequence at the end of a string that could
			   begin a valid character */
			return 0;
		
		/* Make room for the next 6-bits */
		c <<= 6;
		
		/*
		  Take only the least 6 significance bits of the current octet (utf8[i]) and fill the created room
		  of c.ucs4 with them.
		  This done by AND'ing utf8[i] with 00111111 and the OR'ing the result with c.ucs4.
		*/
		c |= utf8[i] & 0x3F;
	}
	return c;
}

/* Given a UTF-8 encoded string pointed to by utf8 of length length in
   bytes, returns the corresponding UTF-16 encoded string in the
   buffer pointed to by utf16.  The maximum number of UTF-16 encoding
   units (i.e., Unit16s) allowed in the buffer is specified in
   utf16_max_length.  The return value is the number of UTF-16
   encoding units placed in the output buffer pointed to by utf16.

   In case of an error, -1 is returned, leaving some unusable partial
   results in the output buffer.

   The caller must estimate the size of utf16 buffer by itself before
   calling this function.  Insufficient output buffer is considered as
   an error, and once an error occured, this function doesn't give any
   clue how large the result will be.

   The error cases include following:

   - Invalid byte sequences were in the input UTF-8 bytes.  The caller
     has no way to know what point in the input buffer was the
     errornous byte.

   - The input contained a character (a valid UTF-8 byte sequence)
     whose scalar value exceeded the range that UTF-16 can represent
     (i.e., characters whose Unicode scalar value above 0x110000).

   - The output buffer has no enough space to hold entire utf16 data.

   Please note:

   - '\0'-termination is not assumed both on the input UTF-8 string
     and on the output UTF-16 string; any legal zero byte in the input
     UTF-8 string will be converted to a 16-bit zero in output.  As a
     side effect, the last UTF-16 encoding unit stored in the output
     buffer will have a non-zero value if the input UTF-8 was not
     '\0'-terminated.

   - UTF-8 aliases are *not* considered as an error.  They are
     converted to UTF-16.  For example, 0xC0 0xA0, 0xE0 0x80 0xA0, 
     and 0xF0 0x80 0x80 0xA0 are all mapped to a single UTF-16
     encoding unit 0x0020.

   - Three byte UTF-8 sequences whose value corresponds to a surrogate
     code or other reserved scalar value are not considered as an
     error either.  They may cause an invalid UTF-16 data (e.g., those
     containing unpaired surrogates).

*/

static int Utf8ToUtf16(const Uint8 *utf8, const int utf8_length, Uint16 *utf16, const int utf16_max_length) {

    /* p moves over the output buffer.  max_ptr points to the next to the last slot of the buffer.  */
    Uint16 *p = utf16;
    Uint16 const *const max_ptr = utf16 + utf16_max_length;

    /* end_of_input points to the last byte of input as opposed to the next to the last byte.  */
    Uint8 const *const end_of_input = utf8 + utf8_length - 1;

    while (utf8 <= end_of_input) {
	Uint8 const c = *utf8;
	if (p >= max_ptr) {
	    /* No more output space.  */
	    return -1;
	}
	if (c < 0x80) {
	    /* One byte ASCII.  */
	    *p++ = c;
	    utf8 += 1;
	} else if (c < 0xC0) {
	    /* Follower byte without preceeding leader bytes.  */
	    return -1;
	} else if (c < 0xE0) {
	    /* Two byte sequence.  We need one follower byte.  */
	    if (end_of_input - utf8 < 1 || (((utf8[1] ^ 0x80)) & 0xC0)) {
		return -1;
	    }
	    *p++ = (Uint16)(0xCF80 + (c << 6) + utf8[1]);
	    utf8 += 2;
	} else if (c < 0xF0) {
	    /* Three byte sequence.  We need two follower byte.  */
	    if (end_of_input - utf8 < 2 || (((utf8[1] ^ 0x80) | (utf8[2] ^ 0x80)) & 0xC0)) {
		return -1;
	    }
	    *p++ = (Uint16)(0xDF80 + (c << 12) + (utf8[1] << 6) + utf8[2]);
	    utf8 += 3;
	} else if (c < 0xF8) {
	    int plane;
	    /* Four byte sequence.  We need three follower bytes.  */
	    if (end_of_input - utf8 < 3 || (((utf8[1] ^ 0x80) | (utf8[2] ^0x80) | (utf8[3] ^ 0x80)) & 0xC0)) {
		return -1;
	    }
	    plane = (-0xC8 + (c << 2) + (utf8[1] >> 4));
	    if (plane == 0) {
		/* This four byte sequence is an alias that
                   corresponds to a Unicode scalar value in BMP.
		   It fits in an UTF-16 encoding unit.  */
		*p++ = (Uint16)(0xDF80 + (utf8[1] << 12) + (utf8[2] << 6) + utf8[3]);
	    } else if (plane <= 16) {
		/* This is a legal four byte sequence that corresponds to a surrogate pair.  */
		if (p + 1 >= max_ptr) {
		    /* No enough space on the output buffer for the pair.  */
		    return -1;
		}
		*p++ = (Uint16)(0xE5B8 + (c << 8) + (utf8[1] << 2) + (utf8[2] >> 4));
		*p++ = (Uint16)(0xDB80 + ((utf8[2] & 0x0F) << 6) + utf8[3]);
	    } else {
		/* This four byte sequence is out of UTF-16 code space.  */
		return -1;
	    }
	    utf8 += 4;
	} else {
	    /* Longer sequence or unused byte.  */
	    return -1;
	}
    }
    return p - utf16;
}

#endif

/* Check to see if this is a repeated key.
   (idea shamelessly lifted from GII -- thanks guys! :)
 */
static int X11_KeyRepeat(Display *display, XEvent *event)
{
	XEvent peekevent;
	int repeated;

	repeated = 0;
	if ( XPending(display) ) {
		XPeekEvent(display, &peekevent);
		if ( (peekevent.type == KeyPress) &&
		     (peekevent.xkey.keycode == event->xkey.keycode) &&
		     ((peekevent.xkey.time-event->xkey.time) < 2) ) {
			repeated = 1;
			XNextEvent(display, &peekevent);
		}
	}
	return(repeated);
}

/* Note:  The X server buffers and accumulates mouse motion events, so
   the motion event generated by the warp may not appear exactly as we
   expect it to.  We work around this (and improve performance) by only
   warping the pointer when it reaches the edge, and then wait for it.
*/
#define MOUSE_FUDGE_FACTOR	8

static __inline__ int X11_WarpedMotion(_THIS, XEvent *xevent)
{
	int w, h, i;
	int deltax, deltay;
	int posted;

	w = SDL_VideoSurface->w;
	h = SDL_VideoSurface->h;
	deltax = xevent->xmotion.x - mouse_last.x;
	deltay = xevent->xmotion.y - mouse_last.y;
#ifdef DEBUG_MOTION
  printf("Warped mouse motion: %d,%d\n", deltax, deltay);
#endif
	mouse_last.x = xevent->xmotion.x;
	mouse_last.y = xevent->xmotion.y;
	posted = SDL_PrivateMouseMotion(0, 1, deltax, deltay);

	if ( (xevent->xmotion.x < MOUSE_FUDGE_FACTOR) ||
	     (xevent->xmotion.x > (w-MOUSE_FUDGE_FACTOR)) ||
	     (xevent->xmotion.y < MOUSE_FUDGE_FACTOR) ||
	     (xevent->xmotion.y > (h-MOUSE_FUDGE_FACTOR)) ) {
		/* Get the events that have accumulated */
		while ( XCheckTypedEvent(SDL_Display, MotionNotify, xevent) ) {
			deltax = xevent->xmotion.x - mouse_last.x;
			deltay = xevent->xmotion.y - mouse_last.y;
#ifdef DEBUG_MOTION
  printf("Extra mouse motion: %d,%d\n", deltax, deltay);
#endif
			mouse_last.x = xevent->xmotion.x;
			mouse_last.y = xevent->xmotion.y;
			posted += SDL_PrivateMouseMotion(0, 1, deltax, deltay);
		}
		mouse_last.x = w/2;
		mouse_last.y = h/2;
		XWarpPointer(SDL_Display, None, SDL_Window, 0, 0, 0, 0,
					mouse_last.x, mouse_last.y);
		for ( i=0; i<10; ++i ) {
        		XMaskEvent(SDL_Display, PointerMotionMask, xevent);
			if ( (xevent->xmotion.x >
			          (mouse_last.x-MOUSE_FUDGE_FACTOR)) &&
			     (xevent->xmotion.x <
			          (mouse_last.x+MOUSE_FUDGE_FACTOR)) &&
			     (xevent->xmotion.y >
			          (mouse_last.y-MOUSE_FUDGE_FACTOR)) &&
			     (xevent->xmotion.y <
			          (mouse_last.y+MOUSE_FUDGE_FACTOR)) ) {
				break;
			}
#ifdef DEBUG_XEVENTS
  printf("Lost mouse motion: %d,%d\n", xevent->xmotion.x, xevent->xmotion.y);
#endif
		}
#ifdef DEBUG_XEVENTS
		if ( i == 10 ) {
			printf("Warning: didn't detect mouse warp motion\n");
		}
#endif
	}
	return(posted);
}

static int X11_DispatchEvent(_THIS)
{
	int posted;
	XEvent xevent;

	SDL_memset(&xevent, '\0', sizeof (XEvent));  /* valgrind fix. --ryan. */
	XNextEvent(SDL_Display, &xevent);

	/* Discard KeyRelease and KeyPress events generated by auto-repeat.
	   We need to do it before passing event to XFilterEvent.  Otherwise,
	   KeyRelease aware IMs are confused...  */
	if ( xevent.type == KeyRelease
	     && X11_KeyRepeat(SDL_Display, &xevent) ) {
		return 0;
	}

#ifdef X_HAVE_UTF8_STRING
	/* If we are translating with IM, we need to pass all events
	   to XFilterEvent, and discard those filtered events immediately.  */
	if ( SDL_TranslateUNICODE
	     && SDL_IM != NULL
	     && XFilterEvent(&xevent, None) ) {
		return 0;
	}
#endif

	posted = 0;
	switch (xevent.type) {

	    /* Gaining mouse coverage? */
	    case EnterNotify: {
#ifdef DEBUG_XEVENTS
printf("EnterNotify! (%d,%d)\n", xevent.xcrossing.x, xevent.xcrossing.y);
if ( xevent.xcrossing.mode == NotifyGrab )
printf("Mode: NotifyGrab\n");
if ( xevent.xcrossing.mode == NotifyUngrab )
printf("Mode: NotifyUngrab\n");
#endif
		if ( this->input_grab == SDL_GRAB_OFF ) {
			posted = SDL_PrivateAppActive(1, SDL_APPMOUSEFOCUS);
		}
		posted = SDL_PrivateMouseMotion(0, 0,
				xevent.xcrossing.x,
				xevent.xcrossing.y);
	    }
	    break;

	    /* Losing mouse coverage? */
	    case LeaveNotify: {
#ifdef DEBUG_XEVENTS
printf("LeaveNotify! (%d,%d)\n", xevent.xcrossing.x, xevent.xcrossing.y);
if ( xevent.xcrossing.mode == NotifyGrab )
printf("Mode: NotifyGrab\n");
if ( xevent.xcrossing.mode == NotifyUngrab )
printf("Mode: NotifyUngrab\n");
#endif
		if ( (xevent.xcrossing.mode != NotifyGrab) &&
		     (xevent.xcrossing.mode != NotifyUngrab) &&
		     (xevent.xcrossing.detail != NotifyInferior) ) {
               		if ( this->input_grab == SDL_GRAB_OFF ) {
				posted = SDL_PrivateAppActive(0, SDL_APPMOUSEFOCUS);
			} else {
				posted = SDL_PrivateMouseMotion(0, 0,
						xevent.xcrossing.x,
						xevent.xcrossing.y);
			}
		}
	    }
	    break;

	    /* Gaining input focus? */
	    case FocusIn: {
#ifdef DEBUG_XEVENTS
printf("FocusIn!\n");
#endif
		posted = SDL_PrivateAppActive(1, SDL_APPINPUTFOCUS);

#ifdef X_HAVE_UTF8_STRING
		if ( SDL_IC != NULL ) {
			XSetICFocus(SDL_IC);
		}
#endif
		/* Queue entry into fullscreen mode */
		switch_waiting = 0x01 | SDL_FULLSCREEN;
		switch_time = SDL_GetTicks() + 1500;
	    }
	    break;

	    /* Losing input focus? */
	    case FocusOut: {
#ifdef DEBUG_XEVENTS
printf("FocusOut!\n");
#endif
		posted = SDL_PrivateAppActive(0, SDL_APPINPUTFOCUS);

#ifdef X_HAVE_UTF8_STRING
		if ( SDL_IC != NULL ) {
			XUnsetICFocus(SDL_IC);
		}
#endif
		/* Queue leaving fullscreen mode */
		switch_waiting = 0x01;
		switch_time = SDL_GetTicks() + 200;
	    }
	    break;

#ifdef X_HAVE_UTF8_STRING
	    /* Some IM requires MappingNotify to be passed to
	       XRefreshKeyboardMapping by the app.  */
	    case MappingNotify: {
		XRefreshKeyboardMapping(&xevent.xmapping);
	    }
	    break;
#endif /* X_HAVE_UTF8_STRING */

	    /* Generated upon EnterWindow and FocusIn */
	    case KeymapNotify: {
#ifdef DEBUG_XEVENTS
printf("KeymapNotify!\n");
#endif
		X11_SetKeyboardState(SDL_Display,  xevent.xkeymap.key_vector);
	    }
	    break;

	    /* Mouse motion? */
	    case MotionNotify: {
		if ( SDL_VideoSurface ) {
			if ( mouse_relative ) {
				if ( using_dga & DGA_MOUSE ) {
#ifdef DEBUG_MOTION
  printf("DGA motion: %d,%d\n", xevent.xmotion.x_root, xevent.xmotion.y_root);
#endif
					posted = SDL_PrivateMouseMotion(0, 1,
							xevent.xmotion.x_root,
							xevent.xmotion.y_root);
				} else {
					posted = X11_WarpedMotion(this,&xevent);
				}
			} else {
#ifdef DEBUG_MOTION
  printf("X11 motion: %d,%d\n", xevent.xmotion.x, xevent.xmotion.y);
#endif
				posted = SDL_PrivateMouseMotion(0, 0,
						xevent.xmotion.x,
						xevent.xmotion.y);
			}
		}
	    }
	    break;

	    /* Mouse button press? */
	    case ButtonPress: {
		posted = SDL_PrivateMouseButton(SDL_PRESSED, 
					xevent.xbutton.button, 0, 0);
	    }
	    break;

	    /* Mouse button release? */
	    case ButtonRelease: {
		posted = SDL_PrivateMouseButton(SDL_RELEASED, 
					xevent.xbutton.button, 0, 0);
	    }
	    break;

	    /* Key press? */
	    case KeyPress: {
		SDL_keysym keysym;
		KeyCode keycode = xevent.xkey.keycode;

#ifdef DEBUG_XEVENTS
printf("KeyPress (X11 keycode = 0x%X)\n", xevent.xkey.keycode);
#endif
		/* If we're not doing translation, we're done! */
		if ( !SDL_TranslateUNICODE ) {
			/* Get the translated SDL virtual keysym and put it on the queue.*/
			keysym.scancode = keycode;
			keysym.sym = X11_TranslateKeycode(SDL_Display, keycode);
			keysym.mod = KMOD_NONE;
			keysym.unicode = 0;
			posted = SDL_PrivateKeyboard(SDL_PRESSED, &keysym);
			break;
		}

		/* Look up the translated value for the key event */
#ifdef X_HAVE_UTF8_STRING
		if ( SDL_IC != NULL ) {
			Status status;
			KeySym xkeysym;
			int i;
			/* A UTF-8 character can be at most 6 bytes */
			/* ... It's true, but Xutf8LookupString can
			   return more than one characters.  Moreover,
			   the spec. put no upper bound, so we should
			   be ready for longer strings.  */
			char keybuf[32];
			char *keydata = keybuf;
			int count;
			Uint16 utf16buf[32];
			Uint16 *utf16data = utf16buf;
			int utf16size;
			int utf16length;

			count = Xutf8LookupString(SDL_IC, &xevent.xkey, keydata, sizeof(keybuf), &xkeysym, &status);
			if (XBufferOverflow == status) {
			  /* The IM has just generated somewhat long
			     string.  We need a longer buffer in this
			     case.  */
			  keydata = SDL_malloc(count);
			  if ( keydata == NULL ) {
			    SDL_OutOfMemory();
			    break;
			  }
			  count = Xutf8LookupString(SDL_IC, &xevent.xkey, keydata, count, &xkeysym, &status);
			}

			switch (status) {

			case XBufferOverflow: {
			  /* Oops!  We have allocated the bytes as
			     requested by Xutf8LookupString, so the
			     length of the buffer must be
			     sufficient.  This case should never
			     happen! */
			  SDL_SetError("Xutf8LookupString indicated a double buffer overflow!");
			  break;
			}

			case XLookupChars:
			case XLookupBoth: {
			  if (0 == count) {
			    break;
			  }

			  /* We got a converted string from IM.  Make
			     sure to deliver all characters to the
			     application as SDL events.  Note that
			     an SDL event can only carry one UTF-16
			     encoding unit, and a surrogate pair is
			     delivered as two SDL events.  I guess
			     this behaviour is probably _imported_
			     from Windows or MacOS.  To do so, we need
			     to convert the UTF-8 data into UTF-16
			     data (not UCS4/UTF-32!).  We need an
			     estimate of the number of UTF-16 encoding
			     units here.  The worst case is pure ASCII
			     string.  Assume so. */
			  /* In 1.3 SDL may have a text event instead, that
			     carries the whole UTF-8 string with it. */
			  utf16size = count * sizeof(Uint16);
			  if (utf16size > sizeof(utf16buf)) {
			    utf16data = (Uint16 *) SDL_malloc(utf16size);
			    if (utf16data == NULL) {
			      SDL_OutOfMemory();
			      break;
			    }
			  }
			  utf16length = Utf8ToUtf16((Uint8 *)keydata, count, utf16data, utf16size);
			  if (utf16length < 0) {
			    /* The keydata contained an invalid byte
			       sequence.  It should be a bug of the IM
			       or Xlib... */
			    SDL_SetError("Oops! Xutf8LookupString returned an invalid UTF-8 sequence!");
			    break;
			  }

			  /* Deliver all UTF-16 encoding units.  At
			     this moment, SDL event queue has a
			     fixed size (128 events), and an SDL
			     event can hold just one UTF-16 encoding
			     unit.  So, if we receive more than 128
			     UTF-16 encoding units from a commit,
			     exceeded characters will be lost.  */
			  for (i = 0; i < utf16length - 1; i++) {
			    keysym.scancode = 0;
			    keysym.sym = SDLK_UNKNOWN;
			    keysym.mod = KMOD_NONE;
			    keysym.unicode = utf16data[i];
			    posted = SDL_PrivateKeyboard(SDL_PRESSED, &keysym);
			  }
			  /* The keysym for the last character carries the
			     scancode and symbol that corresponds to the X11
			     keycode.  */
			  if (utf16length > 0) {			       
			    keysym.scancode = keycode;
			    keysym.sym = (keycode ? X11_TranslateKeycode(SDL_Display, keycode) : 0);
			    keysym.mod = KMOD_NONE;
			    keysym.unicode = utf16data[utf16length - 1];
			    posted = SDL_PrivateKeyboard(SDL_PRESSED, &keysym);
			  }
			  break;
			}

			case XLookupKeySym: {
			  /* I'm not sure whether it is possible that
			     a zero keycode makes XLookupKeySym
			     status.  What I'm sure is that a
			     combination of a zero scan code and a non
			     zero sym makes SDL_PrivateKeyboard
			     strange state...  So, just discard it.
			     If this doesn't work, I'm receiving bug
			     reports, and I can know under what
			     condition this case happens.  */
			  if (keycode) {
			    keysym.scancode = keycode;
			    keysym.sym = X11_TranslateKeycode(SDL_Display, keycode);
			    keysym.mod = KMOD_NONE;
			    keysym.unicode = 0;
			    posted = SDL_PrivateKeyboard(SDL_PRESSED, &keysym);
			  }
			  break;
			}

			case XLookupNone: {
			  /* IM has eaten the event.  */
			  break;
			}

			default:
			  /* An unknown status from Xutf8LookupString.  */
			  SDL_SetError("Oops! Xutf8LookupStringreturned an unknown status");
			}

			/* Release dynamic buffers if allocated.  */
			if (keydata != NULL && keybuf != keydata) {
			  SDL_free(keydata);
			}
			if (utf16data != NULL && utf16buf != utf16data) {
			  SDL_free(utf16data);
			}
		}
		else
#endif
		{
			static XComposeStatus state;
			char keybuf[32];

			keysym.scancode = keycode;
			keysym.sym = X11_TranslateKeycode(SDL_Display, keycode);
			keysym.mod = KMOD_NONE;
			keysym.unicode = 0;
			if ( XLookupString(&xevent.xkey,
			                    keybuf, sizeof(keybuf),
			                    NULL, &state) ) {
				/*
				* FIXME: XLookupString() may yield more than one
				* character, so we need a mechanism to allow for
				* this (perhaps null keypress events with a
				* unicode value)
				*/
				keysym.unicode = (Uint8)keybuf[0];
			}

			posted = SDL_PrivateKeyboard(SDL_PRESSED, &keysym);
		}
	    }
	    break;

	    /* Key release? */
	    case KeyRelease: {
		SDL_keysym keysym;
		KeyCode keycode = xevent.xkey.keycode;

		if (keycode == 0) {
		  /* There should be no KeyRelease for keycode == 0,
		     since it is a notification from IM but a real
		     keystroke.  */
		  /* We need to emit some diagnostic message here.  */
		  break;
		}

#ifdef DEBUG_XEVENTS
printf("KeyRelease (X11 keycode = 0x%X)\n", xevent.xkey.keycode);
#endif

		/* Get the translated SDL virtual keysym */
		keysym.scancode = keycode;
		keysym.sym = X11_TranslateKeycode(SDL_Display, keycode);
		keysym.mod = KMOD_NONE;
		keysym.unicode = 0;

		posted = SDL_PrivateKeyboard(SDL_RELEASED, &keysym);
	    }
	    break;

	    /* Have we been iconified? */
	    case UnmapNotify: {
#ifdef DEBUG_XEVENTS
printf("UnmapNotify!\n");
#endif
		/* If we're active, make ourselves inactive */
		if ( SDL_GetAppState() & SDL_APPACTIVE ) {
			/* Swap out the gamma before we go inactive */
			X11_SwapVidModeGamma(this);

			/* Send an internal deactivate event */
			posted = SDL_PrivateAppActive(0,
					SDL_APPACTIVE|SDL_APPINPUTFOCUS);
		}
	    }
	    break;

	    /* Have we been restored? */
	    case MapNotify: {
#ifdef DEBUG_XEVENTS
printf("MapNotify!\n");
#endif
		/* If we're not active, make ourselves active */
		if ( !(SDL_GetAppState() & SDL_APPACTIVE) ) {
			/* Send an internal activate event */
			posted = SDL_PrivateAppActive(1, SDL_APPACTIVE);

			/* Now that we're active, swap the gamma back */
			X11_SwapVidModeGamma(this);
		}

		if ( SDL_VideoSurface &&
		     (SDL_VideoSurface->flags & SDL_FULLSCREEN) ) {
			X11_EnterFullScreen(this);
		} else {
			X11_GrabInputNoLock(this, this->input_grab);
		}
		X11_CheckMouseModeNoLock(this);

		if ( SDL_VideoSurface ) {
			X11_RefreshDisplay(this);
		}
	    }
	    break;

	    /* Have we been resized or moved? */
	    case ConfigureNotify: {
#ifdef DEBUG_XEVENTS
printf("ConfigureNotify! (resize: %dx%d)\n", xevent.xconfigure.width, xevent.xconfigure.height);
#endif
		if ((X11_PendingConfigureNotifyWidth != -1) &&
		    (X11_PendingConfigureNotifyHeight != -1)) {
		    if ((xevent.xconfigure.width != X11_PendingConfigureNotifyWidth) &&
			(xevent.xconfigure.height != X11_PendingConfigureNotifyHeight)) {
			    /* Event is from before the resize, so ignore. */
			    break;
		    }
		    X11_PendingConfigureNotifyWidth = -1;
		    X11_PendingConfigureNotifyHeight = -1;
		}
		if ( SDL_VideoSurface ) {
		    if ((xevent.xconfigure.width != SDL_VideoSurface->w) ||
		        (xevent.xconfigure.height != SDL_VideoSurface->h)) {
			/* FIXME: Find a better fix for the bug with KDE 1.2 */
			if ( ! ((xevent.xconfigure.width == 32) &&
			        (xevent.xconfigure.height == 32)) ) {
				SDL_PrivateResize(xevent.xconfigure.width,
				                  xevent.xconfigure.height);
			}
		    } else {
			/* OpenGL windows need to know about the change */
			if ( SDL_VideoSurface->flags & SDL_OPENGL ) {
				SDL_PrivateExpose();
			}
		    }
		}
	    }
	    break;

	    /* Have we been requested to quit (or another client message?) */
	    case ClientMessage: {
		if ( (xevent.xclient.format == 32) &&
		     (xevent.xclient.data.l[0] == WM_DELETE_WINDOW) )
		{
			posted = SDL_PrivateQuit();
		} else
		if ( SDL_ProcessEvents[SDL_SYSWMEVENT] == SDL_ENABLE ) {
			SDL_SysWMmsg wmmsg;

			SDL_VERSION(&wmmsg.version);
			wmmsg.subsystem = SDL_SYSWM_X11;
			wmmsg.event.xevent = xevent;
			posted = SDL_PrivateSysWMEvent(&wmmsg);
		}
	    }
	    break;

	    /* Do we need to refresh ourselves? */
	    case Expose: {
#ifdef DEBUG_XEVENTS
printf("Expose (count = %d)\n", xevent.xexpose.count);
#endif
		if ( SDL_VideoSurface && (xevent.xexpose.count == 0) ) {
			X11_RefreshDisplay(this);
		}
	    }
	    break;

	    default: {
#ifdef DEBUG_XEVENTS
printf("Unhandled event %d\n", xevent.type);
#endif
		/* Only post the event if we're watching for it */
		if ( SDL_ProcessEvents[SDL_SYSWMEVENT] == SDL_ENABLE ) {
			SDL_SysWMmsg wmmsg;

			SDL_VERSION(&wmmsg.version);
			wmmsg.subsystem = SDL_SYSWM_X11;
			wmmsg.event.xevent = xevent;
			posted = SDL_PrivateSysWMEvent(&wmmsg);
		}
	    }
	    break;
	}
	return(posted);
}

/* Ack!  XPending() actually performs a blocking read if no events available */
int X11_Pending(Display *display)
{
	/* Flush the display connection and look to see if events are queued */
	XFlush(display);
	if ( XEventsQueued(display, QueuedAlready) ) {
		return(1);
	}

	/* More drastic measures are required -- see if X is ready to talk */
	{
		static struct timeval zero_time;	/* static == 0 */
		int x11_fd;
		fd_set fdset;

		x11_fd = ConnectionNumber(display);
		FD_ZERO(&fdset);
		FD_SET(x11_fd, &fdset);
		if ( select(x11_fd+1, &fdset, NULL, NULL, &zero_time) == 1 ) {
			return(XPending(display));
		}
	}

	/* Oh well, nothing is ready .. */
	return(0);
}

void X11_PumpEvents(_THIS)
{
	int pending;

	/* Update activity every five seconds to prevent screensaver. --ryan. */
	if (!allow_screensaver) {
		static Uint32 screensaverTicks;
		Uint32 nowTicks = SDL_GetTicks();
		if ((nowTicks - screensaverTicks) > 5000) {
			XResetScreenSaver(SDL_Display);
			screensaverTicks = nowTicks;
		}
	}

	/* Keep processing pending events */
	pending = 0;
	while ( X11_Pending(SDL_Display) ) {
		X11_DispatchEvent(this);
		++pending;
	}
	if ( switch_waiting ) {
		Uint32 now;

		now  = SDL_GetTicks();
		if ( pending || !SDL_VideoSurface ) {
			/* Try again later... */
			if ( switch_waiting & SDL_FULLSCREEN ) {
				switch_time = now + 1500;
			} else {
				switch_time = now + 200;
			}
		} else if ( (int)(switch_time-now) <= 0 ) {
			Uint32 go_fullscreen;

			go_fullscreen = switch_waiting & SDL_FULLSCREEN;
			switch_waiting = 0;
			if ( SDL_VideoSurface->flags & SDL_FULLSCREEN ) {
				if ( go_fullscreen ) {
					X11_EnterFullScreen(this);
				} else {
					X11_LeaveFullScreen(this);
				}
			}
			/* Handle focus in/out when grabbed */
			if ( go_fullscreen ) {
				X11_GrabInputNoLock(this, this->input_grab);
			} else {
				X11_GrabInputNoLock(this, SDL_GRAB_OFF);
			}
			X11_CheckMouseModeNoLock(this);
		}
	}
}

void X11_InitKeymap(void)
{
	int i;

	/* Odd keys used in international keyboards */
	for ( i=0; i<SDL_arraysize(ODD_keymap); ++i )
		ODD_keymap[i] = SDLK_UNKNOWN;

 	/* Some of these might be mappable to an existing SDLK_ code */
 	ODD_keymap[XK_dead_grave&0xFF] = SDLK_COMPOSE;
 	ODD_keymap[XK_dead_acute&0xFF] = SDLK_COMPOSE;
 	ODD_keymap[XK_dead_tilde&0xFF] = SDLK_COMPOSE;
 	ODD_keymap[XK_dead_macron&0xFF] = SDLK_COMPOSE;
 	ODD_keymap[XK_dead_breve&0xFF] = SDLK_COMPOSE;
 	ODD_keymap[XK_dead_abovedot&0xFF] = SDLK_COMPOSE;
 	ODD_keymap[XK_dead_diaeresis&0xFF] = SDLK_COMPOSE;
 	ODD_keymap[XK_dead_abovering&0xFF] = SDLK_COMPOSE;
 	ODD_keymap[XK_dead_doubleacute&0xFF] = SDLK_COMPOSE;
 	ODD_keymap[XK_dead_caron&0xFF] = SDLK_COMPOSE;
 	ODD_keymap[XK_dead_cedilla&0xFF] = SDLK_COMPOSE;
 	ODD_keymap[XK_dead_ogonek&0xFF] = SDLK_COMPOSE;
 	ODD_keymap[XK_dead_iota&0xFF] = SDLK_COMPOSE;
 	ODD_keymap[XK_dead_voiced_sound&0xFF] = SDLK_COMPOSE;
 	ODD_keymap[XK_dead_semivoiced_sound&0xFF] = SDLK_COMPOSE;
 	ODD_keymap[XK_dead_belowdot&0xFF] = SDLK_COMPOSE;
#ifdef XK_dead_hook
 	ODD_keymap[XK_dead_hook&0xFF] = SDLK_COMPOSE;
#endif
#ifdef XK_dead_horn
 	ODD_keymap[XK_dead_horn&0xFF] = SDLK_COMPOSE;
#endif

#ifdef XK_dead_circumflex
	/* These X keysyms have 0xFE as the high byte */
	ODD_keymap[XK_dead_circumflex&0xFF] = SDLK_CARET;
#endif
#ifdef XK_ISO_Level3_Shift
	ODD_keymap[XK_ISO_Level3_Shift&0xFF] = SDLK_MODE; /* "Alt Gr" key */
#endif

	/* Map the miscellaneous keys */
	for ( i=0; i<SDL_arraysize(MISC_keymap); ++i )
		MISC_keymap[i] = SDLK_UNKNOWN;

	/* These X keysyms have 0xFF as the high byte */
	MISC_keymap[XK_BackSpace&0xFF] = SDLK_BACKSPACE;
	MISC_keymap[XK_Tab&0xFF] = SDLK_TAB;
	MISC_keymap[XK_Clear&0xFF] = SDLK_CLEAR;
	MISC_keymap[XK_Return&0xFF] = SDLK_RETURN;
	MISC_keymap[XK_Pause&0xFF] = SDLK_PAUSE;
	MISC_keymap[XK_Escape&0xFF] = SDLK_ESCAPE;
	MISC_keymap[XK_Delete&0xFF] = SDLK_DELETE;

	MISC_keymap[XK_KP_0&0xFF] = SDLK_KP0;		/* Keypad 0-9 */
	MISC_keymap[XK_KP_1&0xFF] = SDLK_KP1;
	MISC_keymap[XK_KP_2&0xFF] = SDLK_KP2;
	MISC_keymap[XK_KP_3&0xFF] = SDLK_KP3;
	MISC_keymap[XK_KP_4&0xFF] = SDLK_KP4;
	MISC_keymap[XK_KP_5&0xFF] = SDLK_KP5;
	MISC_keymap[XK_KP_6&0xFF] = SDLK_KP6;
	MISC_keymap[XK_KP_7&0xFF] = SDLK_KP7;
	MISC_keymap[XK_KP_8&0xFF] = SDLK_KP8;
	MISC_keymap[XK_KP_9&0xFF] = SDLK_KP9;
	MISC_keymap[XK_KP_Insert&0xFF] = SDLK_KP0;
	MISC_keymap[XK_KP_End&0xFF] = SDLK_KP1;	
	MISC_keymap[XK_KP_Down&0xFF] = SDLK_KP2;
	MISC_keymap[XK_KP_Page_Down&0xFF] = SDLK_KP3;
	MISC_keymap[XK_KP_Left&0xFF] = SDLK_KP4;
	MISC_keymap[XK_KP_Begin&0xFF] = SDLK_KP5;
	MISC_keymap[XK_KP_Right&0xFF] = SDLK_KP6;
	MISC_keymap[XK_KP_Home&0xFF] = SDLK_KP7;
	MISC_keymap[XK_KP_Up&0xFF] = SDLK_KP8;
	MISC_keymap[XK_KP_Page_Up&0xFF] = SDLK_KP9;
	MISC_keymap[XK_KP_Delete&0xFF] = SDLK_KP_PERIOD;
	MISC_keymap[XK_KP_Decimal&0xFF] = SDLK_KP_PERIOD;
	MISC_keymap[XK_KP_Divide&0xFF] = SDLK_KP_DIVIDE;
	MISC_keymap[XK_KP_Multiply&0xFF] = SDLK_KP_MULTIPLY;
	MISC_keymap[XK_KP_Subtract&0xFF] = SDLK_KP_MINUS;
	MISC_keymap[XK_KP_Add&0xFF] = SDLK_KP_PLUS;
	MISC_keymap[XK_KP_Enter&0xFF] = SDLK_KP_ENTER;
	MISC_keymap[XK_KP_Equal&0xFF] = SDLK_KP_EQUALS;

	MISC_keymap[XK_Up&0xFF] = SDLK_UP;
	MISC_keymap[XK_Down&0xFF] = SDLK_DOWN;
	MISC_keymap[XK_Right&0xFF] = SDLK_RIGHT;
	MISC_keymap[XK_Left&0xFF] = SDLK_LEFT;
	MISC_keymap[XK_Insert&0xFF] = SDLK_INSERT;
	MISC_keymap[XK_Home&0xFF] = SDLK_HOME;
	MISC_keymap[XK_End&0xFF] = SDLK_END;
	MISC_keymap[XK_Page_Up&0xFF] = SDLK_PAGEUP;
	MISC_keymap[XK_Page_Down&0xFF] = SDLK_PAGEDOWN;

	MISC_keymap[XK_F1&0xFF] = SDLK_F1;
	MISC_keymap[XK_F2&0xFF] = SDLK_F2;
	MISC_keymap[XK_F3&0xFF] = SDLK_F3;
	MISC_keymap[XK_F4&0xFF] = SDLK_F4;
	MISC_keymap[XK_F5&0xFF] = SDLK_F5;
	MISC_keymap[XK_F6&0xFF] = SDLK_F6;
	MISC_keymap[XK_F7&0xFF] = SDLK_F7;
	MISC_keymap[XK_F8&0xFF] = SDLK_F8;
	MISC_keymap[XK_F9&0xFF] = SDLK_F9;
	MISC_keymap[XK_F10&0xFF] = SDLK_F10;
	MISC_keymap[XK_F11&0xFF] = SDLK_F11;
	MISC_keymap[XK_F12&0xFF] = SDLK_F12;
	MISC_keymap[XK_F13&0xFF] = SDLK_F13;
	MISC_keymap[XK_F14&0xFF] = SDLK_F14;
	MISC_keymap[XK_F15&0xFF] = SDLK_F15;

	MISC_keymap[XK_Num_Lock&0xFF] = SDLK_NUMLOCK;
	MISC_keymap[XK_Caps_Lock&0xFF] = SDLK_CAPSLOCK;
	MISC_keymap[XK_Scroll_Lock&0xFF] = SDLK_SCROLLOCK;
	MISC_keymap[XK_Shift_R&0xFF] = SDLK_RSHIFT;
	MISC_keymap[XK_Shift_L&0xFF] = SDLK_LSHIFT;
	MISC_keymap[XK_Control_R&0xFF] = SDLK_RCTRL;
	MISC_keymap[XK_Control_L&0xFF] = SDLK_LCTRL;
	MISC_keymap[XK_Alt_R&0xFF] = SDLK_RALT;
	MISC_keymap[XK_Alt_L&0xFF] = SDLK_LALT;
	MISC_keymap[XK_Meta_R&0xFF] = SDLK_RMETA;
	MISC_keymap[XK_Meta_L&0xFF] = SDLK_LMETA;
	MISC_keymap[XK_Super_L&0xFF] = SDLK_LSUPER; /* Left "Windows" */
	MISC_keymap[XK_Super_R&0xFF] = SDLK_RSUPER; /* Right "Windows */
	MISC_keymap[XK_Mode_switch&0xFF] = SDLK_MODE; /* "Alt Gr" key */
	MISC_keymap[XK_Multi_key&0xFF] = SDLK_COMPOSE; /* Multi-key compose */

	MISC_keymap[XK_Help&0xFF] = SDLK_HELP;
	MISC_keymap[XK_Print&0xFF] = SDLK_PRINT;
	MISC_keymap[XK_Sys_Req&0xFF] = SDLK_SYSREQ;
	MISC_keymap[XK_Break&0xFF] = SDLK_BREAK;
	MISC_keymap[XK_Menu&0xFF] = SDLK_MENU;
	MISC_keymap[XK_Hyper_R&0xFF] = SDLK_MENU;   /* Windows "Menu" key */
}

/* Get the translated SDL virtual keysym */
SDLKey X11_TranslateKeycode(Display *display, KeyCode kc)
{
	KeySym xsym;
	SDLKey key;

	xsym = XKeycodeToKeysym(display, kc, 0);
#ifdef DEBUG_KEYS
	fprintf(stderr, "Translating key code %d -> 0x%.4x\n", kc, xsym);
#endif
	key = SDLK_UNKNOWN;
	if ( xsym ) {
		switch (xsym>>8) {
		    case 0x1005FF:
#ifdef SunXK_F36
			if ( xsym == SunXK_F36 )
				key = SDLK_F11;
#endif
#ifdef SunXK_F37
			if ( xsym == SunXK_F37 )
				key = SDLK_F12;
#endif
			break;
		    case 0x00:	/* Latin 1 */
			key = (SDLKey)(xsym & 0xFF);
			break;
		    case 0x01:	/* Latin 2 */
		    case 0x02:	/* Latin 3 */
		    case 0x03:	/* Latin 4 */
		    case 0x04:	/* Katakana */
		    case 0x05:	/* Arabic */
		    case 0x06:	/* Cyrillic */
		    case 0x07:	/* Greek */
		    case 0x08:	/* Technical */
		    case 0x0A:	/* Publishing */
		    case 0x0C:	/* Hebrew */
		    case 0x0D:	/* Thai */
			/* These are wrong, but it's better than nothing */
			key = (SDLKey)(xsym & 0xFF);
			break;
		    case 0xFE:
			key = ODD_keymap[xsym&0xFF];
			break;
		    case 0xFF:
			key = MISC_keymap[xsym&0xFF];
			break;
		    default:
			/*
			fprintf(stderr, "X11: Unhandled xsym, sym = 0x%04x\n",
					(unsigned int)xsym);
			*/
			break;
		}
	} else {
		/* X11 doesn't know how to translate the key! */
		switch (kc) {
		    /* Caution:
		       These keycodes are from the Microsoft Keyboard
		     */
		    case 115:
			key = SDLK_LSUPER;
			break;
		    case 116:
			key = SDLK_RSUPER;
			break;
		    case 117:
			key = SDLK_MENU;
			break;
		    default:
			/*
			 * no point in an error message; happens for
			 * several keys when we get a keymap notify
			 */
			break;
		}
	}
	return key;
}

/* X11 modifier masks for various keys */
static unsigned meta_l_mask, meta_r_mask, alt_l_mask, alt_r_mask;
static unsigned num_mask, mode_switch_mask;

static void get_modifier_masks(Display *display)
{
	static unsigned got_masks;
	int i, j;
	XModifierKeymap *xmods;
	unsigned n;

	if(got_masks)
		return;

	xmods = XGetModifierMapping(display);
	n = xmods->max_keypermod;
	for(i = 3; i < 8; i++) {
		for(j = 0; j < n; j++) {
			KeyCode kc = xmods->modifiermap[i * n + j];
			KeySym ks = XKeycodeToKeysym(display, kc, 0);
			unsigned mask = 1 << i;
			switch(ks) {
			case XK_Num_Lock:
				num_mask = mask; break;
			case XK_Alt_L:
				alt_l_mask = mask; break;
			case XK_Alt_R:
				alt_r_mask = mask; break;
			case XK_Meta_L:
				meta_l_mask = mask; break;
			case XK_Meta_R:
				meta_r_mask = mask; break;
			case XK_Mode_switch:
				mode_switch_mask = mask; break;
			}
		}
	}
	XFreeModifiermap(xmods);
	got_masks = 1;
}


/*
 * This function is semi-official; it is not officially exported and should
 * not be considered part of the SDL API, but may be used by client code
 * that *really* needs it (including legacy code).
 * It is slow, though, and should be avoided if possible.
 *
 * Note that it isn't completely accurate either; in particular, multi-key
 * sequences (dead accents, compose key sequences) will not work since the
 * state has been irrevocably lost.
 */
Uint16 X11_KeyToUnicode(SDLKey keysym, SDLMod modifiers)
{
	struct SDL_VideoDevice *this = current_video;
	char keybuf[32];
	int i;
	KeySym xsym = 0;
	XKeyEvent xkey;
	Uint16 unicode;

	if ( !this || !SDL_Display ) {
		return 0;
	}

	SDL_memset(&xkey, 0, sizeof(xkey));
	xkey.display = SDL_Display;

	xsym = keysym;		/* last resort if not found */
	for (i = 0; i < 256; ++i) {
		if ( MISC_keymap[i] == keysym ) {
			xsym = 0xFF00 | i;
			break;
		} else if ( ODD_keymap[i] == keysym ) {
			xsym = 0xFE00 | i;
			break;
		}
	}

	xkey.keycode = XKeysymToKeycode(xkey.display, xsym);

	get_modifier_masks(SDL_Display);
	if(modifiers & KMOD_SHIFT)
		xkey.state |= ShiftMask;
	if(modifiers & KMOD_CAPS)
		xkey.state |= LockMask;
	if(modifiers & KMOD_CTRL)
		xkey.state |= ControlMask;
	if(modifiers & KMOD_MODE)
		xkey.state |= mode_switch_mask;
	if(modifiers & KMOD_LALT)
		xkey.state |= alt_l_mask;
	if(modifiers & KMOD_RALT)
		xkey.state |= alt_r_mask;
	if(modifiers & KMOD_LMETA)
		xkey.state |= meta_l_mask;
	if(modifiers & KMOD_RMETA)
		xkey.state |= meta_r_mask;
	if(modifiers & KMOD_NUM)
		xkey.state |= num_mask;

	unicode = 0;
	if ( XLookupString(&xkey, keybuf, sizeof(keybuf), NULL, NULL) )
		unicode = (unsigned char)keybuf[0];
	return(unicode);
}


/*
 * Called when focus is regained, to read the keyboard state and generate
 * synthetic keypress/release events.
 * key_vec is a bit vector of keycodes (256 bits)
 */
void X11_SetKeyboardState(Display *display, const char *key_vec)
{
	char keys_return[32];
	int i;
	Uint8 *kstate = SDL_GetKeyState(NULL);
	SDLMod modstate;
	Window junk_window;
	int x, y;
	unsigned int mask;

	/* The first time the window is mapped, we initialize key state */
	if ( ! key_vec ) {
		XQueryKeymap(display, keys_return);
		key_vec = keys_return;
	}

	/* Get the keyboard modifier state */
	modstate = 0;
	get_modifier_masks(display);
	if ( XQueryPointer(display, DefaultRootWindow(display),
		&junk_window, &junk_window, &x, &y, &x, &y, &mask) ) {
		if ( mask & LockMask ) {
			modstate |= KMOD_CAPS;
		}
		if ( mask & mode_switch_mask ) {
			modstate |= KMOD_MODE;
		}
		if ( mask & num_mask ) {
			modstate |= KMOD_NUM;
		}
	}

	/* Zero the new keyboard state and generate it */
	SDL_memset(kstate, 0, SDLK_LAST);
	/*
	 * An obvious optimisation is to check entire longwords at a time in
	 * both loops, but we can't be sure the arrays are aligned so it's not
	 * worth the extra complexity
	 */
	for ( i = 0; i < 32; i++ ) {
		int j;
		if ( !key_vec[i] )
			continue;
		for ( j = 0; j < 8; j++ ) {
			if ( key_vec[i] & (1 << j) ) {
				SDLKey key;
				KeyCode kc = (i << 3 | j);
				key = X11_TranslateKeycode(display, kc);
				if ( key == SDLK_UNKNOWN ) {
					continue;
				}
				kstate[key] = SDL_PRESSED;
				switch (key) {
				    case SDLK_LSHIFT:
					modstate |= KMOD_LSHIFT;
					break;
				    case SDLK_RSHIFT:
					modstate |= KMOD_RSHIFT;
					break;
				    case SDLK_LCTRL:
					modstate |= KMOD_LCTRL;
					break;
				    case SDLK_RCTRL:
					modstate |= KMOD_RCTRL;
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
				    default:
					break;
				}
			}
		}
	}

	/* Hack - set toggle key state */
	if ( modstate & KMOD_CAPS ) {
		kstate[SDLK_CAPSLOCK] = SDL_PRESSED;
	} else {
		kstate[SDLK_CAPSLOCK] = SDL_RELEASED;
	}
	if ( modstate & KMOD_NUM ) {
		kstate[SDLK_NUMLOCK] = SDL_PRESSED;
	} else {
		kstate[SDLK_NUMLOCK] = SDL_RELEASED;
	}

	/* Set the final modifier state */
	SDL_SetModState(modstate);
}

void X11_InitOSKeymap(_THIS)
{
	X11_InitKeymap();
}

