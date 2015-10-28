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

/*~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
    AudioFilePlayer.h
*/
#ifndef __AudioFilePlayer_H__
#define __AudioFilePlayer_H__

#include <CoreServices/CoreServices.h>

#include <AudioUnit/AudioUnit.h>
#if MAC_OS_X_VERSION_MAX_ALLOWED <= 1050
#include <AudioUnit/AUNTComponent.h>
#endif

#if (MAC_OS_X_VERSION_MAX_ALLOWED < 1050)
typedef SInt16 FSIORefNum;
#endif

#include "SDL_error.h"

const char* AudioFilePlayerErrorStr (OSStatus error);

/*
void ThrowResult (OSStatus result, const char *str);

#define THROW_RESULT(str)                                       \
    if (result) {                                               \
        ThrowResult (result, str);                              \
    }
*/

typedef void (*AudioFilePlayNotifier)(void          *inRefCon,
                                    OSStatus        inStatus);

enum {
    kAudioFilePlayErr_FilePlayUnderrun = -10000,
    kAudioFilePlay_FileIsFinished = -10001,
    kAudioFilePlay_PlayerIsUninitialized = -10002
};


struct S_AudioFileManager;

#pragma mark __________ AudioFilePlayer
typedef struct S_AudioFilePlayer
{
/*public:*/
    int             (*SetDestination)(struct S_AudioFilePlayer *afp, AudioUnit *inDestUnit);
    void            (*SetNotifier)(struct S_AudioFilePlayer *afp, AudioFilePlayNotifier inNotifier, void *inRefCon);
    void            (*SetStartFrame)(struct S_AudioFilePlayer *afp, int frame); /* seek in the file */
    int             (*GetCurrentFrame)(struct S_AudioFilePlayer *afp); /* get the current frame position */
    void            (*SetStopFrame)(struct S_AudioFilePlayer *afp, int frame);   /* set limit in the file */
    int             (*Connect)(struct S_AudioFilePlayer *afp);
    void            (*Disconnect)(struct S_AudioFilePlayer *afp);
    void            (*DoNotification)(struct S_AudioFilePlayer *afp, OSStatus inError);
    int             (*IsConnected)(struct S_AudioFilePlayer *afp);
    AudioUnit       (*GetDestUnit)(struct S_AudioFilePlayer *afp);
    void            (*Print)(struct S_AudioFilePlayer *afp);

/*private:*/
    AudioUnit                       mPlayUnit;
    FSIORefNum                      mForkRefNum;
    
    AURenderCallbackStruct          mInputCallback;

    AudioStreamBasicDescription     mFileDescription;
    
    int                             mConnected;
    
    struct S_AudioFileManager*      mAudioFileManager;
    
    AudioFilePlayNotifier           mNotifier;
    void*                           mRefCon;
    
    int                             mStartFrame;
    
#pragma mark __________ Private_Methods
    
    int          (*OpenFile)(struct S_AudioFilePlayer *afp, const FSRef *inRef, SInt64 *outFileSize);
} AudioFilePlayer;


AudioFilePlayer *new_AudioFilePlayer(const FSRef    *inFileRef);
void delete_AudioFilePlayer(AudioFilePlayer *afp);



#pragma mark __________ AudioFileManager
typedef struct S_AudioFileManager
{
/*public:*/
        /* this method should NOT be called by an object of this class
           as it is called by the parent's Disconnect() method */
    void                (*Disconnect)(struct S_AudioFileManager *afm);
    int                 (*DoConnect)(struct S_AudioFileManager *afm);
    OSStatus            (*Read)(struct S_AudioFileManager *afm, char *buffer, ByteCount *len);
    const char*         (*GetFileBuffer)(struct S_AudioFileManager *afm);
    const AudioFilePlayer *(*GetParent)(struct S_AudioFileManager *afm);
    void                (*SetPosition)(struct S_AudioFileManager *afm, SInt64 pos);  /* seek/rewind in the file */
    int                 (*GetByteCounter)(struct S_AudioFileManager *afm);  /* return actual bytes streamed to audio hardware */
    void                (*SetEndOfFile)(struct S_AudioFileManager *afm, SInt64 pos);  /* set the "EOF" (will behave just like it reached eof) */
   
/*protected:*/
    AudioFilePlayer*    mParent;
    SInt16              mForkRefNum;
    SInt64              mAudioDataOffset;
    
    char*               mFileBuffer;

    int                 mByteCounter;

    int                mReadFromFirstBuffer;
    int                mLockUnsuccessful;
    int                mIsEngaged;
    
    int                 mNumTimesAskedSinceFinished;


	void*               mTmpBuffer;
	UInt32              mBufferSize;
	UInt32              mBufferOffset;
/*public:*/
    UInt32              mChunkSize;
    SInt64              mFileLength;
    SInt64              mReadFilePosition;
    int                 mWriteToFirstBuffer;
    int                 mFinishedReadingData;

/*protected:*/
    OSStatus            (*Render)(struct S_AudioFileManager *afm, AudioBufferList *ioData);
    OSStatus            (*GetFileData)(struct S_AudioFileManager *afm, void** inOutData, UInt32 *inOutDataSize);
    void                (*AfterRender)(struct S_AudioFileManager *afm);

/*public:*/
    /*static*/
    OSStatus            (*FileInputProc)(void                            *inRefCon,
                                         AudioUnitRenderActionFlags      *ioActionFlags,
                                         const AudioTimeStamp            *inTimeStamp,
                                         UInt32                          inBusNumber,
                                         UInt32                          inNumberFrames,
                                         AudioBufferList                 *ioData);
} AudioFileManager;


AudioFileManager *new_AudioFileManager (AudioFilePlayer *inParent,
                      SInt16          inForkRefNum, 
                      SInt64          inFileLength,
                      UInt32          inChunkSize);
    
void delete_AudioFileManager(AudioFileManager *afm);

#endif

