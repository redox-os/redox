#include "dsa.h"
#include "sdlepocapi.h"
#include <cdsb.h>


LOCAL_C TInt BytesPerPixel(TDisplayMode aMode)
	{
	return ((TDisplayModeUtils::NumDisplayModeBitsPerPixel(aMode) - 1) >> 3) + 1; 
	}


NONSHARABLE_CLASS(TDsa)
	{
	public:
		inline TDsa(const CDsa& aDsa);
		inline TBool IsFlip() const;
		inline TBool IsTurn() const;
		inline const TSize& SwSize() const;
		inline void Copy(TUint32* aTarget, const TUint8* aSrc, TInt aBytes, TInt aHeight) const;
	private:
		const CDsa& iDsa;
	};


inline TDsa::TDsa(const CDsa& aDsa) : iDsa(aDsa)
	{	
	}

inline TBool TDsa::IsTurn() const
	{
	return iDsa.iStateFlags & CDsa::EOrientation90;
	}
	
inline TBool TDsa::IsFlip() const
	{
	return iDsa.iStateFlags & CDsa::EOrientation180;
	}	
	
inline const TSize& TDsa::SwSize() const
	{
	return iDsa.SwSize();
	}
		
inline void TDsa::Copy(TUint32* aTarget, const TUint8* aSrc, TInt aBytes, TInt aHeight) const
	{
	iDsa.iCopyFunction(iDsa, aTarget, aSrc, aBytes, aHeight);
	}
	
template<class T, class S>	
void ClipCopy(const TDsa& iDsa, TUint8* aTarget,
 					const TUint8* aSource,
 					const TRect& aUpdateRect,
 					const TRect& aSourceRect)
	{
	const S* source = reinterpret_cast<const S*>(aSource);
	const TInt lineWidth = aSourceRect.Width();
	
	source += (aUpdateRect.iTl.iY * lineWidth); 
	const TInt sourceStartOffset = aUpdateRect.iTl.iX;
	source += sourceStartOffset;
	
	T* targetPtr = reinterpret_cast<T*>(aTarget);
	
	const TInt scanLineWidth = iDsa.SwSize().iWidth;
	
	targetPtr += (aSourceRect.iTl.iY + aUpdateRect.iTl.iY ) * scanLineWidth; 
	const TInt targetStartOffset = (aUpdateRect.iTl.iX + aSourceRect.iTl.iX);
	
	targetPtr += targetStartOffset;
	
	
	const TInt height = aUpdateRect.Height(); 
		
	const TInt lineMove = iDsa.IsTurn() ? 1 : lineWidth;
	const TInt copyLen = aUpdateRect.Width();
	
	
	if(iDsa.IsFlip())
		{
		
		targetPtr += scanLineWidth *  (height - 1);
	
		for(TInt i = 0; i < height; i++) //source is always smaller
			{
			iDsa.Copy(reinterpret_cast<TUint32*>(targetPtr), reinterpret_cast<const TUint8*>(source), copyLen, height);
			source += lineMove;
			targetPtr -= scanLineWidth;
			}
		}
	else
		{
		
		
		for(TInt i = 0; i < height; i++) //source is always smaller
			{
			iDsa.Copy(reinterpret_cast<TUint32*>(targetPtr), reinterpret_cast<const TUint8*>(source), copyLen, height);
			source += lineMove;
			targetPtr += scanLineWidth; // >> 2;
			}
		}

	}
	

  
NONSHARABLE_CLASS(CDsaA) : public CDsa
	{
	public:
		CDsaA(RWsSession& aSession);
	private:
		~CDsaA();
		TUint8* LockSurface();
		void UnlockHWSurfaceRequestComplete();
		void UnlockHwSurface();
		void CreateSurfaceL();
		void Wipe(TInt aLength);
		void Free();
		void Update(CFbsBitmap& aBmp);
		void ConstructL(RWindow& aWindow, CWsScreenDevice& aDevice);
		TInt ExternalUpdate();
	//	void ExternalUpdate();
	protected:
		CFbsBitmap* iBmp;
		CFbsBitmap* iCopyBmp;
	};
	
	
CDsaA::CDsaA(RWsSession& aSession) : CDsa(aSession)
	{
	}
	
	
void CDsaA::Free()
	{
	delete iBmp;
	iBmp = NULL;
	}

CDsaA::~CDsaA()
	{
	__ASSERT_DEBUG(iBmp == NULL, PANIC(KErrNotReady));
	delete iCopyBmp;
	}
	
TUint8* CDsaA::LockSurface()
	{
	iBmp->LockHeap();
	return reinterpret_cast<TUint8*>(iBmp->DataAddress());
	}

void CDsaA::UnlockHWSurfaceRequestComplete()
	{
	PANIC(KErrNotSupported);
	}

void CDsaA::UnlockHwSurface()
	{
	iBmp->UnlockHeap();
	SetUpdating(EFalse);
	Update(*iBmp);
	}
	
void CDsaA::Update(CFbsBitmap& aBmp)
	{
	if(!Blitter(aBmp))
		{
		if(SwSize() == HwRect().Size())
			Dsa().Gc()->BitBlt(HwRect().iTl, &aBmp);
		else
			Dsa().Gc()->DrawBitmap(HwRect(), &aBmp);
		}
	DrawOverlays();
	Dsa().ScreenDevice()->Update();	
	}
void CDsaA::CreateSurfaceL()
	{
	delete iBmp;
	iBmp = NULL;
	iBmp  = new (ELeave) CFbsBitmap();
	User::LeaveIfError(iBmp->Create(SwSize(), DisplayMode()));
	}

