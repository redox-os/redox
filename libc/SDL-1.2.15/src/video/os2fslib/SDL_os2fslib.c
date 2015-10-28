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
*/
#include "SDL_config.h"

#define _ULS_CALLCONV_
#define CALLCONV _System
#include <unidef.h>                    // Unicode API
#include <uconv.h>                     // Unicode API (codepage conversion)

#include <process.h>
#include <time.h>

#include "SDL_video.h"
#include "SDL_mouse.h"
#include "../SDL_sysvideo.h"
#include "../SDL_pixels_c.h"
#include "../../events/SDL_events_c.h"

#include "SDL_os2fslib.h"

static ULONG ulFCFToUse =
        FCF_TITLEBAR |
        FCF_SYSMENU |
        FCF_MINBUTTON |
        FCF_MAXBUTTON |
        FCF_NOBYTEALIGN |
        FCF_SIZEBORDER |
        FCF_TASKLIST;

static int bMouseCaptured   = 0;
static int bMouseCapturable = 0;
static HPOINTER hptrGlobalPointer = NULL;
static HPOINTER hptrCurrentIcon = NULL;
static int iWindowSizeX = 320;
static int iWindowSizeY = 200;
static int bWindowResized = 0;

#pragma pack(1)
typedef struct BMPINFO
{
   BITMAPINFO;
   RGB  clr;
} BMPINFO, *PBMPINFO;
#pragma pack()


// Backdoors:
DECLSPEC void SDLCALL SDL_OS2FSLIB_SetFCFToUse(ULONG ulFCF)
{
  ulFCFToUse = ulFCF;
}

// Configuration defines:

// We have to report empty alpha mask, otherwise SDL will select
// alpha blitters, and this will have unwanted results, as we don't
// support alpha channel in FSLib yet.
#define REPORT_EMPTY_ALPHA_MASK

// Experimental: Move every FSLib_BitBlt() call into window message
// processing function.
// This may fix dirt left on desktop. Or not.
//#define BITBLT_IN_WINMESSAGEPROC

// Experimental-2: Use WinLockWindowUpdate() in around bitblts!
// This is not enabled, because it seems to cause more problems
// than good.
//#define USE_WINLOCKWINDOWUPDATE_AROUND_BITBLTS

// Use the following to show resized image instead of black stuff
// even if the surface is resizable.
//#define RESIZE_EVEN_IF_RESIZABLE

/* The translation table from a VK keysym to a SDL keysym */
static SDLKey HWScanKeyMap[256];
static SDL_keysym *TranslateKey(int vkey, int chcode, int scancode, SDL_keysym *keysym, int iPressed);
static int  iShiftIsPressed;

#ifdef BITBLT_IN_WINMESSAGEPROC
#define WM_UPDATERECTSREQUEST   WM_USER+50
#endif

#ifdef USE_WINLOCKWINDOWUPDATE_AROUND_BITBLTS
#define FSLIB_BITBLT(hwnd, buffer, top, left, width, height) \
    { \
      WinLockWindowUpdate(HWND_DESKTOP, HWND_DESKTOP); \
      FSLib_BitBlt(hwnd, buffer, top, left, width, height); \
      WinLockWindowUpdate(HWND_DESKTOP, NULL); \
    }
#else
#define FSLIB_BITBLT(hwnd, buffer, top, left, width, height) \
    FSLib_BitBlt(hwnd, buffer, top, left, width, height);
#endif

/////////////////////////////////////////////////////////////////////
//
// SetAccessableWindowPos
//
// Same as WinSetWindowPos(), but takes care for the window to be
// always on the screen, the titlebar will be accessable everytime.
//
/////////////////////////////////////////////////////////////////////
static BOOL SetAccessableWindowPos(HWND hwnd, HWND hwndInsertBehind,
                                   LONG x, LONG y,
                                   LONG cx, LONG cy,
                                   ULONG fl)
{
  SWP swpDesktop, swp;
  // Get desktop area
  WinQueryWindowPos(HWND_DESKTOP, &swpDesktop);

  if ((fl & SWP_MOVE) && (fl & SWP_SIZE))
  {
    // If both moving and sizing, then change size and pos now!!
    if (x+cx>swpDesktop.cx)
      x = swpDesktop.cx - cx;
    if (x<0)
      x = 0;
    if (y<0)
      y = 0;
    if (y+cy>swpDesktop.cy)
      y = swpDesktop.cy - cy;
    return WinSetWindowPos(hwnd, hwndInsertBehind, x, y, cx, cy, fl);
  } else
  if (fl & SWP_MOVE)
  {
    // Just moving
    WinQueryWindowPos(hwnd, &swp);
    if (x+swp.cx>swpDesktop.cx)
      x = swpDesktop.cx - swp.cx;
    if (x<0)
      x = 0;
    if (y<0)
      y = 0;
    if (y+swp.cy>swpDesktop.cy)
      y = swpDesktop.cy - swp.cy;
    return WinSetWindowPos(hwnd, hwndInsertBehind, x, y, cx, cy, fl);
  } else
  if (fl & SWP_SIZE)
  {
    // Just sizing
    WinQueryWindowPos(hwnd, &swp);
    x = swp.x;
    y = swp.y;
    if (x+cx>swpDesktop.cx)
      x = swpDesktop.cx - cx;
    if (x<0)
      x = 0;
    if (y<0)
      y = 0;
    if (y+cy>swpDesktop.cy)
      y = swpDesktop.cy - cy;
    return WinSetWindowPos(hwnd, hwndInsertBehind, x, y, cx, cy, fl | SWP_MOVE);
  } else
  return WinSetWindowPos(hwnd, hwndInsertBehind, x, y, cx, cy, fl);
}

static UniChar NativeCharToUniChar(int chcode)
{
  UniChar ucResult = (UniChar) chcode;
  int rc;
  UconvObject ucoTemp;
  char     achFrom[2];
  char     *pchFrom;
  size_t   iFromCount;
  UniChar  aucTo[10];
  UniChar  *pucTo;
  size_t   iToCount;
  size_t   iNonIdentical;

  // Create unicode convert object
  rc = UniCreateUconvObject(L"", &ucoTemp);
  if (rc!=ULS_SUCCESS)
  {
    // Could not create convert object!
    return ucResult;
  }

  // Convert language code string to unicode string
  achFrom[0] = (char) chcode;
  achFrom[1] = 0;
  iFromCount = sizeof(char) * 2;
  iToCount = sizeof(UniChar) * 2;
  pucTo = &(aucTo[0]);
  pchFrom = &(achFrom[0]);

  rc = UniUconvToUcs(ucoTemp,
                     &pchFrom,
                     &iFromCount,
                     &pucTo,
                     &iToCount,
                     &iNonIdentical);

  if (rc!=ULS_SUCCESS)
  {
    // Could not convert language code to UCS string!
    UniFreeUconvObject(ucoTemp);
    return ucResult;
  }

  UniFreeUconvObject(ucoTemp);

#ifdef DEBUG_BUILD
  printf("%02x converted to %02x\n", (int) chcode, (int) (aucTo[0]));
#endif

  return aucTo[0];
}

/////////////////////////////////////////////////////////////////////
//
// TranslateKey
//
// This creates SDL Keycodes from VK_ and hardware scan codes
//
/////////////////////////////////////////////////////////////////////
static SDL_keysym *TranslateKey(int vkey, int chcode, int scancode, SDL_keysym *keysym, int iPressed)
{
  keysym->scancode = (unsigned char) scancode;
  keysym->mod = KMOD_NONE;
  keysym->unicode = 0;

  if (iPressed && SDL_TranslateUNICODE)
  {
    if (chcode)
      keysym->unicode = NativeCharToUniChar(chcode);
    else
      keysym->unicode = vkey;
  }

  keysym->sym = HWScanKeyMap[scancode];

  // Now stuffs based on state of shift key(s)!
  if (vkey == VK_SHIFT)
  {
    iShiftIsPressed = iPressed;
  }

  if ((iShiftIsPressed) && (SDL_TranslateUNICODE))
  {
    // Change syms, if Unicode stuff is required
    // I think it's silly, but it's SDL...
    switch (keysym->sym)
    {
      case SDLK_BACKQUOTE:
        keysym->sym = '~';
        break;
      case SDLK_1:
        keysym->sym = SDLK_EXCLAIM;
        break;
      case SDLK_2:
        keysym->sym = SDLK_AT;
        break;
      case SDLK_3:
        keysym->sym = SDLK_HASH;
        break;
      case SDLK_4:
        keysym->sym = SDLK_DOLLAR;
        break;
      case SDLK_5:
        keysym->sym = '%';
        break;
      case SDLK_6:
        keysym->sym = SDLK_CARET;
        break;
      case SDLK_7:
        keysym->sym = SDLK_AMPERSAND;
        break;
      case SDLK_8:
        keysym->sym = SDLK_ASTERISK;
        break;
      case SDLK_9:
        keysym->sym = SDLK_LEFTPAREN;
        break;
      case SDLK_0:
        keysym->sym = SDLK_RIGHTPAREN;
        break;
      case SDLK_MINUS:
        keysym->sym = SDLK_UNDERSCORE;
        break;
      case SDLK_PLUS:
        keysym->sym = SDLK_EQUALS;
        break;

      case SDLK_LEFTBRACKET:
        keysym->sym = '{';
        break;
      case SDLK_RIGHTBRACKET:
        keysym->sym = '}';
        break;

      case SDLK_SEMICOLON:
        keysym->sym = SDLK_COLON;
        break;
      case SDLK_QUOTE:
        keysym->sym = SDLK_QUOTEDBL;
        break;
      case SDLK_BACKSLASH:
        keysym->sym = '|';
        break;

      case SDLK_COMMA:
        keysym->sym = SDLK_LESS;
        break;
      case SDLK_PERIOD:
        keysym->sym = SDLK_GREATER;
        break;
      case SDLK_SLASH:
        keysym->sym = SDLK_QUESTION;
        break;

      default:
        break;
    }
  }
  return keysym;
}

#define CONVERTMOUSEPOSITION()  \
        /* We have to inverse the mouse position, because every non-os/2 system */                                                \
        /* has a coordinate system where the (0;0) is the top-left corner,      */                                                \
        /* while on os/2 it's the bottom left corner!                           */                                                \
        if (FSLib_QueryFSMode(hwnd))                                                                                              \
        {                                                                                                                         \
          /* We're in FS mode!                                                        */                                          \
          /* In FS mode our window is as big as fullscreen mode, but not necessary as */                                          \
          /* big as the source buffer (can be bigger)                                 */                                          \
          /* So, limit mouse pos to source buffer size!                               */                                          \
          if (ppts->x<0) ppts->x = 0;                                                                                             \
          if (ppts->y<0) ppts->y = 0;                                                                                             \
          if (ppts->x>=pVideo->hidden->SrcBufferDesc.uiXResolution) ppts->x = pVideo->hidden->SrcBufferDesc.uiXResolution-1;      \
          if (ppts->y>=pVideo->hidden->SrcBufferDesc.uiYResolution) ppts->y = pVideo->hidden->SrcBufferDesc.uiYResolution-1;      \
          pVideo->hidden->iSkipWMMOUSEMOVE++; /* Don't take next WM_MOUSEMOVE into account!  */                                   \
          ptl.x = ppts->x; ptl.y = ppts->y;                                                                                       \
          WinMapWindowPoints(pVideo->hidden->hwndClient, HWND_DESKTOP, &ptl, 1);                                                  \
          WinSetPointerPos(HWND_DESKTOP, ptl.x, ptl.y);                                                                           \
          /* Then convert OS/2 position to SDL position */                                                                        \
          ppts->y = pVideo->hidden->SrcBufferDesc.uiYResolution - ppts->y - 1;                                                    \
        } else                                                                                                                    \
        {                                                                                                                         \
          SWP swpClient;                                                                                                          \
          /* We're in windowed mode! */                                                                                           \
          WinQueryWindowPos(pVideo->hidden->hwndClient, &swpClient);                                                              \
          /* Convert OS/2 mouse position to SDL position, and also scale it! */                                                   \
          (ppts->x) = (ppts->x) * pVideo->hidden->SrcBufferDesc.uiXResolution / swpClient.cx;                                       \
          (ppts->y) = (ppts->y) * pVideo->hidden->SrcBufferDesc.uiYResolution / swpClient.cy;                                       \
          (ppts->y) = pVideo->hidden->SrcBufferDesc.uiYResolution - (ppts->y)  - 1;                                                 \
        }



