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

#include <errno.h>

#include "SDL_config.h"

/* RISC OS semiphores based on linux code */


#include "SDL_timer.h"
#include "SDL_thread.h"
#include "SDL_systhread_c.h"

#if !SDL_THREADS_DISABLED

SDL_sem *SDL_CreateSemaphore(Uint32 initial_value)
{
	SDL_SetError("SDL not configured with thread support");
	return (SDL_sem *)0;
}

void SDL_DestroySemaphore(SDL_sem *sem)
{
	return;
}

int SDL_SemTryWait(SDL_sem *sem)
{
	SDL_SetError("SDL not configured with thread support");
	return -1;
}

int SDL_SemWaitTimeout(SDL_sem *sem, Uint32 timeout)
{
	SDL_SetError("SDL not configured with thread support");
	return -1;
}

int SDL_SemWait(SDL_sem *sem)
{
	SDL_SetError("SDL not configured with thread support");
	return -1;
}

Uint32 SDL_SemValue(SDL_sem *sem)
{
	return 0;
}

int SDL_SemPost(SDL_sem *sem)
{
	SDL_SetError("SDL not configured with thread support");
	return -1;
}

#else


#include <unistd.h>			/* For getpid() */
#include <pthread.h>
#include <semaphore.h>

struct SDL_semaphore {
	sem_t *sem;
	sem_t sem_data;
};

/* Create a semaphore, initialized with value */
SDL_sem *SDL_CreateSemaphore(Uint32 initial_value)
{
	SDL_sem *sem = (SDL_sem *) SDL_malloc(sizeof(SDL_sem));
	if ( sem ) {
		if ( sem_init(&sem->sem_data, 0, initial_value) < 0 ) {
			SDL_SetError("sem_init() failed");
			SDL_free(sem);
			sem = NULL;
		} else {
			sem->sem = &sem->sem_data;
		}
	} else {
		SDL_OutOfMemory();
	}
	return sem;
}

void SDL_DestroySemaphore(SDL_sem *sem)
{
	if ( sem ) {
		sem_destroy(sem->sem);
		SDL_free(sem);
	}
}

int SDL_SemTryWait(SDL_sem *sem)
{
	int retval;

	if ( ! sem ) {
		SDL_SetError("Passed a NULL semaphore");
		return -1;
	}
	retval = SDL_MUTEX_TIMEDOUT;
	if ( sem_trywait(sem->sem) == 0 ) {
		retval = 0;
	}
	return retval;
}

int SDL_SemWait(SDL_sem *sem)
{
	int retval;

	if ( ! sem ) {
		SDL_SetError("Passed a NULL semaphore");
		return -1;
	}

	while ( ((retval = sem_wait(sem->sem)) == -1) && (errno == EINTR) ) {}
	if ( retval < 0 ) {
		SDL_SetError("sem_wait() failed");
	}
	return retval;
}

int SDL_SemWaitTimeout(SDL_sem *sem, Uint32 timeout)
{
	int retval;

	if ( ! sem ) {
		SDL_SetError("Passed a NULL semaphore");
		return -1;
	}

	/* Try the easy cases first */
	if ( timeout == 0 ) {
		return SDL_SemTryWait(sem);
	}
	if ( timeout == SDL_MUTEX_MAXWAIT ) {
		return SDL_SemWait(sem);
	}

	/* Ack!  We have to busy wait... */
	timeout += SDL_GetTicks();
	do {
		retval = SDL_SemTryWait(sem);
		if ( retval == 0 ) {
			break;
		}
		SDL_Delay(1);
	} while ( SDL_GetTicks() < timeout );

	return retval;
}

Uint32 SDL_SemValue(SDL_sem *sem)
{
	int ret = 0;
	if ( sem ) {
		sem_getvalue(sem->sem, &ret);
		if ( ret < 0 ) {
			ret = 0;
		}
	}
	return (Uint32)ret;
}

int SDL_SemPost(SDL_sem *sem)
{
	int retval;

	if ( ! sem ) {
		SDL_SetError("Passed a NULL semaphore");
		return -1;
	}

	retval = sem_post(sem->sem);
	if ( retval < 0 ) {
		SDL_SetError("sem_post() failed");
	}
	return retval;
}

#endif /* !SDL_THREADS_DISABLED */
