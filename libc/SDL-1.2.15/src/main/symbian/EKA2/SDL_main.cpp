/*
    SDL_Main.cpp
    Symbian OS services for SDL

    Markus Mertama
*/


#include "epoc_sdl.h"

#include"sdlepocapi.h"
#include <e32base.h>
#include <estlib.h>
#include <stdio.h>
#include <badesca.h>

#include "vectorbuffer.h"
#include <w32std.h>
#include <aknappui.h>
#include <aknapp.h>
#include "SDL_epocevents_c.h"
#include "SDL_keysym.h"
#include "dsa.h"


#ifdef SYMBIANC
#include <reent.h>
#endif

//Markus Mertama


extern SDLKey* KeyMap();
extern void ResetKeyMap();

class CCurrentAppUi;

//const TUid KSDLUid =  { 0xF01F3D69 };

NONSHARABLE_CLASS(EnvUtils)
	{
	public:
	static void DisableKeyBlocking();
	static TBool Rendezvous(RThread& aThread, TRequestStatus& aStatus);
	};

TInt Panic(TInt aErr, TInt aLine)
	{
	TBuf<64> b;
	b.Format(_L("Main at %d"), aLine);
	User::Panic(b, aErr);
	return 0;	
	}
	

NONSHARABLE_CLASS(CCurrentAppUi) : public CAknAppUi
	{
	public:
	static CCurrentAppUi* Cast(CEikAppUi* aUi);
	void DisableKeyBlocking();
	};
	
	
CCurrentAppUi* CCurrentAppUi::Cast(CEikAppUi* aUi)
	{
	return static_cast<CCurrentAppUi*>(aUi);	
	}
	
void CCurrentAppUi::DisableKeyBlocking()	
	{
	SetKeyBlockMode(ENoKeyBlock);	
	}


class CEventQueue : public CBase, public MEventQueue
    {
    public:
        static CEventQueue* NewL();
        ~CEventQueue();
    public:
        TInt Append(const TWsEvent& aEvent);
       	const TWsEvent& Shift();
       	void Lock();
       	void Unlock();
        TBool HasData();
    private:
        TVector<TWsEvent, 64> iVector;
        RCriticalSection iCS;
    };
    
 CEventQueue* CEventQueue::NewL()
    {
    CEventQueue* q = new (ELeave) CEventQueue();
    CleanupStack::PushL(q);
    User::LeaveIfError(q->iCS.CreateLocal());
    CleanupStack::Pop();
    return q;
    }
    
CEventQueue::~CEventQueue()
    {
    iCS.Close();
    }
    
TInt CEventQueue::Append(const TWsEvent& aEvent)
    {
    iCS.Wait();
   	const TInt err = iVector.Append(aEvent);
    iCS.Signal();
    return err;
    }
    
    
TBool CEventQueue::HasData()
    {
    return iVector.Size() > 0;
    }


void CEventQueue::Lock()
	{
    iCS.Wait();
	}
	
void CEventQueue::Unlock()
	{
	iCS.Signal();
	}

const TWsEvent& CEventQueue::Shift()
    {
    const TWsEvent& event =  iVector.Shift();
    return event;
    }


TSdlCleanupItem::TSdlCleanupItem(TSdlCleanupOperation aOperation, TAny* aItem) :
iOperation(aOperation), iItem(aItem), iThread(RThread().Id())
    {
    }

class CEikonEnv;
class CSdlAppServ;

    
NONSHARABLE_CLASS(EpocSdlEnvData)
    {
    public:
    void Free();
    CEventQueue*            iEventQueue;
    TMainFunc				iMain;
    TInt            		iEpocEnvFlags;
    int                     iArgc;
    char**                  iArgv;
    CDsa*                   iDsa;
    CSdlAppServ*            iAppSrv;
    TThreadId               iId;
    CArrayFix<TSdlCleanupItem>* iCleanupItems; 
    CEikAppUi*				iAppUi;
    CSDL*					iSdl;
    };
  
   
EpocSdlEnvData* gEpocEnv;

