Microsoft Developer Studio Project File, Format Version 6.00 for libpng.

Copyright (C) 2000-2004 Simon-Pierre Cadieux.
Copyright (C) 2004 Cosmin Truta.
For conditions of distribution and use, see copyright notice in png.h

Assumptions:
* The libpng source files are in ..\..
* The zlib source files are in ..\..\..\zlib
* The zlib project files are in ..\..\..\zlib\projects\visualc6

To use:

1) On the main menu, select "File | Open Workspace".
   Open "libpng.dsw".

2) Select "Build | Set Active Configuration".
   Choose the configuration you wish to build.
   (Choose libpng or pngtest; zlib will be built automatically.)

3) Select "Build | Clean".

4) Select "Build | Build ... (F7)".  Ignore warning messages about
   not being able to find certain include files (e.g. alloc.h).

5) If you built the sample program (pngtest),
   select "Build | Execute ... (Ctrl+F5)".

This project builds the libpng binaries as follows:

* Win32_DLL_Release\libpng13.dll      DLL build
* Win32_DLL_Debug\libpng13d.dll       DLL build (debug version)
* Win32_DLL_ASM_Release\libpng13.dll  DLL build using ASM code
* Win32_DLL_ASM_Debug\libpng13d.dll   DLL build using ASM (debug version)
* Win32_DLL_VB\libpng13vb.dll         DLL build for Visual Basic, using stdcall
* Win32_LIB_Release\libpng.lib        static build
* Win32_LIB_Debug\libpngd.lib         static build (debug version)
* Win32_LIB_ASM_Release\libpng.lib    static build using ASM code
* Win32_LIB_ASM_Debug\libpngd.lib     static build using ASM (debug version)

Notes:

If you change anything in the source files, or select different compiler
settings, please change the DLL name to something different than any of
the above names. Also, make sure that in your "pngusr.h" you define
PNG_USER_PRIVATEBUILD and PNG_USER_DLLFNAME_POSTFIX according to the
instructions provided in "pngconf.h".

All DLLs built by this project use the Microsoft dynamic C runtime library
MSVCRT.DLL (MSVCRTD.DLL for debug versions).  If you distribute any of the
above mentioned libraries you should also include this DLL in your package.
For a list of files that are redistributable in Visual C++ 6.0, see
Common\Redist\Redist.txt on Disc 1 of the Visual C++ 6.0 product CDs.
