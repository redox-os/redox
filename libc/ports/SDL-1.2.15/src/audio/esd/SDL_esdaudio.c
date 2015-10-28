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

/* Allow access to an ESD network stream mixing buffer */

#include <sys/types.h>
#include <unistd.h>
#include <signal.h>
#include <errno.h>
#include <esd.h>

#include "SDL_timer.h"
#include "SDL_audio.h"
#include "../SDL_audiomem.h"
#include "../SDL_audio_c.h"
#include "../SDL_audiodev_c.h"
#include "SDL_esdaudio.h"

#ifdef SDL_AUDIO_DRIVER_ESD_DYNAMIC
#include "SDL_name.h"
#include "SDL_loadso.h"
#else
#define SDL_NAME(X)	X
#endif

/* The tag name used by ESD audio */
#define ESD_DRIVER_NAME		"esd"

/* Audio driver functions */
static int ESD_OpenAudio(_THIS, SDL_AudioSpec *spec);
static void ESD_WaitAudio(_THIS);
static void ESD_PlayAudio(_THIS);
static Uint8 *ESD_GetAudioBuf(_THIS);
static void ESD_CloseAudio(_THIS);

#ifdef SDL_AUDIO_DRIVER_ESD_DYNAMIC

static const char *esd_library = SDL_AUDIO_DRIVER_ESD_DYNAMIC;
static void *esd_handle = NULL;
static int esd_loaded = 0;

static int (*SDL_NAME(esd_open_sound))( const char *host );
static int (*SDL_NAME(esd_close))( int esd );
static int (*SDL_NAME(esd_play_stream))( esd_format_t format, int rate,
                                         const char *host, const char *name );
static struct {
	const char *name;
	void **func;
} esd_functions[] = {
	{ "esd_open_sound",	(void **)&SDL_NAME(esd_open_sound)	},
	{ "esd_close",		(void **)&SDL_NAME(esd_close)		},
	{ "esd_play_stream",	(void **)&SDL_NAME(esd_play_stream)	},
};

static void UnloadESDLibrary()
{
	if ( esd_loaded ) {
		SDL_UnloadObject(esd_handle);
		esd_handle = NULL;
		esd_loaded = 0;
	}
}

static int LoadESDLibrary(void)
{
	int i, retval = -1;

	esd_handle = SDL_LoadObject(esd_library);
	if ( esd_handle ) {
		esd_loaded = 1;
		retval = 0;
		for ( i=0; i<SDL_arraysize(esd_functions); ++i ) {
			*esd_functions[i].func = SDL_LoadFunction(esd_handle, esd_functions[i].name);
			if ( !*esd_functions[i].func ) {
				retval = -1;
				UnloadESDLibrary();
				break;
			}
		}
	}
	return retval;
}

#else

static void UnloadESDLibrary()
{
	return;
}

static int LoadESDLibrary(void)
{
	return 0;
}

#endif /* SDL_AUDIO_DRIVER_ESD_DYNAMIC */

/* Audio driver bootstrap functions */

static int Audio_Available(void)
{
	int connection;
	int available;

	available = 0;
	if ( LoadESDLibrary() < 0 ) {
		return available;
	}
	connection = SDL_NAME(esd_open_sound)(NULL);
	if ( connection >= 0 ) {
		available = 1;
		SDL_NAME(esd_close)(connection);
	}
	UnloadESDLibrary();
	return(available);
}

static void Audio_DeleteDevice(SDL_AudioDevice *device)
{
	SDL_free(device->hidden);
	SDL_free(device);
	UnloadESDLibrary();
}

static SDL_AudioDevice *Audio_CreateDevice(int devindex)
{
	SDL_AudioDevice *this;

	/* Initialize all variables that we clean on shutdown */
	LoadESDLibrary();
	this = (SDL_AudioDevice *)SDL_malloc(sizeof(SDL_AudioDevice));
	if ( this ) {
		SDL_memset(this, 0, (sizeof *this));
		this->hidden = (struct SDL_PrivateAudioData *)
				SDL_malloc((sizeof *this->hidden));
	}
	if ( (this == NULL) || (this->hidden == NULL) ) {
		SDL_OutOfMemory();
		if ( this ) {
			SDL_free(this);
		}
		return(0);
	}
	SDL_memset(this->hidden, 0, (sizeof *this->hidden));
	audio_fd = -1;

	/* Set the function pointers */
	this->OpenAudio = ESD_OpenAudio;
	this->WaitAudio = ESD_WaitAudio;
	this->PlayAudio = ESD_PlayAudio;
	this->GetAudioBuf = ESD_GetAudioBuf;
	this->CloseAudio = ESD_CloseAudio;

	this->free = Audio_DeleteDevice;

	return this;
}

