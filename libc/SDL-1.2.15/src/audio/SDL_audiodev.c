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

/* Get the name of the audio device we use for output */

#if SDL_AUDIO_DRIVER_BSD || SDL_AUDIO_DRIVER_OSS || SDL_AUDIO_DRIVER_SUNAUDIO

#include <fcntl.h>
#include <sys/types.h>
#include <sys/stat.h>

#include "SDL_stdinc.h"
#include "SDL_audiodev_c.h"

#ifndef _PATH_DEV_DSP
#if defined(__NETBSD__) || defined(__OPENBSD__)
#define _PATH_DEV_DSP  "/dev/audio"
#else
#define _PATH_DEV_DSP  "/dev/dsp"
#endif
#endif
#ifndef _PATH_DEV_DSP24
#define _PATH_DEV_DSP24	"/dev/sound/dsp"
#endif
#ifndef _PATH_DEV_AUDIO
#define _PATH_DEV_AUDIO	"/dev/audio"
#endif


int SDL_OpenAudioPath(char *path, int maxlen, int flags, int classic)
{
	const char *audiodev;
	int audio_fd;
	char audiopath[1024];

	/* Figure out what our audio device is */
	if ( ((audiodev=SDL_getenv("SDL_PATH_DSP")) == NULL) &&
	     ((audiodev=SDL_getenv("AUDIODEV")) == NULL) ) {
		if ( classic ) {
			audiodev = _PATH_DEV_AUDIO;
		} else {
			struct stat sb;

			/* Added support for /dev/sound/\* in Linux 2.4 */
			if ( ((stat("/dev/sound", &sb) == 0) && S_ISDIR(sb.st_mode)) &&
				 ((stat(_PATH_DEV_DSP24, &sb) == 0) && S_ISCHR(sb.st_mode)) ) {
				audiodev = _PATH_DEV_DSP24;
			} else {
				audiodev = _PATH_DEV_DSP;
			}
		}
	}
	audio_fd = open(audiodev, flags, 0);

	/* If the first open fails, look for other devices */
	if ( (audio_fd < 0) && (SDL_strlen(audiodev) < (sizeof(audiopath)-3)) ) {
		int exists, instance;
		struct stat sb;

		instance = 1;
		do { /* Don't use errno ENOENT - it may not be thread-safe */
			SDL_snprintf(audiopath, SDL_arraysize(audiopath),
			             "%s%d", audiodev, instance++);
			exists = 0;
			if ( stat(audiopath, &sb) == 0 ) {
				exists = 1;
				audio_fd = open(audiopath, flags, 0); 
			}
		} while ( exists && (audio_fd < 0) );
		audiodev = audiopath;
	}
	if ( path != NULL ) {
		SDL_strlcpy(path, audiodev, maxlen);
		path[maxlen-1] = '\0';
	}
	return(audio_fd);
}

#elif SDL_AUDIO_DRIVER_PAUD

/* Get the name of the audio device we use for output */

#include <sys/types.h>
#include <sys/stat.h>

#include "SDL_stdinc.h"
#include "SDL_audiodev_c.h"

#ifndef _PATH_DEV_DSP
#define _PATH_DEV_DSP	"/dev/%caud%c/%c"
#endif

char devsettings[][3] =
{
    { 'p', '0', '1' }, { 'p', '0', '2' }, { 'p', '0', '3' }, { 'p', '0', '4' },
    { 'p', '1', '1' }, { 'p', '1', '2' }, { 'p', '1', '3' }, { 'p', '1', '4' },
    { 'p', '2', '1' }, { 'p', '2', '2' }, { 'p', '2', '3' }, { 'p', '2', '4' },
    { 'p', '3', '1' }, { 'p', '3', '2' }, { 'p', '3', '3' }, { 'p', '3', '4' },
    { 'b', '0', '1' }, { 'b', '0', '2' }, { 'b', '0', '3' }, { 'b', '0', '4' },
    { 'b', '1', '1' }, { 'b', '1', '2' }, { 'b', '1', '3' }, { 'b', '1', '4' },
    { 'b', '2', '1' }, { 'b', '2', '2' }, { 'b', '2', '3' }, { 'b', '2', '4' },
    { 'b', '3', '1' }, { 'b', '3', '2' }, { 'b', '3', '3' }, { 'b', '3', '4' },
    { '\0', '\0', '\0' }
};

static int OpenUserDefinedDevice(char *path, int maxlen, int flags)
{
	const char *audiodev;
	int  audio_fd;

	/* Figure out what our audio device is */
	if ((audiodev=SDL_getenv("SDL_PATH_DSP")) == NULL) {
	    audiodev=SDL_getenv("AUDIODEV");
	}
	if ( audiodev == NULL ) {
	    return -1;
	}
	audio_fd = open(audiodev, flags, 0);
	if ( path != NULL ) {
		SDL_strlcpy(path, audiodev, maxlen);
		path[maxlen-1] = '\0';
	}
	return audio_fd;
}

int SDL_OpenAudioPath(char *path, int maxlen, int flags, int classic)
{
    struct stat sb;
    int         audio_fd;
    char        audiopath[1024];
    int         cycle;

    audio_fd = OpenUserDefinedDevice(path,maxlen,flags);
    if ( audio_fd != -1 ) {
        return audio_fd;
    }

    cycle    = 0;
    while( devsettings[cycle][0] != '\0' ) {
        SDL_snprintf( audiopath, SDL_arraysize(audiopath),
                 _PATH_DEV_DSP,
                 devsettings[cycle][0],
                 devsettings[cycle][1],
                 devsettings[cycle][2]);

	if ( stat(audiopath, &sb) == 0 ) {
	    audio_fd = open(audiopath, flags, 0);
	    if ( audio_fd > 0 ) {
		if ( path != NULL ) {
		    SDL_strlcpy( path, audiopath, maxlen );
		}
	        return audio_fd;
	    }
	}
    }
    return -1;
}

#endif /* Audio driver selection */
