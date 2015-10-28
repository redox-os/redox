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

/* Semaphore functions using the OS/2 API */

#define INCL_DOS
#define INCL_DOSERRORS
#define INCL_DOSSEMAPHORES
#include <os2.h>

#include "SDL_thread.h"
#include "SDL_timer.h"


struct SDL_semaphore {
        HMTX id;
        HEV  changed;
        Uint32 value;
};


/* Create a semaphore */
DECLSPEC SDL_sem * SDLCALL SDL_CreateSemaphore(Uint32 initial_value)
{
        SDL_sem *sem;
        ULONG ulrc;

        /* Allocate sem memory */
        sem = (SDL_sem *)SDL_malloc(sizeof(*sem));
        if ( sem ) {
                /* Create the mutex semaphore */
                ulrc = DosCreateMutexSem(NULL,&(sem->id),0,TRUE);
                if ( ulrc ) {
                        SDL_SetError("Couldn't create semaphore");
                        SDL_free(sem);
                        sem = NULL;
                } else
                {
                    DosCreateEventSem(NULL, &(sem->changed), 0, FALSE);
                    sem->value = initial_value;
                    DosReleaseMutexSem(sem->id);
                }
        } else {
                SDL_OutOfMemory();
        }
        return(sem);
}

/* Free the semaphore */
DECLSPEC void SDLCALL SDL_DestroySemaphore(SDL_sem *sem)
{
        if ( sem ) {
                if ( sem->id ) {
                        DosCloseEventSem(sem->changed);
                        DosCloseMutexSem(sem->id);
                        sem->id = 0;
                }
                SDL_free(sem);
        }
}

DECLSPEC int SDLCALL SDL_SemWaitTimeout(SDL_sem *sem, Uint32 timeout)
{
        ULONG ulrc;

        if ( ! sem ) {
                SDL_SetError("Passed a NULL sem");
                return -1;
        }

        if ( timeout == SDL_MUTEX_MAXWAIT ) {
           while (1) {
              ulrc = DosRequestMutexSem(sem->id, SEM_INDEFINITE_WAIT);
              if (ulrc) {
                 /* if error waiting mutex */
                 SDL_SetError("DosRequestMutexSem() failed");
                 return -1;
              } else if (sem->value) {
                        sem->value--;
                        DosReleaseMutexSem(sem->id);
                        return 0;
                     } else {
                        ULONG ulPostCount;
                        DosResetEventSem(sem->changed, &ulPostCount);
                        DosReleaseMutexSem(sem->id);
                        /* continue waiting until somebody posts the semaphore */
                        DosWaitEventSem(sem->changed, SEM_INDEFINITE_WAIT);
                     }
           }
        } else
        if ( timeout == 0 )
        {
            ulrc = DosRequestMutexSem(sem->id, SEM_INDEFINITE_WAIT);
            if (ulrc==NO_ERROR)
            {
                if (sem->value)
                {
                    sem->value--;
                    DosReleaseMutexSem(sem->id);
                    return 0;
                } else
                {
                    DosReleaseMutexSem(sem->id);
                    return SDL_MUTEX_TIMEDOUT;
                }
            } else
            {
                SDL_SetError("DosRequestMutexSem() failed");
                return -1;
            }
        } else {
            ulrc = DosRequestMutexSem(sem->id, SEM_INDEFINITE_WAIT);
            if (ulrc) {
               /* if error waiting mutex */
               SDL_SetError("DosRequestMutexSem() failed");
               return -1;
            } else
              if (sem->value) {
                sem->value--;
                DosReleaseMutexSem(sem->id);
                return 0;
              } else {
                ULONG ulPostCount;
                DosResetEventSem(sem->changed, &ulPostCount);
                DosReleaseMutexSem(sem->id);
                /* continue waiting until somebody posts the semaphore */
                ulrc = DosWaitEventSem(sem->changed, timeout);
                if (ulrc==NO_ERROR)
                  return 0;
                else
                  return SDL_MUTEX_TIMEDOUT;
              }
        }
        /* never reached */
        return -1;
}

DECLSPEC int SDLCALL SDL_SemTryWait(SDL_sem *sem)
{
        return SDL_SemWaitTimeout(sem, 0);
}

DECLSPEC int SDLCALL SDL_SemWait(SDL_sem *sem)
{
        return SDL_SemWaitTimeout(sem, SDL_MUTEX_MAXWAIT);
}

/* Returns the current count of the semaphore */
DECLSPEC Uint32 SDLCALL SDL_SemValue(SDL_sem *sem)
{
        if ( ! sem ) {
                SDL_SetError("Passed a NULL sem");
                return 0;
        }
        return sem->value;
}

DECLSPEC int SDLCALL SDL_SemPost(SDL_sem *sem)
{
        if ( ! sem ) {
                SDL_SetError("Passed a NULL sem");
                return -1;
        }
        if ( DosRequestMutexSem(sem->id,SEM_INDEFINITE_WAIT) ) {
                SDL_SetError("DosRequestMutexSem() failed");
                return -1;
        }
        sem->value++;
        DosPostEventSem(sem->changed);
        DosReleaseMutexSem(sem->id);
        return 0;
}
