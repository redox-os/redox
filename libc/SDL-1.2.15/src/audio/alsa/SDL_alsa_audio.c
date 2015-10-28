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

/* Allow access to a raw mixing buffer */

#include <sys/types.h>
#include <signal.h>	/* For kill() */

#include "SDL_timer.h"
#include "SDL_audio.h"
#include "../SDL_audiomem.h"
#include "../SDL_audio_c.h"
#include "SDL_alsa_audio.h"

#ifdef SDL_AUDIO_DRIVER_ALSA_DYNAMIC
#include "SDL_name.h"
#include "SDL_loadso.h"
#else
#define SDL_NAME(X)	X
#endif


/* The tag name used by ALSA audio */
#define DRIVER_NAME         "alsa"

/* Audio driver functions */
static int ALSA_OpenAudio(_THIS, SDL_AudioSpec *spec);
static void ALSA_WaitAudio(_THIS);
static void ALSA_PlayAudio(_THIS);
static Uint8 *ALSA_GetAudioBuf(_THIS);
static void ALSA_CloseAudio(_THIS);

#ifdef SDL_AUDIO_DRIVER_ALSA_DYNAMIC

static const char *alsa_library = SDL_AUDIO_DRIVER_ALSA_DYNAMIC;
static void *alsa_handle = NULL;
static int alsa_loaded = 0;

static int (*SDL_NAME(snd_pcm_open))(snd_pcm_t **pcm, const char *name, snd_pcm_stream_t stream, int mode);
static int (*SDL_NAME(snd_pcm_close))(snd_pcm_t *pcm);
static snd_pcm_sframes_t (*SDL_NAME(snd_pcm_writei))(snd_pcm_t *pcm, const void *buffer, snd_pcm_uframes_t size);
static int (*SDL_NAME(snd_pcm_recover))(snd_pcm_t *pcm, int err, int silent);
static int (*SDL_NAME(snd_pcm_prepare))(snd_pcm_t *pcm);
static int (*SDL_NAME(snd_pcm_drain))(snd_pcm_t *pcm);
static const char *(*SDL_NAME(snd_strerror))(int errnum);
static size_t (*SDL_NAME(snd_pcm_hw_params_sizeof))(void);
static size_t (*SDL_NAME(snd_pcm_sw_params_sizeof))(void);
static void (*SDL_NAME(snd_pcm_hw_params_copy))(snd_pcm_hw_params_t *dst, const snd_pcm_hw_params_t *src);
static int (*SDL_NAME(snd_pcm_hw_params_any))(snd_pcm_t *pcm, snd_pcm_hw_params_t *params);
static int (*SDL_NAME(snd_pcm_hw_params_set_access))(snd_pcm_t *pcm, snd_pcm_hw_params_t *params, snd_pcm_access_t access);
static int (*SDL_NAME(snd_pcm_hw_params_set_format))(snd_pcm_t *pcm, snd_pcm_hw_params_t *params, snd_pcm_format_t val);
static int (*SDL_NAME(snd_pcm_hw_params_set_channels))(snd_pcm_t *pcm, snd_pcm_hw_params_t *params, unsigned int val);
static int (*SDL_NAME(snd_pcm_hw_params_get_channels))(const snd_pcm_hw_params_t *params, unsigned int *val);
static int (*SDL_NAME(snd_pcm_hw_params_set_rate_near))(snd_pcm_t *pcm, snd_pcm_hw_params_t *params, unsigned int *val, int *dir);
static int (*SDL_NAME(snd_pcm_hw_params_set_period_size_near))(snd_pcm_t *pcm, snd_pcm_hw_params_t *params, snd_pcm_uframes_t *val, int *dir);
static int (*SDL_NAME(snd_pcm_hw_params_get_period_size))(const snd_pcm_hw_params_t *params, snd_pcm_uframes_t *frames, int *dir);
static int (*SDL_NAME(snd_pcm_hw_params_set_periods_near))(snd_pcm_t *pcm, snd_pcm_hw_params_t *params, unsigned int *val, int *dir);
static int (*SDL_NAME(snd_pcm_hw_params_get_periods))(const snd_pcm_hw_params_t *params, unsigned int *val, int *dir);
static int (*SDL_NAME(snd_pcm_hw_params_set_buffer_size_near))(snd_pcm_t *pcm, snd_pcm_hw_params_t *params, snd_pcm_uframes_t *val);
static int (*SDL_NAME(snd_pcm_hw_params_get_buffer_size))(const snd_pcm_hw_params_t *params, snd_pcm_uframes_t *val);
static int (*SDL_NAME(snd_pcm_hw_params))(snd_pcm_t *pcm, snd_pcm_hw_params_t *params);
/*
*/
static int (*SDL_NAME(snd_pcm_sw_params_set_avail_min))(snd_pcm_t *pcm, snd_pcm_sw_params_t *swparams, snd_pcm_uframes_t val);
static int (*SDL_NAME(snd_pcm_sw_params_current))(snd_pcm_t *pcm, snd_pcm_sw_params_t *swparams);
static int (*SDL_NAME(snd_pcm_sw_params_set_start_threshold))(snd_pcm_t *pcm, snd_pcm_sw_params_t *params, snd_pcm_uframes_t val);
static int (*SDL_NAME(snd_pcm_sw_params))(snd_pcm_t *pcm, snd_pcm_sw_params_t *params);
static int (*SDL_NAME(snd_pcm_nonblock))(snd_pcm_t *pcm, int nonblock);
static int (*SDL_NAME(snd_pcm_wait))(snd_pcm_t *pcm, int timeout);
#define snd_pcm_hw_params_sizeof SDL_NAME(snd_pcm_hw_params_sizeof)
#define snd_pcm_sw_params_sizeof SDL_NAME(snd_pcm_sw_params_sizeof)

