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

#ifndef __SDL_NTO_AUDIO_H__
#define __SDL_NTO_AUDIO_H__

#include <sys/asoundlib.h>

#include "../SDL_sysaudio.h"

/* Hidden "this" pointer for the audio functions */
#define _THIS	SDL_AudioDevice *this

struct SDL_PrivateAudioData
{
    /* The audio device handle */
    int cardno;
    int deviceno;
    snd_pcm_t* audio_handle;

    /* The audio file descriptor */
    int audio_fd;

    /* The parent process id, to detect when application quits */
    pid_t parent;

    /* Raw mixing buffer */
    Uint8* pcm_buf;
    Uint32 pcm_len;

    /* QSA parameters */
    snd_pcm_channel_status_t cstatus;
    snd_pcm_channel_params_t cparams;
    snd_pcm_channel_setup_t  csetup;
};

#define cardno          (this->hidden->cardno)
#define deviceno        (this->hidden->deviceno)
#define audio_handle    (this->hidden->audio_handle)
#define audio_fd        (this->hidden->audio_fd)
#define parent          (this->hidden->parent)
#define pcm_buf         (this->hidden->pcm_buf)
#define pcm_len         (this->hidden->pcm_len)
#define cstatus         (this->hidden->cstatus)
#define cparams         (this->hidden->cparams)
#define csetup          (this->hidden->csetup)

#endif /* __SDL_NTO_AUDIO_H__ */