void CDsaA::Wipe(TInt aLength) //dont call in drawing
	{
	iBmp->LockHeap();
	Mem::FillZ(iBmp->DataAddress(), aLength);
	iBmp->UnlockHeap();
	}
	

		
TInt CDsaA::ExternalUpdate()
	{
	if(iCopyBmp->Handle() == 0)
		{
		const TInt err = iCopyBmp->Duplicate(iBmp->Handle());
		if(err != KErrNone)
			return err;
		}
	Update(*iCopyBmp);
	return KErrNone;
	}
	
void CDsaA::ConstructL(RWindow& aWindow, CWsScreenDevice& aDevice)		
	{
	iCopyBmp = new (ELeave) CFbsBitmap();
	CDsa::ConstructL(aWindow, aDevice);
	}
	
	
//////////////////////////////////////////////////////////////////////////////////////////////////////////////////

NONSHARABLE_CLASS(MDsbObs)
	{
	public:
		virtual void SurfaceReady() = 0;
		virtual CDirectScreenBitmap& Dsb() = 0;
	};
	
NONSHARABLE_CLASS(CDsbSurface) : public CActive
	{
	public:
		CDsbSurface(MDsbObs& aDsb);
		TUint8* Address();
		void Complete();
		~CDsbSurface();
	private:
		void RunL();
		void DoCancel();
	private:
		MDsbObs& iDsb; 
		TUint8* iAddress;
	};	

CDsbSurface::CDsbSurface(MDsbObs& aDsb) : CActive(CActive::EPriorityHigh) , iDsb(aDsb)
	{
	CActiveScheduler::Add(this);
	}

CDsbSurface::~CDsbSurface()
	{
	Cancel();
	}
	
void CDsbSurface::Complete()
	{
	if(iAddress != NULL && !IsActive())
		{
		iAddress = NULL;
		SetActive();
		iDsb.Dsb().EndUpdate(iStatus);
		}
	}
	
TUint8* CDsbSurface::Address()
	{
	if(iAddress == NULL && !IsActive())
		{
		TAcceleratedBitmapInfo info;
		if(KErrNone == iDsb.Dsb().BeginUpdate(info))
			iAddress = info.iAddress;
		}
	return iAddress;
	}
	
void CDsbSurface::RunL()
	{
	iDsb.SurfaceReady();
	}

void CDsbSurface::DoCancel()
	{
	//empty
	}
		
NONSHARABLE_CLASS(CDsaB) : public CDsa, public MDsbObs
	{
	public:
		CDsaB(RWsSession& aSession, TInt aFlags);
	private:
		~CDsaB();
		TUint8* LockSurface();
		void UnlockHWSurfaceRequestComplete();
		void UnlockHwSurface();
		void CreateSurfaceL();
		void Wipe(TInt aLength);
		void RecreateL();
		void ConstructL(RWindow& aWindow, CWsScreenDevice& aDevice);
		CDirectScreenBitmap& Dsb();
		void SurfaceReady();
		TInt ExternalUpdate();
	private:
		CDsbSurface* iSurface1;
		CDsbSurface* iSurface2;
		CDirectScreenBitmap* iDsb;
		TInt iType;
	};

CDsaB::CDsaB(RWsSession& aSession, TInt aFlags) : CDsa(aSession), iType(aFlags)
	{
	}


	
void CDsaB::UnlockHWSurfaceRequestComplete()
	{
	iSurface1->Complete();
	if(iSurface2 != NULL)
		iSurface2->Complete();
	}	

void CDsaB::CreateSurfaceL()
	{
	__ASSERT_ALWAYS(SwSize() == HwRect().Size(), PANIC(KErrNotSupported));
	}
	
void CDsaB::Wipe(TInt aLength) //dont call in drawing
	{
	TUint8* addr = LockSurface();
	if(addr != NULL) 
		{
		Mem::FillZ(addr, aLength);
		UnlockHwSurface();
		}
	}
	

void CDsaB::UnlockHwSurface()
	{
	EpocSdlEnv::Request(CDsa::ERequestUpdate);
	}
	
TUint8* CDsaB::LockSurface()
	{
	TUint8* addr =  iSurface1->Address();
	if(addr == NULL && iSurface2 != NULL)
		addr =  iSurface2->Address();
	SetUpdating(addr == NULL);
	return addr;
	}
	
void CDsaB::SurfaceReady()	
	{
	SetUpdating(EFalse);
	}

CDirectScreenBitmap& CDsaB::Dsb()
	{
	return *iDsb;
	}
	
void CDsaB::ConstructL(RWindow& aWindow, CWsScreenDevice& aDevice)
	{
	if(iDsb == NULL)
		iDsb = CDirectScreenBitmap::NewL();	
	CDsa::ConstructL(aWindow, aDevice);
	if(iSurface1 == NULL)	
		iSurface1 = new (ELeave) CDsbSurface(*this);
	if(iSurface2 == NULL && iType & CDirectScreenBitmap::EDoubleBuffer)
		iSurface2 = new (ELeave) CDsbSurface(*this);
	}
	
CDsaB::~CDsaB()
	{
	delete iSurface1;
	delete iSurface2;
	delete iDsb;
	}	

void CDsaB::RecreateL()
	{
    iDsb->Close();
    iDsb->Create(HwRect(), CDirectScreenBitmap::TSettingsFlags(iType));
	}
	
TInt CDsaB::ExternalUpdate()
	{
	if(LockSurface())
		{
		UnlockHWSurfaceRequestComplete();
		return KErrNone;
		}
	return KErrNotReady;
	}
		

/////////////////////////////////////////////////////////////////////////////////////////////////////	


