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

    Carsten Griwodz
    griff@kom.tu-darmstadt.de

    based on linux/SDL_dspaudio.c by Sam Lantinga
*/
#include "SDL_config.h"

/* Allow access to a raw mixing buffer */

#include <errno.h>
#include <unistd.h>
#include <fcntl.h>
#include <sys/time.h>
#include <sys/ioctl.h>
#include <sys/stat.h>

#include "SDL_timer.h"
#include "SDL_audio.h"
#include "../SDL_audiomem.h"
#include "../SDL_audio_c.h"
#include "../SDL_audiodev_c.h"
#include "SDL_paudio.h"

#define DEBUG_AUDIO 1

/* A conflict within AIX 4.3.3 <sys/> headers and probably others as well.
 * I guess nobody ever uses audio... Shame over AIX header files.  */
#include <sys/machine.h>
#undef BIG_ENDIAN
#include <sys/audio.h>

/* The tag name used by paud audio */
#define Paud_DRIVER_NAME         "paud"

/* Open the audio device for playback, and don't block if busy */
/* #define OPEN_FLAGS	(O_WRONLY|O_NONBLOCK) */
#define OPEN_FLAGS	O_WRONLY

/* Audio driver functions */
static int Paud_OpenAudio(_THIS, SDL_AudioSpec *spec);
static void Paud_WaitAudio(_THIS);
static void Paud_PlayAudio(_THIS);
static Uint8 *Paud_GetAudioBuf(_THIS);
static void Paud_CloseAudio(_THIS);

/* Audio driver bootstrap functions */

static int Audio_Available(void)
{
	int fd;
	int available;

	available = 0;
	fd = SDL_OpenAudioPath(NULL, 0, OPEN_FLAGS, 0);
	if ( fd >= 0 ) {
		available = 1;
		close(fd);
	}
	return(available);
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
	audio_fd = -1;

	/* Set the function pointers */
	this->OpenAudio = Paud_OpenAudio;
	this->WaitAudio = Paud_WaitAudio;
	this->PlayAudio = Paud_PlayAudio;
	this->GetAudioBuf = Paud_GetAudioBuf;
	this->CloseAudio = Paud_CloseAudio;

	this->free = Audio_DeleteDevice;

	return this;
}

AudioBootStrap Paud_bootstrap = {
	Paud_DRIVER_NAME, "AIX Paudio",
	Audio_Available, Audio_CreateDevice
};

/* This function waits until it is possible to write a full sound buffer */
static void Paud_WaitAudio(_THIS)
{
    fd_set fdset;

    /* See if we need to use timed audio synchronization */
    if ( frame_ticks ) {
        /* Use timer for general audio synchronization */
        Sint32 ticks;

        ticks = ((Sint32)(next_frame - SDL_GetTicks()))-FUDGE_TICKS;
        if ( ticks > 0 ) {
	    SDL_Delay(ticks);
        }
    } else {
        audio_buffer  paud_bufinfo;

        /* Use select() for audio synchronization */
        struct timeval timeout;
        FD_ZERO(&fdset);
        FD_SET(audio_fd, &fdset);

        if ( ioctl(audio_fd, AUDIO_BUFFER, &paud_bufinfo) < 0 ) {
#ifdef DEBUG_AUDIO
            fprintf(stderr, "Couldn't get audio buffer information\n");
#endif
            timeout.tv_sec  = 10;
            timeout.tv_usec = 0;
        } else {
	    long ms_in_buf = paud_bufinfo.write_buf_time;
            timeout.tv_sec  = ms_in_buf/1000;
	    ms_in_buf       = ms_in_buf - timeout.tv_sec*1000;
            timeout.tv_usec = ms_in_buf*1000;
#ifdef DEBUG_AUDIO
            fprintf( stderr,
		     "Waiting for write_buf_time=%ld,%ld\n",
		     timeout.tv_sec,
		     timeout.tv_usec );
#endif
	}

#ifdef DEBUG_AUDIO
        fprintf(stderr, "Waiting for audio to get ready\n");
#endif
        if ( select(audio_fd+1, NULL, &fdset, NULL, &timeout) <= 0 ) {
            const char *message = "Audio timeout - buggy audio driver? (disabled)";
            /*
	     * In general we should never print to the screen,
             * but in this case we have no other way of letting
             * the user know what happened.
             */
            fprintf(stderr, "SDL: %s - %s\n", strerror(errno), message);
            this->enabled = 0;
            /* Don't try to close - may hang */
            audio_fd = -1;
#ifdef DEBUG_AUDIO
            fprintf(stderr, "Done disabling audio\n");
#endif
        }
#ifdef DEBUG_AUDIO
        fprintf(stderr, "Ready!\n");
#endif
    }
}

