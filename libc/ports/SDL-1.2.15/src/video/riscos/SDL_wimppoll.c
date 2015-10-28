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
#include "SDL_config.h"

/*
     File added by Alan Buckley (alan_baa@hotmail.com) for RISC OS compatability
	 27 March 2003

     Implements Pumping of events and WIMP polling
*/

#include "SDL.h"
#include "SDL_syswm.h"
#include "../../events/SDL_sysevents.h"
#include "../../events/SDL_events_c.h"
#include "SDL_riscosvideo.h"
#include "SDL_riscosevents_c.h"
#include "SDL_riscosmouse_c.h"
#include "../../timer/SDL_timer_c.h"

#include "memory.h"
#include "stdlib.h"
#include "ctype.h"

#include "kernel.h"
#include "swis.h"
#include "unixlib/os.h"

#if !SDL_THREADS_DISABLED
#include <pthread.h>
#endif

/* Local functions */
void WIMP_Poll(_THIS, int waitTime);
void WIMP_SetFocus(int win);

/* SDL_riscossprite functions */
void WIMP_PlotSprite(_THIS, int x, int y);
void WIMP_ModeChanged(_THIS);
void WIMP_PaletteChanged(_THIS);


extern void WIMP_PollMouse(_THIS);
extern void RISCOS_PollKeyboard();

#if SDL_THREADS_DISABLED
/* Timer running function */
extern void RISCOS_CheckTimer();
#else
extern int riscos_using_threads;
#endif

/* Mouse cursor handling */
extern void WIMP_ReshowCursor(_THIS);
extern void WIMP_RestoreWimpCursor();

int hasFocus = 0;
int mouseInWindow = 0;
 
/* Flag to ensure window is correct size after a mode change */
static int resizeOnOpen = 0;

void WIMP_PumpEvents(_THIS)
{
	WIMP_Poll(this, 0);
	if (hasFocus) RISCOS_PollKeyboard();
	if (mouseInWindow) WIMP_PollMouse(this);
#if SDL_THREADS_DISABLED
	if (SDL_timer_running) RISCOS_CheckTimer();
#endif
}


