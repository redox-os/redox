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

/* Allow access to the audio stream on BeOS */

#include <SoundPlayer.h>

#include "../../main/beos/SDL_BeApp.h"

extern "C" {

#include "SDL_audio.h"
#include "../SDL_audio_c.h"
#include "../SDL_sysaudio.h"
#include "../../thread/beos/SDL_systhread_c.h"
#include "SDL_beaudio.h"


/* Audio driver functions */
static int BE_OpenAudio(_THIS, SDL_AudioSpec *spec);
static void BE_WaitAudio(_THIS);
static void BE_PlayAudio(_THIS);
static Uint8 *BE_GetAudioBuf(_THIS);
static void BE_CloseAudio(_THIS);

/* Audio driver bootstrap functions */

static int Audio_Available(void)
{
	return(1);
}

static void Audio_DeleteDevice(SDL_AudioDevice *device)
{
	SDL_free(device->hidden);
	SDL_free(device);
}

static SDL_AudioDevice *Audio_CreateDevice(int devindex)
{
	SDL_AudioDevice *device;

	/* Initialize all variables that we clean on shutdown */
	device = (SDL_AudioDevice *)SDL_malloc(sizeof(SDL_AudioDevice));
	if ( device ) {
		SDL_memset(device, 0, (sizeof *device));
		device->hidden = (struct SDL_PrivateAudioData *)
				SDL_malloc((sizeof *device->hidden));
	}
	if ( (device == NULL) || (device->hidden == NULL) ) {
		SDL_OutOfMemory();
		if ( device ) {
			SDL_free(device);
		}
		return(0);
	}
	SDL_memset(device->hidden, 0, (sizeof *device->hidden));

	/* Set the function pointers */
	device->OpenAudio = BE_OpenAudio;
	device->WaitAudio = BE_WaitAudio;
	device->PlayAudio = BE_PlayAudio;
	device->GetAudioBuf = BE_GetAudioBuf;
	device->CloseAudio = BE_CloseAudio;

	device->free = Audio_DeleteDevice;

	return device;
}

AudioBootStrap BAUDIO_bootstrap = {
	"baudio", "BeOS BSoundPlayer",
	Audio_Available, Audio_CreateDevice
};

/* The BeOS callback for handling the audio buffer */
static void FillSound(void *device, void *stream, size_t len, 
					const media_raw_audio_format &format)
{
	SDL_AudioDevice *audio = (SDL_AudioDevice *)device;

	/* Silence the buffer, since it's ours */
	SDL_memset(stream, audio->spec.silence, len);

	/* Only do soemthing if audio is enabled */
	if ( ! audio->enabled )
		return;

	if ( ! audio->paused ) {
		if ( audio->convert.needed ) {
			SDL_mutexP(audio->mixer_lock);
			(*audio->spec.callback)(audio->spec.userdata,
				(Uint8 *)audio->convert.buf,audio->convert.len);
			SDL_mutexV(audio->mixer_lock);
			SDL_ConvertAudio(&audio->convert);
			SDL_memcpy(stream,audio->convert.buf,audio->convert.len_cvt);
		} else {
			SDL_mutexP(audio->mixer_lock);
			(*audio->spec.callback)(audio->spec.userdata,
						(Uint8 *)stream, len);
			SDL_mutexV(audio->mixer_lock);
		}
	}
	return;
}

/* Dummy functions -- we don't use thread-based audio */
void BE_WaitAudio(_THIS)
{
	return;
}
void BE_PlayAudio(_THIS)
{
	return;
}
Uint8 *BE_GetAudioBuf(_THIS)
{
	return(NULL);
}

void BE_CloseAudio(_THIS)
{
	if ( audio_obj ) {
		audio_obj->Stop();
		delete audio_obj;
		audio_obj = NULL;
	}

	/* Quit the Be Application, if there's nothing left to do */
	SDL_QuitBeApp();
}

int BE_OpenAudio(_THIS, SDL_AudioSpec *spec)
{
    int valid_datatype = 0;
    media_raw_audio_format format;
    Uint16 test_format = SDL_FirstAudioFormat(spec->format);

    /* Parse the audio format and fill the Be raw audio format */
    memset(&format, '\0', sizeof (media_raw_audio_format));
    format.byte_order = B_MEDIA_LITTLE_ENDIAN;
    format.frame_rate = (float) spec->freq;
    format.channel_count = spec->channels;  /* !!! FIXME: support > 2? */
    while ((!valid_datatype) && (test_format)) {
        valid_datatype = 1;
        spec->format = test_format;
        switch (test_format) {
            case AUDIO_S8:
                format.format = media_raw_audio_format::B_AUDIO_CHAR;
                break;

            case AUDIO_U8:
                format.format = media_raw_audio_format::B_AUDIO_UCHAR;
                break;

            case AUDIO_S16LSB:
                format.format = media_raw_audio_format::B_AUDIO_SHORT;
                break;

            case AUDIO_S16MSB:
                format.format = media_raw_audio_format::B_AUDIO_SHORT;
                format.byte_order = B_MEDIA_BIG_ENDIAN;
                break;

            default:
                valid_datatype = 0;
                test_format = SDL_NextAudioFormat();
                break;
        }
    }

    if (!valid_datatype) { /* shouldn't happen, but just in case... */
        SDL_SetError("Unsupported audio format");
        return (-1);
    }

    /* Initialize the Be Application, if it's not already started */
    if (SDL_InitBeApp() < 0) {
        return (-1);
    }

    format.buffer_size = spec->samples;

	/* Calculate the final parameters for this audio specification */
	SDL_CalculateAudioSpec(spec);

	/* Subscribe to the audio stream (creates a new thread) */
	{ sigset_t omask;
		SDL_MaskSignals(&omask);
		audio_obj = new BSoundPlayer(&format, "SDL Audio", FillSound,
		                                                 NULL, _this);
		SDL_UnmaskSignals(&omask);
	}
	if ( audio_obj->Start() == B_NO_ERROR ) {
		audio_obj->SetHasData(true);
	} else {
		SDL_SetError("Unable to start Be audio");
		return(-1);
	}

	/* We're running! */
	return(1);
}

};	/* Extern C */