#define MAINFUNC(x) EXPORT_C TMainFunc::TMainFunc(mainfunc##x aFunc){Mem::FillZ(iMainFunc, sizeof(iMainFunc)); iMainFunc[x - 1] = (void*) aFunc;}
    
MAINFUNC(1)
MAINFUNC(2)
MAINFUNC(3)
MAINFUNC(4)
MAINFUNC(5)
MAINFUNC(6)

EXPORT_C TMainFunc::TMainFunc() 
	{
	Mem::FillZ(iMainFunc, sizeof(iMainFunc));
	}
	

const void* TMainFunc::operator[](TInt aIndex) const
	{
	return iMainFunc[aIndex];
	}


NONSHARABLE_CLASS(CSdlAppServ) : public CActive
    {
    public:
        enum
            {
            EAppSrvNoop = CDsa::ELastDsaRequest,
            EAppSrvWindowWidth,
            EAppSrvWindowHeight,
            EAppSrvWindowDisplayMode,
            EAppSrvWindowPointerCursorMode,
            EAppSrvDsaStatus,
            EAppSrvStopThread,
            EAppSrvWaitDsa
            };
        CSdlAppServ();
        void ConstructL();
        ~CSdlAppServ();
        TInt Request(TInt aService);
        TInt RequestValue(TInt aService);
        void Init(); 
        void PanicMain(TInt aReason);
        void PanicMain(const TDesC& aInfo, TInt aReason);
        void SetObserver(MSDLObserver* aObserver);
        TInt ObserverEvent(TInt aEvent, TInt aParam);
        void SetParam(TInt aParam);
        void HandleObserverValue(TInt aService, TInt aReturnValue, TBool aMainThread);
        MSDLObserver* Observer();
    private:
        void RunL();
        void DoCancel();
    private:
        const TThreadId iMainId;
        RThread iAppThread;
        TInt iService;
        TInt iReturnValue;
        RSemaphore iSema;
        MSDLObserver* iObserver;
        TRequestStatus* iStatusPtr;
    };
    
CSdlAppServ::CSdlAppServ() : CActive(CActive::EPriorityHigh), iMainId(RThread().Id())
    {
    }
    
    
    
MSDLObserver* CSdlAppServ::Observer()
	{
	return iObserver;
	}
	
	
void CSdlAppServ::SetObserver(MSDLObserver* aObserver)
	{
	iObserver = aObserver;
	}	
	
TInt CSdlAppServ::ObserverEvent(TInt aEvent, TInt aParam)
	{
	if(iObserver != NULL)
		{
		if(RThread().Id() == gEpocEnv->iId)
			{
			return iObserver->SdlThreadEvent(aEvent, aParam);
			}
		else if(RThread().Id() == iMainId)
			{
			return iObserver->SdlEvent(aEvent, aParam);
			}
		PANIC(KErrNotSupported);
		}
	return 0;
	}
	
void CSdlAppServ::PanicMain(TInt aReason)    
    {
    iAppThread.Panic(RThread().Name(), aReason);
    }
    
void CSdlAppServ::PanicMain(const TDesC& aInfo, TInt aReason)    
    {
    iAppThread.Panic(aInfo, aReason);
    }    
    
void CSdlAppServ::ConstructL()
    {
    CActiveScheduler::Add(this);
    User::LeaveIfError(iSema.CreateLocal(1));
    iStatus = KRequestPending;
    iStatusPtr = &iStatus;
    SetActive();
    }
        
 CSdlAppServ::~CSdlAppServ()
    {
    Cancel();
    if(iSema.Handle() != NULL)
        iSema.Signal();
    iSema.Close();
    iAppThread.Close();
    }
    
TInt CSdlAppServ::Request(TInt aService)
    {
    if(RThread().Id() != iAppThread.Id())
    	{
    	iSema.Wait();
    	iService = aService;
    	iAppThread.RequestComplete(iStatusPtr, KErrNone); 
    	return KErrNone;
    	}
    return KErrBadHandle;
    }
    
TInt CSdlAppServ::RequestValue(TInt aService)
    {
    Request(aService);
    Request(EAppSrvNoop);
    return iReturnValue;
    }
   