CDsa* CDsa::CreateL(RWsSession& aSession)
	{
	if(EpocSdlEnv::Flags(CSDL::EDrawModeDSB))
		{
		TInt flags = CDirectScreenBitmap::ENone;
		if(EpocSdlEnv::Flags(CSDL::EDrawModeDSBDoubleBuffer))
			flags |= CDirectScreenBitmap::EDoubleBuffer;
		if(EpocSdlEnv::Flags(CSDL::EDrawModeDSBIncrentalUpdate))
			flags |= CDirectScreenBitmap::EIncrementalUpdate;
		return new (ELeave) CDsaB(aSession, flags);
		}
    else
        return new (ELeave) CDsaA(aSession);
	}   
	
	
void CDsa::RecreateL()
	{
	}

void CDsa::Free()
	{
	}
	
TSize CDsa::WindowSize() const
	{
	TSize size = iSwSize;
	if(iStateFlags & EOrientation90)
		{
		const TInt tmp = size.iWidth;
		size.iWidth = size.iHeight;
		size.iHeight = tmp;
		}
	return size;
	}
	
void CDsa::SetSuspend()
	{
	iStateFlags |= ESdlThreadSuspend;
	}

void CDsa::ReleaseStop()
	{
	iStateFlags &= ~ESdlThreadExplicitStop;
	}


TBool CDsa::Stopped() const
	{
	return (iStateFlags & ESdlThreadExplicitStop);
	}

void CDsa::SetOrientation(CSDL::TOrientationMode aOrientation)
	{
	TInt flags = 0;
	switch(aOrientation)
		{
		case CSDL::EOrientation90:
			flags = EOrientation90;
			break;
		case CSDL::EOrientation180:
			flags = EOrientation180;
			break;
		case CSDL::EOrientation270:
			flags = EOrientation90 | EOrientation180;
			break;
		case CSDL::EOrientation0:
			flags = 0;
			break;
		}
	if(flags != (iStateFlags & EOrientationFlags))
		{
		iStateFlags |= EOrientationChanged;
		iNewFlags = flags; //cannot be set during drawing...
		}
	}

CDsa::~CDsa()
    {
    if(iDsa != NULL)
        {
        iDsa->Cancel();
        }
    iOverlays.Close();
    delete iDsa;
    User::Free(iLut256);
    }
         
void CDsa::ConstructL(RWindow& aWindow, CWsScreenDevice& aDevice)
    {
    if(iDsa != NULL)
    	{
    	iDsa->Cancel();
    	delete iDsa;
    	iDsa = NULL;
    	}
    	
    iDsa = CDirectScreenAccess::NewL(
    				iSession,
					aDevice,
					aWindow,
					*this);				

	if(iLut256 == NULL)
		iLut256 = (TUint32*) User::AllocL(256 * sizeof(TUint32));
	iTargetMode = aWindow.DisplayMode();
	iTargetBpp = BytesPerPixel(DisplayMode());
	iScreenRect = TRect(aWindow.Position(), aWindow.Size());
	SetTargetRect();
    RestartL();
    }
    
void CDsa::DrawOverlays()
	{
	const TInt last = iOverlays.Count() - 1;
	for(TInt i = last; i >= 0 ; i--)
		iOverlays[i].iOverlay->Draw(*iDsa->Gc(), HwRect(), SwSize());
	}

TInt CDsa::AppendOverlay(MOverlay& aOverlay, TInt aPriority)
	{
	TInt i;
	for(i = 0; i < iOverlays.Count() && iOverlays[i].iPriority < aPriority; i++)
		{}
	const TOverlay overlay = {&aOverlay, aPriority};
	return iOverlays.Insert(overlay, i);
	}
	
TInt CDsa::RemoveOverlay(MOverlay& aOverlay)
	{
	for(TInt i = 0; i < iOverlays.Count(); i++)
		{
		if(iOverlays[i].iOverlay == &aOverlay)
			{
			iOverlays.Remove(i);
			return KErrNone;
			}
		}
	return KErrNotFound;
	}

void CDsa::LockPalette(TBool aLock)
	{
	if(aLock)
		iStateFlags |= EPaletteLocked;
	else
		iStateFlags &= ~EPaletteLocked;
	}
TInt CDsa::SetPalette(TInt aFirst, TInt aCount, TUint32* aPalette)
	{
	if(iLut256 == NULL)
		return KErrNotFound;
	const TInt count = aCount - aFirst;
	if(count > 256)
		return KErrArgument;
	if(iStateFlags & EPaletteLocked)
		return KErrNone;
	for(TInt i = aFirst; i < count; i++) //not so busy here:-)
		{
		iLut256[i] = aPalette[i];
		}
	return KErrNone;
	}
	
	

void CDsa::RestartL()
    {
    //const TBool active = iDsa->IsActive();
    
    //if(!active)
    
    iDsa->StartL();	
    
    const RRegion* r = iDsa->DrawingRegion();
    const TRect rect = r->BoundingRect();
    iDsa->Gc()->SetClippingRegion(r);	
   
    if(rect != iScreenRect)
    	{
    //	iDsa->Cancel();
    	return ;	
   	 	}
   	 	
     
    	
    //iScreenRect = rect; //to ensure properly set, albeit may not(?) match to value SDL has - therefore may has to clip
	//targetrect shall no change
	SetTargetRect();
	RecreateL();

    iStateFlags |= ERunning;
    
    ReleaseStop();
    if(iStateFlags & ESdlThreadSuspend)
    	{
    	EpocSdlEnv::Resume();
    	iStateFlags &= ~ ESdlThreadSuspend;
    	}	
    EpocSdlEnv::ObserverEvent(MSDLObserver::EEventWindowReserved);	
    }
    
