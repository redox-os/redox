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

    Stéphan Kochen
    stephan@kochen.nl

    Based on parts of the ALSA and ESounD output drivers.
*/
#include "SDL_config.h"

/* Allow access to an PulseAudio network stream mixing buffer */

#include <sys/types.h>
#include <unistd.h>
#include <signal.h>
#include <errno.h>
#include <pulse/pulseaudio.h>
#include <pulse/simple.h>

#include "SDL_timer.h"
#include "SDL_audio.h"
#include "../SDL_audiomem.h"
#include "../SDL_audio_c.h"
#include "../SDL_audiodev_c.h"
#include "../../../include/SDL_video.h"  /* for SDL_WM_GetCaption(). */
#include "SDL_pulseaudio.h"

#ifdef SDL_AUDIO_DRIVER_PULSE_DYNAMIC
#include "SDL_name.h"
#include "SDL_loadso.h"
#else
#define SDL_NAME(X)	X
#endif

/* The tag name used by the driver */
#define PULSE_DRIVER_NAME	"pulse"

/* Audio driver functions */
static int PULSE_OpenAudio(_THIS, SDL_AudioSpec *spec);
static void PULSE_WaitAudio(_THIS);
static void PULSE_PlayAudio(_THIS);
static Uint8 *PULSE_GetAudioBuf(_THIS);
static void PULSE_CloseAudio(_THIS);
static void PULSE_WaitDone(_THIS);
static void PULSE_SetCaption(_THIS, const char *str);

#ifdef SDL_AUDIO_DRIVER_PULSE_DYNAMIC

static const char *pulse_library = SDL_AUDIO_DRIVER_PULSE_DYNAMIC;
static void *pulse_handle = NULL;
static int pulse_loaded = 0;

static pa_simple* (*SDL_NAME(pa_simple_new))(
	const char *server,
	const char *name,
	pa_stream_direction_t dir,
	const char *dev,
	const char *stream_name,
	const pa_sample_spec *ss,
	const pa_channel_map *map,
	const pa_buffer_attr *attr,
	int *error
);
static void (*SDL_NAME(pa_simple_free))(pa_simple *s);

static pa_channel_map* (*SDL_NAME(pa_channel_map_init_auto))(
	pa_channel_map *m,
	unsigned channels,
	pa_channel_map_def_t def
);

static pa_mainloop * (*SDL_NAME(pa_mainloop_new))(void);
static pa_mainloop_api * (*SDL_NAME(pa_mainloop_get_api))(pa_mainloop *m);
static int (*SDL_NAME(pa_mainloop_iterate))(pa_mainloop *m, int block, int *retval);
static void (*SDL_NAME(pa_mainloop_free))(pa_mainloop *m);

static pa_operation_state_t (*SDL_NAME(pa_operation_get_state))(pa_operation *o);
static void (*SDL_NAME(pa_operation_cancel))(pa_operation *o);
static void (*SDL_NAME(pa_operation_unref))(pa_operation *o);

static pa_context * (*SDL_NAME(pa_context_new))(
	pa_mainloop_api *m, const char *name);
static int (*SDL_NAME(pa_context_connect))(
	pa_context *c, const char *server,
	pa_context_flags_t flags, const pa_spawn_api *api);
static pa_context_state_t (*SDL_NAME(pa_context_get_state))(pa_context *c);
static void (*SDL_NAME(pa_context_disconnect))(pa_context *c);
static void (*SDL_NAME(pa_context_unref))(pa_context *c);

static pa_stream * (*SDL_NAME(pa_stream_new))(pa_context *c,
	const char *name, const pa_sample_spec *ss, const pa_channel_map *map);
static int (*SDL_NAME(pa_stream_connect_playback))(pa_stream *s, const char *dev,
	const pa_buffer_attr *attr, pa_stream_flags_t flags,
	pa_cvolume *volume, pa_stream *sync_stream);
static pa_stream_state_t (*SDL_NAME(pa_stream_get_state))(pa_stream *s);
static size_t (*SDL_NAME(pa_stream_writable_size))(pa_stream *s);
static int (*SDL_NAME(pa_stream_write))(pa_stream *s, const void *data, size_t nbytes,
	pa_free_cb_t free_cb, int64_t offset, pa_seek_mode_t seek);
static pa_operation * (*SDL_NAME(pa_stream_drain))(pa_stream *s,
	pa_stream_success_cb_t cb, void *userdata);
static int (*SDL_NAME(pa_stream_disconnect))(pa_stream *s);
static void (*SDL_NAME(pa_stream_unref))(pa_stream *s);
static pa_operation* (*SDL_NAME(pa_context_set_name))(pa_context *c,
	const char *name, pa_context_success_cb_t cb, void *userdata);

