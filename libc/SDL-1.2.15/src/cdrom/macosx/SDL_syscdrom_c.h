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

/* This is the Mac OS X / CoreAudio specific header for the SDL CD-ROM API
   Contributed by Darrell Walisser and Max Horn
 */

/***********************************************************************************
 Implementation Notes
 *********************

    This code has several limitations currently (all of which are proabaly fixable):
    
    1. A CD-ROM device is inferred from a mounted cdfs volume, so device 0 is
       not necessarily the first CD-ROM device on the system. (Somewhat easy to fix
       by useing the device name from the volume id's to reorder the volumes)
       
    2. You can only open and control 1 CD-ROM device at a time. (Challenging to fix,
       due to extensive code restructuring)
    
    3. The status reported by SDL_CDStatus only changes to from CD_PLAYING to CD_STOPPED in
       1-second intervals (because the audio is buffered in 1-second chunks) If
       the audio data is less than 1 second, the remainder is filled with silence.
       
       If you need to play sequences back-to-back that are less that 1 second long,
       use the frame position to determine when to play the next sequence, instead
       of SDL_CDStatus.
       
       This may be possible to fix with a clever usage of the AudioUnit API.
       
    4. When new volumes are inserted, our volume information is not updated. The only way
       to refresh this information is to reinit the CD-ROM subsystem of SDL. To fix this,
       one would probably have to fix point 1 above first, then figure out how to register
       for a notification when new media is mounted in order to perform an automatic
       rescan for cdfs volumes.
    
    
    
    So, here comes a description of how this all works.
    
        < Initializing >
        
        To get things rolling, we have to locate mounted volumes that contain
        audio (since nearly all Macs don't have analog audio-in on the sound card).
        That's easy, since these volumes have a flag that indicates this special
        filesystem. See DetectAudioCDVolumes() in CDPlayer.cpp for this code.
        
        Next, we parse the invisible .TOC.plist in the root of the volume, which gets us
        the track information (number, offset, length, leadout, etc). See ReadTOCData() in
        CDPlayer.cpp for the skinny on this.
        
        
        < The Playback Loop >
        
        Now come the tricky parts. Let's start with basic audio playback. When a frame
        range to play is requested, we must first find the .aiff files on the volume, 
        hopefully in the right order. Since these files all begin with a number "1 Audio Track", 
        etc, this is used to determine the correct track order.
        
        Once all files are determined, we have to find what file corresponds to the start
        and length parameter to SDL_SYS_CDPlay(). Again, this is quite simple by walking the
        cdrom's track list. At this point, we also save the offset to the next track and frames
        remaining, if we're going to have to play another file after the first one. See
        GetFileForOffset() for this code.
        
        At this point we have all info needed to start playback, so we hand off to the LoadFile()
        function, which proceeds to do its magic and plays back the file.
        
        When the file is finished playing, CompletionProc() is invoked, at which time we can
        play the next file if the previously saved next track and frames remaining
        indicates that we should. 
        
        
        < Magic >
        
        OK, so it's not really magic, but since I don't fully understand all the hidden details it
        seems like it to me ;-) The API's involved are the AudioUnit and AudioFile API's. These
        appear to be an extension of CoreAudio for creating modular playback and f/x entities.
        The important thing is that CPU usage is very low and reliability is very high. You'd
        be hard-pressed to find a way to stutter the playback with other CPU-intensive tasks.
    
        One part of this magic is that it uses multiple threads, which carries the usual potential
        for disaster if not handled carefully. Playback currently requires 4 additional threads:
            1. The coreaudio runloop thread
            2. The coreaudio device i/o thread
            3. The file streaming thread
            4. The notification/callback thread
        
        The first 2 threads are necessary evil - CoreAudio creates this no matter what the situation
        is (even the SDL sound implementation creates theses suckers). The last two are are created
        by us.
        
        The file is streamed from disk using a threaded double-buffer approach. 
        This way, the high latency operation of reading from disk can be performed without interrupting
        the real-time device thread (which amounts to avoiding dropouts). The device thread grabs the
        buffer that isn't being read and sends it to the CoreAudio mixer where it eventually gets 
        to the sound card.
        
        The device thread posts a notification when the file streaming thread is out of data. This
        notification must be handled in a separate thread to avoid potential deadlock in the
        device thread. That's where the notification thread comes in. This thread is signaled
        whenever a notification needs to be processed, so another file can be played back if need be.
        
        The API in CDPlayer.cpp contains synchronization because otherwise both the notification thread
        and main thread (or another other thread using the SDL CD api) can potentially call it at the same time.
    
************************************************************************************/


#include "SDL_cdrom.h"
#include "../SDL_syscdrom.h"

#include "CDPlayer.h"

#define kErrorFakeDevice "Error: Cannot proceed since we're faking a CD-ROM device. Reinit the CD-ROM subsystem to scan for new volumes."

