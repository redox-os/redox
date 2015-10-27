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
 *	XBIOS mouse & joystick vectors
 *
 *	Patrice Mandin
 */

	.text

	.globl	_SDL_AtariXbios_Install
	.globl	_SDL_AtariXbios_Restore
	.globl	_SDL_AtariXbios_MouseVector
	.globl	_SDL_AtariXbios_JoystickVector

	.globl	_SDL_AtariXbios_mouselock
	.globl	_SDL_AtariXbios_mouseb
	.globl	_SDL_AtariXbios_mousex
	.globl	_SDL_AtariXbios_mousey
	.globl	_SDL_AtariXbios_joystick

/*--- Vector installer ---*/

_SDL_AtariXbios_Install:
	movel	sp@(4),a0

	/* Stop interrupts */

	movew	#0x2700,sr

	/* Save old mouse vector, set our routine */

	clrl	oldmousevector
	movel	sp@(8),d0
	beqs	no_new_mouse_vector
#if defined(__mcoldfire__)
	movel	a0@(16),d1
	movel	d1,oldmousevector
#else
	movel	a0@(16),oldmousevector
#endif
	movel	d0,a0@(16)
no_new_mouse_vector:

	/* Save old joystick vector, set our routine */

	clrl	oldjoystickvector
	movel	sp@(12),d0
	beqs	no_new_joystick_vector
#if defined(__mcoldfire__)
	movel	a0@(24),d1
	movel	d1,oldjoystickvector
#else
	movel	a0@(24),oldjoystickvector
#endif
	movel	d0,a0@(24)
no_new_joystick_vector:

	/* Restart interrupts */

	movew	#0x2300,sr

	rts

/*--- Vector restorer ---*/

_SDL_AtariXbios_Restore:
	movel	sp@(4),a0

	/* Stop interrupts */

	movew	#0x2700,sr

	/* Restore mouse vector */

	movel	oldmousevector,d0
	beqs	no_restore_mouse
	movel	d0,a0@(16)
no_restore_mouse:

	/* Restore joystick vector */

	movel	oldjoystickvector,d0
	beqs	no_restore_joystick
	movel	d0,a0@(24)
no_restore_joystick:

	/* Restart interrupts */

	movew	#0x2300,sr

	rts

/*--- Our mouse vector ---*/

	.text
	.even
	.ascii "XBRA"
	.ascii "LSDL"
	.comm	oldmousevector,4*1
_SDL_AtariXbios_MouseVector:
#if defined(__mcoldfire__)
	lea	sp@(-8),sp
	moveml	d0-d1,sp@
#else
	movel	d0,sp@-
#endif

	/* Mouse buttons */
	moveb	(a0),d0
#if defined(__mcoldfire__)
	andl	#3,d0
#else
	andw	#3,d0
#endif
	movew	d0,_SDL_AtariXbios_mouseb

	/* X movement */
	moveb	a0@(1),d0
	extw	d0
#if defined(__mcoldfire__)
	movew	_SDL_AtariXbios_mousex,d1
	addl	d1,d0
	movew	d0,_SDL_AtariXbios_mousex
#else
	addw	d0,_SDL_AtariXbios_mousex
#endif

	/* Y movement */
	moveb	a0@(2),d0
	extw	d0
#if defined(__mcoldfire__)
	movew	_SDL_AtariXbios_mousey,d1
	addl	d1,d0
	movew	d0,_SDL_AtariXbios_mousey
#else
	addw	d0,_SDL_AtariXbios_mousey
#endif

	/* Lock mouse position ? */
	tstw	_SDL_AtariXbios_mouselock
	beq.s	no_mouse_lock
	clrb	a0@(1)
	clrb	a0@(2)
no_mouse_lock:

	/* Jump through old vector */
#if defined(__mcoldfire__)
	moveml	sp@,d0-d1
	lea	sp@(8),sp
#else
	movel	sp@+,d0
#endif

	movel	oldmousevector,sp@-
	rts

	.data
	.even
	.comm	_SDL_AtariXbios_mouselock,2*1
	.comm	_SDL_AtariXbios_mousex,2*1
	.comm	_SDL_AtariXbios_mousey,2*1
	.comm	_SDL_AtariXbios_mouseb,2*1

/*--- Our joystick vector ---*/

	.text
	.even
	.ascii "XBRA"
	.ascii "LSDL"
	.comm	oldjoystickvector,4*1
_SDL_AtariXbios_JoystickVector:
	movel	d0,sp@-

	/* New joystick state */
	moveb	a0@(2),d0
#if defined(__mcoldfire__)
	andl	#0x8f,d0
#else
	andw	#0x8f,d0
#endif
	movew	d0,_SDL_AtariXbios_joystick

	/* Jump through old vector */
	movel	sp@+,d0

	movel	oldjoystickvector,sp@-
	rts

	.data
	.even
	.comm	_SDL_AtariXbios_joystick,2*1
