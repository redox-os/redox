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
 *	IKBD 6301 interrupt routine
 *
 *	Patrice Mandin
 */

	.text

	.globl	_SDL_AtariIkbdInstall
	.globl	_SDL_AtariIkbdUninstall

	.globl	_SDL_AtariIkbd_keyboard
	.globl	_SDL_AtariIkbd_mouseb
	.globl	_SDL_AtariIkbd_mousex
	.globl	_SDL_AtariIkbd_mousey
	.globl	_SDL_AtariIkbd_joystick

	.globl	_SDL_AtariIkbd_enabled

/*--- Install our IKBD vector ---*/

_SDL_AtariIkbdInstall:
#if defined(__mcoldfire__)
	lea	sp@(-16),sp
	moveml	d0-d1/a0-a1,sp@
#else
	moveml	d0-d1/a0-a1,sp@-
#endif

	| Disable interrupts

	movew	#0x2700,sr

	| Save MFP registers used for keyboard

	lea	0xfffffa00:w,a0
	btst	#6,a0@(0x09)
#if defined(__mcoldfire__)
	sne	d0
	move.b	d0,ikbd_ierb
#else
	sne	ikbd_ierb
#endif
	btst	#6,a0@(0x15)
#if defined(__mcoldfire__)
	sne	d0
	move.b	d0,ikbd_imrb
#else
	sne	ikbd_imrb
#endif

	| Set our routine

#if defined(__mcoldfire__)
	movel	0x118:w,d0
	movel	d0,old_ikbd
	lea	ikbd,a0
	movel	a0,0x118:w
	moveql	#6,d0
	bset	d0,0xfffffa09:w	| IERB
	bset	d0,0xfffffa15:w	| IMRB
#else
	movel	0x118:w,old_ikbd
	movel	#ikbd,0x118:w
	bset	#6,0xfffffa09:w	| IERB
	bset	#6,0xfffffa15:w	| IMRB
#endif

	| Set mouse relative mode

#if defined(__mcoldfire__)
	moveql	#8,d0
	moveb	d0,0xfffffc02:w
#else
	moveb	#8,0xfffffc02:w
#endif

	| Reenable interrupts

	movew	#0x2300,sr

	| Interrupts done

#if defined(__mcoldfire__)
	movel	#0xffff,d0
	movew	d0,_SDL_AtariIkbd_enabled

	moveml	sp@,d0-d1/a0-a1
	lea	sp@(16),sp
#else
	movew	#0xffff,_SDL_AtariIkbd_enabled

	moveml	sp@+,d0-d1/a0-a1
#endif
	rts

/*--- Uninstall our IKBD vector ---*/

_SDL_AtariIkbdUninstall:
	movel	a0,sp@-

	| Disable interrupts

	movew	#0x2700,sr

	| Restore previous MFP registers

	lea	0xfffffa00:w,a0

	bclr	#6,a0@(0x09)
	tstb	ikbd_ierb
	beqs	ikbd_restoreierb
	bset	#6,a0@(0x09)
ikbd_restoreierb:

	bclr	#6,a0@(0x15)
	tstb	ikbd_imrb
	beqs	ikbd_restoreimrb
	bset	#6,a0@(0x15)
ikbd_restoreimrb:

#if defined(__mcoldfire__)
	movel	old_ikbd,a0
	movel	a0,0x118:w
#else
	movel	old_ikbd,0x118:w
#endif

	| Clear keyboard buffer

	lea	0xfffffc00:w,a0
ikbd_videbuffer:
	btst	#0,a0@
	beqs	ikbd_finbuffer
	tstb	a0@(0x02)
	bras	ikbd_videbuffer
ikbd_finbuffer:

	| Reenable interrupts

	movew	#0x2300,sr

	movel	sp@+,a0
	rts

	.bss

	.even
	.comm	ikbd_ierb,1
	.comm	ikbd_imrb,1

/*--- Our custom IKBD vector ---*/

	.text
	.even
	.ascii	"XBRA"
	.ascii	"LSDL"
	.comm	old_ikbd,4*1
ikbd:
#if defined(__mcoldfire__)
	lea	sp@(-12),sp
	moveml	d0-d1/a0,sp@
#else
	moveml	d0-d1/a0,sp@-
#endif

	| Check if source is IKBD or MIDI
#if defined(__mcoldfire__)
	moveql	#0,d0
	btst	d0,0xfffffc00.w
#else
	btst	#0,0xfffffc00.w
#endif
	beqs	ikbd_oldmidi

	moveb	0xfffffc02:w,d0

	| Joystick packet ?
	
	cmpb	#0xff,d0
	beqs	ikbd_yes_joystick

	| Mouse packet ?

	cmpb	#0xf8,d0
	bmis	ikbd_no_mouse
	cmpb	#0xfc,d0
	bpls	ikbd_no_mouse

	| Mouse packet, byte #1

ikbd_yes_mouse:
#if defined(__mcoldfire__)
	andl	#3,d0