void CSdlAppServ::Init()
    {
    PANIC_IF_ERROR(iAppThread.Open(iMainId));
    }

void CSdlAppServ::SetParam(TInt aParam)
	{
	iReturnValue = aParam;
	}
	
void CSdlAppServ::HandleObserverValue(TInt aService, TInt aReturnValue, TBool aMainThread)
	{
	if(iObserver != NULL && aMainThread)
		{
		switch(aService)
			{
			case MSDLObserver::EEventScreenSizeChanged:
			if(aReturnValue == MSDLObserver::EScreenSizeChangedDefaultPalette)
				EpocSdlEnv::LockPalette(EFalse);
			break;
			}
		}
	if(!aMainThread && aService == MSDLObserver::EEventSuspend)
		{
		if(iObserver == NULL || 
		(gEpocEnv->iDsa->Stopped() && aReturnValue != MSDLObserver::ESuspendNoSuspend))
			{
			EpocSdlEnv::Suspend();
			}
		}
	}

void CSdlAppServ::RunL()
    {
    if(iStatus == KErrNone)
        {
        switch(iService)
            {
            case CSdlAppServ::EAppSrvWaitDsa:
            	EpocSdlEnv::SetWaitDsa();
            	iReturnValue = EpocSdlEnv::IsDsaAvailable();
            //		}
            //	gEpocEnv->iDsa->Stop();
            //	gEpocEnv->iDsa->RestartL();
            	break;
           	 case CSdlAppServ::EAppSrvStopThread:
            	gEpocEnv->iDsa->SetSuspend();
            	break;
            case EpocSdlEnv::EDisableKeyBlocking:
                EnvUtils::DisableKeyBlocking();
                break;
          
            case EAppSrvWindowPointerCursorMode:
                iReturnValue = gEpocEnv->iDsa != NULL ?
                 gEpocEnv->iDsa->Session().PointerCursorMode() : KErrNotReady; 
                break;
            case EAppSrvDsaStatus:
            	gEpocEnv->iDsa->Stop();
                iReturnValue = KErrNone;
                break;
            case CDsa::ERequestUpdate:
            	gEpocEnv->iDsa->UnlockHWSurfaceRequestComplete();
            	break;
            case EAppSrvNoop:
                break;
            case MSDLObserver::EEventResume:
            case MSDLObserver::EEventSuspend:
            case MSDLObserver::EEventScreenSizeChanged:
            case MSDLObserver::EEventWindowReserved:
            case MSDLObserver::EEventKeyMapInit:
            case MSDLObserver::EEventWindowNotAvailable:
            case MSDLObserver::EEventMainExit:
            	iReturnValue = ObserverEvent(iService, iReturnValue);
            	HandleObserverValue(iService, iReturnValue, ETrue);
            	break;
            default:
                PANIC(KErrNotSupported);
            }
        iStatus = KRequestPending;
        iStatusPtr = &iStatus;
        SetActive();
        }
    iSema.Signal();
    }
    
void CSdlAppServ::DoCancel()
    {
    iSema.Wait();
    TRequestStatus* s = &iStatus;
    iAppThread.RequestComplete(s, KErrCancel); 
    }
 


MEventQueue& EpocSdlEnv::EventQueue()
    {
    __ASSERT_DEBUG(gEpocEnv != NULL, PANIC(KErrNotReady));
    return *gEpocEnv->iEventQueue;
    }


TBool EpocSdlEnv::Flags(TInt aFlag)
    {
	const TInt flag = gEpocEnv->iEpocEnvFlags & aFlag;
	return flag == aFlag;
    }

TInt EpocSdlEnv::Argc()
    {
    __ASSERT_DEBUG(gEpocEnv != NULL, PANIC(KErrNotReady));
    return gEpocEnv->iArgc;
    }
    
    
char** EpocSdlEnv::Argv()
    {
    __ASSERT_DEBUG(gEpocEnv != NULL, PANIC(KErrNotReady));
    return gEpocEnv->iArgv;
    }
    
    
TBool EpocSdlEnv::IsDsaAvailable()
    {
    __ASSERT_DEBUG(gEpocEnv != NULL, PANIC(KErrNotReady));
    return gEpocEnv->iDsa != NULL && gEpocEnv->iDsa->IsDsaAvailable();
    }

  
