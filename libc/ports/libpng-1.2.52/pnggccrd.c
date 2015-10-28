/* pnggccrd.c
 *
 * Last changed in libpng 1.2.48 [March 8, 2012]
 * Copyright (c) 1998-2012 Glenn Randers-Pehrson
 *
 * This code is released under the libpng license.
 * For conditions of distribution and use, see the disclaimer
 * and license in png.h
 *
 * This code snippet is for use by configure's compilation test. Most of the
 * remainder of the file was removed from libpng-1.2.20, and all of the
 * assembler code was removed from libpng-1.2.48.
 */

#if (!defined _MSC_VER) && \
    defined(PNG_ASSEMBLER_CODE_SUPPORTED) && \
    defined(PNG_MMX_CODE_SUPPORTED)

int PNGAPI png_dummy_mmx_support(void);

int PNGAPI png_dummy_mmx_support(void)
{
     /* 0: no MMX; 1: MMX supported; 2: not tested */
     return 2;
}
#endif