/////////////////////////////////////////////////////////////////////
//
// WndProc
//
// This is the message processing window procedure for the
// SDLWindowClass, which is the client window in our application.
// It handles switching back and away from the app (taking care of
// going out and back to and from fullscreen mode), sending keystrokes
// and mouse events to where it has to be sent, etc...
//
/////////////////////////////////////////////////////////////////////
static MRESULT EXPENTRY WndProc( HWND hwnd, ULONG msg, MPARAM mp1, MPARAM mp2)
{
  HPS ps;
  RECTL rcl;
  SDL_VideoDevice *pVideo = NULL;

  switch (msg)
  {
    case WM_CHAR:  // Keypress notification
#ifdef DEBUG_BUILD
//      printf("WM_CHAR\n"); fflush(stdout);
#endif
      pVideo = WinQueryWindowPtr(hwnd, 0);
      if (pVideo)
      {
        /*
        // We skip repeated keys:
        if (CHARMSG(&msg)->cRepeat>1)
        {
#ifdef DEBUG_BUILD
//          printf("Repeated key (%d), skipping...\n", CHARMSG(&msg)->cRepeat); fflush(stdout);
#endif
          return (MRESULT) TRUE;
        }
        */

        // If it's not repeated, then let's see if its pressed or released!
        if (SHORT1FROMMP(mp1) & KC_KEYUP)
        {
          // A key has been released
          SDL_keysym keysym;

#ifdef DEBUG_BUILD
//          printf("WM_CHAR, keyup, code is [0x%0x]\n", CHAR4FROMMP(mp1)); // HW scan code
#endif

          // One problem is with F1, which gets only the keyup message because
          // it is a system key.
          // So, when we get keyup message, we simulate keydown too!
          // UPDATE:
          //  This problem should be solved now, that the accelerator keys are
          //  disabled for this window!
          /*
          if (SHORT2FROMMP(mp2)==VK_F1)
          {
            SDL_PrivateKeyboard(SDL_PRESSED, TranslateKey(SHORT2FROMMP(mp2), // VK_ code
                                                           SHORT1FROMMP(mp2), // Character code
                                                           CHAR4FROMMP(mp1),  // HW Scan code
                                                           &keysym,0));
          }*/

          SDL_PrivateKeyboard(SDL_RELEASED, TranslateKey(SHORT2FROMMP(mp2), // VK_ code
                                                         SHORT1FROMMP(mp2), // Character code
                                                         CHAR4FROMMP(mp1),  // HW Scan code
                                                         &keysym,0));
          
        } else
        {
          // A key has been pressed
          SDL_keysym keysym;

#ifdef DEBUG_BUILD
//          printf("WM_CHAR, keydown, code is [0x%0x]\n", CHAR4FROMMP(mp1)); // HW scan code
#endif
          // Check for fastkeys: ALT+HOME to toggle FS mode
          //                     ALT+END to close app
          if ((SHORT1FROMMP(mp1) & KC_ALT) &&
              (SHORT2FROMMP(mp2) == VK_HOME))
          {
#ifdef DEBUG_BUILD
            printf(" Pressed ALT+HOME!\n"); fflush(stdout);
#endif
            // Only switch between fullscreen and back if it's not
            // a resizable mode!
            if (
                (!pVideo->hidden->pSDLSurface) ||
                ((pVideo->hidden->pSDLSurface)
                 && ((pVideo->hidden->pSDLSurface->flags & SDL_RESIZABLE)==0)
                )
               )
              FSLib_ToggleFSMode(hwnd, !FSLib_QueryFSMode(hwnd));
#ifdef DEBUG_BUILD
            else
              printf(" Resizable mode, so discarding ALT+HOME!\n"); fflush(stdout);
#endif
          } else
          if ((SHORT1FROMMP(mp1) & KC_ALT) &&
              (SHORT2FROMMP(mp2) == VK_END))
          {
#ifdef DEBUG_BUILD
            printf(" Pressed ALT+END!\n"); fflush(stdout);
#endif
            // Close window, and get out of loop!
            // Also send event to SDL application, but we won't
            // wait for it to be processed!
            SDL_PrivateQuit();
            WinPostMsg(hwnd, WM_QUIT, 0, 0);
          } else
          {
            
            SDL_PrivateKeyboard(SDL_PRESSED, TranslateKey(SHORT2FROMMP(mp2), // VK_ code
                                                          SHORT1FROMMP(mp2), // Character code
                                                          CHAR4FROMMP(mp1),  // HW Scan code
                                                          &keysym,1));
            
          }
        }
      }
      return (MRESULT) TRUE;

    case WM_TRANSLATEACCEL:
      {
        PQMSG pqmsg;
        pqmsg = (PQMSG) mp1;
        if (mp1)
        {
          if (pqmsg->msg == WM_CHAR)
          {
            // WM_CHAR message!
            // Let's filter the ALT keypress and all other acceleration keys!
            return (MRESULT) FALSE;
          }
        }
        break; // Default processing (pass to parent until frame control)
      }

    case WM_PAINT:  // Window redraw!
#ifdef DEBUG_BUILD
      printf("WM_PAINT (0x%x)\n", hwnd); fflush(stdout);
#endif
      ps = WinBeginPaint(hwnd,0,&rcl);
      pVideo = FSLib_GetUserParm(hwnd);
      if (pVideo)
      {
        if (!pVideo->hidden->pSDLSurface)
        {
          RECTL rclRect;
          // So, don't blit now!
#ifdef DEBUG_BUILD
          printf("WM_PAINT : Skipping blit while resizing (Pre!)!\n"); fflush(stdout);
#endif
          WinQueryWindowRect(hwnd, &rclRect);
          // Fill with black
          WinFillRect(ps, &rclRect, CLR_BLACK);
        } else
        {
          if (DosRequestMutexSem(pVideo->hidden->hmtxUseSrcBuffer, 1000)==NO_ERROR)
          {
            int iTop, iLeft, iWidth, iHeight;
            int iXScaleError, iYScaleError;
            int iXScaleError2, iYScaleError2;
            SWP swp;
            
            // Re-blit the modified area!
            // For this, we have to calculate the points, scaled!
            WinQueryWindowPos(hwnd, &swp);
#ifdef DEBUG_BUILD
            printf("WM_PAINT : WinSize: %d %d, BufSize: %d %d\n",
                   swp.cx,
                   swp.cy,
                   pVideo->hidden->SrcBufferDesc.uiXResolution,
                   pVideo->hidden->SrcBufferDesc.uiYResolution
                  );
            fflush(stdout);
#endif

#ifndef RESIZE_EVEN_IF_RESIZABLE
            // But only blit if the window is not resizable, or if
            // the window is resizable and the source buffer size is the
            // same as the destination buffer size!
            if ((!pVideo->hidden->pSDLSurface) ||
                ((pVideo->hidden->pSDLSurface) &&
                 (pVideo->hidden->pSDLSurface->flags & SDL_RESIZABLE) &&
                 ((swp.cx != pVideo->hidden->SrcBufferDesc.uiXResolution) ||
                  (swp.cy != pVideo->hidden->SrcBufferDesc.uiYResolution)
                 ) &&
                 (!FSLib_QueryFSMode(hwnd))
                )
               )
            {
              RECTL rclRect;
              // Resizable surface and in resizing!
              // So, don't blit now!
#ifdef DEBUG_BUILD
              printf("WM_PAINT : Skipping blit while resizing!\n"); fflush(stdout);
#endif
              WinQueryWindowRect(hwnd, &rclRect);
              // Fill with black
              WinFillRect(ps, &rclRect, CLR_BLACK);
            } else
#endif
            {
  
              iXScaleError = (pVideo->hidden->SrcBufferDesc.uiXResolution-1) / swp.cx;
              iYScaleError = (pVideo->hidden->SrcBufferDesc.uiYResolution-1) / swp.cy;
              if (iXScaleError<0) iXScaleError = 0;
              if (iYScaleError<0) iYScaleError = 0;
              iXScaleError2 = (swp.cx-1)/(pVideo->hidden->SrcBufferDesc.uiXResolution);
              iYScaleError2 = (swp.cy-1)/(pVideo->hidden->SrcBufferDesc.uiYResolution);
              if (iXScaleError2<0) iXScaleError2 = 0;
              if (iYScaleError2<0) iYScaleError2 = 0;
      
              iTop = (swp.cy - rcl.yTop) * pVideo->hidden->SrcBufferDesc.uiYResolution / swp.cy - iYScaleError;
              iLeft = rcl.xLeft * pVideo->hidden->SrcBufferDesc.uiXResolution / swp.cx - iXScaleError;
              iWidth = ((rcl.xRight-rcl.xLeft) * pVideo->hidden->SrcBufferDesc.uiXResolution + swp.cx-1)
                / swp.cx + 2*iXScaleError;
              iHeight = ((rcl.yTop-rcl.yBottom) * pVideo->hidden->SrcBufferDesc.uiYResolution + swp.cy-1)
                / swp.cy + 2*iYScaleError;
      
              iWidth+=iXScaleError2;
              iHeight+=iYScaleError2;
      
              if (iTop<0) iTop = 0;
              if (iLeft<0) iLeft = 0;
              if (iTop+iHeight>pVideo->hidden->SrcBufferDesc.uiYResolution) iHeight = pVideo->hidden->SrcBufferDesc.uiYResolution-iTop;
              if (iLeft+iWidth>pVideo->hidden->SrcBufferDesc.uiXResolution) iWidth = pVideo->hidden->SrcBufferDesc.uiXResolution-iLeft;
    
#ifdef DEBUG_BUILD
              printf("WM_PAINT : BitBlt: %d %d -> %d %d (Buf %d x %d)\n",
                     iTop, iLeft, iWidth, iHeight,
                     pVideo->hidden->SrcBufferDesc.uiXResolution,
                     pVideo->hidden->SrcBufferDesc.uiYResolution
                    );
              fflush(stdout);
#endif
                    
              FSLIB_BITBLT(hwnd, pVideo->hidden->pchSrcBuffer, iTop, iLeft, iWidth, iHeight);
            }
  
            DosReleaseMutexSem(pVideo->hidden->hmtxUseSrcBuffer);
          }
        }
      }
#ifdef DEBUG_BUILD
      else
      {
        printf("WM_PAINT : No pVideo!\n"); fflush(stdout);
      }
#endif
      WinEndPaint(ps);
#ifdef DEBUG_BUILD
      printf("WM_PAINT : Done.\n");
      fflush(stdout);
#endif
      return 0;

    case WM_SIZE:
      {
#ifdef DEBUG_BUILD
        printf("WM_SIZE : (%d %d)\n",
               SHORT1FROMMP(mp2), SHORT2FROMMP(mp2)); fflush(stdout);
#endif
        iWindowSizeX = SHORT1FROMMP(mp2);
        iWindowSizeY = SHORT2FROMMP(mp2);
        bWindowResized = 1;

        // Make sure the window will be redrawn
        WinInvalidateRegion(hwnd, NULL, TRUE);
      }
      break;

    case WM_FSLIBNOTIFICATION:
#ifdef DEBUG_BUILD
        printf("WM_FSLIBNOTIFICATION\n"); fflush(stdout);
#endif
      if ((int)mp1 == FSLN_TOGGLEFSMODE)
      {
        // FS mode changed, reblit image!
        pVideo = FSLib_GetUserParm(hwnd);
        if (pVideo)
        {
          if (!pVideo->hidden->pSDLSurface)
          {
            // Resizable surface and in resizing!
            // So, don't blit now!
#ifdef DEBUG_BUILD
            printf("WM_FSLIBNOTIFICATION : Can not blit if there is no surface, doing nothing.\n"); fflush(stdout);
#endif
          } else
          {
            if (DosRequestMutexSem(pVideo->hidden->hmtxUseSrcBuffer, 1000)==NO_ERROR)
            {
              if (pVideo->hidden->pSDLSurface)
              {
#ifndef RESIZE_EVEN_IF_RESIZABLE
                SWP swp;

                // But only blit if the window is not resizable, or if
                // the window is resizable and the source buffer size is the
                // same as the destination buffer size!
                WinQueryWindowPos(hwnd, &swp);
                if ((!pVideo->hidden->pSDLSurface) ||
                    (
                     (pVideo->hidden->pSDLSurface) &&
                     (pVideo->hidden->pSDLSurface->flags & SDL_RESIZABLE) &&
                     ((swp.cx != pVideo->hidden->SrcBufferDesc.uiXResolution) ||
                      (swp.cy != pVideo->hidden->SrcBufferDesc.uiYResolution)
                     ) &&
                     (!FSLib_QueryFSMode(hwnd))
                    )
                   )
                {
                  // Resizable surface and in resizing!
                  // So, don't blit now!
#ifdef DEBUG_BUILD
                  printf("WM_FSLIBNOTIFICATION : Cannot blit while resizing, doing nothing.\n"); fflush(stdout);
#endif
                } else
#endif
                {
#ifdef DEBUG_BUILD
                  printf("WM_FSLIBNOTIFICATION : Blitting!\n"); fflush(stdout);
#endif
                  FSLIB_BITBLT(hwnd, pVideo->hidden->pchSrcBuffer,
                               0, 0,
                               pVideo->hidden->SrcBufferDesc.uiXResolution,
                               pVideo->hidden->SrcBufferDesc.uiYResolution);
                }
              }
#ifdef DEBUG_BUILD
              else
                printf("WM_FSLIBNOTIFICATION : No public surface!\n"); fflush(stdout);
#endif
  
              DosReleaseMutexSem(pVideo->hidden->hmtxUseSrcBuffer);
            }
          }
        }
      }
      return (MPARAM) 1;

    case WM_ACTIVATE:
#ifdef DEBUG_BUILD
      printf("WM_ACTIVATE\n"); fflush(stdout);
#endif

      pVideo = FSLib_GetUserParm(hwnd);
      if (pVideo)
      {
        pVideo->hidden->fInFocus = (int) mp1;
        if (pVideo->hidden->fInFocus)
        {
          // Went into focus
          if ((pVideo->hidden->iMouseVisible) && (!bMouseCaptured))
            WinSetPointer(HWND_DESKTOP, WinQuerySysPointer(HWND_DESKTOP, SPTR_ARROW, FALSE));
          else
            WinSetPointer(HWND_DESKTOP, NULL);

          if (bMouseCapturable)
          {
            // Re-capture the mouse, if we captured it before!
            WinSetCapture(HWND_DESKTOP, hwnd);
            bMouseCaptured = 1;
            {
              SWP swpClient;
              POINTL ptl;
              // Center the mouse to the middle of the window!
              WinQueryWindowPos(pVideo->hidden->hwndClient, &swpClient);
              ptl.x = 0; ptl.y = 0;
              WinMapWindowPoints(pVideo->hidden->hwndClient, HWND_DESKTOP, &ptl, 1);
              pVideo->hidden->iSkipWMMOUSEMOVE++; /* Don't take next WM_MOUSEMOVE into account!  */
              WinSetPointerPos(HWND_DESKTOP,
                               ptl.x + swpClient.cx/2,
                               ptl.y + swpClient.cy/2);
            }
          }
        } else
        {
          // Went out of focus
          WinSetPointer(HWND_DESKTOP, WinQuerySysPointer(HWND_DESKTOP, SPTR_ARROW, FALSE));

          if (bMouseCaptured)
          {
            // Release the mouse
            WinSetCapture(HWND_DESKTOP, hwnd);
            bMouseCaptured = 0;
          }
        }
      }
#ifdef DEBUG_BUILD
      printf("WM_ACTIVATE done\n"); fflush(stdout);
#endif

      break;

    case WM_BUTTON1DOWN:
#ifdef DEBUG_BUILD
      printf("WM_BUTTON1DOWN\n"); fflush(stdout);
#endif

      pVideo = FSLib_GetUserParm(hwnd);
      if (pVideo)
      {
        SDL_PrivateMouseButton(SDL_PRESSED,
                               SDL_BUTTON_LEFT,
                               0, 0); // Don't report mouse movement!

        if (bMouseCapturable)
        {
          // We should capture the mouse!
          if (!bMouseCaptured)
          {
            WinSetCapture(HWND_DESKTOP, hwnd);
            WinSetPointer(HWND_DESKTOP, NULL);
            bMouseCaptured = 1;
            {
              SWP swpClient;
              POINTL ptl;
              // Center the mouse to the middle of the window!
              WinQueryWindowPos(pVideo->hidden->hwndClient, &swpClient);
              ptl.x = 0; ptl.y = 0;
              WinMapWindowPoints(pVideo->hidden->hwndClient, HWND_DESKTOP, &ptl, 1);
              pVideo->hidden->iSkipWMMOUSEMOVE++; /* Don't take next WM_MOUSEMOVE into account!  */
              WinSetPointerPos(HWND_DESKTOP,
                               ptl.x + swpClient.cx/2,
                               ptl.y + swpClient.cy/2);
            }
          }
        }
      }
      break;
    case WM_BUTTON1UP:
#ifdef DEBUG_BUILD
      printf("WM_BUTTON1UP\n"); fflush(stdout);
#endif
      SDL_PrivateMouseButton(SDL_RELEASED,
                             SDL_BUTTON_LEFT,
                             0, 0); // Don't report mouse movement!
      break;
    case WM_BUTTON2DOWN:
#ifdef DEBUG_BUILD
      printf("WM_BUTTON2DOWN\n"); fflush(stdout);
#endif

      pVideo = FSLib_GetUserParm(hwnd);
      if (pVideo)
      {
        SDL_PrivateMouseButton(SDL_PRESSED,
                               SDL_BUTTON_RIGHT,
                               0, 0); // Don't report mouse movement!

        if (bMouseCapturable)
        {
          // We should capture the mouse!
          if (!bMouseCaptured)
          {
            WinSetCapture(HWND_DESKTOP, hwnd);
            WinSetPointer(HWND_DESKTOP, NULL);
            bMouseCaptured = 1;
            {
              SWP swpClient;
              POINTL ptl;
              // Center the mouse to the middle of the window!
              WinQueryWindowPos(pVideo->hidden->hwndClient, &swpClient);
              ptl.x = 0; ptl.y = 0;
              WinMapWindowPoints(pVideo->hidden->hwndClient, HWND_DESKTOP, &ptl, 1);
              pVideo->hidden->iSkipWMMOUSEMOVE++; /* Don't take next WM_MOUSEMOVE into account!  */
              WinSetPointerPos(HWND_DESKTOP,
                               ptl.x + swpClient.cx/2,
                               ptl.y + swpClient.cy/2);
            }
          }
        }

      }
      break;
    case WM_BUTTON2UP:
#ifdef DEBUG_BUILD
      printf("WM_BUTTON2UP\n"); fflush(stdout);
#endif
      SDL_PrivateMouseButton(SDL_RELEASED,
                             SDL_BUTTON_RIGHT,
                             0, 0); // Don't report mouse movement!
      break;
    case WM_BUTTON3DOWN:
#ifdef DEBUG_BUILD
      printf("WM_BUTTON3DOWN\n"); fflush(stdout);
#endif

      pVideo = FSLib_GetUserParm(hwnd);
      if (pVideo)
      {
        SDL_PrivateMouseButton(SDL_PRESSED,
                               SDL_BUTTON_MIDDLE,
                               0, 0); // Don't report mouse movement!
        
        if (bMouseCapturable)
        {
          // We should capture the mouse!
          if (!bMouseCaptured)
          {
            WinSetCapture(HWND_DESKTOP, hwnd);
            WinSetPointer(HWND_DESKTOP, NULL);
            bMouseCaptured = 1;
            {
              SWP swpClient;
              POINTL ptl;
              // Center the mouse to the middle of the window!
              WinQueryWindowPos(pVideo->hidden->hwndClient, &swpClient);
              ptl.x = 0; ptl.y = 0;
              WinMapWindowPoints(pVideo->hidden->hwndClient, HWND_DESKTOP, &ptl, 1);
              pVideo->hidden->iSkipWMMOUSEMOVE++; /* Don't take next WM_MOUSEMOVE into account!  */
              WinSetPointerPos(HWND_DESKTOP,
                               ptl.x + swpClient.cx/2,
                               ptl.y + swpClient.cy/2);
            }
          }
        }
      }
      break;
    case WM_BUTTON3UP:
#ifdef DEBUG_BUILD
      printf("WM_BUTTON3UP\n"); fflush(stdout);
#endif
      SDL_PrivateMouseButton(SDL_RELEASED,
                             SDL_BUTTON_MIDDLE,
                             0, 0); // Don't report mouse movement!
      break;
    case WM_MOUSEMOVE:
#ifdef DEBUG_BUILD
//      printf("WM_MOUSEMOVE\n"); fflush(stdout);
#endif

      pVideo = FSLib_GetUserParm(hwnd);
      if (pVideo)
      {
        if (pVideo->hidden->iSkipWMMOUSEMOVE)
        {
          pVideo->hidden->iSkipWMMOUSEMOVE--;
        } else
        {
          POINTS *ppts = (POINTS *) (&mp1);
          POINTL ptl;

          if (bMouseCaptured)
          {
            SWP swpClient;

            WinQueryWindowPos(pVideo->hidden->hwndClient, &swpClient);

            // Send relative mouse position, and re-center the mouse
            // Reposition the mouse to the center of the screen/window
            SDL_PrivateMouseMotion(0, // Buttons not changed
                                   1, // Relative position
                                   ppts->x - (swpClient.cx/2),
                                   (swpClient.cy/2) - ppts->y);

            ptl.x = 0; ptl.y = 0;
            WinMapWindowPoints(pVideo->hidden->hwndClient, HWND_DESKTOP, &ptl, 1);
            pVideo->hidden->iSkipWMMOUSEMOVE++; /* Don't take next WM_MOUSEMOVE into account!  */
            // Center the mouse to the middle of the window!
            WinSetPointerPos(HWND_DESKTOP,
                             ptl.x + swpClient.cx/2,
                             ptl.y + swpClient.cy/2);
          } else
          {
            CONVERTMOUSEPOSITION();

            // Send absolute mouse position
            SDL_PrivateMouseMotion(0, // Buttons not changed
                                   0, // Absolute position
                                   ppts->x,
                                   ppts->y);
          }
        }
        if ((pVideo->hidden->iMouseVisible) && (!bMouseCaptured))
        {
#ifdef DEBUG_BUILD
//          printf("WM_MOUSEMOVE : ptr = %p\n", hptrGlobalPointer); fflush(stdout);
#endif

          if (hptrGlobalPointer)
            WinSetPointer(HWND_DESKTOP, hptrGlobalPointer);
          else
            WinSetPointer(HWND_DESKTOP, WinQuerySysPointer(HWND_DESKTOP, SPTR_ARROW, FALSE));
        }
        else
        {
          WinSetPointer(HWND_DESKTOP, NULL);
        }
      }
#ifdef DEBUG_BUILD
//      printf("WM_MOUSEMOVE done\n"); fflush(stdout);
#endif

      return (MRESULT) FALSE;
    case WM_CLOSE: // Window close
#ifdef DEBUG_BUILD
      printf("WM_CLOSE\n"); fflush(stdout);
#endif

      pVideo = FSLib_GetUserParm(hwnd);
      if (pVideo)
      {
        // Send Quit message to the SDL application!
        SDL_PrivateQuit();
        return 0;
      }
      break;

#ifdef BITBLT_IN_WINMESSAGEPROC
    case WM_UPDATERECTSREQUEST:
      pVideo = FSLib_GetUserParm(hwnd);
      if ((pVideo) && (pVideo->hidden->pSDLSurface))
      {
        if (DosRequestMutexSem(pVideo->hidden->hmtxUseSrcBuffer, SEM_INDEFINITE_WAIT)==NO_ERROR)
        {
          int numrects;
          SDL_Rect *rects;
          int i;
          SWP swp;

          numrects = (int) mp1;
          rects = (SDL_Rect *) mp2;

          WinQueryWindowPos(hwnd, &swp);
#ifndef RESIZE_EVEN_IF_RESIZABLE
          if ((!pVideo->hidden->pSDLSurface) ||
              (
               (pVideo->hidden->pSDLSurface) &&
               (pVideo->hidden->pSDLSurface->flags & SDL_RESIZABLE) &&
               ((swp.cx != pVideo->hidden->SrcBufferDesc.uiXResolution) ||
                (swp.cy != pVideo->hidden->SrcBufferDesc.uiYResolution)
               ) &&
               (!FSLib_QueryFSMode(hwnd))
              )
             )
          {
            // Resizable surface and in resizing!
            // So, don't blit now!
#ifdef DEBUG_BUILD
            printf("[WM_UPDATERECTSREQUEST] : Skipping blit while resizing!\n"); fflush(stdout);
#endif
          } else
#endif
          {
#ifdef DEBUG_BUILD
            printf("[WM_UPDATERECTSREQUEST] : Blitting!\n"); fflush(stdout);
#endif
          
            // Blit the changed areas
            for (i=0; i<numrects; i++)
              FSLIB_BITBLT(hwnd, pVideo->hidden->pchSrcBuffer,
                           rects[i].y, rects[i].x, rects[i].w, rects[i].h);
          }
          DosReleaseMutexSem(pVideo->hidden->hmtxUseSrcBuffer);
        }
      }
      return 0;
#endif

    default:
#ifdef DEBUG_BUILD
      printf("Unhandled: %x\n", msg); fflush(stdout);
#endif

      break;
  }
  // Run the default window procedure for unhandled stuffs
  return WinDefWindowProc(hwnd, msg, mp1, mp2);
}

