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

#if defined(__APPLE__) && defined(__MACH__)
#  include <Carbon/Carbon.h>
#elif TARGET_API_MAC_CARBON && (UNIVERSAL_INTERFACES_VERSION > 0x0335)
#  include <Carbon.h>
#else
#  include <Sound.h> /* SoundManager interface */
#  include <Gestalt.h>
#  include <DriverServices.h>
#endif

#if !defined(NewSndCallBackUPP) && (UNIVERSAL_INTERFACES_VERSION < 0x0335)
#if !defined(NewSndCallBackProc) /* avoid circular redefinition... */
#define NewSndCallBackUPP NewSndCallBackProc
#endif
#if !defined(NewSndCallBackUPP)
#define NewSndCallBackUPP NewSndCallBackProc
#endif
#endif

#include "SDL_audio.h"
#include "../SDL_audio_c.h"
#include "../SDL_sysaudio.h"
#include "SDL_romaudio.h"

/* Audio driver functions */

static void Mac_CloseAudio(_THIS);
static int Mac_OpenAudio(_THIS, SDL_AudioSpec *spec);
static void Mac_LockAudio(_THIS);
static void Mac_UnlockAudio(_THIS);

/* Audio driver bootstrap functions */


static int Audio_Available(void)
{
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
    this->OpenAudio   = Mac_OpenAudio;
    this->CloseAudio  = Mac_CloseAudio;
    this->LockAudio   = Mac_LockAudio;
    this->UnlockAudio = Mac_UnlockAudio;
    this->free        = Audio_DeleteDevice;

#ifdef __MACOSX__	/* Mac OS X uses threaded audio, so normal thread code is okay */
    this->LockAudio   = NULL;
    this->UnlockAudio = NULL;
#endif
    return this;
}

AudioBootStrap SNDMGR_bootstrap = {
	"sndmgr", "MacOS SoundManager 3.0",
	Audio_Available, Audio_CreateDevice
};

#if defined(TARGET_API_MAC_CARBON) || defined(USE_RYANS_SOUNDCODE)
/* This works correctly on Mac OS X */

#pragma options align=power

static volatile SInt32 audio_is_locked = 0;
static volatile SInt32 need_to_mix = 0;

static UInt8  *buffer[2];
static volatile UInt32 running = 0;
static CmpSoundHeader header;
static volatile Uint32 fill_me = 0;

static void mix_buffer(SDL_AudioDevice *audio, UInt8 *buffer)
{
   if ( ! audio->paused ) {
#ifdef __MACOSX__
        SDL_mutexP(audio->mixer_lock);
#endif
        if ( audio->convert.needed ) {
            audio->spec.callback(audio->spec.userdata,
                    (Uint8 *)audio->convert.buf,audio->convert.len);
            SDL_ConvertAudio(&audio->convert);
            if ( audio->convert.len_cvt != audio->spec.size ) {
                /* Uh oh... probably crashes here */;
            }
            SDL_memcpy(buffer, audio->convert.buf, audio->convert.len_cvt);
        } else {
            audio->spec.callback(audio->spec.userdata, buffer, audio->spec.size);
        }
#ifdef __MACOSX__
        SDL_mutexV(audio->mixer_lock);
#endif
    }

    DecrementAtomic((SInt32 *) &need_to_mix);
}

static void Mac_LockAudio(_THIS)
{
    IncrementAtomic((SInt32 *) &audio_is_locked);
}

static void Mac_UnlockAudio(_THIS)
{
    SInt32 oldval;
         
    oldval = DecrementAtomic((SInt32 *) &audio_is_locked);
    if ( oldval != 1 )  /* != 1 means audio is still locked. */
        return;

    /* Did we miss the chance to mix in an interrupt? Do it now. */
    if ( BitAndAtomic (0xFFFFFFFF, (UInt32 *) &need_to_mix) ) {
        /*
         * Note that this could be a problem if you missed an interrupt
         *  while the audio was locked, and get preempted by a second
         *  interrupt here, but that means you locked for way too long anyhow.
         */
        mix_buffer (this, buffer[fill_me]);
    }
}

static void callBackProc (SndChannel *chan, SndCommand *cmd_passed ) {
   UInt32 play_me;
   SndCommand cmd; 
   SDL_AudioDevice *audio = (SDL_AudioDevice *)chan->userInfo;

   IncrementAtomic((SInt32 *) &need_to_mix);

   fill_me = cmd_passed->param2;  /* buffer that has just finished playing, so fill it */      
   play_me = ! fill_me;           /* filled buffer to play _now_ */

   if ( ! audio->enabled ) {
      return;
   }
   
   /* queue previously mixed buffer for playback. */
   header.samplePtr = (Ptr)buffer[play_me];
   cmd.cmd = bufferCmd;
   cmd.param1 = 0; 
   cmd.param2 = (long)&header;
   SndDoCommand (chan, &cmd, 0);

   memset (buffer[fill_me], 0, audio->spec.size);

   /*
    * if audio device isn't locked, mix the next buffer to be queued in
    *  the memory block that just finished playing.
    */
   if ( ! BitAndAtomic(0xFFFFFFFF, (UInt32 *) &audio_is_locked) ) {
      mix_buffer (audio, buffer[fill_me]);
   } 

   /* set this callback to run again when current buffer drains. */
   if ( running ) {
      cmd.cmd = callBackCmd;
      cmd.param1 = 0;
      cmd.param2 = play_me;
   
      SndDoCommand (chan, &cmd, 0);
   }
}

