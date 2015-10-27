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

#ifdef SDL_TIMER_WINCE

#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <mmsystem.h>

#include "SDL_thread.h"
#include "SDL_timer.h"
#include "../SDL_timer_c.h"

static Uint64 start_date;
static Uint64 start_ticks;

static Uint64 wce_ticks(void)
{
  return((Uint64)GetTickCount());
}

static Uint64 wce_date(void)
{
  union
  {
	FILETIME ftime;
	Uint64 itime;
  } ftime;
  SYSTEMTIME stime;

  GetSystemTime(&stime);
  SystemTimeToFileTime(&stime,&ftime.ftime);
  ftime.itime/=10000; // Convert 100ns intervals to 1ms intervals
  // Remove ms portion, which can't be relied on
  ftime.itime -= (ftime.itime % 1000);
  return(ftime.itime);
}

static Sint32 wce_rel_ticks(void)
{
  return((Sint32)(wce_ticks()-start_ticks));
}

static Sint32 wce_rel_date(void)
{
  return((Sint32)(wce_date()-start_date));
}

/* Return time in ms relative to when SDL was started */
Uint32 SDL_GetTicks()
{
  Sint32 offset=wce_rel_date()-wce_rel_ticks();
  if((offset < -1000) || (offset > 1000))
  {
//    fprintf(stderr,"Time desync(%+d), resyncing\n",offset/1000);
	start_ticks-=offset;
  }

  return((Uint32)wce_rel_ticks());
}

/* Give up approx. givem milliseconds to the OS. */
void SDL_Delay(Uint32 ms)
{
  Sleep(ms);
}

/* Recard start-time of application for reference */
void SDL_StartTicks(void)
{
  start_date=wce_date();
  start_ticks=wce_ticks();
}

static UINT WIN_timer;

#if ( _WIN32_WCE <= 420 )

static HANDLE timersThread = 0;
static HANDLE timersQuitEvent = 0;

DWORD TimersThreadProc(void *data)
{
	while(WaitForSingleObject(timersQuitEvent, 10) == WAIT_TIMEOUT)
	{
		SDL_ThreadedTimerCheck();
	}
	return 0;
}

int SDL_SYS_TimerInit(void)
{
	// create a thread to process a threaded timers
	// SetTimer does not suit the needs because 
	// TimerCallbackProc will be called only when WM_TIMER occured

	timersQuitEvent = CreateEvent(0, TRUE, FALSE, 0);
	if( !timersQuitEvent )
	{
		SDL_SetError("Cannot create event for timers thread");
		return -1;
	}
	timersThread = CreateThread(NULL, 0, TimersThreadProc, 0, 0, 0);
	if( !timersThread )
	{
		SDL_SetError("Cannot create timers thread, check amount of RAM available");
		return -1;
	}
	SetThreadPriority(timersThread, THREAD_PRIORITY_HIGHEST);

	return(SDL_SetTimerThreaded(1));
}

void SDL_SYS_TimerQuit(void)
{
	SetEvent(timersQuitEvent);
	if( WaitForSingleObject(timersThread, 2000) == WAIT_TIMEOUT )
		TerminateThread(timersThread, 0);
	CloseHandle(timersThread);
	CloseHandle(timersQuitEvent);
	return;
}

#else

#pragma comment(lib, "mmtimer.lib")

/* Data to handle a single periodic alarm */
static UINT timerID = 0;

static void CALLBACK HandleAlarm(UINT uID,  UINT uMsg, DWORD dwUser,
						DWORD dw1, DWORD dw2)
{
	SDL_ThreadedTimerCheck();
}


int SDL_SYS_TimerInit(void)
{
	MMRESULT result;

	/* Set timer resolution */
	result = timeBeginPeriod(TIMER_RESOLUTION);
	if ( result != TIMERR_NOERROR ) {
		SDL_SetError("Warning: Can't set %d ms timer resolution",
							TIMER_RESOLUTION);
	}
	/* Allow 10 ms of drift so we don't chew on CPU */
	timerID = timeSetEvent(TIMER_RESOLUTION,1,HandleAlarm,0,TIME_PERIODIC);
	if ( ! timerID ) {
		SDL_SetError("timeSetEvent() failed");
		return(-1);
	}
	return(SDL_SetTimerThreaded(1));
}

void SDL_SYS_TimerQuit(void)
{
	if ( timerID ) {
		timeKillEvent(timerID);
	}
	timeEndPeriod(TIMER_RESOLUTION);
}

#endif

int SDL_SYS_StartTimer(void)
{
	SDL_SetError("Internal logic error: WinCE uses threaded timer");
	return(-1);
}

void SDL_SYS_StopTimer(void)
{
	return;
}

#endif /* SDL_TIMER_WINCE */
