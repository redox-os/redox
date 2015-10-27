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
    Note: This file hasn't been modified so technically we have to keep the disclaimer :-(


    Copyright:  © Copyright 2002 Apple Computer, Inc. All rights reserved.

    Disclaimer: IMPORTANT:  This Apple software is supplied to you by Apple Computer, Inc.
            ("Apple") in consideration of your agreement to the following terms, and your
            use, installation, modification or redistribution of this Apple software
            constitutes acceptance of these terms.  If you do not agree with these terms,
            please do not use, install, modify or redistribute this Apple software.

            In consideration of your agreement to abide by the following terms, and subject
            to these terms, Apple grants you a personal, non-exclusive license, under Apple’s
            copyrights in this original Apple software (the "Apple Software"), to use,
            reproduce, modify and redistribute the Apple Software, with or without
            modifications, in source and/or binary forms; provided that if you redistribute
            the Apple Software in its entirety and without modifications, you must retain
            this notice and the following text and disclaimers in all such redistributions of
            the Apple Software.  Neither the name, trademarks, service marks or logos of
            Apple Computer, Inc. may be used to endorse or promote products derived from the
            Apple Software without specific prior written permission from Apple.  Except as
            expressly stated in this notice, no other rights or licenses, express or implied,
            are granted by Apple herein, including but not limited to any patent rights that
            may be infringed by your derivative works or by other works in which the Apple
            Software may be incorporated.

            The Apple Software is provided by Apple on an "AS IS" basis.  APPLE MAKES NO
            WARRANTIES, EXPRESS OR IMPLIED, INCLUDING WITHOUT LIMITATION THE IMPLIED
            WARRANTIES OF NON-INFRINGEMENT, MERCHANTABILITY AND FITNESS FOR A PARTICULAR
            PURPOSE, REGARDING THE APPLE SOFTWARE OR ITS USE AND OPERATION ALONE OR IN
            COMBINATION WITH YOUR PRODUCTS.

            IN NO EVENT SHALL APPLE BE LIABLE FOR ANY SPECIAL, INDIRECT, INCIDENTAL OR
            CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE
            GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
            ARISING IN ANY WAY OUT OF THE USE, REPRODUCTION, MODIFICATION AND/OR DISTRIBUTION
            OF THE APPLE SOFTWARE, HOWEVER CAUSED AND WHETHER UNDER THEORY OF CONTRACT, TORT
            (INCLUDING NEGLIGENCE), STRICT LIABILITY OR OTHERWISE, EVEN IF APPLE HAS BEEN
            ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
*/
/*=============================================================================
    CAGuard.h

=============================================================================*/
#if !defined(__CAGuard_h__)
#define __CAGuard_h__

/*=============================================================================
    Includes
  =============================================================================*/

#include <CoreAudio/CoreAudioTypes.h>
#include <pthread.h>


/*=============================================================================
    CAGuard
  
    This is your typical mutex with signalling implemented via pthreads.
    Lock() will return true if and only if the guard is locked on that call.
    A thread that already has the guard will receive 'false' if it locks it
    again. Use of the stack-based CAGuard::Locker class is highly recommended
    to properly manage the recursive nesting. The Wait calls with timeouts
    will return true if and only if the timeout period expired. They will
    return false if they receive notification any other way.
  =============================================================================*/

typedef struct S_SDLOSXCAGuard
{

/*  Construction/Destruction */
/*public:*/
/*  Actions */
/*public:*/
    int     (*Lock)(struct S_SDLOSXCAGuard *cag);
    void    (*Unlock)(struct S_SDLOSXCAGuard *cag);
    int     (*Try)(struct S_SDLOSXCAGuard *cag, int *outWasLocked);    /* returns true if lock is free, false if not */
    void    (*Wait)(struct S_SDLOSXCAGuard *cag);
    void    (*Notify)(struct S_SDLOSXCAGuard *cag);

/*  Implementation */
/*protected:*/
    pthread_mutex_t mMutex;
    pthread_cond_t  mCondVar;
    pthread_t       mOwner;
} SDLOSXCAGuard;

SDLOSXCAGuard *new_SDLOSXCAGuard(void);
void delete_SDLOSXCAGuard(SDLOSXCAGuard *cag);

#endif

