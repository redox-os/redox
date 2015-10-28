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
   AudioFileManager.cpp
*/
#include "AudioFilePlayer.h"
#include <mach/mach.h> /* used for setting policy of thread */
#include "SDLOSXCAGuard.h"
#include <pthread.h>

/*#include <list>*/

/*typedef void *FileData;*/
typedef struct S_FileData
{
    AudioFileManager *obj;
    struct S_FileData *next;
} FileData;


typedef struct S_FileReaderThread {
/*public:*/
    SDLOSXCAGuard*                    (*GetGuard)(struct S_FileReaderThread *frt);
    void                        (*AddReader)(struct S_FileReaderThread *frt);
    void                        (*RemoveReader)(struct S_FileReaderThread *frt, AudioFileManager* inItem);
    int                         (*TryNextRead)(struct S_FileReaderThread *frt, AudioFileManager* inItem);

    int     mThreadShouldDie;
    
/*private:*/
    /*typedef std::list<AudioFileManager*> FileData;*/

    SDLOSXCAGuard             *mGuard;
    UInt32              mThreadPriority;
    
    int                 mNumReaders;    
    FileData            *mFileData;


    void                        (*ReadNextChunk)(struct S_FileReaderThread *frt);
    int                         (*StartFixedPriorityThread)(struct S_FileReaderThread *frt);
    /*static*/
    UInt32               (*GetThreadBasePriority)(pthread_t inThread);
    /*static*/
    void*                (*DiskReaderEntry)(void *inRefCon);
} FileReaderThread;


static SDLOSXCAGuard* FileReaderThread_GetGuard(FileReaderThread *frt)
{
    return frt->mGuard;
}

/* returns 1 if succeeded */
static int FileReaderThread_TryNextRead (FileReaderThread *frt, AudioFileManager* inItem)
{
    int didLock = 0;
    int succeeded = 0;
    if (frt->mGuard->Try(frt->mGuard, &didLock))
    {
        /*frt->mFileData.push_back (inItem);*/
        /* !!! FIXME: this could be faster with a "tail" member. --ryan. */
        FileData *i = frt->mFileData;
        FileData *prev = NULL;

        FileData *newfd = (FileData *) SDL_malloc(sizeof (FileData));
        newfd->obj = inItem;
        newfd->next = NULL;

        while (i != NULL) { prev = i; i = i->next; }
        if (prev == NULL)
            frt->mFileData = newfd;
        else
            prev->next = newfd;

        frt->mGuard->Notify(frt->mGuard);
        succeeded = 1;

        if (didLock)
            frt->mGuard->Unlock(frt->mGuard);
    }
                
    return succeeded;
}

static void    FileReaderThread_AddReader(FileReaderThread *frt)
{
    if (frt->mNumReaders == 0)
    {
        frt->mThreadShouldDie = 0;
        frt->StartFixedPriorityThread (frt);
    }
    frt->mNumReaders++;
}

static void    FileReaderThread_RemoveReader (FileReaderThread *frt, AudioFileManager* inItem)
{
    if (frt->mNumReaders > 0)
    {
        int bNeedsRelease = frt->mGuard->Lock(frt->mGuard);
        
        /*frt->mFileData.remove (inItem);*/
        FileData *i = frt->mFileData;
        FileData *prev = NULL;
        while (i != NULL)
        {
            FileData *next = i->next;
            if (i->obj != inItem)
                prev = i;
            else
            {
                if (prev == NULL)
                    frt->mFileData = next;
                else
                    prev->next = next;
                SDL_free(i);
            }
            i = next;
        }

        if (--frt->mNumReaders == 0) {
            frt->mThreadShouldDie = 1;
            frt->mGuard->Notify(frt->mGuard); /* wake up thread so it will quit */
            frt->mGuard->Wait(frt->mGuard);   /* wait for thread to die */
        }

        if (bNeedsRelease) frt->mGuard->Unlock(frt->mGuard);
    }   
}

