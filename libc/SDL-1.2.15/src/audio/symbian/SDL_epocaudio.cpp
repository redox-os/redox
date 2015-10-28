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
    slouken@devolution.com
*/

/*
    SDL_epocaudio.cpp
    Epoc based SDL audio driver implementation
    
    Markus Mertama
*/

#ifdef SAVE_RCSID
static char rcsid =
 "@(#) $Id: SDL_epocaudio.c,v 0.0.0.0 2001/06/19 17:19:56 hercules Exp $";
#endif


#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <errno.h>
#include <unistd.h>
#include <fcntl.h>
#include <signal.h>
#include <sys/time.h>
#include <sys/ioctl.h>
#include <sys/stat.h>

#include "epoc_sdl.h"

#include <e32hal.h>


extern "C" {
#include "SDL_audio.h"
#include "SDL_error.h"
#include "SDL_audiomem.h"
#include "SDL_audio_c.h"
#include "SDL_timer.h"
#include "SDL_audiodev_c.h"
}

#include "SDL_epocaudio.h"

#include "streamplayer.h"


//#define DEBUG_AUDIO


/* Audio driver functions */

static int EPOC_OpenAudio(SDL_AudioDevice *thisdevice, SDL_AudioSpec *spec);
static void EPOC_WaitAudio(SDL_AudioDevice *thisdevice);
static void EPOC_PlayAudio(SDL_AudioDevice *thisdevice);
static Uint8 *EPOC_GetAudioBuf(SDL_AudioDevice *thisdevice);
static void EPOC_CloseAudio(SDL_AudioDevice *thisdevice);
static void EPOC_ThreadInit(SDL_AudioDevice *thisdevice);

static int Audio_Available(void);
static SDL_AudioDevice *Audio_CreateDevice(int devindex);
static void Audio_DeleteDevice(SDL_AudioDevice *device);


//void sos_adump(SDL_AudioDevice* thisdevice, void* data, int len);

#ifdef __WINS__
#define DODUMP
#endif

#ifdef DODUMP
NONSHARABLE_CLASS(TDump)
	{
	public:
	TInt Open();
	void Close();
	void Dump(const TDesC8& aDes);
	private:
		RFile iFile;
    	RFs iFs; 
	};
	
TInt TDump::Open()
	{
	TInt err = iFs.Connect();
	if(err == KErrNone)
		{
#ifdef __WINS__
_LIT(target, "C:\\sdlau.raw");
#else
_LIT(target, "E:\\sdlau.raw");
#endif 
		err = iFile.Replace(iFs, target, EFileWrite);
		}
	return err;
	}
void TDump::Close()
	{
	iFile.Close();
	iFs.Close();
	}
void TDump::Dump(const TDesC8& aDes)
	{
	iFile.Write(aDes);
	}
#endif


NONSHARABLE_CLASS(CSimpleWait) : public CTimer
	{
	public:
		void Wait(TTimeIntervalMicroSeconds32 aWait);
		static CSimpleWait* NewL();
	private:
		CSimpleWait();
		void RunL();
	};


CSimpleWait* CSimpleWait::NewL()
	{
	CSimpleWait* wait = new (ELeave) CSimpleWait();
	CleanupStack::PushL(wait);
	wait->ConstructL();
	CleanupStack::Pop();
	return wait;
	}

void CSimpleWait::Wait(TTimeIntervalMicroSeconds32 aWait)
	{
	After(aWait);
	CActiveScheduler::Start();
	}
	
CSimpleWait::CSimpleWait() : CTimer(CActive::EPriorityStandard)	
	{
	CActiveScheduler::Add(this);
	}

void CSimpleWait::RunL()
	{
	CActiveScheduler::Stop();
	}

const TInt KAudioBuffers(2);
	