/////////////////////////////////////////////////////////////////////
//
// FrameWndProc
//
// This is the message processing window procedure for the
// frame window of SDLWindowClass.
//
/////////////////////////////////////////////////////////////////////
static MRESULT EXPENTRY FrameWndProc( HWND hwnd, ULONG msg, MPARAM mp1, MPARAM mp2)
{
  PFNWP pOldFrameProc;
  MRESULT result;
  PTRACKINFO ti;
  int cx, cy, ncx, ncy;
  RECTL rclTemp;
  PSWP pswpTemp;

  SDL_VideoDevice *pVideo = NULL;

  pVideo = (SDL_VideoDevice *) WinQueryWindowULong(hwnd, QWL_USER);

  pOldFrameProc = pVideo->hidden->pfnOldFrameProc;

  if ((pVideo->hidden->bProportionalResize) &&
      (msg==WM_ADJUSTWINDOWPOS) &&
      (!FSLib_QueryFSMode(pVideo->hidden->hwndClient))
     )
  {
    pswpTemp = (PSWP) mp1;

    /* Resizing? */
    if (pswpTemp->fl & SWP_SIZE)
    {
      /* Calculate client size */
      rclTemp.xLeft = pswpTemp->x;
      rclTemp.xRight = pswpTemp->x + pswpTemp->cx;
      rclTemp.yBottom = pswpTemp->y;
      rclTemp.yTop = pswpTemp->y + pswpTemp->cy;
      WinCalcFrameRect(hwnd, &rclTemp, TRUE);

      ncx = cx = rclTemp.xRight - rclTemp.xLeft;
      ncy = cy = rclTemp.yTop - rclTemp.yBottom;

      /* Calculate new size to keep it proportional */

      if ((pVideo->hidden->ulResizingFlag & TF_LEFT) || (pVideo->hidden->ulResizingFlag & TF_RIGHT))
      {
        /* The window is resized horizontally */
        ncy = pVideo->hidden->SrcBufferDesc.uiYResolution * cx / pVideo->hidden->SrcBufferDesc.uiXResolution;
      } else
      if ((pVideo->hidden->ulResizingFlag & TF_TOP) || (pVideo->hidden->ulResizingFlag & TF_BOTTOM))
      {
        /* The window is resized vertically */
        ncx = pVideo->hidden->SrcBufferDesc.uiXResolution * cy / pVideo->hidden->SrcBufferDesc.uiYResolution;
      }

      /* Calculate back frame coordinates */
      rclTemp.xLeft = pswpTemp->x;
      rclTemp.xRight = pswpTemp->x + ncx;
      rclTemp.yBottom = pswpTemp->y;
      rclTemp.yTop = pswpTemp->y + ncy;
      WinCalcFrameRect(hwnd, &rclTemp, FALSE);

      /* Store new size/position info */
      pswpTemp->cx = rclTemp.xRight - rclTemp.xLeft;

      if (!(pVideo->hidden->ulResizingFlag & TF_TOP))
      {
        pswpTemp->y = pswpTemp->y + pswpTemp->cy - (rclTemp.yTop - rclTemp.yBottom);
        pswpTemp->cy = rclTemp.yTop - rclTemp.yBottom;
      } else
      {
        pswpTemp->cy = rclTemp.yTop - rclTemp.yBottom;
      }
    }
  }

  result = (*pOldFrameProc)(hwnd, msg, mp1, mp2);

  if ((pVideo->hidden->bProportionalResize) && (msg==WM_QUERYTRACKINFO))
  {
    ti = (PTRACKINFO) mp2;

    /* Store the direction of resizing */
    if ((ti->fs & TF_LEFT) || (ti->fs & TF_RIGHT) ||
        (ti->fs & TF_TOP) || (ti->fs & TF_BOTTOM))
      pVideo->hidden->ulResizingFlag = ti->fs;
  }

  return result;
}