static int    FileReaderThread_StartFixedPriorityThread (FileReaderThread *frt)
{
    pthread_attr_t      theThreadAttrs;
    pthread_t           pThread;

    OSStatus result = pthread_attr_init(&theThreadAttrs);
        if (result) return 0; /*THROW_RESULT("pthread_attr_init - Thread attributes could not be created.")*/
    
    result = pthread_attr_setdetachstate(&theThreadAttrs, PTHREAD_CREATE_DETACHED);
        if (result) return 0; /*THROW_RESULT("pthread_attr_setdetachstate - Thread attributes could not be detached.")*/
    
    result = pthread_create (&pThread, &theThreadAttrs, frt->DiskReaderEntry, frt);
        if (result) return 0; /*THROW_RESULT("pthread_create - Create and start the thread.")*/
    
    pthread_attr_destroy(&theThreadAttrs);
    
    /* we've now created the thread and started it
       we'll now set the priority of the thread to the nominated priority
       and we'll also make the thread fixed */
    thread_extended_policy_data_t       theFixedPolicy;
    thread_precedence_policy_data_t     thePrecedencePolicy;
    SInt32                              relativePriority;
    
    /* make thread fixed */
    theFixedPolicy.timeshare = 0;   /* set to 1 for a non-fixed thread */
    result = thread_policy_set (pthread_mach_thread_np(pThread), THREAD_EXTENDED_POLICY, (thread_policy_t)&theFixedPolicy, THREAD_EXTENDED_POLICY_COUNT);
        if (result) return 0; /*THROW_RESULT("thread_policy - Couldn't set thread as fixed priority.")*/
    /* set priority */
    /* precedency policy's "importance" value is relative to spawning thread's priority */
    relativePriority = frt->mThreadPriority - frt->GetThreadBasePriority(pthread_self());
        
    thePrecedencePolicy.importance = relativePriority;
    result = thread_policy_set (pthread_mach_thread_np(pThread), THREAD_PRECEDENCE_POLICY, (thread_policy_t)&thePrecedencePolicy, THREAD_PRECEDENCE_POLICY_COUNT);
        if (result) return 0; /*THROW_RESULT("thread_policy - Couldn't set thread priority.")*/

    return 1;
}

static UInt32  FileReaderThread_GetThreadBasePriority (pthread_t inThread)
{
    thread_basic_info_data_t            threadInfo;
    policy_info_data_t                  thePolicyInfo;
    unsigned int                        count;
    
    /* get basic info */
    count = THREAD_BASIC_INFO_COUNT;
    thread_info (pthread_mach_thread_np (inThread), THREAD_BASIC_INFO, (integer_t*)&threadInfo, &count);
    
    switch (threadInfo.policy) {
        case POLICY_TIMESHARE:
            count = POLICY_TIMESHARE_INFO_COUNT;
            thread_info(pthread_mach_thread_np (inThread), THREAD_SCHED_TIMESHARE_INFO, (integer_t*)&(thePolicyInfo.ts), &count);
            return thePolicyInfo.ts.base_priority;
            break;
            
        case POLICY_FIFO:
            count = POLICY_FIFO_INFO_COUNT;
            thread_info(pthread_mach_thread_np (inThread), THREAD_SCHED_FIFO_INFO, (integer_t*)&(thePolicyInfo.fifo), &count);
            if (thePolicyInfo.fifo.depressed) {
                return thePolicyInfo.fifo.depress_priority;
            } else {
                return thePolicyInfo.fifo.base_priority;
            }
            break;
            
        case POLICY_RR:
            count = POLICY_RR_INFO_COUNT;
            thread_info(pthread_mach_thread_np (inThread), THREAD_SCHED_RR_INFO, (integer_t*)&(thePolicyInfo.rr), &count);
            if (thePolicyInfo.rr.depressed) {
                return thePolicyInfo.rr.depress_priority;
            } else {
                return thePolicyInfo.rr.base_priority;
            }
            break;
    }
    
    return 0;
}

static void    *FileReaderThread_DiskReaderEntry (void *inRefCon)
{
    FileReaderThread *frt = (FileReaderThread *)inRefCon;
    frt->ReadNextChunk(frt);
    #if DEBUG
    printf ("finished with reading file\n");
    #endif
    
    return 0;
}

