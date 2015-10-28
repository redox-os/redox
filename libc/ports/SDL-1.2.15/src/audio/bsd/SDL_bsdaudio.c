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
 * Driver for native OpenBSD/NetBSD audio(4).
 * vedge@vedge.com.ar.
 */

#include <errno.h>
#include <unistd.h>
#include <fcntl.h>
#include <sys/time.h>
#include <sys/ioctl.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/audioio.h>

#include "SDL_timer.h"
#include "SDL_audio.h"
#include "../SDL_audiomem.h"
#include "../SDL_audio_c.h"
#include "../SDL_audiodev_c.h"
#include "SDL_bsdaudio.h"

/* The tag name used by NetBSD/OpenBSD audio */
#ifdef __NetBSD__
#define BSD_AUDIO_DRIVER_NAME         "netbsd"
#define BSD_AUDIO_DRIVER_DESC         "Native NetBSD audio"
#else
#define BSD_AUDIO_DRIVER_NAME         "openbsd"
#define BSD_AUDIO_DRIVER_DESC         "Native OpenBSD audio"
#endif

/* Open the audio device for playback, and don't block if busy */
/* #define USE_BLOCKING_WRITES */

/* Use timer for synchronization */
/* #define USE_TIMER_SYNC */

/* #define DEBUG_AUDIO */
/* #define DEBUG_AUDIO_STREAM */

#ifdef USE_BLOCKING_WRITES
#define OPEN_FLAGS	O_WRONLY
#else
#define OPEN_FLAGS	(O_WRONLY|O_NONBLOCK)
#endif

/* Audio driver functions */
static void OBSD_WaitAudio(_THIS);
static int OBSD_OpenAudio(_THIS, SDL_AudioSpec *spec);
static void OBSD_PlayAudio(_THIS);
static Uint8 *OBSD_GetAudioBuf(_THIS);
static void OBSD_CloseAudio(_THIS);

#ifdef DEBUG_AUDIO
static void OBSD_Status(_THIS);
#endif

/* Audio driver bootstrap functions */

static int
Audio_Available(void)
{
    int fd;
    int available;

    available = 0;
    fd = SDL_OpenAudioPath(NULL, 0, OPEN_FLAGS, 0);
    if(fd >= 0) {
	available = 1;
	close(fd);
    }
    return(available);
}

static void
Audio_DeleteDevice(SDL_AudioDevice *device)
{
    SDL_free(device->hidden);
    SDL_free(device);
}

static SDL_AudioDevice
*Audio_CreateDevice(int devindex)
{
    SDL_AudioDevice *this;

    /* Initialize all variables that we clean on shutdown */
    this = (SDL_AudioDevice*)SDL_malloc(sizeof(SDL_AudioDevice));
    if(this) {
	SDL_memset(this, 0, (sizeof *this));
	this->hidden =
	    (struct SDL_PrivateAudioData*)SDL_malloc((sizeof *this->hidden));
    }
    if((this == NULL) || (this->hidden == NULL)) {
	SDL_OutOfMemory();
	if(this) SDL_free(this);
	return(0);
    }
    SDL_memset(this->hidden, 0, (sizeof *this->hidden));
    audio_fd = -1;

    /* Set the function pointers */
    this->OpenAudio = OBSD_OpenAudio;
    this->WaitAudio = OBSD_WaitAudio;
    this->PlayAudio = OBSD_PlayAudio;
    this->GetAudioBuf = OBSD_GetAudioBuf;
    this->CloseAudio = OBSD_CloseAudio;

    this->free = Audio_DeleteDevice;
    
    return this;
}

AudioBootStrap BSD_AUDIO_bootstrap = {
	BSD_AUDIO_DRIVER_NAME, BSD_AUDIO_DRIVER_DESC,
	Audio_Available, Audio_CreateDevice
};