static struct {
	const char *name;
	void **func;
} pulse_functions[] = {
	{ "pa_simple_new",
		(void **)&SDL_NAME(pa_simple_new)		},
	{ "pa_simple_free",
		(void **)&SDL_NAME(pa_simple_free)		},
	{ "pa_channel_map_init_auto",
		(void **)&SDL_NAME(pa_channel_map_init_auto)	},
	{ "pa_mainloop_new",
		(void **)&SDL_NAME(pa_mainloop_new)		},
	{ "pa_mainloop_get_api",
		(void **)&SDL_NAME(pa_mainloop_get_api)		},
	{ "pa_mainloop_iterate",
		(void **)&SDL_NAME(pa_mainloop_iterate)		},
	{ "pa_mainloop_free",
		(void **)&SDL_NAME(pa_mainloop_free)		},
	{ "pa_operation_get_state",
		(void **)&SDL_NAME(pa_operation_get_state)	},
	{ "pa_operation_cancel",
		(void **)&SDL_NAME(pa_operation_cancel)		},
	{ "pa_operation_unref",
		(void **)&SDL_NAME(pa_operation_unref)		},
	{ "pa_context_new",
		(void **)&SDL_NAME(pa_context_new)		},
	{ "pa_context_connect",
		(void **)&SDL_NAME(pa_context_connect)		},
	{ "pa_context_get_state",
		(void **)&SDL_NAME(pa_context_get_state)	},
	{ "pa_context_disconnect",
		(void **)&SDL_NAME(pa_context_disconnect)	},
	{ "pa_context_unref",
		(void **)&SDL_NAME(pa_context_unref)		},
	{ "pa_stream_new",
		(void **)&SDL_NAME(pa_stream_new)		},
	{ "pa_stream_connect_playback",
		(void **)&SDL_NAME(pa_stream_connect_playback)	},
	{ "pa_stream_get_state",
		(void **)&SDL_NAME(pa_stream_get_state)		},
	{ "pa_stream_writable_size",
		(void **)&SDL_NAME(pa_stream_writable_size)	},
	{ "pa_stream_write",
		(void **)&SDL_NAME(pa_stream_write)		},
	{ "pa_stream_drain",
		(void **)&SDL_NAME(pa_stream_drain)		},
	{ "pa_stream_disconnect",
		(void **)&SDL_NAME(pa_stream_disconnect)	},
	{ "pa_stream_unref",
		(void **)&SDL_NAME(pa_stream_unref)		},
	{ "pa_context_set_name",
		(void **)&SDL_NAME(pa_context_set_name)		},
};

static void UnloadPulseLibrary()
{
	if ( pulse_loaded ) {
		SDL_UnloadObject(pulse_handle);
		pulse_handle = NULL;
		pulse_loaded = 0;
	}
}

static int LoadPulseLibrary(void)
{
	int i, retval = -1;

	pulse_handle = SDL_LoadObject(pulse_library);
	if ( pulse_handle ) {
		pulse_loaded = 1;
		retval = 0;
		for ( i=0; i<SDL_arraysize(pulse_functions); ++i ) {
			*pulse_functions[i].func = SDL_LoadFunction(pulse_handle, pulse_functions[i].name);
			if ( !*pulse_functions[i].func ) {
				retval = -1;
				UnloadPulseLibrary();
				break;
			}
		}
	}
	return retval;
}

#else

static void UnloadPulseLibrary()
{
	return;
}

static int LoadPulseLibrary(void)
{
	return 0;
}

#endif /* SDL_AUDIO_DRIVER_PULSE_DYNAMIC */

/* Audio driver bootstrap functions */

static int Audio_Available(void)
{
	pa_sample_spec paspec;
	pa_simple *connection;
	int available;

	available = 0;
	if ( LoadPulseLibrary() < 0 ) {
		return available;
	}

	/* Connect with a dummy format. */
	paspec.format = PA_SAMPLE_U8;
	paspec.rate = 11025;
	paspec.channels = 1;
	connection = SDL_NAME(pa_simple_new)(
		NULL,                        /* server */
		"Test stream",               /* application name */
		PA_STREAM_PLAYBACK,          /* playback mode */
		NULL,                        /* device on the server */
		"Simple DirectMedia Layer",  /* stream description */
		&paspec,                     /* sample format spec */
		NULL,                        /* channel map */
		NULL,                        /* buffering attributes */
		NULL                         /* error code */
	);
	if ( connection != NULL ) {
		available = 1;
		SDL_NAME(pa_simple_free)(connection);
	}

	UnloadPulseLibrary();
	return(available);
}

static void Audio_DeleteDevice(SDL_AudioDevice *device)
{
	SDL_free(device->hidden->caption);
	SDL_free(device->hidden);
	SDL_free(device);
	UnloadPulseLibrary();
}

