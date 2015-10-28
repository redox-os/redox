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

	Patrice Mandin
*/

#ifndef _SDL_mintaudio_h
#define _SDL_mintaudio_h

#include "../SDL_sysaudio.h"
#include "SDL_mintaudio_stfa.h"

/* Hidden "this" pointer for the audio functions */
#define _THIS	SDL_AudioDevice *this

/* 16 predivisors with 3 clocks max. */
#define MINTAUDIO_maxfreqs		(16*3)		

typedef struct {
	Uint32	frequency;
	Uint32	masterclock;
	Uint32	predivisor;
	int	gpio_bits;	/* in case of external clock */
} mint_frequency_t;

struct SDL_PrivateAudioData {
	mint_frequency_t	frequencies[MINTAUDIO_maxfreqs];
	int 	freq_count;		/* Number of frequencies in the array */
	int		numfreq;		/* Number of selected frequency */
};

/* Old variable names */

#define MINTAUDIO_frequencies	(this->hidden->frequencies)
#define MINTAUDIO_freqcount		(this->hidden->freq_count)
#define MINTAUDIO_numfreq		(this->hidden->numfreq)

/* _MCH cookie (values>>16) */
enum {
	MCH_ST=0,
	MCH_STE,
	MCH_TT,
	MCH_F30,
	MCH_CLONE,
	MCH_ARANYM
};

/* Master clocks for replay frequencies */
#define MASTERCLOCK_STE		8010666		/* Not sure of this one */
#define MASTERCLOCK_TT		16107953	/* Not sure of this one */
#define MASTERCLOCK_FALCON1	25175000
#define MASTERCLOCK_FALCON2	32000000	/* Only usable for DSP56K */
#define MASTERCLOCK_FALCONEXT	-1		/* Clock on DSP56K port, unknown */
#define MASTERCLOCK_44K		22579200	/* Standard clock for 44.1 Khz */
#define MASTERCLOCK_48K		24576000	/* Standard clock for 48 Khz */

/* Master clock predivisors */
#define MASTERPREDIV_STE	160
#define MASTERPREDIV_TT		320
#define MASTERPREDIV_FALCON	256
#define MASTERPREDIV_MILAN	256

/* Variables */
extern SDL_AudioDevice *SDL_MintAudio_device;
extern Uint8 *SDL_MintAudio_audiobuf[2];	/* Pointers to buffers */
extern unsigned long SDL_MintAudio_audiosize;		/* Length of audio buffer=spec->size */
extern volatile unsigned short SDL_MintAudio_numbuf;		/* Buffer to play */
extern volatile unsigned short SDL_MintAudio_mutex;
extern cookie_stfa_t *SDL_MintAudio_stfa;
extern volatile unsigned long SDL_MintAudio_clocktics;
extern unsigned short SDL_MintAudio_hasfpu;	/* To preserve fpu registers if needed */

/* MiNT thread variables */
extern SDL_bool	SDL_MintAudio_mint_present;
extern SDL_bool SDL_MintAudio_quit_thread;
extern SDL_bool SDL_MintAudio_thread_finished;
extern long SDL_MintAudio_thread_pid;

/* Functions */
void SDL_MintAudio_Callback(void);
void SDL_MintAudio_AddFrequency(_THIS, Uint32 frequency, Uint32 clock,
	Uint32 prediv, int gpio_bits);
int SDL_MintAudio_SearchFrequency(_THIS, int desired_freq);
void SDL_MintAudio_CheckFpu(void);

/* MiNT thread functions */
int SDL_MintAudio_Thread(long param);
void SDL_MintAudio_WaitThread(void);

/* ASM interrupt functions */
void SDL_MintAudio_GsxbInterrupt(void);
void SDL_MintAudio_EmptyGsxbInterrupt(void);
void SDL_MintAudio_XbiosInterruptMeasureClock(void);
void SDL_MintAudio_XbiosInterrupt(void);
void SDL_MintAudio_Dma8Interrupt(void);
void SDL_MintAudio_StfaInterrupt(void);

#endif /* _SDL_mintaudio_h */