static void    FileReaderThread_ReadNextChunk (FileReaderThread *frt)
{
    OSStatus result;
    ByteCount dataChunkSize;
    AudioFileManager* theItem = 0;

    for (;;) 
    {
        { /* this is a scoped based lock */
            int bNeedsRelease = frt->mGuard->Lock(frt->mGuard);
            
            if (frt->mThreadShouldDie) {
                frt->mGuard->Notify(frt->mGuard);
                if (bNeedsRelease) frt->mGuard->Unlock(frt->mGuard);
                return;
            }
            
            /*if (frt->mFileData.empty())*/
            if (frt->mFileData == NULL)
            {
                frt->mGuard->Wait(frt->mGuard);
            }
                        
            /* kill thread */
            if (frt->mThreadShouldDie) {
            
                frt->mGuard->Notify(frt->mGuard);
                if (bNeedsRelease) frt->mGuard->Unlock(frt->mGuard);
                return;
            }

            /*theItem = frt->mFileData.front();*/
            /*frt->mFileData.pop_front();*/
            theItem = NULL;
            if (frt->mFileData != NULL)
            {
                FileData *next = frt->mFileData->next;
                theItem = frt->mFileData->obj;
                SDL_free(frt->mFileData);
                frt->mFileData = next;
            }

            if (bNeedsRelease) frt->mGuard->Unlock(frt->mGuard);
        }
    
        if ((theItem->mFileLength - theItem->mReadFilePosition) < theItem->mChunkSize)
            dataChunkSize = theItem->mFileLength - theItem->mReadFilePosition;
        else
            dataChunkSize = theItem->mChunkSize;
        
            /* this is the exit condition for the thread */
        if (dataChunkSize <= 0) {
            theItem->mFinishedReadingData = 1;
            continue;
        }
            /* construct pointer */
        char* writePtr = (char *) (theItem->GetFileBuffer(theItem) +
                                (theItem->mWriteToFirstBuffer ? 0 : theItem->mChunkSize));
    
            /* read data */
        result = theItem->Read(theItem, writePtr, &dataChunkSize);
        if (result != noErr && result != eofErr) {
            AudioFilePlayer *afp = (AudioFilePlayer *) theItem->GetParent(theItem);
            afp->DoNotification(afp, result);
            continue;
        }
        
        if (dataChunkSize != theItem->mChunkSize)
        {
            writePtr += dataChunkSize;

            /* can't exit yet.. we still have to pass the partial buffer back */
            SDL_memset(writePtr, 0, (theItem->mChunkSize - dataChunkSize));
        }
        
        theItem->mWriteToFirstBuffer = !theItem->mWriteToFirstBuffer;   /* switch buffers */
        
        if (result == eofErr)
            theItem->mReadFilePosition = theItem->mFileLength;
        else
            theItem->mReadFilePosition += dataChunkSize;        /* increment count */
    }
}

void delete_FileReaderThread(FileReaderThread *frt)
{
    if (frt != NULL)
    {
        delete_SDLOSXCAGuard(frt->mGuard);
        SDL_free(frt);
    }
}

FileReaderThread *new_FileReaderThread ()
{
    FileReaderThread *frt = (FileReaderThread *) SDL_malloc(sizeof (FileReaderThread));
    if (frt == NULL)
        return NULL;
    SDL_memset(frt, '\0', sizeof (*frt));

    frt->mGuard = new_SDLOSXCAGuard();
    if (frt->mGuard == NULL)
    {
        SDL_free(frt);
        return NULL;
    }

    #define SET_FILEREADERTHREAD_METHOD(m) frt->m = FileReaderThread_##m
    SET_FILEREADERTHREAD_METHOD(GetGuard);
    SET_FILEREADERTHREAD_METHOD(AddReader);
    SET_FILEREADERTHREAD_METHOD(RemoveReader);
    SET_FILEREADERTHREAD_METHOD(TryNextRead);
    SET_FILEREADERTHREAD_METHOD(ReadNextChunk);
    SET_FILEREADERTHREAD_METHOD(StartFixedPriorityThread);
    SET_FILEREADERTHREAD_METHOD(GetThreadBasePriority);
    SET_FILEREADERTHREAD_METHOD(DiskReaderEntry);
    #undef SET_FILEREADERTHREAD_METHOD

    frt->mThreadPriority = 62;
    return frt;
}


static FileReaderThread *sReaderThread;


