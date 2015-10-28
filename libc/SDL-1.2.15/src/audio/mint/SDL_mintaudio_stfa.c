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

/*
	MiNT audio driver
	using XBIOS functions (STFA driver)

	Patrice Mandin
*/

/* Mint includes */
#include <mint/osbind.h>
#include <mint/falcon.h>
#include <mint/cookie.h>

#include "SDL_audio.h"
#include "../SDL_audio_c.h"
#include "../SDL_sysaudio.h"

#include "../../video/ataricommon/SDL_atarimxalloc_c.h"
#include "../../video/ataricommon/SDL_atarisuper.h"

#include "SDL_mintaudio.h"
#include "SDL_mintaudio_stfa.h"

/*--- Defines ---*/

#define MINT_AUDIO_DRIVER_NAME "mint_stfa"

/* Debug print info */
#define DEBUG_NAME "audio:stfa: "
#if 0
#define DEBUG_PRINT(what) \
	{ \
		printf what; \
	}
#else
#define DEBUG_PRINT(what)
#endif

/*--- Static variables ---*/

static long cookie_snd, cookie_mch;
static cookie_stfa_t *cookie_stfa;

static const int freqs[16]={
	4995,	6269,	7493,	8192,
	9830,	10971,	12538,	14985,
	16384,	19819,	21943,	24576,
	30720,	32336,	43885,	49152
};

/*--- Audio driver functions ---*/

static void Mint_CloseAudio(_THIS);
static int Mint_OpenAudio(_THIS, SDL_AudioSpec *spec);
static void Mint_LockAudio(_THIS);
static void Mint_UnlockAudio(_THIS);

/* To check/init hardware audio */
static int Mint_CheckAudio(_THIS, SDL_AudioSpec *spec);
static void Mint_InitAudio(_THIS, SDL_AudioSpec *spec);

/*--- Audio driver bootstrap functions ---*/

static int Audio_Available(void)
{
	long dummy;
	const char *envr = SDL_getenv("SDL_AUDIODRIVER");

	/* Check if user asked a different audio driver */
	if ((envr) && (SDL_strcmp(envr, MINT_AUDIO_DRIVER_NAME)!=0)) {
		DEBUG_PRINT((DEBUG_NAME "user asked a different audio driver\n"));
		return(0);
	}

	/* Cookie _MCH present ? if not, assume ST machine */
	if (Getcookie(C__MCH, &cookie_mch) == C_NOTFOUND) {
		cookie_mch = MCH_ST;
	}

	/* Cookie _SND present ? if not, assume ST machine */
	if (Getcookie(C__SND, &cookie_snd) == C_NOTFOUND) {
		cookie_snd = SND_PSG;
	}

	/* Cookie STFA present ? */
	if (Getcookie(C_STFA, &dummy) != C_FOUND) {
		DEBUG_PRINT((DEBUG_NAME "no STFA audio\n"));
		return(0);
	}
	cookie_stfa = (cookie_stfa_t *) dummy;

	SDL_MintAudio_stfa = cookie_stfa;

	DEBUG_PRINT((DEBUG_NAME "STFA audio available!\n"));
	return(1);
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
    this->OpenAudio   = Mint_OpenAudio;
    this->CloseAudio  = Mint_CloseAudio;
    this->LockAudio   = Mint_LockAudio;
    this->UnlockAudio = Mint_UnlockAudio;
    this->free        = Audio_DeleteDevice;

    return this;
}

AudioBootStrap MINTAUDIO_STFA_bootstrap = {
	MINT_AUDIO_DRIVER_NAME, "MiNT STFA audio driver",
	Audio_Available, Audio_CreateDevice
};

static void Mint_LockAudio(_THIS)
{
	void *oldpile;

	/* Stop replay */
	oldpile=(void *)Super(0);
	cookie_stfa->sound_enable=STFA_PLAY_DISABLE;
	SuperToUser(oldpile);
}

static void Mint_UnlockAudio(_THIS)
{
	void *oldpile;

	/* Restart replay */
	oldpile=(void *)Super(0);
	cookie_stfa->sound_enable=STFA_PLAY_ENABLE|STFA_PLAY_REPEAT;
	SuperToUser(oldpile);
}

static void Mint_CloseAudio(_THIS)
{
	void *oldpile;

	/* Stop replay */
	oldpile=(void *)Super(0);
	cookie_stfa->sound_enable=STFA_PLAY_DISABLE;
	SuperToUser(oldpile);

	/* Wait if currently playing sound */
	while (SDL_MintAudio_mutex != 0) {
	}

	/* Clear buffers */
	if (SDL_MintAudio_audiobuf[0]) {
		Mfree(SDL_MintAudio_audiobuf[0]);
		SDL_MintAudio_audiobuf[0] = SDL_MintAudio_audiobuf[1] = NULL;
	}
}

