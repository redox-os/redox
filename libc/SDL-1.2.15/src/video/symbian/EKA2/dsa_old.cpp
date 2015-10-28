#include "dsa.h"
#include "sdlepocapi.h"
#include <cdsb.h>

LOCAL_C TInt BytesPerPixel(TDisplayMode aMode)
	{
	return ((TDisplayModeUtils::NumDisplayModeBitsPerPixel(aMode) - 1) >> 3) + 1; 
	}


////////////////////////////////////////////////////////////////////////////////////////////////
  
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
		void RecreateL();
		void Free();
		TInt ExternalUpdate() {return 0;}
	private:
		CFbsBitmap* iBmp;	
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
	Dsa().Gc()->BitBlt(HwRect().iTl, iBmp);
	Dsa().ScreenDevice()->Update();
	}

void CDsaA::CreateSurfaceL()
	{
	delete iBmp;
	iBmp = NULL;
	iBmp  = new (ELeave) CFbsBitmap();
	User::LeaveIfError(iBmp->Create(HwRect().Size(), DisplayMode()));
	}

void CDsaA::Wipe(TInt aLength) //dont call in drawing
	{
	iBmp->LockHeap();
	Mem::FillZ(iBmp->DataAddress(), aLength);
	iBmp->UnlockHeap();
	}
	
void CDsaA::RecreateL()
	{
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
		CDsaB(RWsSession& aSession);
	private:
		~CDsaB();
		TUint8* LockSurface();
		void UnlockHWSurfaceRequestComplete();
		void UnlockHwSurface();
		void CreateSurfaceL();
		void Wipe(TInt aLength);
		void RecreateL();
		void ConstructL(RWindow& aWindow, CWsScreenDevice& aDevice);
		void Free();
		CDirectScreenBitmap& Dsb();
		void SurfaceReady();
		TInt ExternalUpdate() {return 0;}
	private:
		CDsbSurface* iSurface1;
		CDsbSurface* iSurface2;
		CDirectScreenBitmap* iDsb;
	};

CDsaB::CDsaB(RWsSession& aSession) : CDsa(aSession)
	{
	}

void CDsaB::Free()
	{
	}
	
void CDsaB::UnlockHWSurfaceRequestComplete()
	{
	iSurface1->Complete();
	iSurface2->Complete();
	}	

void CDsaB::CreateSurfaceL()
	{
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
	if(addr == NULL)
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
	iSurface1 = new (ELeave) CDsbSurface(*this);
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
    iDsb->Create(HwRect(), CDirectScreenBitmap::EDoubleBuffer);
	}

/////////////////////////////////////////////////////////////////////////////////////////////////////	