void EpocSdlEnv::WaitDsaAvailable()
	{
	EpocSdlEnv::ObserverEvent(MSDLObserver::EEventWindowNotAvailable, 0);
	gEpocEnv->iAppSrv->Request(CSdlAppServ::EAppSrvStopThread);
	if(EpocSdlEnv::Flags(CSDL::EEnableFocusStop))
		{
		EpocSdlEnv::ObserverEvent(MSDLObserver::EEventSuspend, 0);
		}
	}
	
void EpocSdlEnv::Suspend()
	{
	if(gEpocEnv->iDsa->Stopped() || EpocSdlEnv::Flags(CSDL::EEnableFocusStop))
		{
	//	gEpocEnv->iDsa->ReleaseStop(); 
		gEpocEnv->iDsa->SetSuspend(); 
		RThread().Suspend();
		EpocSdlEnv::ObserverEvent(MSDLObserver::EEventResume, 0);
		}
	}
	
void EpocSdlEnv::SetWaitDsa()
	{
	if(!IsDsaAvailable())
		{
		RThread th;
		th.Open(gEpocEnv->iId);
		th.Suspend();
		th.Close();
		gEpocEnv->iDsa->SetSuspend(); 
		}
	}
	
void EpocSdlEnv::Resume()
	{
	gEpocEnv->iDsa->Resume();
	RThread th;
	th.Open(gEpocEnv->iId);
	th.Resume();
	th.Close();
	
	const TInt value = gEpocEnv->iAppSrv->ObserverEvent(MSDLObserver::EEventResume, 0);
	gEpocEnv->iAppSrv->HandleObserverValue(MSDLObserver::EEventResume, value, ETrue);
	}
    

TInt EpocSdlEnv::AllocSwSurface(const TSize& aSize, TDisplayMode aMode)
	{
	return gEpocEnv->iDsa->AllocSurface(EFalse, aSize, aMode);
	}
	
TInt EpocSdlEnv::AllocHwSurface(const TSize& aSize, TDisplayMode aMode)
	{
	return gEpocEnv->iDsa->AllocSurface(ETrue, aSize, aMode);
	}
		
	
void EpocSdlEnv::UnlockHwSurface()
	{
	gEpocEnv->iDsa->UnlockHwSurface();
	}
	
TUint8* EpocSdlEnv::LockHwSurface()
	{
	return gEpocEnv->iDsa->LockHwSurface();
	}


void EpocSdlEnv::UpdateSwSurface()
	{
	gEpocEnv->iDsa->UpdateSwSurface();
	}
	
TBool EpocSdlEnv::AddUpdateRect(TUint8* aAddress, const TRect& aUpdateRect, const TRect& aRect)
	{
	return gEpocEnv->iDsa->AddUpdateRect(aAddress, aUpdateRect, aRect);
	}
	
void EpocSdlEnv::Request(TInt aService)
    {
    __ASSERT_DEBUG(gEpocEnv != NULL, PANIC(KErrNotReady));
    gEpocEnv->iAppSrv->Request(aService);
    }
    
    
TSize EpocSdlEnv::WindowSize(const TSize& aRequestedSize)
    { 
    __ASSERT_DEBUG(gEpocEnv != NULL, PANIC(KErrNotReady));
    if(EpocSdlEnv::Flags(CSDL::EAllowImageResize) && gEpocEnv->iDsa->WindowSize() != aRequestedSize)
    	{
    	TRAP_IGNORE(gEpocEnv->iDsa->CreateZoomerL(aRequestedSize));
    	}
    return gEpocEnv->iDsa->WindowSize();
    }
    
 TSize EpocSdlEnv::WindowSize()
    { 
    __ASSERT_DEBUG(gEpocEnv != NULL, PANIC(KErrNotReady));
    return gEpocEnv->iDsa->WindowSize();
    }   
    
TDisplayMode EpocSdlEnv::DisplayMode()
    {
    return gEpocEnv->iDsa->DisplayMode();
    }
    
