#include "streamplayer.h"
#include<mda/common/audio.h>



const TInt KMaxVolume(256);

LOCAL_C TInt GetSampleRate(TInt aRate)
    {
    switch(aRate)
        {
        case 8000: return TMdaAudioDataSettings::ESampleRate8000Hz;
        case 11025: return TMdaAudioDataSettings::ESampleRate11025Hz;
        case 12000: return TMdaAudioDataSettings::ESampleRate12000Hz;
        case 16000: return TMdaAudioDataSettings::ESampleRate16000Hz;
        case 22050: return TMdaAudioDataSettings::ESampleRate22050Hz;
        case 24000: return TMdaAudioDataSettings::ESampleRate24000Hz;
        case 32000: return TMdaAudioDataSettings::ESampleRate32000Hz;
        case 44100: return TMdaAudioDataSettings::ESampleRate44100Hz;
        case 48000: return TMdaAudioDataSettings::ESampleRate48000Hz;
        case 96000: return TMdaAudioDataSettings::ESampleRate96000Hz;
        case 64000: return TMdaAudioDataSettings::ESampleRate64000Hz;
        }
    return KErrNotFound;
    }

LOCAL_C TInt GetChannels(TInt aChannels)
    {
    switch(aChannels)
        {
        case 1: return TMdaAudioDataSettings::EChannelsMono;
        case 2: return TMdaAudioDataSettings::EChannelsStereo;
        }
    return KErrNotFound;
    }
    
TInt CStreamPlayer::ClosestSupportedRate(TInt aRate)
	{
	if(aRate > 96000)
		return 96000;
	TInt rate = aRate;
	while(GetSampleRate(rate) == KErrNotFound)
		{
		++rate;
		}
	return rate;
	}

CStreamPlayer::CStreamPlayer(MStreamProvider& aProvider,  MStreamObs& aObs) :
 iProvider(aProvider), iObs(aObs), iVolume(KMaxVolume)
	{	
	}
		
CStreamPlayer::~CStreamPlayer()
	{
	iState |= EDied;
	if(iState & EInited)
		Close();
	User::After(100000); //wait buffer to be flushed
	ASSERT(iPtr.Length() == 0);
	delete iStream;
	}
		
		
void CStreamPlayer::ConstructL()
	{
	iStream = CMdaAudioOutputStream::NewL(*this, EMdaPriorityMax);
	iSilence.SetMax();
	iSilence.FillZ();
	}
		

TInt CStreamPlayer::OpenStream(TInt aRate, TInt aChannels, TUint32 aType)
	{
	Close();
	
	iType = aType;
	
	iRate = GetSampleRate(aRate);
	if(iRate == KErrNotFound)
		return KErrNotSupported;
	
	iChannels = GetChannels(aChannels);
	if(iChannels == KErrNotFound)
		return KErrNotSupported;
	
	Open();
    
    return KErrNone;
	}


TInt CStreamPlayer::MaxVolume() const
	{
	return KMaxVolume;
	}

void CStreamPlayer::SetVolume(TInt aNew)
    {
    
    const TInt maxi = MaxVolume();
    if(aNew > maxi)
		return;
    if(aNew < 0)
    	return;
    
    iVolume = aNew;
    
    iState |= EVolumeChange;
    }
    
 TInt CStreamPlayer::Volume() const
    {
	return iVolume;
    }

void CStreamPlayer::Open()
	{
	TMdaAudioDataSettings audioSettings;
	audioSettings.Query();
    audioSettings.iCaps = TMdaAudioDataSettings::ERealTime |
     	TMdaAudioDataSettings::ESampleRateFixed; 
    audioSettings.iSampleRate = iRate; 
    audioSettings.iChannels = iChannels;
	audioSettings.iFlags = TMdaAudioDataSettings::ENoNetworkRouting;
	audioSettings.iVolume = 0;
	
	iState &= ~EStopped;
    iStream->Open(&audioSettings);    
	}
	
void CStreamPlayer::Stop()
	{
	if(iState & (EStarted | EInited))
		{
		Close();
		iState |= EStopped;
		}
	}
	
void CStreamPlayer::Start()
	{
	if(iPtr.Length() == 0)
		{
		iState |= EStarted;
		if(iState & EInited)
			{
			Request();
			}
		else if(iState & EStopped)
			{
			Open();
			}
		}
	}

void CStreamPlayer::Close()
	{
	iState &= ~EInited;
	iStream->Stop();
	iState &= ~EStarted;
	}
	
void CStreamPlayer::Request()
	{
	if(iState & EInited)
		{
		iPtr.Set(KNullDesC8);
		
		if(iState & EVolumeChange)
			{
       		const TReal newVol = iVolume;
       		const TReal newMax = MaxVolume();
       		const TInt maxVol = iStream->MaxVolume();
       		const TReal max = static_cast<TReal>(maxVol);
       		const TReal newvolume = (newVol * max) / newMax;
       		const TInt vol = static_cast<TReal>(newvolume);
       		iStream->SetVolume(vol);
			iState &= ~EVolumeChange;
			}
			
		if(iState & EStarted)
			{
	 		iPtr.Set(iProvider.Data());
			}
		if(iPtr.Length() == 0)
			{
			iPtr.Set(iSilence);
			}
		TRAPD(err, iStream->WriteL(iPtr));
		if(err != KErrNone)
			{
			iObs.Complete(MStreamObs::EWrite, err);	
			}
	/*	else
			{
			iProvider.Written(iPtr.Length());
			}*/
		}
	}
	

void CStreamPlayer::SetCapsL()
	{
	iStream->SetDataTypeL(iType);
	iStream->SetAudioPropertiesL(iRate, iChannels);
	}

void CStreamPlayer::MaoscOpenComplete(TInt aError) 
	{
	if(aError == KErrNone)
		{
		TRAPD(err, SetCapsL());
    	if(err == KErrNone)
    		{
    		iStream->SetPriority(EPriorityNormal, EMdaPriorityPreferenceTime);
    		iState |= EInited;
    		
    		
        	SetVolume(Volume());	
    		
    		if(iState & EStarted)
    			{
    			Request();
    			}
    			
			}
		aError = err;
		}
	if(!(iState & EDied))
		iObs.Complete(MStreamObs::EInit, aError);
	}
	
void CStreamPlayer::MaoscBufferCopied(TInt aError, const TDesC8& /*aBuffer*/)
	{
	iPtr.Set(KNullDesC8);
	if(aError == KErrNone)
		{
		if(iState & EInited)
			Request();
		else
			iStream->Stop();
		}
	else if(!(iState & EDied))
		iObs.Complete(MStreamObs::EPlay, aError);
	}
	
void CStreamPlayer::MaoscPlayComplete(TInt aError)
	{	
	iPtr.Set(KNullDesC8);
	iState &= ~EStarted;
	if(!(iState & EDied))
		iObs.Complete(MStreamObs::EClose, aError);
	}
	
TBool CStreamPlayer::Playing() const
	{
	return (iState & EInited) && (iState & EStarted);
	}

TBool CStreamPlayer::Closed() const
	{
	return !(iState & EInited) && !(iState & EDied);
	}
	
	/*
void CStreamPlayer::Request()
	{
	SetActive();
	TRequestStatus* s = &iStatus;
	User::RequestComplete(s, KErrNone);
	}
	//		iTimer.After(0);
	*/
	