/* cast funcs to char* first, to please GCC's strict aliasing rules. */
static struct {
	const char *name;
	void **func;
} alsa_functions[] = {
	{ "snd_pcm_open",	(void**)(char*)&SDL_NAME(snd_pcm_open)		},
	{ "snd_pcm_close",	(void**)(char*)&SDL_NAME(snd_pcm_close)	},
	{ "snd_pcm_writei",	(void**)(char*)&SDL_NAME(snd_pcm_writei)	},
	{ "snd_pcm_recover",	(void**)(char*)&SDL_NAME(snd_pcm_recover)	},
	{ "snd_pcm_prepare",	(void**)(char*)&SDL_NAME(snd_pcm_prepare)	},
	{ "snd_pcm_drain",	(void**)(char*)&SDL_NAME(snd_pcm_drain)	},
	{ "snd_strerror",	(void**)(char*)&SDL_NAME(snd_strerror)		},
	{ "snd_pcm_hw_params_sizeof",		(void**)(char*)&SDL_NAME(snd_pcm_hw_params_sizeof)		},
	{ "snd_pcm_sw_params_sizeof",		(void**)(char*)&SDL_NAME(snd_pcm_sw_params_sizeof)		},
	{ "snd_pcm_hw_params_copy",		(void**)(char*)&SDL_NAME(snd_pcm_hw_params_copy)		},
	{ "snd_pcm_hw_params_any",		(void**)(char*)&SDL_NAME(snd_pcm_hw_params_any)		},
	{ "snd_pcm_hw_params_set_access",	(void**)(char*)&SDL_NAME(snd_pcm_hw_params_set_access)		},
	{ "snd_pcm_hw_params_set_format",	(void**)(char*)&SDL_NAME(snd_pcm_hw_params_set_format)		},
	{ "snd_pcm_hw_params_set_channels",	(void**)(char*)&SDL_NAME(snd_pcm_hw_params_set_channels)	},
	{ "snd_pcm_hw_params_get_channels",	(void**)(char*)&SDL_NAME(snd_pcm_hw_params_get_channels)	},
	{ "snd_pcm_hw_params_set_rate_near",	(void**)(char*)&SDL_NAME(snd_pcm_hw_params_set_rate_near)	},
	{ "snd_pcm_hw_params_set_period_size_near",	(void**)(char*)&SDL_NAME(snd_pcm_hw_params_set_period_size_near)	},
	{ "snd_pcm_hw_params_get_period_size",	(void**)(char*)&SDL_NAME(snd_pcm_hw_params_get_period_size)	},
	{ "snd_pcm_hw_params_set_periods_near",	(void**)(char*)&SDL_NAME(snd_pcm_hw_params_set_periods_near)	},
	{ "snd_pcm_hw_params_get_periods",	(void**)(char*)&SDL_NAME(snd_pcm_hw_params_get_periods)	},
	{ "snd_pcm_hw_params_set_buffer_size_near",	(void**)(char*)&SDL_NAME(snd_pcm_hw_params_set_buffer_size_near) },
	{ "snd_pcm_hw_params_get_buffer_size",	(void**)(char*)&SDL_NAME(snd_pcm_hw_params_get_buffer_size) },
	{ "snd_pcm_hw_params",	(void**)(char*)&SDL_NAME(snd_pcm_hw_params)	},
	{ "snd_pcm_sw_params_set_avail_min",	(void**)(char*)&SDL_NAME(snd_pcm_sw_params_set_avail_min) },
	{ "snd_pcm_sw_params_current",	(void**)(char*)&SDL_NAME(snd_pcm_sw_params_current)	},
	{ "snd_pcm_sw_params_set_start_threshold",	(void**)(char*)&SDL_NAME(snd_pcm_sw_params_set_start_threshold)	},
	{ "snd_pcm_sw_params",	(void**)(char*)&SDL_NAME(snd_pcm_sw_params)	},
	{ "snd_pcm_nonblock",	(void**)(char*)&SDL_NAME(snd_pcm_nonblock)	},
	{ "snd_pcm_wait",	(void**)(char*)&SDL_NAME(snd_pcm_wait)	},
};