CDsa::CDsa(RWsSession& aSession) : 
 	iSession(aSession),
  	iStateFlags(0)
	{
//	CActiveScheduler::Add(this);
	iCFTable[0] = CopyMem;
	iCFTable[1] = CopyMemFlipReversed;
	iCFTable[2] = CopyMemReversed;
	iCFTable[3] = CopyMemFlip;	
	
	iCFTable[4] = Copy256;
	iCFTable[5] = Copy256FlipReversed;
	iCFTable[6] = Copy256Reversed;
	iCFTable[7] = Copy256Flip;	
	
	
	iCFTable[8] = CopySlow;
	iCFTable[9] = CopySlowFlipReversed;
	iCFTable[10] = CopySlowReversed;
	iCFTable[11] = CopySlowFlip;	
	}
	
RWsSession& CDsa::Session()
	{
	return iSession;
	}

TInt CDsa::RedrawRequest()
	{
	if(!(iStateFlags & (EUpdating) && (iStateFlags & ERunning)))
		{
		return ExternalUpdate();
		}
	return KErrNotReady;
	}

TUint8* CDsa::LockHwSurface()
	{
	if((iStateFlags & EUpdating) == 0) //else frame is skipped
		{
		return LockSurface();
		}
	return NULL; 
	}

/*	
void CDsa::RunL()
	{
	iStateFlags &= ~EUpdating;
	}
		
	
void CDsa::DoCancel()
	{
	iStateFlags &= ~EUpdating;
	//nothing can do, just wait?
	}
*/	

	
TInt CDsa::AllocSurface(TBool aHwSurface, const TSize& aSize, TDisplayMode aMode)
	{
	if(aHwSurface && aMode != DisplayMode())
		return KErrArgument;
	
	iSourceMode = aMode;
	
	iSourceBpp = BytesPerPixel(aMode);
	
	const TSize size = WindowSize();
	if(aSize.iWidth > size.iWidth)
		return KErrTooBig;
	if(aSize.iHeight > size.iHeight)
		return KErrTooBig;
	
	TRAPD(err, CreateSurfaceL());
	if(err != KErrNone)
		return err;

	SetCopyFunction();
	
	return KErrNone;
	}
	

void CDsa::CreateZoomerL(const TSize& aSize)
	{
	iSwSize = aSize;
	iStateFlags |= EResizeRequest;
	CreateSurfaceL();
	SetTargetRect();
	}
	

/*
void SaveBmp(const TDesC& aName, const TAny* aData, TInt aLength, const TSize& aSz, TDisplayMode aMode)
	{
	CFbsBitmap* s = new CFbsBitmap();
	s->Create(aSz, aMode);
	s->LockHeap();
	TUint32* addr = s->DataAddress();
	Mem::Copy(addr, aData, aLength);
	s->UnlockHeap();
	s->Save(aName);
	s->Reset();
	delete s;
	}
	
void SaveBmp(const TDesC& aName, const TUint32* aData, const TSize& aSz)
	{
	CFbsBitmap* s = new CFbsBitmap();
	s->Create(aSz, EColor64K);
	TBitmapUtil bmp(s);
	bmp.Begin(TPoint(0, 0));
	for(TInt j = 0; j < aSz.iHeight; j++)
		{
		bmp.SetPos(TPoint(0, j));
		for(TInt i = 0; i < aSz.iWidth; i++)
			{
			bmp.SetPixel(*aData);
			aData++;
			bmp.IncXPos();
			}
		}
	bmp.End();
	s->Save(aName);
	s->Reset();
	delete s;
	}	
	
TBuf<16> FooName(TInt aFoo)
	{
	TBuf<16> b;
	b.Format(_L("C:\\pic%d.mbm"), aFoo);
	return b;
	}
	
void ClipCopy(TUint8* aTarget, const TUint8* aSource, const TRect& aRect, const TPoint& aTargetPos)
	{
	const TInt iSourceBpp = 1;
	const TInt iTargetBpp = 4;
	const TInt iScanLineWidth = 800; 
	
	TUint8* target = aTarget;
	const TUint8* source = aSource;
	const TInt lineWidth = aRect.Width();
	source += iSourceBpp * (aRect.iTl.iY * lineWidth); 
	const TInt sourceStartOffset = iSourceBpp *  aRect.iTl.iX;
	source += sourceStartOffset;
	target += iTargetBpp * ((aTargetPos.iY + aRect.iTl.iY ) * lineWidth); 
	const TInt targetStartOffset = iTargetBpp * (aRect.iTl.iX + aTargetPos.iX);
	target += targetStartOffset;
	TUint32* targetPtr = reinterpret_cast<TUint32*>(target);
	const TInt targetWidth = iScanLineWidth >> 2;
	const TInt height = aRect.Height(); 
	}	
*/
/*
void CDsa::ClipCopy(TUint8* aTarget,
 					const TUint8* aSource,
 					const TRect& aUpdateRect,
 					const TRect& aSourceRect) const
	{
	//TUint8* target = aTarget;
	const TUint32* source = (const TUint32*) aSource;
	const TInt lineWidth = aSourceRect.Width();
	
	source +=  (aUpdateRect.iTl.iY * lineWidth); 
	const TInt sourceStartOffset =   aUpdateRect.iTl.iX;
	source += sourceStartOffset;
	
	TUint32* targetPtr = reinterpret_cast<TUint32*>(aTarget);
	
	targetPtr +=  (aSourceRect.iTl.iY + aUpdateRect.iTl.iY ) * SwSize().iWidth; 
	const TInt targetStartOffset =  (aUpdateRect.iTl.iX + aSourceRect.iTl.iX);
	
	targetPtr += targetStartOffset;
	
//	TUint32* targetPtr = reinterpret_cast<TUint32*>(target);
	
	const TInt targetWidth32 = SwSize().iWidth;
	
	const TInt height = aUpdateRect.Height(); 
		
	const TInt lineMove = iStateFlags & EOrientation90 ? 1 : lineWidth;
	const TInt copyLen = aUpdateRect.Width();
	
	
	if(iStateFlags & EOrientation180)
		{
		
		targetPtr += targetWidth32 *  (height - 1);
	
		for(TInt i = 0; i < height; i++) //source is always smaller
			{
			iCopyFunction(*this, targetPtr, (TUint8*)source, copyLen, height);
			source += lineMove;
			targetPtr -= targetWidth32;
			}
		}
	else
		{
		
		
		for(TInt i = 0; i < height; i++) //source is always smaller
			{
			iCopyFunction(*this, targetPtr, (TUint8*)source, copyLen, height);
			source += lineMove;
			targetPtr += targetWidth32; // >> 2;
			}
		}

	}
	
*/

