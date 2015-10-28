/* File "FastTimes.h" - Original code by Matt Slot <fprefect@ambrosiasw.com>  */
#include "SDL_config.h"
/* Created 4/24/99    - This file is hereby placed in the public domain       */
/* Updated 5/21/99    - Calibrate to VIA, add TBR support, renamed functions  */
/* Updated 10/4/99    - Use AbsoluteToNanoseconds() in case Absolute = double */
/* Updated 2/15/00    - Check for native Time Manager, no need to calibrate   */
/* Updated 3/21/00    - Fixed ns conversion, create 2 different scale factors */
/* Updated 5/03/00    - Added copyright and placed into PD. No code changes   */

/* This file is Copyright (C) Matt Slot, 1999-2012. It is hereby placed into 
   the public domain. The author makes no warranty as to fitness or stability */

#ifndef __FAST_TIMES_HEADER__
#define __FAST_TIMES_HEADER__

/* **** **** **** **** **** **** **** **** **** **** **** **** **** **** **** */
/* **** **** **** **** **** **** **** **** **** **** **** **** **** **** **** */

extern void			FastInitialize(void);
extern UInt64		FastMicroseconds(void);
extern UInt64		FastMilliseconds(void);
extern StringPtr	FastMethod(void);

/* **** **** **** **** **** **** **** **** **** **** **** **** **** **** **** */
/* **** **** **** **** **** **** **** **** **** **** **** **** **** **** **** */

#endif /* __FAST_TIMES_HEADER__ */
