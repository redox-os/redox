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

#ifdef SDL_TIMER_OS2

#define INCL_DOSMISC
#define INCL_DOSERRORS
#define INCL_DOSSEMAPHORES
#define INCL_DOSDATETIME
#define INCL_DOSPROCESS
#define INCL_DOSPROFILE
#define INCL_DOSEXCEPTIONS
#include <os2.h>

#include "SDL_thread.h"
#include "SDL_timer.h"
#include "../SDL_timer_c.h"


#define TIME_WRAP_VALUE (~(DWORD)0)

/* The first high-resolution ticks value of the application */
static long long hires_start_ticks;
/* The number of ticks per second of the high-resolution performance counter */
static ULONG hires_ticks_per_second;

void SDL_StartTicks(void)
{
        DosTmrQueryFreq(&hires_ticks_per_second);
        DosTmrQueryTime((PQWORD)&hires_start_ticks);
}

DECLSPEC Uint32 SDLCALL SDL_GetTicks(void)
{
        long long hires_now;
        ULONG ticks = ticks;

        DosTmrQueryTime((PQWORD)&hires_now);
/*
        hires_now -= hires_start_ticks;
        hires_now *= 1000;
        hires_now /= hires_ticks_per_second;
*/
        /* inline asm to avoid runtime inclusion */
        _asm {
           push edx
           push eax
           mov eax, dword ptr hires_now
           mov edx, dword ptr hires_now+4
           sub eax, dword ptr hires_start_ticks
           sbb edx, dword ptr hires_start_ticks+4
           mov ebx,1000
           mov ecx,edx
           mul ebx
           push eax
           push edx
           mov eax,ecx
           mul ebx
           pop eax
           add edx,eax
           pop eax
           mov ebx, dword ptr hires_ticks_per_second
           div ebx
           mov dword ptr ticks, eax
           pop edx
           pop eax
        }

        return ticks;

}

/* High resolution sleep, originally made by Ilya Zakharevich */
DECLSPEC void SDLCALL SDL_Delay(Uint32 ms)
{
  /* This is similar to DosSleep(), but has 8ms granularity in time-critical
     threads even on Warp3. */
  HEV     hevEvent1     = 0;   /* Event semaphore handle    */
  HTIMER  htimerEvent1  = 0;   /* Timer handle              */
  APIRET  rc            = NO_ERROR;  /* Return code               */
  int ret = 1;
  ULONG priority = 0, nesting;   /* Shut down the warnings */
  PPIB pib;
  PTIB tib;
  char *e = NULL;
  APIRET badrc;
  int switch_priority = 50;

  DosCreateEventSem(NULL,      /* Unnamed */
                    &hevEvent1,  /* Handle of semaphore returned */
                    DC_SEM_SHARED, /* Shared needed for DosAsyncTimer */
                    FALSE);      /* Semaphore is in RESET state  */

  if (ms >= switch_priority)
    switch_priority = 0;
  if (switch_priority)
  {
    if (DosGetInfoBlocks(&tib, &pib)!=NO_ERROR)
      switch_priority = 0;
    else
    {
 /* In Warp3, to switch scheduling to 8ms step, one needs to do 
    DosAsyncTimer() in time-critical thread.  On laters versions,
    more and more cases of wait-for-something are covered.

    It turns out that on Warp3fp42 it is the priority at the time
    of DosAsyncTimer() which matters.  Let's hope that this works
    with later versions too...  XXXX
  */
      priority = (tib->tib_ptib2->tib2_ulpri);
      if ((priority & 0xFF00) == 0x0300) /* already time-critical */
        switch_priority = 0;
 /* Make us time-critical.  Just modifying TIB is not enough... */
 /* tib->tib_ptib2->tib2_ulpri = 0x0300;*/
 /* We do not want to run at high priority if a signal causes us
    to longjmp() out of this section... */
      if (DosEnterMustComplete(&nesting))
        switch_priority = 0;
      else
        DosSetPriority(PRTYS_THREAD, PRTYC_TIMECRITICAL, 0, 0);
    }
  }

  if ((badrc = DosAsyncTimer(ms,
        (HSEM) hevEvent1, /* Semaphore to post        */
        &htimerEvent1))) /* Timer handler (returned) */
    e = "DosAsyncTimer";

  if (switch_priority && tib->tib_ptib2->tib2_ulpri == 0x0300)
  {
 /* Nobody switched priority while we slept...  Ignore errors... */
 /* tib->tib_ptib2->tib2_ulpri = priority; */ /* Get back... */
    if (!(rc = DosSetPriority(PRTYS_THREAD, (priority>>8) & 0xFF, 0, 0)))
      rc = DosSetPriority(PRTYS_THREAD, 0, priority & 0xFF, 0);
  }
  if (switch_priority)
    rc = DosExitMustComplete(&nesting); /* Ignore errors */

  /* The actual blocking call is made with "normal" priority.  This way we
     should not bother with DosSleep(0) etc. to compensate for us interrupting
     higher-priority threads.  The goal is to prohibit the system spending too
     much time halt()ing, not to run us "no matter what". */
  if (!e)     /* Wait for AsyncTimer event */
    badrc = DosWaitEventSem(hevEvent1, SEM_INDEFINITE_WAIT);

  if (e) ;    /* Do nothing */
  else if (badrc == ERROR_INTERRUPT)
    ret = 0;
  else if (badrc)
    e = "DosWaitEventSem";
  if ((rc = DosCloseEventSem(hevEvent1)) && !e) { /* Get rid of semaphore */
    e = "DosCloseEventSem";
    badrc = rc;
  }
  if (e)
  {
    SDL_SetError("[SDL_Delay] : Had error in %s(), rc is 0x%x\n", e, badrc);
  }
}

/* Data to handle a single periodic alarm */
static int timer_alive = 0;
static SDL_Thread *timer = NULL;

static int SDLCALL RunTimer(void *unused)
{
        DosSetPriority(PRTYS_THREAD, PRTYC_TIMECRITICAL, 0, 0);
        while ( timer_alive ) {
                if ( SDL_timer_running ) {
                        SDL_ThreadedTimerCheck();
                }
                SDL_Delay(10);
        }
        return(0);
}

/* This is only called if the event thread is not running */
int SDL_SYS_TimerInit(void)
{
        timer_alive = 1;
        timer = SDL_CreateThread(RunTimer, NULL);
        if ( timer == NULL )
                return(-1);
        return(SDL_SetTimerThreaded(1));
}

void SDL_SYS_TimerQuit(void)
{
        timer_alive = 0;
        if ( timer ) {
                SDL_WaitThread(timer, NULL);
                timer = NULL;
        }
}

int SDL_SYS_StartTimer(void)
{
        SDL_SetError("Internal logic error: OS/2 uses threaded timer");
        return(-1);
}

void SDL_SYS_StopTimer(void)
{
        return;
}

#endif /* SDL_TIMER_OS2 */
