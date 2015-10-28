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
	using XBIOS functions (GSXB compatible driver)

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
#include "SDL_mintaudio_gsxb.h"

/*--- Defines ---*/

#define MINT_AUDIO_DRIVER_NAME "mint_gsxb"

/* Debug print info */
#define DEBUG_NAME "audio:gsxb: "
#if 0
#define DEBUG_PRINT(what) \
	{ \
		printf what; \
	}
#else
#define DEBUG_PRINT(what)
#endif

/*--- Static variables ---*/

static long cookie_snd, cookie_gsxb;

/*--- Audio driver functions ---*/

static void Mint_CloseAudio(_THIS);
static int Mint_OpenAudio(_THIS, SDL_AudioSpec *spec);
static void Mint_LockAudio(_THIS);
static void Mint_UnlockAudio(_THIS);

/* To check/init hardware audio */
static int Mint_CheckAudio(_THIS, SDL_AudioSpec *spec);
static void Mint_InitAudio(_THIS, SDL_AudioSpec *spec);

/* GSXB callbacks */
static void Mint_GsxbInterrupt(void);
static void Mint_GsxbNullInterrupt(void);

/*--- Audio driver bootstrap functions ---*/

static int Audio_Available(void)
{
	const char *envr = SDL_getenv("SDL_AUDIODRIVER");

	/* Check if user asked a different audio driver */
	if ((envr) && (SDL_strcmp(envr, MINT_AUDIO_DRIVER_NAME)!=0)) {
		DEBUG_PRINT((DEBUG_NAME "user asked a different audio driver\n"));
		return(0);
	}

	/* Cookie _SND present ? if not, assume ST machine */
	if (Getcookie(C__SND, &cookie_snd) == C_NOTFOUND) {
		cookie_snd = SND_PSG;
	}

	/* Check if we have 16 bits audio */
	if ((cookie_snd & SND_16BIT)==0) {
		DEBUG_PRINT((DEBUG_NAME "no 16 bits sound\n"));
	    return(0);
	}

	/* Cookie GSXB present ? */
	cookie_gsxb = (Getcookie(C_GSXB, &cookie_gsxb) == C_FOUND);

	/* Is it GSXB ? */
	if (((cookie_snd & SND_GSXB)==0) || (cookie_gsxb==0)) {
		DEBUG_PRINT((DEBUG_NAME "no GSXB audio\n"));
		return(0);
	}

	/* Check if audio is lockable */
	if (Locksnd()!=1) {
		DEBUG_PRINT((DEBUG_NAME "audio locked by other application\n"));
		return(0);
	}

	Unlocksnd();

	DEBUG_PRINT((DEBUG_NAME "GSXB audio available!\n"));
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

AudioBootStrap MINTAUDIO_GSXB_bootstrap = {
	MINT_AUDIO_DRIVER_NAME, "MiNT GSXB audio driver",
	Audio_Available, Audio_CreateDevice
};

static void Mint_LockAudio(_THIS)
{
	/* Stop replay */
	Buffoper(0);
}

static void Mint_UnlockAudio(_THIS)
{
	/* Restart replay */
	Buffoper(SB_PLA_ENA|SB_PLA_RPT);
}

static void Mint_CloseAudio(_THIS)
{
	/* Stop replay */
	Buffoper(0);

	/* Uninstall interrupt */
	if (NSetinterrupt(2, SI_NONE, Mint_GsxbNullInterrupt)<0) {
		DEBUG_PRINT((DEBUG_NAME "NSetinterrupt() failed in close\n"));
	}

	/* Wait if currently playing sound */
	while (SDL_MintAudio_mutex != 0) {
	}

	/* Clear buffers */
	if (SDL_MintAudio_audiobuf[0]) {
		Mfree(SDL_MintAudio_audiobuf[0]);
		SDL_MintAudio_audiobuf[0] = SDL_MintAudio_audiobuf[1] = NULL;
	}

	/* Unlock sound system */
	Unlocksnd();
}

static int Mint_CheckAudio(_THIS, SDL_AudioSpec *spec)
{
	long snd_format = 0;
	int i, resolution, format_signed, format_bigendian;
    Uint16 test_format = SDL_FirstAudioFormat(spec->format);
    int valid_datatype = 0;

	resolution = spec->format & 0x00ff;
	format_signed = ((spec->format & 0x8000)!=0);
	format_bigendian = ((spec->format & 0x1000)!=0);

	DEBUG_PRINT((DEBUG_NAME "asked: %d bits, ",spec->format & 0x00ff));
	DEBUG_PRINT(("signed=%d, ", ((spec->format & 0x8000)!=0)));
	DEBUG_PRINT(("big endian=%d, ", ((spec->format & 0x1000)!=0)));
	DEBUG_PRINT(("channels=%d, ", spec->channels));
	DEBUG_PRINT(("freq=%d\n", spec->freq));

    if (spec->channels > 2) {
        spec->channels = 2;  /* no more than stereo! */
    }

    while ((!valid_datatype) && (test_format)) {
        /* Check formats available */
        snd_format = Sndstatus(SND_QUERYFORMATS);
        spec->format = test_format;
        resolution = spec->format & 0xff;
        format_signed = (spec->format & (1<<15));
        format_bigendian = (spec->format & (1<<12));
        switch (test_format) {
            case AUDIO_U8:
            case AUDIO_S8:
                if (snd_format & SND_FORMAT8) {
                    valid_datatype = 1;
                    snd_format = Sndstatus(SND_QUERY8BIT);
                }
                break;

            case AUDIO_U16LSB:
            case AUDIO_S16LSB:
            case AUDIO_U16MSB:
            case AUDIO_S16MSB:
                if (snd_format & SND_FORMAT16) {
                    valid_datatype = 1;
                    snd_format = Sndstatus(SND_QUERY16BIT);
                }
                break;

            default:
                test_format = SDL_NextAudioFormat();
                break;
        }
    }

    if (!valid_datatype) {
        SDL_SetError("Unsupported audio format");
        return (-1);
    }

	/* Check signed/unsigned format */
	if (format_signed) {
		if (snd_format & SND_FORMATSIGNED) {
			/* Ok */
		} else if (snd_format & SND_FORMATUNSIGNED) {
			/* Give unsigned format */
			spec->format = spec->format & (~0x8000);
		}
	} else {
		if (snd_format & SND_FORMATUNSIGNED) {
			/* Ok */
		} else if (snd_format & SND_FORMATSIGNED) {
			/* Give signed format */
			spec->format |= 0x8000;
		}
	}

	if (format_bigendian) {
		if (snd_format & SND_FORMATBIGENDIAN) {
			/* Ok */
		} else if (snd_format & SND_FORMATLITTLEENDIAN) {
			/* Give little endian format */
			spec->format = spec->format & (~0x1000);
		}
	} else {
		if (snd_format & SND_FORMATLITTLEENDIAN) {
			/* Ok */
		} else if (snd_format & SND_FORMATBIGENDIAN) {
			/* Give big endian format */
			spec->format |= 0x1000;
		}
	}
	
	/* Calculate and select the closest frequency */
	MINTAUDIO_freqcount=0;
	for (i=1;i<4;i++) {
		SDL_MintAudio_AddFrequency(this,
			MASTERCLOCK_44K/(MASTERPREDIV_MILAN*(1<<i)), MASTERCLOCK_44K,
			(1<<i)-1, -1);
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
	int channels_mode, prediv;
	void *buffer;

	/* Stop currently playing sound */
	Buffoper(0);

	/* Set replay tracks */
	Settracks(0,0);
	Setmontracks(0);

	/* Select replay format */
	switch (spec->format & 0xff) {
		case 8:
			if (spec->channels==2) {
				channels_mode=STEREO8;
			} else {
				channels_mode=MONO8;
			}
			break;
		case 16:
			if (spec->channels==2) {
				channels_mode=STEREO16;
			} else {
				channels_mode=MONO16;
			}
			break;
		default:
			channels_mode=STEREO16;
			break;
	}
	if (Setmode(channels_mode)<0) {
		DEBUG_PRINT((DEBUG_NAME "Setmode() failed\n"));
	}

	prediv = MINTAUDIO_frequencies[MINTAUDIO_numfreq].predivisor;
	Devconnect(DMAPLAY, DAC, CLKEXT, prediv, 1);

	/* Set buffer */
	buffer = SDL_MintAudio_audiobuf[SDL_MintAudio_numbuf];
	if (Setbuffer(0, buffer, buffer + spec->size)<0) {
		DEBUG_PRINT((DEBUG_NAME "Setbuffer() failed\n"));
	}
	
	/* Install interrupt */
	if (NSetinterrupt(2, SI_PLAY, Mint_GsxbInterrupt)<0) {
		DEBUG_PRINT((DEBUG_NAME "NSetinterrupt() failed\n"));
	}

	/* Go */
	Buffoper(SB_PLA_ENA|SB_PLA_RPT);
	DEBUG_PRINT((DEBUG_NAME "hardware initialized\n"));
}

static int Mint_OpenAudio(_THIS, SDL_AudioSpec *spec)
{
	/* Lock sound system */
	if (Locksnd()!=1) {
   	    SDL_SetError("Mint_OpenAudio: Audio system already in use");
        return(-1);
	}

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

static void Mint_GsxbInterrupt(void)
{
	Uint8 *newbuf;

	if (SDL_MintAudio_mutex)
		return;

	SDL_MintAudio_mutex=1;

	SDL_MintAudio_numbuf ^= 1;
	SDL_MintAudio_Callback();
	newbuf = SDL_MintAudio_audiobuf[SDL_MintAudio_numbuf];
	Setbuffer(0, newbuf, newbuf + SDL_MintAudio_audiosize);

	SDL_MintAudio_mutex=0;
}

static void Mint_GsxbNullInterrupt(void)
{
}
