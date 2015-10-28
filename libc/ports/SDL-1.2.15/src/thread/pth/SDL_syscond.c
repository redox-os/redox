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

/*
 *	GNU pth conditions variables
 *
 *	Patrice Mandin
 */

#include <pth.h>

#include "SDL_thread.h"
#include "SDL_sysmutex_c.h"

struct SDL_cond
{
	pth_cond_t	condpth_p;
};

/* Create a condition variable */
SDL_cond * SDL_CreateCond(void)
{
	SDL_cond *cond;

	cond = (SDL_cond *) SDL_malloc(sizeof(SDL_cond));
	if ( cond ) {
		if ( pth_cond_init(&(cond->condpth_p)) < 0 ) {
			SDL_SetError("pthread_cond_init() failed");
			SDL_free(cond);
			cond = NULL;
		}
	} else {
		SDL_OutOfMemory();
	}
	return(cond);
}

/* Destroy a condition variable */
void SDL_DestroyCond(SDL_cond *cond)
{
	if ( cond ) {
		SDL_free(cond);
	}
}

/* Restart one of the threads that are waiting on the condition variable */
int SDL_CondSignal(SDL_cond *cond)
{
	int retval;

	if ( ! cond ) {
		SDL_SetError("Passed a NULL condition variable");
		return -1;
	}

	retval = 0;
	if ( pth_cond_notify(&(cond->condpth_p), FALSE) != 0 ) {
		SDL_SetError("pth_cond_notify() failed");
		retval = -1;
	}
	return retval;
}

/* Restart all threads that are waiting on the condition variable */
int SDL_CondBroadcast(SDL_cond *cond)
{
	int retval;

	if ( ! cond ) {
		SDL_SetError("Passed a NULL condition variable");
		return -1;
	}

	retval = 0;
	if ( pth_cond_notify(&(cond->condpth_p), TRUE) != 0 ) {
		SDL_SetError("pth_cond_notify() failed");
		retval = -1;
	}
	return retval;
}

/* Wait on the condition variable for at most 'ms' milliseconds.
   The mutex must be locked before entering this function!
   The mutex is unlocked during the wait, and locked again after the wait.

Typical use:

Thread A:
	SDL_LockMutex(lock);
	while ( ! condition ) {
		SDL_CondWait(cond);
	}
	SDL_UnlockMutex(lock);

Thread B:
	SDL_LockMutex(lock);
	...
	condition = true;
	...
	SDL_UnlockMutex(lock);
 */
int SDL_CondWaitTimeout(SDL_cond *cond, SDL_mutex *mutex, Uint32 ms)
{
	int retval;
	pth_event_t ev;
	int sec;

	if ( ! cond ) {
		SDL_SetError("Passed a NULL condition variable");
		return -1;
	}

	retval = 0;

	sec = ms/1000;
	ev = pth_event(PTH_EVENT_TIME, pth_timeout(sec,(ms-sec*1000)*1000));

	if ( pth_cond_await(&(cond->condpth_p), &(mutex->mutexpth_p), ev) != 0 ) {
		SDL_SetError("pth_cond_await() failed");
		retval = -1;
	}

    pth_event_free(ev, PTH_FREE_ALL);

	return retval;
}

/* Wait on the condition variable forever */
int SDL_CondWait(SDL_cond *cond, SDL_mutex *mutex)
{
	int retval;

	if ( ! cond ) {
		SDL_SetError("Passed a NULL condition variable");
		return -1;
	}

	retval = 0;
	if ( pth_cond_await(&(cond->condpth_p), &(mutex->mutexpth_p), NULL) != 0 ) {
		SDL_SetError("pth_cond_await() failed");
		retval = -1;
	}
	return retval;
}
