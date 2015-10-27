#ifndef STREAMPLAYER_H
#define STREAMPLAYER_H

#include<MdaAudioOutputStream.h>

const TInt KSilenceBuffer = 256;

class MStreamObs
    {
    public:
    	enum 
    	{
    	EInit, 
    	EPlay,
    	EWrite,
    	EClose,	
    	};
        virtual void Complete(TInt aState, TInt aError) = 0;
    };

class MStreamProvider
	{
	public:
		virtual TPtrC8 Data() = 0;	
	};

NONSHARABLE_CLASS(CStreamPlayer) : public CBase, public MMdaAudioOutputStreamCallback
	{
	public:
		CStreamPlayer(MStreamProvider& aProvider, MStreamObs& aObs);
		~CStreamPlayer();
		void ConstructL();
		
		static TInt ClosestSupportedRate(TInt aRate);
		
		TInt OpenStream(TInt aRate, TInt aChannels, TUint32 aType = KMMFFourCCCodePCM16);
		
		void SetVolume(TInt aNew);
		TInt Volume() const;
		TInt MaxVolume() const;
		
		void Stop();
		void Start();
		void Open();
		void Close();
		
		TBool Playing() const;
		TBool Closed() const;
		
	private:

		void MaoscOpenComplete(TInt aError) ;
		void MaoscBufferCopied(TInt aError, const TDesC8& aBuffer);
		void MaoscPlayComplete(TInt aError);
	
	private:
		void Request();
		void SetCapsL();

	private:
		MStreamProvider& iProvider;
		MStreamObs& iObs;	
		TInt iVolume;
	
		CMdaAudioOutputStream* iStream;
	
		TInt iRate;
		TInt iChannels;
		TUint32 iType;
		
		enum 
			{
				ENone = 0,
				EInited = 0x1,
				EStarted = 0x2,
				EStopped = 0x4,
				EVolumeChange = 0x8,
				EDied		  = 0x10
			};
		
		TInt iState;
		TBuf8<KSilenceBuffer> iSilence;
		TPtrC8 iPtr;
	
	};


#endif

