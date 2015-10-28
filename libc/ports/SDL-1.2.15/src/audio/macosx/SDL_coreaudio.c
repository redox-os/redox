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

#include <CoreAudio/CoreAudio.h>
#include <CoreServices/CoreServices.h>
#include <AudioUnit/AudioUnit.h>
#if MAC_OS_X_VERSION_MAX_ALLOWED <= 1050
#include <AudioUnit/AUNTComponent.h>
#endif

#include "SDL_audio.h"
#include "../SDL_audio_c.h"
#include "../SDL_sysaudio.h"
#include "SDL_coreaudio.h"


/* Audio driver functions */

static int Core_OpenAudio(_THIS, SDL_AudioSpec *spec);
static void Core_WaitAudio(_THIS);
static void Core_PlayAudio(_THIS);
static Uint8 *Core_GetAudioBuf(_THIS);
static void Core_CloseAudio(_THIS);

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
    this->OpenAudio = Core_OpenAudio;
    this->WaitAudio = Core_WaitAudio;
    this->PlayAudio = Core_PlayAudio;
    this->GetAudioBuf = Core_GetAudioBuf;
    this->CloseAudio = Core_CloseAudio;

    this->free = Audio_DeleteDevice;

    return this;
}

AudioBootStrap COREAUDIO_bootstrap = {
    "coreaudio", "Mac OS X CoreAudio",
    Audio_Available, Audio_CreateDevice
};

/* The CoreAudio callback */
static OSStatus     audioCallback (void                            *inRefCon,
                                   AudioUnitRenderActionFlags      *ioActionFlags,
                                   const AudioTimeStamp            *inTimeStamp,
                                   UInt32                          inBusNumber,
                                   UInt32                          inNumberFrames,
                                   AudioBufferList                 *ioData)
{
    SDL_AudioDevice *this = (SDL_AudioDevice *)inRefCon;
    UInt32 remaining, len;
    AudioBuffer *abuf;
    void *ptr;
    UInt32 i;

    /* Only do anything if audio is enabled and not paused */
    if ( ! this->enabled || this->paused ) {
        for (i = 0; i < ioData->mNumberBuffers; i++) {
            abuf = &ioData->mBuffers[i];
            SDL_memset(abuf->mData, this->spec.silence, abuf->mDataByteSize);
        }
        return 0;
    }
    
    /* No SDL conversion should be needed here, ever, since we accept
       any input format in OpenAudio, and leave the conversion to CoreAudio.
     */
    /*
    assert(!this->convert.needed);
    assert(this->spec.channels == ioData->mNumberChannels);
     */

    for (i = 0; i < ioData->mNumberBuffers; i++) {
        abuf = &ioData->mBuffers[i];
        remaining = abuf->mDataByteSize;
        ptr = abuf->mData;
        while (remaining > 0) {
            if (bufferOffset >= bufferSize) {
                /* Generate the data */
                SDL_memset(buffer, this->spec.silence, bufferSize);
                SDL_mutexP(this->mixer_lock);
                (*this->spec.callback)(this->spec.userdata,
                            buffer, bufferSize);
                SDL_mutexV(this->mixer_lock);
                bufferOffset = 0;
            }
        
            len = bufferSize - bufferOffset;
            if (len > remaining)
                len = remaining;
            SDL_memcpy(ptr, (char *)buffer + bufferOffset, len);
            ptr = (char *)ptr + len;
            remaining -= len;
            bufferOffset += len;
        }
    }

    return 0;
}

/* Dummy functions -- we don't use thread-based audio */
void Core_WaitAudio(_THIS)
{
    return;
}

void Core_PlayAudio(_THIS)
{
    return;
}

Uint8 *Core_GetAudioBuf(_THIS)
{
    return(NULL);
}

