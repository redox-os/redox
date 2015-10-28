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
    SDL_epocevents.cpp
    Handle the event stream, converting Epoc events into SDL events

    Epoc version by Hannu Viitala (hannu.j.viitala@mbnet.fi)
*/


#include <stdio.h>
#undef NULL
extern "C" {
//#define DEBUG_TRACE_ENABLED
#include "SDL_error.h"
#include "SDL_video.h"
#include "SDL_keysym.h"
#include "SDL_keyboard.h"
#include "SDL_events_c.h"
#include "SDL_timer.h"
}; /* extern "C" */

#include "SDL_epocvideo.h"
#include "SDL_epocevents_c.h"

#include<linereader.h>
#include<bautils.h>


#include <hal.h>

extern "C" {
/* The translation tables from a console scancode to a SDL keysym */
static SDLKey keymap[MAX_SCANCODE];
static SDL_keysym *TranslateKey(_THIS, int scancode, SDL_keysym *keysym);
void DisableKeyBlocking(_THIS);
}; /* extern "C" */

TBool isCursorVisible = EFalse;

int EPOC_HandleWsEvent(_THIS, const TWsEvent& aWsEvent)
{
    int posted = 0;
    SDL_keysym keysym;
    
//    SDL_TRACE1("hws %d", aWsEvent.Type());

    switch (aWsEvent.Type())
		{    
    case EEventPointer: /* Mouse pointer events */
		{

        const TPointerCursorMode mode =  Private->EPOC_WsSession.PointerCursorMode();

        if(mode == EPointerCursorNone) 
            {
            return 0; //TODO: Find out why events are get despite of cursor should be off
            }

        const TPointerEvent* pointerEvent = aWsEvent.Pointer();
        TPoint mousePos = pointerEvent->iPosition;

        /*!! TODO Pointer do not yet work properly
        //SDL_TRACE1("SDL: EPOC_HandleWsEvent, pointerEvent->iType=%d", pointerEvent->iType); //!!

        if (Private->EPOC_ShrinkedHeight) {
            mousePos.iY <<= 1; // Scale y coordinate to shrinked screen height
        }
        if (Private->EPOC_ShrinkedWidth) {
            mousePos.iX <<= 1; // Scale x coordinate to shrinked screen width
        }
        */

		posted += SDL_PrivateMouseMotion(0, 0, mousePos.iX, mousePos.iY); /* Absolute position on screen */

		switch (pointerEvent->iType)
			{
        case TPointerEvent::EButton1Down:
            posted += SDL_PrivateMouseButton(SDL_PRESSED, SDL_BUTTON_LEFT, 0, 0);
			break;
        case TPointerEvent::EButton1Up:
			posted += SDL_PrivateMouseButton(SDL_RELEASED, SDL_BUTTON_LEFT, 0, 0);
			break;
        case TPointerEvent::EButton2Down:
            posted += SDL_PrivateMouseButton(SDL_PRESSED, SDL_BUTTON_RIGHT, 0, 0);
			break;
		case TPointerEvent::EButton2Up:
			posted += SDL_PrivateMouseButton(SDL_RELEASED, SDL_BUTTON_RIGHT, 0, 0);
			break;
        case TPointerEvent::EButton3Down:
            posted += SDL_PrivateMouseButton(SDL_PRESSED, SDL_BUTTON_MIDDLE, 0, 0);
			break;
        case TPointerEvent::EButton3Up:
			posted += SDL_PrivateMouseButton(SDL_RELEASED, SDL_BUTTON_MIDDLE, 0, 0);
			break;
			} // switch
        break;
	    }
    
    case EEventKeyDown: /* Key events */
    {
#ifdef SYMBIAN_CRYSTAL
		// special case: 9300/9500 rocker down, simulate left mouse button
		if (aWsEvent.Key()->iScanCode == EStdKeyDeviceA)
			{
            const TPointerCursorMode mode =  Private->EPOC_WsSession.PointerCursorMode();
            if(mode != EPointerCursorNone) 
                posted += SDL_PrivateMouseButton(SDL_PRESSED, SDL_BUTTON_LEFT, 0, 0);
			}
#endif
       (void*)TranslateKey(_this, aWsEvent.Key()->iScanCode, &keysym);
            
#ifndef DISABLE_JOYSTICK
        /* Special handling */
        switch((int)keysym.sym) {
        case SDLK_CAPSLOCK:
            if (!isCursorVisible) {
                /* Enable virtual cursor */
	            HAL::Set(HAL::EMouseState, HAL::EMouseState_Visible);
            }
            else {
                /* Disable virtual cursor */
                HAL::Set(HAL::EMouseState, HAL::EMouseState_Invisible);
            }
            isCursorVisible = !isCursorVisible;
            break;
        }
#endif        
	    posted += SDL_PrivateKeyboard(SDL_PRESSED, &keysym);
        break;
	} 

    case EEventKeyUp: /* Key events */
		{
#ifdef SYMBIAN_CRYSTAL
		// special case: 9300/9500 rocker up, simulate left mouse button
		if (aWsEvent.Key()->iScanCode == EStdKeyDeviceA)
			{
            posted += SDL_PrivateMouseButton(SDL_RELEASED, SDL_BUTTON_LEFT, 0, 0);
			}
#endif
	    posted += SDL_PrivateKeyboard(SDL_RELEASED, TranslateKey(_this, aWsEvent.Key()->iScanCode, &keysym));
        break;
		}
    
    case EEventFocusGained: /* SDL window got focus */
	    {
        Private->EPOC_IsWindowFocused = ETrue;
		posted += SDL_PrivateAppActive(1, SDL_APPINPUTFOCUS|SDL_APPMOUSEFOCUS);
        /* Draw window background and screen buffer */
        DisableKeyBlocking(_this);  //Markus: guess why:-)
 
        RedrawWindowL(_this);  
        break;
	    }

    case EEventFocusLost: /* SDL window lost focus */
		{
/*        
        CFbsBitmap* bmp = new (ELeave) CFbsBitmap();
        bmp->Create(Private->EPOC_ScreenSize, Private->EPOC_DisplayMode);
        Private->EPOC_WsScreen->CopyScreenToBitmap(bmp);
        Private->EPOC_WindowGc->Activate(Private->EPOC_WsWindow);
        Private->EPOC_WsWindow.BeginRedraw(TRect(Private->EPOC_WsWindow.Size()));
	    Private->EPOC_WindowGc->BitBlt(TPoint(0, 0), bmp);
	    Private->EPOC_WsWindow.EndRedraw();
	    Private->EPOC_WindowGc->Deactivate();
        bmp->Save(_L("C:\\scr.mbm"));
        delete bmp;
*/       

		Private->EPOC_IsWindowFocused = EFalse;

		posted += SDL_PrivateAppActive(0, SDL_APPINPUTFOCUS|SDL_APPMOUSEFOCUS);

        RWsSession s;
        s.Connect();
        RWindowGroup g(s);
        g.Construct(TUint32(&g), EFalse);
        g.EnableReceiptOfFocus(EFalse);
        RWindow w(s);
        w.Construct(g, TUint32(&w));
        w.SetExtent(TPoint(0, 0), Private->EPOC_WsWindow.Size());
        w.SetOrdinalPosition(0);
        w.Activate();
        w.Close();
        g.Close();
        s.Close();

/*
        Private->EPOC_WsSession.SetWindowGroupOrdinalPosition(Private->EPOC_WsWindowGroupID, -1);

            
        SDL_Delay(500);
        TInt focus = -1;
        while(focus < 0)
            {
            const TInt curr = Private->EPOC_WsSession.GetFocusWindowGroup();
            if(curr != Private->EPOC_WsWindowGroupID)
                focus = curr;
            else
                SDL_Delay(500);
            }

        if(1 < Private->EPOC_WsSession.GetWindowGroupOrdinalPriority(Private->EPOC_WsWindowGroupID))
            {
            Private->EPOC_WsSession.SetWindowGroupOrdinalPosition(focus, -1);
            SDL_Delay(500);
            Private->EPOC_WsSession.SetWindowGroupOrdinalPosition(focus, 0);
            }
*/
        /*//and the request redraw
        TRawEvent redrawEvent;
        redrawEvent.Set(TRawEvent::ERedraw);
        Private->EPOC_WsSession.SimulateRawEvent(redrawEvent);
        Private->EPOC_WsSession.Flush();*/
#if 0
        //!! Not used
        // Wait and eat events until focus is gained again
	    while (ETrue) {
            Private->EPOC_WsSession.EventReady(&Private->EPOC_WsEventStatus);
            User::WaitForRequest(Private->EPOC_WsEventStatus);
		    Private->EPOC_WsSession.GetEvent(Private->EPOC_WsEvent);
            TInt eventType = Private->EPOC_WsEvent.Type();
		    Private->EPOC_WsEventStatus = KRequestPending;
		    //Private->EPOC_WsSession.EventReady(&Private->EPOC_WsEventStatus);
            if (eventType == EEventFocusGained) {
                RedrawWindowL(_this);
                break;
            }
	    }
#endif
        break;
	    }

    case EEventModifiersChanged: 
    {
	    TModifiersChangedEvent* modEvent = aWsEvent.ModifiersChanged();
        TUint modstate = KMOD_NONE;
        if (modEvent->iModifiers == EModifierLeftShift)
            modstate |= KMOD_LSHIFT;
        if (modEvent->iModifiers == EModifierRightShift)
            modstate |= KMOD_RSHIFT;
        if (modEvent->iModifiers == EModifierLeftCtrl)
            modstate |= KMOD_LCTRL;
        if (modEvent->iModifiers == EModifierRightCtrl)
            modstate |= KMOD_RCTRL;
        if (modEvent->iModifiers == EModifierLeftAlt)
            modstate |= KMOD_LALT;
        if (modEvent->iModifiers == EModifierRightAlt)
            modstate |= KMOD_RALT;
        if (modEvent->iModifiers == EModifierLeftFunc)
            modstate |= KMOD_LMETA;
        if (modEvent->iModifiers == EModifierRightFunc)
            modstate |= KMOD_RMETA;
        if (modEvent->iModifiers == EModifierCapsLock)
            modstate |= KMOD_CAPS;
        SDL_SetModState(STATIC_CAST(SDLMod,(modstate | KMOD_LSHIFT)));
        break;
    }
    default:            
        break;
	} 
	
    return posted;
}