/////////////////////////////////////////////////////////////////////
//
// PMThreadFunc
//
// This function implements the PM-Thread, which initializes the
// application window itself, the DIVE, and start message processing.
//
/////////////////////////////////////////////////////////////////////
int iNumOfPMThreadInstances = 0; // Global!
static void PMThreadFunc(void *pParm)
{
  SDL_VideoDevice *pVideo = pParm;
  HAB hab;
  HMQ hmq;
  QMSG msg;
  ULONG fcf;

#ifdef DEBUG_BUILD
  printf("[PMThreadFunc] : Starting\n"); fflush(stdout);
#endif

  iNumOfPMThreadInstances++;

  // Initialize PM, create a message queue.

  hab=WinInitialize(0);
  hmq=WinCreateMsgQueue(hab,0);
  if (hmq==0)
  {
#ifdef DEBUG_BUILD
    printf("[PMThreadFunc] : Could not create message queue!\n");
    printf("                 It might be that the application using SDL is not a PM app!\n");
    fflush(stdout);
#endif
    pVideo->hidden->iPMThreadStatus = 2;
  } else
  {
    int rc;
    RECTL rectl;

    fcf = ulFCFToUse; // Get from global setting

#ifdef DEBUG_BUILD
    printf("[PMThreadFunc] : FSLib_CreateWindow()!\n");
    fflush(stdout);
#endif

    rc = FSLib_CreateWindow(HWND_DESKTOP, 0, &fcf,
                            "SDL Application",
                            NULLHANDLE, 0,
                            &(pVideo->hidden->SrcBufferDesc),
                            WndProc,
                            &(pVideo->hidden->hwndClient),
                            &(pVideo->hidden->hwndFrame));

#ifdef DEBUG_BUILD
    printf("[PMThreadFunc] : FSLib_CreateWindow() rc = %d\n", rc);
    fflush(stdout);
#endif

    if (!rc)
    {
#ifdef DEBUG_BUILD
      printf("[PMThreadFunc] : Could not create FSLib window!\n");
      fflush(stdout);
#endif
      pVideo->hidden->iPMThreadStatus = 3;
    } else
    {
#ifdef DEBUG_BUILD
      printf("[PMThreadFunc] : FSLib_AddUserParm()!\n");
      fflush(stdout);
#endif

      // Store pVideo pointer in window data for client window, so
      // it will know the instance to which it belongs to.
      FSLib_AddUserParm(pVideo->hidden->hwndClient, pVideo);

      // Now set default image width height and fourcc!
#ifdef DEBUG_BUILD
      printf("[PMThreadFunc] : SetWindowPos()!\n");
      fflush(stdout);
#endif

      // Set the position and size of the main window,
      // and make it visible!
      // Calculate frame window size from client window size
      rectl.xLeft = 0;
      rectl.yBottom = 0;
      rectl.xRight = pVideo->hidden->SrcBufferDesc.uiXResolution; // Noninclusive
      rectl.yTop = pVideo->hidden->SrcBufferDesc.uiYResolution; // Noninclusive
      WinCalcFrameRect(pVideo->hidden->hwndFrame, &rectl, FALSE);

      SetAccessableWindowPos(pVideo->hidden->hwndFrame,
                             HWND_TOP,
                             (WinQuerySysValue (HWND_DESKTOP, SV_CXSCREEN) - (rectl.xRight-rectl.xLeft)) / 2,
                             (WinQuerySysValue (HWND_DESKTOP, SV_CYSCREEN) - (rectl.yTop-rectl.yBottom)) / 2,
                             (rectl.xRight-rectl.xLeft),
                             (rectl.yTop-rectl.yBottom),
                             SWP_SIZE | SWP_ACTIVATE | SWP_SHOW | SWP_MOVE);

      // Subclass frame procedure and store old window proc address
      pVideo->hidden->pfnOldFrameProc =
        WinSubclassWindow(pVideo->hidden->hwndFrame, FrameWndProc);
      WinSetWindowULong(pVideo->hidden->hwndFrame, QWL_USER, (ULONG) pVideo);

#ifdef DEBUG_BUILD
      printf("[PMThreadFunc] : Entering message loop\n"); fflush(stdout);
#endif
      pVideo->hidden->iPMThreadStatus = 1;
  
      while (WinGetMsg(hab, (PQMSG)&msg, 0, 0, 0))
        WinDispatchMsg(hab, (PQMSG) &msg);

#ifdef DEBUG_BUILD
      printf("[PMThreadFunc] : Leaving message loop\n"); fflush(stdout);
#endif
      // We should release the captured the mouse!
      if (bMouseCaptured)
      {
        WinSetCapture(HWND_DESKTOP, NULLHANDLE);
        bMouseCaptured = 0;
      }
      // Destroy our window
      WinDestroyWindow(pVideo->hidden->hwndFrame); pVideo->hidden->hwndFrame=NULL;
      // Show pointer to make sure it will not be left hidden.
      WinSetPointer(HWND_DESKTOP, WinQuerySysPointer(HWND_DESKTOP, SPTR_ARROW, FALSE));
      WinShowPointer(HWND_DESKTOP, TRUE);
    }
    // Uninitialize PM
    WinDestroyMsgQueue(hmq);
    // All done!
    pVideo->hidden->iPMThreadStatus = 0;
  }
  WinTerminate(hab);
  /* Commented out, should not be needed anymore, because we send it
     from WM_CLOSE.
  // Notify SDL that it should really die now...
  SDL_PrivateQuit(); SDL_PrivateQuit(); SDL_PrivateQuit(); //... :))
  */
#ifdef DEBUG_BUILD
  printf("[PMThreadFunc] : End, status is %d!\n", pVideo->hidden->iPMThreadStatus); fflush(stdout);
#endif

  iNumOfPMThreadInstances--;

  // HACK to prevent zombie and hanging SDL applications, which does not take
  // care of closing the window for some reason:
  // There are some apps which do not process messages, so do a lot of things
  // without noticing that the application should close. To close these,
  // I've thought about the following:
  // If the window is closed (the execution came here), I wait a bit to
  // give time to the app to finish its execution. If it does not, I kill it
  // using DosExit(). Brute force, but should work.
  if (pVideo->hidden->iPMThreadStatus==0)
  {
    DosSleep(5000); // Wait 5 secs
    // If a new PM thread has been spawned (reinitializing video mode), then all right.
    // Otherwise, we have a problem, the app doesn't want to stop. Kill!
    if (iNumOfPMThreadInstances==0)
    {
#ifdef DEBUG_BUILD
      printf("[PMThreadFunc] : It seems that the application haven't terminated itself\n"); fflush(stdout);
      printf("[PMThreadFunc] : in the last 5 seconds, so we go berserk.\n"); fflush(stdout);
      printf("[PMThreadFunc] : Brute force mode. :) Killing process! Dieeeee...\n"); fflush(stdout);
#endif
      DosExit(EXIT_PROCESS, -1);
    }
  }
  _endthread();
}

struct WMcursor
{
  HBITMAP hbm;
  HPOINTER hptr;
  char *pchData;
};

/* Free a window manager cursor */
void os2fslib_FreeWMCursor(_THIS, WMcursor *cursor)
{
  if (cursor)
  {
    GpiDeleteBitmap(cursor->hbm);
    WinDestroyPointer(cursor->hptr);
    SDL_free(cursor->pchData);
    SDL_free(cursor);
  }
}

/* Local functions to convert the SDL cursor mask into OS/2 format */
static void memnot(Uint8 *dst, Uint8 *src, int len)
{
  while ( len-- > 0 )
    *dst++ = ~*src++;
}
static void memxor(Uint8 *dst, Uint8 *src1, Uint8 *src2, int len)
{
  while ( len-- > 0 )
    *dst++ = (*src1++)^(*src2++);
}

/* Create a black/white window manager cursor */
WMcursor *os2fslib_CreateWMCursor_Win(_THIS, Uint8 *data, Uint8 *mask,
                                      int w, int h, int hot_x, int hot_y)
{
  HPOINTER hptr;
  HBITMAP hbm;
  BITMAPINFOHEADER bmih;
  BMPINFO          bmi;
  HPS              hps;
  char *pchTemp;
  char *xptr, *aptr;
  int maxx, maxy;
  int i, run, pad;
  WMcursor *pResult;

  maxx = WinQuerySysValue(HWND_DESKTOP, SV_CXPOINTER);
  maxy = WinQuerySysValue(HWND_DESKTOP, SV_CYPOINTER);

  // Check for max size!
  if ((w>maxx) || (h>maxy))
    return (WMcursor *) NULL;

  pResult = (WMcursor *) SDL_malloc(sizeof(WMcursor));
  if (!pResult) return (WMcursor *) NULL;

  pchTemp = (char *) SDL_malloc((maxx + 7)/8 * maxy*2);
  if (!pchTemp)
  {
    SDL_free(pResult);
    return (WMcursor *) NULL;
  }

  SDL_memset(pchTemp, 0, (maxx + 7)/8 * maxy*2);

  hps = WinGetPS(_this->hidden->hwndClient);

  bmi.cbFix = sizeof(BITMAPINFOHEADER);
  bmi.cx = maxx;
  bmi.cy = 2*maxy;
  bmi.cPlanes = 1;
  bmi.cBitCount = 1;
  bmi.argbColor[0].bBlue = 0x00;
  bmi.argbColor[0].bGreen = 0x00;
  bmi.argbColor[0].bRed = 0x00;
  bmi.argbColor[1].bBlue = 0x00;
  bmi.argbColor[1].bGreen = 0x00;
  bmi.argbColor[1].bRed = 0xff;

  SDL_memset(&bmih, 0, sizeof(BITMAPINFOHEADER));
  bmih.cbFix = sizeof(BITMAPINFOHEADER);
  bmih.cx = maxx;
  bmih.cy = 2*maxy;
  bmih.cPlanes = 1;
  bmih.cBitCount = 1;

  run = (w+7)/8;
  pad = (maxx+7)/8 - run;

  for (i=0; i<h; i++)
  {
    xptr = pchTemp + (maxx+7)/8 * (maxy-1-i);
    aptr = pchTemp + (maxx+7)/8 * (maxy+maxy-1-i);
    memxor(xptr, data, mask, run);
    xptr += run;
    data += run;
    memnot(aptr, mask, run);
    mask += run;
    aptr += run;
    SDL_memset(xptr,  0, pad);
    xptr += pad;
    SDL_memset(aptr, ~0, pad);
    aptr += pad;
  }
  pad += run;
  for (i=h ; i<maxy; i++ )
  {
    xptr = pchTemp + (maxx+7)/8 * (maxy-1-i);
    aptr = pchTemp + (maxx+7)/8 * (maxy+maxy-1-i);

    SDL_memset(xptr,  0, (maxx+7)/8);
    xptr += (maxx+7)/8;
    SDL_memset(aptr, ~0, (maxx+7)/8);
    aptr += (maxx+7)/8;
  }

  hbm = GpiCreateBitmap(hps, (PBITMAPINFOHEADER2)&bmih, CBM_INIT, (PBYTE) pchTemp, (PBITMAPINFO2)&bmi);
  hptr = WinCreatePointer(HWND_DESKTOP, hbm, TRUE, hot_x, maxy - hot_y - 1);

#ifdef DEBUG_BUILD
  printf("HotSpot          : %d ; %d\n", hot_x, hot_y);
  printf("HPS returned     : %x\n", (ULONG)hps);
  printf("HBITMAP returned : %x\n", (ULONG)hbm);
  printf("HPOINTER returned: %x\n", (ULONG)hptr);
#endif

  WinReleasePS(hps);

#ifdef DEBUG_BUILD
  printf("[CreateWMCursor] : ptr = %p\n", hptr); fflush(stdout);
#endif

  pResult->hptr = hptr;
  pResult->hbm = hbm;
  pResult->pchData = pchTemp;

#ifdef DEBUG_BUILD
  printf("[CreateWMCursor] : ptr = %p return.\n", hptr); fflush(stdout);
#endif

  return (WMcursor *) pResult;
}

WMcursor *os2fslib_CreateWMCursor_FS(_THIS, Uint8 *data, Uint8 *mask,
                                     int w, int h, int hot_x, int hot_y)
{
#ifdef DEBUG_BUILD
  printf("[CreateWMCursor_FS] : returning pointer NULL\n"); fflush(stdout);
#endif

  // In FS mode we'll use software cursor
  return (WMcursor *) NULL;
}

/* Show the specified cursor, or hide if cursor is NULL */
int os2fslib_ShowWMCursor(_THIS, WMcursor *cursor)
{
#ifdef DEBUG_BUILD
  printf("[ShowWMCursor] : ptr = %p\n", cursor); fflush(stdout);
#endif

  if (cursor)
  {
    WinSetPointer(HWND_DESKTOP, cursor->hptr);
    hptrGlobalPointer = cursor->hptr;
    _this->hidden->iMouseVisible = 1;
  }
  else
  {
    WinSetPointer(HWND_DESKTOP, FALSE);
    hptrGlobalPointer = NULL;
    _this->hidden->iMouseVisible = 0;
  }

#ifdef DEBUG_BUILD
  printf("[ShowWMCursor] : ptr = %p, DONE\n", cursor); fflush(stdout);
#endif

  return 1;
}

/* Warp the window manager cursor to (x,y)
 If NULL, a mouse motion event is posted internally.
 */
void os2fslib_WarpWMCursor(_THIS, Uint16 x, Uint16 y)
{
  LONG lx, ly;
  SWP swpClient;
  POINTL ptlPoints;
  WinQueryWindowPos(_this->hidden->hwndClient, &swpClient);
  ptlPoints.x = swpClient.x;
  ptlPoints.y = swpClient.y;
  WinMapWindowPoints(_this->hidden->hwndFrame, HWND_DESKTOP, &ptlPoints, 1);
  lx = ptlPoints.x + (x*swpClient.cx) / _this->hidden->SrcBufferDesc.uiXResolution;
  ly = ptlPoints.y + swpClient.cy - ((y*swpClient.cy) / _this->hidden->SrcBufferDesc.uiYResolution) - 1;

  SDL_PrivateMouseMotion(0, // Buttons not changed
                         0, // Absolute position
                         x,
                         y);

  WinSetPointerPos(HWND_DESKTOP, lx, ly);

}

/* If not NULL, this is called when a mouse motion event occurs */
void os2fslib_MoveWMCursor(_THIS, int x, int y)
{
  /*
  SDL_Rect rect;

#ifdef DEBUG_BUILD
  printf("[MoveWMCursor] : at %d ; %d\n", x, y); fflush(stdout);
#endif

  rect.x = x;
  rect.y = y;
  rect.w = 32;
  rect.h = 32;
  os2fslib_UpdateRects(_this, 1, &rect);
  // TODO!
  */
}

/* Determine whether the mouse should be in relative mode or not.
 This function is called when the input grab state or cursor
 visibility state changes.
 If the cursor is not visible, and the input is grabbed, the
 driver can place the mouse in relative mode, which may result
 in higher accuracy sampling of the pointer motion.
 */
void os2fslib_CheckMouseMode(_THIS)
{
}

