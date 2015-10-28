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

    Modified in Oct 2004 by Hannu Savolainen 
    hannu@opensound.com
*/
#include "SDL_config.h"

/* Allow access to a raw mixing buffer */

#include <stdio.h>	/* For perror() */
#include <string.h>	/* For strerror() */
#include <errno.h>
#include <unistd.h>
#include <fcntl.h>
#include <signal.h>
#include <sys/time.h>
#include <sys/ioctl.h>
#include <sys/stat.h>

#if SDL_AUDIO_DRIVER_OSS_SOUNDCARD_H
/* This is installed on some systems */
#include <soundcard.h>
#else
/* This is recommended by OSS */
#include <sys/soundcard.h>
#endif

#include "SDL_timer.h"
#include "SDL_audio.h"
#include "../SDL_audiomem.h"
#include "../SDL_audio_c.h"
#include "../SDL_audiodev_c.h"
#include "SDL_dspaudio.h"

/* The tag name used by DSP audio */
#define DSP_DRIVER_NAME         "dsp"

/* Open the audio device for playback, and don't block if busy */
#define OPEN_FLAGS	(O_WRONLY|O_NONBLOCK)

/* Audio driver functions */
static int DSP_OpenAudio(_THIS, SDL_AudioSpec *spec);
static void DSP_WaitAudio(_THIS);
static void DSP_PlayAudio(_THIS);
static Uint8 *DSP_GetAudioBuf(_THIS);
static void DSP_CloseAudio(_THIS);

/* Audio driver bootstrap functions */

static int Audio_Available(void)
{
	int fd;
	int available;

	available = 0;
	fd = SDL_OpenAudioPath(NULL, 0, OPEN_FLAGS, 0);
	if ( fd >= 0 ) {
		available = 1;
		close(fd);
	}
	return(available);
}

static void Audio_DeleteDevice(SDL_AudioDevice *device)
{
	SDL_free(device->hidden);
	SDL_free(device);
}

static SDL_AudioDevice *Audio_CreateDevice(int devindex)
{
	SDL_AudioDevice *this;

	/* Initialize all variables that we clean on shutdown */
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
	this->OpenAudio = DSP_OpenAudio;
	this->WaitAudio = DSP_WaitAudio;
	this->PlayAudio = DSP_PlayAudio;
	this->GetAudioBuf = DSP_GetAudioBuf;
	this->CloseAudio = DSP_CloseAudio;

	this->free = Audio_DeleteDevice;

	return this;
}

AudioBootStrap DSP_bootstrap = {
	DSP_DRIVER_NAME, "OSS /dev/dsp standard audio",
	Audio_Available, Audio_CreateDevice
};

/* This function waits until it is possible to write a full sound buffer */
static void DSP_WaitAudio(_THIS)
{
	/* Not needed at all since OSS handles waiting automagically */
}

static void DSP_PlayAudio(_THIS)
{
	if (write(audio_fd, mixbuf, mixlen)==-1)
	{
		perror("Audio write");
		this->enabled = 0;
	}

#ifdef DEBUG_AUDIO
	fprintf(stderr, "Wrote %d bytes of audio data\n", mixlen);
#endif
}

static Uint8 *DSP_GetAudioBuf(_THIS)
{
	return(mixbuf);
}

static void DSP_CloseAudio(_THIS)
{
	if ( mixbuf != NULL ) {
		SDL_FreeAudioMem(mixbuf);
		mixbuf = NULL;
	}
	if ( audio_fd >= 0 ) {
		close(audio_fd);
		audio_fd = -1;
	}
}

