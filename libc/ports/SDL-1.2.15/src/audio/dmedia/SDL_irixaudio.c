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

/* Allow access to a raw mixing buffer (For IRIX 6.5 and higher) */
/* patch for IRIX 5 by Georg Schwarz 18/07/2004 */

#include "SDL_timer.h"
#include "SDL_audio.h"
#include "../SDL_audiomem.h"
#include "../SDL_audio_c.h"
#include "SDL_irixaudio.h"


#ifndef AL_RESOURCE /* as a test whether we use the old IRIX audio libraries */
#define OLD_IRIX_AUDIO
#define alClosePort(x) ALcloseport(x)
#define alFreeConfig(x) ALfreeconfig(x)
#define alGetFillable(x) ALgetfillable(x)
#define alNewConfig() ALnewconfig()
#define alOpenPort(x,y,z) ALopenport(x,y,z)
#define alSetChannels(x,y) ALsetchannels(x,y)
#define alSetQueueSize(x,y) ALsetqueuesize(x,y)
#define alSetSampFmt(x,y) ALsetsampfmt(x,y)
#define alSetWidth(x,y) ALsetwidth(x,y)
#endif

/* Audio driver functions */
static int AL_OpenAudio(_THIS, SDL_AudioSpec *spec);
static void AL_WaitAudio(_THIS);
static void AL_PlayAudio(_THIS);
static Uint8 *AL_GetAudioBuf(_THIS);
static void AL_CloseAudio(_THIS);

/* Audio driver bootstrap functions */

static int Audio_Available(void)
{
	return 1;
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

	/* Set the function pointers */
	this->OpenAudio = AL_OpenAudio;
	this->WaitAudio = AL_WaitAudio;
	this->PlayAudio = AL_PlayAudio;
	this->GetAudioBuf = AL_GetAudioBuf;
	this->CloseAudio = AL_CloseAudio;

	this->free = Audio_DeleteDevice;

	return this;
}

AudioBootStrap DMEDIA_bootstrap = {
	"AL", "IRIX DMedia audio",
	Audio_Available, Audio_CreateDevice
};


void static AL_WaitAudio(_THIS)
{
	Sint32 timeleft;

	timeleft = this->spec.samples - alGetFillable(audio_port);
	if ( timeleft > 0 ) {
		timeleft /= (this->spec.freq/1000);
		SDL_Delay((Uint32)timeleft);
	}
}

static void AL_PlayAudio(_THIS)
{
	/* Write the audio data out */
	if ( alWriteFrames(audio_port, mixbuf, this->spec.samples) < 0 ) {
		/* Assume fatal error, for now */
		this->enabled = 0;
	}
}

static Uint8 *AL_GetAudioBuf(_THIS)
{
	return(mixbuf);
}

static void AL_CloseAudio(_THIS)
{
	if ( mixbuf != NULL ) {
		SDL_FreeAudioMem(mixbuf);
		mixbuf = NULL;
	}
	if ( audio_port != NULL ) {
		alClosePort(audio_port);
		audio_port = NULL;
	}
}

static int AL_OpenAudio(_THIS, SDL_AudioSpec * spec)
{
	Uint16 test_format = SDL_FirstAudioFormat(spec->format);
	long width = 0;
	long fmt = 0;
	int valid = 0;

#ifdef OLD_IRIX_AUDIO
	{
		long audio_param[2];
		audio_param[0] = AL_OUTPUT_RATE;
		audio_param[1] = spec->freq;
		valid = (ALsetparams(AL_DEFAULT_DEVICE, audio_param, 2) < 0);
	}
#else
	{
		ALpv audio_param;
		audio_param.param = AL_RATE;
		audio_param.value.i = spec->freq;
		valid = (alSetParams(AL_DEFAULT_OUTPUT, &audio_param, 1) < 0);
	}
#endif

	while ((!valid) && (test_format)) {
		valid = 1;
		spec->format = test_format;

		switch (test_format) {
			case AUDIO_S8:
				width = AL_SAMPLE_8;
				fmt = AL_SAMPFMT_TWOSCOMP;
				break;

			case AUDIO_S16SYS:
				width = AL_SAMPLE_16;
				fmt = AL_SAMPFMT_TWOSCOMP;
				break;

			default:
				valid = 0;
				test_format = SDL_NextAudioFormat();
				break;
		}

		if (valid) {
			ALconfig audio_config = alNewConfig();
			valid = 0;
			if (audio_config) {
				if (alSetChannels(audio_config, spec->channels) < 0) {
					if (spec->channels > 2) {  /* can't handle > stereo? */
						spec->channels = 2;  /* try again below. */
					}
				}

				if ((alSetSampFmt(audio_config, fmt) >= 0) &&
				    ((!width) || (alSetWidth(audio_config, width) >= 0)) &&
				    (alSetQueueSize(audio_config, spec->samples * 2) >= 0) &&
				    (alSetChannels(audio_config, spec->channels) >= 0)) {

					audio_port = alOpenPort("SDL audio", "w", audio_config);
					if (audio_port == NULL) {
						/* docs say AL_BAD_CHANNELS happens here, too. */
						int err = oserror();
						if (err == AL_BAD_CHANNELS) {
							spec->channels = 2;
							alSetChannels(audio_config, spec->channels);
							audio_port = alOpenPort("SDL audio", "w",
							                        audio_config);
						}
					}

					if (audio_port != NULL) {
						valid = 1;
					}
				}

				alFreeConfig(audio_config);
			}
		}
	}

	if (!valid) {
		SDL_SetError("Unsupported audio format");
		return (-1);
	}

	/* Update the fragment size as size in bytes */
	SDL_CalculateAudioSpec(spec);

	/* Allocate mixing buffer */
	mixbuf = (Uint8 *) SDL_AllocAudioMem(spec->size);
	if (mixbuf == NULL) {
		SDL_OutOfMemory();
		return (-1);
	}
	SDL_memset(mixbuf, spec->silence, spec->size);

	/* We're ready to rock and roll. :-) */
	return (0);
}

