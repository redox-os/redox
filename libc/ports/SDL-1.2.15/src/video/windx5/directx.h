
#ifndef _directx_h
#define _directx_h

/* Include all of the DirectX 5.0 headers and adds any necessary tweaks */

#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <mmsystem.h>
#ifndef WIN32
#define WIN32
#endif
#undef  WINNT

/* Far pointers don't exist in 32-bit code */
#ifndef FAR
#define FAR
#endif

/* Error codes not yet included in Win32 API header files */
#ifndef MAKE_HRESULT
#define MAKE_HRESULT(sev,fac,code) \
	((HRESULT)(((unsigned long)(sev)<<31) | ((unsigned long)(fac)<<16) | ((unsigned long)(code))))
#endif

#ifndef S_OK
#define S_OK		(HRESULT)0x00000000L
#endif

#ifndef SUCCEEDED
#define SUCCEEDED(x)	((HRESULT)(x) >= 0)
#endif
#ifndef FAILED
#define FAILED(x)	((HRESULT)(x)<0)
#endif

#ifndef E_FAIL
#define E_FAIL		(HRESULT)0x80000008L
#endif
#ifndef E_NOINTERFACE
#define E_NOINTERFACE	(HRESULT)0x80004002L
#endif
#ifndef E_OUTOFMEMORY
#define E_OUTOFMEMORY	(HRESULT)0x8007000EL
#endif
#ifndef E_INVALIDARG
#define E_INVALIDARG	(HRESULT)0x80070057L
#endif
#ifndef E_NOTIMPL
#define E_NOTIMPL	(HRESULT)0x80004001L
#endif
#ifndef REGDB_E_CLASSNOTREG
#define REGDB_E_CLASSNOTREG	(HRESULT)0x80040154L
#endif

/* Severity codes */
#ifndef SEVERITY_ERROR
#define SEVERITY_ERROR	1
#endif

/* Error facility codes */
#ifndef FACILITY_WIN32
#define FACILITY_WIN32	7
#endif

#ifndef FIELD_OFFSET
#define FIELD_OFFSET(type, field)    ((LONG)&(((type *)0)->field))
#endif

/* DirectX headers (if it isn't included, I haven't tested it yet)
 */
/* We need these defines to mark what version of DirectX API we use */
#define DIRECTDRAW_VERSION  0x0700
#define DIRECTSOUND_VERSION 0x0500
#define DIRECTINPUT_VERSION 0x0700

#include <ddraw.h>
#include <dsound.h>
#include <dinput.h>

#if DIRECTINPUT_VERSION >= 0x0700 && !defined(DIMOFS_BUTTON4)
typedef struct _DIMOUSESTATE2 {
    LONG    lX;
    LONG    lY;
    LONG    lZ;
    BYTE    rgbButtons[8];
} DIMOUSESTATE2, *LPDIMOUSESTATE2;

#define DIMOFS_BUTTON4 (FIELD_OFFSET(DIMOUSESTATE2, rgbButtons) + 4)
#define DIMOFS_BUTTON5 (FIELD_OFFSET(DIMOUSESTATE2, rgbButtons) + 5)
#define DIMOFS_BUTTON6 (FIELD_OFFSET(DIMOUSESTATE2, rgbButtons) + 6)
#define DIMOFS_BUTTON7 (FIELD_OFFSET(DIMOUSESTATE2, rgbButtons) + 7)

extern const DIDATAFORMAT c_dfDIMouse2;
#endif

#endif /* _directx_h */
