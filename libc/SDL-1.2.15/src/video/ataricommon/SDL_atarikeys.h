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

/*
 *	Atari Scancode definitions
 *
 *	Patrice Mandin
 */

#ifndef _SDL_ATARIKEYS_H_
#define _SDL_ATARIKEYS_H_ 

/* --- Keyboard scancodes --- */
/* taken from svgalib/vgakeyboard.h */

#define SCANCODE_ESCAPE		0x01
#define SCANCODE_1		0x02
#define SCANCODE_2		0x03
#define SCANCODE_3		0x04
#define SCANCODE_4		0x05
#define SCANCODE_5		0x06
#define SCANCODE_6		0x07
#define SCANCODE_7		0x08
#define SCANCODE_8		0x09
#define SCANCODE_9		0x0a
#define SCANCODE_0		0x0b
#define SCANCODE_MINUS		0x0c
#define SCANCODE_EQUAL		0x0d
#define SCANCODE_BACKSPACE	0x0e

#define SCANCODE_TAB		0x0f
#define SCANCODE_Q		0x10
#define SCANCODE_W		0x11
#define SCANCODE_E		0x12
#define SCANCODE_R		0x13
#define SCANCODE_T		0x14
#define SCANCODE_Y		0x15
#define SCANCODE_U		0x16
#define SCANCODE_I		0x17
#define SCANCODE_O		0x18
#define SCANCODE_P		0x19
#define SCANCODE_BRACKET_LEFT	0x1a
#define SCANCODE_BRACKET_RIGHT	0x1b
#define SCANCODE_ENTER		0x1c
#define SCANCODE_DELETE		0x53

#define SCANCODE_LEFTCONTROL	0x1d
#define SCANCODE_A		0x1e
#define SCANCODE_S		0x1f
#define SCANCODE_D		0x20
#define SCANCODE_F		0x21
#define SCANCODE_G		0x22
#define SCANCODE_H		0x23
#define SCANCODE_J		0x24
#define SCANCODE_K		0x25
#define SCANCODE_L		0x26
#define SCANCODE_SEMICOLON	0x27
#define SCANCODE_APOSTROPHE	0x28
#define SCANCODE_GRAVE		0x29

#define SCANCODE_LEFTSHIFT	0x2a
#define SCANCODE_BACKSLASH	0x2b
#define SCANCODE_Z		0x2c
#define SCANCODE_X		0x2d
#define SCANCODE_C		0x2e
#define SCANCODE_V		0x2f
#define SCANCODE_B		0x30
#define SCANCODE_N		0x31
#define SCANCODE_M		0x32
#define SCANCODE_COMMA		0x33
#define SCANCODE_PERIOD		0x34
#define SCANCODE_SLASH		0x35
#define SCANCODE_RIGHTSHIFT	0x36

#define SCANCODE_LEFTALT	0x38
#define SCANCODE_SPACE		0x39
#define SCANCODE_CAPSLOCK	0x3a

/* Functions keys */
#define SCANCODE_F1		0x3b
#define SCANCODE_F2		0x3c
#define SCANCODE_F3		0x3d
#define SCANCODE_F4		0x3e
#define SCANCODE_F5		0x3f
#define SCANCODE_F6		0x40
#define SCANCODE_F7		0x41
#define SCANCODE_F8		0x42
#define SCANCODE_F9		0x43
#define SCANCODE_F10	0x44

/* Numeric keypad */
#define SCANCODE_KP0			0x70
#define SCANCODE_KP1			0x6d
#define SCANCODE_KP2			0x6e
#define SCANCODE_KP3			0x6f
#define SCANCODE_KP4			0x6a
#define SCANCODE_KP5			0x6b
#define SCANCODE_KP6			0x6c
#define SCANCODE_KP7			0x67
#define SCANCODE_KP8			0x68
#define SCANCODE_KP9			0x69
#define SCANCODE_KP_PERIOD		0x71
#define SCANCODE_KP_DIVIDE		0x65
#define SCANCODE_KP_MULTIPLY	0x66
#define SCANCODE_KP_MINUS		0x4a
#define SCANCODE_KP_PLUS		0x4e
#define SCANCODE_KP_ENTER		0x72
#define SCANCODE_KP_LEFTPAREN	0x63
#define SCANCODE_KP_RIGHTPAREN	0x64

/* Cursor keypad */
#define SCANCODE_HELP		0x62
#define SCANCODE_UNDO		0x61
#define SCANCODE_INSERT		0x52
#define SCANCODE_CLRHOME	0x47
#define SCANCODE_UP			0x48
#define SCANCODE_DOWN		0x50
#define SCANCODE_RIGHT		0x4d
#define SCANCODE_LEFT		0x4b

#endif /* _SDL_ATARIKEYS_H_ */