NONSHARABLE_CLASS(CEpocAudio) : public CBase, public MStreamObs, public MStreamProvider
    {
    public:
    	static void* NewL(TInt BufferSize, TInt aFill);
    	inline static CEpocAudio& Current(SDL_AudioDevice* thisdevice);
    	
    	static void Free(SDL_AudioDevice* thisdevice);
 		
    	void Wait();
    	void Play();
    //	void SetBuffer(const TDesC8& aBuffer);
    	void ThreadInitL(TAny* aDevice);
    	void Open(TInt iRate, TInt iChannels, TUint32 aType, TInt aBytes);
    	~CEpocAudio();
    	TUint8* Buffer();
    	TBool SetPause(TBool aPause);
    #ifdef DODUMP
    	void Dump(const TDesC8& aBuf) {iDump.Dump(aBuf);}
    #endif
    private:
    	CEpocAudio(TInt aBufferSize);
    	void Complete(TInt aState, TInt aError);
    	TPtrC8 Data();
    	void ConstructL(TInt aFill);
    private:
    	TInt iBufferSize;
    	CStreamPlayer* iPlayer;
    	TInt iBufferRate;
    	TInt iRate;
    	TInt iChannels;
    	TUint32 iType;
    	TInt iPosition;
    	TThreadId iTid;
    	TUint8* iAudioPtr;
    	TUint8* iBuffer;
    //	TTimeIntervalMicroSeconds iStart;
    	TTime iStart;
    	TInt iTune;
    	CSimpleWait* iWait;
    #ifdef DODUMP
    	TDump iDump;
    #endif
    };

inline CEpocAudio& CEpocAudio::Current(SDL_AudioDevice* thisdevice)
	{
	return *static_cast<CEpocAudio*>((void*)thisdevice->hidden);
	}
	
/*

TBool EndSc(TAny*)
	{	
	CActiveScheduler::Stop();
	}
	
LOCAL_C void CleanScL()
	{
	CIdle* d = CIdle::NewLC(CActive:::EPriorityIdle);
	d->Start(TCallBack(EndSc));
	CActiveScheduler::Start();
	
	}
*/
	
void CEpocAudio::Free(SDL_AudioDevice* thisdevice)
	{
    CEpocAudio* ea = static_cast<CEpocAudio*>((void*)thisdevice->hidden);
    if(ea)
    	{
		ASSERT(ea->iTid == RThread().Id());
    	delete ea;
    	thisdevice->hidden = NULL;	
   
    	CActiveScheduler* as =  CActiveScheduler::Current();
    	ASSERT(as->StackDepth() == 0);    	
    	delete as;
    	CActiveScheduler::Install(NULL);
    	}
    ASSERT(thisdevice->hidden == NULL);
	}
	
CEpocAudio::CEpocAudio(TInt aBufferSize) : iBufferSize(aBufferSize), iPosition(-1) 
	{
	}

void* CEpocAudio::NewL(TInt aBufferSize, TInt aFill)
	{
	CEpocAudio* eAudioLib = new (ELeave) CEpocAudio(aBufferSize);
	CleanupStack::PushL(eAudioLib);
	eAudioLib->ConstructL(aFill);
	CleanupStack::Pop();
	return eAudioLib;
	}
	
void CEpocAudio::ConstructL(TInt aFill)
	{
	iBuffer = (TUint8*) User::AllocL(KAudioBuffers * iBufferSize);
	memset(iBuffer, aFill, KAudioBuffers * iBufferSize);
	iAudioPtr = iBuffer;
	}


TBool CEpocAudio::SetPause(TBool aPause)
	{
	if(aPause && iPosition >= 0)
		{
		iPosition = -1;
		if(iPlayer != NULL)
			iPlayer->Stop();
		}
	if(!aPause && iPosition < 0)
		{
		iPosition = 0;
		if(iPlayer != NULL)
			iPlayer->Start();
		}
	return iPosition < 0;
	}
	
void CEpocAudio::ThreadInitL(TAny* aDevice)
	{
	iTid = RThread().Id(); 
	CActiveScheduler* as =  new (ELeave) CActiveScheduler();
    CActiveScheduler::Install(as);
    
    EpocSdlEnv::AppendCleanupItem(TSdlCleanupItem((TSdlCleanupOperation)EPOC_CloseAudio, aDevice));
   
    iWait = CSimpleWait::NewL();
   
    iPlayer = new (ELeave) CStreamPlayer(*this, *this);
    iPlayer->ConstructL();	
    iPlayer->OpenStream(iRate, iChannels, iType);
    
    #ifdef DODUMP
    User::LeaveIfError(iDump.Open());
    #endif
	}
	
	
	
TUint8* CEpocAudio::Buffer()
	{
	iStart.UniversalTime();
//	iStart = iPlayer->Position();		
	return iAudioPtr;

	}
	