/* This function waits until it is possible to write a full sound buffer */
static void
OBSD_WaitAudio(_THIS)
{
#ifndef USE_BLOCKING_WRITES /* Not necessary when using blocking writes */
	/* See if we need to use timed audio synchronization */
	if ( frame_ticks ) {
		/* Use timer for general audio synchronization */
		Sint32 ticks;

		ticks = ((Sint32)(next_frame - SDL_GetTicks()))-FUDGE_TICKS;
		if ( ticks > 0 ) {
			SDL_Delay(ticks);
		}
	} else {
		/* Use select() for audio synchronization */
		fd_set fdset;
		struct timeval timeout;

		FD_ZERO(&fdset);
		FD_SET(audio_fd, &fdset);
		timeout.tv_sec = 10;
		timeout.tv_usec = 0;
#ifdef DEBUG_AUDIO
		fprintf(stderr, "Waiting for audio to get ready\n");
#endif
		if ( select(audio_fd+1, NULL, &fdset, NULL, &timeout) <= 0 ) {
			const char *message =
			"Audio timeout - buggy audio driver? (disabled)";
			/* In general we should never print to the screen,
			   but in this case we have no other way of letting
			   the user know what happened.
			*/
			fprintf(stderr, "SDL: %s\n", message);
			this->enabled = 0;
			/* Don't try to close - may hang */
			audio_fd = -1;
#ifdef DEBUG_AUDIO
			fprintf(stderr, "Done disabling audio\n");
#endif
		}
#ifdef DEBUG_AUDIO
		fprintf(stderr, "Ready!\n");
#endif
	}
#endif /* !USE_BLOCKING_WRITES */
}

static void
OBSD_PlayAudio(_THIS)
{
	int written, p=0;

	/* Write the audio data, checking for EAGAIN on broken audio drivers */
	do {
		written = write(audio_fd, &mixbuf[p], mixlen-p);
		if (written>0)
		   p += written;
		if (written == -1 && errno != 0 && errno != EAGAIN && errno != EINTR)
		{
		   /* Non recoverable error has occurred. It should be reported!!! */
		   perror("audio");
		   break;
		}

		if ( p < written || ((written < 0) && ((errno == 0) || (errno == EAGAIN))) ) {
			SDL_Delay(1);	/* Let a little CPU time go by */
		}
	} while ( p < written );

	/* If timer synchronization is enabled, set the next write frame */
	if ( frame_ticks ) {
		next_frame += frame_ticks;
	}

	/* If we couldn't write, assume fatal error for now */
	if ( written < 0 ) {
		this->enabled = 0;
	}
#ifdef DEBUG_AUDIO
	fprintf(stderr, "Wrote %d bytes of audio data\n", written);
#endif
}

static Uint8
*OBSD_GetAudioBuf(_THIS)
{
    return(mixbuf);
}

static void
OBSD_CloseAudio(_THIS)
{
    if(mixbuf != NULL) {
	SDL_FreeAudioMem(mixbuf);
	mixbuf = NULL;
    }
    if(audio_fd >= 0) {
	close(audio_fd);
	audio_fd = -1;
    }
}

#ifdef DEBUG_AUDIO
void
OBSD_Status(_THIS)
{
    audio_info_t info;

    if(ioctl(audio_fd, AUDIO_GETINFO, &info) < 0) {
	fprintf(stderr,"AUDIO_GETINFO failed.\n");
	return;
    }

    fprintf(stderr,"\n"
"[play/record info]\n"
"buffer size	:   %d bytes\n"
"sample rate	:   %i Hz\n"
"channels	:   %i\n"
"precision	:   %i-bit\n"
"encoding	:   0x%x\n"
"seek		:   %i\n"
"sample count	:   %i\n"
"EOF count	:   %i\n"
"paused		:   %s\n"
"error occured	:   %s\n"
"waiting		:   %s\n"
"active		:   %s\n"
"",
    info.play.buffer_size,
    info.play.sample_rate,
    info.play.channels,
    info.play.precision,
    info.play.encoding,
    info.play.seek,
    info.play.samples,
    info.play.eof,
    info.play.pause ? "yes" : "no",
    info.play.error ? "yes" : "no",
    info.play.waiting ? "yes" : "no",
    info.play.active ? "yes": "no");

    fprintf(stderr,"\n"
"[audio info]\n"
"monitor_gain	:   %i\n"
"hw block size	:   %d bytes\n"
"hi watermark	:   %i\n"
"lo watermark	:   %i\n"
"audio mode	:   %s\n"
"",  
    info.monitor_gain,
    info.blocksize,
    info.hiwat, info.lowat,
    (info.mode == AUMODE_PLAY) ? "PLAY"
    : (info.mode = AUMODE_RECORD) ? "RECORD"
    : (info.mode == AUMODE_PLAY_ALL ? "PLAY_ALL"
    : "?"));
}
#endif /* DEBUG_AUDIO */