TSize CDsa::WindowSize() const
	{
	TSize size = HwRect().Size();
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
	iTargetRect = TRect(aWindow.Position(), aWindow.Size());
    RestartL();
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
    
    RRegion* r = iDsa->DrawingRegion();
    iDsa->Gc()->SetClippingRegion(r);
    TRect rect = r->BoundingRect();
   
    if(rect.IsEmpty())
    	{
    	return;
    	}
    	
    iScreenRect = rect; //to ensure properly set, albeit may not(?) match to value SDL has - therefore may has to clip
	
	RecreateL();

    iStateFlags |= ERunning;
//    iScanLineWidth = iTargetBpp * HwRect().Width();
    ReleaseStop();
    if(iStateFlags & ESdlThreadSuspend)
    	{
    	EpocSdlEnv::Resume();
    	iStateFlags &= ~ ESdlThreadSuspend;
    	}
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
	
	EpocSdlEnv::ObserverEvent(MSDLObserver::EEventWindowReserved);
	
	return KErrNone;
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
*/
void CDsa::ClipCopy(TUint8* aTarget, const TUint8* aSource, const TRect& aRect, const TRect& aTargetPos) const
	{
	TUint8* target = aTarget;
	const TUint8* source = aSource;
	const TInt lineWidth = aRect.Width();
	source += iSourceBpp * (aRect.iTl.iY * lineWidth); 
	TInt sourceStartOffset = iSourceBpp *  aRect.iTl.iX;
	source += sourceStartOffset;
	target += iTargetBpp * ((aTargetPos.iTl.iY + aRect.iTl.iY ) * lineWidth); 
	TInt targetStartOffset = iTargetBpp * (aRect.iTl.iX + aTargetPos.iTl.iX);
	target += targetStartOffset;
	TUint32* targetPtr = reinterpret_cast<TUint32*>(target);
	const TInt targetWidth = HwRect().Size().iWidth;
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
	
	


void CDsa::Wipe() //dont call in drawing
	{
	if(IsDsaAvailable())
		Wipe(iTargetBpp * iScreenRect.Width() * iScreenRect.Height());
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
	
	
	TRect targetRect = HwRect();
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
		ClipCopy(target, aBits, updateRect, sourceRect);
		}
	else
		{
		const TInt byteCount = aRect.Width() * aRect.Height() * iSourceBpp; //this could be stored
		Mem::Copy(target, aBits, byteCount);
		}

	return ETrue;
	}
	
CDsa* CDsa::CreateL(RWsSession& aSession)
	{
	if(EpocSdlEnv::Flags(CSDL::EDrawModeDSB))
		{
		TInt flags = CDirectScreenBitmap::ENone;
		if(EpocSdlEnv::Flags(CSDL::EDrawModeDSBDoubleBuffer))
			flags |= CDirectScreenBitmap::EDoubleBuffer;
		if(EpocSdlEnv::Flags(CSDL::EDrawModeDSBIncrentalUpdate))
			flags |= CDirectScreenBitmap::EIncrementalUpdate;
		return new (ELeave) CDsaB(aSession);
		}
    else
        return new (ELeave) CDsaA(aSession);
	} 	

void CDsa::CreateZoomerL(const TSize& aSize)
	{
	iSwSize = aSize;
	iStateFlags |= EResizeRequest;
	CreateSurfaceL();
	SetTargetRect();
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
		
void CDsa::RecreateL()
	{
	}

void CDsa::Free()
	{
	}
		
void CDsa::UpdateSwSurface()
	{
	iTargetAddr = NULL;
	UnlockHwSurface();	//could be faster if does not use AO, but only check status before redraw, then no context switch needed
	}

void CDsa::SetBlitter(MBlitter* aBlitter)
	{
	iBlitter = aBlitter;
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
		
TInt CDsa::RedrawRequest()
	{
	if(!(iStateFlags & (EUpdating) && (iStateFlags & ERunning)))
		{
		return ExternalUpdate();
		}
	return KErrNotReady;
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
	Mem::Copy(aTarget, aSource, aBytes);
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
			

typedef TRgb (*TRgbFunc) (TInt aValue);

LOCAL_C TRgb rgb16MA(TInt aValue)
	{
	return TRgb::Color16MA(aValue);
	}
	
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
	private:
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
	case EGray256 : iFunc = TRgb::Gray256; break;
	case EColor256 : iFunc = TRgb::Color256; break;
	case EColor4K : iFunc = TRgb::Color4K; break;
	case EColor64K : iFunc = TRgb::Color64K; break;
	case EColor16M : iFunc = TRgb::Color16M; break;
	case EColor16MU : iFunc = TRgb::Color16MU; break;
	case EColor16MA : iFunc = rgb16MA; break;
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
			TUint32 value = *source++;
			*(--endt) = iFunc(value).Value();
			}
		}
	else
		{
		while(target < endt)
			{
			TUint32 value = *source++;
			*target++ = iFunc(value).Value();
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
			*(--endt) = iFunc(*column).Value();
			column += aLineLen;
			}
		}
	else
		{
		while(target < endt)
			{
			*target++ = iFunc(*column).Value();
			column += aLineLen;
			}
		}
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