static void UnloadALSALibrary(void) {
	if (alsa_loaded) {
		SDL_UnloadObject(alsa_handle);
		alsa_handle = NULL;
		alsa_loaded = 0;
	}
}

static int LoadALSALibrary(void) {
	int i, retval = -1;

	alsa_handle = SDL_LoadObject(alsa_library);
	if (alsa_handle) {
		alsa_loaded = 1;
		retval = 0;
		for (i = 0; i < SDL_arraysize(alsa_functions); i++) {
			*alsa_functions[i].func = SDL_LoadFunction(alsa_handle,alsa_functions[i].name);
			if (!*alsa_functions[i].func) {
				retval = -1;
				UnloadALSALibrary();
				break;
			}
		}
	}
	return retval;
}

#else

static void UnloadALSALibrary(void) {
	return;
}

static int LoadALSALibrary(void) {
	return 0;
}

#endif /* SDL_AUDIO_DRIVER_ALSA_DYNAMIC */

static const char *get_audio_device(int channels)
{
	const char *device;
	
	device = SDL_getenv("AUDIODEV");	/* Is there a standard variable name? */
	if ( device == NULL ) {
		switch (channels) {
		case 6:
			device = "plug:surround51";
			break;
		case 4:
			device = "plug:surround40";
			break;
		default:
			device = "default";
			break;
		}
	}
	return device;
}

/* Audio driver bootstrap functions */

static int Audio_Available(void)
{
	int available;
	int status;
	snd_pcm_t *handle;

	available = 0;
	if (LoadALSALibrary() < 0) {
		return available;
	}
	status = SDL_NAME(snd_pcm_open)(&handle, get_audio_device(2), SND_PCM_STREAM_PLAYBACK, SND_PCM_NONBLOCK);
	if ( status >= 0 ) {
		available = 1;
        	SDL_NAME(snd_pcm_close)(handle);
	}
	UnloadALSALibrary();
	return(available);
}

static void Audio_DeleteDevice(SDL_AudioDevice *device)
{
	SDL_free(device->hidden);
	SDL_free(device);
	UnloadALSALibrary();
}

static SDL_AudioDevice *Audio_CreateDevice(int devindex)
{
	SDL_AudioDevice *this;

	/* Initialize all variables that we clean on shutdown */
	LoadALSALibrary();
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
	this->OpenAudio = ALSA_OpenAudio;
	this->WaitAudio = ALSA_WaitAudio;
	this->PlayAudio = ALSA_PlayAudio;
	this->GetAudioBuf = ALSA_GetAudioBuf;
	this->CloseAudio = ALSA_CloseAudio;

	this->free = Audio_DeleteDevice;

	return this;
}

AudioBootStrap ALSA_bootstrap = {
	DRIVER_NAME, "ALSA PCM audio",
	Audio_Available, Audio_CreateDevice
};

/* This function waits until it is possible to write a full sound buffer */
static void ALSA_WaitAudio(_THIS)
{
	/* We're in blocking mode, so there's nothing to do here */
}


/*
 * http://bugzilla.libsdl.org/show_bug.cgi?id=110
 * "For Linux ALSA, this is FL-FR-RL-RR-C-LFE
 *  and for Windows DirectX [and CoreAudio], this is FL-FR-C-LFE-RL-RR"
 */
