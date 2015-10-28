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
    slouken@devolution.com
*/

/*
    SDL_sysmutex.cpp

    Epoc version by Markus Mertama (w@iki.fi)
*/


#ifdef SAVE_RCSID
static char rcsid =
 "@(#) $Id: SDL_sysmutex.c,v 1.1.2.3 2000/06/22 15:25:23 hercules Exp $";
#endif

/* Mutex functions using the Win32 API */

//#include <stdio.h>
//#include <stdlib.h>

#include <e32std.h>

#include "epoc_sdl.h"

#include "SDL_error.h"
#include "SDL_mutex.h"


#ifdef EKA2 //???
struct SDL_mutex
    {
    TInt handle;
    };
#else
struct _SDL_mutex
    {
    TInt handle;
    };
#endif

extern TInt CreateUnique(TInt (*aFunc)(const TDesC& aName, TAny*, TAny*), TAny*, TAny*);

TInt NewMutex(const TDesC& aName, TAny* aPtr1, TAny*)
    {
    return ((RMutex*)aPtr1)->CreateGlobal(aName);
    }
    
void DeleteMutex(TAny* aMutex)
    {
    SDL_DestroyMutex ((SDL_mutex*) aMutex);
    }

/* Create a mutex */
SDL_mutex *SDL_CreateMutex(void)
{
    RMutex rmutex;

    TInt status = CreateUnique(NewMutex, &rmutex, NULL);
	if(status != KErrNone)
	    {
			SDL_SetError("Couldn't create mutex");
		}
    SDL_mutex* mutex = new /*(ELeave)*/ SDL_mutex;
    mutex->handle = rmutex.Handle();
    EpocSdlEnv::AppendCleanupItem(TSdlCleanupItem(DeleteMutex, mutex));
	return(mutex);
}

/* Free the mutex */
void SDL_DestroyMutex(SDL_mutex *mutex)
{
	if ( mutex ) 
	{
    RMutex rmutex;
    rmutex.SetHandle(mutex->handle);
    if(rmutex.IsHeld())
        {
	    rmutex.Signal();
        }
	rmutex.Close();
	EpocSdlEnv::RemoveCleanupItem(mutex);
	delete(mutex);
    mutex = NULL;
	}
}

/* Lock the mutex */
int SDL_mutexP(SDL_mutex *mutex)
{
	if ( mutex == NULL ) {
		SDL_SetError("Passed a NULL mutex");
		return -1;
	}
    RMutex rmutex;
    rmutex.SetHandle(mutex->handle);
	rmutex.Wait(); 
	return(0);
}

/* Unlock the mutex */
int SDL_mutexV(SDL_mutex *mutex)
{
	if ( mutex == NULL ) {
		SDL_SetError("Passed a NULL mutex");
		return -1;
	}
	RMutex rmutex;
    rmutex.SetHandle(mutex->handle);
	rmutex.Signal();
	return(0);
}
