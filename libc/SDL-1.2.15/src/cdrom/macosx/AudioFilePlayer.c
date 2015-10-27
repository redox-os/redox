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

    This file based on Apple sample code. We haven't changed the file name, 
    so if you want to see the original search for it on apple.com/developer
*/
#include "SDL_config.h"
#include "SDL_endian.h"

/*~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    AudioFilePlayer.cpp
*/
#include "AudioFilePlayer.h"

/*
void ThrowResult (OSStatus result, const char* str)
{
    SDL_SetError ("Error: %s %d", str, result);
    throw result;
}
*/

#if DEBUG
static void PrintStreamDesc (AudioStreamBasicDescription *inDesc)
{
    if (!inDesc) {
        printf ("Can't print a NULL desc!\n");
        return;
    }
    
    printf ("- - - - - - - - - - - - - - - - - - - -\n");
    printf ("  Sample Rate:%f\n", inDesc->mSampleRate);
    printf ("  Format ID:%s\n", (char*)&inDesc->mFormatID);
    printf ("  Format Flags:%lX\n", inDesc->mFormatFlags);
    printf ("  Bytes per Packet:%ld\n", inDesc->mBytesPerPacket);
    printf ("  Frames per Packet:%ld\n", inDesc->mFramesPerPacket);
    printf ("  Bytes per Frame:%ld\n", inDesc->mBytesPerFrame);
    printf ("  Channels per Frame:%ld\n", inDesc->mChannelsPerFrame);
    printf ("  Bits per Channel:%ld\n", inDesc->mBitsPerChannel);
    printf ("- - - - - - - - - - - - - - - - - - - -\n");
}
#endif


static int AudioFilePlayer_SetDestination (AudioFilePlayer *afp, AudioUnit  *inDestUnit)
{
    /*if (afp->mConnected) throw static_cast<OSStatus>(-1);*/ /* can't set dest if already engaged */
    if (afp->mConnected)
        return 0 ;

    SDL_memcpy(&afp->mPlayUnit, inDestUnit, sizeof (afp->mPlayUnit));

    OSStatus result = noErr;
    

        /* we can "down" cast a component instance to a component */
    ComponentDescription desc;
    result = GetComponentInfo ((Component)*inDestUnit, &desc, 0, 0, 0);
    if (result) return 0; /*THROW_RESULT("GetComponentInfo")*/
        
        /* we're going to use this to know which convert routine to call
           a v1 audio unit will have a type of 'aunt'
           a v2 audio unit will have one of several different types. */
    if (desc.componentType != kAudioUnitType_Output) {
        result = badComponentInstance;
        /*THROW_RESULT("BAD COMPONENT")*/
        if (result) return 0;
    }

    /* Set the input format of the audio unit. */
    result = AudioUnitSetProperty (*inDestUnit,
                               kAudioUnitProperty_StreamFormat,
                               kAudioUnitScope_Input,
                               0,
                               &afp->mFileDescription,
                               sizeof (afp->mFileDescription));
        /*THROW_RESULT("AudioUnitSetProperty")*/
    if (result) return 0;
    return 1;
}

static void AudioFilePlayer_SetNotifier(AudioFilePlayer *afp, AudioFilePlayNotifier inNotifier, void *inRefCon)
{
    afp->mNotifier = inNotifier;
    afp->mRefCon = inRefCon;
}

static int AudioFilePlayer_IsConnected(AudioFilePlayer *afp)
{
    return afp->mConnected;
}

static AudioUnit AudioFilePlayer_GetDestUnit(AudioFilePlayer *afp)
{
   return afp->mPlayUnit;
}

static void AudioFilePlayer_Print(AudioFilePlayer *afp)
{
#if DEBUG    
    printf ("Is Connected:%s\n", (IsConnected() ? "true" : "false"));
    printf ("- - - - - - - - - - - - - - \n");
#endif
}

static void    AudioFilePlayer_SetStartFrame (AudioFilePlayer *afp, int frame)
{
    SInt64 position = frame * 2352;

    afp->mStartFrame = frame;
    afp->mAudioFileManager->SetPosition (afp->mAudioFileManager, position);
}

    
static int    AudioFilePlayer_GetCurrentFrame (AudioFilePlayer *afp)
{
    return afp->mStartFrame + (afp->mAudioFileManager->GetByteCounter(afp->mAudioFileManager) / 2352);
}
    
static void    AudioFilePlayer_SetStopFrame (AudioFilePlayer *afp, int frame)
{
    SInt64 position  = frame * 2352;
    
    afp->mAudioFileManager->SetEndOfFile (afp->mAudioFileManager, position);
}
    
