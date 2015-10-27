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

/* Allow access to a raw mixing buffer */

#include "SDL.h"
#include "SDL_audio_c.h"
#include "SDL_audiomem.h"
#include "SDL_sysaudio.h"

#ifdef __OS2__
/* We'll need the DosSetPriority() API! */
#define INCL_DOSPROCESS
#include <os2.h>
#endif

/* Available audio drivers */
static AudioBootStrap *bootstrap[] = {
#if SDL_AUDIO_DRIVER_PULSE
	&PULSE_bootstrap,
#endif
#if SDL_AUDIO_DRIVER_ALSA
	&ALSA_bootstrap,
#endif
#if SDL_AUDIO_DRIVER_BSD
	&BSD_AUDIO_bootstrap,
#endif
#if SDL_AUDIO_DRIVER_OSS
	&DSP_bootstrap,
	&DMA_bootstrap,
#endif
#if SDL_AUDIO_DRIVER_QNXNTO
	&QNXNTOAUDIO_bootstrap,
#endif
#if SDL_AUDIO_DRIVER_SUNAUDIO
	&SUNAUDIO_bootstrap,
#endif
#if SDL_AUDIO_DRIVER_DMEDIA
	&DMEDIA_bootstrap,
#endif
#if SDL_AUDIO_DRIVER_ARTS
	&ARTS_bootstrap,
#endif
#if SDL_AUDIO_DRIVER_ESD
	&ESD_bootstrap,
#endif
#if SDL_AUDIO_DRIVER_NAS
	&NAS_bootstrap,
#endif
#if SDL_AUDIO_DRIVER_DSOUND
	&DSOUND_bootstrap,
#endif
#if SDL_AUDIO_DRIVER_WAVEOUT
	&WAVEOUT_bootstrap,
#endif
#if SDL_AUDIO_DRIVER_PAUD
	&Paud_bootstrap,
#endif
#if SDL_AUDIO_DRIVER_BAUDIO
	&BAUDIO_bootstrap,
#endif
#if SDL_AUDIO_DRIVER_COREAUDIO
	&COREAUDIO_bootstrap,
#endif
#if SDL_AUDIO_DRIVER_SNDMGR
	&SNDMGR_bootstrap,
#endif
#if SDL_AUDIO_DRIVER_MINT
	&MINTAUDIO_GSXB_bootstrap,
	&MINTAUDIO_MCSN_bootstrap,
	&MINTAUDIO_STFA_bootstrap,
	&MINTAUDIO_XBIOS_bootstrap,
	&MINTAUDIO_DMA8_bootstrap,
#endif
#if SDL_AUDIO_DRIVER_DISK
	&DISKAUD_bootstrap,
#endif
#if SDL_AUDIO_DRIVER_DUMMY
	&DUMMYAUD_bootstrap,
#endif
#if SDL_AUDIO_DRIVER_DC
	&DCAUD_bootstrap,
#endif
#if SDL_AUDIO_DRIVER_NDS
	&NDSAUD_bootstrap,
#endif
#if SDL_AUDIO_DRIVER_MMEAUDIO
	&MMEAUDIO_bootstrap,
#endif
#if SDL_AUDIO_DRIVER_DART
	&DART_bootstrap,
#endif
#if SDL_AUDIO_DRIVER_EPOCAUDIO
	&EPOCAudio_bootstrap,
#endif
	NULL
};
SDL_AudioDevice *current_audio = NULL;

/* Various local functions */
int SDL_AudioInit(const char *driver_name);
void SDL_AudioQuit(void);