void WIMP_Poll(_THIS, int waitTime)
{
	_kernel_swi_regs regs;
	int message[64];
	unsigned int code;
	int pollMask = 0;
	int doPoll = 1;
	int sysEvent;
	int sdlWindow = this->hidden->window_handle;

    if (this->PumpEvents != WIMP_PumpEvents) return;

    if (waitTime > 0)
    {
		_kernel_swi(OS_ReadMonotonicTime, &regs, &regs);
		waitTime += regs.r[0];
    }

    while (doPoll)
    {
#if !SDL_THREADS_DISABLED
       /* Stop thread callbacks while program is paged out */
       if (riscos_using_threads) __pthread_stop_ticker();
#endif

        if (waitTime <= 0)
        {
        	regs.r[0] = pollMask; /* Poll Mask */
        	/* For no wait time mask out null event so we wait until something happens */
        	if (waitTime < 0) regs.r[0] |= 1;
        	regs.r[1] = (int)message;
        	_kernel_swi(Wimp_Poll, &regs, &regs);
        } else
        {
        	regs.r[0] = pollMask;
        	regs.r[1] = (int)message;
        	regs.r[2] = waitTime;
        	_kernel_swi(Wimp_PollIdle, &regs, &regs);
        }

		/* Flag to specify if we post a SDL_SysWMEvent */
	sysEvent = 0;
        
        code = (unsigned int)regs.r[0];

	switch(code)
	{
        case 0:  /* Null Event - drop out for standard processing*/
	   doPoll = 0;
	   break;

	case 1:     /* Redraw window */
           _kernel_swi(Wimp_RedrawWindow, &regs,&regs);
	   if (message[0] == sdlWindow)
	   {
                 while (regs.r[0])
                 {
           	    WIMP_PlotSprite(this, message[1], message[2]);
           	    _kernel_swi(Wimp_GetRectangle, &regs, &regs);
                 }
	   } else
	  {
	/* TODO: Currently we just eat them - we may need to pass them on */
        	while (regs.r[0])
        	{
                        _kernel_swi(Wimp_GetRectangle, &regs, &regs);
        	}
	  }
          break;
        	
		case 2:		/* Open window */
		   if ( resizeOnOpen && message[0] == sdlWindow)
		   {
		      /* Ensure window is correct size */
		      resizeOnOpen = 0;
		      message[3] = message[1] + (this->screen->w << this->hidden->xeig);
		      message[4] = message[2] + (this->screen->h << this->hidden->yeig);       
		   }
        	_kernel_swi(Wimp_OpenWindow, &regs, &regs);
       	    break;
        	
		case 3:		/* Close window */
			if (message[0] == sdlWindow)
			{
				/* Documentation makes it looks as if the following line is correct:
				**    if (SDL_PrivateQuit() == 1) _kernel_swi(Wimp_CloseWindow, &regs, &regs);
				** However some programs don't process this message and so sit there invisibly
				** in the background so I just post the quit message and hope the application
				** does the correct thing.
				*/
				SDL_PrivateQuit();
			} else
				sysEvent = 1;
        	doPoll = 0;
        	break;

		case 4: /* Pointer_Leaving_Window */
			if (message[0] == sdlWindow)
			{
				mouseInWindow = 0;
				//TODO: Lose buttons / dragging
				 /* Reset to default pointer */
				 WIMP_RestoreWimpCursor();
				 SDL_PrivateAppActive(0, SDL_APPMOUSEFOCUS);
			} else
				sysEvent = 1;
			break;

		case 5: /* Pointer_Entering_Window */
			if (message[0] == sdlWindow) 
			{
				mouseInWindow = 1;
				WIMP_ReshowCursor(this);
				SDL_PrivateAppActive(1, SDL_APPMOUSEFOCUS);
			} else sysEvent = 1;
			break;

		case 6:		/* Mouse_Click */
			if (hasFocus == 0)
			{
			   /* First click gives focus if it's not a menu */
			   /* we only count non-menu clicks on a window that has the focus */
			   WIMP_SetFocus(message[3]);
			} else
				doPoll = 0; // So PollMouse gets a chance to pick it up
		   break;

		case 7: /* User_Drag_Box - Used for mouse release */
			//TODO: May need to implement this in the future
			sysEvent = 1;
			break;

		case 8: /* Keypressed */
			doPoll = 0; /* PollKeyboard should pick it up */
			if (message[0] != sdlWindow) sysEvent = 1;
			/*TODO: May want to always pass F12 etc to the wimp
			{
				regs.r[0] = message[6];
				_kernel_swi(Wimp_ProcessKey, &regs, &regs);
			}
			*/
			break;

		case 11: /* Lose Caret */
			 hasFocus = 0;
			 if (message[0] == sdlWindow) SDL_PrivateAppActive(0, SDL_APPINPUTFOCUS);
			 else sysEvent = 1;
			 break;

		case 12: /* Gain Caret */
			 hasFocus = 1;
			 if (message[0] == sdlWindow) SDL_PrivateAppActive(1, SDL_APPINPUTFOCUS);
			 else sysEvent = 1;
			 break;
        	
		case 17:
		case 18:
			sysEvent = 1; /* All messages are passed on */

			switch(message[4])
			{
			case 0: /* Quit Event */
				/* No choice - have to quit */
			   SDL_Quit();
        	   exit(0);
			   break;

			case 8: /* Pre Quit */
				SDL_PrivateQuit();
				break;

			case 0x400c1: /* Mode change */
				WIMP_ModeChanged(this);
				resizeOnOpen = 1;
				break;

			case 9:      /* Palette changed */
				WIMP_PaletteChanged(this);
				break;
			}
			break;

		default:
			/* Pass unknown events on */
			sysEvent = 1;
			break;
		}

		if (sysEvent)
		{
	        SDL_SysWMmsg wmmsg;

			SDL_VERSION(&wmmsg.version);
			wmmsg.eventCode = code;
			SDL_memcpy(wmmsg.pollBlock, message, 64 * sizeof(int));

			/* Fall out of polling loop if message is successfully posted */
			if (SDL_PrivateSysWMEvent(&wmmsg)) doPoll = 0;
		}
#if !SDL_THREADS_DISABLED
		if (riscos_using_threads)
		{
                   /* Restart ticker here so other thread can not interfere
                      with the Redraw processing */
		   if (riscos_using_threads) __pthread_start_ticker();
                   /* Give other threads a better chance of running */
		   pthread_yield();
		}
#endif
    }
}

/* Set focus to specified window */
void WIMP_SetFocus(int win)
{
	_kernel_swi_regs regs;

	regs.r[0] = win;
	regs.r[1] = -1; /* Icon handle */
	regs.r[2] = 0;  /* X-offset we just put it at position 0 */
	regs.r[3] = 0;  /* Y-offset as above */
	regs.r[4] = 1 << 25; /* Caret is invisible */
	regs.r[5] = 0;  /* index into string */

	_kernel_swi(Wimp_SetCaretPosition, &regs, &regs);
}

/** Run background task while in a sleep command */
void RISCOS_BackgroundTasks(void)
{
	if (current_video && current_video->hidden->window_handle)
	{
		WIMP_Poll(current_video, 0);
	}
#if SDL_THREADS_DISABLED
	if (SDL_timer_running) RISCOS_CheckTimer();
#endif
}
