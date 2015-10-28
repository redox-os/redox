/* File "FastTimes.c" - Original code by Matt Slot <fprefect@ambrosiasw.com>  */
/* Created 4/24/99    - This file is hereby placed in the public domain       */
/* Updated 5/21/99    - Calibrate to VIA, add TBR support, renamed functions  */
/* Updated 10/4/99    - Use AbsoluteToNanoseconds() in case Absolute = double */
/* Updated 2/15/00    - Check for native Time Manager, no need to calibrate   */
/* Updated 2/19/00    - Fixed default value for gScale under native Time Mgr  */
/* Updated 3/21/00    - Fixed ns conversion, create 2 different scale factors */
/* Updated 5/03/00    - Added copyright and placed into PD. No code changes   */
/* Updated 8/01/00    - Made "Carbon-compatible" by replacing LMGetTicks()    */

/* This file is Copyright (C) Matt Slot, 1999-2012. It is hereby placed into 
   the public domain. The author makes no warranty as to fitness or stability */

#include <Gestalt.h>
#include <LowMem.h>
#include <CodeFragments.h>
#include <DriverServices.h>
#include <Timer.h>

#include "FastTimes.h"

#ifdef TARGET_CPU_PPC
#undef GENERATINGPOWERPC /* stop whining */
#define GENERATINGPOWERPC TARGET_CPU_PPC
#endif

/* **** **** **** **** **** **** **** **** **** **** **** **** **** **** **** */
/* **** **** **** **** **** **** **** **** **** **** **** **** **** **** **** */
/*
	On 680x0 machines, we just use Microseconds().
	
	On PowerPC machines, we try several methods:
	  * DriverServicesLib is available on all PCI PowerMacs, and perhaps
	    some NuBus PowerMacs. If it is, we use UpTime() : Overhead = 2.1 탎ec.
	  * The PowerPC 601 has a built-in "real time clock" RTC, and we fall
	    back to that, accessing it directly from asm. Overhead = 1.3 탎ec.
	  * Later PowerPCs have an accurate "time base register" TBR, and we 
	    fall back to that, access it from PowerPC asm. Overhead = 1.3 탎ec.
	  * We can also try Microseconds() which is emulated : Overhead = 36 탎ec.

	On PowerPC machines, we avoid the following:
	  * OpenTransport is available on all PCI and some NuBus PowerMacs, but it
	    uses UpTime() if available and falls back to Microseconds() otherwise.
	  * InputSprocket is available on many PowerMacs, but again it uses
	    UpTime() if available and falls back to Microseconds() otherwise.

	Another PowerPC note: certain configurations, especially 3rd party upgrade
	cards, may return inaccurate timings for the CPU or memory bus -- causing
	skew in various system routines (up to 20% drift!). The VIA chip is very
	accurate, and it's the basis for the Time Manager and Microseconds().
	Unfortunately, it's also very slow because the MacOS has to (a) switch to
	68K and (b) poll for a VIA event.
	
	We compensate for the drift by calibrating a floating point scale factor
	between our fast method and the accurate timer at startup, then convert
	each sample quickly on the fly. I'd rather not have the initialization 
	overhead -- but it's simply necessary for accurate timing. You can drop
	it down to 30 ticks if you prefer, but that's as low as I'd recommend.

	Under MacOS 9, "new world" Macs (iMacs, B+W G3s and G+W G4s) have a native
	Time Manager implementation: UpTime(), Microseconds(), and TickCount() are
	all based on the same underlying counter. This makes it silly to calibrate
	UpTime() against TickCount(). We now check for this feature using Gestalt(),
	and skip the whole calibration step if possible.

*/
/* **** **** **** **** **** **** **** **** **** **** **** **** **** **** **** */
/* **** **** **** **** **** **** **** **** **** **** **** **** **** **** **** */

#define RTCToNano(w)	((double) (w).hi * 1000000000.0 + (double) (w).lo)
#define WideTo64bit(w)	(*(UInt64 *) &(w))

/* LMGetTicks() is not in Carbon and TickCount() has a fair bit of overhead,
   so for speed we always read lowmem directly. This is a Mac OS X no-no, but 
   it always work on those systems that don't have a native Time Manager (ie,
   anything before MacOS 9) -- regardless whether we are in Carbon or not! */
#define MyLMGetTicks()	(*(volatile UInt32 *) 0x16A)

#if GENERATINGPOWERPC

static asm UnsignedWide PollRTC(void);
static asm UnsignedWide PollTBR(void);
static Ptr FindFunctionInSharedLib(StringPtr libName, StringPtr funcName);

static Boolean			gInited = false;
static Boolean			gNative = false;
static Boolean			gUseRTC = false;
static Boolean			gUseTBR = false;
static double			gScaleUSec = 1.0 / 1000.0;    /* 1 / ( nsec / usec) */
static double			gScaleMSec = 1.0 / 1000000.0; /* 1 / ( nsec / msec) */