static SDL_AudioDevice *Audio_CreateDevice(int devindex)
{
	SDL_AudioDevice *this;

	/* Initialize all variables that we clean on shutdown */
	LoadPulseLibrary();
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
	this->OpenAudio = PULSE_OpenAudio;
	this->WaitAudio = PULSE_WaitAudio;
	this->PlayAudio = PULSE_PlayAudio;
	this->GetAudioBuf = PULSE_GetAudioBuf;
	this->CloseAudio = PULSE_CloseAudio;
	this->WaitDone = PULSE_WaitDone;
	this->SetCaption = PULSE_SetCaption;

	this->free = Audio_DeleteDevice;

	return this;
}

AudioBootStrap PULSE_bootstrap = {
	PULSE_DRIVER_NAME, "PulseAudio",
	Audio_Available, Audio_CreateDevice
};

/* This function waits until it is possible to write a full sound buffer */
static void PULSE_WaitAudio(_THIS)
{
	int size;
	while(1) {
		if (SDL_NAME(pa_context_get_state)(context) != PA_CONTEXT_READY ||
		    SDL_NAME(pa_stream_get_state)(stream) != PA_STREAM_READY ||
		    SDL_NAME(pa_mainloop_iterate)(mainloop, 1, NULL) < 0) {
			this->enabled = 0;
			return;
		}
		size = SDL_NAME(pa_stream_writable_size)(stream);
		if (size >= mixlen)
			return;
	}
}

static void PULSE_PlayAudio(_THIS)
{
	/* Write the audio data */
	if (SDL_NAME(pa_stream_write)(stream, mixbuf, mixlen, NULL, 0LL, PA_SEEK_RELATIVE) < 0)
		this->enabled = 0;
}

static Uint8 *PULSE_GetAudioBuf(_THIS)
{
	return(mixbuf);
}

static void PULSE_CloseAudio(_THIS)
{
	if ( mixbuf != NULL ) {
		SDL_FreeAudioMem(mixbuf);
		mixbuf = NULL;
	}
	if ( stream != NULL ) {
		SDL_NAME(pa_stream_disconnect)(stream);
		SDL_NAME(pa_stream_unref)(stream);
		stream = NULL;
	}
	if (context != NULL) {
		SDL_NAME(pa_context_disconnect)(context);
		SDL_NAME(pa_context_unref)(context);
		context = NULL;
	}
	if (mainloop != NULL) {
		SDL_NAME(pa_mainloop_free)(mainloop);
		mainloop = NULL;
	}
}

/* Try to get the name of the program */
static char *get_progname(void)
{
#ifdef __LINUX__
	char *progname = NULL;
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
	return(progname);
#elif defined(__NetBSD__)
	return getprogname();
#else
	return("unknown");
#endif
}

static void caption_set_complete(pa_context *c, int success, void *userdata)
{
	/* no-op. */
}

static void PULSE_SetCaption(_THIS, const char *str)
{
	SDL_free(this->hidden->caption);
	if ((str == NULL) || (*str == '\0')) {
		str = get_progname();  /* set a default so SOMETHING shows up. */
	}
	this->hidden->caption = SDL_strdup(str);
	if (context != NULL) {
		SDL_NAME(pa_context_set_name)(context, this->hidden->caption,
		                              caption_set_complete, 0);
	}
}

static void stream_drain_complete(pa_stream *s, int success, void *userdata)
{
	/* no-op. */
}

static void PULSE_WaitDone(_THIS)
{
	pa_operation *o;

	o = SDL_NAME(pa_stream_drain)(stream, stream_drain_complete, NULL);
	if (!o)
		return;

	while (SDL_NAME(pa_operation_get_state)(o) != PA_OPERATION_DONE) {
		if (SDL_NAME(pa_context_get_state)(context) != PA_CONTEXT_READY ||
		    SDL_NAME(pa_stream_get_state)(stream) != PA_STREAM_READY ||
		    SDL_NAME(pa_mainloop_iterate)(mainloop, 1, NULL) < 0) {
			SDL_NAME(pa_operation_cancel)(o);
			break;
		}
	}
	SDL_NAME(pa_operation_unref)(o);
}