CEpocAudio::~CEpocAudio()
	{
	if(iWait != NULL)
		iWait->Cancel();
	delete iWait; 
	if(iPlayer != NULL)
		iPlayer->Close();
	delete iPlayer;
	delete iBuffer;
	}
	
void CEpocAudio::Complete(TInt aState, TInt aError)
	{
	if(aState == MStreamObs::EClose)
		{
		}
	if(iPlayer->Closed())
		return;
	switch(aError)
		{
		case KErrUnderflow:
		case KErrInUse:
			iPlayer->Start();
			break;
		case KErrAbort:
			iPlayer->Open();
		}
	}
	

void sos_adump(SDL_AudioDevice* thisdevice, void* data, int len)
	{
#ifdef DODUMP
	const TPtrC8 buf((TUint8*)data, len);
	CEpocAudio::Current(thisdevice).Dump(buf);
#endif
	}

const TInt KClip(256);
	
TPtrC8 CEpocAudio::Data()
	{
	if(iPosition < 0)
		return KNullDesC8();
	
	TPtrC8 data(iAudioPtr + iPosition, KClip);
	
#ifdef DODUMP
	iDump.Dump(data);
#endif
	
	iPosition += KClip;
	if(iPosition >= iBufferSize) 
		{
		
/*		if(iAudioPtr == iBuffer)
			iAudioPtr = iBuffer + iBufferSize;
		else
			iAudioPtr = iBuffer;
*/		
		iAudioPtr += iBufferSize;
		
		if((iAudioPtr - iBuffer) >= KAudioBuffers * iBufferSize)
			iAudioPtr = iBuffer;
		
		iPosition = -1;
		if(iWait->IsActive())
			{
			iWait->Cancel();
			CActiveScheduler::Stop();
			}
		}
	return data;
	}
		


	
void CEpocAudio::Play()
	{
	iPosition = 0;
	}

void CEpocAudio::Wait()
	{
	if(iPosition >= 0 /*&& iPlayer->Playing()*/)
		{
		const TInt64 bufMs = TInt64(iBufferSize - KClip) * TInt64(1000000);
		const TInt64 specTime =  bufMs / TInt64(iRate * iChannels * 2);
		iWait->After(specTime);
		
		CActiveScheduler::Start();
		TTime end;
		end.UniversalTime();
		const TTimeIntervalMicroSeconds delta = end.MicroSecondsFrom(iStart);
	
	
//		const TTimeIntervalMicroSeconds end = iPlayer->Position();
		
		
	
		
		const TInt diff = specTime - delta.Int64();
		
		if(diff > 0 && diff < 200000)
			{
			User::After(diff);
			}
		
		}
	else
		{
	User::After(10000); 
//	iWait->Wait(10000); //just give some time...	
		}	
	}
	
void CEpocAudio::Open(TInt aRate, TInt aChannels, TUint32 aType, TInt aBytes)	
	{
	iRate = aRate;
	iChannels = aChannels;
	iType = aType;
    iBufferRate = iRate * iChannels * aBytes; //1/x
	}
	

/* Audio driver bootstrap functions */

AudioBootStrap EPOCAudio_bootstrap = {
	"epoc\0\0\0",
	"EPOC streaming audio\0\0\0",
	Audio_Available,
	Audio_CreateDevice
};


static SDL_AudioDevice *Audio_CreateDevice(int /*devindex*/)
{
	SDL_AudioDevice *thisdevice;

	/* Initialize all variables that we clean on shutdown */
	thisdevice = (SDL_AudioDevice *)malloc(sizeof(SDL_AudioDevice));
	if ( thisdevice ) {
		memset(thisdevice, 0, (sizeof *thisdevice));
		thisdevice->hidden = NULL; /*(struct SDL_PrivateAudioData *)
			 malloc((sizeof thisdevice->hidden)); */
	}
	if ( (thisdevice == NULL) /*|| (thisdevice->hidden == NULL) */) {
		SDL_OutOfMemory();
		if ( thisdevice ) {
			free(thisdevice);
		}
		return(0);
	}
//	memset(thisdevice->hidden, 0, (sizeof *thisdevice->hidden));

	/* Set the function pointers */
	thisdevice->OpenAudio = EPOC_OpenAudio;
	thisdevice->WaitAudio = EPOC_WaitAudio;
	thisdevice->PlayAudio = EPOC_PlayAudio;
	thisdevice->GetAudioBuf = EPOC_GetAudioBuf;
	thisdevice->CloseAudio = EPOC_CloseAudio;
    thisdevice->ThreadInit = EPOC_ThreadInit;
	thisdevice->free = Audio_DeleteDevice;

	return thisdevice;
}