void CDsa::ClipCopy(TUint8* aTarget, const TUint8* aSource, const TRect& aRect, const TPoint& aTargetPos) const
	{
	TUint8* target = aTarget;
	const TUint8* source = aSource;
	const TInt lineWidth = aRect.Width();
	source += iSourceBpp * (aRect.iTl.iY * lineWidth); 
	TInt sourceStartOffset = iSourceBpp *  aRect.iTl.iX;
	source += sourceStartOffset;
	target += iTargetBpp * ((aTargetPos.iY + aRect.iTl.iY ) * lineWidth); 
	TInt targetStartOffset = iTargetBpp * (aRect.iTl.iX + aTargetPos.iX);
	target += targetStartOffset;
	TUint32* targetPtr = reinterpret_cast<TUint32*>(target);
	const TInt targetWidth = iScanLineWidth >> 2;
	const TInt height = aRect.Height(); 
	
	TInt lineMove = iStateFlags & EOrientation90 ? 1 : lineWidth;
	
	if(iStateFlags & EOrientation180)
		{
		
		targetPtr += targetWidth *  (height - 1);
	
		for(TInt i = 0; i < height; i++) //source is always smaller
			{
			iCopyFunction(*this, targetPtr, source, lineWidth, height);
			source += lineMove;
			targetPtr -= targetWidth;
			}
		}
	else
		{
		
		
		for(TInt i = 0; i < height; i++) //source is always smaller
			{
			iCopyFunction(*this, targetPtr, source, lineWidth, height);
			source += lineMove;
			targetPtr += targetWidth;
			}
		}

	}
	
		
	
	/*
void CDsa::ClipCopy(TUint8* aTarget,
 					const TUint8* aSource,
 					const TRect& aUpdateRect,
 					const TRect& aSourceRect) const
 		{
 		const TDsa dsa(*this);
 		switch(iSourceBpp)
 			{
 			case 1:
 				::ClipCopy<TUint32, TUint8>(dsa, aTarget, aSource, aUpdateRect, aSourceRect);
 				break;
 			case 2:
 				::ClipCopy<TUint32, TUint16>(dsa, aTarget, aSource, aUpdateRect, aSourceRect);
 				break;
 			case 4:
 				::ClipCopy<TUint32, TUint32>(dsa, aTarget, aSource, aUpdateRect, aSourceRect);
 				break;
 			}
 		}	


*/



void CDsa::Wipe() //dont call in drawing
	{
	if(IsDsaAvailable())
		Wipe(iTargetBpp * SwSize().iWidth * SwSize().iHeight);
	}
	
void CDsa::SetCopyFunction()
	{
	//calculate offset to correct function in iCFTable according to given parameters
	TInt function = 0;
	const TInt KCopyFunctions = 4;
	const TInt KOffsetToNative = 0;
	const TInt KOffsetTo256 = KOffsetToNative + KCopyFunctions;
	const TInt KOffsetToOtherModes = KOffsetTo256 + KCopyFunctions;
	const TInt KOffsetTo90Functions = 1;
	const TInt KOffsetTo180Functions = 2;
	
	if(iSourceMode == DisplayMode())
		function = KOffsetToNative; 		//0
	else if(iSourceMode == EColor256)
		function = KOffsetTo256;			//4
	else
		function = KOffsetToOtherModes; 	//8
	
	if(iStateFlags & EOrientation90)
		function += KOffsetTo90Functions; 	// + 1
	if(iStateFlags & EOrientation180)
		function += KOffsetTo180Functions; 	//+ 2
	
	iCopyFunction = iCFTable[function];
	
	Wipe();
	}
	
inline void Rotate(TRect& aRect)
	{
	const TInt dx = aRect.iBr.iX - aRect.iTl.iX;
	const TInt dy = aRect.iBr.iY - aRect.iTl.iY;

	aRect.iBr.iX = aRect.iTl.iX + dy;
	aRect.iBr.iY = aRect.iTl.iY + dx;
	
	const TInt tmp = aRect.iTl.iX;
	aRect.iTl.iX = aRect.iTl.iY;
	aRect.iTl.iY = tmp;
	}
	