#define SWIZ6(T) \
    T *ptr = (T *) mixbuf; \
    Uint32 i; \
    for (i = 0; i < this->spec.samples; i++, ptr += 6) { \
        T tmp; \
        tmp = ptr[2]; ptr[2] = ptr[4]; ptr[4] = tmp; \
        tmp = ptr[3]; ptr[3] = ptr[5]; ptr[5] = tmp; \
    }

static __inline__ void swizzle_alsa_channels_6_64bit(_THIS) { SWIZ6(Uint64); }
static __inline__ void swizzle_alsa_channels_6_32bit(_THIS) { SWIZ6(Uint32); }
static __inline__ void swizzle_alsa_channels_6_16bit(_THIS) { SWIZ6(Uint16); }
static __inline__ void swizzle_alsa_channels_6_8bit(_THIS) { SWIZ6(Uint8); }

#undef SWIZ6


/*
 * Called right before feeding this->mixbuf to the hardware. Swizzle channels
 *  from Windows/Mac order to the format alsalib will want.
 */
static __inline__ void swizzle_alsa_channels(_THIS)
{
    if (this->spec.channels == 6) {
        const Uint16 fmtsize = (this->spec.format & 0xFF); /* bits/channel. */
        if (fmtsize == 16)
            swizzle_alsa_channels_6_16bit(this);
        else if (fmtsize == 8)
            swizzle_alsa_channels_6_8bit(this);
        else if (fmtsize == 32)
            swizzle_alsa_channels_6_32bit(this);
        else if (fmtsize == 64)
            swizzle_alsa_channels_6_64bit(this);
    }

    /* !!! FIXME: update this for 7.1 if needed, later. */
}


static void ALSA_PlayAudio(_THIS)
{
	int status;
	snd_pcm_uframes_t frames_left;
	const Uint8 *sample_buf = (const Uint8 *) mixbuf;
	const int frame_size = (((int) (this->spec.format & 0xFF)) / 8) * this->spec.channels;

	swizzle_alsa_channels(this);

	frames_left = ((snd_pcm_uframes_t) this->spec.samples);

	while ( frames_left > 0 && this->enabled ) {
		/* This works, but needs more testing before going live */
		/*SDL_NAME(snd_pcm_wait)(pcm_handle, -1);*/

		status = SDL_NAME(snd_pcm_writei)(pcm_handle, sample_buf, frames_left);
		if ( status < 0 ) {
			if ( status == -EAGAIN ) {
				/* Apparently snd_pcm_recover() doesn't handle this case - does it assume snd_pcm_wait() above? */
				SDL_Delay(1);
				continue;
			}
			status = SDL_NAME(snd_pcm_recover)(pcm_handle, status, 0);
			if ( status < 0 ) {
				/* Hmm, not much we can do - abort */
				fprintf(stderr, "ALSA write failed (unrecoverable): %s\n", SDL_NAME(snd_strerror)(status));
				this->enabled = 0;
				return;
			}
			continue;
		}
		sample_buf += status * frame_size;
		frames_left -= status;
	}
}

static Uint8 *ALSA_GetAudioBuf(_THIS)
{
	return(mixbuf);
}

static void ALSA_CloseAudio(_THIS)
{
	if ( mixbuf != NULL ) {
		SDL_FreeAudioMem(mixbuf);
		mixbuf = NULL;
	}
	if ( pcm_handle ) {
		SDL_NAME(snd_pcm_drain)(pcm_handle);
		SDL_NAME(snd_pcm_close)(pcm_handle);
		pcm_handle = NULL;
	}
}

static int ALSA_finalize_hardware(_THIS, SDL_AudioSpec *spec, snd_pcm_hw_params_t *hwparams, int override)
{
	int status;
	snd_pcm_uframes_t bufsize;

	/* "set" the hardware with the desired parameters */
	status = SDL_NAME(snd_pcm_hw_params)(pcm_handle, hwparams);
	if ( status < 0 ) {
		return(-1);
	}

	/* Get samples for the actual buffer size */
	status = SDL_NAME(snd_pcm_hw_params_get_buffer_size)(hwparams, &bufsize);
	if ( status < 0 ) {
		return(-1);
	}
	if ( !override && bufsize != spec->samples * 2 ) {
		return(-1);
	}

	/* FIXME: Is this safe to do? */
	spec->samples = bufsize / 2;

	/* This is useful for debugging */
	if ( getenv("SDL_AUDIO_ALSA_DEBUG") ) {
		snd_pcm_uframes_t persize = 0;
		unsigned int periods = 0;

		SDL_NAME(snd_pcm_hw_params_get_period_size)(hwparams, &persize, NULL);
		SDL_NAME(snd_pcm_hw_params_get_periods)(hwparams, &periods, NULL);

		fprintf(stderr, "ALSA: period size = %ld, periods = %u, buffer size = %lu\n", persize, periods, bufsize);
	}
	return(0);
}