static void Audio_DeleteDevice(SDL_AudioDevice *device)
    {
	//free(device->hidden);
	free(device);
    }

static int Audio_Available(void)
{
	return(1); // Audio stream modules should be always there!
}


static int EPOC_OpenAudio(SDL_AudioDevice *thisdevice, SDL_AudioSpec *spec)
{
	SDL_TRACE("SDL:EPOC_OpenAudio");

	
	TUint32 type = KMMFFourCCCodePCM16;
	TInt bytes = 2;
	
	switch(spec->format)
		{
		case AUDIO_U16LSB: 
			type = KMMFFourCCCodePCMU16; 
			break;
		case AUDIO_S16LSB: 
			type = KMMFFourCCCodePCM16; 
			break;
		case AUDIO_U16MSB: 
			type = KMMFFourCCCodePCMU16B; 
			break;
		case AUDIO_S16MSB: 
			type = KMMFFourCCCodePCM16B; 
			break; 
			//8 bit not supported!
		case AUDIO_U8: 
		case AUDIO_S8:
		default:
			spec->format = AUDIO_S16LSB;
		};
	

	
	if(spec->channels > 2)
		spec->channels = 2;
	
	spec->freq = CStreamPlayer::ClosestSupportedRate(spec->freq);
	

	/* Allocate mixing buffer */
	const TInt buflen = spec->size;// * bytes * spec->channels;
//	audiobuf = NULL;
    
    TRAPD(err, thisdevice->hidden = static_cast<SDL_PrivateAudioData*>(CEpocAudio::NewL(buflen, spec->silence)));
    if(err != KErrNone)
        return -1;

	CEpocAudio::Current(thisdevice).Open(spec->freq, spec->channels, type, bytes);
	
	CEpocAudio::Current(thisdevice).SetPause(ETrue);
	
   // isSDLAudioPaused = 1;

    thisdevice->enabled = 0; /* enable only after audio engine has been initialized!*/

	/* We're ready to rock and roll. :-) */
	return(0);
}


static void EPOC_CloseAudio(SDL_AudioDevice* thisdevice)
    {
#ifdef DEBUG_AUDIO
    SDL_TRACE("Close audio\n");
#endif

	CEpocAudio::Free(thisdevice);
	}


static void EPOC_ThreadInit(SDL_AudioDevice *thisdevice)
    {
	SDL_TRACE("SDL:EPOC_ThreadInit");
    CEpocAudio::Current(thisdevice).ThreadInitL(thisdevice);
    RThread().SetPriority(EPriorityMore);
    thisdevice->enabled = 1;
    }

/* This function waits until it is possible to write a full sound buffer */
static void EPOC_WaitAudio(SDL_AudioDevice* thisdevice)
{
#ifdef DEBUG_AUDIO
    SDL_TRACE1("wait %d audio\n", CEpocAudio::AudioLib().StreamPlayer(KSfxChannel).SyncTime());
    TInt tics = User::TickCount();
#endif

	CEpocAudio::Current(thisdevice).Wait();

#ifdef DEBUG_AUDIO
    TInt ntics =  User::TickCount() - tics;
    SDL_TRACE1("audio waited %d\n", ntics);
    SDL_TRACE1("audio at %d\n", tics);
#endif
}


 
static void EPOC_PlayAudio(SDL_AudioDevice* thisdevice)
	{
 	if(CEpocAudio::Current(thisdevice).SetPause(SDL_GetAudioStatus() == SDL_AUDIO_PAUSED))
 		SDL_Delay(500); //hold on the busy loop
 	else
 		CEpocAudio::Current(thisdevice).Play();

#ifdef DEBUG_AUDIO
    SDL_TRACE("buffer has audio data\n");
#endif

	
#ifdef DEBUG_AUDIO
	SDL_TRACE1("Wrote %d bytes of audio data\n", buflen);
#endif
}

static Uint8 *EPOC_GetAudioBuf(SDL_AudioDevice* thisdevice)
	{
	return CEpocAudio::Current(thisdevice).Buffer();
	}



