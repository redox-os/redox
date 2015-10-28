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

/* Initialization code for SDL */

#include "SDL.h"
#include "SDL_fatal.h"
#if !SDL_VIDEO_DISABLED
#include "video/SDL_leaks.h"
#endif

#if SDL_THREAD_PTH
#include <pth.h>
#endif

/* Initialization/Cleanup routines */
#if !SDL_JOYSTICK_DISABLED
extern int  SDL_JoystickInit(void);
extern void SDL_JoystickQuit(void);
#endif
#if !SDL_CDROM_DISABLED
extern int  SDL_CDROMInit(void);
extern void SDL_CDROMQuit(void);
#endif
#if !SDL_TIMERS_DISABLED
extern void SDL_StartTicks(void);
extern int  SDL_TimerInit(void);
extern void SDL_TimerQuit(void);
#endif

/* The current SDL version */
static SDL_version version = 
	{ SDL_MAJOR_VERSION, SDL_MINOR_VERSION, SDL_PATCHLEVEL };

/* The initialized subsystems */
static Uint32 SDL_initialized = 0;
#if !SDL_TIMERS_DISABLED
static Uint32 ticks_started = 0;
#endif

#ifdef CHECK_LEAKS
int surfaces_allocated = 0;
#endif

int SDL_InitSubSystem(Uint32 flags)
{
#if !SDL_TIMERS_DISABLED
	/* Initialize the timer subsystem */
	if ( ! ticks_started ) {
		SDL_StartTicks();
		ticks_started = 1;
	}
	if ( (flags & SDL_INIT_TIMER) && !(SDL_initialized & SDL_INIT_TIMER) ) {
		if ( SDL_TimerInit() < 0 ) {
			return(-1);
		}
		SDL_initialized |= SDL_INIT_TIMER;
	}
#else
	if ( flags & SDL_INIT_TIMER ) {
		SDL_SetError("SDL not built with timer support");
		return(-1);
	}
#endif

#if !SDL_VIDEO_DISABLED
	/* Initialize the video/event subsystem */
	if ( (flags & SDL_INIT_VIDEO) && !(SDL_initialized & SDL_INIT_VIDEO) ) {
		if ( SDL_VideoInit(SDL_getenv("SDL_VIDEODRIVER"),
		                   (flags&SDL_INIT_EVENTTHREAD)) < 0 ) {
			return(-1);
		}
		SDL_initialized |= SDL_INIT_VIDEO;
	}
#else
	if ( flags & SDL_INIT_VIDEO ) {
		SDL_SetError("SDL not built with video support");
		return(-1);
	}
#endif

#if !SDL_AUDIO_DISABLED
	/* Initialize the audio subsystem */
	if ( (flags & SDL_INIT_AUDIO) && !(SDL_initialized & SDL_INIT_AUDIO) ) {
		if ( SDL_AudioInit(SDL_getenv("SDL_AUDIODRIVER")) < 0 ) {
			return(-1);
		}
		SDL_initialized |= SDL_INIT_AUDIO;
	}
#else
	if ( flags & SDL_INIT_AUDIO ) {
		SDL_SetError("SDL not built with audio support");
		return(-1);
	}
#endif

#if !SDL_JOYSTICK_DISABLED
	/* Initialize the joystick subsystem */
	if ( (flags & SDL_INIT_JOYSTICK) &&
	     !(SDL_initialized & SDL_INIT_JOYSTICK) ) {
		if ( SDL_JoystickInit() < 0 ) {
			return(-1);
		}
		SDL_initialized |= SDL_INIT_JOYSTICK;
	}
#else
	if ( flags & SDL_INIT_JOYSTICK ) {
		SDL_SetError("SDL not built with joystick support");
		return(-1);
	}
#endif

#if !SDL_CDROM_DISABLED
	/* Initialize the CD-ROM subsystem */
	if ( (flags & SDL_INIT_CDROM) && !(SDL_initialized & SDL_INIT_CDROM) ) {
		if ( SDL_CDROMInit() < 0 ) {
			return(-1);
		}
		SDL_initialized |= SDL_INIT_CDROM;
	}
#else
	if ( flags & SDL_INIT_CDROM ) {
		SDL_SetError("SDL not built with cdrom support");
		return(-1);
	}
#endif
	return(0);
}

int SDL_Init(Uint32 flags)
{
#if !SDL_THREADS_DISABLED && SDL_THREAD_PTH
	if (!pth_init()) {
		return -1;
	}
#endif

	/* Clear the error message */
	SDL_ClearError();

	/* Initialize the desired subsystems */
	if ( SDL_InitSubSystem(flags) < 0 ) {
		return(-1);
	}

	/* Everything is initialized */
	if ( !(flags & SDL_INIT_NOPARACHUTE) ) {
		SDL_InstallParachute();
	}
	return(0);
}

void SDL_QuitSubSystem(Uint32 flags)
{
	/* Shut down requested initialized subsystems */
#if !SDL_CDROM_DISABLED
	if ( (flags & SDL_initialized & SDL_INIT_CDROM) ) {
		SDL_CDROMQuit();
		SDL_initialized &= ~SDL_INIT_CDROM;
	}
#endif
#if !SDL_JOYSTICK_DISABLED
	if ( (flags & SDL_initialized & SDL_INIT_JOYSTICK) ) {
		SDL_JoystickQuit();
		SDL_initialized &= ~SDL_INIT_JOYSTICK;
	}
#endif
#if !SDL_AUDIO_DISABLED
	if ( (flags & SDL_initialized & SDL_INIT_AUDIO) ) {
		SDL_AudioQuit();
		SDL_initialized &= ~SDL_INIT_AUDIO;
	}
#endif
#if !SDL_VIDEO_DISABLED
	if ( (flags & SDL_initialized & SDL_INIT_VIDEO) ) {
		SDL_VideoQuit();
		SDL_initialized &= ~SDL_INIT_VIDEO;
	}
#endif
#if !SDL_TIMERS_DISABLED
	if ( (flags & SDL_initialized & SDL_INIT_TIMER) ) {
		SDL_TimerQuit();
		SDL_initialized &= ~SDL_INIT_TIMER;
	}
#endif
}