static int Mac_OpenAudio(_THIS, SDL_AudioSpec *spec) {

   SndCallBackUPP callback;
   int sample_bits;
   int i;
   long initOptions;
      
   /* Very few conversions are required, but... */
    switch (spec->format) {
        case AUDIO_S8:
        spec->format = AUDIO_U8;
        break;
        case AUDIO_U16LSB:
        spec->format = AUDIO_S16LSB;
        break;
        case AUDIO_U16MSB:
        spec->format = AUDIO_S16MSB;
        break;
    }
    SDL_CalculateAudioSpec(spec);
    
    /* initialize bufferCmd header */
    memset (&header, 0, sizeof(header));
    callback = (SndCallBackUPP) NewSndCallBackUPP (callBackProc);
    sample_bits = spec->size / spec->samples / spec->channels * 8;

#ifdef DEBUG_AUDIO
    fprintf(stderr,
	"Audio format 0x%x, channels = %d, sample_bits = %d, frequency = %d\n",
	spec->format, spec->channels, sample_bits, spec->freq);
#endif /* DEBUG_AUDIO */
    
    header.numChannels = spec->channels;
    header.sampleSize  = sample_bits;
    header.sampleRate  = spec->freq << 16;
    header.numFrames   = spec->samples;
    header.encode      = cmpSH;
    
    /* Note that we install the 16bitLittleEndian Converter if needed. */
    if ( spec->format == 0x8010 ) {
        header.compressionID = fixedCompression;
        header.format = k16BitLittleEndianFormat;
    }
    
    /* allocate 2 buffers */
    for (i=0; i<2; i++) {
       buffer[i] = (UInt8*)malloc (sizeof(UInt8) * spec->size);
      if (buffer[i] == NULL) {
         SDL_OutOfMemory();
         return (-1);
      }
     memset (buffer[i], 0, spec->size);
   }
   
   /* Create the sound manager channel */
    channel = (SndChannelPtr)SDL_malloc(sizeof(*channel));
    if ( channel == NULL ) {
        SDL_OutOfMemory();
        return(-1);
    }
    if ( spec->channels >= 2 ) {
        initOptions = initStereo;
    } else {
        initOptions = initMono;
    }
    channel->userInfo = (long)this;
    channel->qLength = 128;
    if ( SndNewChannel(&channel, sampledSynth, initOptions, callback) != noErr ) {
        SDL_SetError("Unable to create audio channel");
        SDL_free(channel);
        channel = NULL;
        return(-1);
    }
   
   /* start playback */
   {
      SndCommand cmd;
      cmd.cmd = callBackCmd;
      cmd.param2 = 0;
      running = 1;
      SndDoCommand (channel, &cmd, 0);
   }
   
   return 1;
}

static void Mac_CloseAudio(_THIS) {
   
   int i;
   
   running = 0;
   
   if (channel) {
      SndDisposeChannel (channel, true);
      channel = NULL;
   }
   
    for ( i=0; i<2; ++i ) {
        if ( buffer[i] ) {
            SDL_free(buffer[i]);
            buffer[i] = NULL;
        }
    }
}

#else /* !TARGET_API_MAC_CARBON && !USE_RYANS_SOUNDCODE */

static void Mac_LockAudio(_THIS)
{
    /* no-op. */
}

static void Mac_UnlockAudio(_THIS)
{
    /* no-op. */
}


/* This function is called by Sound Manager when it has exhausted one of
   the buffers, so we'll zero it to silence and fill it with audio if
   we're not paused.
*/
static pascal
void sndDoubleBackProc (SndChannelPtr chan, SndDoubleBufferPtr newbuf)
{
    SDL_AudioDevice *audio = (SDL_AudioDevice *)newbuf->dbUserInfo[0];

    /* If audio is quitting, don't do anything */
    if ( ! audio->enabled ) {
        return;
    }
    memset (newbuf->dbSoundData, 0, audio->spec.size);
    newbuf->dbNumFrames = audio->spec.samples;
    if ( ! audio->paused ) {
        if ( audio->convert.needed ) {
            audio->spec.callback(audio->spec.userdata,
                (Uint8 *)audio->convert.buf,audio->convert.len);
            SDL_ConvertAudio(&audio->convert);
#if 0
            if ( audio->convert.len_cvt != audio->spec.size ) {
                /* Uh oh... probably crashes here */;
            }
#endif
            SDL_memcpy(newbuf->dbSoundData, audio->convert.buf,
                            audio->convert.len_cvt);
        } else {
            audio->spec.callback(audio->spec.userdata,
                (Uint8 *)newbuf->dbSoundData, audio->spec.size);
        }
    }
    newbuf->dbFlags    |= dbBufferReady;
}