static void os2fslib_PumpEvents(_THIS)
{
  // Notify SDL that if window has been resized!
  if (
      (_this->hidden->pSDLSurface) &&
      (_this->hidden->pSDLSurface->flags & SDL_RESIZABLE) &&
      (
       (_this->hidden->SrcBufferDesc.uiXResolution!=iWindowSizeX) ||
       (_this->hidden->SrcBufferDesc.uiYResolution!=iWindowSizeY)
      ) &&
      (iWindowSizeX>0) &&
      (iWindowSizeY>0)
     )
  {
    static time_t prev_time;
    time_t curr_time;

    curr_time = time(NULL);
    if ((difftime(curr_time, prev_time)>=0.25) ||
        (bWindowResized))
    {
      // Make sure we won't flood the event queue with resize events,
      // only send them at 250 msecs!
      // (or when the window is resized)
#ifdef DEBUG_BUILD
      printf("[os2fslib_PumpEvents] : Calling PrivateResize (%d %d).\n",
             iWindowSizeX, iWindowSizeY);
      fflush(stdout);
#endif
      // Tell SDL the new size
      SDL_PrivateResize(iWindowSizeX, iWindowSizeY);
      prev_time = curr_time;
      bWindowResized = 0;
    }
  }
}

/* We don't actually allow hardware surfaces other than the main one */
static int os2fslib_AllocHWSurface(_THIS, SDL_Surface *surface)
{
  return(-1);
}
static void os2fslib_FreeHWSurface(_THIS, SDL_Surface *surface)
{
  return;
}

/* We need to wait for vertical retrace on page flipped displays */
static int os2fslib_LockHWSurface(_THIS, SDL_Surface *surface)
{
  return(0);
}

static void os2fslib_UnlockHWSurface(_THIS, SDL_Surface *surface)
{
  return;
}

static int os2fslib_SetColors(_THIS, int firstcolor, int ncolors, SDL_Color *colors)
{
  printf("[os2fslib_SetColors] : TODO!\n"); fflush(stdout);
  // TODO: Implement paletted modes
  return(1);
}

static void os2fslib_DestroyIcon(HWND hwndFrame)
{
  if (hptrCurrentIcon)
  {
    WinDestroyPointer(hptrCurrentIcon);
    hptrCurrentIcon = NULL;

    WinSendMsg(hwndFrame,
               WM_SETICON,
               NULL,
               NULL);
  }

}

/* Set the window icon image */
void os2fslib_SetIcon(_THIS, SDL_Surface *icon, Uint8 *mask)
{
  HWND hwndFrame;
  SDL_Surface *icon_rgb;
  HPOINTER hptrIcon;
  HBITMAP hbm;
  BITMAPINFOHEADER bmih;
  BMPINFO          bmi;
  HPS              hps;
  char *pchTemp;
  char *pptr, *mptr, *dptr, *dmptr;
  int maxx, maxy, w, h, x, y;
  SDL_Rect bounds;

#ifdef DEBUG_BUILD
  printf("[os2fslib_SetIcon] : Creating and setting new icon\n"); fflush(stdout);
#endif

  hwndFrame = WinQueryWindow(_this->hidden->hwndClient, QW_PARENT);

  // Make sure the old icon resource will be free'd!
  os2fslib_DestroyIcon(hwndFrame);

  if ((!icon) || (!mask))
    return;

  w = icon->w;
  h = icon->h;

  maxx = WinQuerySysValue(HWND_DESKTOP, SV_CXICON);
  maxy = WinQuerySysValue(HWND_DESKTOP, SV_CYICON);

  // Check for max size!
  if ((w>maxx) || (h>maxy))
    return;

  pchTemp = (char *) SDL_malloc(w * h*2 * 4);
  if (!pchTemp)
    return;

  SDL_memset(pchTemp, 0, w * h*2 * 4);

  // Convert surface to RGB, if it's not RGB yet!
  icon_rgb = SDL_CreateRGBSurface(SDL_SWSURFACE, icon->w, icon->h,
                                  32, 0, 0, 0, 0);
  if ( icon_rgb == NULL )
  {
    SDL_free(pchTemp);
    return;
  }
  bounds.x = 0;
  bounds.y = 0;
  bounds.w = icon->w;
  bounds.h = icon->h;
  if ( SDL_LowerBlit(icon, &bounds, icon_rgb, &bounds) < 0 )
  {
    SDL_FreeSurface(icon_rgb);
    SDL_free(pchTemp);
    return;
  }

  /* Copy pixels upside-down from RGB surface into BMP, masked with the icon mask */

  // Pixels
  pptr = (char *) (icon_rgb->pixels);
  // Mask
  mptr = mask;

  for (y=0; y<h; y++)
  {
    unsigned char uchMaskByte;

    // Destination
    dptr = pchTemp + w*4 * (h-y-1);
    // Destination mask
    dmptr = pchTemp + w*h*4 + w*4 * (h-y-1);

    for (x=0; x<w; x++)
    {
      if (x%8==0)
      {
        uchMaskByte = (unsigned char) (*mptr);
        mptr++;
      } else
        uchMaskByte <<= 1;

      if (uchMaskByte & 0x80)
      {
        // Copy RGB
        *dptr++ = *pptr++;
        *dptr++ = *pptr++;
        *dptr++ = *pptr++;
        *dptr++ = *pptr++;

        *dmptr++ = 0;
        *dmptr++ = 0;
        *dmptr++ = 0;
        *dmptr++ = 0;
      } else
      {
        // Set pixels to fully transparent
        *dptr++ = 0; pptr++;
        *dptr++ = 0; pptr++;
        *dptr++ = 0; pptr++;
        *dptr++ = 0; pptr++;

        *dmptr++ = 255;
        *dmptr++ = 255;
        *dmptr++ = 255;
        *dmptr++ = 255;
      }
    }
  }

  // There is no more need for the RGB surface
  SDL_FreeSurface(icon_rgb);

  hps = WinGetPS(_this->hidden->hwndClient);

  bmi.cbFix = sizeof(BITMAPINFOHEADER);
  bmi.cx = w;
  bmi.cy = 2*h;
  bmi.cPlanes = 1;
  bmi.cBitCount = 32;

  SDL_memset(&bmih, 0, sizeof(BITMAPINFOHEADER));
  bmih.cbFix = sizeof(BITMAPINFOHEADER);
  bmih.cx = w;
  bmih.cy = 2*h;
  bmih.cPlanes = 1;
  bmih.cBitCount = 32;

  hbm = GpiCreateBitmap(hps, (PBITMAPINFOHEADER2)&bmih, CBM_INIT, (PBYTE) pchTemp, (PBITMAPINFO2)&bmi);
  hptrIcon = WinCreatePointer(HWND_DESKTOP, hbm, FALSE, 0, 0);

  WinReleasePS(hps);

  // Free pixel array
  SDL_free(pchTemp);

  // Change icon in frame window
  WinSendMsg(hwndFrame,
             WM_SETICON,
             (MPARAM) hptrIcon,
             NULL);

  /*
  // Change icon in switchlist
  // Seems like it's not needed, the WM_SETICON already does it.
  {
    PID pidFrame;
    HSWITCH hswitchFrame;
    SWCNTRL swctl;

    WinQueryWindowProcess(hwndFrame, &pidFrame, NULL);
    hswitchFrame = WinQuerySwitchHandle(hwndFrame, pidFrame);
    WinQuerySwitchEntry(hswitchFrame, &swctl);

    swctl.hwndIcon = hptrIcon;

    WinChangeSwitchEntry(hswitchFrame, &swctl);
  }
  */

  // Store icon handle in global variable
  hptrCurrentIcon = hptrIcon;
}

// ------------------------ REAL FUNCTIONS -----------------


static void os2fslib_SetCursorManagementFunctions(_THIS, int iForWindowedMode)
{
  if (iForWindowedMode)
  {
    _this->FreeWMCursor = os2fslib_FreeWMCursor;
    _this->CreateWMCursor = os2fslib_CreateWMCursor_Win;
    _this->ShowWMCursor = os2fslib_ShowWMCursor;
    _this->WarpWMCursor = os2fslib_WarpWMCursor;
    _this->MoveWMCursor = os2fslib_MoveWMCursor;
    _this->CheckMouseMode = NULL;//os2fslib_CheckMouseMode;
  } else
  {
    // We'll have software mouse cursor in FS mode!
    _this->FreeWMCursor = os2fslib_FreeWMCursor;
    _this->CreateWMCursor = os2fslib_CreateWMCursor_FS;
    _this->ShowWMCursor = os2fslib_ShowWMCursor;
    _this->WarpWMCursor = os2fslib_WarpWMCursor;
    _this->MoveWMCursor = os2fslib_MoveWMCursor;
    _this->CheckMouseMode = NULL;//os2fslib_CheckMouseMode;
  }
}

static void os2fslib_InitOSKeymap(_THIS)
{
  int i;

  iShiftIsPressed = 0;

  /* Map the VK and CH keysyms */
  for ( i=0; i<=255; ++i )
    HWScanKeyMap[i] = SDLK_UNKNOWN;

  // First line of keyboard:
  HWScanKeyMap[0x1] = SDLK_ESCAPE;
  HWScanKeyMap[0x3b] = SDLK_F1;
  HWScanKeyMap[0x3c] = SDLK_F2;
  HWScanKeyMap[0x3d] = SDLK_F3;
  HWScanKeyMap[0x3e] = SDLK_F4;
  HWScanKeyMap[0x3f] = SDLK_F5;
  HWScanKeyMap[0x40] = SDLK_F6;
  HWScanKeyMap[0x41] = SDLK_F7;
  HWScanKeyMap[0x42] = SDLK_F8;
  HWScanKeyMap[0x43] = SDLK_F9;
  HWScanKeyMap[0x44] = SDLK_F10;
  HWScanKeyMap[0x57] = SDLK_F11;
  HWScanKeyMap[0x58] = SDLK_F12;
  HWScanKeyMap[0x5d] = SDLK_PRINT;
  HWScanKeyMap[0x46] = SDLK_SCROLLOCK;
  HWScanKeyMap[0x5f] = SDLK_PAUSE;

  // Second line of keyboard:
  HWScanKeyMap[0x29] = SDLK_BACKQUOTE;
  HWScanKeyMap[0x2] = SDLK_1;
  HWScanKeyMap[0x3] = SDLK_2;
  HWScanKeyMap[0x4] = SDLK_3;
  HWScanKeyMap[0x5] = SDLK_4;
  HWScanKeyMap[0x6] = SDLK_5;
  HWScanKeyMap[0x7] = SDLK_6;
  HWScanKeyMap[0x8] = SDLK_7;
  HWScanKeyMap[0x9] = SDLK_8;
  HWScanKeyMap[0xa] = SDLK_9;
  HWScanKeyMap[0xb] = SDLK_0;
  HWScanKeyMap[0xc] = SDLK_MINUS;
  HWScanKeyMap[0xd] = SDLK_EQUALS;
  HWScanKeyMap[0xe] = SDLK_BACKSPACE;
  HWScanKeyMap[0x68] = SDLK_INSERT;
  HWScanKeyMap[0x60] = SDLK_HOME;
  HWScanKeyMap[0x62] = SDLK_PAGEUP;
  HWScanKeyMap[0x45] = SDLK_NUMLOCK;
  HWScanKeyMap[0x5c] = SDLK_KP_DIVIDE;
  HWScanKeyMap[0x37] = SDLK_KP_MULTIPLY;
  HWScanKeyMap[0x4a] = SDLK_KP_MINUS;

  // Third line of keyboard:
  HWScanKeyMap[0xf] = SDLK_TAB;
  HWScanKeyMap[0x10] = SDLK_q;
  HWScanKeyMap[0x11] = SDLK_w;
  HWScanKeyMap[0x12] = SDLK_e;
  HWScanKeyMap[0x13] = SDLK_r;
  HWScanKeyMap[0x14] = SDLK_t;
  HWScanKeyMap[0x15] = SDLK_y;
  HWScanKeyMap[0x16] = SDLK_u;
  HWScanKeyMap[0x17] = SDLK_i;
  HWScanKeyMap[0x18] = SDLK_o;
  HWScanKeyMap[0x19] = SDLK_p;
  HWScanKeyMap[0x1a] = SDLK_LEFTBRACKET;
  HWScanKeyMap[0x1b] = SDLK_RIGHTBRACKET;
  HWScanKeyMap[0x1c] = SDLK_RETURN;
  HWScanKeyMap[0x69] = SDLK_DELETE;
  HWScanKeyMap[0x65] = SDLK_END;
  HWScanKeyMap[0x67] = SDLK_PAGEDOWN;
  HWScanKeyMap[0x47] = SDLK_KP7;
  HWScanKeyMap[0x48] = SDLK_KP8;
  HWScanKeyMap[0x49] = SDLK_KP9;
  HWScanKeyMap[0x4e] = SDLK_KP_PLUS;

  // Fourth line of keyboard:
  HWScanKeyMap[0x3a] = SDLK_CAPSLOCK;
  HWScanKeyMap[0x1e] = SDLK_a;
  HWScanKeyMap[0x1f] = SDLK_s;
  HWScanKeyMap[0x20] = SDLK_d;
  HWScanKeyMap[0x21] = SDLK_f;
  HWScanKeyMap[0x22] = SDLK_g;
  HWScanKeyMap[0x23] = SDLK_h;
  HWScanKeyMap[0x24] = SDLK_j;
  HWScanKeyMap[0x25] = SDLK_k;
  HWScanKeyMap[0x26] = SDLK_l;
  HWScanKeyMap[0x27] = SDLK_SEMICOLON;
  HWScanKeyMap[0x28] = SDLK_QUOTE;
  HWScanKeyMap[0x2b] = SDLK_BACKSLASH;
  HWScanKeyMap[0x4b] = SDLK_KP4;
  HWScanKeyMap[0x4c] = SDLK_KP5;
  HWScanKeyMap[0x4d] = SDLK_KP6;

  // Fifth line of keyboard:
  HWScanKeyMap[0x2a] = SDLK_LSHIFT;
  HWScanKeyMap[0x56] = SDLK_WORLD_1; // Code 161, letter i' on hungarian keyboard
  HWScanKeyMap[0x2c] = SDLK_z;
  HWScanKeyMap[0x2d] = SDLK_x;
  HWScanKeyMap[0x2e] = SDLK_c;
  HWScanKeyMap[0x2f] = SDLK_v;
  HWScanKeyMap[0x30] = SDLK_b;
  HWScanKeyMap[0x31] = SDLK_n;
  HWScanKeyMap[0x32] = SDLK_m;
  HWScanKeyMap[0x33] = SDLK_COMMA;
  HWScanKeyMap[0x34] = SDLK_PERIOD;
  HWScanKeyMap[0x35] = SDLK_SLASH;
  HWScanKeyMap[0x36] = SDLK_RSHIFT;
  HWScanKeyMap[0x61] = SDLK_UP;
  HWScanKeyMap[0x4f] = SDLK_KP1;
  HWScanKeyMap[0x50] = SDLK_KP2;
  HWScanKeyMap[0x51] = SDLK_KP3;
  HWScanKeyMap[0x5a] = SDLK_KP_ENTER;

  // Sixth line of keyboard:
  HWScanKeyMap[0x1d] = SDLK_LCTRL;
  HWScanKeyMap[0x7e] = SDLK_LSUPER; // Windows key
  HWScanKeyMap[0x38] = SDLK_LALT;
  HWScanKeyMap[0x39] = SDLK_SPACE;
  HWScanKeyMap[0x5e] = SDLK_RALT;// Actually, altgr on my keyboard...
  HWScanKeyMap[0x7f] = SDLK_RSUPER;
  HWScanKeyMap[0x7c] = SDLK_MENU;
  HWScanKeyMap[0x5b] = SDLK_RCTRL;
  HWScanKeyMap[0x63] = SDLK_LEFT;
  HWScanKeyMap[0x66] = SDLK_DOWN;
  HWScanKeyMap[0x64] = SDLK_RIGHT;
  HWScanKeyMap[0x52] = SDLK_KP0;
  HWScanKeyMap[0x53] = SDLK_KP_PERIOD;
}


