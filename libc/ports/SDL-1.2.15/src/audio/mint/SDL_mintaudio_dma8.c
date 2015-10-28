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
	using DMA 8bits (hardware access)

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

#include "SDL_mintaudio.h"
#include "SDL_mintaudio_dma8.h"

/*--- Defines ---*/

#define MINT_AUDIO_DRIVER_NAME "mint_dma8"

/* Debug print info */
#define DEBUG_NAME "audio:dma8: "
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

/*--- Audio driver functions ---*/

static void Mint_CloseAudio(_THIS);
static int Mint_OpenAudio(_THIS, SDL_AudioSpec *spec);
static void Mint_LockAudio(_THIS);
static void Mint_UnlockAudio(_THIS);

/* To check/init hardware audio */
static int Mint_CheckAudio(_THIS, SDL_AudioSpec *spec);

/* Functions called in supervisor mode */
static void Mint_InitDma(void);
static void Mint_StopReplay(void);
static void Mint_StartReplay(void);

/*--- Audio driver bootstrap functions ---*/

static int Audio_Available(void)
{
	const char *envr = SDL_getenv("SDL_AUDIODRIVER");

	/* Check if user asked a different audio driver */
	if ((envr) && (SDL_strcmp(envr, MINT_AUDIO_DRIVER_NAME)!=0)) {
		DEBUG_PRINT((DEBUG_NAME "user asked a different audio driver\n"));
		return 0;
	}

	/* Cookie _MCH present ? if not, assume ST machine */
	if (Getcookie(C__MCH, &cookie_mch) == C_NOTFOUND) {
		cookie_mch = MCH_ST;
	}

	/* Cookie _SND present ? if not, assume ST machine */
	if (Getcookie(C__SND, &cookie_snd) == C_NOTFOUND) {
		cookie_snd = SND_PSG;
	}

	/* Check if we have 8 bits audio */
	if ((cookie_snd & SND_8BIT)==0) {
		DEBUG_PRINT((DEBUG_NAME "no 8 bits sound\n"));
	    return(0);
	}

	/* Check if audio is lockable */
	if (cookie_snd & SND_16BIT) {
		if (Locksnd()!=1) {
			DEBUG_PRINT((DEBUG_NAME "audio locked by other application\n"));
			return(0);
		}

		Unlocksnd();
	}

	DEBUG_PRINT((DEBUG_NAME "8 bits audio available!\n"));
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

AudioBootStrap MINTAUDIO_DMA8_bootstrap = {
	MINT_AUDIO_DRIVER_NAME, "MiNT DMA 8 bits audio driver",
	Audio_Available, Audio_CreateDevice
};

static void Mint_LockAudio(_THIS)
{
	Supexec(Mint_StopReplay);
}

static void Mint_UnlockAudio(_THIS)
{
	Supexec(Mint_StartReplay);
}

static void Mint_CloseAudio(_THIS)
{
	Supexec(Mint_StopReplay);

	DEBUG_PRINT((DEBUG_NAME "closeaudio: replay stopped\n"));

	/* Disable interrupt */
	Jdisint(MFP_DMASOUND);

	DEBUG_PRINT((DEBUG_NAME "closeaudio: interrupt disabled\n"));

	/* Wait if currently playing sound */
	while (SDL_MintAudio_mutex != 0) {
	}

	DEBUG_PRINT((DEBUG_NAME "closeaudio: no more interrupt running\n"));

	/* Clear buffers */
	if (SDL_MintAudio_audiobuf[0]) {
		Mfree(SDL_MintAudio_audiobuf[0]);
		SDL_MintAudio_audiobuf[0] = SDL_MintAudio_audiobuf[1] = NULL;
	}

	DEBUG_PRINT((DEBUG_NAME "closeaudio: buffers freed\n"));
}

static int Mint_CheckAudio(_THIS, SDL_AudioSpec *spec)
{
	int i, masterprediv, sfreq;
	unsigned long masterclock;

	DEBUG_PRINT((DEBUG_NAME "asked: %d bits, ",spec->format & 0x00ff));
	DEBUG_PRINT(("signed=%d, ", ((spec->format & 0x8000)!=0)));
	DEBUG_PRINT(("big endian=%d, ", ((spec->format & 0x1000)!=0)));
	DEBUG_PRINT(("channels=%d, ", spec->channels));
	DEBUG_PRINT(("freq=%d\n", spec->freq));

	if (spec->channels > 2)
		spec->channels = 2;

	/* Check formats available */
	spec->format = AUDIO_S8;
	
	/* Calculate and select the closest frequency */
	sfreq=0;
	masterclock=MASTERCLOCK_STE;
	masterprediv=MASTERPREDIV_STE;
	switch(cookie_mch>>16) {
/*
		case MCH_STE:
			masterclock=MASTERCLOCK_STE;
			masterprediv=MASTERPREDIV_STE;
			break;
*/
		case MCH_TT:
			masterclock=MASTERCLOCK_TT;
			masterprediv=MASTERPREDIV_TT;
			break;
		case MCH_F30:
		case MCH_ARANYM:
			masterclock=MASTERCLOCK_FALCON1;
			masterprediv=MASTERPREDIV_FALCON;
			sfreq=1;
			break;
	}
	
	MINTAUDIO_freqcount=0;
	for (i=sfreq;i<4;i++) {
		SDL_MintAudio_AddFrequency(this, masterclock/(masterprediv*(1<<i)),
			masterclock, i-sfreq, -1);
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

	/* Set replay tracks */
	if (cookie_snd & SND_16BIT) {
		Settracks(0,0);
		Setmontracks(0);
	}

	Supexec(Mint_InitDma);

	/* Set interrupt */
	Jdisint(MFP_DMASOUND);
	Xbtimer(XB_TIMERA, 8, 1, SDL_MintAudio_Dma8Interrupt);
	Jenabint(MFP_DMASOUND);

	if (cookie_snd & SND_16BIT) {
		if (Setinterrupt(SI_TIMERA, SI_PLAY)<0) {
			DEBUG_PRINT((DEBUG_NAME "Setinterrupt() failed\n"));
		}
	}

	Supexec(Mint_StartReplay);

    return(1);	/* We don't use threaded audio */
}

/* Functions called in supervisor mode */

static void Mint_InitDma(void)
{
	unsigned long buffer;
	unsigned char mode;
	SDL_AudioDevice *this = SDL_MintAudio_device;

	Mint_StopReplay();

	/* Set buffer */
	buffer = (unsigned long) SDL_MintAudio_audiobuf[SDL_MintAudio_numbuf];
	DMAAUDIO_IO.start_high = (buffer>>16) & 255;
	DMAAUDIO_IO.start_mid = (buffer>>8) & 255;
	DMAAUDIO_IO.start_low = buffer & 255;

	buffer += SDL_MintAudio_audiosize;
	DMAAUDIO_IO.end_high = (buffer>>16) & 255;
	DMAAUDIO_IO.end_mid = (buffer>>8) & 255;
	DMAAUDIO_IO.end_low = buffer & 255;

	mode = 3-MINTAUDIO_frequencies[MINTAUDIO_numfreq].predivisor;
	if (this->spec.channels==1) {
		mode |= 1<<7;
	}
	DMAAUDIO_IO.sound_ctrl = mode;	
}

static void Mint_StopReplay(void)
{
	/* Stop replay */
	DMAAUDIO_IO.control=0;
}

static void Mint_StartReplay(void)
{
	/* Start replay */
	DMAAUDIO_IO.control=3;
}
