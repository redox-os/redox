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

/* RISC OS version based on pthreads linux source */

#include "SDL_thread.h"
#include "../SDL_systhread.h"

#if SDL_THREADS_DISABLED

int SDL_SYS_CreateThread(SDL_Thread *thread, void *args)
{
	SDL_SetError("Threads have not been compiled into this version of the library");
	return(-1);
}

void SDL_SYS_SetupThread(void)
{
	return;
}

Uint32 SDL_ThreadID(void)
{
	return(0);
}

void SDL_SYS_WaitThread(SDL_Thread *thread)
{
	return;
}

void SDL_SYS_KillThread(SDL_Thread *thread)
{
	return;
}

#else

#include <signal.h>

/* List of signals to mask in the subthreads */
static int sig_list[] = {
	SIGHUP, SIGINT, SIGQUIT, SIGPIPE, SIGALRM, SIGTERM, SIGCHLD, SIGWINCH,
	SIGVTALRM, SIGPROF, 0
};

#include <pthread.h>

int riscos_using_threads = 0;
Uint32 riscos_main_thread = 0; /* Thread running events */

static void *RunThread(void *data)
{
	SDL_RunThread(data);
	pthread_exit((void*)0);
	return((void *)0);		/* Prevent compiler warning */
}

int SDL_SYS_CreateThread(SDL_Thread *thread, void *args)
{
	pthread_attr_t type;

	/* Set the thread attributes */
	if ( pthread_attr_init(&type) != 0 ) {
		SDL_SetError("Couldn't initialize pthread attributes");
		return(-1);
	}
	pthread_attr_setdetachstate(&type, PTHREAD_CREATE_JOINABLE);

	/* Create the thread and go! */
	if ( pthread_create(&thread->handle, &type, RunThread, args) != 0 ) {
		SDL_SetError("Not enough resources to create thread");
		return(-1);
	}

        if (riscos_using_threads == 0)
        {
           riscos_using_threads = 1;
           riscos_main_thread = SDL_ThreadID();
        }
      
	return(0);
}

void SDL_SYS_SetupThread(void)
{
	int i;
	sigset_t mask;

	/* Mask asynchronous signals for this thread */
	sigemptyset(&mask);
	for ( i=0; sig_list[i]; ++i ) {
		sigaddset(&mask, sig_list[i]);
	}
	pthread_sigmask(SIG_BLOCK, &mask, 0);
	
#ifdef PTHREAD_CANCEL_ASYNCHRONOUS
	/* Allow ourselves to be asynchronously cancelled */
	{ int oldstate;
		pthread_setcanceltype(PTHREAD_CANCEL_ASYNCHRONOUS, &oldstate);
	}
#endif
}

Uint32 SDL_ThreadID(void)
{
	return((Uint32)pthread_self());
}

void SDL_SYS_WaitThread(SDL_Thread *thread)
{
	pthread_join(thread->handle, 0);
}

void SDL_SYS_KillThread(SDL_Thread *thread)
{
#ifdef PTHREAD_CANCEL_ASYNCHRONOUS
	pthread_cancel(thread->handle);
#else
	pthread_kill(thread->handle, SIGKILL);
#endif
}

#endif