static void Paud_PlayAudio(_THIS)
{
	int written;

	/* Write the audio data, checking for EAGAIN on broken audio drivers */
	do {
		written = write(audio_fd, mixbuf, mixlen);
		if ( (written < 0) && ((errno == 0) || (errno == EAGAIN)) ) {
			SDL_Delay(1);	/* Let a little CPU time go by */
		}
	} while ( (written < 0) && 
	          ((errno == 0) || (errno == EAGAIN) || (errno == EINTR)) );

	/* If timer synchronization is enabled, set the next write frame */
	if ( frame_ticks ) {
		next_frame += frame_ticks;
	}

	/* If we couldn't write, assume fatal error for now */
	if ( written < 0 ) {
		this->enabled = 0;
	}
#ifdef DEBUG_AUDIO
	fprintf(stderr, "Wrote %d bytes of audio data\n", written);
#endif
}

static Uint8 *Paud_GetAudioBuf(_THIS)
{
	return mixbuf;
}

static void Paud_CloseAudio(_THIS)
{
	if ( mixbuf != NULL ) {
		SDL_FreeAudioMem(mixbuf);
		mixbuf = NULL;
	}
	if ( audio_fd >= 0 ) {
		close(audio_fd);
		audio_fd = -1;
	}
}

static int Paud_OpenAudio(_THIS, SDL_AudioSpec *spec)
{
	char          audiodev[1024];
	int           format;
	int           bytes_per_sample;
	Uint16        test_format;
	audio_init    paud_init;
	audio_buffer  paud_bufinfo;
	audio_status  paud_status;
	audio_control paud_control;
	audio_change  paud_change;

	/* Reset the timer synchronization flag */
	frame_ticks = 0.0;

	/* Open the audio device */
	audio_fd = SDL_OpenAudioPath(audiodev, sizeof(audiodev), OPEN_FLAGS, 0);
	if ( audio_fd < 0 ) {
		SDL_SetError("Couldn't open %s: %s", audiodev, strerror(errno));
		return -1;
	}

	/*
	 * We can't set the buffer size - just ask the device for the maximum
	 * that we can have.
	 */
	if ( ioctl(audio_fd, AUDIO_BUFFER, &paud_bufinfo) < 0 ) {
		SDL_SetError("Couldn't get audio buffer information");
		return -1;
	}

	mixbuf = NULL;

	if ( spec->channels > 1 )
	    spec->channels = 2;
	else
	    spec->channels = 1;

	/*
	 * Fields in the audio_init structure:
	 *
	 * Ignored by us:
	 *
	 * paud.loadpath[LOAD_PATH]; * DSP code to load, MWave chip only?
	 * paud.slot_number;         * slot number of the adapter
	 * paud.device_id;           * adapter identification number
	 *
	 * Input:
	 *
	 * paud.srate;           * the sampling rate in Hz
	 * paud.bits_per_sample; * 8, 16, 32, ...
	 * paud.bsize;           * block size for this rate
	 * paud.mode;            * ADPCM, PCM, MU_LAW, A_LAW, SOURCE_MIX
	 * paud.channels;        * 1=mono, 2=stereo
	 * paud.flags;           * FIXED - fixed length data
	 *                       * LEFT_ALIGNED, RIGHT_ALIGNED (var len only)
	 *                       * TWOS_COMPLEMENT - 2's complement data
	 *                       * SIGNED - signed? comment seems wrong in sys/audio.h
	 *                       * BIG_ENDIAN
	 * paud.operation;       * PLAY, RECORD
	 *
	 * Output:
	 *
	 * paud.flags;           * PITCH            - pitch is supported
	 *                       * INPUT            - input is supported
	 *                       * OUTPUT           - output is supported
	 *                       * MONITOR          - monitor is supported
	 *                       * VOLUME           - volume is supported
	 *                       * VOLUME_DELAY     - volume delay is supported
	 *                       * BALANCE          - balance is supported
	 *                       * BALANCE_DELAY    - balance delay is supported
	 *                       * TREBLE           - treble control is supported
	 *                       * BASS             - bass control is supported
	 *                       * BESTFIT_PROVIDED - best fit returned
	 *                       * LOAD_CODE        - DSP load needed
	 * paud.rc;              * NO_PLAY         - DSP code can't do play requests
	 *                       * NO_RECORD       - DSP code can't do record requests
	 *                       * INVALID_REQUEST - request was invalid
	 *                       * CONFLICT        - conflict with open's flags
	 *                       * OVERLOADED      - out of DSP MIPS or memory
	 * paud.position_resolution; * smallest increment for position
	 */

        paud_init.srate = spec->freq;
	paud_init.mode = PCM;
	paud_init.operation = PLAY;
	paud_init.channels = spec->channels;

	/* Try for a closest match on audio format */
	format = 0;
	for ( test_format = SDL_FirstAudioFormat(spec->format);
						! format && test_format; ) {
#ifdef DEBUG_AUDIO
		fprintf(stderr, "Trying format 0x%4.4x\n", test_format);
#endif
		switch ( test_format ) {
			case AUDIO_U8:
			    bytes_per_sample = 1;
			    paud_init.bits_per_sample = 8;
			    paud_init.flags = TWOS_COMPLEMENT | FIXED;
			    format = 1;
			    break;
			case AUDIO_S8:
			    bytes_per_sample = 1;
			    paud_init.bits_per_sample = 8;
			    paud_init.flags = SIGNED |
					      TWOS_COMPLEMENT | FIXED;
			    format = 1;
			    break;
			case AUDIO_S16LSB:
			    bytes_per_sample = 2;
			    paud_init.bits_per_sample = 16;
			    paud_init.flags = SIGNED |
					      TWOS_COMPLEMENT | FIXED;
			    format = 1;
			    break;
			case AUDIO_S16MSB:
			    bytes_per_sample = 2;
			    paud_init.bits_per_sample = 16;
			    paud_init.flags = BIG_ENDIAN |
					      SIGNED |
					      TWOS_COMPLEMENT | FIXED;
			    format = 1;
			    break;
			case AUDIO_U16LSB:
			    bytes_per_sample = 2;
			    paud_init.bits_per_sample = 16;
			    paud_init.flags = TWOS_COMPLEMENT | FIXED;
			    format = 1;
			    break;
			case AUDIO_U16MSB:
			    bytes_per_sample = 2;
			    paud_init.bits_per_sample = 16;
			    paud_init.flags = BIG_ENDIAN |
					      TWOS_COMPLEMENT | FIXED;
			    format = 1;
			    break;
			default:
				break;
		}
		if ( ! format ) {
			test_format = SDL_NextAudioFormat();
		}
	}
	if ( format == 0 ) {
#ifdef DEBUG_AUDIO
            fprintf(stderr, "Couldn't find any hardware audio formats\n");
#endif
	    SDL_SetError("Couldn't find any hardware audio formats");
	    return -1;
	}
	spec->format = test_format;

	/*
	 * We know the buffer size and the max number of subsequent writes
	 * that can be pending. If more than one can pend, allow the application
	 * to do something like double buffering between our write buffer and
	 * the device's own buffer that we are filling with write() anyway.
	 *
	 * We calculate spec->samples like this because SDL_CalculateAudioSpec()
	 * will give put paud_bufinfo.write_buf_cap (or paud_bufinfo.write_buf_cap/2)
	 * into spec->size in return.
	 */
	if ( paud_bufinfo.request_buf_cap == 1 )
	{
	    spec->samples = paud_bufinfo.write_buf_cap
			  / bytes_per_sample
			  / spec->channels;
	}
	else
	{
	    spec->samples = paud_bufinfo.write_buf_cap
			  / bytes_per_sample
			  / spec->channels
			  / 2;
	}
        paud_init.bsize = bytes_per_sample * spec->channels;

	SDL_CalculateAudioSpec(spec);

	/*
	 * The AIX paud device init can't modify the values of the audio_init
	 * structure that we pass to it. So we don't need any recalculation
	 * of this stuff and no reinit call as in linux dsp and dma code.
	 *
	 * /dev/paud supports all of the encoding formats, so we don't need
	 * to do anything like reopening the device, either.
	 */
	if ( ioctl(audio_fd, AUDIO_INIT, &paud_init) < 0 ) {
	    switch ( paud_init.rc )
	    {
	    case 1 :
		SDL_SetError("Couldn't set audio format: DSP can't do play requests");
		return -1;
		break;
	    case 2 :
		SDL_SetError("Couldn't set audio format: DSP can't do record requests");
		return -1;
		break;
	    case 4 :
		SDL_SetError("Couldn't set audio format: request was invalid");
		return -1;
		break;
	    case 5 :
		SDL_SetError("Couldn't set audio format: conflict with open's flags");
		return -1;
		break;
	    case 6 :
		SDL_SetError("Couldn't set audio format: out of DSP MIPS or memory");
		return -1;
		break;
	    default :
		SDL_SetError("Couldn't set audio format: not documented in sys/audio.h");
		return -1;
		break;
	    }
	}

	/* Allocate mixing buffer */
	mixlen = spec->size;
	mixbuf = (Uint8 *)SDL_AllocAudioMem(mixlen);
	if ( mixbuf == NULL ) {
		return -1;
	}
	SDL_memset(mixbuf, spec->silence, spec->size);

	/*
	 * Set some paramters: full volume, first speaker that we can find.
	 * Ignore the other settings for now.
	 */
	paud_change.input = AUDIO_IGNORE;         /* the new input source */
        paud_change.output = OUTPUT_1;            /* EXTERNAL_SPEAKER,INTERNAL_SPEAKER,OUTPUT_1 */
        paud_change.monitor = AUDIO_IGNORE;       /* the new monitor state */
        paud_change.volume = 0x7fffffff;          /* volume level [0-0x7fffffff] */
        paud_change.volume_delay = AUDIO_IGNORE;  /* the new volume delay */
        paud_change.balance = 0x3fffffff;         /* the new balance */
        paud_change.balance_delay = AUDIO_IGNORE; /* the new balance delay */
        paud_change.treble = AUDIO_IGNORE;        /* the new treble state */
        paud_change.bass = AUDIO_IGNORE;          /* the new bass state */
        paud_change.pitch = AUDIO_IGNORE;         /* the new pitch state */

	paud_control.ioctl_request = AUDIO_CHANGE;
	paud_control.request_info = (char*)&paud_change;
	if ( ioctl(audio_fd, AUDIO_CONTROL, &paud_control) < 0 ) {
#ifdef DEBUG_AUDIO
            fprintf(stderr, "Can't change audio display settings\n" );
#endif
	}

	/*
	 * Tell the device to expect data. Actual start will wait for
	 * the first write() call.
	 */
	paud_control.ioctl_request = AUDIO_START;
	paud_control.position = 0;
	if ( ioctl(audio_fd, AUDIO_CONTROL, &paud_control) < 0 ) {
#ifdef DEBUG_AUDIO
            fprintf(stderr, "Can't start audio play\n" );
#endif
	    SDL_SetError("Can't start audio play");
	    return -1;
	}

        /* Check to see if we need to use select() workaround */
        { char *workaround;
                workaround = SDL_getenv("SDL_DSP_NOSELECT");
                if ( workaround ) {
                        frame_ticks = (float)(spec->samples*1000)/spec->freq;
                        next_frame = SDL_GetTicks()+frame_ticks;
                }
        }

	/* Get the parent process id (we're the parent of the audio thread) */
	parent = getpid();

	/* We're ready to rock and roll. :-) */
	return 0;
}

