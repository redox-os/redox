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

#ifndef _ALSA_PCM_audio_h
#define _ALSA_PCM_audio_h

#include <alsa/asoundlib.h>

#include "../SDL_sysaudio.h"

/* Hidden "this" pointer for the video functions */
#define _THIS	SDL_AudioDevice *this

struct SDL_PrivateAudioData {
	/* The audio device handle */
	snd_pcm_t *pcm_handle;

	/* Raw mixing buffer */
	Uint8 *mixbuf;
	int    mixlen;
};

/* Old variable names */
#define pcm_handle		(this->hidden->pcm_handle)
#define mixbuf			(this->hidden->mixbuf)
#define mixlen			(this->hidden->mixlen)

#endif /* _ALSA_PCM_audio_h */