static int Mint_CheckAudio(_THIS, SDL_AudioSpec *spec)
{
	int i;

	DEBUG_PRINT((DEBUG_NAME "asked: %d bits, ",spec->format & 0x00ff));
	DEBUG_PRINT(("signed=%d, ", ((spec->format & 0x8000)!=0)));
	DEBUG_PRINT(("big endian=%d, ", ((spec->format & 0x1000)!=0)));
	DEBUG_PRINT(("channels=%d, ", spec->channels));
	DEBUG_PRINT(("freq=%d\n", spec->freq));

    if (spec->channels > 2) {
        spec->channels = 2;  /* no more than stereo! */
    }

	/* Check formats available */
	MINTAUDIO_freqcount=0;
	for (i=0;i<16;i++) {
		SDL_MintAudio_AddFrequency(this, freqs[i], 0, i, -1);
	}

#if 1
	for (i=0; i<MINTAUDIO_freqcount; i++) {
		DEBUG_PRINT((DEBUG_NAME "freq %d: %lu Hz, clock %lu, prediv %d\n",
			i, MINTAUDIO_frequencies[i].frequency, MINTAUDIO_frequencies[i].masterclock,
			MINTAUDIO_frequencies[i].predivisor
		));
	}
#endif

	MINTAUDIO_numfreq=SDL_MintAudio_SearchFrequency(this, spec->freq);
	spec->freq=MINTAUDIO_frequencies[MINTAUDIO_numfreq].frequency;

	DEBUG_PRINT((DEBUG_NAME "obtained: %d bits, ",spec->format & 0x00ff));
	DEBUG_PRINT(("signed=%d, ", ((spec->format & 0x8000)!=0)));
	DEBUG_PRINT(("big endian=%d, ", ((spec->format & 0x1000)!=0)));
	DEBUG_PRINT(("channels=%d, ", spec->channels));
	DEBUG_PRINT(("freq=%d\n", spec->freq));

	return 0;
}

static void Mint_InitAudio(_THIS, SDL_AudioSpec *spec)
{
	void *buffer;
	void *oldpile;

	buffer = SDL_MintAudio_audiobuf[SDL_MintAudio_numbuf];

	oldpile=(void *)Super(0);

	/* Stop replay */
	cookie_stfa->sound_enable=STFA_PLAY_DISABLE;

	/* Select replay format */
	cookie_stfa->sound_control = MINTAUDIO_frequencies[MINTAUDIO_numfreq].predivisor;
	if ((spec->format & 0xff)==8) {
		cookie_stfa->sound_control |= STFA_FORMAT_8BIT;
	} else {
		cookie_stfa->sound_control |= STFA_FORMAT_16BIT;
	}
	if (spec->channels==2) {
		cookie_stfa->sound_control |= STFA_FORMAT_STEREO;
	} else {
		cookie_stfa->sound_control |= STFA_FORMAT_MONO;
	}
	if ((spec->format & 0x8000)!=0) {
		cookie_stfa->sound_control |= STFA_FORMAT_SIGNED;
	} else {
		cookie_stfa->sound_control |= STFA_FORMAT_UNSIGNED;
	}
	if ((spec->format & 0x1000)!=0) {
		cookie_stfa->sound_control |= STFA_FORMAT_BIGENDIAN;
	} else {
		cookie_stfa->sound_control |= STFA_FORMAT_LITENDIAN;
	}

	/* Set buffer */
	cookie_stfa->sound_start = (unsigned long) buffer;
	cookie_stfa->sound_end = (unsigned long) (buffer + spec->size);

	/* Set interrupt */
	cookie_stfa->stfa_it = SDL_MintAudio_StfaInterrupt;

	/* Restart replay */
	cookie_stfa->sound_enable=STFA_PLAY_ENABLE|STFA_PLAY_REPEAT;

	SuperToUser(oldpile);

	DEBUG_PRINT((DEBUG_NAME "hardware initialized\n"));
}

static int Mint_OpenAudio(_THIS, SDL_AudioSpec *spec)
{
	SDL_MintAudio_device = this;

	/* Check audio capabilities */
	if (Mint_CheckAudio(this, spec)==-1) {
		return -1;
	}

	SDL_CalculateAudioSpec(spec);

	/* Allocate memory for audio buffers in DMA-able RAM */
	DEBUG_PRINT((DEBUG_NAME "buffer size=%d\n", spec->size));

	SDL_MintAudio_audiobuf[0] = Atari_SysMalloc(spec->size *2, MX_STRAM);
	if (SDL_MintAudio_audiobuf[0]==NULL) {
		SDL_SetError("MINT_OpenAudio: Not enough memory for audio buffer");
		return (-1);
	}
	SDL_MintAudio_audiobuf[1] = SDL_MintAudio_audiobuf[0] + spec->size ;
	SDL_MintAudio_numbuf=0;
	SDL_memset(SDL_MintAudio_audiobuf[0], spec->silence, spec->size *2);
	SDL_MintAudio_audiosize = spec->size;
	SDL_MintAudio_mutex = 0;

	DEBUG_PRINT((DEBUG_NAME "buffer 0 at 0x%08x\n", SDL_MintAudio_audiobuf[0]));
	DEBUG_PRINT((DEBUG_NAME "buffer 1 at 0x%08x\n", SDL_MintAudio_audiobuf[1]));

	SDL_MintAudio_CheckFpu();

	/* Setup audio hardware */
	Mint_InitAudio(this, spec);

    return(1);	/* We don't use threaded audio */
}