/* Iconify the window.
 This function returns 1 if there is a window manager and the
 window was actually iconified, it returns 0 otherwise.
 */
int os2fslib_IconifyWindow(_THIS)
{
  HAB hab;
  HMQ hmq;
  ERRORID hmqerror;

  // If there is no more window, nothing we can do!
  if (_this->hidden->iPMThreadStatus!=1) return 0;

  // Cannot do anything in fullscreen mode!
  if (FSLib_QueryFSMode(_this->hidden->hwndClient))
    return 0;

  // Make sure this thread is prepared for using the Presentation Manager!
  hab = WinInitialize(0);
  hmq = WinCreateMsgQueue(hab,0);
  // Remember if there was an error at WinCreateMsgQueue(), because we don't
  // want to destroy somebody else's queue later. :)
  hmqerror = WinGetLastError(hab);

  WinSetWindowPos(_this->hidden->hwndFrame, HWND_TOP,
                 0, 0, 0, 0, SWP_MINIMIZE);

  // Now destroy the message queue, if we've created it!
  if (ERRORIDERROR(hmqerror)==0)
    WinDestroyMsgQueue(hmq);

  return 1;
}

static SDL_GrabMode os2fslib_GrabInput(_THIS, SDL_GrabMode mode)
{
  HAB hab;
  HMQ hmq;
  ERRORID hmqerror;


  // If there is no more window, nothing we can do!
  if (_this->hidden->iPMThreadStatus!=1)
    return SDL_GRAB_OFF;

  // Make sure this thread is prepared for using the Presentation Manager!
  hab = WinInitialize(0);
  hmq = WinCreateMsgQueue(hab,0);
  // Remember if there was an error at WinCreateMsgQueue(), because we don't
  // want to destroy somebody else's queue later. :)
  hmqerror = WinGetLastError(hab);


  if (mode == SDL_GRAB_OFF)
  {
#ifdef DEBUG_BUILD
    printf("[os2fslib_GrabInput] : Releasing mouse\n"); fflush(stdout);
#endif

    // Release the mouse
    bMouseCapturable = 0;
    if (bMouseCaptured)
    {
      WinSetCapture(HWND_DESKTOP, NULLHANDLE);
      bMouseCaptured = 0;
    }
  } else
  {
#ifdef DEBUG_BUILD
    printf("[os2fslib_GrabInput] : Capturing mouse\n"); fflush(stdout);
#endif

    // Capture the mouse
    bMouseCapturable = 1;
    if (WinQueryFocus(HWND_DESKTOP) == _this->hidden->hwndClient)
    {
      WinSetCapture(HWND_DESKTOP, _this->hidden->hwndClient);
      bMouseCaptured = 1;
      {
        SWP swpClient;
        POINTL ptl;
        // Center the mouse to the middle of the window!
        WinQueryWindowPos(_this->hidden->hwndClient, &swpClient);
        ptl.x = 0; ptl.y = 0;
        WinMapWindowPoints(_this->hidden->hwndClient, HWND_DESKTOP, &ptl, 1);
        _this->hidden->iSkipWMMOUSEMOVE++; /* Don't take next WM_MOUSEMOVE into account!  */
        WinSetPointerPos(HWND_DESKTOP,
                         ptl.x + swpClient.cx/2,
                         ptl.y + swpClient.cy/2);
      }
    }
  }

  // Now destroy the message queue, if we've created it!
  if (ERRORIDERROR(hmqerror)==0)
    WinDestroyMsgQueue(hmq);

  return mode;
}

/* Set the title and icon text */
static void os2fslib_SetCaption(_THIS, const char *title, const char *icon)
{
  HAB hab;
  HMQ hmq;
  ERRORID hmqerror;

  // If there is no more window, nothing we can do!
  if (_this->hidden->iPMThreadStatus!=1) return;

  // Make sure this thread is prepared for using the Presentation Manager!
  hab = WinInitialize(0);
  hmq = WinCreateMsgQueue(hab,0);
  // Remember if there was an error at WinCreateMsgQueue(), because we don't
  // want to destroy somebody else's queue later. :)
  hmqerror = WinGetLastError(hab);

  WinSetWindowText(_this->hidden->hwndFrame, (char *) title);

  // Now destroy the message queue, if we've created it!
  if (ERRORIDERROR(hmqerror)==0)
    WinDestroyMsgQueue(hmq);
}

static int os2fslib_ToggleFullScreen(_THIS, int on)
{
#ifdef DEBUG_BUILD
  printf("[os2fslib_ToggleFullScreen] : %d\n", on); fflush(stdout);
#endif
  // If there is no more window, nothing we can do!
  if (_this->hidden->iPMThreadStatus!=1) return 0;

  FSLib_ToggleFSMode(_this->hidden->hwndClient, on);
  /* Cursor manager functions to Windowed/FS mode*/
  os2fslib_SetCursorManagementFunctions(_this, !on);
  return 1;
}

/* This is called after the video mode has been set, to get the
 initial mouse state.  It should queue events as necessary to
 properly represent the current mouse focus and position.
 */
static void os2fslib_UpdateMouse(_THIS)
{
  POINTL ptl;
  HAB hab;
  HMQ hmq;
  ERRORID hmqerror;
  SWP swpClient;

  // If there is no more window, nothing we can do!
  if (_this->hidden->iPMThreadStatus!=1) return;


  // Make sure this thread is prepared for using the Presentation Manager!
  hab = WinInitialize(0);
  hmq = WinCreateMsgQueue(hab,0);
  // Remember if there was an error at WinCreateMsgQueue(), because we don't
  // want to destroy somebody else's queue later. :)
  hmqerror = WinGetLastError(hab);

  

  if (_this->hidden->fInFocus)
  {
    // If our app is in focus
    SDL_PrivateAppActive(1, SDL_APPMOUSEFOCUS);
    SDL_PrivateAppActive(1, SDL_APPINPUTFOCUS);
    SDL_PrivateAppActive(1, SDL_APPACTIVE);
    WinQueryPointerPos(HWND_DESKTOP, &ptl);
    WinMapWindowPoints(HWND_DESKTOP, _this->hidden->hwndClient, &ptl, 1);
    WinQueryWindowPos(_this->hidden->hwndClient, &swpClient);
    // Convert OS/2 mouse position to SDL position, and also scale it!
    ptl.x = ptl.x * _this->hidden->SrcBufferDesc.uiXResolution / swpClient.cx;
    ptl.y = ptl.y * _this->hidden->SrcBufferDesc.uiYResolution / swpClient.cy;
    ptl.y = _this->hidden->SrcBufferDesc.uiYResolution - ptl.y - 1;
    SDL_PrivateMouseMotion(0, 0, (Sint16) (ptl.x), (Sint16) (ptl.y));
  } else
  {
    // If we're not in focus
    SDL_PrivateAppActive(0, SDL_APPMOUSEFOCUS);
    SDL_PrivateAppActive(0, SDL_APPINPUTFOCUS);
    SDL_PrivateAppActive(0, SDL_APPACTIVE);
    SDL_PrivateMouseMotion(0, 0, (Sint16) -1, (Sint16) -1);
  }

  // Now destroy the message queue, if we've created it!
  if (ERRORIDERROR(hmqerror)==0)
    WinDestroyMsgQueue(hmq);

}

/* This pointer should exist in the native video subsystem and should
 point to an appropriate update function for the current video mode
 */
static void os2fslib_UpdateRects(_THIS, int numrects, SDL_Rect *rects)
{
  // If there is no more window, nothing we can do!
  if (_this->hidden->iPMThreadStatus!=1) return;

#ifdef BITBLT_IN_WINMESSAGEPROC
  WinSendMsg(_this->hidden->hwndClient,
                 WM_UPDATERECTSREQUEST,
                 (MPARAM) numrects,
                 (MPARAM) rects);
#else
  if (DosRequestMutexSem(_this->hidden->hmtxUseSrcBuffer, SEM_INDEFINITE_WAIT)==NO_ERROR)
  {
    int i;

    if (_this->hidden->pSDLSurface)
    {
#ifndef RESIZE_EVEN_IF_RESIZABLE
      SWP swp;
      // But only blit if the window is not resizable, or if
      // the window is resizable and the source buffer size is the
      // same as the destination buffer size!
      WinQueryWindowPos(_this->hidden->hwndClient, &swp);
      if ((_this->hidden->pSDLSurface) &&
          (_this->hidden->pSDLSurface->flags & SDL_RESIZABLE) &&
          ((swp.cx != _this->hidden->SrcBufferDesc.uiXResolution) ||
           (swp.cy != _this->hidden->SrcBufferDesc.uiYResolution)
          ) &&
          (!FSLib_QueryFSMode(_this->hidden->hwndClient))
         )
      {
        // Resizable surface and in resizing!
        // So, don't blit now!
#ifdef DEBUG_BUILD
        printf("[UpdateRects] : Skipping blit while resizing!\n"); fflush(stdout);
#endif
      } else
#endif
      {
      /*
        // Blit the whole window
        FSLIB_BITBLT(_this->hidden->hwndClient, _this->hidden->pchSrcBuffer,
                     0, 0,
                     _this->hidden->SrcBufferDesc.uiXResolution,
                     _this->hidden->SrcBufferDesc.uiYResolution);
                     */
#ifdef DEBUG_BUILD
          printf("[os2fslib_UpdateRects] : Blitting!\n"); fflush(stdout);
#endif
  
        // Blit the changed areas
        for (i=0; i<numrects; i++)
          FSLIB_BITBLT(_this->hidden->hwndClient, _this->hidden->pchSrcBuffer,
                       rects[i].y, rects[i].x, rects[i].w, rects[i].h);
      }
    }
#ifdef DEBUG_BUILD
     else
       printf("[os2fslib_UpdateRects] : No public surface!\n"); fflush(stdout);
#endif
    DosReleaseMutexSem(_this->hidden->hmtxUseSrcBuffer);
  }
#ifdef DEBUG_BUILD
  else
    printf("[os2fslib_UpdateRects] : Error in mutex!\n"); fflush(stdout);
#endif
#endif
}


/* Reverse the effects VideoInit() -- called if VideoInit() fails
 or if the application is shutting down the video subsystem.
 */
static void os2fslib_VideoQuit(_THIS)
{
#ifdef DEBUG_BUILD
  printf("[os2fslib_VideoQuit]\n"); fflush(stdout);
#endif
  // Close PM stuff if running!
  if (_this->hidden->iPMThreadStatus == 1)
  {
    int iTimeout;
    WinPostMsg(_this->hidden->hwndFrame, WM_QUIT, (MPARAM) 0, (MPARAM) 0);
    // HACK: We had this line before:
    //DosWaitThread((TID *) &(_this->hidden->tidPMThread), DCWW_WAIT);
    // We don't use it, because the PMThread will never stop, or if it stops,
    // it will kill the whole process as a emergency fallback.
    // So, we only check for the iPMThreadStatus stuff!
#ifdef DEBUG_BUILD
    printf("[os2fslib_VideoQuit] : Waiting for PM thread to die\n"); fflush(stdout);
#endif

    iTimeout=0;
    while ((_this->hidden->iPMThreadStatus == 1) && (iTimeout<100))
    {
      iTimeout++;
      DosSleep(64);
    }

#ifdef DEBUG_BUILD
    printf("[os2fslib_VideoQuit] : End of wait.\n"); fflush(stdout);
#endif

    if (_this->hidden->iPMThreadStatus == 1)
    {
#ifdef DEBUG_BUILD
      printf("[os2fslib_VideoQuit] : Killing PM thread!\n"); fflush(stdout);
#endif
      
      _this->hidden->iPMThreadStatus = 0;
      DosKillThread(_this->hidden->tidPMThread);

      if (_this->hidden->hwndFrame)
      {
#ifdef DEBUG_BUILD
        printf("[os2fslib_VideoQuit] : Destroying PM window!\n"); fflush(stdout);
#endif

        WinDestroyWindow(_this->hidden->hwndFrame); _this->hidden->hwndFrame=NULL;
      }
    }

  }

  // Free result of an old ListModes() call, because there is
  // no FreeListModes() call in SDL!
  if (_this->hidden->pListModesResult)
  {
    SDL_free(_this->hidden->pListModesResult); _this->hidden->pListModesResult = NULL;
  }

  // Free list of available fullscreen modes
  if (_this->hidden->pAvailableFSLibVideoModes)
  {
    FSLib_FreeVideoModeList(_this->hidden->pAvailableFSLibVideoModes);
    _this->hidden->pAvailableFSLibVideoModes = NULL;
  }

  // Free application icon if we had one
  if (hptrCurrentIcon)
  {
    WinDestroyPointer(hptrCurrentIcon);
    hptrCurrentIcon = NULL;
  }
}

