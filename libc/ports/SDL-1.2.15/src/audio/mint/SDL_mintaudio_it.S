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

/*
	Audio interrupts

	Patrice Mandin, Didier Méquignon
 */

	.text

	.globl	_SDL_MintAudio_Callback

	.globl	_SDL_MintAudio_XbiosInterrupt
	.globl	_SDL_MintAudio_XbiosInterruptMeasureClock
	.globl	_SDL_MintAudio_Dma8Interrupt
	.globl	_SDL_MintAudio_StfaInterrupt

	.globl	_SDL_MintAudio_mutex
	.globl	_SDL_MintAudio_audiobuf
	.globl	_SDL_MintAudio_numbuf
	.globl	_SDL_MintAudio_audiosize
	.globl	_SDL_MintAudio_clocktics
	.globl	_SDL_MintAudio_hasfpu

	.globl	_SDL_MintAudio_stfa

/*
	How it works:
	- Audio is playing buffer #0 (resp. #1)
	- We must calculate a sample in buffer #1 (resp. #0)
	  so we first call the callback to do it
	- Then we swap the buffers
*/

#define	savptr	0x4a2
#define	savamt	0x46

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

/*--- Xbios interrupt vector to measure Falcon external clock ---*/

_SDL_MintAudio_XbiosInterruptMeasureClock:          /* 1 mS */
#if defined(__mcoldfire__)
	movel	d0,sp@-
	
	moveql	#0,d0
	btst	d0,0xFFFF8901:w	/* state DMA sound */
#else
	btst	#0,0xFFFF8901:w	/* state DMA sound */
#endif
	beqs	SDL_MintAudio_EndIntMeasure
	addql	#1,_SDL_MintAudio_clocktics
SDL_MintAudio_EndIntMeasure:
#if defined(__mcoldfire__)
	moveql	#5,d0
	bclr	d0,0xFFFFFA0F:w	/* Clear service bit */

	movel	sp@+,d0
#else
	bclr	#5,0xFFFFFA0F:w	/* Clear service bit */
#endif
	rte

/*--- Xbios interrupt vector ---*/

_SDL_MintAudio_XbiosInterrupt:
#if defined(__mcoldfire__)
	lea	sp@(-60),sp
	moveml	d0-d7/a0-a6,sp@
#else
	moveml	d0-d7/a0-a6,sp@-
#endif

	/* Reenable interrupts, so other interrupts can work */
	movew	#0x2300,sr

	/* Clear service bit, so other MFP interrupts can work */
#if defined(__mcoldfire__)
	moveql	#5,d0
	bclr	d0,0xfffffa0f:w
#else
	bclr	#5,0xfffffa0f:w
#endif

	/* Check if we are not already running */
	tstw	_SDL_MintAudio_mutex
	bne	SDL_MintAudio_XbiosEnd

#if defined(__mcoldfire__)
	movew	_SDL_MintAudio_mutex,d0
	notl	d0
	movew	d0,_SDL_MintAudio_mutex

	movew	_SDL_MintAudio_numbuf,d1
	eorl	#1,d1
	movew	d1,_SDL_MintAudio_numbuf
#else
	notw	_SDL_MintAudio_mutex

	/* Swap buffers */
	eorw	#1,_SDL_MintAudio_numbuf
#endif
	
	/* Save FPU if needed */
	tstw	_SDL_MintAudio_hasfpu
	beqs	SDL_MintAudio_Xbios_nofpu1
	SAVE_FPU_CONTEXT
SDL_MintAudio_Xbios_nofpu1:

	/* Callback */
	jsr	_SDL_MintAudio_Callback

	/* Restore FPU if needed */
	tstw	_SDL_MintAudio_hasfpu
	beqs	SDL_MintAudio_Xbios_nofpu2
	RESTORE_FPU_CONTEXT
SDL_MintAudio_Xbios_nofpu2:

	/* Reserve space for registers */
#if defined(__mcoldfire__)
	movel	#savamt,d0
	subl	d0,savptr
#else
	subl	#savamt,savptr
#endif

	/* Set new buffer */

	moveq	#0,d0
	movel	_SDL_MintAudio_audiosize,d1

	movew	_SDL_MintAudio_numbuf,d0
	lsll	#2,d0
	lea	_SDL_MintAudio_audiobuf,a0
	movel	a0@(d0:l),a1

	lea	a1@(d1:l),a2

	movel	a2,sp@-
	movel	a1,sp@-
	clrw	sp@-
	movew	#131,sp@-
	trap	#14
	lea	sp@(12),sp

	/* Restore registers space */
#if defined(__mcoldfire__)
	movel	#savamt,d0
	addl	d0,savptr
#else
	addl	#savamt,savptr
#endif

	clrw	_SDL_MintAudio_mutex
SDL_MintAudio_XbiosEnd:
#if defined(__mcoldfire__)
	moveml	sp@,d0-d7/a0-a6
	lea	sp@(60),sp
#else
	moveml	sp@+,d0-d7/a0-a6
#endif
	rte

/*--- DMA 8 bits interrupt vector ---*/

_SDL_MintAudio_Dma8Interrupt:
#if defined(__mcoldfire__)
	lea	sp@(-16),sp
	moveml	d0-d1/a0-a1,sp@