/* The general mixing thread function */
int SDLCALL SDL_RunAudio(void *audiop)
{
	SDL_AudioDevice *audio = (SDL_AudioDevice *)audiop;
	Uint8 *stream;
	int    stream_len;
	void  *udata;
	void (SDLCALL *fill)(void *userdata,Uint8 *stream, int len);
	int    silence;

	/* Perform any thread setup */
	if ( audio->ThreadInit ) {
		audio->ThreadInit(audio);
	}
	audio->threadid = SDL_ThreadID();

	/* Set up the mixing function */
	fill  = audio->spec.callback;
	udata = audio->spec.userdata;

	if ( audio->convert.needed ) {
		if ( audio->convert.src_format == AUDIO_U8 ) {
			silence = 0x80;
		} else {
			silence = 0;
		}
		stream_len = audio->convert.len;
	} else {
		silence = audio->spec.silence;
		stream_len = audio->spec.size;
	}

#ifdef __OS2__
        /* Increase the priority of this thread to make sure that
           the audio will be continuous all the time! */
#ifdef USE_DOSSETPRIORITY
        if (SDL_getenv("SDL_USE_TIMECRITICAL_AUDIO"))
        {
#ifdef DEBUG_BUILD
          printf("[SDL_RunAudio] : Setting priority to TimeCritical+0! (TID%d)\n", SDL_ThreadID());
#endif
          DosSetPriority(PRTYS_THREAD, PRTYC_TIMECRITICAL, 0, 0);
        }
        else
        {
#ifdef DEBUG_BUILD
          printf("[SDL_RunAudio] : Setting priority to ForegroundServer+0! (TID%d)\n", SDL_ThreadID());
#endif
          DosSetPriority(PRTYS_THREAD, PRTYC_FOREGROUNDSERVER, 0, 0);
        }
#endif
#endif

	/* Loop, filling the audio buffers */
	while ( audio->enabled ) {

		/* Fill the current buffer with sound */
		if ( audio->convert.needed ) {
			if ( audio->convert.buf ) {
				stream = audio->convert.buf;
			} else {
				continue;
			}
		} else {
			stream = audio->GetAudioBuf(audio);
			if ( stream == NULL ) {
				stream = audio->fake_stream;
			}
		}

		SDL_memset(stream, silence, stream_len);

		if ( ! audio->paused ) {
			SDL_mutexP(audio->mixer_lock);
			(*fill)(udata, stream, stream_len);
			SDL_mutexV(audio->mixer_lock);
		}

		/* Convert the audio if necessary */
		if ( audio->convert.needed ) {
			SDL_ConvertAudio(&audio->convert);
			stream = audio->GetAudioBuf(audio);
			if ( stream == NULL ) {
				stream = audio->fake_stream;
			}
			SDL_memcpy(stream, audio->convert.buf,
			               audio->convert.len_cvt);
		}

		/* Ready current buffer for play and change current buffer */
		if ( stream != audio->fake_stream ) {
			audio->PlayAudio(audio);
		}

		/* Wait for an audio buffer to become available */
		if ( stream == audio->fake_stream ) {
			SDL_Delay((audio->spec.samples*1000)/audio->spec.freq);
		} else {
			audio->WaitAudio(audio);
		}
	}

	/* Wait for the audio to drain.. */
	if ( audio->WaitDone ) {
		audio->WaitDone(audio);
	}

#ifdef __OS2__
#ifdef DEBUG_BUILD
        printf("[SDL_RunAudio] : Task exiting. (TID%d)\n", SDL_ThreadID());
#endif
#endif
	return(0);
}

static void SDL_LockAudio_Default(SDL_AudioDevice *audio)
{
	if ( audio->thread && (SDL_ThreadID() == audio->threadid) ) {
		return;
	}
	SDL_mutexP(audio->mixer_lock);
}

static void SDL_UnlockAudio_Default(SDL_AudioDevice *audio)
{
	if ( audio->thread && (SDL_ThreadID() == audio->threadid) ) {
		return;
	}
	SDL_mutexV(audio->mixer_lock);
}

static Uint16 SDL_ParseAudioFormat(const char *string)
{
	Uint16 format = 0;

	switch (*string) {
	    case 'U':
		++string;
		format |= 0x0000;
		break;
	    case 'S':
		++string;
		format |= 0x8000;
		break;
	    default:
		return 0;
	}
	switch (SDL_atoi(string)) {
	    case 8:
		string += 1;
		format |= 8;
		break;
	    case 16:
		string += 2;
		format |= 16;
		if ( SDL_strcmp(string, "LSB") == 0
#if SDL_BYTEORDER == SDL_LIL_ENDIAN
		     || SDL_strcmp(string, "SYS") == 0
#endif
		    ) {
			format |= 0x0000;
		}
		if ( SDL_strcmp(string, "MSB") == 0
#if SDL_BYTEORDER == SDL_BIG_ENDIAN
		     || SDL_strcmp(string, "SYS") == 0
#endif
		    ) {
			format |= 0x1000;
		}
		break;
	    default:
		return 0;
	}
	return format;
}

