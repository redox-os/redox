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

/* Tru64 UNIX MME support */
#include <mme_api.h>

#include "SDL_timer.h"
#include "SDL_audio.h"
#include "../SDL_audio_c.h"
#include "SDL_mmeaudio.h"

static BOOL inUse[NUM_BUFFERS];

/* Audio driver functions */
static int MME_OpenAudio(_THIS, SDL_AudioSpec *spec);
static void MME_WaitAudio(_THIS);
static Uint8 *MME_GetAudioBuf(_THIS);
static void MME_PlayAudio(_THIS);
static void MME_WaitDone(_THIS);
static void MME_CloseAudio(_THIS);

/* Audio driver bootstrap functions */
static int Audio_Available(void)
{
    return(1);
}

static void Audio_DeleteDevice(SDL_AudioDevice *device)
{
    if ( device ) {
	if ( device->hidden ) {
	    SDL_free(device->hidden);
	    device->hidden = NULL;
	}
	SDL_free(device);
	device = NULL;
    }
}

static SDL_AudioDevice *Audio_CreateDevice(int devindex)
{
    SDL_AudioDevice *this;

/* Initialize all variables that we clean on shutdown */
    this = SDL_malloc(sizeof(SDL_AudioDevice));
    if ( this ) {
	SDL_memset(this, 0, (sizeof *this));
	this->hidden = SDL_malloc((sizeof *this->hidden));
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
    this->OpenAudio       =       MME_OpenAudio;
    this->WaitAudio       =       MME_WaitAudio;
    this->PlayAudio       =       MME_PlayAudio;
    this->GetAudioBuf     =     MME_GetAudioBuf;
    this->WaitDone        =        MME_WaitDone;
    this->CloseAudio      =      MME_CloseAudio;
    this->free            =  Audio_DeleteDevice;

    return this;
}

AudioBootStrap MMEAUDIO_bootstrap = {
    "waveout", "Tru64 MME WaveOut",
    Audio_Available, Audio_CreateDevice
};

static void SetMMerror(char *function, MMRESULT code)
{
    int len;
    char errbuf[MAXERRORLENGTH];

    SDL_snprintf(errbuf, SDL_arraysize(errbuf), "%s: ", function);
    len = SDL_strlen(errbuf);
    waveOutGetErrorText(code, errbuf+len, MAXERRORLENGTH-len);
    SDL_SetError("%s",errbuf);
}

static void CALLBACK MME_CALLBACK(HWAVEOUT hwo,
				  UINT uMsg,
				  DWORD dwInstance,
				  LPARAM dwParam1,
				  LPARAM dwParam2)
{
    WAVEHDR *wp = (WAVEHDR *) dwParam1;

    if ( uMsg == WOM_DONE )
	inUse[wp->dwUser] = FALSE;
}

static int MME_OpenAudio(_THIS, SDL_AudioSpec *spec)
{
    MMRESULT result;
    int i;

    mixbuf = NULL;

    /* Set basic WAVE format parameters */
    shm = mmeAllocMem(sizeof(*shm));
    if ( shm == NULL ) {
	SDL_SetError("Out of memory: shm");
	return(-1);
    }
    shm->sound = 0;
    shm->wFmt.wf.wFormatTag = WAVE_FORMAT_PCM;

    /* Determine the audio parameters from the AudioSpec */
    switch ( spec->format & 0xFF ) {
	case 8:
	    /* Unsigned 8 bit audio data */
	    spec->format = AUDIO_U8;
	    shm->wFmt.wBitsPerSample = 8;
	    break;
	case 16:
	    /* Signed 16 bit audio data */
	    spec->format = AUDIO_S16;
	    shm->wFmt.wBitsPerSample = 16;
	    break;
	    default:
	    SDL_SetError("Unsupported audio format");
	    return(-1);
    }

    shm->wFmt.wf.nChannels = spec->channels;
    shm->wFmt.wf.nSamplesPerSec = spec->freq;
    shm->wFmt.wf.nBlockAlign =
	shm->wFmt.wf.nChannels * shm->wFmt.wBitsPerSample / 8;
    shm->wFmt.wf.nAvgBytesPerSec =
	shm->wFmt.wf.nSamplesPerSec * shm->wFmt.wf.nBlockAlign;

    /* Check the buffer size -- minimum of 1/4 second (word aligned) */
    if ( spec->samples < (spec->freq/4) )
	spec->samples = ((spec->freq/4)+3)&~3;

    /* Update the fragment size as size in bytes */
    SDL_CalculateAudioSpec(spec);

    /* Open the audio device */
    result = waveOutOpen(&(shm->sound),
			 WAVE_MAPPER,
			 &(shm->wFmt.wf),
			 MME_CALLBACK,
			 NULL,
			 (CALLBACK_FUNCTION|WAVE_OPEN_SHAREABLE));
    if ( result != MMSYSERR_NOERROR ) {
	    SetMMerror("waveOutOpen()", result);
	    return(-1);
    }

    /* Create the sound buffers */
    mixbuf = (Uint8 *)mmeAllocBuffer(NUM_BUFFERS * (spec->size));
    if ( mixbuf == NULL ) {
	SDL_SetError("Out of memory: mixbuf");
	return(-1);
    }

    for (i = 0; i < NUM_BUFFERS; i++) {
	shm->wHdr[i].lpData         = &mixbuf[i * (spec->size)];
	shm->wHdr[i].dwBufferLength = spec->size;
	shm->wHdr[i].dwFlags        = 0;
	shm->wHdr[i].dwUser         = i;
	shm->wHdr[i].dwLoops        = 0;       /* loop control counter */
	shm->wHdr[i].lpNext         = NULL;    /* reserved for driver */
	shm->wHdr[i].reserved       = 0;
	inUse[i] = FALSE;
    }
    next_buffer = 0;
    return 0;
}

static void MME_WaitAudio(_THIS)
{
    while ( inUse[next_buffer] ) {
	mmeWaitForCallbacks();
	mmeProcessCallbacks();
    }
}

static Uint8 *MME_GetAudioBuf(_THIS)
{
    Uint8 *retval;

    inUse[next_buffer] = TRUE;
    retval = (Uint8 *)(shm->wHdr[next_buffer].lpData);
    return retval;
}

static void MME_PlayAudio(_THIS)
{
    /* Queue it up */
    waveOutWrite(shm->sound, &(shm->wHdr[next_buffer]), sizeof(WAVEHDR));
    next_buffer = (next_buffer+1)%NUM_BUFFERS;
}

static void MME_WaitDone(_THIS)
{
    MMRESULT result;
    int i;

    if ( shm->sound ) {
	for (i = 0; i < NUM_BUFFERS; i++)
	    while ( inUse[i] ) {
		mmeWaitForCallbacks();
		mmeProcessCallbacks();
	    }
	result = waveOutReset(shm->sound);
	if ( result != MMSYSERR_NOERROR )
	    SetMMerror("waveOutReset()", result);
	mmeProcessCallbacks();
    }
}

static void MME_CloseAudio(_THIS)
{
    MMRESULT result;

    if ( mixbuf ) {
	result = mmeFreeBuffer(mixbuf);
	if (result != MMSYSERR_NOERROR )
	    SetMMerror("mmeFreeBuffer", result);
	mixbuf = NULL;
    }

    if ( shm ) {
	if ( shm->sound ) {
	    result = waveOutClose(shm->sound);
	    if (result != MMSYSERR_NOERROR )
		SetMMerror("waveOutClose()", result);
	    mmeProcessCallbacks();
	}
	result = mmeFreeMem(shm);
	if (result != MMSYSERR_NOERROR )
	    SetMMerror("mmeFreeMem()", result);
	shm = NULL;
    }
}