/* Functions loaded from DriverServicesLib */
typedef AbsoluteTime 	(*UpTimeProcPtr)(void);
typedef Nanoseconds 	(*A2NSProcPtr)(AbsoluteTime);
static UpTimeProcPtr 	gUpTime = NULL;
static A2NSProcPtr 		gA2NS = NULL;

#endif /* GENERATINGPOWERPC */

/* **** **** **** **** **** **** **** **** **** **** **** **** **** **** **** */
/* **** **** **** **** **** **** **** **** **** **** **** **** **** **** **** */

void FastInitialize() {
	SInt32			result;

	if (!gInited) {

#if GENERATINGPOWERPC

		/* Initialize the feature flags */
		gNative = gUseRTC = gUseTBR = false;

		/* We use CFM to find and load needed symbols from shared libraries, so
		   the application doesn't have to weak-link them, for convenience.   */
		gUpTime = (UpTimeProcPtr) FindFunctionInSharedLib(
				"\pDriverServicesLib", "\pUpTime");
		if (gUpTime) gA2NS = (A2NSProcPtr) FindFunctionInSharedLib(
				"\pDriverServicesLib", "\pAbsoluteToNanoseconds");
		if (!gA2NS) gUpTime = nil; /* Pedantic but necessary */

		if (gUpTime) {
			/* If we loaded UpTime(), then we need to know if the system has
			   a native implementation of the Time Manager. If so, then it's
			   pointless to calculate a scale factor against the missing VIA */

			/* gestaltNativeTimeMgr = 4 in some future version of the headers */
			if (!Gestalt(gestaltTimeMgrVersion, &result) &&
					(result > gestaltExtendedTimeMgr)) 
				gNative = true;
			}
		  else {
			/* If no DriverServicesLib, use Gestalt() to get the processor type. 
			   Only NuBus PowerMacs with old System Software won't have DSL, so
			   we know it should either be a 601 or 603. */

			/* Use the processor gestalt to determine which register to use */
		 	if (!Gestalt(gestaltNativeCPUtype, &result)) {
				if (result == gestaltCPU601) gUseRTC = true;
				  else if (result > gestaltCPU601) gUseTBR = true;
				}
			}

		/* Now calculate a scale factor to keep us accurate. */
		if ((gUpTime && !gNative) || gUseRTC || gUseTBR) {
			UInt64			tick, usec1, usec2;
			UnsignedWide	wide;

			/* Wait for the beginning of the very next tick */
			for(tick = MyLMGetTicks() + 1; tick > MyLMGetTicks(); );
			
			/* Poll the selected timer and prepare it (since we have time) */
			wide = (gUpTime) ? (*gA2NS)((*gUpTime)()) : 
					((gUseRTC) ? PollRTC() : PollTBR());
			usec1 = (gUseRTC) ? RTCToNano(wide) : WideTo64bit(wide);
			
			/* Wait for the exact 60th tick to roll over */
			while(tick + 60 > MyLMGetTicks());

			/* Poll the selected timer again and prepare it  */
			wide = (gUpTime) ? (*gA2NS)((*gUpTime)()) : 
					((gUseRTC) ? PollRTC() : PollTBR());
			usec2 = (gUseRTC) ? RTCToNano(wide) : WideTo64bit(wide);
			
			/* Calculate a scale value that will give microseconds per second.
			   Remember, there are actually 60.15 ticks in a second, not 60.  */
			gScaleUSec = (60.0 * 1000000.0) / ((usec2 - usec1) * 60.15);
			gScaleMSec = gScaleUSec / 1000.0;
			}

#endif /* GENERATINGPOWERPC */

		/* We've initialized our globals */
		gInited = true;
		}
	}

/* **** **** **** **** **** **** **** **** **** **** **** **** **** **** **** */
/* **** **** **** **** **** **** **** **** **** **** **** **** **** **** **** */

UInt64 FastMicroseconds() {
	UnsignedWide	wide;
	UInt64			usec;
	
#if GENERATINGPOWERPC
	/* Initialize globals the first time we are called */
	if (!gInited) FastInitialize();
	
	if (gNative) {
		/* Use DriverServices if it's available -- it's fast and compatible */
		wide = (*gA2NS)((*gUpTime)());
		usec = (double) WideTo64bit(wide) * gScaleUSec + 0.5;
		}
	  else if (gUpTime) {
		/* Use DriverServices if it's available -- it's fast and compatible */
		wide = (*gA2NS)((*gUpTime)());
		usec = (double) WideTo64bit(wide) * gScaleUSec + 0.5;
		}
	  else if (gUseTBR) {
		/* On a recent PowerPC, we poll the TBR directly */
		wide = PollTBR();
		usec = (double) WideTo64bit(wide) * gScaleUSec + 0.5;
		}
	  else if (gUseRTC) {
		/* On a 601, we can poll the RTC instead */
		wide = PollRTC();
		usec = (double) RTCToNano(wide) * gScaleUSec + 0.5;
		}
	  else 
#endif /* GENERATINGPOWERPC */
		{
		/* If all else fails, suffer the mixed mode overhead */
		Microseconds(&wide);
		usec = WideTo64bit(wide);
		}

	return(usec);
	}