int SDL_AudioInit(const char *driver_name)
{
	SDL_AudioDevice *audio;
	int i = 0, idx;

	/* Check to make sure we don't overwrite 'current_audio' */
	if ( current_audio != NULL ) {
		SDL_AudioQuit();
	}

	/* Select the proper audio driver */
	audio = NULL;
	idx = 0;
#if SDL_AUDIO_DRIVER_ESD
	if ( (driver_name == NULL) && (SDL_getenv("ESPEAKER") != NULL) ) {
		/* Ahem, we know that if ESPEAKER is set, user probably wants
		   to use ESD, but don't start it if it's not already running.
		   This probably isn't the place to do this, but... Shh! :)
		 */
		for ( i=0; bootstrap[i]; ++i ) {
			if ( SDL_strcasecmp(bootstrap[i]->name, "esd") == 0 ) {
#ifdef HAVE_PUTENV
				const char *esd_no_spawn;

				/* Don't start ESD if it's not running */
				esd_no_spawn = getenv("ESD_NO_SPAWN");
				if ( esd_no_spawn == NULL ) {
					putenv("ESD_NO_SPAWN=1");
				}
#endif
				if ( bootstrap[i]->available() ) {
					audio = bootstrap[i]->create(0);
					break;
				}
#ifdef HAVE_UNSETENV
				if ( esd_no_spawn == NULL ) {
					unsetenv("ESD_NO_SPAWN");
				}
#endif
			}
		}
	}
#endif /* SDL_AUDIO_DRIVER_ESD */
	if ( audio == NULL ) {
		if ( driver_name != NULL ) {
#if 0	/* This will be replaced with a better driver selection API */
			if ( SDL_strrchr(driver_name, ':') != NULL ) {
				idx = atoi(SDL_strrchr(driver_name, ':')+1);
			}
#endif
			for ( i=0; bootstrap[i]; ++i ) {
				if (SDL_strcasecmp(bootstrap[i]->name, driver_name) == 0) {
					if ( bootstrap[i]->available() ) {
						audio=bootstrap[i]->create(idx);
						break;
					}
				}
			}
		} else {
			for ( i=0; bootstrap[i]; ++i ) {
				if ( bootstrap[i]->available() ) {
					audio = bootstrap[i]->create(idx);
					if ( audio != NULL ) {
						break;
					}
				}
			}
		}
		if ( audio == NULL ) {
			SDL_SetError("No available audio device");
#if 0 /* Don't fail SDL_Init() if audio isn't available.
         SDL_OpenAudio() will handle it at that point.  *sigh*
       */
			return(-1);
#endif
		}
	}
	current_audio = audio;
	if ( current_audio ) {
		current_audio->name = bootstrap[i]->name;
		if ( !current_audio->LockAudio && !current_audio->UnlockAudio ) {
			current_audio->LockAudio = SDL_LockAudio_Default;
			current_audio->UnlockAudio = SDL_UnlockAudio_Default;
		}
	}
	return(0);
}

char *SDL_AudioDriverName(char *namebuf, int maxlen)
{
	if ( current_audio != NULL ) {
		SDL_strlcpy(namebuf, current_audio->name, maxlen);
		return(namebuf);
	}
	return(NULL);
}

