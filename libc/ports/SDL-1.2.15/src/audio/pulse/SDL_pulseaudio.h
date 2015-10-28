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

#ifndef _SDL_pulseaudio_h
#define _SDL_pulseaudio_h

#include "../SDL_sysaudio.h"

/* Hidden "this" pointer for the video functions */
#define _THIS	SDL_AudioDevice *this

struct SDL_PrivateAudioData {
	pa_mainloop *mainloop;
	pa_mainloop_api *mainloop_api;
	pa_context *context;
	pa_stream *stream;

	char *caption;

	/* Raw mixing buffer */
	Uint8 *mixbuf;
	int    mixlen;
};

#if (PA_API_VERSION < 12)
/** Return non-zero if the passed state is one of the connected states */
static inline int PA_CONTEXT_IS_GOOD(pa_context_state_t x) {
    return
        x == PA_CONTEXT_CONNECTING ||
        x == PA_CONTEXT_AUTHORIZING ||
        x == PA_CONTEXT_SETTING_NAME ||
        x == PA_CONTEXT_READY;
}
/** Return non-zero if the passed state is one of the connected states */
static inline int PA_STREAM_IS_GOOD(pa_stream_state_t x) {
    return
        x == PA_STREAM_CREATING ||
        x == PA_STREAM_READY;
}
#endif	/* pulseaudio <= 0.9.10 */

/* Old variable names */
#define mainloop		(this->hidden->mainloop)
#define mainloop_api		(this->hidden->mainloop_api)
#define context			(this->hidden->context)
#define stream			(this->hidden->stream)
#define mixbuf			(this->hidden->mixbuf)
#define mixlen			(this->hidden->mixlen)

#endif /* _SDL_pulseaudio_h */