extern "C" {

void EPOC_PumpEvents(_THIS)
{
    int posted = 0; // !! Do we need this?
    //Private->EPOC_WsSession.EventReady(&Private->EPOC_WsEventStatus);
	while (Private->EPOC_WsEventStatus != KRequestPending) {

		Private->EPOC_WsSession.GetEvent(Private->EPOC_WsEvent);
		posted = EPOC_HandleWsEvent(_this, Private->EPOC_WsEvent);
		Private->EPOC_WsEventStatus = KRequestPending;
		Private->EPOC_WsSession.EventReady(&Private->EPOC_WsEventStatus);
	}
}


_LIT(KMapFileName, "C:\\sdl_info\\sdlkeymap.cfg");
LOCAL_C void ReadL(RFs& aFs, RArray<TInt>& aArray)
    {
    TInt drive = -1;
    TFileName name(KMapFileName);
    for(TInt i = 'z'; drive < 0 && i >= 'a'; i--)
        {
        name[0] = (TUint16)i;
        if(BaflUtils::FileExists(aFs, name))
            drive = i;
        }
    if(drive < 0)
        return;
    CLineReader* reader = CLineReader::NewLC(aFs, name);
    while(reader->NextL())
        {
        TPtrC ln = reader->Current();
        TLex line(ln);
        TInt n = 0;
        for(;;)
            {
            const TPtrC token = line.NextToken();
            if(token.Length() == 0)
                break;
            if((n & 1) != 0)
                {
                TInt value;
                TLex lex(token);
                User::LeaveIfError(lex.Val(value));
                User::LeaveIfError(aArray.Append(value));
                }
            n++;
            }
        }
    CleanupStack::PopAndDestroy();
    }


void EPOC_InitOSKeymap(_THIS)
{
	int i;

	/* Initialize the key translation table */
	for ( i=0; i<SDL_TABLESIZE(keymap); ++i )
		keymap[i] = SDLK_UNKNOWN;


	/* Numbers */
	for ( i = 0; i<32; ++i ){
		keymap[' ' + i] = (SDLKey)(SDLK_SPACE+i);
	}
	/* e.g. Alphabet keys */
	for ( i = 0; i<32; ++i ){
		keymap['A' + i] = (SDLKey)(SDLK_a+i);
	}

	keymap[EStdKeyBackspace]    = SDLK_BACKSPACE;
	keymap[EStdKeyTab]          = SDLK_TAB;
	keymap[EStdKeyEnter]        = SDLK_RETURN;
	keymap[EStdKeyEscape]       = SDLK_ESCAPE;
   	keymap[EStdKeySpace]        = SDLK_SPACE;
   	keymap[EStdKeyPause]        = SDLK_PAUSE;
   	keymap[EStdKeyHome]         = SDLK_HOME;
   	keymap[EStdKeyEnd]          = SDLK_END;
   	keymap[EStdKeyPageUp]       = SDLK_PAGEUP;
   	keymap[EStdKeyPageDown]     = SDLK_PAGEDOWN;
	keymap[EStdKeyDelete]       = SDLK_DELETE;
	keymap[EStdKeyUpArrow]      = SDLK_UP;
	keymap[EStdKeyDownArrow]    = SDLK_DOWN;
	keymap[EStdKeyLeftArrow]    = SDLK_LEFT;
	keymap[EStdKeyRightArrow]   = SDLK_RIGHT;
	keymap[EStdKeyCapsLock]     = SDLK_CAPSLOCK;
	keymap[EStdKeyLeftShift]    = SDLK_LSHIFT;
	keymap[EStdKeyRightShift]   = SDLK_RSHIFT;
	keymap[EStdKeyLeftAlt]      = SDLK_LALT;
	keymap[EStdKeyRightAlt]     = SDLK_RALT;
	keymap[EStdKeyLeftCtrl]     = SDLK_LCTRL;
	keymap[EStdKeyRightCtrl]    = SDLK_RCTRL;
	keymap[EStdKeyLeftFunc]     = SDLK_LMETA;
	keymap[EStdKeyRightFunc]    = SDLK_RMETA;
	keymap[EStdKeyInsert]       = SDLK_INSERT;
	keymap[EStdKeyComma]        = SDLK_COMMA;
	keymap[EStdKeyFullStop]     = SDLK_PERIOD;
	keymap[EStdKeyForwardSlash] = SDLK_SLASH;
	keymap[EStdKeyBackSlash]    = SDLK_BACKSLASH;
	keymap[EStdKeySemiColon]    = SDLK_SEMICOLON;
	keymap[EStdKeySingleQuote]  = SDLK_QUOTE;
	keymap[EStdKeyHash]         = SDLK_HASH;
	keymap[EStdKeySquareBracketLeft]    = SDLK_LEFTBRACKET;
	keymap[EStdKeySquareBracketRight]   = SDLK_RIGHTBRACKET;
	keymap[EStdKeyMinus]        = SDLK_MINUS;
	keymap[EStdKeyEquals]       = SDLK_EQUALS;

   	keymap[EStdKeyF1]          = SDLK_F1;  /* chr + q */
   	keymap[EStdKeyF2]          = SDLK_F2;  /* chr + w */
   	keymap[EStdKeyF3]          = SDLK_F3;  /* chr + e */
   	keymap[EStdKeyF4]          = SDLK_F4;  /* chr + r */
   	keymap[EStdKeyF5]          = SDLK_F5;  /* chr + t */
   	keymap[EStdKeyF6]          = SDLK_F6;  /* chr + y */
   	keymap[EStdKeyF7]          = SDLK_F7;  /* chr + i */
   	keymap[EStdKeyF8]          = SDLK_F8;  /* chr + o */

   	keymap[EStdKeyF9]          = SDLK_F9;  /* chr + a */
   	keymap[EStdKeyF10]         = SDLK_F10; /* chr + s */
   	keymap[EStdKeyF11]         = SDLK_F11; /* chr + d */
   	keymap[EStdKeyF12]         = SDLK_F12; /* chr + f */

	#ifndef SYMBIAN_CRYSTAL 
	//!!7650 additions
    #ifdef __WINS__
   	keymap[EStdKeyXXX]         = SDLK_RETURN;	/* "fire" key */
	#else
   	keymap[EStdKeyDevice3]     = SDLK_RETURN;	/* "fire" key */
	#endif
   	keymap[EStdKeyNkpAsterisk] = SDLK_ASTERISK; 
   	keymap[EStdKeyYes]         = SDLK_HOME;		/* "call" key */
   	keymap[EStdKeyNo]		   = SDLK_END;		/* "end call" key */
   	keymap[EStdKeyDevice0]     = SDLK_SPACE;	/* right menu key */
   	keymap[EStdKeyDevice1]     = SDLK_ESCAPE;	/* left menu key */
   	keymap[EStdKeyDevice2]     = SDLK_POWER;	/* power key */
	#endif

 #ifdef SYMBIAN_CRYSTAL 
    keymap[EStdKeyMenu]        = SDLK_ESCAPE;   // menu key
    keymap[EStdKeyDevice6]     = SDLK_LEFT;     // Rocker (joystick) left
    keymap[EStdKeyDevice7]     = SDLK_RIGHT;    // Rocker (joystick) right
    keymap[EStdKeyDevice8]     = SDLK_UP;       // Rocker (joystick) up
    keymap[EStdKeyDevice9]     = SDLK_DOWN;     // Rocker (joystick) down
    keymap[EStdKeyLeftFunc]     = SDLK_LALT;    //chr?
	keymap[EStdKeyRightFunc]    = SDLK_RALT;
    keymap[EStdKeyDeviceA]      = SDLK_RETURN;	/* "fire" key */
#endif

    ///////////////////////////////////////////////////////////

    RFs fs;
    if(KErrNone == fs.Connect())
        {
        RArray<TInt> array;
        TRAPD(err, ReadL(fs, array));
        if(err == KErrNone && array.Count() > 0)
            {
            
            SDLKey temp[MAX_SCANCODE];
            Mem::Copy(temp, keymap, MAX_SCANCODE * sizeof(SDLKey));

            for(TInt k = 0; k < array.Count(); k+= 2)
                {
                const TInt oldval = array[k]; 
                const TInt newval = array[k + 1]; 
                if(oldval >=  0 && oldval < MAX_SCANCODE && newval >=  0 && newval < MAX_SCANCODE)
                    {
                    keymap[oldval] = temp[newval];
                    }
                }
            }
        array.Close();
        }

    fs.Close();
    ///////////////////////////////////////////////////////////

    /* !!TODO
	EStdKeyNumLock=0x1b,
	EStdKeyScrollLock=0x1c,

	EStdKeyNkpForwardSlash=0x84,
	EStdKeyNkpAsterisk=0x85,
	EStdKeyNkpMinus=0x86,
	EStdKeyNkpPlus=0x87,
	EStdKeyNkpEnter=0x88,
	EStdKeyNkp1=0x89,
	EStdKeyNkp2=0x8a,
	EStdKeyNkp3=0x8b,
	EStdKeyNkp4=0x8c,
	EStdKeyNkp5=0x8d,
	EStdKeyNkp6=0x8e,
	EStdKeyNkp7=0x8f,
	EStdKeyNkp8=0x90,
	EStdKeyNkp9=0x91,
	EStdKeyNkp0=0x92,
	EStdKeyNkpFullStop=0x93,
    EStdKeyMenu=0x94,
    EStdKeyBacklightOn=0x95,
    EStdKeyBacklightOff=0x96,
    EStdKeyBacklightToggle=0x97,
    EStdKeyIncContrast=0x98,
    EStdKeyDecContrast=0x99,
    EStdKeySliderDown=0x9a,
    EStdKeySliderUp=0x9b,
    EStdKeyDictaphonePlay=0x9c,
    EStdKeyDictaphoneStop=0x9d,
    EStdKeyDictaphoneRecord=0x9e,
    EStdKeyHelp=0x9f,
    EStdKeyOff=0xa0,
    EStdKeyDial=0xa1,
    EStdKeyIncVolume=0xa2,
    EStdKeyDecVolume=0xa3,
    EStdKeyDevice0=0xa4,
    EStdKeyDevice1=0xa5,
    EStdKeyDevice2=0xa6,
    EStdKeyDevice3=0xa7,
    EStdKeyDevice4=0xa8,
    EStdKeyDevice5=0xa9,
    EStdKeyDevice6=0xaa,
    EStdKeyDevice7=0xab,
    EStdKeyDevice8=0xac,
    EStdKeyDevice9=0xad,
    EStdKeyDeviceA=0xae,
    EStdKeyDeviceB=0xaf,
    EStdKeyDeviceC=0xb0,
    EStdKeyDeviceD=0xb1,
    EStdKeyDeviceE=0xb2,
    EStdKeyDeviceF=0xb3,
    EStdKeyApplication0=0xb4,
    EStdKeyApplication1=0xb5,
    EStdKeyApplication2=0xb6,
    EStdKeyApplication3=0xb7,
    EStdKeyApplication4=0xb8,
    EStdKeyApplication5=0xb9,
    EStdKeyApplication6=0xba,
    EStdKeyApplication7=0xbb,
    EStdKeyApplication8=0xbc,
    EStdKeyApplication9=0xbd,
    EStdKeyApplicationA=0xbe,
    EStdKeyApplicationB=0xbf,
    EStdKeyApplicationC=0xc0,
    EStdKeyApplicationD=0xc1,
    EStdKeyApplicationE=0xc2,
    EStdKeyApplicationF=0xc3,
    EStdKeyYes=0xc4,
    EStdKeyNo=0xc5,
    EStdKeyIncBrightness=0xc6,
    EStdKeyDecBrightness=0xc7, 
    EStdKeyCaseOpen=0xc8,
    EStdKeyCaseClose=0xc9
    */

}



static SDL_keysym *TranslateKey(_THIS, int scancode, SDL_keysym *keysym)
{
//    char debug[256];
    //SDL_TRACE1("SDL: TranslateKey, scancode=%d", scancode); //!!

	/* Set the keysym information */ 

	keysym->scancode = scancode;

    if ((scancode >= MAX_SCANCODE) && 
        ((scancode - ENonCharacterKeyBase + 0x0081) >= MAX_SCANCODE)) {
        SDL_SetError("Too big scancode");
        keysym->scancode = SDLK_UNKNOWN;
	    keysym->mod = KMOD_NONE; 
        return keysym;
    }

	keysym->mod = SDL_GetModState();

    /* Handle function keys: F1, F2, F3 ... */
    if (keysym->mod & KMOD_META) {
        if (scancode >= 'A' && scancode < ('A' + 24)) { /* first 32 alphabet keys */
            switch(scancode) {
                case 'Q': scancode = EStdKeyF1; break;
                case 'W': scancode = EStdKeyF2; break;
                case 'E': scancode = EStdKeyF3; break;
                case 'R': scancode = EStdKeyF4; break;
                case 'T': scancode = EStdKeyF5; break;
                case 'Y': scancode = EStdKeyF6; break;
                case 'U': scancode = EStdKeyF7; break;
                case 'I': scancode = EStdKeyF8; break;
                case 'A': scancode = EStdKeyF9; break;
                case 'S': scancode = EStdKeyF10; break;
                case 'D': scancode = EStdKeyF11; break;
                case 'F': scancode = EStdKeyF12; break;
            }
            keysym->sym = keymap[scancode];
        }
    }

    if (scancode >= ENonCharacterKeyBase) {
        // Non character keys
	    keysym->sym = keymap[scancode - 
            ENonCharacterKeyBase + 0x0081]; // !!hard coded
    } else {
	    keysym->sym = keymap[scancode];
    }

	/* Remap the arrow keys if the device is rotated */
	if (Private->EPOC_ScreenOrientation == CFbsBitGc::EGraphicsOrientationRotated270) {
		switch(keysym->sym) {
			case SDLK_UP:	keysym->sym = SDLK_LEFT;  break;
			case SDLK_DOWN: keysym->sym = SDLK_RIGHT; break;
			case SDLK_LEFT: keysym->sym = SDLK_DOWN;  break;
			case SDLK_RIGHT:keysym->sym = SDLK_UP;    break;
		}
	}

	/* If UNICODE is on, get the UNICODE value for the key */
	keysym->unicode = 0;

#if 0 // !!TODO:unicode

	if ( SDL_TranslateUNICODE ) 
    {
		/* Populate the unicode field with the ASCII value */
		keysym->unicode = scancode;
	}
#endif

    //!!
    //sprintf(debug, "SDL: TranslateKey: keysym->scancode=%d, keysym->sym=%d, keysym->mod=%d",
    //    keysym->scancode, keysym->sym, keysym->mod);
    //SDL_TRACE(debug); //!!

	return(keysym);
}

}; /* extern "C" */