#else
	moveml	d0-d1/a0-a1,sp@-
#endif

	/* Reenable interrupts, so other interrupts can work */
	movew	#0x2300,sr

	/* Clear service bit, so other MFP interrupts can work */
#if defined(__mcoldfire__)
	moveql	#5,d0
	bclr	d0,0xfffffa0f:w
#else
	bclr	#5,0xfffffa0f:w
#endif
	/* Check if we are not already running */
	tstw	_SDL_MintAudio_mutex
	bne	SDL_MintAudio_Dma8End

#if defined(__mcoldfire__)
	movew	_SDL_MintAudio_mutex,d0
	notl	d0
	movew	d0,_SDL_MintAudio_mutex

	movew	_SDL_MintAudio_numbuf,d1
	eorl	#1,d1
	movew	d1,_SDL_MintAudio_numbuf
#else
	notw	_SDL_MintAudio_mutex

	/* Swap buffers */
	eorw	#1,_SDL_MintAudio_numbuf
#endif

	/* Save FPU if needed */
	tstw	_SDL_MintAudio_hasfpu
	beqs	SDL_MintAudio_Dma8_nofpu1
	SAVE_FPU_CONTEXT
SDL_MintAudio_Dma8_nofpu1:

	/* Callback */
	jsr	_SDL_MintAudio_Callback

	/* Restore FPU if needed */
	tstw	_SDL_MintAudio_hasfpu
	beqs	SDL_MintAudio_Dma8_nofpu2
	RESTORE_FPU_CONTEXT
SDL_MintAudio_Dma8_nofpu2:

	/* Set new buffer */

	moveq	#0,d0

	movew	_SDL_MintAudio_numbuf,d0
	lsll	#2,d0
	lea	_SDL_MintAudio_audiobuf,a0
	movel	a0@(d0:l),d1

	/* Modify DMA addresses */
	lea	0xffff8900:w,a0

	movel	d1,d0

	moveb	d0,a0@(0x07)	/* Start address */
	lsrl	#8,d0
	moveb	d0,a0@(0x05)
	lsrl	#8,d0
	moveb	d0,a0@(0x03)

	addl	_SDL_MintAudio_audiosize,d1

	movel	d1,d0

	moveb	d0,a0@(0x13)	/* End address */
	lsrl	#8,d0
	moveb	d0,a0@(0x11)
	lsrl	#8,d0
	moveb	d0,a0@(0x0f)

	clrw	_SDL_MintAudio_mutex
SDL_MintAudio_Dma8End:
#if defined(__mcoldfire__)
	moveml	sp@,d0-d1/a0-a1
	lea	sp@(16),sp
#else
	moveml	sp@+,d0-d1/a0-a1
#endif
	rte

/*--- STFA interrupt vector ---*/

STFA_SOUND_START	=	6
STFA_SOUND_END		=	STFA_SOUND_START+8

_SDL_MintAudio_StfaInterrupt:
	/* Reenable interrupts, so other interrupts can work */
	movew	#0x2300,sr

	/* Check if we are not already running */
	tstw	_SDL_MintAudio_mutex

#if defined(__mcoldfire__)
	bne	SDL_MintAudio_StfaEnd

	lea	sp@(-60),sp
	moveml	d0-d7/a0-a6,sp@

	movew	_SDL_MintAudio_mutex,d0
	notl	d0
	movew	d0,_SDL_MintAudio_mutex

	movew	_SDL_MintAudio_numbuf,d1
	eorl	#1,d1
	movew	d1,_SDL_MintAudio_numbuf
#else
	bnes	SDL_MintAudio_StfaEnd

	moveml	d0-d7/a0-a6,sp@-

	notw	_SDL_MintAudio_mutex

	/* Swap buffers */
	eorw	#1,_SDL_MintAudio_numbuf
#endif
	
	/* Save FPU if needed */
	tstw	_SDL_MintAudio_hasfpu
	beqs	SDL_MintAudio_Stfa_nofpu1
	SAVE_FPU_CONTEXT
SDL_MintAudio_Stfa_nofpu1:

	/* Callback */
	jsr	_SDL_MintAudio_Callback

	/* Restore FPU if needed */
	tstw	_SDL_MintAudio_hasfpu
	beqs	SDL_MintAudio_Stfa_nofpu2
	RESTORE_FPU_CONTEXT
SDL_MintAudio_Stfa_nofpu2:

	/* Set new buffer */

	moveq	#0,d0
	movel	_SDL_MintAudio_stfa,a1

	movew	_SDL_MintAudio_numbuf,d0
	lsll	#2,d0
	lea	_SDL_MintAudio_audiobuf,a0
	movel	a0@(d0:l),d1

	/* Modify STFA replay buffers */
	movel	d1,a1@(STFA_SOUND_START)
	addl	_SDL_MintAudio_audiosize,d1
	movel	d1,a1@(STFA_SOUND_END)

#if defined(__mcoldfire__)
	moveml	sp@,d0-d7/a0-a6
	lea	sp@(60),sp
#else
	moveml	sp@+,d0-d7/a0-a6
#endif
	clrw	_SDL_MintAudio_mutex
SDL_MintAudio_StfaEnd:
	rte