void delete_AudioFilePlayer(AudioFilePlayer *afp)
{
    if (afp != NULL)
    {
        afp->Disconnect(afp);
        
        if (afp->mAudioFileManager) {
            delete_AudioFileManager(afp->mAudioFileManager);
            afp->mAudioFileManager = 0;
        }
    
        if (afp->mForkRefNum) {
            FSCloseFork (afp->mForkRefNum);
            afp->mForkRefNum = 0;
        }
        SDL_free(afp);
    }
}

static int    AudioFilePlayer_Connect(AudioFilePlayer *afp)
{
#if DEBUG
    printf ("Connect:%x, engaged=%d\n", (int)afp->mPlayUnit, (afp->mConnected ? 1 : 0));
#endif
    if (!afp->mConnected)
    {           
        if (!afp->mAudioFileManager->DoConnect(afp->mAudioFileManager))
            return 0;

        /* set the render callback for the file data to be supplied to the sound converter AU */
        afp->mInputCallback.inputProc = afp->mAudioFileManager->FileInputProc;
        afp->mInputCallback.inputProcRefCon = afp->mAudioFileManager;

        OSStatus result = AudioUnitSetProperty (afp->mPlayUnit, 
                            kAudioUnitProperty_SetRenderCallback,
                            kAudioUnitScope_Input, 
                            0,
                            &afp->mInputCallback, 
                            sizeof(afp->mInputCallback));
        if (result) return 0;  /*THROW_RESULT("AudioUnitSetProperty")*/
        afp->mConnected = 1;
    }

    return 1;
}

/* warning noted, now please go away ;-) */
/* #warning This should redirect the calling of notification code to some other thread */
static void    AudioFilePlayer_DoNotification (AudioFilePlayer *afp, OSStatus inStatus)
{
    if (afp->mNotifier) {
        (*afp->mNotifier) (afp->mRefCon, inStatus);
    } else {
        SDL_SetError ("Notification posted with no notifier in place");
        
        if (inStatus == kAudioFilePlay_FileIsFinished)
            afp->Disconnect(afp);
        else if (inStatus != kAudioFilePlayErr_FilePlayUnderrun)
            afp->Disconnect(afp);
    }
}

static void    AudioFilePlayer_Disconnect (AudioFilePlayer *afp)
{
#if DEBUG
    printf ("Disconnect:%x,%ld, engaged=%d\n", (int)afp->mPlayUnit, 0, (afp->mConnected ? 1 : 0));
#endif
    if (afp->mConnected)
    {
        afp->mConnected = 0;
            
        afp->mInputCallback.inputProc = 0;
        afp->mInputCallback.inputProcRefCon = 0;
        OSStatus result = AudioUnitSetProperty (afp->mPlayUnit, 
                                        kAudioUnitProperty_SetRenderCallback,
                                        kAudioUnitScope_Input, 
                                        0,
                                        &afp->mInputCallback, 
                                        sizeof(afp->mInputCallback));
        if (result) 
            SDL_SetError ("AudioUnitSetProperty:RemoveInputCallback:%ld", result);

        afp->mAudioFileManager->Disconnect(afp->mAudioFileManager);
    }
}

typedef struct {
    UInt32 offset;
    UInt32 blockSize;
} SSNDData;