TPointerCursorMode EpocSdlEnv::PointerMode()
    {
    return static_cast<TPointerCursorMode>
    (gEpocEnv->iAppSrv->RequestValue(CSdlAppServ::EAppSrvWindowPointerCursorMode));
    }
    
TInt EpocSdlEnv::SetPalette(TInt aFirstcolor, TInt aColorCount, TUint32* aPalette)  
	{
	return 	gEpocEnv->iDsa->SetPalette(aFirstcolor, aColorCount, aPalette);
	}

void EpocSdlEnv::PanicMain(TInt aErr)
    {
    gEpocEnv->iAppSrv->PanicMain(aErr);
    }
    
    
TInt EpocSdlEnv::AppendCleanupItem(const TSdlCleanupItem& aItem)
    {
    TRAPD(err, gEpocEnv->iCleanupItems->AppendL(aItem));
    return err;
    }
    
void EpocSdlEnv::RemoveCleanupItem(TAny* aItem)
    {
    for(TInt i = 0; i < gEpocEnv->iCleanupItems->Count(); i++)
        {
        if(gEpocEnv->iCleanupItems->At(i).iItem == aItem)
            gEpocEnv->iCleanupItems->Delete(i);
        }
    }
    
void EpocSdlEnv::CleanupItems()     
	{
	const TThreadId id = RThread().Id();
	TInt last = gEpocEnv->iCleanupItems->Count() - 1;
	TInt i;
	for(i = last; i >= 0 ; i--)
        {
        TSdlCleanupItem& item = gEpocEnv->iCleanupItems->At(i);
        if(item.iThread == id)
        	{
        	item.iThread = TThreadId(0); 
        	item.iOperation(item.iItem);
        	}
        }
    last = gEpocEnv->iCleanupItems->Count() - 1;
	for(i = last; i >= 0 ; i--)
        {
        TSdlCleanupItem& item = gEpocEnv->iCleanupItems->At(i);
        if(item.iThread == TThreadId(0))
        	{
        	gEpocEnv->iCleanupItems->Delete(i);
        	}
        }
	}
	
void EpocSdlEnv::FreeSurface()
	{
	Request(CSdlAppServ::EAppSrvDsaStatus);
	gEpocEnv->iDsa->Free();
	}
  
void EpocSdlEnv::LockPalette(TBool aLock)
	{
	gEpocEnv->iDsa->LockPalette(aLock);
	}
    
void EpocSdlEnv::ObserverEvent(TInt aService, TInt aParam)
	{
	const TBool sdlThread = RThread().Id() == gEpocEnv->iId;
	const TInt valuea = gEpocEnv->iAppSrv->ObserverEvent(aService, aParam);
	gEpocEnv->iAppSrv->HandleObserverValue(aService, valuea, !sdlThread);
	if(sdlThread)
		{
		gEpocEnv->iAppSrv->SetParam(aParam);
		const TInt valuet = gEpocEnv->iAppSrv->RequestValue(aService);
		gEpocEnv->iAppSrv->HandleObserverValue(aService, valuet, EFalse);	
		}
	}
			
    
TPoint EpocSdlEnv::WindowCoordinates(const TPoint& aPoint)    
    {
    return gEpocEnv->iDsa->WindowCoordinates(aPoint);	
    }
    
void EpocSdlEnv::PanicMain(const TDesC& aInfo, TInt aErr)
    {
    gEpocEnv->iAppSrv->PanicMain(aInfo, aErr);
    }
//Dsa is a low priority ao, it has to wait if its pending event, but ws
//event has been prioritized before it
//this is not called from app thread!
void EpocSdlEnv::WaitDeviceChange() 
    {
  	LockPalette(ETrue);
    gEpocEnv->iAppSrv->RequestValue(CSdlAppServ::EAppSrvWaitDsa);
    const TSize sz = WindowSize();
    const TInt param = reinterpret_cast<TInt>(&sz);
    ObserverEvent(MSDLObserver::EEventScreenSizeChanged, param);
    	
   // RThread().Suspend();
    }  
    
