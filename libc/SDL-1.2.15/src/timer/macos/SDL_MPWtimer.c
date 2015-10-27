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

#ifdef SDL_TIMER_MACOS

#include <Types.h>
#include <Timer.h>
#include <OSUtils.h>
#include <Gestalt.h>
#include <Processes.h>

#include <LowMem.h>

#include "SDL_timer.h"
#include "../SDL_timer_c.h"

#define MS_PER_TICK	(1000/60)		/* MacOS tick = 1/60 second */

/* Note: This is only a step above the original 1/60s implementation.
 *       For a good implementation, see FastTimes.[ch], by Matt Slot.
 */
#define USE_MICROSECONDS
#define WideTo64bit(w)	(*(UInt64 *) &(w))

UInt64 start;

void SDL_StartTicks(void)
{
#ifdef USE_MICROSECONDS
	UnsignedWide now;
	
	Microseconds(&now);
	start = WideTo64bit(now);
#else
	/* FIXME: Should we implement a wrapping algorithm, like Win32? */
#endif
}

Uint32 SDL_GetTicks(void)
{
#ifdef USE_MICROSECONDS
	UnsignedWide now;
	
	Microseconds(&now);
	return (Uint32)((WideTo64bit(now)-start)/1000);
#else
	return(LMGetTicks()*MS_PER_TICK);
#endif
}

void SDL_Delay(Uint32 ms)
{
#ifdef USE_MICROSECONDS
	Uint32 end_ms;
	
	end_ms = SDL_GetTicks() + ms;
	do {
		/* FIXME: Yield CPU? */ ;
	} while ( SDL_GetTicks() < end_ms );
#else
	UInt32		unused; /* MJS */
	Delay(ms/MS_PER_TICK, &unused);
#endif
}


/* Data to handle a single periodic alarm */
typedef struct _ExtendedTimerRec
{
	TMTask		     tmTask;
	ProcessSerialNumber  taskPSN;
} ExtendedTimerRec, *ExtendedTimerPtr;

static ExtendedTimerRec gExtendedTimerRec;


int SDL_SYS_TimerInit(void)
{
	/* We don't need a setup? */
	return(0);
}

void SDL_SYS_TimerQuit(void)
{
	/* We don't need a cleanup? */
	return;
}

/* Our Stub routine to set up and then call the real routine. */
pascal void TimerCallbackProc(TMTaskPtr tmTaskPtr)
{
	Uint32 ms;

	WakeUpProcess(&((ExtendedTimerPtr) tmTaskPtr)->taskPSN);

	ms = SDL_alarm_callback(SDL_alarm_interval);
	if ( ms ) {
		SDL_alarm_interval = ROUND_RESOLUTION(ms);
		PrimeTime((QElemPtr)&gExtendedTimerRec.tmTask,
		          SDL_alarm_interval);
	} else {
		SDL_alarm_interval = 0;
	}
}

int SDL_SYS_StartTimer(void)
{
	/*
	 * Configure the global structure that stores the timing information.
	 */
	gExtendedTimerRec.tmTask.qLink = NULL;
	gExtendedTimerRec.tmTask.qType = 0;
	gExtendedTimerRec.tmTask.tmAddr = NewTimerUPP(TimerCallbackProc);
	gExtendedTimerRec.tmTask.tmCount = 0;
	gExtendedTimerRec.tmTask.tmWakeUp = 0;
	gExtendedTimerRec.tmTask.tmReserved = 0;
	GetCurrentProcess(&gExtendedTimerRec.taskPSN);

	/* Install the task record */
	InsXTime((QElemPtr)&gExtendedTimerRec.tmTask);

	/* Go! */
	PrimeTime((QElemPtr)&gExtendedTimerRec.tmTask, SDL_alarm_interval);
	return(0);
}

void SDL_SYS_StopTimer(void)
{
	RmvTime((QElemPtr)&gExtendedTimerRec.tmTask);
}

#endif /* SDL_TIMER_MACOS */