static int    AudioFileManager_DoConnect (AudioFileManager *afm)
{
    if (!afm->mIsEngaged)
    {
        OSStatus result;

        /*afm->mReadFilePosition = 0;*/
        afm->mFinishedReadingData = 0;

        afm->mNumTimesAskedSinceFinished = 0;
        afm->mLockUnsuccessful = 0;
        
        ByteCount dataChunkSize;
        
        if ((afm->mFileLength - afm->mReadFilePosition) < afm->mChunkSize)
            dataChunkSize = afm->mFileLength - afm->mReadFilePosition;
        else
            dataChunkSize = afm->mChunkSize;
        
        result = afm->Read(afm, afm->mFileBuffer, &dataChunkSize);
           if (result) return 0; /*THROW_RESULT("AudioFileManager::DoConnect(): Read")*/

        afm->mReadFilePosition += dataChunkSize;
                
        afm->mWriteToFirstBuffer = 0;
        afm->mReadFromFirstBuffer = 1;

        sReaderThread->AddReader(sReaderThread);
        
        afm->mIsEngaged = 1;
    }
    /*
    else
        throw static_cast<OSStatus>(-1); */ /* thread has already been started */

    return 1;
}

static void    AudioFileManager_Disconnect (AudioFileManager *afm)
{
    if (afm->mIsEngaged)
    {
        sReaderThread->RemoveReader (sReaderThread, afm);
        afm->mIsEngaged = 0;
    }
}

static OSStatus AudioFileManager_Read(AudioFileManager *afm, char *buffer, ByteCount *len)
{
    return FSReadFork (afm->mForkRefNum,
                       fsFromStart,
                       afm->mReadFilePosition + afm->mAudioDataOffset,
                       *len,
                       buffer,
                       len);
}

static OSStatus AudioFileManager_GetFileData (AudioFileManager *afm, void** inOutData, UInt32 *inOutDataSize)
{
    if (afm->mFinishedReadingData)
    {
        ++afm->mNumTimesAskedSinceFinished;
        *inOutDataSize = 0;
        *inOutData = 0;
        return noErr;
    }
    
    if (afm->mReadFromFirstBuffer == afm->mWriteToFirstBuffer) {
        #if DEBUG
        printf ("* * * * * * * Can't keep up with reading file\n");
        #endif
        
        afm->mParent->DoNotification (afm->mParent, kAudioFilePlayErr_FilePlayUnderrun);
        *inOutDataSize = 0;
        *inOutData = 0;
    } else {
        *inOutDataSize = afm->mChunkSize;
        *inOutData = afm->mReadFromFirstBuffer ? afm->mFileBuffer : (afm->mFileBuffer + afm->mChunkSize);
    }

    afm->mLockUnsuccessful = !sReaderThread->TryNextRead (sReaderThread, afm);
    
    afm->mReadFromFirstBuffer = !afm->mReadFromFirstBuffer;

    return noErr;
}

static void    AudioFileManager_AfterRender (AudioFileManager *afm)
{
    if (afm->mNumTimesAskedSinceFinished > 0)
    {
        int didLock = 0;
        SDLOSXCAGuard *guard = sReaderThread->GetGuard(sReaderThread);
        if (guard->Try(guard, &didLock)) {
            afm->mParent->DoNotification (afm->mParent, kAudioFilePlay_FileIsFinished);
            if (didLock)
                guard->Unlock(guard);
        }
    }

    if (afm->mLockUnsuccessful)
        afm->mLockUnsuccessful = !sReaderThread->TryNextRead (sReaderThread, afm);
}

static void    AudioFileManager_SetPosition (AudioFileManager *afm, SInt64 pos)
{
    if (pos < 0 || pos >= afm->mFileLength) {
        SDL_SetError ("AudioFileManager::SetPosition - position invalid: %d filelen=%d\n", 
            (unsigned int)pos, (unsigned int)afm->mFileLength);
        pos = 0;
    }
        
    afm->mReadFilePosition = pos;
}
    
static void    AudioFileManager_SetEndOfFile (AudioFileManager *afm, SInt64 pos)
{
    if (pos <= 0 || pos > afm->mFileLength) {
        SDL_SetError ("AudioFileManager::SetEndOfFile - position beyond actual eof\n");
        pos = afm->mFileLength;
    }
    
    afm->mFileLength = pos;
}

static const char *AudioFileManager_GetFileBuffer(AudioFileManager *afm)
{
    return afm->mFileBuffer;
}