#else
	andw	#3,d0
#endif
	movew	d0,_SDL_AtariIkbd_mouseb

#if defined(__mcoldfire__)
	movel	#ikbd_mousex,d0
	movel	d0,0x118:w
#else
	movel	#ikbd_mousex,0x118:w
#endif
	bras	ikbd_endit_stack

	| Joystick packet, byte #1

ikbd_yes_joystick:
#if defined(__mcoldfire__)
	movel	#ikbd_joystick,d0
	movel	d0,0x118:w
#else
	movel	#ikbd_joystick,0x118:w
#endif
	bras	ikbd_endit_stack

	| Keyboard press/release

ikbd_no_mouse:
	moveb	d0,d1
	lea		_SDL_AtariIkbd_keyboard,a0
#if defined(__mcoldfire__)
	andl	#0x7f,d1
	btst	#7,d0
	spl	d0
	moveb	d0,a0@(0,d1:l)
#else
	andw	#0x7f,d1
	tas	d0
	spl	a0@(0,d1:w)
#endif

	| End of interrupt

ikbd_endit_stack:
#if defined(__mcoldfire__)
	moveql	#6,d0
	bclr	d0,0xfffffa11:w

	moveml	sp@,d0-d1/a0
	lea	sp@(12),sp
#else
	moveml	sp@+,d0-d1/a0

	bclr	#6,0xfffffa11:w
#endif
	rte

	| Call old MIDI interrupt

ikbd_oldmidi:
#if defined(__mcoldfire__)
	moveml	sp@,d0-d1/a0
	lea	sp@(12),sp
#else
	moveml	sp@+,d0-d1/a0
#endif

	movel	old_ikbd,sp@-
	rts

	| Mouse packet, byte #2

ikbd_mousex:
#if defined(__mcoldfire__)
	lea	sp@(-12),sp
	moveml	d0-d1/a0,sp@
#else
	moveml	d0-d1/a0,sp@-
#endif

	| Check if source is IKBD or MIDI
#if defined(__mcoldfire__)
	moveql	#0,d0
	btst	d0,0xfffffc00.w
#else
	btst	#0,0xfffffc00.w
#endif
	beqs	ikbd_oldmidi

	moveb	0xfffffc02:w,d0
	extw	d0
#if defined(__mcoldfire__)
	movew	_SDL_AtariIkbd_mousex,d1
	addl	d1,d0
	movew	d0,_SDL_AtariIkbd_mousex

	movel	#ikbd_mousey,d0
	movel	d0,0x118:w
#else
	addw	d0,_SDL_AtariIkbd_mousex

	movel	#ikbd_mousey,0x118:w
#endif
	bras	ikbd_endit_stack

	| Mouse packet, byte #3

ikbd_mousey:
#if defined(__mcoldfire__)
	lea	sp@(-12),sp
	moveml	d0-d1/a0,sp@
#else
	moveml	d0-d1/a0,sp@-
#endif

	| Check if source is IKBD or MIDI
#if defined(__mcoldfire__)
	moveql	#0,d0
	btst	d0,0xfffffc00.w
#else
	btst	#0,0xfffffc00.w
#endif
	beqs	ikbd_oldmidi

	moveb	0xfffffc02:w,d0
	extw	d0
#if defined(__mcoldfire__)
	movew	_SDL_AtariIkbd_mousey,d1
	addl	d1,d0
	movew	d0,_SDL_AtariIkbd_mousey

	movel	#ikbd,d0
	movel	d0,0x118:w
#else
	addw	d0,_SDL_AtariIkbd_mousey

	movel	#ikbd,0x118:w
#endif
	bras	ikbd_endit_stack

	| Joystick packet, byte #2

ikbd_joystick:
#if defined(__mcoldfire__)
	lea	sp@(-12),sp
	moveml	d0-d1/a0,sp@
#else
	moveml	d0-d1/a0,sp@-
#endif

	| Check if source is IKBD or MIDI
#if defined(__mcoldfire__)
	moveql	#0,d0
	btst	d0,0xfffffc00.w
#else
	btst	#0,0xfffffc00.w
#endif
	beqs	ikbd_oldmidi

#if defined(__mcoldfire__)
	moveb	0xfffffc02:w,d0
	moveb	d0,_SDL_AtariIkbd_joystick+1

	movel	#ikbd,d0
	movel	d0,0x118:w

	bra	ikbd_endit_stack
#else
	moveb	0xfffffc02:w,_SDL_AtariIkbd_joystick+1

	movel	#ikbd,0x118:w

	bras	ikbd_endit_stack
#endif

	.data
	
	.even
_SDL_AtariIkbd_enabled:
	.word	0

	.bss

	.even
	.comm	_SDL_AtariIkbd_keyboard,128
	.comm	_SDL_AtariIkbd_mousex,2
	.comm	_SDL_AtariIkbd_mousey,2
	.comm	_SDL_AtariIkbd_mouseb,2
	.comm	_SDL_AtariIkbd_joystick,2