/*	
int bar = 0;
*/	
/*
TBool CDsa::AddUpdateRect(const TUint8* aBits, const TRect& aUpdateRect, const TRect& aRect)
	{

	if(iStateFlags & EOrientationChanged)
		{
		iStateFlags &= ~EOrientationFlags;
		iStateFlags |= iNewFlags;
		SetCopyFunction();
		iStateFlags &= ~EOrientationChanged;
	    EpocSdlEnv::WaitDeviceChange();
	    return EFalse; //skip this frame as data is may be changed
		}

	if(iTargetAddr == NULL)
		{
		iTargetAddr = LockHwSurface();
		}
		
	TUint8* target = iTargetAddr;
	if(target == NULL)
		return EFalse;
	
	
	TRect targetRect = TRect(TPoint(0, 0), SwSize());
	
	TRect sourceRect = aRect;
	TRect updateRect = aUpdateRect;
	
//	TPoint move(0, 0);
	
	
	if(iStateFlags & EOrientation90)
		{
		Rotate(sourceRect);
		Rotate(updateRect);
		}
		
	if(iSourceMode != DisplayMode() ||  targetRect != sourceRect || targetRect != updateRect || ((iStateFlags & EOrientationFlags) != 0))
		{
		sourceRect.Intersection(targetRect); //so source always smaller or equal than target
		//updateRect.Intersection(targetRect);
		ClipCopy(target, aBits, updateRect, sourceRect);
		}
	else
		{
		const TInt byteCount = aRect.Width() * aRect.Height() * iSourceBpp; //this could be stored
		Mem::Copy(target, aBits, byteCount);
		}

	return ETrue;
	}
	*/
	
TBool CDsa::AddUpdateRect(const TUint8* aBits, const TRect& aUpdateRect, const TRect& aRect)
	{

	if(iStateFlags & EOrientationChanged)
		{
		iStateFlags &= ~EOrientationFlags;
		iStateFlags |= iNewFlags;
		SetCopyFunction();
		iStateFlags &= ~EOrientationChanged;
	    EpocSdlEnv::WaitDeviceChange();
	    return EFalse; //skip this frame as data is may be changed
		}

	if(iTargetAddr == NULL)
		{
		iTargetAddr = LockHwSurface();
		}
	TUint8* target = iTargetAddr;
	if(target == NULL)
		return EFalse;
	
	
	TRect targetRect = Rect();
	TRect sourceRect = aRect;
	TRect updateRect = aUpdateRect;
	
	if(iStateFlags & EOrientation90)
		{
		Rotate(sourceRect);
		Rotate(updateRect);
		}
		
	if(iSourceMode != DisplayMode() ||  targetRect != sourceRect || targetRect != updateRect || ((iStateFlags & EOrientationFlags) != 0))
		{
		sourceRect.Intersection(targetRect); //so source always smaller or equal than target
		updateRect.Intersection(targetRect);
		ClipCopy(target, aBits, updateRect, sourceRect.iTl);
		}
	else
		{
		const TInt byteCount = aRect.Width() * aRect.Height() * iSourceBpp; //this could be stored
		Mem::Copy(target, aBits, byteCount);
		}

	return ETrue;
	}	
void CDsa::UpdateSwSurface()
	{
	iTargetAddr = NULL;
	UnlockHwSurface();	//could be faster if does not use AO, but only check status before redraw, then no context switch needed
	}
	

void CDsa::Resume()	
	{
	if(Stopped())
		Restart(RDirectScreenAccess::ETerminateRegion);
	}
	
void CDsa::DoStop()
	{
	if(IsDsaAvailable())
		iStateFlags |= ESdlThreadExplicitStop;
	Stop();
	}
	
void CDsa::Stop()
	{
	iStateFlags &= ~ERunning;
//	Cancel(); //can be called only from main!
	iDsa->Cancel();
	}
	
void CDsa::AbortNow(RDirectScreenAccess::TTerminationReasons /*aReason*/)
	{
//	iStateFlags |= EChangeNotify;
	Stop();
	}
	
void CDsa::Restart(RDirectScreenAccess::TTerminationReasons aReason)
	{
	if(aReason == RDirectScreenAccess::ETerminateRegion) //auto restart
		{												
		TRAPD(err, RestartL());
		PANIC_IF_ERROR(err);
		}
	}
	
void CDsa::SetBlitter(MBlitter* aBlitter)
	{
	iBlitter = aBlitter;
	}
	
	
TPoint CDsa::WindowCoordinates(const TPoint& aPoint) const	
	{
	TPoint pos = aPoint - iScreenRect.iTl;
	const TSize asz = iScreenRect.Size();
	if(iStateFlags & EOrientation180)
		{
		pos.iX = asz.iWidth - pos.iX;
		pos.iY = asz.iHeight - pos.iY;	
		}	
	if(iStateFlags & EOrientation90)
		{
		pos.iX = aPoint.iY;
		pos.iY = aPoint.iX;	
		}
	pos.iX <<= 16;
	pos.iY <<= 16;
	pos.iX /= asz.iWidth; 
	pos.iY /= asz.iHeight;
	pos.iX *= iSwSize.iWidth;
	pos.iY *= iSwSize.iHeight;
	pos.iX >>= 16;
	pos.iY >>= 16;
	return pos; 	
	}
	
void CDsa::SetTargetRect()
	{
	iTargetRect = iScreenRect;
	if(iStateFlags & EResizeRequest && EpocSdlEnv::Flags(CSDL::EAllowImageResizeKeepRatio))
		{
		const TSize asz = iScreenRect.Size();
		const TSize sz = iSwSize;
		
		TRect rect;
		
		const TInt dh = (sz.iHeight << 16) / sz.iWidth;

		if((asz.iWidth * dh ) >> 16 <= asz.iHeight)
			{
			rect.SetRect(TPoint(0, 0), TSize(asz.iWidth, (asz.iWidth * dh) >> 16));
			}
		else
			{
			const TInt dw = (sz.iWidth << 16) / sz.iHeight;
	    	rect.SetRect(TPoint(0, 0), TSize((asz.iHeight * dw) >> 16, asz.iHeight));
			}
		rect.Move((asz.iWidth - rect.Size().iWidth) >> 1, (asz.iHeight - rect.Size().iHeight) >> 1);  
		
		iTargetRect = rect;
		iTargetRect.Move(iScreenRect.iTl);

		} 
	if(!(iStateFlags & EResizeRequest))
		iSwSize = iScreenRect.Size();
//	iScanLineWidth = /*iTargetBpp **/ SwSize().iWidth;
	}
		