static int ALSA_set_period_size(_THIS, SDL_AudioSpec *spec, snd_pcm_hw_params_t *params, int override)
{
	const char *env;
	int status;
	snd_pcm_hw_params_t *hwparams;
	snd_pcm_uframes_t frames;
	unsigned int periods;

	/* Copy the hardware parameters for this setup */
	snd_pcm_hw_params_alloca(&hwparams);
	SDL_NAME(snd_pcm_hw_params_copy)(hwparams, params);

	if ( !override ) {
		env = getenv("SDL_AUDIO_ALSA_SET_PERIOD_SIZE");
		if ( env ) {
			override = SDL_atoi(env);
			if ( override == 0 ) {
				return(-1);
			}
		}
	}

	frames = spec->samples;
	status = SDL_NAME(snd_pcm_hw_params_set_period_size_near)(pcm_handle, hwparams, &frames, NULL);
	if ( status < 0 ) {
		return(-1);
	}

	periods = 2;
	status = SDL_NAME(snd_pcm_hw_params_set_periods_near)(pcm_handle, hwparams, &periods, NULL);
	if ( status < 0 ) {
		return(-1);
	}

	return ALSA_finalize_hardware(this, spec, hwparams, override);
}

static int ALSA_set_buffer_size(_THIS, SDL_AudioSpec *spec, snd_pcm_hw_params_t *params, int override)
{
	const char *env;
	int status;
	snd_pcm_hw_params_t *hwparams;
	snd_pcm_uframes_t frames;

	/* Copy the hardware parameters for this setup */
	snd_pcm_hw_params_alloca(&hwparams);
	SDL_NAME(snd_pcm_hw_params_copy)(hwparams, params);

	if ( !override ) {
		env = getenv("SDL_AUDIO_ALSA_SET_BUFFER_SIZE");
		if ( env ) {
			override = SDL_atoi(env);
			if ( override == 0 ) {
				return(-1);
			}
		}
	}

	frames = spec->samples * 2;
	status = SDL_NAME(snd_pcm_hw_params_set_buffer_size_near)(pcm_handle, hwparams, &frames);
	if ( status < 0 ) {
		return(-1);
	}

	return ALSA_finalize_hardware(this, spec, hwparams, override);
}