LOCAL_C TBool CheckSdl() 
    {
    TInt isExit = ETrue;
    RThread sdl;
    if(sdl.Open(gEpocEnv->iId) == KErrNone)
        {
        if(sdl.ExitType() == EExitPending)
            {
            isExit = EFalse;
            }
        sdl.Close();
        }
    return isExit;
    }
    
void EpocSdlEnvData::Free()
    {
    if(RThread().Id() == gEpocEnv->iId)
    	{
    	iDsa->Free();
    	return;
    	}
   
    __ASSERT_ALWAYS(iArgv == NULL || CheckSdl(), PANIC(KErrNotReady));
        
    for(TInt i = 0; i < iArgc; i++)
        User::Free( iArgv[i] );
        
    User::Free(iArgv);	
     
    
    delete iEventQueue;
    
    if(iDsa != NULL)
    	iDsa->Free();
    
	delete iDsa;
	delete iAppSrv;
    }

_LIT(KSDLMain, "SDLMain");

LOCAL_C int MainL()
    {
    gEpocEnv->iCleanupItems = new (ELeave) CArrayFixFlat<TSdlCleanupItem>(8);
    
    char** envp=0;
     /* !! process exits here if there is "exit()" in main! */
    int ret = 0;
    for(TInt i = 0; i  < 6; i++)
        {
        void* f = (void*) gEpocEnv->iMain[i];
        if(f != NULL)
            {
            switch(i)
                {
                case 0:
                    ret = ((mainfunc1)f)(); 
                    return ret;
                case 3:
                    ((mainfunc1)f)(); 
                    return ret;
                case 1:
                    ret = ((mainfunc2)f)(EpocSdlEnv::Argc(), EpocSdlEnv::Argv());
                    return ret;
                case 4:
                    ((mainfunc2)f)(EpocSdlEnv::Argc(), EpocSdlEnv::Argv());
                    return ret;
                case 2:
                    ret = ((mainfunc3)f)(EpocSdlEnv::Argc(), EpocSdlEnv::Argv(), envp);
                    return ret;
                case 5:
                    ((mainfunc3)f)(EpocSdlEnv::Argc(), EpocSdlEnv::Argv(), envp);
                    return ret;
                }
            }
        }
    PANIC(KErrNotFound);
    return 0;
    }

LOCAL_C TInt DoMain(TAny* /*aParam*/)
    {
    
    
    CTrapCleanup* cleanup = CTrapCleanup::New();
      	
	TBool fbsconnected = EFalse;
	if(RFbsSession::GetSession() == NULL)
	    {
	    PANIC_IF_ERROR(RFbsSession::Connect());
	    fbsconnected = ETrue;
	    }
	
 	gEpocEnv->iAppSrv->Init();	

#ifdef SYMBIANC 
    // Create stdlib 
    _REENT;
#endif

    // Call stdlib main
    int ret = 0;
    
    //completes waiting rendesvous
    RThread::Rendezvous(KErrNone);
    
    TRAPD(err, err = MainL());
    
    EpocSdlEnv::ObserverEvent(MSDLObserver::EEventMainExit, err);
   
    // Free resources and return
    
  	EpocSdlEnv::CleanupItems();
        
    gEpocEnv->iCleanupItems->Reset();
    delete gEpocEnv->iCleanupItems;
    gEpocEnv->iCleanupItems = NULL;
    
    gEpocEnv->Free(); //free up in thread resources 
    
#ifdef SYMBIANC    
    _cleanup(); //this is normally called at exit, I call it here
#endif

    if(fbsconnected)
        RFbsSession::Disconnect();
    
#ifdef SYMBIANC     
    CloseSTDLIB();
#endif
       
 //   delete as;
   	delete cleanup;	

    return err == KErrNone ? ret : err;;
    }
    

    
EXPORT_C CSDL::~CSDL()
    {
   	gEpocEnv->Free();
    User::Free(gEpocEnv);
    gEpocEnv->iSdl = NULL;
    }