Uint32 SDL_WasInit(Uint32 flags)
{
	if ( ! flags ) {
		flags = SDL_INIT_EVERYTHING;
	}
	return (SDL_initialized&flags);
}

void SDL_Quit(void)
{
	/* Quit all subsystems */
#ifdef DEBUG_BUILD
  printf("[SDL_Quit] : Enter! Calling QuitSubSystem()\n"); fflush(stdout);
#endif
	SDL_QuitSubSystem(SDL_INIT_EVERYTHING);

#ifdef CHECK_LEAKS
#ifdef DEBUG_BUILD
  printf("[SDL_Quit] : CHECK_LEAKS\n"); fflush(stdout);
#endif

	/* Print the number of surfaces not freed */
	if ( surfaces_allocated != 0 ) {
		fprintf(stderr, "SDL Warning: %d SDL surfaces extant\n", 
							surfaces_allocated);
	}
#endif
#ifdef DEBUG_BUILD
  printf("[SDL_Quit] : SDL_UninstallParachute()\n"); fflush(stdout);
#endif

	/* Uninstall any parachute signal handlers */
	SDL_UninstallParachute();

#if !SDL_THREADS_DISABLED && SDL_THREAD_PTH
	pth_kill();
#endif
#ifdef DEBUG_BUILD
  printf("[SDL_Quit] : Returning!\n"); fflush(stdout);
#endif

}

/* Return the library version number */
const SDL_version * SDL_Linked_Version(void)
{
	return(&version);
}

#if defined(__OS2__)
/* Building for OS/2 */
#ifdef __WATCOMC__

#define INCL_DOSERRORS
#define INCL_DOSEXCEPTIONS
#include <os2.h>

/* Exception handler to prevent the Audio thread hanging, making a zombie process! */
ULONG _System SDL_Main_ExceptionHandler(PEXCEPTIONREPORTRECORD pERepRec,
                                        PEXCEPTIONREGISTRATIONRECORD pERegRec,
                                        PCONTEXTRECORD pCtxRec,
                                        PVOID p)
{
  if (pERepRec->fHandlerFlags & EH_EXIT_UNWIND)
    return XCPT_CONTINUE_SEARCH;
  if (pERepRec->fHandlerFlags & EH_UNWINDING)
    return XCPT_CONTINUE_SEARCH;
  if (pERepRec->fHandlerFlags & EH_NESTED_CALL)
    return XCPT_CONTINUE_SEARCH;

  /* Do cleanup at every fatal exception! */
  if (((pERepRec->ExceptionNum & XCPT_SEVERITY_CODE) == XCPT_FATAL_EXCEPTION) &&
      (pERepRec->ExceptionNum != XCPT_BREAKPOINT) &&
      (pERepRec->ExceptionNum != XCPT_SINGLE_STEP)
     )
  {
    if (SDL_initialized & SDL_INIT_AUDIO)
    {
      /* This removes the zombie audio thread in case of emergency. */
#ifdef DEBUG_BUILD
      printf("[SDL_Main_ExceptionHandler] : Calling SDL_CloseAudio()!\n");
#endif
      SDL_CloseAudio();
    }
  }
  return (XCPT_CONTINUE_SEARCH);
}


EXCEPTIONREGISTRATIONRECORD SDL_Main_xcpthand = {0, SDL_Main_ExceptionHandler};

/* The main DLL entry for DLL Initialization and Uninitialization: */
unsigned _System LibMain(unsigned hmod, unsigned termination)
{
  if (termination)
  {
#ifdef DEBUG_BUILD
/*    printf("[SDL DLL Unintialization] : Removing exception handler\n"); */
#endif
    DosUnsetExceptionHandler(&SDL_Main_xcpthand);
    return 1;
  } else
  {
#ifdef DEBUG_BUILD
    /* Make stdout and stderr unbuffered! */
    setbuf(stdout, NULL);
    setbuf(stderr, NULL);
#endif
    /* Fire up exception handler */
#ifdef DEBUG_BUILD
/*    printf("[SDL DLL Initialization] : Setting exception handler\n"); */
#endif
    /* Set exception handler */
    DosSetExceptionHandler(&SDL_Main_xcpthand);

    return 1;
  }
}
#endif /* __WATCOMC__ */

#elif defined(__WIN32__)  && !defined(__SYMBIAN32__)

#if !defined(HAVE_LIBC) || (defined(__WATCOMC__) && defined(BUILD_DLL))
/* Need to include DllMain() on Watcom C for some reason.. */
#define WIN32_LEAN_AND_MEAN
#include <windows.h>

BOOL APIENTRY _DllMainCRTStartup( HANDLE hModule, 
                       DWORD  ul_reason_for_call, 
                       LPVOID lpReserved )
{
	switch (ul_reason_for_call) {
		case DLL_PROCESS_ATTACH:
		case DLL_THREAD_ATTACH:
		case DLL_THREAD_DETACH:
		case DLL_PROCESS_DETACH:
			break;
	}
	return TRUE;
}
#endif /* building DLL with Watcom C */

#endif /* OS/2 elif __WIN32__ */