int SDL_OpenAudio(SDL_AudioSpec *desired, SDL_AudioSpec *obtained)
{
	SDL_AudioDevice *audio;
	const char *env;

	/* Start up the audio driver, if necessary */
	if ( ! current_audio ) {
		if ( (SDL_InitSubSystem(SDL_INIT_AUDIO) < 0) ||
		     (current_audio == NULL) ) {
			return(-1);
		}
	}
	audio = current_audio;

	if (audio->opened) {
		SDL_SetError("Audio device is already opened");
		return(-1);
	}

	/* Verify some parameters */
	if ( desired->freq == 0 ) {
		env = SDL_getenv("SDL_AUDIO_FREQUENCY");
		if ( env ) {
			desired->freq = SDL_atoi(env);
		}
	}
	if ( desired->freq == 0 ) {
		/* Pick some default audio frequency */
		desired->freq = 22050;
	}
	if ( desired->format == 0 ) {
		env = SDL_getenv("SDL_AUDIO_FORMAT");
		if ( env ) {
			desired->format = SDL_ParseAudioFormat(env);
		}
	}
	if ( desired->format == 0 ) {
		/* Pick some default audio format */
		desired->format = AUDIO_S16;
	}
	if ( desired->channels == 0 ) {
		env = SDL_getenv("SDL_AUDIO_CHANNELS");
		if ( env ) {
			desired->channels = (Uint8)SDL_atoi(env);
		}
	}
	if ( desired->channels == 0 ) {
		/* Pick a default number of channels */
		desired->channels = 2;
	}
	switch ( desired->channels ) {
	    case 1:	/* Mono */
	    case 2:	/* Stereo */
	    case 4:	/* surround */
	    case 6:	/* surround with center and lfe */
		break;
	    default:
		SDL_SetError("1 (mono) and 2 (stereo) channels supported");
		return(-1);
	}
	if ( desired->samples == 0 ) {
		env = SDL_getenv("SDL_AUDIO_SAMPLES");
		if ( env ) {
			desired->samples = (Uint16)SDL_atoi(env);
		}
	}
	if ( desired->samples == 0 ) {
		/* Pick a default of ~46 ms at desired frequency */
		int samples = (desired->freq / 1000) * 46;
		int power2 = 1;
		while ( power2 < samples ) {
			power2 *= 2;
		}
		desired->samples = power2;
	}
	if ( desired->callback == NULL ) {
		SDL_SetError("SDL_OpenAudio() passed a NULL callback");
		return(-1);
	}

#if SDL_THREADS_DISABLED
	/* Uses interrupt driven audio, without thread */
#else
	/* Create a semaphore for locking the sound buffers */
	audio->mixer_lock = SDL_CreateMutex();
	if ( audio->mixer_lock == NULL ) {
		SDL_SetError("Couldn't create mixer lock");
		SDL_CloseAudio();
		return(-1);
	}
#endif /* SDL_THREADS_DISABLED */

	/* Calculate the silence and size of the audio specification */
	SDL_CalculateAudioSpec(desired);

	/* Open the audio subsystem */
	SDL_memcpy(&audio->spec, desired, sizeof(audio->spec));
	audio->convert.needed = 0;
	audio->enabled = 1;
	audio->paused  = 1;

	audio->opened = audio->OpenAudio(audio, &audio->spec)+1;

	if ( ! audio->opened ) {
		SDL_CloseAudio();
		return(-1);
	}

	/* If the audio driver changes the buffer size, accept it */
	if ( audio->spec.samples != desired->samples ) {
		desired->samples = audio->spec.samples;
		SDL_CalculateAudioSpec(desired);
	}

	/* Allocate a fake audio memory buffer */
	audio->fake_stream = SDL_AllocAudioMem(audio->spec.size);
	if ( audio->fake_stream == NULL ) {
		SDL_CloseAudio();
		SDL_OutOfMemory();
		return(-1);
	}

	/* See if we need to do any conversion */
	if ( obtained != NULL ) {
		SDL_memcpy(obtained, &audio->spec, sizeof(audio->spec));
	} else if ( desired->freq != audio->spec.freq ||
                    desired->format != audio->spec.format ||
	            desired->channels != audio->spec.channels ) {
		/* Build an audio conversion block */
		if ( SDL_BuildAudioCVT(&audio->convert,
			desired->format, desired->channels,
					desired->freq,
			audio->spec.format, audio->spec.channels,
					audio->spec.freq) < 0 ) {
			SDL_CloseAudio();
			return(-1);
		}
		if ( audio->convert.needed ) {
			audio->convert.len = (int) ( ((double) audio->spec.size) /
                                          audio->convert.len_ratio );
			audio->convert.buf =(Uint8 *)SDL_AllocAudioMem(
			   audio->convert.len*audio->convert.len_mult);
			if ( audio->convert.buf == NULL ) {
				SDL_CloseAudio();
				SDL_OutOfMemory();
				return(-1);
			}
		}
	}

	/* Start the audio thread if necessary */
	switch (audio->opened) {
		case  1:
			/* Start the audio thread */
#if (defined(__WIN32__) && !defined(_WIN32_WCE)) && !defined(HAVE_LIBC) && !defined(__SYMBIAN32__)
#undef SDL_CreateThread
			audio->thread = SDL_CreateThread(SDL_RunAudio, audio, NULL, NULL);
#else
			audio->thread = SDL_CreateThread(SDL_RunAudio, audio);
#endif
			if ( audio->thread == NULL ) {
				SDL_CloseAudio();
				SDL_SetError("Couldn't create audio thread");
				return(-1);
			}
			break;

		default:
			/* The audio is now playing */
			break;
	}

	return(0);
}