EXPORT_C CSDL* CSDL::NewL(TInt aFlags)
    {
    __ASSERT_ALWAYS(gEpocEnv == NULL, PANIC(KErrAlreadyExists));
    gEpocEnv = (EpocSdlEnvData*) User::AllocL(sizeof(EpocSdlEnvData));
    Mem::FillZ(gEpocEnv, sizeof(EpocSdlEnvData));
   
    gEpocEnv->iEpocEnvFlags = aFlags;
    gEpocEnv->iEventQueue = CEventQueue::NewL();
   
    gEpocEnv->iAppSrv = new (ELeave) CSdlAppServ();
    gEpocEnv->iAppSrv->ConstructL();
    
    CSDL* sdl = new (ELeave) CSDL();
    
    gEpocEnv->iSdl = sdl;
    
    return sdl;
    }
    
  /*  
EXPORT_C void CSDL::ReInitL(TFlags aFlags)
	{
	const TFlags prevFlags = gEpocEnv->iEpocEnvFlags;
	gEpocEnv->iEpocEnvFlags = aFlags;
	TInt err = KErrNone;
	if(((prevFlags & EDrawModeDSB) != (aFlags & EDrawModeDSB)) && gEpocEnv->iDsa)
		{
		delete gEpocEnv->iDsa;
		gEpocEnv->iDsa = NULL;
		gEpocEnv->iDsa = CDsa::RecreateL(EpocSdlEnv::Flags(CSDL::EDrawModeDSB));
		}
	}
 */


EXPORT_C void CSDL::SetContainerWindowL(RWindow& aWindow, RWsSession& aSession, CWsScreenDevice& aDevice)
    {
    if(gEpocEnv->iDsa == NULL)
    	gEpocEnv->iDsa = CDsa::CreateL(aSession);
    gEpocEnv->iDsa->ConstructL(aWindow, aDevice);
    }
        
   
EXPORT_C TThreadId CSDL::CallMainL(const TMainFunc& aFunc, TRequestStatus* const aStatus, const CDesC8Array* const aArg, TInt aFlags, TInt aStackSize)
    {
    ASSERT(gEpocEnv != NULL);
    gEpocEnv->iMain = aFunc;
    const TBool args = aArg != NULL;
    
    gEpocEnv->iArgc = aArg->Count() + 1;
    gEpocEnv->iArgv = (char**) User::AllocL(sizeof(char*) * (gEpocEnv->iArgc + 1));  
      
    TInt k = 0;
    const TFileName processName = RProcess().FileName();
    const TInt len = processName.Length();
    gEpocEnv->iArgv[k] = (char*) User::AllocL(len + 1);
    Mem::Copy(gEpocEnv->iArgv[k], processName.Ptr(), len);
    gEpocEnv->iArgv[k][len] = 0;
      
    for(TInt i =  0; args && (i < aArg->Count()); i++)
        {
        k++;
        const TInt len = aArg->MdcaPoint(i).Length();
        gEpocEnv->iArgv[k] = (char*) User::AllocL(len + 1);
        Mem::Copy(gEpocEnv->iArgv[k], aArg->MdcaPoint(i).Ptr(), len);
        gEpocEnv->iArgv[k][len] = 0;
        }
        
    gEpocEnv->iArgv[gEpocEnv->iArgc] = NULL;
         
    RThread thread;
    User::LeaveIfError(thread.Create(KSDLMain, DoMain, aStackSize, NULL, NULL));
    
    if(aStatus != NULL)
    	{
    	thread.Logon(*aStatus);
    	}
    	
    gEpocEnv->iId = thread.Id();
    thread.SetPriority(EPriorityLess);
    if((aFlags & CSDL::ERequestResume) == 0)
        {
        thread.Resume();
        }
    thread.Close();
    return gEpocEnv->iId;
    }
    
EXPORT_C TInt CSDL::AppendWsEvent(const TWsEvent& aEvent)
    {
    return EpocSdlEnv::EventQueue().Append(aEvent);
    }
    
EXPORT_C void CSDL::SDLPanic(const TDesC& aInfo, TInt aErr)
    {
    EpocSdlEnv::PanicMain(aInfo, aErr);
    }
    
EXPORT_C TInt CSDL::GetSDLCode(TInt aScanCode)
    {
    if(aScanCode < 0)
        return MAX_SCANCODE;
    if(aScanCode >= MAX_SCANCODE)
        return -1;
    return KeyMap()[aScanCode];
    }
    