/*)
TBool CDsa::ChangeTrigger()
	{
	const TBool change = iStateFlags & EChangeNotify;
	iStateFlags &= ~EChangeNotify;
	return change;
	}
*/	
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////

void CDsa::Copy256(const CDsa& aDsa, TUint32* aTarget, const TUint8* aSource, TInt aBytes, TInt)
	{
	TUint32* target = aTarget;
	const TUint32* endt = target + aBytes; 
	const TUint8* source = aSource;
	while(target < endt)
		{
		*target++ = aDsa.iLut256[*source++]; 
		}
	}
	
void CDsa::Copy256Reversed(const CDsa& aDsa, TUint32* aTarget, const TUint8* aSource, TInt aBytes, TInt)
	{
	const TUint32* target = aTarget;
	TUint32* endt = aTarget + aBytes; 
	const TUint8* source = aSource;
	while(target < endt)
		{
		*(--endt) = aDsa.iLut256[*source++]; 
		}
	}	
	
void CDsa::Copy256Flip(const CDsa& aDsa, TUint32* aTarget, const TUint8* aSource, TInt aBytes, TInt aLineLen)
	{
	TUint32* target = aTarget;
	const TUint32* endt = target + aBytes; 
	const TUint8* column = aSource;

	while(target < endt)
		{
		*target++ = aDsa.iLut256[*column];
		column += aLineLen;
		}
	}
	
void CDsa::Copy256FlipReversed(const CDsa& aDsa, TUint32* aTarget, const TUint8* aSource, TInt aBytes, TInt aLineLen)
	{
	const TUint32* target = aTarget;
	TUint32* endt = aTarget + aBytes; 
	const TUint8* column = aSource;

	while(target < endt)
		{
		*(--endt) = aDsa.iLut256[*column];
		column += aLineLen;
		}
	}		

void CDsa::CopyMem(const CDsa& /*aDsa*/, TUint32* aTarget, const TUint8* aSource, TInt aBytes, TInt)
	{
	const TUint32* src = reinterpret_cast<const TUint32*>(aSource);
	Mem::Copy(aTarget, src, aBytes << 2);
	}
	
void CDsa::CopyMemFlip(const CDsa& /*aDsa*/, TUint32* aTarget, const TUint8* aSource, TInt aBytes, TInt aLineLen)
	{
	TUint32* target = aTarget;
	const TUint32* endt = target + aBytes; 
	const TUint32* column = reinterpret_cast<const TUint32*>(aSource);

	while(target < endt)
		{
		*target++ = *column;
		column += aLineLen;
		}
	}
	
void CDsa::CopyMemReversed(const CDsa& /*aDsa*/, TUint32* aTarget, const TUint8* aSource, TInt aBytes, TInt)
	{
	const TUint32* target = aTarget;
	TUint32* endt = aTarget + aBytes; 
	const TUint32* source = reinterpret_cast<const TUint32*>(aSource);
	while(target < endt)
		{
		*(--endt) = *source++; 
		}
	}	
	
	
void CDsa::CopyMemFlipReversed(const CDsa& /*aDsa*/, TUint32* aTarget, const TUint8* aSource, TInt aBytes, TInt aLineLen)
	{
	const TUint32* target = aTarget;
	TUint32* endt = aTarget + aBytes; 
	const TUint32* column = reinterpret_cast<const TUint32*>(aSource);

	while(target < endt)
		{
		*(--endt) = *column;
		column += aLineLen;
		}
	}
			
/*

LOCAL_C TRgb rgb16MA(TInt aValue)
	{
	return TRgb::Color16MA(aValue);
	}
*/	
NONSHARABLE_CLASS(MRgbCopy)
	{
	public:
	virtual void Copy(TUint32* aTarget, const TUint8* aSource, TInt aBytes, TBool aReversed) = 0;
	virtual void FlipCopy(TUint32* aTarget, const TUint8* aSource, TInt aBytes, TInt aLineLen, TBool aReversed) = 0;
	};
	
template <class T>
NONSHARABLE_CLASS(TRgbCopy) : public MRgbCopy
	{
	public:
	TRgbCopy(TDisplayMode aMode);
	void* operator new(TUint aBytes, TAny* aMem);
	void Copy(TUint32* aTarget, const TUint8* aSource, TInt aBytes, TBool aReversed);
	void FlipCopy(TUint32* aTarget, const TUint8* aSource, TInt aBytes, TInt aLineLen, TBool aReversed);
	static TUint32 Gray256(const TUint8& aPixel);
	static TUint32 Color256(const TUint8& aPixel);
	static TUint32 Color4K(const TUint16& aPixel);
	static TUint32 Color64K(const TUint16& aPixel);
	static TUint32 Color16M(const TUint32& aPixel);
	static TUint32 Color16MU(const TUint32& aPixel);
	static TUint32 Color16MA(const TUint32& aPixel);
	private:
		typedef TUint32 (*TRgbFunc) (const T& aValue);
		TRgbFunc iFunc;
	};
		
		
template <class T>		
void* TRgbCopy<T>::operator new(TUint /*aBytes*/, TAny* aMem)
	{
	return aMem;
	}
		
