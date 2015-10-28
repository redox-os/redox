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

#ifndef __CDPlayer__H__
#define __CDPlayer__H__ 1

#include <string.h>

#include <Carbon/Carbon.h>
#include <CoreFoundation/CoreFoundation.h>
#include <AudioUnit/AudioUnit.h>

#include "SDL.h"
#include "SDL_thread.h"
#include "SDL_mutex.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef void (*CDPlayerCompletionProc)(SDL_CD *cdrom) ;

void     Lock ();

void     Unlock();

int      LoadFile (const FSRef *ref, int startFrame, int endFrame); /* pass -1 to do nothing */

int      ReleaseFile ();

int      PlayFile  ();

int      PauseFile ();

void     SetCompletionProc (CDPlayerCompletionProc proc, SDL_CD *cdrom);

int      ReadTOCData (FSVolumeRefNum theVolume, SDL_CD *theCD);

int      ListTrackFiles (FSVolumeRefNum theVolume, FSRef *trackFiles, int numTracks);

int      DetectAudioCDVolumes (FSVolumeRefNum *volumes, int numVolumes);

int      GetCurrentFrame ();

#ifdef __cplusplus
};
#endif

#endif /* __CD_Player__H__ */