static int    AudioFilePlayer_OpenFile (AudioFilePlayer *afp, const FSRef *inRef, SInt64 *outFileDataSize)
{
    ContainerChunk chunkHeader;
    ChunkHeader chunk;
    SSNDData ssndData;

    OSErr result;
    HFSUniStr255 dfName;
    ByteCount actual;
    SInt64 offset;

    /* Open the data fork of the input file */
    result = FSGetDataForkName(&dfName);
       if (result) return 0; /*THROW_RESULT("AudioFilePlayer::OpenFile(): FSGetDataForkName")*/

    result = FSOpenFork(inRef, dfName.length, dfName.unicode, fsRdPerm, &afp->mForkRefNum);
       if (result) return 0; /*THROW_RESULT("AudioFilePlayer::OpenFile(): FSOpenFork")*/
 
    /* Read the file header, and check if it's indeed an AIFC file */
    result = FSReadFork(afp->mForkRefNum, fsAtMark, 0, sizeof(chunkHeader), &chunkHeader, &actual);
       if (result) return 0; /*THROW_RESULT("AudioFilePlayer::OpenFile(): FSReadFork")*/

    if (SDL_SwapBE32(chunkHeader.ckID) != 'FORM') {
        result = -1;
        if (result) return 0; /*THROW_RESULT("AudioFilePlayer::OpenFile(): chunk id is not 'FORM'");*/
    }

    if (SDL_SwapBE32(chunkHeader.formType) != 'AIFC') {
        result = -1;
        if (result) return 0; /*THROW_RESULT("AudioFilePlayer::OpenFile(): file format is not 'AIFC'");*/
    }

    /* Search for the SSND chunk. We ignore all compression etc. information
       in other chunks. Of course that is kind of evil, but for now we are lazy
       and rely on the cdfs to always give us the same fixed format.
       TODO: Parse the COMM chunk we currently skip to fill in mFileDescription.
    */
    offset = 0;
    do {
        result = FSReadFork(afp->mForkRefNum, fsFromMark, offset, sizeof(chunk), &chunk, &actual);
        if (result) return 0; /*THROW_RESULT("AudioFilePlayer::OpenFile(): FSReadFork")*/

        chunk.ckID = SDL_SwapBE32(chunk.ckID);
        chunk.ckSize = SDL_SwapBE32(chunk.ckSize);

        /* Skip the chunk data */
        offset = chunk.ckSize;
    } while (chunk.ckID != 'SSND');

    /* Read the header of the SSND chunk. After this, we are positioned right
       at the start of the audio data. */
    result = FSReadFork(afp->mForkRefNum, fsAtMark, 0, sizeof(ssndData), &ssndData, &actual);
    if (result) return 0; /*THROW_RESULT("AudioFilePlayer::OpenFile(): FSReadFork")*/

    ssndData.offset = SDL_SwapBE32(ssndData.offset);

    result = FSSetForkPosition(afp->mForkRefNum, fsFromMark, ssndData.offset);
    if (result) return 0; /*THROW_RESULT("AudioFilePlayer::OpenFile(): FSSetForkPosition")*/

    /* Data size */
    *outFileDataSize = chunk.ckSize - ssndData.offset - 8;

    /* File format */
    afp->mFileDescription.mSampleRate = 44100;
    afp->mFileDescription.mFormatID = kAudioFormatLinearPCM;
    afp->mFileDescription.mFormatFlags = kLinearPCMFormatFlagIsPacked | kLinearPCMFormatFlagIsSignedInteger;
    afp->mFileDescription.mBytesPerPacket = 4;
    afp->mFileDescription.mFramesPerPacket = 1;
    afp->mFileDescription.mBytesPerFrame = 4;
    afp->mFileDescription.mChannelsPerFrame = 2;
    afp->mFileDescription.mBitsPerChannel = 16;

    return 1;
}

AudioFilePlayer *new_AudioFilePlayer (const FSRef *inFileRef)
{
    SInt64 fileDataSize  = 0;

    AudioFilePlayer *afp = (AudioFilePlayer *) SDL_malloc(sizeof (AudioFilePlayer));
    if (afp == NULL)
        return NULL;
    SDL_memset(afp, '\0', sizeof (*afp));

    #define SET_AUDIOFILEPLAYER_METHOD(m) afp->m = AudioFilePlayer_##m
    SET_AUDIOFILEPLAYER_METHOD(SetDestination);
    SET_AUDIOFILEPLAYER_METHOD(SetNotifier);
    SET_AUDIOFILEPLAYER_METHOD(SetStartFrame);
    SET_AUDIOFILEPLAYER_METHOD(GetCurrentFrame);
    SET_AUDIOFILEPLAYER_METHOD(SetStopFrame);
    SET_AUDIOFILEPLAYER_METHOD(Connect);
    SET_AUDIOFILEPLAYER_METHOD(Disconnect);
    SET_AUDIOFILEPLAYER_METHOD(DoNotification);
    SET_AUDIOFILEPLAYER_METHOD(IsConnected);
    SET_AUDIOFILEPLAYER_METHOD(GetDestUnit);
    SET_AUDIOFILEPLAYER_METHOD(Print);
    SET_AUDIOFILEPLAYER_METHOD(OpenFile);
    #undef SET_AUDIOFILEPLAYER_METHOD

    if (!afp->OpenFile (afp, inFileRef, &fileDataSize))
    {
        SDL_free(afp);
        return NULL;
    }
        
    /* we want about 4 seconds worth of data for the buffer */
    int bytesPerSecond = (UInt32) (4 * afp->mFileDescription.mSampleRate * afp->mFileDescription.mBytesPerFrame);
    
#if DEBUG
    printf("File format:\n");
    PrintStreamDesc (&afp->mFileDescription);
#endif
    
    afp->mAudioFileManager = new_AudioFileManager(afp, afp->mForkRefNum,
                                                  fileDataSize,
                                                  bytesPerSecond);
    if (afp->mAudioFileManager == NULL)
    {
        delete_AudioFilePlayer(afp);
        return NULL;
    }

    return afp;
}