void Core_CloseAudio(_THIS)
{
    OSStatus result;
    struct AURenderCallbackStruct callback;

    /* stop processing the audio unit */
    result = AudioOutputUnitStop (outputAudioUnit);
    if (result != noErr) {
        SDL_SetError("Core_CloseAudio: AudioOutputUnitStop");
        return;
    }

    /* Remove the input callback */
    callback.inputProc = 0;
    callback.inputProcRefCon = 0;
    result = AudioUnitSetProperty (outputAudioUnit, 
                        kAudioUnitProperty_SetRenderCallback,
                        kAudioUnitScope_Input, 
                        0,
                        &callback, 
                        sizeof(callback));
    if (result != noErr) {
        SDL_SetError("Core_CloseAudio: AudioUnitSetProperty (kAudioUnitProperty_SetInputCallback)");
        return;
    }

    result = CloseComponent(outputAudioUnit);
    if (result != noErr) {
        SDL_SetError("Core_CloseAudio: CloseComponent");
        return;
    }
    
    SDL_free(buffer);
}

#define CHECK_RESULT(msg) \
    if (result != noErr) { \
        SDL_SetError("Failed to start CoreAudio: " msg); \
        return -1; \
    }


int Core_OpenAudio(_THIS, SDL_AudioSpec *spec)
{
    OSStatus result = noErr;
    Component comp;
    ComponentDescription desc;
    struct AURenderCallbackStruct callback;
    AudioStreamBasicDescription requestedDesc;

    /* Setup a AudioStreamBasicDescription with the requested format */
    requestedDesc.mFormatID = kAudioFormatLinearPCM;
    requestedDesc.mFormatFlags = kLinearPCMFormatFlagIsPacked;
    requestedDesc.mChannelsPerFrame = spec->channels;
    requestedDesc.mSampleRate = spec->freq;
    
    requestedDesc.mBitsPerChannel = spec->format & 0xFF;
    if (spec->format & 0x8000)
        requestedDesc.mFormatFlags |= kLinearPCMFormatFlagIsSignedInteger;
    if (spec->format & 0x1000)
        requestedDesc.mFormatFlags |= kLinearPCMFormatFlagIsBigEndian;

    requestedDesc.mFramesPerPacket = 1;
    requestedDesc.mBytesPerFrame = requestedDesc.mBitsPerChannel * requestedDesc.mChannelsPerFrame / 8;
    requestedDesc.mBytesPerPacket = requestedDesc.mBytesPerFrame * requestedDesc.mFramesPerPacket;


    /* Locate the default output audio unit */
    desc.componentType = kAudioUnitType_Output;
    desc.componentSubType = kAudioUnitSubType_DefaultOutput;
    desc.componentManufacturer = kAudioUnitManufacturer_Apple;
    desc.componentFlags = 0;
    desc.componentFlagsMask = 0;
    
    comp = FindNextComponent (NULL, &desc);
    if (comp == NULL) {
        SDL_SetError ("Failed to start CoreAudio: FindNextComponent returned NULL");
        return -1;
    }
    
    /* Open & initialize the default output audio unit */
    result = OpenAComponent (comp, &outputAudioUnit);
    CHECK_RESULT("OpenAComponent")

    result = AudioUnitInitialize (outputAudioUnit);
    CHECK_RESULT("AudioUnitInitialize")
                
    /* Set the input format of the audio unit. */
    result = AudioUnitSetProperty (outputAudioUnit,
                               kAudioUnitProperty_StreamFormat,
                               kAudioUnitScope_Input,
                               0,
                               &requestedDesc,
                               sizeof (requestedDesc));
    CHECK_RESULT("AudioUnitSetProperty (kAudioUnitProperty_StreamFormat)")

    /* Set the audio callback */
    callback.inputProc = audioCallback;
    callback.inputProcRefCon = this;
    result = AudioUnitSetProperty (outputAudioUnit, 
                        kAudioUnitProperty_SetRenderCallback,
                        kAudioUnitScope_Input, 
                        0,
                        &callback, 
                        sizeof(callback));
    CHECK_RESULT("AudioUnitSetProperty (kAudioUnitProperty_SetInputCallback)")

    /* Calculate the final parameters for this audio specification */
    SDL_CalculateAudioSpec(spec);
    
    /* Allocate a sample buffer */
    bufferOffset = bufferSize = this->spec.size;
    buffer = SDL_malloc(bufferSize);

    /* Finally, start processing of the audio unit */
    result = AudioOutputUnitStart (outputAudioUnit);
    CHECK_RESULT("AudioOutputUnitStart")    
    

    /* We're running! */
    return(1);
}