/* Set the requested video mode, returning a surface which will be
 set to the SDL_VideoSurface.  The width and height will already
 be verified by ListModes(), and the video subsystem is free to
 set the mode to a supported bit depth different from the one
 specified -- the desired bpp will be emulated with a shadow
 surface if necessary.  If a new mode is returned, this function
 should take care of cleaning up the current mode.
 */
static SDL_Surface *os2fslib_SetVideoMode(_THIS, SDL_Surface *current,
                                          int width, int height, int bpp, Uint32 flags)
{
  static int bFirstCall = 1;
  FSLib_VideoMode_p pModeInfo, pModeInfoFound;
  FSLib_VideoMode TempModeInfo;
  HAB hab;
  HMQ hmq;
  ERRORID hmqerror;
  RECTL rectl;
  SDL_Surface *pResult;

  // If there is no more window, nothing we can do!
  if (_this->hidden->iPMThreadStatus!=1) return NULL;

#ifdef DEBUG_BUILD
  printf("[os2fslib_SetVideoMode] : Request for %dx%d @ %dBPP, flags=0x%x\n", width, height, bpp, flags); fflush(stdout);
#endif

  // We don't support palette modes!
  if (bpp==8) bpp=32;

  // Also, we don't support resizable modes in fullscreen mode.
  if (flags & SDL_RESIZABLE)
    flags &= ~SDL_FULLSCREEN;

  // No double buffered mode
  if (flags & SDL_DOUBLEBUF)
    flags &= ~SDL_DOUBLEBUF;

  // And, we don't support HWSURFACE yet.
  if (flags & SDL_HWSURFACE)
  {
    flags &= ~SDL_HWSURFACE;
    flags |= SDL_SWSURFACE;
  }

#ifdef DEBUG_BUILD
  printf("[os2fslib_SetVideoMode] : Changed request to %dx%d @ %dBPP, flags=0x%x\n", width, height, bpp, flags); fflush(stdout);
#endif

  // First check if there is such a video mode they want!
  pModeInfoFound = NULL;

  // For fullscreen mode we don't support every resolution!
  // So, go through the video modes, and check for such a resolution!
  pModeInfoFound = NULL;
  pModeInfo = _this->hidden->pAvailableFSLibVideoModes;

  while (pModeInfo)
  {
    // Check all available fullscreen modes for this resolution
    if ((pModeInfo->uiXResolution == width) &&
        (pModeInfo->uiYResolution == height) &&
        (pModeInfo->uiBPP!=8)) // palettized modes not yet supported
    {
      // If good resolution, try to find the exact BPP, or at least
      // something similar...
      if (!pModeInfoFound)
        pModeInfoFound = pModeInfo;
      else
      if ((pModeInfoFound->uiBPP!=bpp) &&
          (pModeInfoFound->uiBPP<pModeInfo->uiBPP))
        pModeInfoFound = pModeInfo;
    }
    pModeInfo = pModeInfo->pNext;
  }

  // If we did not find a good fullscreen mode, then try a similar
  if (!pModeInfoFound)
  {
#ifdef DEBUG_BUILD
    printf("[os2fslib_SetVideoMode] : Requested video mode not found, looking for a similar one!\n"); fflush(stdout);
#endif
    // Go through the video modes again, and find a similar resolution!
    pModeInfo = _this->hidden->pAvailableFSLibVideoModes;
    while (pModeInfo)
    {
      // Check all available fullscreen modes for this resolution
      if ((pModeInfo->uiXResolution >= width) &&
          (pModeInfo->uiYResolution >= height) &&
          (pModeInfo->uiBPP == bpp))
      {
        if (!pModeInfoFound)
          pModeInfoFound = pModeInfo;
        else
        if (((pModeInfoFound->uiXResolution-width)*(pModeInfoFound->uiYResolution-height))>
            ((pModeInfo->uiXResolution-width)*(pModeInfo->uiYResolution-height)))
        {
          // Found a mode which is closer than the current one
          pModeInfoFound = pModeInfo;
        }
      }
      pModeInfo = pModeInfo->pNext;
    }
  }

  // If we did not find a good fullscreen mode, then return NULL
  if (!pModeInfoFound)
  {
#ifdef DEBUG_BUILD
    printf("[os2fslib_SetVideoMode] : Requested video mode not found!\n"); fflush(stdout);
#endif
    return NULL;
  }

#ifdef DEBUG_BUILD
  printf("[os2fslib_SetVideoMode] : Found mode!\n"); fflush(stdout);
#endif

  // We'll possibly adjust the structure, so copy out the values
  // into TempModeInfo!
  SDL_memcpy(&TempModeInfo, pModeInfoFound, sizeof(TempModeInfo));
  pModeInfoFound = &TempModeInfo;

  if (flags & SDL_RESIZABLE)
  {
#ifdef DEBUG_BUILD
    printf("[os2fslib_SetVideoMode] : Requested mode is resizable, changing width/height\n"); fflush(stdout);
#endif
    // Change width and height to requested one!
    TempModeInfo.uiXResolution = width;
    TempModeInfo.uiYResolution = height;
    TempModeInfo.uiScanLineSize = width * ((TempModeInfo.uiBPP+7)/8);
  }

  // We can try create new surface!

  // Make sure this thread is prepared for using the Presentation Manager!
  hab = WinInitialize(0);
  hmq = WinCreateMsgQueue(hab,0);
  // Remember if there was an error at WinCreateMsgQueue(), because we don't
  // want to destroy somebody else's queue later. :)
  hmqerror = WinGetLastError(hab);

  

  if (DosRequestMutexSem(_this->hidden->hmtxUseSrcBuffer, SEM_INDEFINITE_WAIT)==NO_ERROR)
  {
#ifdef DEBUG_BUILD
    printf("[os2fslib_SetVideoMode] : Creating new SW surface\n"); fflush(stdout);
#endif

    // Create new software surface!
    pResult = SDL_CreateRGBSurface(SDL_SWSURFACE,
                                   pModeInfoFound->uiXResolution,
                                   pModeInfoFound->uiYResolution,
                                   pModeInfoFound->uiBPP,
                                   ((unsigned int) pModeInfoFound->PixelFormat.ucRedMask) << pModeInfoFound->PixelFormat.ucRedPosition,
                                   ((unsigned int) pModeInfoFound->PixelFormat.ucGreenMask) << pModeInfoFound->PixelFormat.ucGreenPosition,
                                   ((unsigned int) pModeInfoFound->PixelFormat.ucBlueMask) << pModeInfoFound->PixelFormat.ucBluePosition,
                                   ((unsigned int) pModeInfoFound->PixelFormat.ucAlphaMask) << pModeInfoFound->PixelFormat.ucAlphaPosition);

    if (pResult == NULL)
    {
      DosReleaseMutexSem(_this->hidden->hmtxUseSrcBuffer);
      SDL_OutOfMemory();
      return NULL;
    }

#ifdef DEBUG_BUILD
    printf("[os2fslib_SetVideoMode] : Adjusting pixel format\n"); fflush(stdout);
#endif

    // Adjust pixel format mask!
    pResult->format->Rmask = ((unsigned int) pModeInfoFound->PixelFormat.ucRedMask) << pModeInfoFound->PixelFormat.ucRedPosition;
    pResult->format->Rshift = pModeInfoFound->PixelFormat.ucRedPosition;
    pResult->format->Rloss = pModeInfoFound->PixelFormat.ucRedAdjust;
    pResult->format->Gmask = ((unsigned int) pModeInfoFound->PixelFormat.ucGreenMask) << pModeInfoFound->PixelFormat.ucGreenPosition;
    pResult->format->Gshift = pModeInfoFound->PixelFormat.ucGreenPosition;
    pResult->format->Gloss = pModeInfoFound->PixelFormat.ucGreenAdjust;
    pResult->format->Bmask = ((unsigned int) pModeInfoFound->PixelFormat.ucBlueMask) << pModeInfoFound->PixelFormat.ucBluePosition;
    pResult->format->Bshift = pModeInfoFound->PixelFormat.ucBluePosition;
    pResult->format->Bloss = pModeInfoFound->PixelFormat.ucBlueAdjust;
    pResult->format->Amask = ((unsigned int) pModeInfoFound->PixelFormat.ucAlphaMask) << pModeInfoFound->PixelFormat.ucAlphaPosition;
    pResult->format->Ashift = pModeInfoFound->PixelFormat.ucAlphaPosition;
    pResult->format->Aloss = pModeInfoFound->PixelFormat.ucAlphaAdjust;

#ifdef REPORT_EMPTY_ALPHA_MASK
    pResult->format->Amask =
        pResult->format->Ashift =
        pResult->format->Aloss = 0;
#endif

    // Adjust surface flags
    pResult->flags |= (flags & SDL_FULLSCREEN);
    pResult->flags |= (flags & SDL_RESIZABLE);

    // It might be that the software surface pitch is not the same as
    // the pitch we have, so adjust that!
    pModeInfoFound->uiScanLineSize = pResult->pitch;

    // Store new source buffer parameters!
    SDL_memcpy(&(_this->hidden->SrcBufferDesc), pModeInfoFound, sizeof(*pModeInfoFound));
    _this->hidden->pchSrcBuffer = pResult->pixels;

#ifdef DEBUG_BUILD
    printf("[os2fslib_SetVideoMode] : Telling FSLib the stuffs\n"); fflush(stdout);
#endif

    // Tell the FSLib window the new source image format
    FSLib_SetSrcBufferDesc(_this->hidden->hwndClient, &(_this->hidden->SrcBufferDesc));

    if (
        ((flags & SDL_RESIZABLE)==0) ||
        (bFirstCall)
       )
    {
      bFirstCall = 0;
#ifdef DEBUG_BUILD
      printf("[os2fslib_SetVideoMode] : Modifying window size\n"); fflush(stdout);
#endif

      // Calculate frame window size from client window size
      rectl.xLeft = 0;
      rectl.yBottom = 0;
      rectl.xRight = pModeInfoFound->uiXResolution; // Noninclusive
      rectl.yTop = pModeInfoFound->uiYResolution; // Noninclusive
      WinCalcFrameRect(_this->hidden->hwndFrame, &rectl, FALSE);

      // Set the new size of the main window
      SetAccessableWindowPos(_this->hidden->hwndFrame,
                             HWND_TOP,
                             0, 0,
                             (rectl.xRight-rectl.xLeft),
                             (rectl.yTop-rectl.yBottom),
                             SWP_SIZE | SWP_ACTIVATE | SWP_SHOW);
    }

    // Set fullscreen mode flag, and switch to fullscreen if needed!
    if (flags & SDL_FULLSCREEN)
    {
#ifdef DEBUG_BUILD
      printf("[os2fslib_SetVideoMode] : Also trying to switch to fullscreen\n");
      fflush(stdout);
#endif
      FSLib_ToggleFSMode(_this->hidden->hwndClient, 1);
      /* Cursor manager functions to FS mode*/
      os2fslib_SetCursorManagementFunctions(_this, 0);
    } else
    {
#ifdef DEBUG_BUILD
      printf("[os2fslib_SetVideoMode] : Also trying to switch to desktop mode\n");
      fflush(stdout);
#endif
      FSLib_ToggleFSMode(_this->hidden->hwndClient, 0);
      /* Cursor manager functions to Windowed mode*/
      os2fslib_SetCursorManagementFunctions(_this, 1);
    }

    _this->hidden->pSDLSurface = pResult;

    DosReleaseMutexSem(_this->hidden->hmtxUseSrcBuffer);
  } else
  {
#ifdef DEBUG_BUILD
    printf("[os2fslib_SetVideoMode] : Could not get hmtxUseSrcBuffer!\n"); fflush(stdout);
#endif
    
    pResult = NULL;
  }

  // As we have the new surface, we don't need the current one anymore!
  if ((pResult) && (current))
  {
#ifdef DEBUG_BUILD
    printf("[os2fslib_SetVideoMode] : Freeing old surface\n"); fflush(stdout);
#endif
    SDL_FreeSurface(current);
  }

  // Redraw window
  WinInvalidateRegion(_this->hidden->hwndClient, NULL, TRUE);

  // Now destroy the message queue, if we've created it!
  if (ERRORIDERROR(hmqerror)==0)
  {
#ifdef DEBUG_BUILD
    printf("[os2fslib_SetVideoMode] : Destroying message queue\n"); fflush(stdout);
#endif
    WinDestroyMsgQueue(hmq);
  }

#ifdef DEBUG_BUILD
  printf("[os2fslib_SetVideoMode] : Done\n"); fflush(stdout);
#endif

  /* We're done */

  // Return with the new surface!
  return pResult;
}

/* List the available video modes for the given pixel format, sorted
 from largest to smallest.
 */