AudioBootStrap ESD_bootstrap = {
	ESD_DRIVER_NAME, "Enlightened Sound Daemon",
	Audio_Available, Audio_CreateDevice
};

/* This function waits until it is possible to write a full sound buffer */
static void ESD_WaitAudio(_THIS)
{
	Sint32 ticks;

	/* Check to see if the thread-parent process is still alive */
	{ static int cnt = 0;
		/* Note that this only works with thread implementations 
		   that use a different process id for each thread.
		*/
		if (parent && (((++cnt)%10) == 0)) { /* Check every 10 loops */
			if ( kill(parent, 0) < 0 ) {
				this->enabled = 0;
			}
		}
	}

	/* Use timer for general audio synchronization */
	ticks = ((Sint32)(next_frame - SDL_GetTicks()))-FUDGE_TICKS;
	if ( ticks > 0 ) {
		SDL_Delay(ticks);
	}
}

static void ESD_PlayAudio(_THIS)
{
	int written;

	/* Write the audio data, checking for EAGAIN on broken audio drivers */
	do {
		written = write(audio_fd, mixbuf, mixlen);
		if ( (written < 0) && ((errno == 0) || (errno == EAGAIN)) ) {
			SDL_Delay(1);	/* Let a little CPU time go by */
		}
	} while ( (written < 0) && 
	          ((errno == 0) || (errno == EAGAIN) || (errno == EINTR)) );

	/* Set the next write frame */
	next_frame += frame_ticks;

	/* If we couldn't write, assume fatal error for now */
	if ( written < 0 ) {
		this->enabled = 0;
	}
}

static Uint8 *ESD_GetAudioBuf(_THIS)
{
	return(mixbuf);
}

static void ESD_CloseAudio(_THIS)
{
	if ( mixbuf != NULL ) {
		SDL_FreeAudioMem(mixbuf);
		mixbuf = NULL;
	}
	if ( audio_fd >= 0 ) {
		SDL_NAME(esd_close)(audio_fd);
		audio_fd = -1;
	}
}

/* Try to get the name of the program */
static char *get_progname(void)
{
	char *progname = NULL;
#ifdef __LINUX__
	FILE *fp;
	static char temp[BUFSIZ];

	SDL_snprintf(temp, SDL_arraysize(temp), "/proc/%d/cmdline", getpid());
	fp = fopen(temp, "r");
	if ( fp != NULL ) {
		if ( fgets(temp, sizeof(temp)-1, fp) ) {
			progname = SDL_strrchr(temp, '/');
			if ( progname == NULL ) {
				progname = temp;
			} else {
				progname = progname+1;
			}
		}
		fclose(fp);
	}
#endif
	return(progname);
}

static int ESD_OpenAudio(_THIS, SDL_AudioSpec *spec)
{
	esd_format_t format;

	/* Convert audio spec to the ESD audio format */
	format = (ESD_STREAM | ESD_PLAY);
	switch ( spec->format & 0xFF ) {
		case 8:
			format |= ESD_BITS8;
			break;
		case 16:
			format |= ESD_BITS16;
			break;
		default:
			SDL_SetError("Unsupported ESD audio format");
			return(-1);
	}
	if ( spec->channels == 1 ) {
		format |= ESD_MONO;
	} else {
		format |= ESD_STEREO;
	}
#if 0
	spec->samples = ESD_BUF_SIZE;	/* Darn, no way to change this yet */
#endif

	/* Open a connection to the ESD audio server */
	audio_fd = SDL_NAME(esd_play_stream)(format, spec->freq, NULL, get_progname());
	if ( audio_fd < 0 ) {
		SDL_SetError("Couldn't open ESD connection");
		return(-1);
	}

	/* Calculate the final parameters for this audio specification */
	SDL_CalculateAudioSpec(spec);
	frame_ticks = (float)(spec->samples*1000)/spec->freq;
	next_frame = SDL_GetTicks()+frame_ticks;

	/* Allocate mixing buffer */
	mixlen = spec->size;
	mixbuf = (Uint8 *)SDL_AllocAudioMem(mixlen);
	if ( mixbuf == NULL ) {
		return(-1);
	}
	SDL_memset(mixbuf, spec->silence, spec->size);

	/* Get the parent process id (we're the parent of the audio thread) */
	parent = getpid();

	/* We're ready to rock and roll. :-) */
	return(0);
}
