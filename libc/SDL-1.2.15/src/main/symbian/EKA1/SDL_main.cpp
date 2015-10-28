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
    SDL_main.cpp
    The Epoc executable startup functions 

    Epoc version by Hannu Viitala (hannu.j.viitala@mbnet.fi)
*/

#include <e32std.h>
#include <e32def.h>
#include <e32svr.h>
#include <e32base.h>
#include <estlib.h>
#include <stdlib.h>
#include <stdio.h>
#include <w32std.h>
#include <apgtask.h>

#include "SDL_error.h"

#if defined(__WINS__)
#include <estw32.h>
IMPORT_C void RegisterWsExe(const TDesC &aName);
#endif

/* The prototype for the application's main() function */
#define main	SDL_main
extern "C" int main (int argc, char *argv[], char *envp[]);
extern "C" void exit (int ret);


/* Epoc main function */

#ifdef __WINS__


void GetCmdLine(int& aArgc, char**& aArgv)
    {
    RChunk chunk;

    if(chunk.OpenGlobal(RThread().Name(), ETrue) != KErrNone)
        return;

    TUint* ptr = (TUint*) chunk.Base();
    if(ptr != NULL)
        {
        aArgc = (int)    *(ptr); // count
        aArgv = (char**) *(ptr + 1);
        }
    chunk.Close();
    }

#endif


TInt E32Main()
    {
    /*  Get the clean-up stack */
	CTrapCleanup* cleanup = CTrapCleanup::New();

    /* Arrange for multi-threaded operation */
	SpawnPosixServerThread();	

    /* Get args and environment */
	int argc=0;
	char** argv=0;
	char** envp=0;

#ifndef __WINS__
	__crt0(argc,argv,envp);	
#else
    GetCmdLine(argc, argv);
#endif
    /* Start the application! */

    /* Create stdlib */
    _REENT;

    /* Set process and thread priority and name */

    RThread currentThread;
	RProcess thisProcess;
	TParse exeName;
	exeName.Set(thisProcess.FileName(), NULL, NULL);
    currentThread.Rename(exeName.Name());
    currentThread.SetProcessPriority(EPriorityLow);
    currentThread.SetPriority(EPriorityMuchLess);

     /* Call stdlib main */
    int ret = main(argc, argv, envp); /* !! process exits here if there is "exit()" in main! */	
    
    /* Call exit */
    //exit(ret); /* !! process exits here! */	
    //Markus: I do not understand above
    //I commented it at let this function
    //to return ret value - was it purpose
    //that cleanup below is not called at all - why?

    /* Free resources and return */
    
    _cleanup(); //this is normally called at exit, I call it here, Markus 

    CloseSTDLIB();
   	delete cleanup;	
#ifdef __WINS__
//    User::Panic(_L("exit"), ret);
  //  RThread().Kill(ret); //Markus  get rid of this thread
  //  RThread().RaiseException(EExcKill);
#endif
    return ret;//Markus, or exit(ret); ??
  //return(KErrNone);
    }


#ifdef __WINS__
EXPORT_C TInt WinsMain()
    {
    return E32Main();
 //   return WinsMain(0, 0, 0);
    }
#endif

/* Epoc dll entry point */
#if defined(__WINS__)
GLDEF_C TInt E32Dll(TDllReason)
	{
	return(KErrNone);
	}
#endif


