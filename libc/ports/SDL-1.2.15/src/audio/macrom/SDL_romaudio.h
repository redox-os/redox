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

#ifndef _SDL_romaudio_h
#define _SDL_romaudio_h

#include "../SDL_sysaudio.h"

/* This is Ryan's improved MacOS sound code, with locking support */
#define USE_RYANS_SOUNDCODE

/* Hidden "this" pointer for the video functions */
#define _THIS	SDL_AudioDevice *this

struct SDL_PrivateAudioData {
	/* Sound manager audio channel */
	SndChannelPtr channel;
#if defined(TARGET_API_MAC_CARBON) || defined(USE_RYANS_SOUNDCODE)
	/* FIXME: Add Ryan's static data here */
#else
	/* Double buffering variables */
	SndDoubleBufferPtr audio_buf[2];
#endif
};

/* Old variable names */
#define channel		(this->hidden->channel)
#define audio_buf	(this->hidden->audio_buf)

#endif /* _SDL_romaudio_h */
