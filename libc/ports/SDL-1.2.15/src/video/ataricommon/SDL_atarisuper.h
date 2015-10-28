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

#ifndef _ATARI_SUPER_h
#define _ATARI_SUPER_h

#include "SDL_stdinc.h"

#ifndef SuperToUser

/*
 * Safe binding to switch back from supervisor to user mode.
 * On TOS or EmuTOS, if the stack pointer has changed between Super(0)
 * and Super(oldssp), the resulting user stack pointer is wrong.
 * This bug does not occur with FreeMiNT.
 * So the safe way to return from supervisor to user mode is to backup
 * the stack pointer then restore it after the trap.
 * Sometimes, GCC optimizes the stack usage, so this matters.
 */
#define SuperToUser(ptr)						\
(void)__extension__							\
({									\
	register long retvalue __asm__("d0");				\
	register long sp_backup;					\
									\
	__asm__ volatile						\
	(								\
		"movl	sp,%1\n\t"					\
		"movl	%2,sp@-\n\t"					\
		"movw	#0x20,sp@-\n\t"					\
		"trap	#1\n\t"						\
		"movl	%1,sp\n\t"					\
	: "=r"(retvalue), "=&r"(sp_backup)	/* outputs */		\
	: "g"((long)(ptr)) 			/* inputs */		\
	: "d1", "d2", "a0", "a1", "a2"		\
	);								\
})

#endif /* SuperToUser */

#endif /* _ATARI_SUPER_h */