EXPORT_C TInt CSDL::SDLCodesCount() const
	{
	return MAX_SCANCODE;
	}
	
EXPORT_C void CSDL::ResetSDLCodes()
	{
	ResetKeyMap();
	}
    
EXPORT_C void CSDL::SetOrientation(TOrientationMode aMode)
	{
	gEpocEnv->iDsa->SetOrientation(aMode);
	}
    
EXPORT_C TInt CSDL::SetSDLCode(TInt aScanCode, TInt aSDLCode)
    {
    const TInt current = GetSDLCode(aScanCode);
    if(aScanCode >= 0 && aScanCode < MAX_SCANCODE)
        KeyMap()[aScanCode] = static_cast<SDLKey>(aSDLCode);
    return current;
    }


EXPORT_C MSDLObserver* CSDL::Observer()
	{
	return gEpocEnv->iAppSrv->Observer();
	}    
    
EXPORT_C void CSDL::SetObserver(MSDLObserver* aObserver)
	{
	gEpocEnv->iAppSrv->SetObserver(aObserver);
	}
	
EXPORT_C void CSDL::Resume()
	{
	EpocSdlEnv::Resume();
	}
	
EXPORT_C void CSDL::Suspend()
	{
	gEpocEnv->iDsa->DoStop();
	}
	
EXPORT_C CSDL::CSDL()
    {
    }

EXPORT_C void CSDL::DisableKeyBlocking(CAknAppUi& aAppUi) const
	{
	gEpocEnv->iAppUi = &aAppUi;
	EnvUtils::DisableKeyBlocking();
	}

EXPORT_C TInt CSDL::SetBlitter(MBlitter* aBlitter)
	{
	if(gEpocEnv && gEpocEnv->iDsa)
		{
		gEpocEnv->iDsa->SetBlitter(aBlitter);
		return KErrNone;
		}
	return KErrNotReady;
	}
		
	
EXPORT_C TInt CSDL::AppendOverlay(MOverlay& aOverlay, TInt aPriority)
	{
	if(gEpocEnv && gEpocEnv->iDsa)
		{
		return gEpocEnv->iDsa->AppendOverlay(aOverlay, aPriority);
		}
	return KErrNotReady;
	}

EXPORT_C TInt CSDL::RemoveOverlay(MOverlay& aOverlay)	
	{
	if(gEpocEnv && gEpocEnv->iDsa)
		{
		return gEpocEnv->iDsa->RemoveOverlay(aOverlay);
		}
	return KErrNotReady;
	}

EXPORT_C TInt CSDL::RedrawRequest()
	{
	if(gEpocEnv && gEpocEnv->iDsa)
		{
		return gEpocEnv->iDsa->RedrawRequest();
		}
	return KErrNotReady;
	}
	
/*
EXPORT_C CSDL* CSDL::Current()
    {
    return gEpocEnv != NULL ? gEpocEnv->iSdl : NULL;
    }

    
EXPORT_C TInt CSDL::SetVolume(TInt aVolume)
    {
    return EpocSdlEnv::SetVolume(aVolume);
    } 
    
EXPORT_C TInt CSDL::Volume() const
    {
    return EpocSdlEnv::Volume();
    }     
	
EXPORT_C TInt CSDL::MaxVolume() const
    {
    return EpocSdlEnv::MaxVolume();
    } 	
*/
			
void EnvUtils::DisableKeyBlocking()
	{
	if(gEpocEnv->iAppUi != NULL)
		return CCurrentAppUi::Cast(gEpocEnv->iAppUi)->DisableKeyBlocking();
	}
	
TBool EnvUtils::Rendezvous(RThread& aThread, TRequestStatus& aStatus)
	{
	if(gEpocEnv->iId != TThreadId(0) &&
    		 	aThread.Open(gEpocEnv->iId) &&
    		  	aThread.ExitType() == EExitPending)
    			{
    			aThread.Rendezvous(aStatus);
    			return ETrue;
    			}
    return EFalse;
	}
	
	