static int ALSA_OpenAudio(_THIS, SDL_AudioSpec *spec)
{
	int                  status;
	snd_pcm_hw_params_t *hwparams;
	snd_pcm_sw_params_t *swparams;
	snd_pcm_format_t     format;
	unsigned int         rate;
	unsigned int 	     channels;
	Uint16               test_format;

	/* Open the audio device */
	/* Name of device should depend on # channels in spec */
	status = SDL_NAME(snd_pcm_open)(&pcm_handle, get_audio_device(spec->channels), SND_PCM_STREAM_PLAYBACK, SND_PCM_NONBLOCK);

	if ( status < 0 ) {
		SDL_SetError("Couldn't open audio device: %s", SDL_NAME(snd_strerror)(status));
		return(-1);
	}

	/* Figure out what the hardware is capable of */
	snd_pcm_hw_params_alloca(&hwparams);
	status = SDL_NAME(snd_pcm_hw_params_any)(pcm_handle, hwparams);
	if ( status < 0 ) {
		SDL_SetError("Couldn't get hardware config: %s", SDL_NAME(snd_strerror)(status));
		ALSA_CloseAudio(this);
		return(-1);
	}

	/* SDL only uses interleaved sample output */
	status = SDL_NAME(snd_pcm_hw_params_set_access)(pcm_handle, hwparams, SND_PCM_ACCESS_RW_INTERLEAVED);
	if ( status < 0 ) {
		SDL_SetError("Couldn't set interleaved access: %s", SDL_NAME(snd_strerror)(status));
		ALSA_CloseAudio(this);
		return(-1);
	}

	/* Try for a closest match on audio format */
	status = -1;
	for ( test_format = SDL_FirstAudioFormat(spec->format);
	      test_format && (status < 0); ) {
		switch ( test_format ) {
			case AUDIO_U8:
				format = SND_PCM_FORMAT_U8;
				break;
			case AUDIO_S8:
				format = SND_PCM_FORMAT_S8;
				break;
			case AUDIO_S16LSB:
				format = SND_PCM_FORMAT_S16_LE;
				break;
			case AUDIO_S16MSB:
				format = SND_PCM_FORMAT_S16_BE;
				break;
			case AUDIO_U16LSB:
				format = SND_PCM_FORMAT_U16_LE;
				break;
			case AUDIO_U16MSB:
				format = SND_PCM_FORMAT_U16_BE;
				break;
			default:
				format = 0;
				break;
		}
		if ( format != 0 ) {
			status = SDL_NAME(snd_pcm_hw_params_set_format)(pcm_handle, hwparams, format);
		}
		if ( status < 0 ) {
			test_format = SDL_NextAudioFormat();
		}
	}
	if ( status < 0 ) {
		SDL_SetError("Couldn't find any hardware audio formats");
		ALSA_CloseAudio(this);
		return(-1);
	}
	spec->format = test_format;

	/* Set the number of channels */
	status = SDL_NAME(snd_pcm_hw_params_set_channels)(pcm_handle, hwparams, spec->channels);
	channels = spec->channels;
	if ( status < 0 ) {
		status = SDL_NAME(snd_pcm_hw_params_get_channels)(hwparams, &channels);
		if ( status < 0 ) {
			SDL_SetError("Couldn't set audio channels");
			ALSA_CloseAudio(this);
			return(-1);
		}
		spec->channels = channels;
	}

	/* Set the audio rate */
	rate = spec->freq;

	status = SDL_NAME(snd_pcm_hw_params_set_rate_near)(pcm_handle, hwparams, &rate, NULL);
	if ( status < 0 ) {
		SDL_SetError("Couldn't set audio frequency: %s", SDL_NAME(snd_strerror)(status));
		ALSA_CloseAudio(this);
		return(-1);
	}
	spec->freq = rate;

	/* Set the buffer size, in samples */
	if ( ALSA_set_period_size(this, spec, hwparams, 0) < 0 &&
	     ALSA_set_buffer_size(this, spec, hwparams, 0) < 0 ) {
		/* Failed to set desired buffer size, do the best you can... */
		if ( ALSA_set_period_size(this, spec, hwparams, 1) < 0 ) {
			SDL_SetError("Couldn't set hardware audio parameters: %s", SDL_NAME(snd_strerror)(status));
			ALSA_CloseAudio(this);
			return(-1);
		}
	}

	/* Set the software parameters */
	snd_pcm_sw_params_alloca(&swparams);
	status = SDL_NAME(snd_pcm_sw_params_current)(pcm_handle, swparams);
	if ( status < 0 ) {
		SDL_SetError("Couldn't get software config: %s", SDL_NAME(snd_strerror)(status));
		ALSA_CloseAudio(this);
		return(-1);
	}
	status = SDL_NAME(snd_pcm_sw_params_set_avail_min)(pcm_handle, swparams, spec->samples);
	if ( status < 0 ) {
		SDL_SetError("Couldn't set minimum available samples: %s", SDL_NAME(snd_strerror)(status));
		ALSA_CloseAudio(this);
		return(-1);
	}
	status = SDL_NAME(snd_pcm_sw_params_set_start_threshold)(pcm_handle, swparams, 1);
	if ( status < 0 ) {
		SDL_SetError("Couldn't set start threshold: %s", SDL_NAME(snd_strerror)(status));
		ALSA_CloseAudio(this);
		return(-1);
	}
	status = SDL_NAME(snd_pcm_sw_params)(pcm_handle, swparams);
	if ( status < 0 ) {
		SDL_SetError("Couldn't set software audio parameters: %s", SDL_NAME(snd_strerror)(status));
		ALSA_CloseAudio(this);
		return(-1);
	}

	/* Calculate the final parameters for this audio specification */
	SDL_CalculateAudioSpec(spec);

	/* Allocate mixing buffer */
	mixlen = spec->size;
	mixbuf = (Uint8 *)SDL_AllocAudioMem(mixlen);
	if ( mixbuf == NULL ) {
		ALSA_CloseAudio(this);
		return(-1);
	}
	SDL_memset(mixbuf, spec->silence, spec->size);

	/* Switch to blocking mode for playback */
	SDL_NAME(snd_pcm_nonblock)(pcm_handle, 0);

	/* We're ready to rock and roll. :-) */
	return(0);
}