SDL_audiostatus SDL_GetAudioStatus(void)
{
	SDL_AudioDevice *audio = current_audio;
	SDL_audiostatus status;

	status = SDL_AUDIO_STOPPED;
	if ( audio && audio->enabled ) {
		if ( audio->paused ) {
			status = SDL_AUDIO_PAUSED;
		} else {
			status = SDL_AUDIO_PLAYING;
		}
	}
	return(status);
}

void SDL_PauseAudio (int pause_on)
{
	SDL_AudioDevice *audio = current_audio;

	if ( audio ) {
		audio->paused = pause_on;
	}
}

void SDL_LockAudio (void)
{
	SDL_AudioDevice *audio = current_audio;

	/* Obtain a lock on the mixing buffers */
	if ( audio && audio->LockAudio ) {
		audio->LockAudio(audio);
	}
}

void SDL_UnlockAudio (void)
{
	SDL_AudioDevice *audio = current_audio;

	/* Release lock on the mixing buffers */
	if ( audio && audio->UnlockAudio ) {
		audio->UnlockAudio(audio);
	}
}

void SDL_CloseAudio (void)
{
	SDL_QuitSubSystem(SDL_INIT_AUDIO);
}

void SDL_AudioQuit(void)
{
	SDL_AudioDevice *audio = current_audio;

	if ( audio ) {
		audio->enabled = 0;
		if ( audio->thread != NULL ) {
			SDL_WaitThread(audio->thread, NULL);
		}
		if ( audio->mixer_lock != NULL ) {
			SDL_DestroyMutex(audio->mixer_lock);
		}
		if ( audio->fake_stream != NULL ) {
			SDL_FreeAudioMem(audio->fake_stream);
		}
		if ( audio->convert.needed ) {
			SDL_FreeAudioMem(audio->convert.buf);

		}
		if ( audio->opened ) {
			audio->CloseAudio(audio);
			audio->opened = 0;
		}
		/* Free the driver data */
		audio->free(audio);
		current_audio = NULL;
	}
}

#define NUM_FORMATS	6
static int format_idx;
static int format_idx_sub;
static Uint16 format_list[NUM_FORMATS][NUM_FORMATS] = {
 { AUDIO_U8, AUDIO_S8, AUDIO_S16LSB, AUDIO_S16MSB, AUDIO_U16LSB, AUDIO_U16MSB },
 { AUDIO_S8, AUDIO_U8, AUDIO_S16LSB, AUDIO_S16MSB, AUDIO_U16LSB, AUDIO_U16MSB },
 { AUDIO_S16LSB, AUDIO_S16MSB, AUDIO_U16LSB, AUDIO_U16MSB, AUDIO_U8, AUDIO_S8 },
 { AUDIO_S16MSB, AUDIO_S16LSB, AUDIO_U16MSB, AUDIO_U16LSB, AUDIO_U8, AUDIO_S8 },
 { AUDIO_U16LSB, AUDIO_U16MSB, AUDIO_S16LSB, AUDIO_S16MSB, AUDIO_U8, AUDIO_S8 },
 { AUDIO_U16MSB, AUDIO_U16LSB, AUDIO_S16MSB, AUDIO_S16LSB, AUDIO_U8, AUDIO_S8 },
};

Uint16 SDL_FirstAudioFormat(Uint16 format)
{
	for ( format_idx=0; format_idx < NUM_FORMATS; ++format_idx ) {
		if ( format_list[format_idx][0] == format ) {
			break;
		}
	}
	format_idx_sub = 0;
	return(SDL_NextAudioFormat());
}

Uint16 SDL_NextAudioFormat(void)
{
	if ( (format_idx == NUM_FORMATS) || (format_idx_sub == NUM_FORMATS) ) {
		return(0);
	}
	return(format_list[format_idx][format_idx_sub++]);
}

void SDL_CalculateAudioSpec(SDL_AudioSpec *spec)
{
	switch (spec->format) {
		case AUDIO_U8:
			spec->silence = 0x80;
			break;
		default:
			spec->silence = 0x00;
			break;
	}
	spec->size = (spec->format&0xFF)/8;
	spec->size *= spec->channels;
	spec->size *= spec->samples;
}

void SDL_Audio_SetCaption(const char *caption)
{
	if ((current_audio) && (current_audio->SetCaption)) {
		current_audio->SetCaption(current_audio, caption);
	}
}

