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

/** 
 *  @file begin_code.h
 *  This file sets things up for C dynamic library function definitions,
 *  static inlined functions, and structures aligned at 4-byte alignment.
 *  If you don't like ugly C preprocessor code, don't look at this file. :)
 */

/** 
 *  @file begin_code.h
 *  This shouldn't be nested -- included it around code only.
 */
#ifdef _begin_code_h
#error Nested inclusion of begin_code.h
#endif
#define _begin_code_h

/** 
 *  @def DECLSPEC
 *  Some compilers use a special export keyword
 */
#ifndef DECLSPEC
# if defined(__BEOS__) || defined(__HAIKU__)
#  if defined(__GNUC__)
#   define DECLSPEC
#  else
#   define DECLSPEC	__declspec(export)
#  endif
# elif defined(__WIN32__)
#  ifdef __BORLANDC__
#   ifdef BUILD_SDL
#    define DECLSPEC 
#   else
#    define DECLSPEC	__declspec(dllimport)
#   endif
#  else
#   define DECLSPEC	__declspec(dllexport)
#  endif
# elif defined(__OS2__)
#  ifdef __WATCOMC__
#   ifdef BUILD_SDL
#    define DECLSPEC	__declspec(dllexport)
#   else
#    define DECLSPEC
#   endif
#  elif defined (__GNUC__) && __GNUC__ < 4
#   /* Added support for GCC-EMX <v4.x */
#   /* this is needed for XFree86/OS2 developement */
#   /* F. Ambacher(anakor@snafu.de) 05.2008 */
#   ifdef BUILD_SDL
#    define DECLSPEC    __declspec(dllexport)
#   else
#    define DECLSPEC
#   endif
#  else
#   define DECLSPEC
#  endif
# else
#  if defined(__GNUC__) && __GNUC__ >= 4
#   define DECLSPEC	__attribute__ ((visibility("default")))
#  else
#   define DECLSPEC
#  endif
# endif
#endif

/** 
 *  @def SDLCALL
 *  By default SDL uses the C calling convention
 */
#ifndef SDLCALL
# if defined(__WIN32__) && !defined(__GNUC__)
#  define SDLCALL __cdecl
# elif defined(__OS2__)
#  if defined (__GNUC__) && __GNUC__ < 4
#   /* Added support for GCC-EMX <v4.x */
#   /* this is needed for XFree86/OS2 developement */
#   /* F. Ambacher(anakor@snafu.de) 05.2008 */
#   define SDLCALL _cdecl
#  else
#   /* On other compilers on OS/2, we use the _System calling convention */
#   /* to be compatible with every compiler */
#   define SDLCALL _System
#  endif
# else
#  define SDLCALL
# endif
#endif /* SDLCALL */

#ifdef __SYMBIAN32__ 
#ifndef EKA2 
#undef DECLSPEC
#define DECLSPEC
#elif !defined(__WINS__)
#undef DECLSPEC
#define DECLSPEC __declspec(dllexport)
#endif /* !EKA2 */
#endif /* __SYMBIAN32__ */

/**
 *  @file begin_code.h
 *  Force structure packing at 4 byte alignment.
 *  This is necessary if the header is included in code which has structure
 *  packing set to an alternate value, say for loading structures from disk.
 *  The packing is reset to the previous value in close_code.h 
 */
#if defined(_MSC_VER) || defined(__MWERKS__) || defined(__BORLANDC__)
#ifdef _MSC_VER
#pragma warning(disable: 4103)
#endif
#ifdef __BORLANDC__
#pragma nopackwarning
#endif
#ifdef _M_X64
/* Use 8-byte alignment on 64-bit architectures, so pointers are aligned */
#pragma pack(push,8)
#else
#pragma pack(push,4)
#endif
#elif (defined(__MWERKS__) && defined(__MACOS__))
#pragma options align=mac68k4byte
#pragma enumsalwaysint on
#endif /* Compiler needs structure packing set */

/**
 *  @def SDL_INLINE_OKAY
 *  Set up compiler-specific options for inlining functions
 */
#ifndef SDL_INLINE_OKAY
#ifdef __GNUC__
#define SDL_INLINE_OKAY
#else
/* Add any special compiler-specific cases here */
#if defined(_MSC_VER) || defined(__BORLANDC__) || \
    defined(__DMC__) || defined(__SC__) || \
    defined(__WATCOMC__) || defined(__LCC__) || \
    defined(__DECC) || defined(__EABI__)
#ifndef __inline__
#define __inline__	__inline
#endif
#define SDL_INLINE_OKAY
#else
#if !defined(__MRC__) && !defined(_SGI_SOURCE)
#ifndef __inline__
#define __inline__ inline
#endif
#define SDL_INLINE_OKAY
#endif /* Not a funky compiler */
#endif /* Visual C++ */
#endif /* GNU C */
#endif /* SDL_INLINE_OKAY */

/**
 *  @def __inline__
 *  If inlining isn't supported, remove "__inline__", turning static
 *  inlined functions into static functions (resulting in code bloat
 *  in all files which include the offending header files)
 */
#ifndef SDL_INLINE_OKAY
#define __inline__
#endif

/**
 *  @def NULL
 *  Apparently this is needed by several Windows compilers
 */
#if !defined(__MACH__)
#ifndef NULL
#ifdef __cplusplus
#define NULL 0
#else
#define NULL ((void *)0)
#endif
#endif /* NULL */
#endif /* ! Mac OS X - breaks precompiled headers */
