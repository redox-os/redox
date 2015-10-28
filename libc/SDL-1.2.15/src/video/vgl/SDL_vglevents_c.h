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

#include "SDL_vglvideo.h"

/* Variables and functions exported by SDL_sysevents.c to other parts 
   of the native video subsystem (SDL_sysvideo.c)
*/
extern int VGL_initkeymaps(int fd);
extern int VGL_initmouse(int fd);
extern void VGL_keyboardcallback(int scancode, int pressed);

extern void VGL_InitOSKeymap(_THIS);
extern void VGL_PumpEvents(_THIS);

/* Mouse buttons */
#define MOUSE_LEFTBUTTON        0x01
#define MOUSE_MIDDLEBUTTON      0x02
#define MOUSE_RIGHTBUTTON       0x04

/* Scancodes */
#define SCANCODE_ESCAPE			1
#define SCANCODE_1			2
#define SCANCODE_2			3
#define SCANCODE_3			4
#define SCANCODE_4			5
#define SCANCODE_5			6
#define SCANCODE_6			7
#define SCANCODE_7			8
#define SCANCODE_8			9
#define SCANCODE_9			10
#define SCANCODE_0			11
#define SCANCODE_MINUS			12
#define SCANCODE_EQUAL			13
#define SCANCODE_BACKSPACE		14
#define SCANCODE_TAB			15
#define SCANCODE_Q			16
#define SCANCODE_W			17
#define SCANCODE_E			18
#define SCANCODE_R			19
#define SCANCODE_T			20
#define SCANCODE_Y			21
#define SCANCODE_U			22
#define SCANCODE_I			23
#define SCANCODE_O			24
#define SCANCODE_P			25
#define SCANCODE_BRACKET_LEFT		26
#define SCANCODE_BRACKET_RIGHT		27
#define SCANCODE_ENTER			28
#define SCANCODE_LEFTCONTROL		29
#define SCANCODE_A			30
#define SCANCODE_S			31
#define SCANCODE_D			32
#define SCANCODE_F			33
#define SCANCODE_G			34
#define SCANCODE_H			35
#define SCANCODE_J			36
#define SCANCODE_K			37
#define SCANCODE_L			38
#define SCANCODE_SEMICOLON		39
#define SCANCODE_APOSTROPHE		40
#define SCANCODE_GRAVE			41
#define SCANCODE_LEFTSHIFT		42
#define SCANCODE_BACKSLASH		43
#define SCANCODE_Z			44
#define SCANCODE_X			45
#define SCANCODE_C			46
#define SCANCODE_V			47
#define SCANCODE_B			48
#define SCANCODE_N			49
#define SCANCODE_M			50
#define SCANCODE_COMMA			51
#define SCANCODE_PERIOD			52
#define SCANCODE_SLASH			53
#define SCANCODE_RIGHTSHIFT		54
#define SCANCODE_KEYPADMULTIPLY		55
#define SCANCODE_LEFTALT		56
#define SCANCODE_SPACE			57
#define SCANCODE_CAPSLOCK		58
#define SCANCODE_F1			59
#define SCANCODE_F2			60
#define SCANCODE_F3			61
#define SCANCODE_F4			62
#define SCANCODE_F5			63
#define SCANCODE_F6			64
#define SCANCODE_F7			65
#define SCANCODE_F8			66
#define SCANCODE_F9			67
#define SCANCODE_F10			68
#define SCANCODE_NUMLOCK		69
#define SCANCODE_SCROLLLOCK		70
#define SCANCODE_KEYPAD7		71
#define SCANCODE_CURSORUPLEFT		71
#define SCANCODE_KEYPAD8		72
#define SCANCODE_CURSORUP		72
#define SCANCODE_KEYPAD9		73
#define SCANCODE_CURSORUPRIGHT		73
#define SCANCODE_KEYPADMINUS		74
#define SCANCODE_KEYPAD4		75
#define SCANCODE_CURSORLEFT		75
#define SCANCODE_KEYPAD5		76
#define SCANCODE_KEYPAD6		77
#define SCANCODE_CURSORRIGHT		77
#define SCANCODE_KEYPADPLUS		78
#define SCANCODE_KEYPAD1		79
#define SCANCODE_CURSORDOWNLEFT		79
#define SCANCODE_KEYPAD2		80
#define SCANCODE_CURSORDOWN		80
#define SCANCODE_KEYPAD3		81
#define SCANCODE_CURSORDOWNRIGHT	81
#define SCANCODE_KEYPAD0		82
#define SCANCODE_KEYPADPERIOD		83
#define SCANCODE_LESS			86
#define SCANCODE_F11			87
#define SCANCODE_F12			88
#define SCANCODE_KEYPADENTER		89
#define SCANCODE_RIGHTCONTROL		90
#define SCANCODE_CONTROL		107
#define SCANCODE_KEYPADDIVIDE		91
#define SCANCODE_PRINTSCREEN		92
#define SCANCODE_RIGHTALT		93
#define SCANCODE_BREAK			104	/* Beware: is 119     */
#define SCANCODE_BREAK_ALTERNATIVE	104	/* on some keyboards! */
#define SCANCODE_HOME			94
#define SCANCODE_CURSORBLOCKUP		95	/* Cursor key block */
#define SCANCODE_PAGEUP			96
#define SCANCODE_CURSORBLOCKLEFT	97	/* Cursor key block */
#define SCANCODE_CURSORBLOCKRIGHT	98	/* Cursor key block */
#define SCANCODE_END			99
#define SCANCODE_CURSORBLOCKDOWN	100	/* Cursor key block */
#define SCANCODE_PAGEDOWN		101
#define SCANCODE_INSERT			102
#define SCANCODE_REMOVE			103
#define SCANCODE_RIGHTWIN		106
#define SCANCODE_LEFTWIN		105
