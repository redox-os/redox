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

    This file written by Ryan C. Gordon (icculus@icculus.org)
*/
#include "SDL_config.h"

/* Output raw audio data to a file. */

#if HAVE_STDIO_H
#include <stdio.h>
#endif

#include "SDL_rwops.h"
#include "SDL_timer.h"
#include "SDL_audio.h"
#include "../SDL_audiomem.h"
#include "../SDL_audio_c.h"
#include "../SDL_audiodev_c.h"
#include "SDL_diskaudio.h"

/* The tag name used by DISK audio */
#define DISKAUD_DRIVER_NAME         "disk"

/* environment variables and defaults. */
#define DISKENVR_OUTFILE         "SDL_DISKAUDIOFILE"
#define DISKDEFAULT_OUTFILE      "sdlaudio.raw"
#define DISKENVR_WRITEDELAY      "SDL_DISKAUDIODELAY"
#define DISKDEFAULT_WRITEDELAY   150

/* Audio driver functions */
static int DISKAUD_OpenAudio(_THIS, SDL_AudioSpec *spec);
static void DISKAUD_WaitAudio(_THIS);
static void DISKAUD_PlayAudio(_THIS);
static Uint8 *DISKAUD_GetAudioBuf(_THIS);
static void DISKAUD_CloseAudio(_THIS);

static const char *DISKAUD_GetOutputFilename(void)
{
	const char *envr = SDL_getenv(DISKENVR_OUTFILE);
	return((envr != NULL) ? envr : DISKDEFAULT_OUTFILE);
}

/* Audio driver bootstrap functions */
static int DISKAUD_Available(void)
{
	const char *envr = SDL_getenv("SDL_AUDIODRIVER");
	if (envr && (SDL_strcmp(envr, DISKAUD_DRIVER_NAME) == 0)) {
		return(1);
	}
	return(0);
}

static void DISKAUD_DeleteDevice(SDL_AudioDevice *device)
{
	SDL_free(device->hidden);
	SDL_free(device);
}

static SDL_AudioDevice *DISKAUD_CreateDevice(int devindex)
{
	SDL_AudioDevice *this;
	const char *envr;

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

	envr = SDL_getenv(DISKENVR_WRITEDELAY);
	this->hidden->write_delay = (envr) ? SDL_atoi(envr) : DISKDEFAULT_WRITEDELAY;

	/* Set the function pointers */
	this->OpenAudio = DISKAUD_OpenAudio;
	this->WaitAudio = DISKAUD_WaitAudio;
	this->PlayAudio = DISKAUD_PlayAudio;
	this->GetAudioBuf = DISKAUD_GetAudioBuf;
	this->CloseAudio = DISKAUD_CloseAudio;

	this->free = DISKAUD_DeleteDevice;

	return this;
}

AudioBootStrap DISKAUD_bootstrap = {
	DISKAUD_DRIVER_NAME, "direct-to-disk audio",
	DISKAUD_Available, DISKAUD_CreateDevice
};

/* This function waits until it is possible to write a full sound buffer */
static void DISKAUD_WaitAudio(_THIS)
{
	SDL_Delay(this->hidden->write_delay);
}

static void DISKAUD_PlayAudio(_THIS)
{
	int written;

	/* Write the audio data */
	written = SDL_RWwrite(this->hidden->output,
                        this->hidden->mixbuf, 1,
                        this->hidden->mixlen);

	/* If we couldn't write, assume fatal error for now */
	if ( (Uint32)written != this->hidden->mixlen ) {
		this->enabled = 0;
	}
#ifdef DEBUG_AUDIO
	fprintf(stderr, "Wrote %d bytes of audio data\n", written);
#endif
}

static Uint8 *DISKAUD_GetAudioBuf(_THIS)
{
	return(this->hidden->mixbuf);
}

static void DISKAUD_CloseAudio(_THIS)
{
	if ( this->hidden->mixbuf != NULL ) {
		SDL_FreeAudioMem(this->hidden->mixbuf);
		this->hidden->mixbuf = NULL;
	}
	if ( this->hidden->output != NULL ) {
		SDL_RWclose(this->hidden->output);
		this->hidden->output = NULL;
	}
}

static int DISKAUD_OpenAudio(_THIS, SDL_AudioSpec *spec)
{
	const char *fname = DISKAUD_GetOutputFilename();

	/* Open the audio device */
	this->hidden->output = SDL_RWFromFile(fname, "wb");
	if ( this->hidden->output == NULL ) {
		return(-1);
	}

#if HAVE_STDIO_H
	fprintf(stderr, "WARNING: You are using the SDL disk writer"
                    " audio driver!\n Writing to file [%s].\n", fname);
#endif

	/* Allocate mixing buffer */
	this->hidden->mixlen = spec->size;
	this->hidden->mixbuf = (Uint8 *) SDL_AllocAudioMem(this->hidden->mixlen);
	if ( this->hidden->mixbuf == NULL ) {
		return(-1);
	}
	SDL_memset(this->hidden->mixbuf, spec->silence, spec->size);

	/* We're ready to rock and roll. :-) */
	return(0);
}

