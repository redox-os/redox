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
 *	VBL queue routine
 *
 *	Patrice Mandin
 */

#define _vbl_queue 0x456
#define _hz_200    0x4ba

	.text

	.globl	_SDL_AtariVblInstall
	.globl	_SDL_AtariVblUninstall

	.globl	_SDL_MintAudio_hasfpu

/*--- Save/restore FPU context ---*/

#if defined(__mcoldfire__)

#define SAVE_FPU_CONTEXT \
	lea		sp@(-216),sp;	\
	fsave		sp@;	\
	fmovel		fpiar,sp@-;	\
	lea		sp@(-64),sp;	\
	fmovemd	fp0-fp7,sp@

#define RESTORE_FPU_CONTEXT	\
	fmovemd		sp@,fp0-fp7;	\
	lea		sp@(64),sp;	\
	fmovel		sp@+,fpiar;	\
	frestore	sp@;	\
	lea		sp@(216),sp

#else

#define SAVE_FPU_CONTEXT	\
	.chip	68k/68881;	\
	fsave	sp@-;	\
	fmoveml fpcr/fpsr/fpiar,sp@-;	\
	fmovemx	fp0-fp7,sp@-;	\
	.chip	68k

#define RESTORE_FPU_CONTEXT	\
	.chip	68k/68881;	\
	fmovemx	sp@+,fp0-fp7;	\
	fmoveml	sp@+,fpcr/fpsr/fpiar;	\
	frestore	sp@+;	\
	.chip	68k

#endif

/*--- Vector installer ---*/

_SDL_AtariVblInstall:
#if defined(__mcoldfire__)
	movel	sp@(4),d0
	movel	d0,my_vector
#else
	movel	sp@(4),my_vector
#endif

	lea		_my_vbl,a0

	clrw	vbl_mutex
#if defined(__mcoldfire__)
	movel	_hz_200.w,d0
	movel	d0, _SDL_Atari_hz200
#else
	movel	_hz_200.w, _SDL_Atari_hz200
#endif

	/* Stop interrupts */

	movew	#0x2700,sr

	/* Read vbl_queue pointer */
	movel	_vbl_queue.w,a1

	/* Search a free place */
	moveq	#7,d0
bcl_search_place:
	movel	(a1),d1
	beqs	place_found
	addql	#4,a1
#if defined(__mcoldfire__)
	subql	#1,d0
	bpls	bcl_search_place
#else
	dbra	d0,bcl_search_place
#endif

	/* Not found */
	moveq	#1,d0
	bras	exit_vbl_queue

	/* Then install ourselves */
place_found:
	movel	a0,(a1)
	moveq	#0,d0	

exit_vbl_queue:
	/* Restart interrupts */
	movew	#0x2300,sr

	rts

/*--- Vector uninstaller ---*/

_SDL_AtariVblUninstall:
	movel	sp@(4),d0
	cmpl	my_vector,d0
	bnes	badvector
	
	movel	#_my_vbl,d0

	/* Stop interrupts */

	movew	#0x2700,sr

	/* Read vbl_queue pointer */
	movel	_vbl_queue.w,a1

	/* Search where we are */
	moveq	#7,d1
bcl2_search_place:
	cmpl	(a1),d0
	bnes	next_place
	clrl	(a1)
	moveq	#0,d1
next_place:
	addql	#4,a1
#if defined(__mcoldfire__)
	subql	#1,d1
	bpls	bcl_search_place
#else
	dbra	d1,bcl2_search_place
#endif

	/* Restart interrupts */
	movew	#0x2300,sr
badvector:
	rts

/*--- Our vbl ---*/

_my_vbl:
#if defined(__mcoldfire__)
	lea	sp@(-60),sp
	moveml	d0-d7/a0-a6,sp@
#else
	moveml	d0-d7/a0-a6,sp@-
#endif

	/* Update _hz_200 */
#if defined(__mcoldfire__)
	movel	_hz_200.w,d0
	movel	d0, _SDL_Atari_hz200
#else
	movel	_hz_200.w, _SDL_Atari_hz200
#endif

	/* Verify if this is not already running */

	tstw	vbl_mutex
	bnes	vbl_end
#if defined(__mcoldfire__)
	movew	vbl_mutex,d0
	notl	d0
	movew	d0,vbl_mutex
#else
	notw	vbl_mutex
#endif

	/* Save FPU if needed */
	tstw	_SDL_MintAudio_hasfpu
	beqs	SDL_AtariVbl_nofpu1
	SAVE_FPU_CONTEXT
SDL_AtariVbl_nofpu1:

	movel	my_vector,a0
	jsr		a0@

	/* Restore FPU if needed */
	tstw	_SDL_MintAudio_hasfpu
	beqs	SDL_AtariVbl_Xbios_nofpu2
	RESTORE_FPU_CONTEXT
SDL_AtariVbl_Xbios_nofpu2:

	clrw	vbl_mutex
vbl_end:
#if defined(__mcoldfire__)
	moveml	sp@,d0-d7/a0-a6
	lea	sp@(60),sp
#else
	moveml	sp@+,d0-d7/a0-a6
#endif
	rts

	.data
	.even
	.comm	_SDL_Atari_hz200,4*1
	.even
	.comm	vbl_mutex,2*1
	.even
	.comm	my_vector,4*1