/* **** **** **** **** **** **** **** **** **** **** **** **** **** **** **** */
/* **** **** **** **** **** **** **** **** **** **** **** **** **** **** **** */

UInt64 FastMilliseconds() {
	UnsignedWide	wide;
	UInt64			msec;	
	
#if GENERATINGPOWERPC
	/* Initialize globals the first time we are called */
	if (!gInited) FastInitialize();
	
	if (gNative) {
		/* Use DriverServices if it's available -- it's fast and compatible */
		wide = (*gA2NS)((*gUpTime)());
		msec = (double) WideTo64bit(wide) * gScaleMSec + 0.5;
		}
	  else if (gUpTime) {
		/* Use DriverServices if it's available -- it's fast and compatible */
		wide = (*gA2NS)((*gUpTime)());
		msec = (double) WideTo64bit(wide) * gScaleMSec + 0.5;
		}
	  else if (gUseTBR) {
		/* On a recent PowerPC, we poll the TBR directly */
		wide = PollTBR();
		msec = (double) WideTo64bit(wide) * gScaleMSec + 0.5;
		}
	  else if (gUseRTC) {
		/* On a 601, we can poll the RTC instead */
		wide = PollRTC();
		msec = (double) RTCToNano(wide) * gScaleMSec + 0.5;
		}
	  else 
#endif /* GENERATINGPOWERPC */
		{
		/* If all else fails, suffer the mixed mode overhead */
		Microseconds(&wide);
		msec = ((double) WideTo64bit(wide) + 500.0) / 1000.0;
		}

	return(msec);
	}

/* **** **** **** **** **** **** **** **** **** **** **** **** **** **** **** */
/* **** **** **** **** **** **** **** **** **** **** **** **** **** **** **** */

StringPtr FastMethod() {
	StringPtr	method = "\p<Unknown>";

#if GENERATINGPOWERPC
	/* Initialize globals the first time we are called */
	if (!gInited) FastInitialize();
	
	if (gNative) {
		/* The Time Manager and UpTime() are entirely native on this machine */
		method = "\pNative UpTime()";
		}
	  else if (gUpTime) {
		/* Use DriverServices if it's available -- it's fast and compatible */
		method = "\pUpTime()";
		}
	  else if (gUseTBR) {
		/* On a recent PowerPC, we poll the TBR directly */
		method = "\pPowerPC TBR";
		}
	  else if (gUseRTC) {
		/* On a 601, we can poll the RTC instead */
		method = "\pPowerPC RTC";
		}
	  else 
#endif /* GENERATINGPOWERPC */
		{
		/* If all else fails, suffer the mixed mode overhead */
		method = "\pMicroseconds()";
		}

	return(method);
	}

/* **** **** **** **** **** **** **** **** **** **** **** **** **** **** **** */
/* **** **** **** **** **** **** **** **** **** **** **** **** **** **** **** */
#pragma mark -

#if GENERATINGPOWERPC
asm static UnsignedWide PollRTC_() {
entry PollRTC /* Avoid CodeWarrior glue */
	machine 601
@AGAIN:
	mfrtcu	r4 /* RTCU = SPR 4 */
	mfrtcl	r5 /* RTCL = SPR 5 */
	mfrtcu	r6
	cmpw	r4,r6
	bne		@AGAIN
	stw		r4,0(r3)
	stw		r5,4(r3)
	blr
	}

/* **** **** **** **** **** **** **** **** **** **** **** **** **** **** **** */
/* **** **** **** **** **** **** **** **** **** **** **** **** **** **** **** */

asm static UnsignedWide PollTBR_() {
entry PollTBR /* Avoid CodeWarrior glue */
	machine 604
@AGAIN:
	mftbu	r4 /* TBRU = SPR 268 */
	mftb	r5 /* TBRL = SPR 269 */
	mftbu	r6
	cmpw	r4,r6
	bne		@AGAIN
	stw		r4,0(r3)
	stw		r5,4(r3)
	blr
	}

/* **** **** **** **** **** **** **** **** **** **** **** **** **** **** **** */
/* **** **** **** **** **** **** **** **** **** **** **** **** **** **** **** */

static Ptr FindFunctionInSharedLib(StringPtr libName, StringPtr funcName) {
	OSErr				error = noErr;
	Str255				errorStr;
	Ptr					func = NULL;
	Ptr					entry = NULL;
	CFragSymbolClass	symClass;
	CFragConnectionID	connID;
	
	/* Find CFM containers for the current archecture -- CFM-PPC or CFM-68K */
	if (/* error = */ GetSharedLibrary(libName, kCompiledCFragArch,
			kLoadCFrag, &connID, &entry, errorStr)) return(NULL);
	if (/* error = */ FindSymbol(connID, funcName, &func, &symClass))
		return(NULL);
	
	return(func);
	}
#endif /* GENERATINGPOWERPC */