static int
OBSD_OpenAudio(_THIS, SDL_AudioSpec *spec)
{
    char audiodev[64];
    Uint16 format;
    audio_info_t info;

    AUDIO_INITINFO(&info);
    
    /* Calculate the final parameters for this audio specification */
    SDL_CalculateAudioSpec(spec);

#ifdef USE_TIMER_SYNC
    frame_ticks = 0.0;
#endif

    /* Open the audio device */
    audio_fd = SDL_OpenAudioPath(audiodev, sizeof(audiodev), OPEN_FLAGS, 0);
    if(audio_fd < 0) {
	SDL_SetError("Couldn't open %s: %s", audiodev, strerror(errno));
	return(-1);
    }
    
    /* Set to play mode */
    info.mode = AUMODE_PLAY;
    if(ioctl(audio_fd, AUDIO_SETINFO, &info) < 0) {
	SDL_SetError("Couldn't put device into play mode");
	return(-1);
    }
    
    mixbuf = NULL;
    AUDIO_INITINFO(&info);
    for (format = SDL_FirstAudioFormat(spec->format); 
    	format; format = SDL_NextAudioFormat())
    {
	switch(format) {
	case AUDIO_U8:
	    info.play.encoding = AUDIO_ENCODING_ULINEAR;
	    info.play.precision = 8;
	    break;
	case AUDIO_S8:
	    info.play.encoding = AUDIO_ENCODING_SLINEAR;
	    info.play.precision = 8;
	    break;
	case AUDIO_S16LSB:
	    info.play.encoding = AUDIO_ENCODING_SLINEAR_LE;
	    info.play.precision = 16;
	    break;
	case AUDIO_S16MSB:
	    info.play.encoding = AUDIO_ENCODING_SLINEAR_BE;
	    info.play.precision = 16;
	    break;
	case AUDIO_U16LSB:
	    info.play.encoding = AUDIO_ENCODING_ULINEAR_LE;
	    info.play.precision = 16;
	    break;
	case AUDIO_U16MSB:
	    info.play.encoding = AUDIO_ENCODING_ULINEAR_BE;
	    info.play.precision = 16;
	    break;
	default:
	    continue;
	}
	if (ioctl(audio_fd, AUDIO_SETINFO, &info) == 0)
	    break;
    }

    if(!format) {
	SDL_SetError("No supported encoding for 0x%x", spec->format);
	return(-1);
    }

    spec->format = format;

    AUDIO_INITINFO(&info);
    info.play.channels = spec->channels;
    if (ioctl(audio_fd, AUDIO_SETINFO, &info) == -1)
    	spec->channels = 1;
    AUDIO_INITINFO(&info);
    info.play.sample_rate = spec->freq;
    info.blocksize = spec->size;
    info.hiwat = 5;
    info.lowat = 3;
    (void)ioctl(audio_fd, AUDIO_SETINFO, &info);
    (void)ioctl(audio_fd, AUDIO_GETINFO, &info);
    spec->freq  = info.play.sample_rate;
    /* Allocate mixing buffer */
    mixlen = spec->size;
    mixbuf = (Uint8*)SDL_AllocAudioMem(mixlen);
    if(mixbuf == NULL) {
	return(-1);
    }
    SDL_memset(mixbuf, spec->silence, spec->size);
    
    /* Get the parent process id (we're the parent of the audio thread) */
    parent = getpid();

#ifdef DEBUG_AUDIO
    OBSD_Status(this);
#endif

    /* We're ready to rock and roll. :-) */
    return(0);
}