const AudioFilePlayer *AudioFileManager_GetParent(AudioFileManager *afm)
{
    return afm->mParent;
}

static int AudioFileManager_GetByteCounter(AudioFileManager *afm)
{
    return afm->mByteCounter;
}

static OSStatus    AudioFileManager_FileInputProc (void                  *inRefCon,
                                         AudioUnitRenderActionFlags      *ioActionFlags,
                                         const AudioTimeStamp            *inTimeStamp,
                                         UInt32                          inBusNumber,
                                         UInt32                          inNumberFrames,
                                         AudioBufferList                 *ioData)
{
    AudioFileManager* afm = (AudioFileManager*)inRefCon;
    return afm->Render(afm, ioData);
}

static OSStatus    AudioFileManager_Render (AudioFileManager *afm, AudioBufferList *ioData)
{
    OSStatus result = noErr;
    AudioBuffer *abuf;
    UInt32 i;

    for (i = 0; i < ioData->mNumberBuffers; i++) {
        abuf = &ioData->mBuffers[i];
        if (afm->mBufferOffset >= afm->mBufferSize) {
            result = afm->GetFileData(afm, &afm->mTmpBuffer, &afm->mBufferSize);
            if (result) {
                SDL_SetError ("AudioConverterFillBuffer:%ld\n", result);
                afm->mParent->DoNotification(afm->mParent, result);
                return result;
            }

            afm->mBufferOffset = 0;
        }

        if (abuf->mDataByteSize > afm->mBufferSize - afm->mBufferOffset)
            abuf->mDataByteSize = afm->mBufferSize - afm->mBufferOffset;
        abuf->mData = (char *)afm->mTmpBuffer + afm->mBufferOffset;
        afm->mBufferOffset += abuf->mDataByteSize;
    
        afm->mByteCounter += abuf->mDataByteSize;
        afm->AfterRender(afm);
    }
    return result;
}


void delete_AudioFileManager (AudioFileManager *afm)
{
    if (afm != NULL) {
        if (afm->mFileBuffer) {
            free(afm->mFileBuffer);
        }

        SDL_free(afm);
    }
}


AudioFileManager *new_AudioFileManager(AudioFilePlayer *inParent,
                                       SInt16          inForkRefNum,
                                       SInt64          inFileLength,
                                       UInt32          inChunkSize)
{
    AudioFileManager *afm;

    if (sReaderThread == NULL)
    {
        sReaderThread = new_FileReaderThread();
        if (sReaderThread == NULL)
            return NULL;
    }

    afm = (AudioFileManager *) SDL_malloc(sizeof (AudioFileManager));
    if (afm == NULL)
        return NULL;
    SDL_memset(afm, '\0', sizeof (*afm));

    #define SET_AUDIOFILEMANAGER_METHOD(m) afm->m = AudioFileManager_##m
    SET_AUDIOFILEMANAGER_METHOD(Disconnect);
    SET_AUDIOFILEMANAGER_METHOD(DoConnect);
    SET_AUDIOFILEMANAGER_METHOD(Read);
    SET_AUDIOFILEMANAGER_METHOD(GetFileBuffer);
    SET_AUDIOFILEMANAGER_METHOD(GetParent);
    SET_AUDIOFILEMANAGER_METHOD(SetPosition);
    SET_AUDIOFILEMANAGER_METHOD(GetByteCounter);
    SET_AUDIOFILEMANAGER_METHOD(SetEndOfFile);
    SET_AUDIOFILEMANAGER_METHOD(Render);
    SET_AUDIOFILEMANAGER_METHOD(GetFileData);
    SET_AUDIOFILEMANAGER_METHOD(AfterRender);
    SET_AUDIOFILEMANAGER_METHOD(FileInputProc);
    #undef SET_AUDIOFILEMANAGER_METHOD

    afm->mParent = inParent;
    afm->mForkRefNum = inForkRefNum;
    afm->mBufferSize = inChunkSize;
    afm->mBufferOffset = inChunkSize;
    afm->mChunkSize = inChunkSize;
    afm->mFileLength = inFileLength;
    afm->mFileBuffer = (char*) SDL_malloc(afm->mChunkSize * 2);
    FSGetForkPosition(afm->mForkRefNum, &afm->mAudioDataOffset);
    assert (afm->mFileBuffer != NULL);
    return afm;
}