static SDL_Rect **os2fslib_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags)
{
#ifdef DEBUG_BUILD
  printf("[os2fslib_ListModes] : ListModes of %d Bpp\n", format->BitsPerPixel);
#endif
  // Destroy result of previous call, if there is any
  if (_this->hidden->pListModesResult)
  {
    SDL_free(_this->hidden->pListModesResult); _this->hidden->pListModesResult = NULL;
  }

  // For resizable and windowed mode we support every resolution!
  if ((flags & SDL_RESIZABLE) && ((flags & SDL_FULLSCREEN) == 0))
    return (SDL_Rect **)-1;

  // Check if they need fullscreen or non-fullscreen video modes!
  if ((flags & SDL_FULLSCREEN) == 0)

  {
    // For windowed mode we support every resolution!
    return (SDL_Rect **)-1;
  } else
  {
    FSLib_VideoMode_p pFSMode;
    // For fullscreen mode we don't support every resolution!
    // Now create a new list
    pFSMode = _this->hidden->pAvailableFSLibVideoModes;
    while (pFSMode)
    {
      if (pFSMode->uiBPP == format->BitsPerPixel)
      {
        SDL_Rect *pRect = (SDL_Rect *) SDL_malloc(sizeof(SDL_Rect));
        if (pRect)
        {
          // Fill description
          pRect->x = 0;
          pRect->y = 0;
          pRect->w = pFSMode->uiXResolution;
          pRect->h = pFSMode->uiYResolution;
#ifdef DEBUG_BUILD
//          printf("!!! Seems to be good!\n");
//        printf("F: %dx%d\n", pRect->w, pRect->h);
#endif
          // And insert into list of pRects
          if (!(_this->hidden->pListModesResult))
          {
#ifdef DEBUG_BUILD
//            printf("!!! Inserting to beginning\n");
#endif

            // We're the first one to be inserted!
            _this->hidden->pListModesResult = (SDL_Rect**) SDL_malloc(2*sizeof(SDL_Rect*));
            if (_this->hidden->pListModesResult)
            {
              _this->hidden->pListModesResult[0] = pRect;
              _this->hidden->pListModesResult[1] = NULL;
            } else
            {
              SDL_free(pRect);
            }
          } else
          {
            // We're not the first ones, so find the place where we
            // have to insert ourselves
            SDL_Rect **pNewList;
            int iPlace, iNumOfSlots, i;

#ifdef DEBUG_BUILD
//            printf("!!! Searching where to insert\n");
#endif

            iPlace = -1; iNumOfSlots = 1; // Count the last NULL too!
            for (i=0; _this->hidden->pListModesResult[i]; i++)
            {
              iNumOfSlots++;
              if (iPlace==-1)
              {
                if ((_this->hidden->pListModesResult[i]->w*_this->hidden->pListModesResult[i]->h)<
                    (pRect->w*pRect->h))
                {
                  iPlace = i;
                }
              }
            }
            if (iPlace==-1) iPlace = iNumOfSlots-1;

#ifdef DEBUG_BUILD
//            printf("!!! From %d slots, it will be at %d\n", iNumOfSlots, iPlace);
#endif

            pNewList = (SDL_Rect**) SDL_realloc(_this->hidden->pListModesResult, (iNumOfSlots+1)*sizeof(SDL_Rect*));
            if (pNewList)
            {
              for (i=iNumOfSlots;i>iPlace;i--)
                pNewList[i] = pNewList[i-1];
              pNewList[iPlace] = pRect;
              _this->hidden->pListModesResult = pNewList;
            } else
            {
              SDL_free(pRect);
            }
          }
        }
      }
      pFSMode = pFSMode->pNext;
    }
  }
#ifdef DEBUG_BUILD
//  printf("Returning list\n");
#endif
  return _this->hidden->pListModesResult;
}

/* Initialize the native video subsystem, filling 'vformat' with the
 "best" display pixel format, returning 0 or -1 if there's an error.
 */
static int os2fslib_VideoInit(_THIS, SDL_PixelFormat *vformat)
{
  FSLib_VideoMode_p pDesktopMode;

#ifdef DEBUG_BUILD
  printf("[os2fslib_VideoInit] : Enter\n"); fflush(stdout);
#endif

  // Report the best pixel format. For this,
  // we'll use the current desktop format.
  pDesktopMode = FSLib_GetDesktopVideoMode();
  if (!pDesktopMode)
  {
    SDL_SetError("Could not query desktop video mode!");
#ifdef DEBUG_BUILD
    printf("[os2fslib_VideoInit] : Could not query desktop video mode!\n");
#endif
    return -1;
  }

  /* Determine the current screen size */
  _this->info.current_w = pDesktopMode->uiXResolution;
  _this->info.current_h = pDesktopMode->uiYResolution;

  /* Determine the screen depth */
  vformat->BitsPerPixel = pDesktopMode->uiBPP;
  vformat->BytesPerPixel = (vformat->BitsPerPixel+7)/8;

  vformat->Rmask = ((unsigned int) pDesktopMode->PixelFormat.ucRedMask) << pDesktopMode->PixelFormat.ucRedPosition;
  vformat->Rshift = pDesktopMode->PixelFormat.ucRedPosition;
  vformat->Rloss = pDesktopMode->PixelFormat.ucRedAdjust;
  vformat->Gmask = ((unsigned int) pDesktopMode->PixelFormat.ucGreenMask) << pDesktopMode->PixelFormat.ucGreenPosition;
  vformat->Gshift = pDesktopMode->PixelFormat.ucGreenPosition;
  vformat->Gloss = pDesktopMode->PixelFormat.ucGreenAdjust;
  vformat->Bmask = ((unsigned int) pDesktopMode->PixelFormat.ucBlueMask) << pDesktopMode->PixelFormat.ucBluePosition;
  vformat->Bshift = pDesktopMode->PixelFormat.ucBluePosition;
  vformat->Bloss = pDesktopMode->PixelFormat.ucBlueAdjust;
  vformat->Amask = ((unsigned int) pDesktopMode->PixelFormat.ucAlphaMask) << pDesktopMode->PixelFormat.ucAlphaPosition;
  vformat->Ashift = pDesktopMode->PixelFormat.ucAlphaPosition;
  vformat->Aloss = pDesktopMode->PixelFormat.ucAlphaAdjust;

#ifdef REPORT_EMPTY_ALPHA_MASK
  vformat->Amask =
      vformat->Ashift =
      vformat->Aloss = 0;
#endif

  // Fill in some window manager capabilities
  _this->info.wm_available = 1;

  // Initialize some internal variables
  _this->hidden->pListModesResult = NULL;
  _this->hidden->fInFocus = 0;
  _this->hidden->iSkipWMMOUSEMOVE = 0;
  _this->hidden->iMouseVisible = 1;

  if (getenv("SDL_USE_PROPORTIONAL_WINDOW"))
    _this->hidden->bProportionalResize = 1;
  else
  {
    PPIB pib;
    PTIB tib;
    char *pchFileName, *pchTemp;
    char achConfigFile[CCHMAXPATH];
    FILE *hFile;

    /* No environment variable to have proportional window.
     * Ok, let's check if this executable is in config file!
     */
    _this->hidden->bProportionalResize = 0;

    DosGetInfoBlocks(&tib, &pib);
    pchTemp = pchFileName = pib->pib_pchcmd;
    while (*pchTemp)
    {
      if (*pchTemp=='\\')
        pchFileName = pchTemp+1;
      pchTemp++;
    }
    if (getenv("HOME"))
    {
      sprintf(achConfigFile, "%s\\.sdl.proportionals", getenv("HOME"));
      hFile = fopen(achConfigFile, "rt");
      if (!hFile)
      {
        /* Seems like the file cannot be opened or does not exist.
         * Let's try to create it with defaults!
         */
        hFile = fopen(achConfigFile, "wt");
        if (hFile)
        {
          fprintf(hFile, "; This file is a config file of SDL/2, containing\n");
          fprintf(hFile, "; the list of executables that must have proportional\n");
          fprintf(hFile, "; windows.\n");
          fprintf(hFile, ";\n");
          fprintf(hFile, "; You can add executable filenames into this file,\n");
          fprintf(hFile, "; one under the other. If SDL finds that a given\n");
          fprintf(hFile, "; program is in this list, then that application\n");
          fprintf(hFile, "; will have proportional windows, just like if\n");
          fprintf(hFile, "; the SET SDL_USE_PROPORTIONAL_WINDOW env. variable\n");
          fprintf(hFile, "; would have been set for that process.\n");
          fprintf(hFile, ";\n");
          fprintf(hFile, "\n");
          fprintf(hFile, "dosbox.exe\n");
          fclose(hFile);
        }

        hFile = fopen(achConfigFile, "rt");
      }

      if (hFile)
      {
        while (fgets(achConfigFile, sizeof(achConfigFile), hFile))
        {
          /* Cut \n from end of string */

          while (achConfigFile[strlen(achConfigFile)-1] == '\n')
            achConfigFile[strlen(achConfigFile)-1] = 0;

          /* Compare... */
          if (stricmp(achConfigFile, pchFileName)==0)
          {
            /* Found it in config file! */
            _this->hidden->bProportionalResize = 1;
            break;
          }
        }
        fclose(hFile);
      }
    }
  }

  DosCreateMutexSem(NULL, &(_this->hidden->hmtxUseSrcBuffer), 0, FALSE);

  // Now create our window with a default size

  // For this, we select the first available fullscreen mode as
  // current window size!
  SDL_memcpy(&(_this->hidden->SrcBufferDesc), _this->hidden->pAvailableFSLibVideoModes, sizeof(_this->hidden->SrcBufferDesc));
  // Allocate new video buffer!
  _this->hidden->pchSrcBuffer = (char *) SDL_malloc(_this->hidden->pAvailableFSLibVideoModes->uiScanLineSize * _this->hidden->pAvailableFSLibVideoModes->uiYResolution);
  if (!_this->hidden->pchSrcBuffer)
  {
#ifdef DEBUG_BUILD
    printf("[os2fslib_VideoInit] : Yikes, not enough memory for new video buffer!\n"); fflush(stdout);
#endif
    SDL_SetError("Not enough memory for new video buffer!\n");
    return -1;
  }

  // For this, we need a message processing thread.
  // We'll create a new thread for this, which will do everything
  // what is related to PM
  _this->hidden->iPMThreadStatus = 0;
  _this->hidden->tidPMThread = _beginthread(PMThreadFunc, NULL, 65536, (void *) _this);
  if (_this->hidden->tidPMThread <= 0)
  {
#ifdef DEBUG_BUILD
    printf("[os2fslib_VideoInit] : Could not create PM thread!\n");
#endif
    SDL_SetError("Could not create PM thread");
    return -1;
  }
#ifdef USE_DOSSETPRIORITY
  // Burst the priority of PM Thread!
  DosSetPriority(PRTYS_THREAD, PRTYC_TIMECRITICAL, 0, _this->hidden->tidPMThread);
#endif
  // Wait for the PM thread to initialize!
  while (_this->hidden->iPMThreadStatus==0)
    DosSleep(32);
  // If the PM thread could not set up everything, then
  // report an error!
  if (_this->hidden->iPMThreadStatus!=1)
  {
#ifdef DEBUG_BUILD
    printf("[os2fslib_VideoInit] : PMThread reported an error : %d\n", _this->hidden->iPMThreadStatus);
#endif
    SDL_SetError("Error initializing PM thread");
    return -1;
  }

  return 0;
}


static void os2fslib_DeleteDevice(_THIS)
{
#ifdef DEBUG_BUILD
  printf("[os2fslib_DeleteDevice]\n"); fflush(stdout);
#endif
  // Free used memory
  FSLib_FreeVideoModeList(_this->hidden->pAvailableFSLibVideoModes);
  if (_this->hidden->pListModesResult)
    SDL_free(_this->hidden->pListModesResult);
  if (_this->hidden->pchSrcBuffer)
    SDL_free(_this->hidden->pchSrcBuffer);
  DosCloseMutexSem(_this->hidden->hmtxUseSrcBuffer);
  SDL_free(_this->hidden);
  SDL_free(_this);
  FSLib_Uninitialize();
}

static int os2fslib_Available(void)
{

  // If we can run, it means that we could load FSLib,
  // so we assume that it's available then!
  return 1;
}

static void os2fslib_MorphToPM()
{
  PPIB pib;
  PTIB tib;

  DosGetInfoBlocks(&tib, &pib);

  // Change flag from VIO to PM:
  if (pib->pib_ultype==2) pib->pib_ultype = 3;
}

static SDL_VideoDevice *os2fslib_CreateDevice(int devindex)
{
  SDL_VideoDevice *device;

#ifdef DEBUG_BUILD
  printf("[os2fslib_CreateDevice] : Enter\n"); fflush(stdout);
#endif

  /* Initialize all variables that we clean on shutdown */
  device = (SDL_VideoDevice *)SDL_malloc(sizeof(SDL_VideoDevice));
  if ( device )
  {
    SDL_memset(device, 0, (sizeof *device));
    // Also allocate memory for private data
    device->hidden = (struct SDL_PrivateVideoData *) SDL_malloc((sizeof(struct SDL_PrivateVideoData)));
  }
  if ( (device == NULL) || (device->hidden == NULL) )
  {
    SDL_OutOfMemory();
    if ( device )
      SDL_free(device);
    return NULL;
  }
  SDL_memset(device->hidden, 0, (sizeof *device->hidden));

  /* Set the function pointers */
#ifdef DEBUG_BUILD
  printf("[os2fslib_CreateDevice] : VideoInit is %p\n", os2fslib_VideoInit); fflush(stdout);
#endif

  /* Initialization/Query functions */
  device->VideoInit = os2fslib_VideoInit;
  device->ListModes = os2fslib_ListModes;
  device->SetVideoMode = os2fslib_SetVideoMode;
  device->ToggleFullScreen = os2fslib_ToggleFullScreen;
  device->UpdateMouse = os2fslib_UpdateMouse;
  device->CreateYUVOverlay = NULL;
  device->SetColors = os2fslib_SetColors;
  device->UpdateRects = os2fslib_UpdateRects;
  device->VideoQuit = os2fslib_VideoQuit;
  /* Hardware acceleration functions */
  device->AllocHWSurface = os2fslib_AllocHWSurface;
  device->CheckHWBlit = NULL;
  device->FillHWRect = NULL;
  device->SetHWColorKey = NULL;
  device->SetHWAlpha = NULL;
  device->LockHWSurface = os2fslib_LockHWSurface;
  device->UnlockHWSurface = os2fslib_UnlockHWSurface;
  device->FlipHWSurface = NULL;
  device->FreeHWSurface = os2fslib_FreeHWSurface;
  /* Window manager functions */
  device->SetCaption = os2fslib_SetCaption;
  device->SetIcon = os2fslib_SetIcon;
  device->IconifyWindow = os2fslib_IconifyWindow;
  device->GrabInput = os2fslib_GrabInput;
  device->GetWMInfo = NULL;
  /* Cursor manager functions to Windowed mode*/
  os2fslib_SetCursorManagementFunctions(device, 1);
  /* Event manager functions */
  device->InitOSKeymap = os2fslib_InitOSKeymap;
  device->PumpEvents = os2fslib_PumpEvents;
  /* The function used to dispose of this structure */
  device->free = os2fslib_DeleteDevice;

  // Make sure we'll be able to use Win* API even if the application
  // was linked to be a VIO application!
  os2fslib_MorphToPM();

  // Now initialize FSLib, and query available video modes!
  if (!FSLib_Initialize())
  {
    // Could not initialize FSLib!
#ifdef DEBUG_BUILD
    printf("[os2fslib_CreateDevice] : Could not initialize FSLib!\n");
#endif
    SDL_SetError("Could not initialize FSLib!");
    SDL_free(device->hidden);
    SDL_free(device);
    return NULL;
  }
  device->hidden->pAvailableFSLibVideoModes =
    FSLib_GetVideoModeList();

  return device;
}

VideoBootStrap OS2FSLib_bootstrap = {
        "os2fslib", "OS/2 Video Output using FSLib",
        os2fslib_Available, os2fslib_CreateDevice
};