static int DSP_OpenAudio(_THIS, SDL_AudioSpec *spec)
{
	char audiodev[1024];
	int format;
	int value;
	int frag_spec;
	Uint16 test_format;

    /* Make sure fragment size stays a power of 2, or OSS fails. */
    /* I don't know which of these are actually legal values, though... */
    if (spec->channels > 8)
        spec->channels = 8;
    else if (spec->channels > 4)
        spec->channels = 4;
    else if (spec->channels > 2)
        spec->channels = 2;

	/* Open the audio device */
	audio_fd = SDL_OpenAudioPath(audiodev, sizeof(audiodev), OPEN_FLAGS, 0);
	if ( audio_fd < 0 ) {
		SDL_SetError("Couldn't open %s: %s", audiodev, strerror(errno));
		return(-1);
	}
	mixbuf = NULL;

	/* Make the file descriptor use blocking writes with fcntl() */
	{ long flags;
		flags = fcntl(audio_fd, F_GETFL);
		flags &= ~O_NONBLOCK;
		if ( fcntl(audio_fd, F_SETFL, flags) < 0 ) {
			SDL_SetError("Couldn't set audio blocking mode");
			DSP_CloseAudio(this);
			return(-1);
		}
	}

	/* Get a list of supported hardware formats */
	if ( ioctl(audio_fd, SNDCTL_DSP_GETFMTS, &value) < 0 ) {
		perror("SNDCTL_DSP_GETFMTS");
		SDL_SetError("Couldn't get audio format list");
		DSP_CloseAudio(this);
		return(-1);
	}

	/* Try for a closest match on audio format */
	format = 0;
	for ( test_format = SDL_FirstAudioFormat(spec->format);
						! format && test_format; ) {
#ifdef DEBUG_AUDIO
		fprintf(stderr, "Trying format 0x%4.4x\n", test_format);
#endif
		switch ( test_format ) {
			case AUDIO_U8:
				if ( value & AFMT_U8 ) {
					format = AFMT_U8;
				}
				break;
			case AUDIO_S16LSB:
				if ( value & AFMT_S16_LE ) {
					format = AFMT_S16_LE;
				}
				break;
			case AUDIO_S16MSB:
				if ( value & AFMT_S16_BE ) {
					format = AFMT_S16_BE;
				}
				break;
#if 0
/*
 * These formats are not used by any real life systems so they are not 
 * needed here.
 */
			case AUDIO_S8:
				if ( value & AFMT_S8 ) {
					format = AFMT_S8;
				}
				break;
			case AUDIO_U16LSB:
				if ( value & AFMT_U16_LE ) {
					format = AFMT_U16_LE;
				}
				break;
			case AUDIO_U16MSB:
				if ( value & AFMT_U16_BE ) {
					format = AFMT_U16_BE;
				}
				break;
#endif
			default:
				format = 0;
				break;
		}
		if ( ! format ) {
			test_format = SDL_NextAudioFormat();
		}
	}
	if ( format == 0 ) {
		SDL_SetError("Couldn't find any hardware audio formats");
		DSP_CloseAudio(this);
		return(-1);
	}
	spec->format = test_format;

	/* Set the audio format */
	value = format;
	if ( (ioctl(audio_fd, SNDCTL_DSP_SETFMT, &value) < 0) ||
						(value != format) ) {
		perror("SNDCTL_DSP_SETFMT");
		SDL_SetError("Couldn't set audio format");
		DSP_CloseAudio(this);
		return(-1);
	}

	/* Set the number of channels of output */
	value = spec->channels;
	if ( ioctl(audio_fd, SNDCTL_DSP_CHANNELS, &value) < 0 ) {
		perror("SNDCTL_DSP_CHANNELS");
		SDL_SetError("Cannot set the number of channels");
		DSP_CloseAudio(this);
		return(-1);
	}
	spec->channels = value;

	/* Set the DSP frequency */
	value = spec->freq;
	if ( ioctl(audio_fd, SNDCTL_DSP_SPEED, &value) < 0 ) {
		perror("SNDCTL_DSP_SPEED");
		SDL_SetError("Couldn't set audio frequency");
		DSP_CloseAudio(this);
		return(-1);
	}
	spec->freq = value;

	/* Calculate the final parameters for this audio specification */
	SDL_CalculateAudioSpec(spec);

	/* Determine the power of two of the fragment size */
	for ( frag_spec = 0; (0x01U<<frag_spec) < spec->size; ++frag_spec );
	if ( (0x01U<<frag_spec) != spec->size ) {
		SDL_SetError("Fragment size must be a power of two");
		DSP_CloseAudio(this);
		return(-1);
	}
	frag_spec |= 0x00020000;	/* two fragments, for low latency */

	/* Set the audio buffering parameters */
#ifdef DEBUG_AUDIO
	fprintf(stderr, "Requesting %d fragments of size %d\n",
		(frag_spec >> 16), 1<<(frag_spec&0xFFFF));
#endif
	if ( ioctl(audio_fd, SNDCTL_DSP_SETFRAGMENT, &frag_spec) < 0 ) {
		perror("SNDCTL_DSP_SETFRAGMENT");
	}
#ifdef DEBUG_AUDIO
	{ audio_buf_info info;
	  ioctl(audio_fd, SNDCTL_DSP_GETOSPACE, &info);
	  fprintf(stderr, "fragments = %d\n", info.fragments);
	  fprintf(stderr, "fragstotal = %d\n", info.fragstotal);
	  fprintf(stderr, "fragsize = %d\n", info.fragsize);
	  fprintf(stderr, "bytes = %d\n", info.bytes);
	}
#endif

	/* Allocate mixing buffer */
	mixlen = spec->size;
	mixbuf = (Uint8 *)SDL_AllocAudioMem(mixlen);
	if ( mixbuf == NULL ) {
		DSP_CloseAudio(this);
		return(-1);
	}
	SDL_memset(mixbuf, spec->silence, spec->size);

	/* Get the parent process id (we're the parent of the audio thread) */
	parent = getpid();

	/* We're ready to rock and roll. :-) */
	return(0);
}