static int DoubleBufferAudio_Available(void)
{
    int available;
    NumVersion sndversion;
    long response;

    available = 0;
    sndversion = SndSoundManagerVersion();
    if ( sndversion.majorRev >= 3 ) {
        if ( Gestalt(gestaltSoundAttr, &response) == noErr ) {
            if ( (response & (1 << gestaltSndPlayDoubleBuffer)) ) {
                available = 1;
            }
        }
    } else {
        if ( Gestalt(gestaltSoundAttr, &response) == noErr ) {
            if ( (response & (1 << gestaltHasASC)) ) {
                available = 1;
            }
        }
    }
    return(available);
}

static void Mac_CloseAudio(_THIS)
{
    int i;

    if ( channel != NULL ) {
        /* Clean up the audio channel */
        SndDisposeChannel(channel, true);
        channel = NULL;
    }
    for ( i=0; i<2; ++i ) {
        if ( audio_buf[i] ) {
            SDL_free(audio_buf[i]);
            audio_buf[i] = NULL;
        }
    }
}

static int Mac_OpenAudio(_THIS, SDL_AudioSpec *spec)
{
    SndDoubleBufferHeader2 audio_dbh;
    int i;
    long initOptions;
    int sample_bits;
    SndDoubleBackUPP doubleBackProc;

    /* Check to make sure double-buffered audio is available */
    if ( ! DoubleBufferAudio_Available() ) {
        SDL_SetError("Sound manager doesn't support double-buffering");
        return(-1);
    }

    /* Very few conversions are required, but... */
    switch (spec->format) {
        case AUDIO_S8:
        spec->format = AUDIO_U8;
        break;
        case AUDIO_U16LSB:
        spec->format = AUDIO_S16LSB;
        break;
        case AUDIO_U16MSB:
        spec->format = AUDIO_S16MSB;
        break;
    }
    SDL_CalculateAudioSpec(spec);

    /* initialize the double-back header */
    SDL_memset(&audio_dbh, 0, sizeof(audio_dbh));
    doubleBackProc = NewSndDoubleBackProc (sndDoubleBackProc);
    sample_bits = spec->size / spec->samples / spec->channels * 8;
    
    audio_dbh.dbhNumChannels = spec->channels;
    audio_dbh.dbhSampleSize    = sample_bits;
    audio_dbh.dbhCompressionID = 0;
    audio_dbh.dbhPacketSize    = 0;
    audio_dbh.dbhSampleRate    = spec->freq << 16;
    audio_dbh.dbhDoubleBack    = doubleBackProc;
    audio_dbh.dbhFormat    = 0;

    /* Note that we install the 16bitLittleEndian Converter if needed. */
    if ( spec->format == 0x8010 ) {
        audio_dbh.dbhCompressionID = fixedCompression;
        audio_dbh.dbhFormat = k16BitLittleEndianFormat;
    }

    /* allocate the 2 double-back buffers */
    for ( i=0; i<2; ++i ) {
        audio_buf[i] = SDL_calloc(1, sizeof(SndDoubleBuffer)+spec->size);
        if ( audio_buf[i] == NULL ) {
            SDL_OutOfMemory();
            return(-1);
        }
        audio_buf[i]->dbNumFrames = spec->samples;
        audio_buf[i]->dbFlags = dbBufferReady;
        audio_buf[i]->dbUserInfo[0] = (long)this;
        audio_dbh.dbhBufferPtr[i] = audio_buf[i];
    }

    /* Create the sound manager channel */
    channel = (SndChannelPtr)SDL_malloc(sizeof(*channel));
    if ( channel == NULL ) {
        SDL_OutOfMemory();
        return(-1);
    }
    if ( spec->channels >= 2 ) {
        initOptions = initStereo;
    } else {
        initOptions = initMono;
    }
    channel->userInfo = 0;
    channel->qLength = 128;
    if ( SndNewChannel(&channel, sampledSynth, initOptions, 0L) != noErr ) {
        SDL_SetError("Unable to create audio channel");
        SDL_free(channel);
        channel = NULL;
        return(-1);
    }
 
    /* Start playback */
    if ( SndPlayDoubleBuffer(channel, (SndDoubleBufferHeaderPtr)&audio_dbh)
                                != noErr ) {
        SDL_SetError("Unable to play double buffered audio");
        return(-1);
    }
    
    return 1;
}

#endif /* TARGET_API_MAC_CARBON || USE_RYANS_SOUNDCODE */