static int PULSE_OpenAudio(_THIS, SDL_AudioSpec *spec)
{
	int             state;
	Uint16          test_format;
	pa_sample_spec  paspec;
	pa_buffer_attr  paattr;
	pa_channel_map  pacmap;
	pa_stream_flags_t flags = 0;

	paspec.format = PA_SAMPLE_INVALID;
	for ( test_format = SDL_FirstAudioFormat(spec->format); test_format; ) {
		switch ( test_format ) {
			case AUDIO_U8:
				paspec.format = PA_SAMPLE_U8;
				break;
			case AUDIO_S16LSB:
				paspec.format = PA_SAMPLE_S16LE;
				break;
			case AUDIO_S16MSB:
				paspec.format = PA_SAMPLE_S16BE;
				break;
		}
		if ( paspec.format != PA_SAMPLE_INVALID )
			break;
		test_format = SDL_NextAudioFormat();
	}
	if (paspec.format == PA_SAMPLE_INVALID ) {
		SDL_SetError("Couldn't find any suitable audio formats");
		return(-1);
	}
	spec->format = test_format;

	paspec.channels = spec->channels;
	paspec.rate = spec->freq;

	/* Calculate the final parameters for this audio specification */
#ifdef PA_STREAM_ADJUST_LATENCY
	spec->samples /= 2; /* Mix in smaller chunck to avoid underruns */
#endif
	SDL_CalculateAudioSpec(spec);

	/* Allocate mixing buffer */
	mixlen = spec->size;
	mixbuf = (Uint8 *)SDL_AllocAudioMem(mixlen);
	if ( mixbuf == NULL ) {
		return(-1);
	}
	SDL_memset(mixbuf, spec->silence, spec->size);

	/* Reduced prebuffering compared to the defaults. */
#ifdef PA_STREAM_ADJUST_LATENCY
	paattr.tlength = mixlen * 4; /* 2x original requested bufsize */
	paattr.prebuf = -1;
	paattr.maxlength = -1;
	paattr.minreq = mixlen; /* -1 can lead to pa_stream_writable_size()
				   >= mixlen never becoming true */
	flags = PA_STREAM_ADJUST_LATENCY;
#else
	paattr.tlength = mixlen*2;
	paattr.prebuf = mixlen*2;
	paattr.maxlength = mixlen*2;
	paattr.minreq = mixlen;
#endif

	/* The SDL ALSA output hints us that we use Windows' channel mapping */
	/* http://bugzilla.libsdl.org/show_bug.cgi?id=110 */
	SDL_NAME(pa_channel_map_init_auto)(
		&pacmap, spec->channels, PA_CHANNEL_MAP_WAVEEX);

	/* Set up a new main loop */
	if (!(mainloop = SDL_NAME(pa_mainloop_new)())) {
		PULSE_CloseAudio(this);
		SDL_SetError("pa_mainloop_new() failed");
		return(-1);
	}

	if (this->hidden->caption == NULL) {
		char *title = NULL;
		SDL_WM_GetCaption(&title, NULL);
		PULSE_SetCaption(this, title);
	}

	mainloop_api = SDL_NAME(pa_mainloop_get_api)(mainloop);
	if (!(context = SDL_NAME(pa_context_new)(mainloop_api,
	                                         this->hidden->caption))) {
		PULSE_CloseAudio(this);
		SDL_SetError("pa_context_new() failed");
		return(-1);
	}

	/* Connect to the PulseAudio server */
	if (SDL_NAME(pa_context_connect)(context, NULL, 0, NULL) < 0) {
		PULSE_CloseAudio(this);
		SDL_SetError("Could not setup connection to PulseAudio");
		return(-1);
	}

	do {
		if (SDL_NAME(pa_mainloop_iterate)(mainloop, 1, NULL) < 0) {
			PULSE_CloseAudio(this);
			SDL_SetError("pa_mainloop_iterate() failed");
			return(-1);
		}
		state = SDL_NAME(pa_context_get_state)(context);
		if (!PA_CONTEXT_IS_GOOD(state)) {
			PULSE_CloseAudio(this);
			SDL_SetError("Could not connect to PulseAudio");
			return(-1);
		}
	} while (state != PA_CONTEXT_READY);

	stream = SDL_NAME(pa_stream_new)(
		context,
		"Simple DirectMedia Layer",  /* stream description */
		&paspec,                     /* sample format spec */
		&pacmap                      /* channel map */
	);
	if ( stream == NULL ) {
		PULSE_CloseAudio(this);
		SDL_SetError("Could not setup PulseAudio stream");
		return(-1);
	}

	if (SDL_NAME(pa_stream_connect_playback)(stream, NULL, &paattr, flags,
			NULL, NULL) < 0) {
		PULSE_CloseAudio(this);
		SDL_SetError("Could not connect PulseAudio stream");
		return(-1);
	}

	do {
		if (SDL_NAME(pa_mainloop_iterate)(mainloop, 1, NULL) < 0) {
			PULSE_CloseAudio(this);
			SDL_SetError("pa_mainloop_iterate() failed");
			return(-1);
		}
		state = SDL_NAME(pa_stream_get_state)(stream);
		if (!PA_STREAM_IS_GOOD(state)) {
			PULSE_CloseAudio(this);
			SDL_SetError("Could not create to PulseAudio stream");
			return(-1);
		}
	} while (state != PA_STREAM_READY);

	return(0);
}