template <class T>
TRgbCopy<T>::TRgbCopy(TDisplayMode aMode)
	{
	switch(aMode)
		{
		case EGray256 : iFunc = (TRgbFunc) Gray256; break;
		case EColor256 : iFunc =  (TRgbFunc) Color256; break;
		case EColor4K : iFunc =  (TRgbFunc) Color4K; break;
		case EColor64K : iFunc =  (TRgbFunc) Color64K; break;
		case EColor16M : iFunc =  (TRgbFunc) Color16M; break;
		case EColor16MU : iFunc =  (TRgbFunc) Color16MU; break;
		case EColor16MA : iFunc =  (TRgbFunc) Color16MA; break;
		default:
			PANIC(KErrNotSupported);
		}
	}
	
template <class T>
void TRgbCopy<T>::Copy(TUint32* aTarget, const TUint8* aSource, TInt aBytes, TBool aReversed)
	{
	const T* source = reinterpret_cast<const T*>(aSource);
	TUint32* target = aTarget;
	TUint32* endt = target + aBytes;
	
	if(aReversed)
		{
		while(target < endt)
			{
			const T value = *source++;
			*(--endt) = iFunc(value);//iFunc(value).Value();
			}
		}
	else
		{
		while(target < endt)
			{
			const T value = *source++;
			*target++ = iFunc(value);//iFunc(value).Value();
			}
		}
	}
	
template <class T>
void TRgbCopy<T>::FlipCopy(TUint32* aTarget, const TUint8* aSource, TInt aBytes, TInt aLineLen, TBool aReversed)
	{
	const T* column = reinterpret_cast<const T*>(aSource);
	TUint32* target = aTarget;
	TUint32* endt = target + aBytes;
	
	if(aReversed)
		{
		while(target < endt)
			{
			*(--endt) = iFunc(*column);
			column += aLineLen;
			}
		}
	else
		{
		while(target < endt)
			{
			*target++ = iFunc(*column);
			column += aLineLen;
			}
		}
	}	
		
template <class T> TUint32 TRgbCopy<T>::Gray256(const TUint8& aPixel)
	{
	const TUint32 px = aPixel << 16 | aPixel << 8 | aPixel;
	return px;
	}
	
template <class T> TUint32 TRgbCopy<T>::Color256(const TUint8& aPixel)
	{
	return TRgb::Color256(aPixel).Value();
	}
	
template <class T> TUint32 TRgbCopy<T>::Color4K(const TUint16& aPixel)
	{
	TUint32 col = (aPixel & 0xF00) << 12;
	col |= (aPixel & 0xF00) << 8; 
	
	col |= (aPixel & 0x0F0) << 8;
	col |= (aPixel & 0x0F0);
	
	col |= (aPixel & 0x00F) << 4;
	col |= (aPixel & 0x00F);
	
	return col;
	}
	
template <class T> TUint32 TRgbCopy<T>::Color64K(const TUint16& aPixel)
	{
	TUint32 col = (aPixel & 0xF800)<< 8;
	col |= (aPixel & 0xE000) << 3; 
	
	col |= (aPixel & 0x07E0) << 5;
	col |= (aPixel & 0xC0) >> 1;
	
	col |= (aPixel & 0x07E0) << 3;
	col |= (aPixel & 0x1C) >> 2;
	
	return col;
	}
	
template <class T> TUint32 TRgbCopy<T>::Color16M(const TUint32& aPixel)
	{
	return TRgb::Color16M(aPixel).Value();
	}
	
template <class T> TUint32 TRgbCopy<T>::Color16MU(const TUint32& aPixel)
	{
	return TRgb::Color16MU(aPixel).Value();
	}
	
template <class T> TUint32 TRgbCopy<T>::Color16MA(const TUint32& aPixel)
	{
	return TRgb::Color16MA(aPixel).Value();
	}

typedef TUint64 TStackMem;

LOCAL_C MRgbCopy* GetCopy(TAny* mem, TDisplayMode aMode)
	{
	if(aMode == EColor256 || aMode == EGray256)
		{
		return new (mem) TRgbCopy<TUint8>(aMode);
		}
	if(aMode == EColor4K || aMode == EColor64K)
		{
		return new (mem) TRgbCopy<TUint16>(aMode);
		}
	if(aMode == EColor16M || aMode == EColor16MU || aMode == EColor16MA)
		{
		return new (mem) TRgbCopy<TUint32>(aMode);
		}
	PANIC(KErrNotSupported);
	return NULL;
	}
	

void CDsa::CopySlowFlipReversed(const CDsa& aDsa, TUint32* aTarget, const TUint8* aSource, TInt aBytes, TInt aLineLen)
	{
	TStackMem mem = 0;
	GetCopy(&mem, aDsa.iSourceMode)->FlipCopy(aTarget, aSource, aBytes, aLineLen, ETrue);	
	}
	
void CDsa::CopySlowFlip(const CDsa& aDsa, TUint32* aTarget, const TUint8* aSource, TInt aBytes, TInt aLineLen)
	{
	TStackMem mem = 0;
	GetCopy(&mem, aDsa.iSourceMode)->FlipCopy(aTarget, aSource, aBytes, aLineLen, EFalse);
	}
	
void CDsa::CopySlow(const CDsa& aDsa, TUint32* aTarget, const TUint8* aSource, TInt aBytes, TInt)
	{
	TStackMem mem = 0;
	GetCopy(&mem, aDsa.iSourceMode)->Copy(aTarget, aSource, aBytes, EFalse);	
	}	

void CDsa::CopySlowReversed(const CDsa& aDsa, TUint32* aTarget, const TUint8* aSource, TInt aBytes, TInt)
	{
	TStackMem mem = 0;
	GetCopy(&mem, aDsa.iSourceMode)->Copy(aTarget, aSource, aBytes, ETrue);	
	}	

////////////////////////////////////////////////////////////////////////////////////////////////////////////////7
