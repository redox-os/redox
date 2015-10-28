/*

SDL_imageFilter.c: byte-image "filter" routines

Copyright (C) 2001-2012  Andreas Schiffler
Copyright (C) 2013  Sylvain Beucler

This software is provided 'as-is', without any express or implied
warranty. In no event will the authors be held liable for any damages
arising from the use of this software.

Permission is granted to anyone to use this software for any purpose,
including commercial applications, and to alter it and redistribute it
freely, subject to the following restrictions:

   1. The origin of this software must not be misrepresented; you must not
   claim that you wrote the original software. If you use this software
   in a product, an acknowledgment in the product documentation would be
   appreciated but is not required.

   2. Altered source versions must be plainly marked as such, and must not be
   misrepresented as being the original software.

   3. This notice may not be removed or altered from any source
   distribution.

Andreas Schiffler -- aschiffler at ferzkopp dot net

*/

/*

Note: Uses inline x86 MMX or ASM optimizations if available and enabled.

Note: Most of the MMX code is based on published routines 
by Vladimir Kravtchenko at vk@cs.ubc.ca - credits go to 
him for his work.

*/

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* Use GCC intrinsics if available: they support both i386 and x86_64,
   provide ASM-grade performances, and lift the PUSHA/POPA issues. */
#ifdef __GNUC__
#  ifdef USE_MMX
#    include <mmintrin.h>
#  endif
#endif
#include <SDL_cpuinfo.h>
#include "SDL_imageFilter.h"

/*!
\brief Swaps the byte order in a 32bit integer (LSB becomes MSB, etc.). 
*/
#define SWAP_32(x) (((x) >> 24) | (((x) & 0x00ff0000) >> 8)  | (((x) & 0x0000ff00) << 8)  | ((x) << 24))

/* ------ Static variables ----- */

/*! 
\brief Static state which enables the use of the MMX routines. Enabled by default 
*/
static int SDL_imageFilterUseMMX = 1;

/* Detect GCC */
#if defined(__GNUC__)
#define GCC__
#endif

/*!
\brief MMX detection routine (with override flag). 

\returns 1 of MMX was detected, 0 otherwise.
*/
int SDL_imageFilterMMXdetect(void)
{
	/* Check override flag */
	if (SDL_imageFilterUseMMX == 0) {
		return (0);
	}

        return SDL_HasMMX();
}

/*!
\brief Disable MMX check for filter functions and and force to use non-MMX C based code.
*/
void SDL_imageFilterMMXoff()
{
	SDL_imageFilterUseMMX = 0;
}

/*!
\brief Enable MMX check for filter functions and use MMX code if available.
*/
void SDL_imageFilterMMXon()
{
	SDL_imageFilterUseMMX = 1;
}

/* ------------------------------------------------------------------------------------ */

/*!
\brief Internal MMX Filter using Add: D = saturation255(S1 + S2) 

\param Src1 Pointer to the start of the first source byte array (S1).
\param Src2 Pointer to the start of the second source byte array (S2).
\param Dest Pointer to the start of the destination byte array (D).
\param SrcLength The number of bytes in the source arrays.

\return Returns 0 for success or -1 for error.
*/
static int SDL_imageFilterAddMMX(unsigned char *Src1, unsigned char *Src2, unsigned char *Dest, unsigned int SrcLength)
{
#ifdef USE_MMX
#if !defined(GCC__)
	__asm
	{
		pusha
			mov eax, Src1	/* load Src1 address into eax */
			mov ebx, Src2	/* load Src2 address into ebx */
			mov edi, Dest	/* load Dest address into edi */
			mov ecx, SrcLength	/* load loop counter (SIZE) into ecx */
			shr ecx, 3	/* counter/8 (MMX loads 8 bytes at a time) */
			align 16	/* 16 byte alignment of the loop entry */
L1010:
		movq mm1, [eax]	/* load 8 bytes from Src1 into mm1 */
		paddusb mm1, [ebx]	/* mm1=Src1+Src2 (add 8 bytes with saturation) */
		movq [edi], mm1	/* store result in Dest */
			add eax, 8	/* increase Src1, Src2 and Dest  */
			add ebx, 8	/* register pointers by 8 */
			add edi, 8
			dec ecx	/* decrease loop counter */
			jnz L1010	/* check loop termination, proceed if required */
			emms /* exit MMX state */
			popa
	}
#else
	/* i386 and x86_64 */
	__m64 *mSrc1 = (__m64*)Src1;
	__m64 *mSrc2 = (__m64*)Src2;
	__m64 *mDest = (__m64*)Dest;
	int i;
	for (i = 0; i < SrcLength/8; i++) {
		*mDest = _m_paddusb(*mSrc1, *mSrc2);	/* Src1+Src2 (add 8 bytes with saturation) */
		mSrc1++;
		mSrc2++;
		mDest++;
	}
	_m_empty();					/* clean MMX state */
#endif
	return (0);
#else
	return (-1);
#endif
}

/*!
\brief Filter using Add: D = saturation255(S1 + S2) 

\param Src1 Pointer to the start of the first source byte array (S1).
\param Src2 Pointer to the start of the second source byte array (S2).
\param Dest Pointer to the start of the destination byte array (D).
\param length The number of bytes in the source arrays.

\return Returns 0 for success or -1 for error.
*/
int SDL_imageFilterAdd(unsigned char *Src1, unsigned char *Src2, unsigned char *Dest, unsigned int length)
{
	unsigned int i, istart;
	unsigned char *cursrc1, *cursrc2, *curdst;
	int result;

	/* Validate input parameters */
	if ((Src1 == NULL) || (Src2 == NULL) || (Dest == NULL))
		return(-1);
	if (length == 0)
		return(0);

	if ((SDL_imageFilterMMXdetect()) && (length > 7)) {

		/* Use MMX assembly routine */
		SDL_imageFilterAddMMX(Src1, Src2, Dest, length);

		/* Check for unaligned bytes */
		if ((length & 7) > 0) {
			/* Setup to process unaligned bytes */
			istart = length & 0xfffffff8;
			cursrc1 = &Src1[istart];
			cursrc2 = &Src2[istart];
			curdst = &Dest[istart];
		} else {
			/* No unaligned bytes - we are done */
			return (0);
		}
	} else {
		/* Setup to process whole image */
		istart = 0;
		cursrc1 = Src1;
		cursrc2 = Src2;
		curdst = Dest;
	}

	/* C routine to process image */
	for (i = istart; i < length; i++) {
		result = (int) *cursrc1 + (int) *cursrc2;
		if (result > 255)
			result = 255;
		*curdst = (unsigned char) result;
		/* Advance pointers */
		cursrc1++;
		cursrc2++;
		curdst++;
	}

	return (0);
}

/*!
\brief Internal MMX Filter using Mean: D = S1/2 + S2/2

\param Src1 Pointer to the start of the first source byte array (S1).
\param Src2 Pointer to the start of the second source byte array (S2).
\param Dest Pointer to the start of the destination byte array (D).
\param SrcLength The number of bytes in the source arrays.
\param Mask Mask array containing 8 bytes with 0x7F value.
]
\return Returns 0 for success or -1 for error.
*/
static int SDL_imageFilterMeanMMX(unsigned char *Src1, unsigned char *Src2, unsigned char *Dest, unsigned int SrcLength,
						   unsigned char *Mask)
{
#ifdef USE_MMX
#if !defined(GCC__)
	__asm
	{ 
		pusha
			mov edx, Mask /* load Mask address into edx */
			movq mm0, [edx] /* load Mask into mm0 */
		mov eax, Src1 /* load Src1 address into eax */
			mov ebx, Src2 /* load Src2 address into ebx */
			mov edi, Dest /* load Dest address into edi */
			mov ecx, SrcLength /* load loop counter (SIZE) into ecx */
			shr ecx, 3 	/* counter/8 (MMX loads 8 bytes at a time) */
			align 16	/* 16 byte alignment of the loop entry */
L21011:
		movq mm1,  [eax] 	/* load 8 bytes from Src1 into mm1 */
		movq mm2,  [ebx] 	/* load 8 bytes from Src2 into mm2 */
		/* --- Byte shift via Word shift --- */
		psrlw mm1, 1 	/* shift 4 WORDS of mm1 1 bit to the right */
			psrlw mm2, 1 	/* shift 4 WORDS of mm2 1 bit to the right */
			pand mm1, mm0   // apply Mask to 8 BYTES of mm1 */
			/* byte     0x0f, 0xdb, 0xc8 */
			pand mm2, mm0   // apply Mask to 8 BYTES of mm2 */
			/* byte     0x0f, 0xdb, 0xd0 */
			paddusb mm1,  mm2 	/* mm1=mm1+mm2 (add 8 bytes with saturation) */
			movq [edi],  mm1 	/* store result in Dest */
			add eax,  8 	/* increase Src1, Src2 and Dest  */
			add ebx,  8 	/* register pointers by 8 */
			add edi,  8
			dec ecx 	/* decrease loop counter */
			jnz L21011	/* check loop termination, proceed if required */
			emms	/* exit MMX state */
			popa
	}
#else
	/* i386 and x86_64 */
	__m64 *mSrc1 = (__m64*)Src1;
	__m64 *mSrc2 = (__m64*)Src2;
	__m64 *mDest = (__m64*)Dest;
	__m64 *mMask = (__m64*)Mask;
	int i;
	for (i = 0; i < SrcLength/8; i++) {
		__m64 mm1 = *mSrc1,
		      mm2 = *mSrc2;
		mm1 = _m_psrlwi(mm1, 1);	/* shift 4 WORDS of mm1 1 bit to the right */
		mm2 = _m_psrlwi(mm2, 1);	/* shift 4 WORDS of mm2 1 bit to the right */
		mm1 = _m_pand(mm1, *mMask);	/* apply Mask to 8 BYTES of mm1 */
		mm2 = _m_pand(mm2, *mMask);	/* apply Mask to 8 BYTES of mm2 */
		*mDest = _m_paddusb(mm1, mm2);	/* mm1+mm2 (add 8 bytes with saturation) */
		mSrc1++;
		mSrc2++;
		mDest++;
	}
	_m_empty();				/* clean MMX state */
#endif
	return (0);
#else
	return (-1);
#endif
}

/*!
\brief Filter using Mean: D = S1/2 + S2/2

\param Src1 Pointer to the start of the first source byte array (S1).
\param Src2 Pointer to the start of the second source byte array (S2).
\param Dest Pointer to the start of the destination byte array (D).
\param length The number of bytes in the source arrays.

\return Returns 0 for success or -1 for error.
*/
int SDL_imageFilterMean(unsigned char *Src1, unsigned char *Src2, unsigned char *Dest, unsigned int length)
{
	static unsigned char Mask[8] = { 0x7F, 0x7F, 0x7F, 0x7F, 0x7F, 0x7F, 0x7F, 0x7F };
	unsigned int i, istart;
	unsigned char *cursrc1, *cursrc2, *curdst;
	int result;

	/* Validate input parameters */
	if ((Src1 == NULL) || (Src2 == NULL) || (Dest == NULL))
		return(-1);
	if (length == 0)
		return(0);

	if ((SDL_imageFilterMMXdetect()) && (length > 7)) {
		/* MMX routine */
		SDL_imageFilterMeanMMX(Src1, Src2, Dest, length, Mask);

		/* Check for unaligned bytes */
		if ((length & 7) > 0) {
			/* Setup to process unaligned bytes */
			istart = length & 0xfffffff8;
			cursrc1 = &Src1[istart];
			cursrc2 = &Src2[istart];
			curdst = &Dest[istart];
		} else {
			/* No unaligned bytes - we are done */
			return (0);
		}
	} else {
		/* Setup to process whole image */
		istart = 0;
		cursrc1 = Src1;
		cursrc2 = Src2;
		curdst = Dest;
	}

	/* C routine to process image */
	for (i = istart; i < length; i++) {
		result = (int) *cursrc1 / 2 + (int) *cursrc2 / 2;
		*curdst = (unsigned char) result;
		/* Advance pointers */
		cursrc1++;
		cursrc2++;
		curdst++;
	}

	return (0);
}

/*!
\brief Internal MMX Filter using Sub: D = saturation0(S1 - S2)

\param Src1 Pointer to the start of the first source byte array (S1).
\param Src2 Pointer to the start of the second source byte array (S2).
\param Dest Pointer to the start of the destination byte array (D).
\param SrcLength The number of bytes in the source arrays.

\return Returns 0 for success or -1 for error.
*/
static int SDL_imageFilterSubMMX(unsigned char *Src1, unsigned char *Src2, unsigned char *Dest, unsigned int SrcLength)
{
#ifdef USE_MMX
#if !defined(GCC__)
	__asm
	{
		pusha
			mov eax,  Src1 	/* load Src1 address into eax */
			mov ebx,  Src2 	/* load Src2 address into ebx */
			mov edi,  Dest 	/* load Dest address into edi */
			mov ecx,  SrcLength 	/* load loop counter (SIZE) into ecx */
			shr ecx,  3 	/* counter/8 (MMX loads 8 bytes at a time) */
			align 16 /* 16 byte alignment of the loop entry */
L1012:
		movq mm1,  [eax] 	/* load 8 bytes from Src1 into mm1 */
		psubusb mm1,  [ebx] 	/* mm1=Src1-Src2 (sub 8 bytes with saturation) */
		movq [edi],  mm1 	/* store result in Dest */
			add eax, 8 	/* increase Src1, Src2 and Dest  */
			add ebx, 8 	/* register pointers by 8 */
			add edi, 8
			dec ecx	/* decrease loop counter */
			jnz L1012	/* check loop termination, proceed if required */
			emms /* exit MMX state */
			popa
	}
#else
	/* i386 and x86_64 */
	__m64 *mSrc1 = (__m64*)Src1;
	__m64 *mSrc2 = (__m64*)Src2;
	__m64 *mDest = (__m64*)Dest;
	int i;
	for (i = 0; i < SrcLength/8; i++) {
		*mDest = _m_psubusb(*mSrc1, *mSrc2);	/* Src1-Src2 (sub 8 bytes with saturation) */
		mSrc1++;
		mSrc2++;
		mDest++;
	}
	_m_empty();					/* clean MMX state */
#endif
	return (0);
#else
	return (-1);
#endif
}

/*!
\brief Filter using Sub: D = saturation0(S1 - S2)

\param Src1 Pointer to the start of the first source byte array (S1).
\param Src2 Pointer to the start of the second source byte array (S2).
\param Dest Pointer to the start of the destination byte array (D).
\param length The number of bytes in the source arrays.

\return Returns 0 for success or -1 for error.
*/
int SDL_imageFilterSub(unsigned char *Src1, unsigned char *Src2, unsigned char *Dest, unsigned int length)
{
	unsigned int i, istart;
	unsigned char *cursrc1, *cursrc2, *curdst;
	int result;

	/* Validate input parameters */
	if ((Src1 == NULL) || (Src2 == NULL) || (Dest == NULL))
		return(-1);
	if (length == 0)
		return(0);

	if ((SDL_imageFilterMMXdetect()) && (length > 7)) {
		/* MMX routine */
		SDL_imageFilterSubMMX(Src1, Src2, Dest, length);

		/* Check for unaligned bytes */
		if ((length & 7) > 0) {
			/* Setup to process unaligned bytes */
			istart = length & 0xfffffff8;
			cursrc1 = &Src1[istart];
			cursrc2 = &Src2[istart];
			curdst = &Dest[istart];
		} else {
			/* No unaligned bytes - we are done */
			return (0);
		}
	} else {
		/* Setup to process whole image */
		istart = 0;
		cursrc1 = Src1;
		cursrc2 = Src2;
		curdst = Dest;
	}

	/* C routine to process image */
	for (i = istart; i < length; i++) {
		result = (int) *cursrc1 - (int) *cursrc2;
		if (result < 0)
			result = 0;
		*curdst = (unsigned char) result;
		/* Advance pointers */
		cursrc1++;
		cursrc2++;
		curdst++;
	}

	return (0);
}

/*!
\brief Internal MMX Filter using AbsDiff: D = | S1 - S2 |

\param Src1 Pointer to the start of the first source byte array (S1).
\param Src2 Pointer to the start of the second source byte array (S2).
\param Dest Pointer to the start of the destination byte array (D).
\param SrcLength The number of bytes in the source arrays.

\return Returns 0 for success or -1 for error.
*/
static int SDL_imageFilterAbsDiffMMX(unsigned char *Src1, unsigned char *Src2, unsigned char *Dest, unsigned int SrcLength)
{
#ifdef USE_MMX
#if !defined(GCC__)
	__asm
	{
		pusha
			mov eax, Src1  	/* load Src1 address into eax */
			mov ebx, Src2 	/* load Src2 address into ebx */
			mov edi, Dest 	/* load Dest address into edi */
			mov ecx, SrcLength 	/* load loop counter (SIZE) into ecx */
			shr ecx,  3 	/* counter/8 (MMX loads 8 bytes at a time) */
			align 16	/* 16 byte alignment of the loop entry */
L1013:
		movq mm1,  [eax] 	/* load 8 bytes from Src1 into mm1 */
		movq mm2,  [ebx] 	/* load 8 bytes from Src2 into mm2 */
		psubusb mm1,  [ebx] 	/* mm1=Src1-Src2 (sub 8 bytes with saturation) */
		psubusb mm2,  [eax] 	/* mm2=Src2-Src1 (sub 8 bytes with saturation) */
		por mm1,  mm2 	/* combine both mm2 and mm1 results */
			movq [edi],  mm1 	/* store result in Dest */
			add eax, 8 	/* increase Src1, Src2 and Dest  */
			add ebx, 8 	/* register pointers by 8 */
			add edi, 8
			dec ecx 	/* decrease loop counter */
			jnz L1013    	/* check loop termination, proceed if required */
			emms         /* exit MMX state */
			popa
	}
#else
	/* i386 and x86_64 */
	__m64 *mSrc1 = (__m64*)Src1;
	__m64 *mSrc2 = (__m64*)Src2;
	__m64 *mDest = (__m64*)Dest;
	int i;
	for (i = 0; i < SrcLength/8; i++) {
		__m64 mm1 = _m_psubusb(*mSrc2, *mSrc1);	/* Src1-Src2 (sub 8 bytes with saturation) */
		__m64 mm2 = _m_psubusb(*mSrc1, *mSrc2);	/* Src2-Src1 (sub 8 bytes with saturation) */
		*mDest = _m_por(mm1, mm2);		/* combine both mm2 and mm1 results */
		mSrc1++;
		mSrc2++;
		mDest++;
	}
	_m_empty();					/* clean MMX state */
#endif
	return (0);
#else
	return (-1);
#endif
}

/*!
\brief Filter using AbsDiff: D = | S1 - S2 |

\param Src1 Pointer to the start of the first source byte array (S1).
\param Src2 Pointer to the start of the second source byte array (S2).
\param Dest Pointer to the start of the destination byte array (D).
\param length The number of bytes in the source arrays.

\return Returns 0 for success or -1 for error.
*/
int SDL_imageFilterAbsDiff(unsigned char *Src1, unsigned char *Src2, unsigned char *Dest, unsigned int length)
{
	unsigned int i, istart;
	unsigned char *cursrc1, *cursrc2, *curdst;
	int result;

	/* Validate input parameters */
	if ((Src1 == NULL) || (Src2 == NULL) || (Dest == NULL))
		return(-1);
	if (length == 0)
		return(0);

	if ((SDL_imageFilterMMXdetect()) && (length > 7)) {
		/* MMX routine */
		SDL_imageFilterAbsDiffMMX(Src1, Src2, Dest, length);

		/* Check for unaligned bytes */
		if ((length & 7) > 0) {
			/* Setup to process unaligned bytes */
			istart = length & 0xfffffff8;
			cursrc1 = &Src1[istart];
			cursrc2 = &Src2[istart];
			curdst = &Dest[istart];
		} else {
			/* No unaligned bytes - we are done */
			return (0);
		}
	} else {
		/* Setup to process whole image */
		istart = 0;
		cursrc1 = Src1;
		cursrc2 = Src2;
		curdst = Dest;
	}

	/* C routine to process image */
	for (i = istart; i < length; i++) {
		result = abs((int) *cursrc1 - (int) *cursrc2);
		*curdst = (unsigned char) result;
		/* Advance pointers */
		cursrc1++;
		cursrc2++;
		curdst++;
	}

	return (0);
}

/*!
\brief Internal MMX Filter using Mult: D = saturation255(S1 * S2)

\param Src1 Pointer to the start of the first source byte array (S1).
\param Src2 Pointer to the start of the second source byte array (S2).
\param Dest Pointer to the start of the destination byte array (D).
\param SrcLength The number of bytes in the source arrays.

\return Returns 0 for success or -1 for error.
*/
static int SDL_imageFilterMultMMX(unsigned char *Src1, unsigned char *Src2, unsigned char *Dest, unsigned int SrcLength)
{
#ifdef USE_MMX
#if !defined(GCC__)
	__asm
	{
		pusha
			mov eax, Src1   /* load Src1 address into eax */
			mov ebx, Src2   /* load Src2 address into ebx */
			mov edi, Dest   /* load Dest address into edi */
			mov ecx, SrcLength   /* load loop counter (SIZE) into ecx */
			shr ecx, 3   /* counter/8 (MMX loads 8 bytes at a time) */
			pxor mm0, mm0   /* zero mm0 register */
			align 16      	/* 16 byte alignment of the loop entry */
L1014:
		movq mm1, [eax]   /* load 8 bytes from Src1 into mm1 */
		movq mm3, [ebx]   /* load 8 bytes from Src2 into mm3 */
		movq mm2, mm1   /* copy mm1 into mm2 */
			movq mm4, mm3   /* copy mm3 into mm4  */
			punpcklbw mm1, mm0   /* unpack low  bytes of Src1 into words */
			punpckhbw mm2, mm0   /* unpack high bytes of Src1 into words */
			punpcklbw mm3, mm0   /* unpack low  bytes of Src2 into words */
			punpckhbw mm4, mm0   /* unpack high bytes of Src2 into words */
			pmullw mm1, mm3   /* mul low  bytes of Src1 and Src2  */
			pmullw mm2, mm4   /* mul high bytes of Src1 and Src2 */
			/* Take abs value of the results (signed words) */
			movq mm5, mm1   /* copy mm1 into mm5 */
			movq mm6, mm2   /* copy mm2 into mm6 */
			psraw mm5, 15   /* fill mm5 words with word sign bit */
			psraw mm6, 15   /* fill mm6 words with word sign bit */
			pxor mm1, mm5   /* take 1's compliment of only neg. words */
			pxor mm2, mm6   /* take 1's compliment of only neg. words */
			psubsw mm1, mm5   /* add 1 to only neg. words, W-(-1) or W-0 */
			psubsw mm2, mm6   /* add 1 to only neg. words, W-(-1) or W-0 */
			packuswb mm1, mm2   /* pack words back into bytes with saturation */
			movq [edi], mm1   /* store result in Dest */
			add eax, 8   /* increase Src1, Src2 and Dest  */
			add ebx, 8   /* register pointers by 8 */
			add edi, 8
			dec ecx 	/* decrease loop counter */
			jnz L1014	/* check loop termination, proceed if required */
			emms /* exit MMX state */
			popa
	}
#else
	/* i386 ASM with constraints: */
	/* asm volatile ( */
	/* 	"shr $3, %%ecx \n\t"	/\* counter/8 (MMX loads 8 bytes at a time) *\/ */
	/* 	"pxor      %%mm0, %%mm0 \n\t"	/\* zero mm0 register *\/ */
	/* 	".align 16       \n\t"	/\* 16 byte alignment of the loop entry *\/ */
	/* 	"1: movq (%%eax), %%mm1 \n\t"     /\* load 8 bytes from Src1 into mm1 *\/ */
	/* 	"movq    (%%ebx), %%mm3 \n\t"	/\* load 8 bytes from Src2 into mm3 *\/ */
	/* 	"movq      %%mm1, %%mm2 \n\t"	/\* copy mm1 into mm2 *\/ */
	/* 	"movq      %%mm3, %%mm4 \n\t"	/\* copy mm3 into mm4  *\/ */
	/* 	"punpcklbw %%mm0, %%mm1 \n\t"	/\* unpack low  bytes of Src1 into words *\/ */
	/* 	"punpckhbw %%mm0, %%mm2 \n\t"	/\* unpack high bytes of Src1 into words *\/ */
	/* 	"punpcklbw %%mm0, %%mm3 \n\t"	/\* unpack low  bytes of Src2 into words *\/ */
	/* 	"punpckhbw %%mm0, %%mm4 \n\t"	/\* unpack high bytes of Src2 into words *\/ */
	/* 	"pmullw    %%mm3, %%mm1 \n\t"	/\* mul low  bytes of Src1 and Src2  *\/ */
	/* 	"pmullw    %%mm4, %%mm2 \n\t"	/\* mul high bytes of Src1 and Src2 *\/ */
	/* 	/\* Take abs value of the results (signed words) *\/ */
	/* 	"movq      %%mm1, %%mm5 \n\t"	/\* copy mm1 into mm5 *\/ */
	/* 	"movq      %%mm2, %%mm6 \n\t"	/\* copy mm2 into mm6 *\/ */
	/* 	"psraw       $15, %%mm5 \n\t"	/\* fill mm5 words with word sign bit *\/ */
	/* 	"psraw       $15, %%mm6 \n\t"	/\* fill mm6 words with word sign bit *\/ */
	/* 	"pxor      %%mm5, %%mm1 \n\t"	/\* take 1's compliment of only neg. words *\/ */
	/* 	"pxor      %%mm6, %%mm2 \n\t"	/\* take 1's compliment of only neg. words *\/ */
	/* 	"psubsw    %%mm5, %%mm1 \n\t"	/\* add 1 to only neg. words, W-(-1) or W-0 *\/ */
	/* 	"psubsw    %%mm6, %%mm2 \n\t"	/\* add 1 to only neg. words, W-(-1) or W-0 *\/ */
	/* 	"packuswb  %%mm2, %%mm1 \n\t"	/\* pack words back into bytes with saturation *\/ */
	/* 	"movq    %%mm1, (%%edi) \n\t"	/\* store result in Dest *\/ */
	/* 	"add $8, %%eax \n\t"	/\* increase Src1, Src2 and Dest  *\/ */
	/* 	"add $8, %%ebx \n\t"	/\* register pointers by 8 *\/ */
	/* 	"add $8, %%edi \n\t" */
	/* 	"dec %%ecx     \n\t"	/\* decrease loop counter *\/ */
	/* 	"jnz 1b        \n\t"	/\* check loop termination, proceed if required *\/ */
	/* 	"emms          \n\t"	/\* exit MMX state *\/ */
	/* 	: "+a" (Src1),		/\* load Src1 address into rax, modified by the loop *\/ */
	/* 	  "+b" (Src2),		/\* load Src2 address into rbx, modified by the loop *\/ */
	/* 	  "+c" (SrcLength),	/\* load loop counter (SIZE) into rcx, modified by the loop *\/ */
	/* 	  "+D" (Dest)		/\* load Dest address into rdi, modified by the loop *\/ */
	/* 	: */
	/* 	: "memory",		/\* *Dest is modified *\/ */
        /*           "mm0","mm1","mm2","mm3","mm4","mm5","mm6"	/\* registers modified *\/ */
	/* ); */

	/* i386 and x86_64 */
	__m64 *mSrc1 = (__m64*)Src1;
	__m64 *mSrc2 = (__m64*)Src2;
	__m64 *mDest = (__m64*)Dest;
	__m64 mm0 = _m_from_int(0); /* zero mm0 register */
	int i;
	for (i = 0; i < SrcLength/8; i++) {
		__m64 mm1, mm2, mm3, mm4, mm5, mm6;
		mm1 = _m_punpcklbw(*mSrc1, mm0);	/* unpack low  bytes of Src1 into words */
		mm2 = _m_punpckhbw(*mSrc1, mm0);	/* unpack high bytes of Src1 into words */
		mm3 = _m_punpcklbw(*mSrc2, mm0);	/* unpack low  bytes of Src2 into words */
		mm4 = _m_punpckhbw(*mSrc2, mm0);	/* unpack high bytes of Src2 into words */
		mm1 = _m_pmullw(mm1, mm3);		/* mul low  bytes of Src1 and Src2  */
		mm2 = _m_pmullw(mm2, mm4);		/* mul high bytes of Src1 and Src2 */
		mm5 = _m_psrawi(mm1, 15);		/* fill mm5 words with word sign bit */
		mm6 = _m_psrawi(mm2, 15);		/* fill mm6 words with word sign bit */
		mm1 = _m_pxor(mm1, mm5);		/* take 1's compliment of only neg. words */
		mm2 = _m_pxor(mm2, mm6);		/* take 1's compliment of only neg. words */
		mm1 = _m_psubsw(mm1, mm5);		/* add 1 to only neg. words, W-(-1) or W-0 */
		mm2 = _m_psubsw(mm2, mm6);		/* add 1 to only neg. words, W-(-1) or W-0 */
		*mDest = _m_packuswb(mm1, mm2);		/* pack words back into bytes with saturation */
		mSrc1++;
		mSrc2++;
		mDest++;
	}
	_m_empty();					/* clean MMX state */
#endif
	return (0);
#else
	return (-1);
#endif
}

/*!
\brief Filter using Mult: D = saturation255(S1 * S2)

\param Src1 Pointer to the start of the first source byte array (S1).
\param Src2 Pointer to the start of the second source byte array (S2).
\param Dest Pointer to the start of the destination byte array (D).
\param length The number of bytes in the source arrays.

\return Returns 0 for success or -1 for error.
*/
int SDL_imageFilterMult(unsigned char *Src1, unsigned char *Src2, unsigned char *Dest, unsigned int length)
{
	unsigned int i, istart;
	unsigned char *cursrc1, *cursrc2, *curdst;
	int result;

	/* Validate input parameters */
	if ((Src1 == NULL) || (Src2 == NULL) || (Dest == NULL))
		return(-1);
	if (length == 0)
		return(0);

	if ((SDL_imageFilterMMXdetect()) && (length > 7)) {
		/* MMX routine */
		SDL_imageFilterMultMMX(Src1, Src2, Dest, length);

		/* Check for unaligned bytes */
		if ((length & 7) > 0) {
			/* Setup to process unaligned bytes */
			istart = length & 0xfffffff8;
			cursrc1 = &Src1[istart];
			cursrc2 = &Src2[istart];
			curdst = &Dest[istart];
		} else {
			/* No unaligned bytes - we are done */
			return (0);
		}
	} else {
		/* Setup to process whole image */
		istart = 0;
		cursrc1 = Src1;
		cursrc2 = Src2;
		curdst = Dest;
	}

	/* C routine to process image */
	for (i = istart; i < length; i++) {

		/* NOTE: this is probably wrong - dunno what the MMX code does */

		result = (int) *cursrc1 * (int) *cursrc2;
		if (result > 255)
			result = 255;
		*curdst = (unsigned char) result;
		/* Advance pointers */
		cursrc1++;
		cursrc2++;
		curdst++;
	}

	return (0);
}

/*!
\brief Internal ASM Filter using MultNor: D = S1 * S2

\param Src1 Pointer to the start of the first source byte array (S1).
\param Src2 Pointer to the start of the second source byte array (S2).
\param Dest Pointer to the start of the destination byte array (D).
\param SrcLength The number of bytes in the source arrays.

\return Returns 0 for success or -1 for error.
*/
int SDL_imageFilterMultNorASM(unsigned char *Src1, unsigned char *Src2, unsigned char *Dest, unsigned int SrcLength)
{
#ifdef USE_MMX
#if !defined(GCC__)
	__asm
	{
		pusha
			mov edx, Src1   /* load Src1 address into edx */
			mov esi, Src2   /* load Src2 address into esi */
			mov edi, Dest   /* load Dest address into edi */
			mov ecx, SrcLength   /* load loop counter (SIZE) into ecx */
			align 16 	/* 16 byte alignment of the loop entry */
L10141:
		mov al, [edx]   /* load a byte from Src1 */
		mul [esi] 	/* mul with a byte from Src2 */
		mov [edi], al   /* move a byte result to Dest */
			inc edx 	/* increment Src1, Src2, Dest */
			inc esi 		/* pointer registers by one */
			inc edi
			dec ecx	/* decrease loop counter */
			jnz L10141  	/* check loop termination, proceed if required */
			popa
	}
#else
	/* Note: ~5% gain on i386, less efficient than C on x86_64 */
	/* Also depends on whether this function is static (?!) */
	asm volatile (
		".align 16       \n\t"	/* 16 byte alignment of the loop entry */
#  if defined(i386)
		"1:mov  (%%edx), %%al \n\t"      /* load a byte from Src1 */
		"mulb (%%esi)       \n\t"	/* mul with a byte from Src2 */
		"mov %%al, (%%edi)  \n\t"       /* move a byte result to Dest */
		"inc %%edx \n\t"		/* increment Src1, Src2, Dest */
		"inc %%esi \n\t"		/* pointer registers by one */
		"inc %%edi \n\t"
		"dec %%ecx      \n\t"	/* decrease loop counter */
#  elif defined(__x86_64__)
		"1:mov  (%%rdx), %%al \n\t"      /* load a byte from Src1 */
		"mulb (%%rsi)       \n\t"	/* mul with a byte from Src2 */
		"mov %%al, (%%rdi)  \n\t"       /* move a byte result to Dest */
		"inc %%rdx \n\t"		/* increment Src1, Src2, Dest */
		"inc %%rsi \n\t"		/* pointer registers by one */
		"inc %%rdi \n\t"
		"dec %%rcx      \n\t"	/* decrease loop counter */
#  endif
		"jnz 1b         \n\t"	/* check loop termination, proceed if required */
		: "+d" (Src1),		/* load Src1 address into edx */
		  "+S" (Src2),		/* load Src2 address into esi */
		  "+c" (SrcLength),	/* load loop counter (SIZE) into ecx */
		  "+D" (Dest)		/* load Dest address into edi */
		:
		: "memory", "rax"
		);
#endif
	return (0);
#else
	return (-1);
#endif
}

/*!
\brief Filter using MultNor: D = S1 * S2

\param Src1 Pointer to the start of the first source byte array (S1).
\param Src2 Pointer to the start of the second source byte array (S2).
\param Dest Pointer to the start of the destination byte array (D).
\param length The number of bytes in the source arrays.

\return Returns 0 for success or -1 for error.
*/
int SDL_imageFilterMultNor(unsigned char *Src1, unsigned char *Src2, unsigned char *Dest, unsigned int length)
{
	unsigned int i, istart;
	unsigned char *cursrc1, *cursrc2, *curdst;

	/* Validate input parameters */
	if ((Src1 == NULL) || (Src2 == NULL) || (Dest == NULL))
		return(-1);
	if (length == 0)
		return(0);

	if (SDL_imageFilterMMXdetect()) {
		if (length > 0) {
			/* ASM routine */
			SDL_imageFilterMultNorASM(Src1, Src2, Dest, length);

			/* Check for unaligned bytes */
			if ((length & 7) > 0) {
				/* Setup to process unaligned bytes */
				istart = length & 0xfffffff8;
				cursrc1 = &Src1[istart];
				cursrc2 = &Src2[istart];
				curdst = &Dest[istart];
			} else {
				/* No unaligned bytes - we are done */
				return (0);
			}
		} else {
			/* No bytes - we are done */
			return (0);
		}
	} else {
		/* Setup to process whole image */
		istart = 0;
		cursrc1 = Src1;
		cursrc2 = Src2;
		curdst = Dest;
	}

	/* C routine to process image */
	for (i = istart; i < length; i++) {
		*curdst = (int)*cursrc1 * (int)*cursrc2;  // (int) for efficiency
		/* Advance pointers */
		cursrc1++;
		cursrc2++;
		curdst++;
	}

	return (0);
}

/*!
\brief Internal MMX Filter using MultDivby2: D = saturation255(S1/2 * S2)

\param Src1 Pointer to the start of the first source byte array (S1).
\param Src2 Pointer to the start of the second source byte array (S2).
\param Dest Pointer to the start of the destination byte array (D).
\param SrcLength The number of bytes in the source arrays.

\return Returns 0 for success or -1 for error.
*/
static int SDL_imageFilterMultDivby2MMX(unsigned char *Src1, unsigned char *Src2, unsigned char *Dest, unsigned int SrcLength)
{
#ifdef USE_MMX
#if !defined(GCC__)
	__asm
	{ 
		pusha
			mov eax, Src1   	/* load Src1 address into eax */
			mov ebx, Src2   	/* load Src2 address into ebx */
			mov edi, Dest   	/* load Dest address into edi */
			mov ecx,  SrcLength 	/* load loop counter (SIZE) into ecx */
			shr ecx,  3 	/* counter/8 (MMX loads 8 bytes at a time) */
			pxor mm0,  mm0 	/* zero mm0 register */
			align 16          	/* 16 byte alignment of the loop entry */
L1015:
		movq mm1,  [eax] 	/* load 8 bytes from Src1 into mm1 */
		movq mm3,  [ebx] 	/* load 8 bytes from Src2 into mm3 */
		movq mm2,  mm1 	/* copy mm1 into mm2 */
			movq mm4,  mm3 	/* copy mm3 into mm4  */
			punpcklbw mm1,  mm0 	/* unpack low  bytes of Src1 into words */
			punpckhbw mm2,  mm0 	/* unpack high bytes of Src1 into words */
			punpcklbw mm3,  mm0 	/* unpack low  bytes of Src2 into words */
			punpckhbw mm4,  mm0 	/* unpack high bytes of Src2 into words */
			psrlw mm1,  1 	/* divide mm1 words by 2, Src1 low bytes */
			psrlw mm2,  1 	/* divide mm2 words by 2, Src1 high bytes */
			pmullw mm1,  mm3 	/* mul low  bytes of Src1 and Src2  */
			pmullw mm2,  mm4 	/* mul high bytes of Src1 and Src2 */
			packuswb mm1,  mm2 	/* pack words back into bytes with saturation */
			movq [edi],  mm1 	/* store result in Dest */
			add eax,  8 	/* increase Src1, Src2 and Dest  */
			add ebx,  8 	/* register pointers by 8 */
			add edi,  8
			dec ecx        	/* decrease loop counter */
			jnz L1015       	/* check loop termination, proceed if required */
			emms             	/* exit MMX state */
			popa
	}
#else
	/* i386 and x86_64 */
	__m64 *mSrc1 = (__m64*)Src1;
	__m64 *mSrc2 = (__m64*)Src2;
	__m64 *mDest = (__m64*)Dest;
	__m64 mm0 = _m_from_int(0); /* zero mm0 register */
	int i;
	for (i = 0; i < SrcLength/8; i++) {
		__m64 mm1, mm2, mm3, mm4, mm5, mm6;
		mm1 = _m_punpcklbw(*mSrc1, mm0);	/* unpack low  bytes of Src1 into words */
		mm2 = _m_punpckhbw(*mSrc1, mm0);	/* unpack high bytes of Src1 into words */
		mm3 = _m_punpcklbw(*mSrc2, mm0);	/* unpack low  bytes of Src2 into words */
		mm4 = _m_punpckhbw(*mSrc2, mm0);	/* unpack high bytes of Src2 into words */
		mm1 = _m_psrlwi(mm1, 1);		/* divide mm1 words by 2, Src1 low bytes */
		mm2 = _m_psrlwi(mm2, 1);		/* divide mm2 words by 2, Src1 high bytes */
		mm1 = _m_pmullw(mm1, mm3);		/* mul low  bytes of Src1 and Src2  */
		mm2 = _m_pmullw(mm2, mm4);		/* mul high bytes of Src1 and Src2 */
		*mDest = _m_packuswb(mm1, mm2);		/* pack words back into bytes with saturation */
		mSrc1++;
		mSrc2++;
		mDest++;
	}
	_m_empty();					/* clean MMX state */
#endif
	return (0);
#else
	return (-1);
#endif
}

/*!
\brief Filter using MultDivby2: D = saturation255(S1/2 * S2)

\param Src1 Pointer to the start of the first source byte array (S1).
\param Src2 Pointer to the start of the second source byte array (S2).
\param Dest Pointer to the start of the destination byte array (D).
\param length The number of bytes in the source arrays.

\return Returns 0 for success or -1 for error.
*/
int SDL_imageFilterMultDivby2(unsigned char *Src1, unsigned char *Src2, unsigned char *Dest, unsigned int length)
{
	unsigned int i, istart;
	unsigned char *cursrc1, *cursrc2, *curdst;
	int result;

	/* Validate input parameters */
	if ((Src1 == NULL) || (Src2 == NULL) || (Dest == NULL))
		return(-1);
	if (length == 0)
		return(0);

	if ((SDL_imageFilterMMXdetect()) && (length > 7)) {
		/* MMX routine */
		SDL_imageFilterMultDivby2MMX(Src1, Src2, Dest, length);

		/* Check for unaligned bytes */
		if ((length & 7) > 0) {
			/* Setup to process unaligned bytes */
			istart = length & 0xfffffff8;
			cursrc1 = &Src1[istart];
			cursrc2 = &Src2[istart];
			curdst = &Dest[istart];
		} else {
			/* No unaligned bytes - we are done */
			return (0);
		}
	} else {
		/* Setup to process whole image */
		istart = 0;
		cursrc1 = Src1;
		cursrc2 = Src2;
		curdst = Dest;
	}

	/* C routine to process image */
	for (i = istart; i < length; i++) {
		result = ((int) *cursrc1 / 2) * (int) *cursrc2;
		if (result > 255)
			result = 255;
		*curdst = (unsigned char) result;
		/* Advance pointers */
		cursrc1++;
		cursrc2++;
		curdst++;
	}

	return (0);
}

/*!
\brief Internal MMX Filter using MultDivby4: D = saturation255(S1/2 * S2/2)

\param Src1 Pointer to the start of the first source byte array (S1).
\param Src2 Pointer to the start of the second source byte array (S2).
\param Dest Pointer to the start of the destination byte array (D).
\param SrcLength The number of bytes in the source arrays.

\return Returns 0 for success or -1 for error.
*/
static int SDL_imageFilterMultDivby4MMX(unsigned char *Src1, unsigned char *Src2, unsigned char *Dest, unsigned int SrcLength)
{
#ifdef USE_MMX
#if !defined(GCC__)
	__asm
	{
		pusha
			mov eax, Src1   	/* load Src1 address into eax */
			mov ebx, Src2   	/* load Src2 address into ebx */
			mov edi, Dest   	/* load Dest address into edi */
			mov ecx, SrcLength 	/* load loop counter (SIZE) into ecx */
			shr ecx,  3 	/* counter/8 (MMX loads 8 bytes at a time) */
			pxor mm0, mm0   	/* zero mm0 register */
			align 16          	/* 16 byte alignment of the loop entry */
L1016:
		movq mm1, [eax]   	/* load 8 bytes from Src1 into mm1 */
		movq mm3, [ebx]   	/* load 8 bytes from Src2 into mm3 */
		movq mm2, mm1   	/* copy mm1 into mm2 */
			movq mm4, mm3   	/* copy mm3 into mm4  */
			punpcklbw mm1, mm0   	/* unpack low  bytes of Src1 into words */
			punpckhbw mm2, mm0   	/* unpack high bytes of Src1 into words */
			punpcklbw mm3, mm0   	/* unpack low  bytes of Src2 into words */
			punpckhbw mm4, mm0   	/* unpack high bytes of Src2 into words */
			psrlw mm1, 1   	/* divide mm1 words by 2, Src1 low bytes */
			psrlw mm2, 1   	/* divide mm2 words by 2, Src1 high bytes */
			psrlw mm3, 1   	/* divide mm3 words by 2, Src2 low bytes */
			psrlw mm4, 1   	/* divide mm4 words by 2, Src2 high bytes */
			pmullw mm1, mm3   	/* mul low  bytes of Src1 and Src2  */
			pmullw mm2, mm4   	/* mul high bytes of Src1 and Src2 */
			packuswb mm1, mm2   	/* pack words back into bytes with saturation */
			movq [edi], mm1   	/* store result in Dest */
			add eax, 8   	/* increase Src1, Src2 and Dest  */
			add ebx, 8   	/* register pointers by 8 */
			add edi,  8
			dec ecx        	/* decrease loop counter */
			jnz L1016       	/* check loop termination, proceed if required */
			emms             	/* exit MMX state */
			popa
	}
#else
	/* i386 and x86_64 */
	__m64 *mSrc1 = (__m64*)Src1;
	__m64 *mSrc2 = (__m64*)Src2;
	__m64 *mDest = (__m64*)Dest;
	__m64 mm0 = _m_from_int(0); /* zero mm0 register */
	int i;
	for (i = 0; i < SrcLength/8; i++) {
		__m64 mm1, mm2, mm3, mm4, mm5, mm6;
		mm1 = _m_punpcklbw(*mSrc1, mm0);	/* unpack low  bytes of Src1 into words */
		mm2 = _m_punpckhbw(*mSrc1, mm0);	/* unpack high bytes of Src1 into words */
		mm3 = _m_punpcklbw(*mSrc2, mm0);	/* unpack low  bytes of Src2 into words */
		mm4 = _m_punpckhbw(*mSrc2, mm0);	/* unpack high bytes of Src2 into words */
		mm1 = _m_psrlwi(mm1, 1);		/* divide mm1 words by 2, Src1 low bytes */
		mm2 = _m_psrlwi(mm2, 1);		/* divide mm2 words by 2, Src1 high bytes */
		mm3 = _m_psrlwi(mm3, 1);		/* divide mm3 words by 2, Src2 low bytes */
		mm4 = _m_psrlwi(mm4, 1);		/* divide mm4 words by 2, Src2 high bytes */
		mm1 = _m_pmullw(mm1, mm3);		/* mul low  bytes of Src1 and Src2  */
		mm2 = _m_pmullw(mm2, mm4);		/* mul high bytes of Src1 and Src2 */
		*mDest = _m_packuswb(mm1, mm2);		/* pack words back into bytes with saturation */
		mSrc1++;
		mSrc2++;
		mDest++;
	}
	_m_empty();					/* clean MMX state */
#endif
	return (0);
#else
	return (-1);
#endif
}

/*!
\brief Filter using MultDivby4: D = saturation255(S1/2 * S2/2)

\param Src1 Pointer to the start of the first source byte array (S1).
\param Src2 Pointer to the start of the second source byte array (S2).
\param Dest Pointer to the start of the destination byte array (D).
\param length The number of bytes in the source arrays.

\return Returns 0 for success or -1 for error.
*/
int SDL_imageFilterMultDivby4(unsigned char *Src1, unsigned char *Src2, unsigned char *Dest, unsigned int length)
{
	unsigned int i, istart;
	unsigned char *cursrc1, *cursrc2, *curdst;
	int result;

	/* Validate input parameters */
	if ((Src1 == NULL) || (Src2 == NULL) || (Dest == NULL))
		return(-1);
	if (length == 0)
		return(0);

	if ((SDL_imageFilterMMXdetect()) && (length > 7)) {
		/* MMX routine */
		SDL_imageFilterMultDivby4MMX(Src1, Src2, Dest, length);

		/* Check for unaligned bytes */
		if ((length & 7) > 0) {
			/* Setup to process unaligned bytes */
			istart = length & 0xfffffff8;
			cursrc1 = &Src1[istart];
			cursrc2 = &Src2[istart];
			curdst = &Dest[istart];
		} else {
			/* No unaligned bytes - we are done */
			return (0);
		}
	} else {
		/* Setup to process whole image */
		istart = 0;
		cursrc1 = Src1;
		cursrc2 = Src2;
		curdst = Dest;
	}

	/* C routine to process image */
	for (i = istart; i < length; i++) {
		result = ((int) *cursrc1 / 2) * ((int) *cursrc2 / 2);
		if (result > 255)
			result = 255;
		*curdst = (unsigned char) result;
		/* Advance pointers */
		cursrc1++;
		cursrc2++;
		curdst++;
	}

	return (0);
}

/*!
\brief Internal MMX Filter using BitAnd: D = S1 & S2

\param Src1 Pointer to the start of the first source byte array (S1).
\param Src2 Pointer to the start of the second source byte array (S2).
\param Dest Pointer to the start of the destination byte array (D).
\param SrcLength The number of bytes in the source arrays.

\return Returns 0 for success or -1 for error.
*/
static int SDL_imageFilterBitAndMMX(unsigned char *Src1, unsigned char *Src2, unsigned char *Dest, unsigned int SrcLength)
{
#ifdef USE_MMX
#if !defined(GCC__)
	__asm
	{
		pusha
			mov eax, Src1   	/* load Src1 address into eax */
			mov ebx, Src2   	/* load Src2 address into ebx */
			mov edi, Dest   	/* load Dest address into edi */
			mov ecx, SrcLength 	/* load loop counter (SIZE) into ecx */
			shr ecx, 3 	/* counter/8 (MMX loads 8 bytes at a time) */
			align 16          	/* 16 byte alignment of the loop entry */
L1017:
		movq mm1, [eax]   	/* load 8 bytes from Src1 into mm1 */
		pand mm1, [ebx]   	/* mm1=Src1&Src2 */
		movq [edi], mm1   	/* store result in Dest */
			add eax, 8   	/* increase Src1, Src2 and Dest  */
			add ebx, 8   	/* register pointers by 8 */
			add edi, 8
			dec ecx        	/* decrease loop counter */
			jnz L1017       	/* check loop termination, proceed if required */
			emms             	/* exit MMX state */
			popa
	}
#else
	/* x86_64 ASM with constraints: */
	/* asm volatile ( */
	/* 	"shr $3, %%rcx \n\t"	/\* counter/8 (MMX loads 8 bytes at a time) *\/ */
	/* 	".align 16       \n\t"	/\* 16 byte alignment of the loop entry *\/ */
	/* 	"1: movq (%%rax), %%mm1 \n\t"	/\* load 8 bytes from Src1 into mm1 *\/ */
	/* 	"pand    (%%rbx), %%mm1 \n\t"	/\* mm1=Src1&Src2 *\/ */
	/* 	"movq    %%mm1, (%%rdi) \n\t"	/\* store result in Dest *\/ */
	/* 	"add $8, %%rax \n\t"	/\* increase Src1, Src2 and Dest  *\/ */
	/* 	"add $8, %%rbx \n\t"	/\* register pointers by 8 *\/ */
	/* 	"add $8, %%rdi \n\t" */
	/* 	"dec %%rcx     \n\t"	/\* decrease loop counter *\/ */
	/* 	"jnz 1b        \n\t"	/\* check loop termination, proceed if required *\/ */
	/* 	"emms          \n\t"	/\* exit MMX state *\/ */
	/* 	: "+a" (Src1),		/\* load Src1 address into rax, modified by the loop *\/ */
	/* 	  "+b" (Src2),		/\* load Src2 address into rbx, modified by the loop *\/ */
	/* 	  "+c" (SrcLength),	/\* load loop counter (SIZE) into rcx, modified by the loop *\/ */
	/* 	  "+D" (Dest)		/\* load Dest address into rdi, modified by the loop *\/ */
	/* 	: */
	/* 	: "memory",		/\* *Dest is modified *\/ */
        /*           "mm1"			/\* register mm1 modified *\/ */
	/* ); */

	/* i386 and x86_64 */
	__m64 *mSrc1 = (__m64*)Src1;
	__m64 *mSrc2 = (__m64*)Src2;
	__m64 *mDest = (__m64*)Dest;
	int i;
	for (i = 0; i < SrcLength/8; i++) {
		*mDest = _m_pand(*mSrc1, *mSrc2);	/* Src1&Src2 */
		mSrc1++;
		mSrc2++;
		mDest++;
	}
	_m_empty();					/* clean MMX state */
#endif
	return (0);
#else
	return (-1);
#endif
}

/*!
\brief Filter using BitAnd: D = S1 & S2

\param Src1 Pointer to the start of the first source byte array (S1).
\param Src2 Pointer to the start of the second source byte array (S2).
\param Dest Pointer to the start of the destination byte array (D).
\param length The number of bytes in the source arrays.

\return Returns 0 for success or -1 for error.
*/
int SDL_imageFilterBitAnd(unsigned char *Src1, unsigned char *Src2, unsigned char *Dest, unsigned int length)
{
	unsigned int i, istart;
	unsigned char *cursrc1, *cursrc2, *curdst;

	/* Validate input parameters */
	if ((Src1 == NULL) || (Src2 == NULL) || (Dest == NULL))
		return(-1);
	if (length == 0)
		return(0);

	if ((SDL_imageFilterMMXdetect()>0) && (length>7)) {
		/*  if (length > 7) { */
		/* Call MMX routine */

		SDL_imageFilterBitAndMMX(Src1, Src2, Dest, length);

		/* Check for unaligned bytes */
		if ((length & 7) > 0) {

			/* Setup to process unaligned bytes */
			istart = length & 0xfffffff8;
			cursrc1 = &Src1[istart];
			cursrc2 = &Src2[istart];
			curdst = &Dest[istart];
		} else {
			/* No unaligned bytes - we are done */
			return (0);
		}
	} else {
		/* Setup to process whole image */
		istart = 0;
		cursrc1 = Src1;
		cursrc2 = Src2;
		curdst = Dest;
	}

	/* C routine to process image */
	for (i = istart; i < length; i++) {
		*curdst = (*cursrc1) & (*cursrc2);
		/* Advance pointers */
		cursrc1++;
		cursrc2++;
		curdst++;
	}

	return (0);
}

/*!
\brief Internal MMX Filter using BitOr: D = S1 | S2

\param Src1 Pointer to the start of the first source byte array (S1).
\param Src2 Pointer to the start of the second source byte array (S2).
\param Dest Pointer to the start of the destination byte array (D).
\param SrcLength The number of bytes in the source arrays.

\return Returns 0 for success or -1 for error.
*/
static int SDL_imageFilterBitOrMMX(unsigned char *Src1, unsigned char *Src2, unsigned char *Dest, unsigned int SrcLength)
{
#ifdef USE_MMX
#if !defined(GCC__)
	__asm
	{
		pusha
			mov eax, Src1   	/* load Src1 address into eax */
			mov ebx, Src2   	/* load Src2 address into ebx */
			mov edi, Dest   	/* load Dest address into edi */
			mov ecx, SrcLength 	/* load loop counter (SIZE) into ecx */
			shr ecx,  3 	/* counter/8 (MMX loads 8 bytes at a time) */
			align 16          	/* 16 byte alignment of the loop entry */
L91017:
		movq mm1, [eax]   	/* load 8 bytes from Src1 into mm1 */
		por mm1, [ebx]   	/* mm1=Src1|Src2 */
		movq [edi], mm1   	/* store result in Dest */
			add eax, 8   	/* increase Src1, Src2 and Dest  */
			add ebx, 8   	/* register pointers by 8 */
			add edi,  8
			dec ecx        	/* decrease loop counter */
			jnz L91017      	/* check loop termination, proceed if required */
			emms             	/* exit MMX state */
			popa
	}
#else
	/* i386 and x86_64 */
	__m64 *mSrc1 = (__m64*)Src1;
	__m64 *mSrc2 = (__m64*)Src2;
	__m64 *mDest = (__m64*)Dest;
	int i;
	for (i = 0; i < SrcLength/8; i++) {
		*mDest = _m_por(*mSrc1, *mSrc2);	/* Src1|Src2 */
		mSrc1++;
		mSrc2++;
		mDest++;
	}
	_m_empty();					/* clean MMX state */
#endif
	return (0);
#else
	return (-1);
#endif
}

/*!
\brief Filter using BitOr: D = S1 | S2

\param Src1 Pointer to the start of the first source byte array (S1).
\param Src2 Pointer to the start of the second source byte array (S2).
\param Dest Pointer to the start of the destination byte array (D).
\param length The number of bytes in the source arrays.

\return Returns 0 for success or -1 for error.
*/
int SDL_imageFilterBitOr(unsigned char *Src1, unsigned char *Src2, unsigned char *Dest, unsigned int length)
{
	unsigned int i, istart;
	unsigned char *cursrc1, *cursrc2, *curdst;

	/* Validate input parameters */
	if ((Src1 == NULL) || (Src2 == NULL) || (Dest == NULL))
		return(-1);
	if (length == 0)
		return(0);

	if ((SDL_imageFilterMMXdetect()) && (length > 7)) {

		/* MMX routine */
		SDL_imageFilterBitOrMMX(Src1, Src2, Dest, length);

		/* Check for unaligned bytes */
		if ((length & 7) > 0) {
			/* Setup to process unaligned bytes */
			istart = length & 0xfffffff8;
			cursrc1 = &Src1[istart];
			cursrc2 = &Src2[istart];
			curdst = &Dest[istart];
		} else {
			/* No unaligned bytes - we are done */
			return (0);
		}
	} else {
		/* Setup to process whole image */
		istart = 0;
		cursrc1 = Src1;
		cursrc2 = Src2;
		curdst = Dest;
	}

	/* C routine to process image */
	for (i = istart; i < length; i++) {
		*curdst = *cursrc1 | *cursrc2;
		/* Advance pointers */
		cursrc1++;
		cursrc2++;
		curdst++;
	}
	return (0);
}

/*!
\brief Internal ASM Filter using Div: D = S1 / S2

\param Src1 Pointer to the start of the first source byte array (S1).
\param Src2 Pointer to the start of the second source byte array (S2).
\param Dest Pointer to the start of the destination byte array (D).
\param SrcLength The number of bytes in the source arrays.

\return Returns 0 for success or -1 for error.
*/
static int SDL_imageFilterDivASM(unsigned char *Src1, unsigned char *Src2, unsigned char *Dest, unsigned int SrcLength)
{
#ifdef USE_MMX
#if !defined(GCC__)
	__asm
	{
		pusha
			mov edx, Src1   	/* load Src1 address into edx */
			mov esi, Src2   	/* load Src2 address into esi */
			mov edi, Dest   	/* load Dest address into edi */
			mov ecx, SrcLength 	/* load loop counter (SIZE) into ecx */
			align 16        	/* 16 byte alignment of the loop entry */
L10191:
		mov bl, [esi]   	/* load a byte from Src2 */
		cmp bl, 0   	/* check if it zero */
			jnz L10192
			mov [edi], 255   	/* division by zero = 255 !!! */
			jmp  L10193
L10192:
		xor ah, ah   	/* prepare AX, zero AH register */
			mov al, [edx]   	/* load a byte from Src1 into AL */
		div   bl             	/* divide AL by BL */
			mov [edi], al   	/* move a byte result to Dest */
L10193:
		inc edx    	/* increment Src1, Src2, Dest */
			inc esi    		/* pointer registers by one */
			inc edi
			dec ecx       	/* decrease loop counter */
			jnz L10191     	/* check loop termination, proceed if required */
			popa
	}
#else
	/* Note: ~15% gain on i386, less efficient than C on x86_64 */
	/* Also depends on whether the function is static (?!) */
	/* Also depends on whether we work on malloc() or static char[] */
	asm volatile (
#  if defined(i386)
		"pushl %%ebx \n\t"		/* %ebx may be the PIC register.  */
		".align 16     \n\t"		/* 16 byte alignment of the loop entry */
		"1: mov (%%esi), %%bl  \n\t"	/* load a byte from Src2 */
		"cmp       $0, %%bl    \n\t"	/* check if it zero */
		"jnz 2f                \n\t"
		"movb  $255, (%%edi)   \n\t"	/* division by zero = 255 !!! */
		"jmp 3f                \n\t"
		"2: xor %%ah, %%ah     \n\t"	/* prepare AX, zero AH register */
		"mov   (%%edx), %%al   \n\t"	/* load a byte from Src1 into AL */
		"div   %%bl            \n\t"	/* divide AL by BL */
		"mov   %%al, (%%edi)   \n\t"	/* move a byte result to Dest */
		"3: inc %%edx          \n\t"	/* increment Src1, Src2, Dest */
		"inc %%esi \n\t"		/* pointer registers by one */
		"inc %%edi \n\t"
		"dec %%ecx \n\t"		/* decrease loop counter */
		"jnz 1b    \n\t"		/* check loop termination, proceed if required */
		"popl %%ebx \n\t"		/* restore %ebx */
		: "+d" (Src1),		/* load Src1 address into edx */
		  "+S" (Src2),		/* load Src2 address into esi */
		  "+c" (SrcLength),	/* load loop counter (SIZE) into ecx */
		  "+D" (Dest)		/* load Dest address into edi */
		:
		: "memory", "rax"
#  elif defined(__x86_64__)
		".align 16     \n\t"		/* 16 byte alignment of the loop entry */
		"1: mov (%%rsi), %%bl  \n\t"	/* load a byte from Src2 */
		"cmp       $0, %%bl    \n\t"	/* check if it zero */
		"jnz 2f                \n\t"
		"movb  $255, (%%rdi)   \n\t"	/* division by zero = 255 !!! */
		"jmp 3f                \n\t"
		"2: xor %%ah, %%ah     \n\t"	/* prepare AX, zero AH register */
		"mov   (%%rdx), %%al   \n\t"	/* load a byte from Src1 into AL */
		"div   %%bl            \n\t"	/* divide AL by BL */
		"mov   %%al, (%%rdi)   \n\t"	/* move a byte result to Dest */
		"3: inc %%rdx          \n\t"	/* increment Src1, Src2, Dest */
		"inc %%rsi \n\t"		/* pointer registers by one */
		"inc %%rdi \n\t"
		"dec %%rcx \n\t"		/* decrease loop counter */
		"jnz 1b    \n\t"		/* check loop termination, proceed if required */
		: "+d" (Src1),		/* load Src1 address into edx */
		  "+S" (Src2),		/* load Src2 address into esi */
		  "+c" (SrcLength),	/* load loop counter (SIZE) into ecx */
		  "+D" (Dest)		/* load Dest address into edi */
		:
		: "memory", "rax", "rbx"
#  endif
		);
#endif
	return (0);
#else
	return (-1);
#endif
}

/*!
\brief Filter using Div: D = S1 / S2

\param Src1 Pointer to the start of the first source byte array (S1).
\param Src2 Pointer to the start of the second source byte array (S2).
\param Dest Pointer to the start of the destination byte array (D).
\param length The number of bytes in the source arrays.

\return Returns 0 for success or -1 for error.
*/
int SDL_imageFilterDiv(unsigned char *Src1, unsigned char *Src2, unsigned char *Dest, unsigned int length)
{
	unsigned int i, istart;
	unsigned char *cursrc1, *cursrc2, *curdst;

	/* Validate input parameters */
	if ((Src1 == NULL) || (Src2 == NULL) || (Dest == NULL))
		return(-1);
	if (length == 0)
		return(0);

	if (SDL_imageFilterMMXdetect()) {
		if (length > 0) {
			/* Call ASM routine */
			SDL_imageFilterDivASM(Src1, Src2, Dest, length);

			/* Never unaligned bytes - we are done */
			return (0);
		} else {
			return (-1);
		}
	} 
	
	/* Setup to process whole image */
	istart = 0;
	cursrc1 = Src1;
	cursrc2 = Src2;
	curdst = Dest;

	/* C routine to process image */
	/* for (i = istart; i < length; i++) { */
	/* 	if (*cursrc2 == 0) { */
	/* 		*curdst = 255; */
	/* 	} else { */
	/* 		result = (int) *cursrc1 / (int) *cursrc2; */
	/* 		*curdst = (unsigned char) result; */
	/* 	} */
	/* 	/\* Advance pointers *\/ */
	/* 	cursrc1++; */
	/* 	cursrc2++; */
	/* 	curdst++; */
	/* } */
	for (i = istart; i < length; i++) {
		if (*cursrc2 == 0) {
			*curdst = 255;
		} else {
			*curdst = (int)*cursrc1 / (int)*cursrc2;  // (int) for efficiency
		}
		/* Advance pointers */
		cursrc1++;
		cursrc2++;
		curdst++;
	}

	return (0);
}

/* ------------------------------------------------------------------------------------ */

/*!
\brief Internal MMX Filter using BitNegation: D = !S

\param Src1 Pointer to the start of the source byte array (S1).
\param Dest Pointer to the start of the destination byte array (D).
\param SrcLength The number of bytes in the source array.

\return Returns 0 for success or -1 for error.
*/
static int SDL_imageFilterBitNegationMMX(unsigned char *Src1, unsigned char *Dest, unsigned int SrcLength)
{
#ifdef USE_MMX
#if !defined(GCC__)
	__asm
	{
		pusha
			pcmpeqb mm1, mm1   	/* generate all 1's in mm1 */
			mov eax, Src1   	/* load Src1 address into eax */
			mov edi, Dest   	/* load Dest address into edi */
			mov ecx, SrcLength 	/* load loop counter (SIZE) into ecx */
			shr ecx,  3 	/* counter/8 (MMX loads 8 bytes at a time) */
			align 16          	/* 16 byte alignment of the loop entry */
L91117:
		movq mm0, [eax]   	/* load 8 bytes from Src1 into mm1 */
		pxor mm0, mm1   	/* negate mm0 by xoring with mm1 */
			movq [edi], mm0   	/* store result in Dest */
			add eax, 8   	/* increase Src1, Src2 and Dest  */
			add edi,  8
			dec ecx        	/* decrease loop counter */
			jnz L91117      	/* check loop termination, proceed if required */
			emms             	/* exit MMX state */
			popa
	}
#else
	/* i386 and x86_64 */
	__m64 *mSrc1 = (__m64*)Src1;
	__m64 *mDest = (__m64*)Dest;
        __m64 mm1;
	mm1 = _m_pcmpeqb(mm1, mm1);		/* generate all 1's in mm1 */
	int i;
	for (i = 0; i < SrcLength/8; i++) {
		*mDest = _m_pxor(*mSrc1, mm1);	/* negate mm0 by xoring with mm1 */
		mSrc1++;
		mDest++;
	}
	_m_empty();				/* clean MMX state */

#endif
	return (0);
#else
	return (-1);
#endif
}

/*!
\brief Filter using BitNegation: D = !S

\param Src1 Pointer to the start of the source byte array (S).
\param Dest Pointer to the start of the destination byte array (D).
\param length The number of bytes in the source array.

\return Returns 0 for success or -1 for error.
*/
int SDL_imageFilterBitNegation(unsigned char *Src1, unsigned char *Dest, unsigned int length)
{
	unsigned int i, istart;
	unsigned char *cursrc1, *curdst;

	/* Validate input parameters */
	if ((Src1 == NULL) || (Dest == NULL))
		return(-1);
	if (length == 0)
		return(0);

	if ((SDL_imageFilterMMXdetect()) && (length > 7)) {
		/* MMX routine */
		SDL_imageFilterBitNegationMMX(Src1, Dest, length);

		/* Check for unaligned bytes */
		if ((length & 7) > 0) {
			/* Setup to process unaligned bytes */
			istart = length & 0xfffffff8;
			cursrc1 = &Src1[istart];
			curdst = &Dest[istart];
		} else {
			/* No unaligned bytes - we are done */
			return (0);
		}
	} else {
		/* Setup to process whole image */
		istart = 0;
		cursrc1 = Src1;
		curdst = Dest;
	}

	/* C routine to process image */
	for (i = istart; i < length; i++) {
		*curdst = ~(*cursrc1);
		/* Advance pointers */
		cursrc1++;
		curdst++;
	}

	return (0);
}

/*!
\brief Internal MMX Filter using AddByte: D = saturation255(S + C) 

\param Src1 Pointer to the start of the source byte array (S).
\param Dest Pointer to the start of the destination byte array (D).
\param SrcLength The number of bytes in the source array.
\param C Constant value to add (C).

\return Returns 0 for success or -1 for error.
*/
static int SDL_imageFilterAddByteMMX(unsigned char *Src1, unsigned char *Dest, unsigned int SrcLength, unsigned char C)
{
#ifdef USE_MMX
#if !defined(GCC__)
	__asm
	{
		pusha
			/* ** Duplicate C in 8 bytes of MM1 ** */
			mov al, C   	/* load C into AL */
			mov ah, al   	/* copy AL into AH */
			mov bx, ax   	/* copy AX into BX */
			shl eax, 16   	/* shift 2 bytes of EAX left */
			mov ax, bx   	/* copy BX into AX */
			movd mm1, eax   	/* copy EAX into MM1 */
			movd mm2, eax   	/* copy EAX into MM2 */
			punpckldq mm1, mm2   	/* fill higher bytes of MM1 with C */
			mov eax, Src1   	/* load Src1 address into eax */
			mov edi, Dest   	/* load Dest address into edi */
			mov ecx, SrcLength 	/* load loop counter (SIZE) into ecx */
			shr ecx,  3 	/* counter/8 (MMX loads 8 bytes at a time) */
			align 16                 	/* 16 byte alignment of the loop entry */
L1021:
		movq mm0, [eax]   	/* load 8 bytes from Src1 into MM0 */
		paddusb mm0,  mm1 	/* MM0=SrcDest+C (add 8 bytes with saturation) */
			movq [edi], mm0   	/* store result in Dest */
			add eax, 8   	/* increase Dest register pointer by 8 */
			add edi, 8   	/* increase Dest register pointer by 8 */
			dec              ecx    	/* decrease loop counter */
			jnz             L1021    	/* check loop termination, proceed if required */
			emms                      	/* exit MMX state */
			popa
	}
#else
	/* i386 and x86_64 */
	__m64 *mSrc1 = (__m64*)Src1;
	__m64 *mDest = (__m64*)Dest;
	/* Duplicate C in 8 bytes of MM1 */
	int i;
	memset(&i, C, 4);
	__m64 mm1 = _m_from_int(i);
	__m64 mm2 = _m_from_int(i);
	mm1 = _m_punpckldq(mm1, mm2);			/* fill higher bytes of MM1 with C */
        //__m64 mm1 = _m_from_int64(lli); // x86_64 only
	for (i = 0; i < SrcLength/8; i++) {
		*mDest = _m_paddusb(*mSrc1, mm1);	/* Src1+C (add 8 bytes with saturation) */
		mSrc1++;
		mDest++;
	}
	_m_empty();					/* clean MMX state */
#endif
	return (0);
#else
	return (-1);
#endif
}

/*!
\brief Filter using AddByte: D = saturation255(S + C) 

\param Src1 Pointer to the start of the source byte array (S).
\param Dest Pointer to the start of the destination byte array (D).
\param length The number of bytes in the source array.
\param C Constant value to add (C).


\return Returns 0 for success or -1 for error.
*/
int SDL_imageFilterAddByte(unsigned char *Src1, unsigned char *Dest, unsigned int length, unsigned char C)
{
	unsigned int i, istart;
	int iC;
	unsigned char *cursrc1, *curdest;
	int result;

	/* Validate input parameters */
	if ((Src1 == NULL) || (Dest == NULL))
		return(-1);
	if (length == 0)
		return(0);

	/* Special case: C==0 */
	if (C == 0) {
		memcpy(Src1, Dest, length);
		return (0); 
	}

	if ((SDL_imageFilterMMXdetect()) && (length > 7)) {

		/* MMX routine */
		SDL_imageFilterAddByteMMX(Src1, Dest, length, C);

		/* Check for unaligned bytes */
		if ((length & 7) > 0) {
			/* Setup to process unaligned bytes */
			istart = length & 0xfffffff8;
			cursrc1 = &Src1[istart];
			curdest = &Dest[istart];
		} else {
			/* No unaligned bytes - we are done */
			return (0);
		}
	} else {
		/* Setup to process whole image */
		istart = 0;
		cursrc1 = Src1;
		curdest = Dest;
	}

	/* C routine to process image */
	iC = (int) C;
	for (i = istart; i < length; i++) {
		result = (int) *cursrc1 + iC;
		if (result > 255)
			result = 255;
		*curdest = (unsigned char) result;
		/* Advance pointers */
		cursrc1++;
		curdest++;
	}
	return (0);
}

/*!
\brief Internal MMX Filter using AddUint: D = saturation255((S[i] + Cs[i % 4]), Cs=Swap32((uint)C)

\param Src1 Pointer to the start of the source byte array (S).
\param Dest Pointer to the start of the destination byte array (D).
\param SrcLength The number of bytes in the source array.
\param C Constant to add (C).
\param D Byteorder-swapped constant to add (Cs).

\return Returns 0 for success or -1 for error.
*/
static int SDL_imageFilterAddUintMMX(unsigned char *Src1, unsigned char *Dest, unsigned int SrcLength, unsigned int C, unsigned int D)
{
#ifdef USE_MMX
#if !defined(GCC__)
	__asm
	{
		pusha
			/* ** Duplicate (int)C in 8 bytes of MM1 ** */
			mov eax, C   	/* load C into EAX */
			movd mm1, eax   	/* copy EAX into MM1 */
			mov eax, D   	/* load D into EAX */
			movd mm2, eax   	/* copy EAX into MM2 */
			punpckldq mm1, mm2   	/* fill higher bytes of MM1 with C */
			mov eax, Src1   	/* load Src1 address into eax */
			mov edi, Dest   	/* load Dest address into edi */
			mov ecx, SrcLength 	/* load loop counter (SIZE) into ecx */
			shr ecx,  3 	/* counter/8 (MMX loads 8 bytes at a time) */
			align 16                 	/* 16 byte alignment of the loop entry */
L11023:
		movq mm0, [eax]   	/* load 8 bytes from SrcDest into MM0 */
		paddusb mm0,  mm1 	/* MM0=SrcDest+C (add 8 bytes with saturation) */
			movq [edi],  mm0 	/* store result in SrcDest */
			add eax, 8   	/* increase Src1 register pointer by 8 */
			add edi, 8   	/* increase Dest register pointer by 8 */
			dec              ecx    	/* decrease loop counter */
			jnz             L11023    	/* check loop termination, proceed if required */
			emms                      	/* exit MMX state */
			popa
	}
#else
	/* i386 and x86_64 */
	__m64 *mSrc1 = (__m64*)Src1;
	__m64 *mDest = (__m64*)Dest;
	/* Duplicate (int)C in 8 bytes of MM1 */
	__m64 mm1 = _m_from_int(C);
	__m64 mm2 = _m_from_int(C);
	mm1 = _m_punpckldq(mm1, mm2);			/* fill higher bytes of MM1 with C */
        //__m64 mm1 = _m_from_int64(lli); // x86_64 only
	int i;
	for (i = 0; i < SrcLength/8; i++) {
		*mDest = _m_paddusb(*mSrc1, mm1);	/* Src1+C (add 8 bytes with saturation) */
		mSrc1++;
		mDest++;
	}
	_m_empty();					/* clean MMX state */
#endif
	return (0);
#else
	return (-1);
#endif
}

/*!
\brief Filter using AddUint: D = saturation255((S[i] + Cs[i % 4]), Cs=Swap32((uint)C)

\param Src1 Pointer to the start of the source byte array (S).
\param Dest Pointer to the start of the destination byte array (D).
\param length The number of bytes in the source array.
\param C Constant to add (C).

\return Returns 0 for success or -1 for error.
*/
int SDL_imageFilterAddUint(unsigned char *Src1, unsigned char *Dest, unsigned int length, unsigned int C)
{
	unsigned int i, j, istart, D;
	int iC[4];
	unsigned char *cursrc1;
	unsigned char *curdest;
	int result;

	/* Validate input parameters */
	if ((Src1 == NULL) || (Dest == NULL))
		return(-1);
	if (length == 0)
		return(0);

	/* Special case: C==0 */
	if (C == 0) {
		memcpy(Src1, Dest, length);
		return (0); 
	}

	if ((SDL_imageFilterMMXdetect()) && (length > 7)) {

		/* MMX routine */
		D=SWAP_32(C);
		SDL_imageFilterAddUintMMX(Src1, Dest, length, C, D);

		/* Check for unaligned bytes */
		if ((length & 7) > 0) {
			/* Setup to process unaligned bytes */
			istart = length & 0xfffffff8;
			cursrc1 = &Src1[istart];
			curdest = &Dest[istart];
		} else {
			/* No unaligned bytes - we are done */
			return (0);
		}
	} else {
		/* Setup to process whole image */
		istart = 0;
		cursrc1 = Src1;
		curdest = Dest;
	}

	/* C routine to process bytes */
	iC[3] = (int) ((C >> 24) & 0xff);
	iC[2] = (int) ((C >> 16) & 0xff);
	iC[1] = (int) ((C >>  8) & 0xff);
	iC[0] = (int) ((C >>  0) & 0xff);
	for (i = istart; i < length; i += 4) {
		for (j = 0; j < 4; j++) {
			if ((i+j)<length) {
				result = (int) *cursrc1 + iC[j];
				if (result > 255) result = 255;
				*curdest = (unsigned char) result;
				/* Advance pointers */
				cursrc1++;
				curdest++;
			}
		}
	}
	return (0);
}

/*!
\brief Internal MMX Filter using AddByteToHalf: D = saturation255(S/2 + C)

\param Src1 Pointer to the start of the source byte array (S).
\param Dest Pointer to the start of the destination byte array (D).
\param SrcLength The number of bytes in the source array.
\param C Constant to add (C).
\param Mask Pointer to 8 mask bytes of value 0x7F.

\return Returns 0 for success or -1 for error.
*/
static int SDL_imageFilterAddByteToHalfMMX(unsigned char *Src1, unsigned char *Dest, unsigned int SrcLength, unsigned char C,
									unsigned char *Mask)
{
#ifdef USE_MMX
#if !defined(GCC__)
	__asm
	{
		pusha
			/* ** Duplicate C in 8 bytes of MM1 ** */
			mov al, C   	/* load C into AL */
			mov ah, al   	/* copy AL into AH */
			mov bx, ax   	/* copy AX into BX */
			shl eax, 16   	/* shift 2 bytes of EAX left */
			mov ax, bx   	/* copy BX into AX */
			movd mm1, eax   	/* copy EAX into MM1 */
			movd mm2, eax   	/* copy EAX into MM2 */
			punpckldq mm1, mm2   	/* fill higher bytes of MM1 with C */
			mov edx, Mask   	/* load Mask address into edx */
			movq mm0, [edx]   	/* load Mask into mm0 */
		mov eax, Src1   	/* load Src1 address into eax */
			mov edi, Dest   	/* load Dest address into edi */
			mov ecx,  SrcLength 	/* load loop counter (SIZE) into ecx */
			shr ecx,  3 	/* counter/8 (MMX loads 8 bytes at a time) */
			align 16                 	/* 16 byte alignment of the loop entry */
L1022:
		movq mm2, [eax]   	/* load 8 bytes from Src1 into MM2 */
		psrlw mm2, 1   	/* shift 4 WORDS of MM2 1 bit to the right */
			pand mm2, mm0        // apply Mask to 8 BYTES of MM2 */
			paddusb mm2,  mm1 	/* MM2=SrcDest+C (add 8 bytes with saturation) */
			movq [edi], mm2   	/* store result in Dest */
			add eax, 8   	/* increase Src1 register pointer by 8 */
			add edi, 8   	/* increase Dest register pointer by 8 */
			dec              ecx    	/* decrease loop counter */
			jnz             L1022    	/* check loop termination, proceed if required */
			emms                      	/* exit MMX state */
			popa
	}
#else
	/* i386 and x86_64 */
	__m64 *mSrc1 = (__m64*)Src1;
	__m64 *mDest = (__m64*)Dest;
	__m64 *mMask = (__m64*)Mask;
	/* Duplicate C in 8 bytes of MM1 */
	int i;
	memset(&i, C, 4);
	__m64 mm1 = _m_from_int(i);
	__m64 mm2 = _m_from_int(i);
	mm1 = _m_punpckldq(mm1, mm2);			/* fill higher bytes of MM1 with C */
        //__m64 mm1 = _m_from_int64(lli); // x86_64 only
	for (i = 0; i < SrcLength/8; i++) {
		__m64 mm2 = _m_psrlwi(*mSrc1, 1);	/* shift 4 WORDS of MM2 1 bit to the right */
		mm2 = _m_pand(mm2, *mMask);		/* apply Mask to 8 BYTES of MM2 */
							/* byte     0x0f, 0xdb, 0xd0 */
		*mDest = _m_paddusb(mm1, mm2);		/* Src1+C (add 8 bytes with saturation) */
		mSrc1++;
		mDest++;
	}
	_m_empty();					/* clean MMX state */
#endif
	return (0);
#else
	return (-1);
#endif
}

/*!
\brief Filter using AddByteToHalf: D = saturation255(S/2 + C)

\param Src1 Pointer to the start of the source byte array (S).
\param Dest Pointer to the start of the destination byte array (D).
\param length The number of bytes in the source array.
\param C Constant to add (C).

\return Returns 0 for success or -1 for error.
*/
int SDL_imageFilterAddByteToHalf(unsigned char *Src1, unsigned char *Dest, unsigned int length, unsigned char C)
{
	static unsigned char Mask[8] = { 0x7F, 0x7F, 0x7F, 0x7F, 0x7F, 0x7F, 0x7F, 0x7F };
	unsigned int i, istart;
	int iC;
	unsigned char *cursrc1;
	unsigned char *curdest;
	int result;

	/* Validate input parameters */
	if ((Src1 == NULL) || (Dest == NULL))
		return(-1);
	if (length == 0)
		return(0);

	if ((SDL_imageFilterMMXdetect()) && (length > 7)) {

		/* MMX routine */
		SDL_imageFilterAddByteToHalfMMX(Src1, Dest, length, C, Mask);

		/* Check for unaligned bytes */
		if ((length & 7) > 0) {
			/* Setup to process unaligned bytes */
			istart = length & 0xfffffff8;
			cursrc1 = &Src1[istart];
			curdest = &Dest[istart];
		} else {
			/* No unaligned bytes - we are done */
			return (0);
		}
	} else {
		/* Setup to process whole image */
		istart = 0;
		cursrc1 = Src1;
		curdest = Dest;
	}

	/* C routine to process image */
	iC = (int) C;
	for (i = istart; i < length; i++) {
		result = (int) (*cursrc1 / 2) + iC;
		if (result > 255)
			result = 255;
		*curdest = (unsigned char) result;
		/* Advance pointers */
		cursrc1++;
		curdest++;
	}

	return (0);
}

/*!
\brief Internal MMX Filter using SubByte: D = saturation0(S - C)

\param Src1 Pointer to the start of the source byte array (S).
\param Dest Pointer to the start of the destination byte array (D).
\param SrcLength The number of bytes in the source array.
\param C Constant to subtract (C).

\return Returns 0 for success or -1 for error.
*/
int SDL_imageFilterSubByteMMX(unsigned char *Src1, unsigned char *Dest, unsigned int SrcLength, unsigned char C)
{
#ifdef USE_MMX
#if !defined(GCC__)
	__asm
	{
		pusha
			/* ** Duplicate C in 8 bytes of MM1 ** */
			mov al, C   	/* load C into AL */
			mov ah, al   	/* copy AL into AH */
			mov bx, ax   	/* copy AX into BX */
			shl eax, 16   	/* shift 2 bytes of EAX left */
			mov ax, bx   	/* copy BX into AX */
			movd mm1, eax   	/* copy EAX into MM1 */
			movd mm2, eax   	/* copy EAX into MM2 */
			punpckldq mm1, mm2   	/* fill higher bytes of MM1 with C */
			mov eax, Src1   	/* load Src1 address into eax */
			mov edi, Dest   	/* load Dest address into edi */
			mov ecx,  SrcLength 	/* load loop counter (SIZE) into ecx */
			shr ecx,  3 	/* counter/8 (MMX loads 8 bytes at a time) */
			align 16                 	/* 16 byte alignment of the loop entry */
L1023:
		movq mm0, [eax]   	/* load 8 bytes from SrcDest into MM0 */
		psubusb mm0,  mm1 	/* MM0=SrcDest-C (sub 8 bytes with saturation) */
			movq [edi], mm0   	/* store result in SrcDest */
			add eax, 8   	/* increase Src1 register pointer by 8 */
			add edi, 8   	/* increase Dest register pointer by 8 */
			dec              ecx    	/* decrease loop counter */
			jnz             L1023    	/* check loop termination, proceed if required */
			emms                      	/* exit MMX state */
			popa
	}
#else
	/* i386 and x86_64 */
	__m64 *mSrc1 = (__m64*)Src1;
	__m64 *mDest = (__m64*)Dest;
	/* Duplicate C in 8 bytes of MM1 */
	int i;
	memset(&i, C, 4);
	__m64 mm1 = _m_from_int(i);
	__m64 mm2 = _m_from_int(i);
	mm1 = _m_punpckldq(mm1, mm2);			/* fill higher bytes of MM1 with C */
        //__m64 mm1 = _m_from_int64(lli); // x86_64 only
	for (i = 0; i < SrcLength/8; i++) {
		*mDest = _m_psubusb(*mSrc1, mm1);	/* Src1-C (sub 8 bytes with saturation) */
		mSrc1++;
		mDest++;
	}
	_m_empty();					/* clean MMX state */
#endif
	return (0);
#else
	return (-1);
#endif
}

/*!
\brief Filter using SubByte: D = saturation0(S - C)

\param Src1 Pointer to the start of the source byte array (S).
\param Dest Pointer to the start of the destination byte array (D).
\param length The number of bytes in the source arrays.
\param C Constant to subtract (C).

\return Returns 0 for success or -1 for error.
*/
int SDL_imageFilterSubByte(unsigned char *Src1, unsigned char *Dest, unsigned int length, unsigned char C)
{
	unsigned int i, istart;
	int iC;
	unsigned char *cursrc1;
	unsigned char *curdest;
	int result;

	/* Validate input parameters */
	if ((Src1 == NULL) || (Dest == NULL))
		return(-1);
	if (length == 0)
		return(0);

	/* Special case: C==0 */
	if (C == 0) {
		memcpy(Src1, Dest, length);
		return (0); 
	}

	if ((SDL_imageFilterMMXdetect()) && (length > 7)) {

		/* MMX routine */
		SDL_imageFilterSubByteMMX(Src1, Dest, length, C);

		/* Check for unaligned bytes */
		if ((length & 7) > 0) {
			/* Setup to process unaligned bytes */
			istart = length & 0xfffffff8;
			cursrc1 = &Src1[istart];
			curdest = &Dest[istart];
		} else {
			/* No unaligned bytes - we are done */
			return (0);
		}
	} else {
		/* Setup to process whole image */
		istart = 0;
		cursrc1 = Src1;
		curdest = Dest;
	}

	/* C routine to process image */
	iC = (int) C;
	for (i = istart; i < length; i++) {
		result = (int) *cursrc1 - iC;
		if (result < 0)
			result = 0;
		*curdest = (unsigned char) result;
		/* Advance pointers */
		cursrc1++;
		curdest++;
	}
	return (0);
}

/*!
\brief Internal MMX Filter using SubUint: D = saturation0(S[i] - Cs[i % 4]), Cs=Swap32((uint)C)

\param Src1 Pointer to the start of the source byte array (S).
\param Dest Pointer to the start of the destination byte array (D).
\param SrcLength The number of bytes in the source array.
\param C Constant to subtract (C).
\param D Byteorder-swapped constant to subtract (Cs).

\return Returns 0 for success or -1 for error.
*/
static int SDL_imageFilterSubUintMMX(unsigned char *Src1, unsigned char *Dest, unsigned int SrcLength, unsigned int C, unsigned int D)
{
#ifdef USE_MMX
#if !defined(GCC__)
	__asm
	{
		pusha
			/* ** Duplicate (int)C in 8 bytes of MM1 ** */
			mov eax, C   	/* load C into EAX */
			movd mm1, eax   	/* copy EAX into MM1 */
			mov eax, D   	/* load D into EAX */
			movd mm2, eax   	/* copy EAX into MM2 */
			punpckldq mm1, mm2   	/* fill higher bytes of MM1 with C */
			mov eax, Src1   	/* load Src1 address into eax */
			mov edi, Dest   	/* load Dest address into edi */
			mov ecx,  SrcLength 	/* load loop counter (SIZE) into ecx */
			shr ecx,  3 	/* counter/8 (MMX loads 8 bytes at a time) */
			align 16                 	/* 16 byte alignment of the loop entry */
L11024:
		movq mm0, [eax]   	/* load 8 bytes from SrcDest into MM0 */
		psubusb mm0, mm1 	/* MM0=SrcDest-C (sub 8 bytes with saturation) */
			movq [edi], mm0   	/* store result in SrcDest */
			add eax, 8   	/* increase Src1 register pointer by 8 */
			add edi, 8   	/* increase Dest register pointer by 8 */
			dec              ecx    	/* decrease loop counter */
			jnz             L11024    	/* check loop termination, proceed if required */
			emms                      	/* exit MMX state */
			popa
	}
#else
	/* i386 and x86_64 */
	__m64 *mSrc1 = (__m64*)Src1;
	__m64 *mDest = (__m64*)Dest;
	/* Duplicate (int)C in 8 bytes of MM1 */
	__m64 mm1 = _m_from_int(C);
	__m64 mm2 = _m_from_int(C);
	mm1 = _m_punpckldq(mm1, mm2);			/* fill higher bytes of MM1 with C */
        //__m64 mm1 = _m_from_int64(lli); // x86_64 only
	int i;
	for (i = 0; i < SrcLength/8; i++) {
		*mDest = _m_psubusb(*mSrc1, mm1);	/* Src1-C (sub 8 bytes with saturation) */
		mSrc1++;
		mDest++;
	}
	_m_empty();					/* clean MMX state */
#endif
	return (0);
#else
	return (-1);
#endif
}

/*!
\brief Filter using SubUint: D = saturation0(S[i] - Cs[i % 4]), Cs=Swap32((uint)C)

\param Src1 Pointer to the start of the source byte array (S1).
\param Dest Pointer to the start of the destination byte array (D).
\param length The number of bytes in the source array.
\param C Constant to subtract (C).

\return Returns 0 for success or -1 for error.
*/
int SDL_imageFilterSubUint(unsigned char *Src1, unsigned char *Dest, unsigned int length, unsigned int C)
{
	unsigned int i, j, istart, D;
	int iC[4];
	unsigned char *cursrc1;
	unsigned char *curdest;
	int result;

	/* Validate input parameters */
	if ((Src1 == NULL) || (Dest == NULL))
		return(-1);
	if (length == 0)
		return(0);

    /* Special case: C==0 */
	if (C == 0) {
		memcpy(Src1, Dest, length);
		return (0); 
	}

	if ((SDL_imageFilterMMXdetect()) && (length > 7)) {

		/* MMX routine */
		D=SWAP_32(C);
		SDL_imageFilterSubUintMMX(Src1, Dest, length, C, D);

		/* Check for unaligned bytes */
		if ((length & 7) > 0) {
			/* Setup to process unaligned bytes */
			istart = length & 0xfffffff8;
			cursrc1 = &Src1[istart];
			curdest = &Dest[istart];
		} else {
			/* No unaligned bytes - we are done */
			return (0);
		}
	} else {
		/* Setup to process whole image */
		istart = 0;
		cursrc1 = Src1;
		curdest = Dest;
	}

	/* C routine to process image */
	iC[3] = (int) ((C >> 24) & 0xff);
	iC[2] = (int) ((C >> 16) & 0xff);
	iC[1] = (int) ((C >>  8) & 0xff);
	iC[0] = (int) ((C >>  0) & 0xff);
	for (i = istart; i < length; i += 4) {
		for (j = 0; j < 4; j++) {
			if ((i+j)<length) {
				result = (int) *cursrc1 - iC[j];
				if (result < 0) result = 0;
				*curdest = (unsigned char) result;
				/* Advance pointers */
				cursrc1++;
				curdest++;
			}
		}
	}
	return (0);
}

/*!
\brief Internal MMX Filter using ShiftRight: D = saturation0(S >> N)

\param Src1 Pointer to the start of the source byte array (S).
\param Dest Pointer to the start of the destination byte array (D).
\param SrcLength The number of bytes in the source array.
\param N Number of bit-positions to shift (N). Valid range is 0 to 8.
\param Mask Byte array containing 8 bytes with 0x7F value.

\return Returns 0 for success or -1 for error.
*/
static int SDL_imageFilterShiftRightMMX(unsigned char *Src1, unsigned char *Dest, unsigned int SrcLength, unsigned char N,
								 unsigned char *Mask)
{
#ifdef USE_MMX
#if !defined(GCC__)
	__asm
	{
		pusha
			mov edx, Mask   	/* load Mask address into edx */
			movq mm0, [edx]   	/* load Mask into mm0 */
		xor ecx, ecx   	/* zero ECX */
			mov cl,  N 	/* load loop counter (N) into CL */
			movd mm3,  ecx 	/* copy (N) into MM3  */
			pcmpeqb mm1, mm1   	/* generate all 1's in mm1 */
L10240:                  	/* ** Prepare proper bit-Mask in MM1 ** */
		psrlw mm1,  1 	/* shift 4 WORDS of MM1 1 bit to the right */
			pand mm1, mm0   // apply Mask to 8 BYTES of MM1 */
			/*  byte     0x0f, 0xdb, 0xc8 */
			dec               cl    	/* decrease loop counter */
			jnz            L10240    	/* check loop termination, proceed if required */
			/* ** Shift all bytes of the image ** */
			mov eax, Src1   	/* load Src1 address into eax */
			mov edi, Dest   	/* load Dest address into edi */
			mov ecx,  SrcLength 	/* load loop counter (SIZE) into ecx */
			shr ecx,  3 	/* counter/8 (MMX loads 8 bytes at a time) */
			align 16                 	/* 16 byte alignment of the loop entry */
L10241:
		movq mm0, [eax]   	/* load 8 bytes from SrcDest into MM0 */
		psrlw mm0, mm3   	/* shift 4 WORDS of MM0 (N) bits to the right */
			pand mm0, mm1    // apply proper bit-Mask to 8 BYTES of MM0 */
			/* byte     0x0f, 0xdb, 0xc1 */
			movq [edi], mm0   	/* store result in SrcDest */
			add eax, 8   	/* increase Src1 register pointer by 8 */
			add edi, 8   	/* increase Dest register pointer by 8 */
			dec              ecx    	/* decrease loop counter */
			jnz            L10241    	/* check loop termination, proceed if required */
			emms                      	/* exit MMX state */
			popa
	}
#else
	/* i386 and x86_64 */
	__m64 *mSrc1 = (__m64*)Src1;
	__m64 *mDest = (__m64*)Dest;
	__m64 *mMask = (__m64*)Mask;
        __m64 mm1;
	int i;
	mm1 = _m_pcmpeqb(mm1, mm1);			/* generate all 1's in mm1 */
	/* Prepare proper bit-Mask in MM1 */
	for (i = 0; i < N; i++) {
		mm1 = _m_psrlwi(mm1, 1);		/* shift 4 WORDS of MM1 1 bit to the right */
		mm1 = _m_pand(mm1, *mMask);		/* apply Mask to 8 BYTES of MM1 */
	}
        /* Shift all bytes of the image */
	for (i = 0; i < SrcLength/8; i++) {
		__m64 mm0 = _m_psrlwi(*mSrc1, N);	/* shift 4 WORDS of MM0 (N) bits to the right */
		*mDest = _m_pand(mm0, mm1);		/* apply proper bit-Mask to 8 BYTES of MM0 */
		mSrc1++;
		mDest++;
	}
	_m_empty();					/* clean MMX state */
#endif
	return (0);
#else
	return (-1);
#endif
}

/*!
\brief Filter using ShiftRight: D = saturation0(S >> N)

\param Src1 Pointer to the start of the source byte array (S).
\param Dest Pointer to the start of the destination byte array (D).
\param length The number of bytes in the source array.
\param N Number of bit-positions to shift (N). Valid range is 0 to 8.

\return Returns 0 for success or -1 for error.
*/
int SDL_imageFilterShiftRight(unsigned char *Src1, unsigned char *Dest, unsigned int length, unsigned char N)
{
	static unsigned char Mask[8] = { 0x7F, 0x7F, 0x7F, 0x7F, 0x7F, 0x7F, 0x7F, 0x7F };
	unsigned int i, istart;
	unsigned char *cursrc1;
	unsigned char *curdest;

	/* Validate input parameters */
	if ((Src1 == NULL) || (Dest == NULL))
		return(-1);
	if (length == 0)
		return(0);

	/* Check shift */
	if (N > 8) {
		return (-1);
	}

	/* Special case: N==0 */
	if (N == 0) {
		memcpy(Src1, Dest, length);
		return (0); 
	}

	if ((SDL_imageFilterMMXdetect()) && (length > 7)) {

		/* MMX routine */
		SDL_imageFilterShiftRightMMX(Src1, Dest, length, N, Mask);

		/* Check for unaligned bytes */
		if ((length & 7) > 0) {
			/* Setup to process unaligned bytes */
			istart = length & 0xfffffff8;
			cursrc1 = &Src1[istart];
			curdest = &Dest[istart];
		} else {
			/* No unaligned bytes - we are done */
			return (0);
		}
	} else {
		/* Setup to process whole image */
		istart = 0;
		cursrc1 = Src1;
		curdest = Dest;
	}

	/* C routine to process image */
	for (i = istart; i < length; i++) {
		*curdest = (unsigned char) *cursrc1 >> N;
		/* Advance pointers */
		cursrc1++;
		curdest++;
	}

	return (0);
}

/*!
\brief Internal MMX Filter using ShiftRightUint: D = saturation0((uint)S[i] >> N)

\param Src1 Pointer to the start of the source byte array (S1).
\param Dest Pointer to the start of the destination byte array (D).
\param SrcLength The number of bytes in the source array.
\param N Number of bit-positions to shift (N).

\return Returns 0 for success or -1 for error.
*/
static int SDL_imageFilterShiftRightUintMMX(unsigned char *Src1, unsigned char *Dest, unsigned int SrcLength, unsigned char N)
{
#ifdef USE_MMX
#if !defined(GCC__)
	__asm
	{
		pusha
			mov eax, Src1   	/* load Src1 address into eax */
			mov edi, Dest   	/* load Dest address into edi */
			mov ecx, SrcLength   	/* load loop counter (SIZE) into ecx */
			shr ecx, 3   	/* counter/8 (MMX loads 8 bytes at a time) */
			align 16                 	/* 16 byte alignment of the loop entry */
L13023:
		movq mm0, [eax]   	/* load 8 bytes from SrcDest into MM0 */
		psrld mm0, N
			movq [edi], mm0   	/* store result in SrcDest */
			add eax, 8   	/* increase Src1 register pointer by 8 */
			add edi, 8   	/* increase Dest register pointer by 8 */
			dec              ecx    	/* decrease loop counter */
			jnz             L13023    	/* check loop termination, proceed if required */
			emms                      	/* exit MMX state */
			popa
	}
#else
	/* i386 and x86_64 */
	__m64 *mSrc1 = (__m64*)Src1;
	__m64 *mDest = (__m64*)Dest;
	int i;
	for (i = 0; i < SrcLength/8; i++) {
		*mDest = _m_psrldi(*mSrc1, N);
		mSrc1++;
		mDest++;
	}
	_m_empty();					/* clean MMX state */
#endif
	return (0);
#else
	return (-1);
#endif
}

/*!
\brief Filter using ShiftRightUint: D = saturation0((uint)S[i] >> N)

\param Src1 Pointer to the start of the source byte array (S1).
\param Dest Pointer to the start of the destination byte array (D).
\param length The number of bytes in the source array.
\param N Number of bit-positions to shift (N). Valid range is 0 to 32.

\return Returns 0 for success or -1 for error.
*/
int SDL_imageFilterShiftRightUint(unsigned char *Src1, unsigned char *Dest, unsigned int length, unsigned char N)
{
	unsigned int i, istart;
	unsigned char *cursrc1, *curdest;
	unsigned int *icursrc1, *icurdest;
	unsigned int result;

	/* Validate input parameters */
	if ((Src1 == NULL) || (Dest == NULL))
		return(-1);
	if (length == 0)
		return(0);

	if (N > 32) {
		return (-1);
	}

	/* Special case: N==0 */
	if (N == 0) {
		memcpy(Src1, Dest, length);
		return (0); 
	}

	if ((SDL_imageFilterMMXdetect()) && (length > 7)) {

		SDL_imageFilterShiftRightUintMMX(Src1, Dest, length, N);

		/* Check for unaligned bytes */
		if ((length & 7) > 0) {
			/* Setup to process unaligned bytes */
			istart = length & 0xfffffff8;
			cursrc1 = &Src1[istart];
			curdest = &Dest[istart];
		} else {
			/* No unaligned bytes - we are done */
			return (0);
		}
	} else {
		/* Setup to process whole image */
		istart = 0;
		cursrc1 = Src1;
		curdest = Dest;
	}

	/* C routine to process image */
	icursrc1=(unsigned int *)cursrc1;
	icurdest=(unsigned int *)curdest;
	for (i = istart; i < length; i += 4) {
		if ((i+4)<length) {
			result = ((unsigned int)*icursrc1 >> N);
			*icurdest = result;
		}
		/* Advance pointers */
		icursrc1++;
		icurdest++;
	}

	return (0);
}

/*!
\brief Internal MMX Filter using MultByByte: D = saturation255(S * C)

\param Src1 Pointer to the start of the source byte array (S).
\param Dest Pointer to the start of the destination byte array (D).
\param SrcLength The number of bytes in the source array.
\param C Constant to multiply with (C).

\return Returns 0 for success or -1 for error.
*/
static int SDL_imageFilterMultByByteMMX(unsigned char *Src1, unsigned char *Dest, unsigned int SrcLength, unsigned char C)
{
#ifdef USE_MMX
#if !defined(GCC__)
	__asm
	{
		pusha
			/* ** Duplicate C in 4 words of MM1 ** */
			mov al, C   	/* load C into AL */
			xor ah, ah   	/* zero AH */
			mov bx, ax   	/* copy AX into BX */
			shl eax, 16   	/* shift 2 bytes of EAX left */
			mov ax, bx   	/* copy BX into AX */
			movd mm1, eax   	/* copy EAX into MM1 */
			movd mm2, eax   	/* copy EAX into MM2 */
			punpckldq mm1, mm2   	/* fill higher words of MM1 with C */
			pxor mm0, mm0   	/* zero MM0 register */
			mov eax, Src1   	/* load Src1 address into eax */
			mov edi, Dest   	/* load Dest address into edi */
			mov ecx, SrcLength   	/* load loop counter (SIZE) into ecx */
			shr ecx, 3   	/* counter/8 (MMX loads 8 bytes at a time) */
			cmp al, 128   	/* if (C <= 128) execute more efficient code */
			jg             L10251
			align 16                 	/* 16 byte alignment of the loop entry */
L10250:
		movq mm3, [eax]   	/* load 8 bytes from Src1 into MM3 */
		movq mm4, mm3   	/* copy MM3 into MM4  */
			punpcklbw mm3, mm0   	/* unpack low  bytes of SrcDest into words */
			punpckhbw mm4, mm0   	/* unpack high bytes of SrcDest into words */
			pmullw mm3, mm1   	/* mul low  bytes of SrcDest and MM1 */
			pmullw mm4, mm1   	/* mul high bytes of SrcDest and MM1 */
			packuswb mm3, mm4   	/* pack words back into bytes with saturation */
			movq [edi], mm3   	/* store result in Dest */
			add eax, 8   	/* increase Src1 register pointer by 8 */
			add edi, 8   	/* increase Dest register pointer by 8 */
			dec              ecx    	/* decrease loop counter */
			jnz            L10250    	/* check loop termination, proceed if required */
			jmp            L10252
			align 16                 	/* 16 byte alignment of the loop entry */
L10251:
		movq mm3, [eax]   	/* load 8 bytes from Src1 into MM3 */
		movq mm4, mm3   	/* copy MM3 into MM4  */
			punpcklbw mm3, mm0   	/* unpack low  bytes of SrcDest into words */
			punpckhbw mm4, mm0   	/* unpack high bytes of SrcDest into words */
			pmullw mm3, mm1   	/* mul low  bytes of SrcDest and MM1 */
			pmullw mm4, mm1   	/* mul high bytes of SrcDest and MM1 */
			/* ** Take abs value of the results (signed words) ** */
			movq mm5, mm3   	/* copy mm3 into mm5 */
			movq mm6, mm4   	/* copy mm4 into mm6 */
			psraw mm5, 15   	/* fill mm5 words with word sign bit */
			psraw mm6, 15   	/* fill mm6 words with word sign bit */
			pxor mm3, mm5   	/* take 1's compliment of only neg words */
			pxor mm4, mm6   	/* take 1's compliment of only neg words */
			psubsw mm3, mm5   	/* add 1 to only neg words, W-(-1) or W-0 */
			psubsw mm4, mm6   	/* add 1 to only neg words, W-(-1) or W-0 */
			packuswb mm3, mm4   	/* pack words back into bytes with saturation */
			movq [edi], mm3   	/* store result in Dest */
			add eax, 8   	/* increase Src1 register pointer by 8 */
			add edi, 8   	/* increase Dest register pointer by 8 */
			dec              ecx    	/* decrease loop counter */
			jnz            L10251    	/* check loop termination, proceed if required */
L10252:
		emms                      	/* exit MMX state */
			popa
	}
#else
	/* i386 and x86_64 */
	__m64 *mSrc1 = (__m64*)Src1;
	__m64 *mDest = (__m64*)Dest;
	__m64 mm0 = _m_from_int(0);				/* zero mm0 register */
	/* Duplicate C in 4 words of MM1 */
	int i;
	i = C | C<<16;
	__m64 mm1 = _m_from_int(i);
	__m64 mm2 = _m_from_int(i);
	mm1 = _m_punpckldq(mm1, mm2);				/* fill higher words of MM1 with C */
	// long long lli = C | C<<16 | (long long)C<<32 | (long long)C<<48;
        //__m64 mm1 = _m_from_int64(lli); // x86_64 only
	if (C <= 128) {						/* if (C <= 128) execute more efficient code */
		for (i = 0; i < SrcLength/8; i++) {
			__m64 mm3, mm4;
			mm3 = _m_punpcklbw(*mSrc1, mm0);	/* unpack low  bytes of Src1 into words */
			mm4 = _m_punpckhbw(*mSrc1, mm0);	/* unpack high bytes of Src1 into words */
			mm3 = _m_pmullw(mm3, mm1);		/* mul low  bytes of Src1 and MM1 */
			mm4 = _m_pmullw(mm4, mm1);		/* mul high bytes of Src1 and MM1 */
			*mDest = _m_packuswb(mm3, mm4);		/* pack words back into bytes with saturation */
			mSrc1++;
			mDest++;
		}
	} else {
		for (i = 0; i < SrcLength/8; i++) {
			__m64 mm3, mm4, mm5, mm6;
			mm3 = _m_punpcklbw(*mSrc1, mm0);	/* unpack low  bytes of Src1 into words */
			mm4 = _m_punpckhbw(*mSrc1, mm0);	/* unpack high bytes of Src1 into words */
			mm3 = _m_pmullw(mm3, mm1);		/* mul low  bytes of Src1 and MM1 */
			mm4 = _m_pmullw(mm4, mm1);		/* mul high bytes of Src1 and MM1 */
			/* Take abs value of the results (signed words) */
			mm5 = _m_psrawi(mm3, 15);		/* fill mm5 words with word sign bit */
			mm6 = _m_psrawi(mm4, 15);		/* fill mm6 words with word sign bit */
			mm3 = _m_pxor(mm3, mm5);		/* take 1's compliment of only neg. words */
			mm4 = _m_pxor(mm4, mm6);		/* take 1's compliment of only neg. words */
			mm3 = _m_psubsw(mm3, mm5);		/* add 1 to only neg. words, W-(-1) or W-0 */
			mm4 = _m_psubsw(mm4, mm6);		/* add 1 to only neg. words, W-(-1) or W-0 */
			*mDest = _m_packuswb(mm3, mm4);		/* pack words back into bytes with saturation */
			mSrc1++;
			mDest++;
		}
	}
	_m_empty();						/* clean MMX state */
#endif
	return (0);
#else
	return (-1);
#endif
}

/*!
\brief Filter using MultByByte: D = saturation255(S * C)

\param Src1 Pointer to the start of the source byte array (S).
\param Dest Pointer to the start of the destination byte array (D).
\param length The number of bytes in the source arrays.
\param C Constant to multiply with (C).

\return Returns 0 for success or -1 for error.
*/
int SDL_imageFilterMultByByte(unsigned char *Src1, unsigned char *Dest, unsigned int length, unsigned char C)
{
	unsigned int i, istart;
	int iC;
	unsigned char *cursrc1;
	unsigned char *curdest;
	int result;

	/* Validate input parameters */
	if ((Src1 == NULL) || (Dest == NULL))
		return(-1);
	if (length == 0)
		return(0);

	/* Special case: C==1 */
	if (C == 1) {
		memcpy(Src1, Dest, length);
		return (0); 
	}

	if ((SDL_imageFilterMMXdetect()) && (length > 7)) {

		SDL_imageFilterMultByByteMMX(Src1, Dest, length, C);

		/* Check for unaligned bytes */
		if ((length & 7) > 0) {
			/* Setup to process unaligned bytes */
			istart = length & 0xfffffff8;
			cursrc1 = &Src1[istart];
			curdest = &Dest[istart];
		} else {
			/* No unaligned bytes - we are done */
			return (0);
		}
	} else {
		/* Setup to process whole image */
		istart = 0;
		cursrc1 = Src1;
		curdest = Dest;
	}

	/* C routine to process image */
	iC = (int) C;
	for (i = istart; i < length; i++) {
		result = (int) *cursrc1 * iC;
		if (result > 255)
			result = 255;
		*curdest = (unsigned char) result;
		/* Advance pointers */
		cursrc1++;
		curdest++;
	}

	return (0);
}

/*!
\brief Internal MMX Filter using ShiftRightAndMultByByteMMX: D = saturation255((S >> N) * C) 

\param Src1 Pointer to the start of the source byte array (S).
\param Dest Pointer to the start of the destination byte array (D).
\param SrcLength The number of bytes in the source array.
\param N Number of bit-positions to shift (N). Valid range is 0 to 8.
\param C Constant to multiply with (C).

\return Returns 0 for success or -1 for error.
*/
static int SDL_imageFilterShiftRightAndMultByByteMMX(unsigned char *Src1, unsigned char *Dest, unsigned int SrcLength, unsigned char N,
											  unsigned char C)
{
#ifdef USE_MMX
#if !defined(GCC__)
	__asm
	{
		pusha
			/* ** Duplicate C in 4 words of MM1 ** */
			mov al, C   	/* load C into AL */
			xor ah, ah   	/* zero AH */
			mov bx, ax   	/* copy AX into BX */
			shl eax, 16   	/* shift 2 bytes of EAX left */
			mov ax, bx   	/* copy BX into AX */
			movd mm1, eax   	/* copy EAX into MM1 */
			movd mm2, eax   	/* copy EAX into MM2 */
			punpckldq mm1, mm2   	/* fill higher words of MM1 with C */
			xor ecx, ecx   	/* zero ECX */
			mov cl, N   	/* load N into CL */
			movd mm7, ecx   	/* copy N into MM7 */
			pxor mm0, mm0   	/* zero MM0 register */
			mov eax, Src1   	/* load Src1 address into eax */
			mov edi, Dest   	/* load Dest address into edi */
			mov ecx, SrcLength   	/* load loop counter (SIZE) into ecx */
			shr ecx, 3   	/* counter/8 (MMX loads 8 bytes at a time) */
			align 16                 	/* 16 byte alignment of the loop entry */
L1026:
		movq mm3, [eax]   	/* load 8 bytes from Src1 into MM3 */
		movq mm4, mm3   	/* copy MM3 into MM4  */
			punpcklbw mm3, mm0   	/* unpack low  bytes of SrcDest into words */
			punpckhbw mm4, mm0   	/* unpack high bytes of SrcDest into words */
			psrlw mm3, mm7   	/* shift 4 WORDS of MM3 (N) bits to the right */
			psrlw mm4, mm7   	/* shift 4 WORDS of MM4 (N) bits to the right */
			pmullw mm3, mm1   	/* mul low  bytes of SrcDest by MM1 */
			pmullw mm4, mm1   	/* mul high bytes of SrcDest by MM1 */
			packuswb mm3, mm4   	/* pack words back into bytes with saturation */
			movq [edi], mm3   	/* store result in Dest */
			add eax, 8   	/* increase Src1 register pointer by 8 */
			add edi, 8   	/* increase Dest register pointer by 8 */
			dec              ecx    	/* decrease loop counter */
			jnz             L1026    	/* check loop termination, proceed if required */
			emms                      	/* exit MMX state */
			popa
	}
#else
	/* i386 and x86_64 */
	__m64 *mSrc1 = (__m64*)Src1;
	__m64 *mDest = (__m64*)Dest;
	__m64 mm0 = _m_from_int(0);			/* zero mm0 register */
	/* Duplicate C in 4 words of MM1 */
	int i;
	i = (C<<16)|C;
	__m64 mm1 = _m_from_int(i);
	__m64 mm2 = _m_from_int(i);
	mm1 = _m_punpckldq(mm1, mm2);			/* fill higher words of MM1 with C */
	for (i = 0; i < SrcLength/8; i++) {
		__m64 mm3, mm4, mm5, mm6;
		mm3 = _m_punpcklbw(*mSrc1, mm0);	/* unpack low  bytes of Src1 into words */
		mm4 = _m_punpckhbw(*mSrc1, mm0);	/* unpack high bytes of Src1 into words */
		mm3 = _m_psrlwi(mm3, N);		/* shift 4 WORDS of MM3 (N) bits to the right */
		mm4 = _m_psrlwi(mm4, N);		/* shift 4 WORDS of MM4 (N) bits to the right */
		mm3 = _m_pmullw(mm3, mm1);		/* mul low  bytes of Src1 and MM1 */
		mm4 = _m_pmullw(mm4, mm1);		/* mul high bytes of Src1 and MM1 */
		*mDest = _m_packuswb(mm3, mm4);		/* pack words back into bytes with saturation */
		mSrc1++;
		mDest++;
	}
	_m_empty();					/* clean MMX state */
#endif
	return (0);
#else
	return (-1);
#endif
}

/*!
\brief Filter using ShiftRightAndMultByByte: D = saturation255((S >> N) * C) 

\param Src1 Pointer to the start of the source byte array (S).
\param Dest Pointer to the start of the destination byte array (D).
\param length The number of bytes in the source array.
\param N Number of bit-positions to shift (N). Valid range is 0 to 8.
\param C Constant to multiply with (C).

\return Returns 0 for success or -1 for error.
*/
int SDL_imageFilterShiftRightAndMultByByte(unsigned char *Src1, unsigned char *Dest, unsigned int length, unsigned char N,
										   unsigned char C)
{
	unsigned int i, istart;
	int iC;
	unsigned char *cursrc1;
	unsigned char *curdest;
	int result;

	/* Validate input parameters */
	if ((Src1 == NULL) || (Dest == NULL))
		return(-1);
	if (length == 0)
		return(0);

	/* Check shift */
	if (N > 8) {
		return (-1);
	}

	/* Special case: N==0 && C==1 */
	if ((N == 0) && (C == 1)) {
		memcpy(Src1, Dest, length);
		return (0); 
	}

	if ((SDL_imageFilterMMXdetect()) && (length > 7)) {

		SDL_imageFilterShiftRightAndMultByByteMMX(Src1, Dest, length, N, C);

		/* Check for unaligned bytes */
		if ((length & 7) > 0) {
			/* Setup to process unaligned bytes */
			istart = length & 0xfffffff8;
			cursrc1 = &Src1[istart];
			curdest = &Dest[istart];
		} else {
			/* No unaligned bytes - we are done */
			return (0);
		}
	} else {
		/* Setup to process whole image */
		istart = 0;
		cursrc1 = Src1;
		curdest = Dest;
	}

	/* C routine to process image */
	iC = (int) C;
	for (i = istart; i < length; i++) {
		result = (int) (*cursrc1 >> N) * iC;
		if (result > 255)
			result = 255;
		*curdest = (unsigned char) result;
		/* Advance pointers */
		cursrc1++;
		curdest++;
	}

	return (0);
}

/*!
\brief Internal MMX Filter using ShiftLeftByte: D = (S << N)

\param Src1 Pointer to the start of the source byte array (S).
\param Dest Pointer to the start of the destination byte array (D).
\param SrcLength The number of bytes in the source arrays.
\param N Number of bit-positions to shift (N). Valid range is 0 to 8.
\param Mask Byte array containing 8 bytes of 0xFE value.

\return Returns 0 for success or -1 for error.
*/
static int SDL_imageFilterShiftLeftByteMMX(unsigned char *Src1, unsigned char *Dest, unsigned int SrcLength, unsigned char N,
									unsigned char *Mask)
{
#ifdef USE_MMX
#if !defined(GCC__)
	__asm
	{
		pusha
			mov edx, Mask   	/* load Mask address into edx */
			movq mm0, [edx]   	/* load Mask into mm0 */
		xor ecx, ecx   	/* zero ECX */
			mov cl, N   	/* load loop counter (N) into CL */
			movd mm3, ecx   	/* copy (N) into MM3  */
			pcmpeqb mm1, mm1   	/* generate all 1's in mm1 */
L10270:                  	/* ** Prepare proper bit-Mask in MM1 ** */
		psllw mm1, 1   	/* shift 4 WORDS of MM1 1 bit to the left */
			pand mm1, mm0        // apply Mask to 8 BYTES of MM1 */
			/*  byte     0x0f, 0xdb, 0xc8 */
			dec cl                  	/* decrease loop counter */
			jnz            L10270    	/* check loop termination, proceed if required */
			/* ** Shift all bytes of the image ** */
			mov eax, Src1   	/* load Src1 address into eax */
			mov edi, Dest   	/* load SrcDest address into edi */
			mov ecx, SrcLength   	/* load loop counter (SIZE) into ecx */
			shr ecx, 3   	/* counter/8 (MMX loads 8 bytes at a time) */
			align 16                 	/* 16 byte alignment of the loop entry */
L10271:
		movq mm0, [eax]   	/* load 8 bytes from Src1 into MM0 */
		psllw mm0, mm3   	/* shift 4 WORDS of MM0 (N) bits to the left */
			pand mm0, mm1    // apply proper bit-Mask to 8 BYTES of MM0 */
			/* byte     0x0f, 0xdb, 0xc1 */
			movq [edi], mm0   	/* store result in Dest */
			add eax, 8   	/* increase Src1 register pointer by 8 */
			add edi, 8   	/* increase Dest register pointer by 8 */
			dec              ecx    	/* decrease loop counter */
			jnz            L10271    	/* check loop termination, proceed if required */
			emms                      	/* exit MMX state */
			popa
	}
#else
	/* i386 and x86_64 */
	__m64 *mSrc1 = (__m64*)Src1;
	__m64 *mDest = (__m64*)Dest;
	__m64 *mMask = (__m64*)Mask;
        __m64 mm1;
	int i;
	mm1 = _m_pcmpeqb(mm1, mm1);			/* generate all 1's in mm1 */
	/* Prepare proper bit-Mask in MM1 */
	for (i = 0; i < N; i++) {
		mm1 = _m_psllwi(mm1, 1);		/* shift 4 WORDS of MM1 1 bit to the left */
		mm1 = _m_pand(mm1, *mMask);		/* apply Mask to 8 BYTES of MM1 */
	}
	/* ** Shift all bytes of the image ** */
	for (i = 0; i < SrcLength/8; i++) {
		__m64 mm0 = _m_psllwi(*mSrc1, N);	/* shift 4 WORDS of MM0 (N) bits to the left */
		*mDest = _m_pand(mm0, mm1);		/* apply proper bit-Mask to 8 BYTES of MM0 */
		mSrc1++;
		mDest++;
	}
	_m_empty();					/* clean MMX state */
#endif
	return (0);
#else
	return (-1);
#endif
}

/*!
\brief Filter using ShiftLeftByte: D = (S << N)

\param Src1 Pointer to the start of the source byte array (S).
\param Dest Pointer to the start of the destination byte array (D).
\param length The number of bytes in the source arrays.
\param N Number of bit-positions to shift (N). Valid range is 0 to 8.

\return Returns 0 for success or -1 for error.
*/
int SDL_imageFilterShiftLeftByte(unsigned char *Src1, unsigned char *Dest, unsigned int length, unsigned char N)
{
	static unsigned char Mask[8] = { 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE, 0xFE };
	unsigned int i, istart;
	unsigned char *cursrc1, *curdest;
	int result;

	/* Validate input parameters */
	if ((Src1 == NULL) || (Dest == NULL))
		return(-1);
	if (length == 0)
		return(0);

	if (N > 8) {
		return (-1);
	}

	/* Special case: N==0 */
	if (N == 0) {
		memcpy(Src1, Dest, length);
		return (0); 
	}

	if ((SDL_imageFilterMMXdetect()) && (length > 7)) {

		SDL_imageFilterShiftLeftByteMMX(Src1, Dest, length, N, Mask);

		/* Check for unaligned bytes */
		if ((length & 7) > 0) {
			/* Setup to process unaligned bytes */
			istart = length & 0xfffffff8;
			cursrc1 = &Src1[istart];
			curdest = &Dest[istart];
		} else {
			/* No unaligned bytes - we are done */
			return (0);
		}
	} else {
		/* Setup to process whole image */
		istart = 0;
		cursrc1 = Src1;
		curdest = Dest;
	}

	/* C routine to process image */
	for (i = istart; i < length; i++) {
		result = ((int) *cursrc1 << N) & 0xff;
		*curdest = (unsigned char) result;
		/* Advance pointers */
		cursrc1++;
		curdest++;
	}

	return (0);
}

/*!
\brief Internal MMX Filter using ShiftLeftUint: D = ((uint)S << N)

\param Src1 Pointer to the start of the source byte array (S).
\param Dest Pointer to the start of the destination byte array (D).
\param SrcLength The number of bytes in the source array.
\param N Number of bit-positions to shift (N). Valid range is 0 to 32.

\return Returns 0 for success or -1 for error.
*/
static int SDL_imageFilterShiftLeftUintMMX(unsigned char *Src1, unsigned char *Dest, unsigned int SrcLength, unsigned char N)
{
#ifdef USE_MMX
#if !defined(GCC__)
	__asm
	{
		pusha
			mov eax, Src1   	/* load Src1 address into eax */
			mov edi, Dest   	/* load Dest address into edi */
			mov ecx, SrcLength   	/* load loop counter (SIZE) into ecx */
			shr ecx, 3   	/* counter/8 (MMX loads 8 bytes at a time) */
			align 16                 	/* 16 byte alignment of the loop entry */
L12023:
		movq mm0, [eax]   	/* load 8 bytes from SrcDest into MM0 */
		pslld mm0, N   	/* MM0=SrcDest+C (add 8 bytes with saturation) */
			movq [edi], mm0   	/* store result in SrcDest */
			add eax, 8   	/* increase Src1 register pointer by 8 */
			add edi, 8   	/* increase Dest register pointer by 8 */
			dec              ecx    	/* decrease loop counter */
			jnz             L12023    	/* check loop termination, proceed if required */
			emms                      	/* exit MMX state */
			popa
	}
#else
	/* i386 and x86_64 */
	__m64 *mSrc1 = (__m64*)Src1;
	__m64 *mDest = (__m64*)Dest;
	int i;
	for (i = 0; i < SrcLength/8; i++) {
		*mDest = _m_pslldi(*mSrc1, N);	/* Src1+C (add 8 bytes with saturation) */
		mSrc1++;
		mDest++;
	}
	_m_empty();				/* clean MMX state */
#endif
	return (0);
#else
	return (-1);
#endif
}

/*!
\brief Filter using ShiftLeftUint: D = ((uint)S << N)

\param Src1 Pointer to the start of the source byte array (S).
\param Dest Pointer to the start of the destination byte array (D).
\param length The number of bytes in the source array.
\param N Number of bit-positions to shift (N). Valid range is 0 to 32.

\return Returns 0 for success or -1 for error.
*/
int SDL_imageFilterShiftLeftUint(unsigned char *Src1, unsigned char *Dest, unsigned int length, unsigned char N)
{
	unsigned int i, istart;
	unsigned char *cursrc1, *curdest;
	unsigned int *icursrc1, *icurdest;
	unsigned int result;

	/* Validate input parameters */
	if ((Src1 == NULL) || (Dest == NULL))
		return(-1);
	if (length == 0)
		return(0);

	if (N > 32) {
		return (-1);
	}

	/* Special case: N==0 */
	if (N == 0) {
		memcpy(Src1, Dest, length);
		return (0); 
	}

	if ((SDL_imageFilterMMXdetect()) && (length > 7)) {

		SDL_imageFilterShiftLeftUintMMX(Src1, Dest, length, N);

		/* Check for unaligned bytes */
		if ((length & 7) > 0) {
			/* Setup to process unaligned bytes */
			istart = length & 0xfffffff8;
			cursrc1 = &Src1[istart];
			curdest = &Dest[istart];
		} else {
			/* No unaligned bytes - we are done */
			return (0);
		}
	} else {
		/* Setup to process whole image */
		istart = 0;
		cursrc1 = Src1;
		curdest = Dest;
	}

	/* C routine to process image */
	icursrc1=(unsigned int *)cursrc1;
	icurdest=(unsigned int *)curdest;
	for (i = istart; i < length; i += 4) {
		if ((i+4)<length) {
			result = ((unsigned int)*icursrc1 << N);
			*icurdest = result;
		}
		/* Advance pointers */
		icursrc1++;
		icurdest++;
	}

	return (0);
}

/*!
\brief Internal MMX Filter ShiftLeft: D = saturation255(S << N)

\param Src1 Pointer to the start of the source byte array (S1).
\param Dest Pointer to the start of the destination byte array (D).
\param SrcLength The number of bytes in the source array.
\param N Number of bit-positions to shift (N). Valid range is 0 to 8.

\return Returns 0 for success or -1 for error.
*/
static int SDL_imageFilterShiftLeftMMX(unsigned char *Src1, unsigned char *Dest, unsigned int SrcLength, unsigned char N)
{
#ifdef USE_MMX
#if !defined(GCC__)
	__asm
	{
		pusha
			xor eax, eax   	/* zero EAX */
			mov al, N   	/* load N into AL */
			movd mm7, eax   	/* copy N into MM7 */
			pxor mm0, mm0   	/* zero MM0 register */
			mov eax, Src1   	/* load Src1 address into eax */
			mov edi, Dest   	/* load Dest address into edi */
			mov ecx, SrcLength   	/* load loop counter (SIZE) into ecx */
			shr ecx, 3   	/* counter/8 (MMX loads 8 bytes at a time) */
			cmp al, 7   	/* if (N <= 7) execute more efficient code */
			jg             L10281
			align 16                 	/* 16 byte alignment of the loop entry */
L10280:
		movq mm3, [eax]   	/* load 8 bytes from Src1 into MM3 */
		movq mm4, mm3   	/* copy MM3 into MM4  */
			punpcklbw mm3, mm0   	/* unpack low  bytes of SrcDest into words */
			punpckhbw mm4, mm0   	/* unpack high bytes of SrcDest into words */
			psllw mm3, mm7   	/* shift 4 WORDS of MM3 (N) bits to the left */
			psllw mm4, mm7   	/* shift 4 WORDS of MM4 (N) bits to the left */
			packuswb mm3, mm4   	/* pack words back into bytes with saturation */
			movq [edi], mm3   	/* store result in Dest */
			add eax, 8   	/* increase Src1 register pointer by 8 */
			add edi, 8   	/* increase Dest register pointer by 8 */
			dec              ecx    	/* decrease loop counter */
			jnz            L10280    	/* check loop termination, proceed if required */
			jmp            L10282
			align 16                 	/* 16 byte alignment of the loop entry */
L10281:
		movq mm3, [eax]   	/* load 8 bytes from Src1 into MM3 */
		movq mm4, mm3   	/* copy MM3 into MM4  */
			punpcklbw mm3, mm0   	/* unpack low  bytes of SrcDest into words */
			punpckhbw mm4, mm0   	/* unpack high bytes of SrcDest into words */
			psllw mm3, mm7   	/* shift 4 WORDS of MM3 (N) bits to the left */
			psllw mm4, mm7   	/* shift 4 WORDS of MM4 (N) bits to the left */
			/* ** Take abs value of the signed words ** */
			movq mm5, mm3   	/* copy mm3 into mm5 */
			movq mm6, mm4   	/* copy mm4 into mm6 */
			psraw mm5, 15   	/* fill mm5 words with word sign bit */
			psraw mm6, 15   	/* fill mm6 words with word sign bit */
			pxor mm3, mm5   	/* take 1's compliment of only neg words */
			pxor mm4, mm6   	/* take 1's compliment of only neg words */
			psubsw mm3, mm5   	/* add 1 to only neg words, W-(-1) or W-0 */
			psubsw mm4, mm6   	/* add 1 to only neg words, W-(-1) or W-0 */
			packuswb mm3, mm4   	/* pack words back into bytes with saturation */
			movq [edi], mm3   	/* store result in Dest */
			add eax, 8   	/* increase Src1 register pointer by 8 */
			add edi, 8   	/* increase Dest register pointer by 8 */
			dec              ecx    	/* decrease loop counter */
			jnz            L10281    	/* check loop termination, proceed if required */
L10282:
		emms                      	/* exit MMX state */
			popa
	}
#else
	/* i386 and x86_64 */
	__m64 *mSrc1 = (__m64*)Src1;
	__m64 *mDest = (__m64*)Dest;
	__m64 mm0 = _m_from_int(0);				/* zero mm0 register */
	int i;
	if (N <= 7) {						/* if (N <= 7) execute more efficient code */
		for (i = 0; i < SrcLength/8; i++) {
			__m64 mm3, mm4;
			mm3 = _m_punpcklbw(*mSrc1, mm0);	/* unpack low  bytes of Src1 into words */
			mm4 = _m_punpckhbw(*mSrc1, mm0);	/* unpack high bytes of Src1 into words */
			mm3 = _m_psllwi(mm3, N);		/* shift 4 WORDS of MM3 (N) bits to the left */
			mm4 = _m_psllwi(mm4, N);		/* shift 4 WORDS of MM4 (N) bits to the left */
			*mDest = _m_packuswb(mm3, mm4);		/* pack words back into bytes with saturation */
			mSrc1++;
			mDest++;
		}
	} else {
		for (i = 0; i < SrcLength/8; i++) {
			__m64 mm3, mm4, mm5, mm6;
			mm3 = _m_punpcklbw(*mSrc1, mm0);	/* unpack low  bytes of Src1 into words */
			mm4 = _m_punpckhbw(*mSrc1, mm0);	/* unpack high bytes of Src1 into words */
			mm3 = _m_psllwi(mm3, N);		/* shift 4 WORDS of MM3 (N) bits to the left */
			mm4 = _m_psllwi(mm4, N);		/* shift 4 WORDS of MM4 (N) bits to the left */
			/* Take abs value of the signed words */
			mm5 = _m_psrawi(mm3, 15);		/* fill mm5 words with word sign bit */
			mm6 = _m_psrawi(mm4, 15);		/* fill mm6 words with word sign bit */
			mm3 = _m_pxor(mm3, mm5);		/* take 1's compliment of only neg. words */
			mm4 = _m_pxor(mm4, mm6);		/* take 1's compliment of only neg. words */
			mm3 = _m_psubsw(mm3, mm5);		/* add 1 to only neg. words, W-(-1) or W-0 */
			mm4 = _m_psubsw(mm4, mm6);		/* add 1 to only neg. words, W-(-1) or W-0 */
			*mDest = _m_packuswb(mm3, mm4);		/* pack words back into bytes with saturation */
			mSrc1++;
			mDest++;
		}
	}
	_m_empty();						/* clean MMX state */
#endif
	return (0);
#else
	return (-1);
#endif
}

/*!
\brief Filter ShiftLeft: D = saturation255(S << N)

\param Src1 Pointer to the start of the source byte array (S1).
\param Dest Pointer to the start of the destination byte array (D).
\param length The number of bytes in the source array.
\param N Number of bit-positions to shift (N). Valid range is 0 to 8.

\return Returns 0 for success or -1 for error.
*/
int SDL_imageFilterShiftLeft(unsigned char *Src1, unsigned char *Dest, unsigned int length, unsigned char N)
{
	unsigned int i, istart;
	unsigned char *cursrc1, *curdest;
	int result;

	/* Validate input parameters */
	if ((Src1 == NULL) || (Dest == NULL))
		return(-1);
	if (length == 0)
		return(0);

	if (N > 8) {
		return (-1);
	}

	/* Special case: N==0 */
	if (N == 0) {
		memcpy(Src1, Dest, length);
		return (0); 
	}

	if ((SDL_imageFilterMMXdetect()) && (length > 7)) {

		SDL_imageFilterShiftLeftMMX(Src1, Dest, length, N);

		/* Check for unaligned bytes */
		if ((length & 7) > 0) {
			/* Setup to process unaligned bytes */
			istart = length & 0xfffffff8;
			cursrc1 = &Src1[istart];
			curdest = &Dest[istart];
		} else {
			/* No unaligned bytes - we are done */
			return (0);
		}
	} else {
		/* Setup to process whole image */
		istart = 0;
		cursrc1 = Src1;
		curdest = Dest;
	}

	/* C routine to process image */
	for (i = istart; i < length; i++) {
		result = (int) *cursrc1 << N;
		if (result > 255)
			result = 255;
		*curdest = (unsigned char) result;
		/* Advance pointers */
		cursrc1++;
		curdest++;
	}

	return (0);
}

/*!
\brief MMX BinarizeUsingThreshold: D = (S >= T) ? 255:0

\param Src1 Pointer to the start of the source byte array (S).
\param Dest Pointer to the start of the destination byte array (D).
\param SrcLength The number of bytes in the source array.
\param T The threshold boundary (inclusive).

\return Returns 0 for success or -1 for error.
*/
static int SDL_imageFilterBinarizeUsingThresholdMMX(unsigned char *Src1, unsigned char *Dest, unsigned int SrcLength, unsigned char T)
{
#ifdef USE_MMX
#if !defined(GCC__)
	__asm
	{
		pusha
			/* ** Duplicate T in 8 bytes of MM3 ** */
			pcmpeqb mm1, mm1   	/* generate all 1's in mm1 */
			pcmpeqb mm2, mm2   	/* generate all 1's in mm2 */
			mov al, T   	/* load T into AL */
			mov ah, al   	/* copy AL into AH */
			mov bx, ax   	/* copy AX into BX */
			shl eax, 16   	/* shift 2 bytes of EAX left */
			mov ax, bx   	/* copy BX into AX */
			movd mm3, eax   	/* copy EAX into MM3 */
			movd mm4, eax   	/* copy EAX into MM4 */
			punpckldq mm3, mm4   	/* fill higher bytes of MM3 with T */
			psubusb mm2, mm3   	/* store 0xFF - T in MM2 */
			mov eax, Src1   	/* load Src1 address into eax */
			mov edi, Dest   	/* load Dest address into edi */
			mov ecx, SrcLength   	/* load loop counter (SIZE) into ecx */
			shr ecx, 3   	/* counter/8 (MMX loads 8 bytes at a time) */
			align 16                 	/* 16 byte alignment of the loop entry */
L1029:
		movq mm0, [eax]   	/* load 8 bytes from SrcDest into MM0 */
		paddusb mm0, mm2   	/* MM0=SrcDest+(0xFF-T) (add 8 bytes with saturation) */
			pcmpeqb mm0, mm1   	/* binarize 255:0, comparing to 255 */
			movq [edi], mm0   	/* store result in SrcDest */
			add eax, 8   	/* increase Src1 register pointer by 8 */
			add edi, 8   	/* increase Dest register pointer by 8 */
			dec              ecx    	/* decrease loop counter */
			jnz             L1029    	/* check loop termination, proceed if required */
			emms                      	/* exit MMX state */
			popa
	}
#else
	/* i386 and x86_64 */
	__m64 *mSrc1 = (__m64*)Src1;
	__m64 *mDest = (__m64*)Dest;
	/* Duplicate T in 8 bytes of MM3 */
	__m64 mm1 = _m_pcmpeqb(mm1, mm1);			/* generate all 1's in mm1 */
	__m64 mm2 = _m_pcmpeqb(mm2, mm2);			/* generate all 1's in mm1 */
	int i;
	memset(&i, T, 4);
	__m64 mm3 = _m_from_int(i);
	__m64 mm4 = _m_from_int(i);
	mm3 = _m_punpckldq(mm3, mm4);			/* fill higher bytes of MM3 with T */
	mm2 = _m_psubusb(mm2, mm3);			/* store 0xFF - T in MM2 */
        //__m64 mm3 = _m_from_int64(lli); // x86_64 only
	for (i = 0; i < SrcLength/8; i++) {
		__m64 mm0 = _m_paddusb(*mSrc1, mm2);	/* Src1+(0xFF-T) (add 8 bytes with saturation) */
		*mDest = _m_pcmpeqb(mm0, mm1);		/* binarize 255:0, comparing to 255 */
		mSrc1++;
		mDest++;
	}
	_m_empty();					/* clean MMX state */
#endif
	return (0);
#else
	return (-1);
#endif
}

/*!
\brief Filter using BinarizeUsingThreshold: D = (S >= T) ? 255:0

\param Src1 Pointer to the start of the source byte array (S).
\param Dest Pointer to the start of the destination byte array (D).
\param length The number of bytes in the source array.
\param T The threshold boundary (inclusive).

\return Returns 0 for success or -1 for error.
*/
int SDL_imageFilterBinarizeUsingThreshold(unsigned char *Src1, unsigned char *Dest, unsigned int length, unsigned char T)
{
	unsigned int i, istart;
	unsigned char *cursrc1;
	unsigned char *curdest;

	/* Validate input parameters */
	if ((Src1 == NULL) || (Dest == NULL))
		return(-1);
	if (length == 0)
		return(0);

	/* Special case: T==0 */
	if (T == 0) {
		memset(Dest, 255, length);
		return (0); 
	}

	if ((SDL_imageFilterMMXdetect()) && (length > 7)) {

		SDL_imageFilterBinarizeUsingThresholdMMX(Src1, Dest, length, T);

		/* Check for unaligned bytes */
		if ((length & 7) > 0) {
			/* Setup to process unaligned bytes */
			istart = length & 0xfffffff8;
			cursrc1 = &Src1[istart];
			curdest = &Dest[istart];
		} else {
			/* No unaligned bytes - we are done */
			return (0);
		}
	} else {
		/* Setup to process whole image */
		istart = 0;
		cursrc1 = Src1;
		curdest = Dest;
	}

	/* C routine to process image */
	for (i = istart; i < length; i++) {
		*curdest = (unsigned char)(((unsigned char)*cursrc1 >= T) ? 255 : 0);
		/* Advance pointers */
		cursrc1++;
		curdest++;
	}

	return (0);
}

/*!
\brief Internal MMX Filter using ClipToRange: D = (S >= Tmin) & (S <= Tmax) S:Tmin | Tmax

\param Src1 Pointer to the start of the source byte array (S).
\param Dest Pointer to the start of the destination byte array (D).
\param SrcLength The number of bytes in the source array.
\param Tmin Lower (inclusive) boundary of the clipping range.
\param Tmax Upper (inclusive) boundary of the clipping range.

\return Returns 0 for success or -1 for error.
*/
static int SDL_imageFilterClipToRangeMMX(unsigned char *Src1, unsigned char *Dest, unsigned int SrcLength, unsigned char Tmin,
								  unsigned char Tmax)
{
#ifdef USE_MMX
#if !defined(GCC__)
	__asm
	{
		pusha
			pcmpeqb mm1, mm1   	/* generate all 1's in mm1 */
			/* ** Duplicate Tmax in 8 bytes of MM3 ** */
			mov al, Tmax   	/* load Tmax into AL */
			mov ah, al   	/* copy AL into AH */
			mov bx, ax   	/* copy AX into BX */
			shl eax, 16   	/* shift 2 bytes of EAX left */
			mov ax, bx   	/* copy BX into AX */
			movd mm3, eax   	/* copy EAX into MM3 */
			movd mm4, eax   	/* copy EAX into MM4 */
			punpckldq mm3, mm4   	/* fill higher bytes of MM3 with Tmax */
			psubusb mm1, mm3   	/* store 0xFF - Tmax in MM1 */
			/* ** Duplicate Tmin in 8 bytes of MM5 ** */
			mov al, Tmin   	/* load Tmin into AL */
			mov ah, al   	/* copy AL into AH */
			mov bx, ax   	/* copy AX into BX */
			shl eax, 16   	/* shift 2 bytes of EAX left */
			mov ax, bx   	/* copy BX into AX */
			movd mm5, eax   	/* copy EAX into MM5 */
			movd mm4, eax   	/* copy EAX into MM4 */
			punpckldq mm5, mm4   	/* fill higher bytes of MM5 with Tmin */
			movq mm7, mm5   	/* copy MM5 into MM7 */
			paddusb mm7, mm1   	/* store 0xFF - Tmax + Tmin in MM7 */
			mov eax, Src1   	/* load Src1 address into eax */
			mov edi, Dest   	/* load Dest address into edi */
			mov ecx, SrcLength   	/* load loop counter (SIZE) into ecx */
			shr ecx, 3   	/* counter/8 (MMX loads 8 bytes at a time) */
			align 16                 	/* 16 byte alignment of the loop entry */
L1030:
		movq mm0, [eax]   	/* load 8 bytes from Src1 into MM0 */
		paddusb mm0, mm1   	/* MM0=SrcDest+(0xFF-Tmax) */
			psubusb mm0, mm7   	/* MM0=MM0-(0xFF-Tmax+Tmin) */
			paddusb mm0, mm5   	/* MM0=MM0+Tmin */
			movq [edi], mm0   	/* store result in Dest */
			add eax, 8   	/* increase Src1 register pointer by 8 */
			add edi, 8   	/* increase Dest register pointer by 8 */
			dec              ecx    	/* decrease loop counter */
			jnz             L1030    	/* check loop termination, proceed if required */
			emms                      	/* exit MMX state */
			popa
	}
#else
	/* i386 and x86_64 */
	__m64 *mSrc1 = (__m64*)Src1;
	__m64 *mDest = (__m64*)Dest;
	__m64 mm1 = _m_pcmpeqb(mm1, mm1);	/* generate all 1's in mm1 */
	int i;
	/* Duplicate Tmax in 8 bytes of MM3 */
	__m64 mm3, mm4;
	memset(&i, Tmax, 4);
	mm3 = _m_from_int(i);
	mm4 = _m_from_int(i);
	mm3 = _m_punpckldq(mm3, mm4);		/* fill higher bytes of MM3 with Tmax */
	mm1 = _m_psubusb(mm1, mm3);		/* store 0xFF - Tmax in MM1 */
        //__m64 mm3 = _m_from_int64(lli); // x86_64 only
	/* Duplicate Tmax in 8 bytes of MM3 */
	__m64 mm5, mm7;
	memset(&i, Tmin, 4);
	mm5 = _m_from_int(i);
	mm4 = _m_from_int(i);
	mm5 = _m_punpckldq(mm5, mm4);		/* fill higher bytes of MM5 with Tmin */
	mm7 = _m_paddusb(mm5, mm1);	/* store 0xFF - Tmax + Tmin in MM7 */
	for (i = 0; i < SrcLength/8; i++) {
		__m64 mm0;
		mm0 = _m_paddusb(*mSrc1, mm1);	/* MM0=Src1+(0xFF-Tmax) */
		mm0 = _m_psubusb(mm0, mm7);	/* MM0=MM0-(0xFF-Tmax+Tmin) */
		*mDest = _m_paddusb(mm0, mm5);	/* MM0+Tmin */
		mSrc1++;
		mDest++;
	}
	_m_empty();				/* clean MMX state */
#endif
	return (0);
#else
	return (-1);
#endif
}

/*!
\brief Filter using ClipToRange: D = (S >= Tmin) & (S <= Tmax) S:Tmin | Tmax

\param Src1 Pointer to the start of the source byte array (S).
\param Dest Pointer to the start of the destination byte array (D).
\param length The number of bytes in the source array.
\param Tmin Lower (inclusive) boundary of the clipping range.
\param Tmax Upper (inclusive) boundary of the clipping range.

\return Returns 0 for success or -1 for error.
*/
int SDL_imageFilterClipToRange(unsigned char *Src1, unsigned char *Dest, unsigned int length, unsigned char Tmin,
							   unsigned char Tmax)
{
	unsigned int i, istart;
	unsigned char *cursrc1;
	unsigned char *curdest;

	/* Validate input parameters */
	if ((Src1 == NULL) || (Dest == NULL))
		return(-1);
	if (length == 0)
		return(0);

	/* Special case: Tmin==0 && Tmax = 255 */
	if ((Tmin == 0) && (Tmax == 25)) {
		memcpy(Src1, Dest, length);
		return (0); 
	}

	if ((SDL_imageFilterMMXdetect()) && (length > 7)) {

		SDL_imageFilterClipToRangeMMX(Src1, Dest, length, Tmin, Tmax);

		/* Check for unaligned bytes */
		if ((length & 7) > 0) {
			/* Setup to process unaligned bytes */
			istart = length & 0xfffffff8;
			cursrc1 = &Src1[istart];
			curdest = &Dest[istart];
		} else {
			/* No unaligned bytes - we are done */
			return (0);
		}
	} else {
		/* Setup to process whole image */
		istart = 0;
		cursrc1 = Src1;
		curdest = Dest;
	}

	/* C routine to process image */
	for (i = istart; i < length; i++) {
		if (*cursrc1 < Tmin) {
			*curdest = Tmin;
		} else if (*cursrc1 > Tmax) {
			*curdest = Tmax;
		} else {
			*curdest = *cursrc1;
		}
		/* Advance pointers */
		cursrc1++;
		curdest++;
	}

	return (0);
}

/*!
\brief Internal MMX Filter using NormalizeLinear: D = saturation255((Nmax - Nmin)/(Cmax - Cmin)*(S - Cmin) + Nmin)

\param Src1 Pointer to the start of the source byte array (S).
\param Dest Pointer to the start of the destination byte array (D).
\param SrcLength The number of bytes in the source array.
\param Cmin Normalization constant (Cmin).
\param Cmax Normalization constant (Cmax).
\param Nmin Normalization constant (Nmin).
\param Nmax Normalization constant (Nmax).

\return Returns 0 for success or -1 for error.
*/
static int SDL_imageFilterNormalizeLinearMMX(unsigned char *Src1, unsigned char *Dest, unsigned int SrcLength, int Cmin, int Cmax,
									  int Nmin, int Nmax)
{
#ifdef USE_MMX
#if !defined(GCC__)
	__asm
	{
		pusha
			mov ax, WORD PTR Nmax   	/* load Nmax in AX */
			mov bx, WORD PTR Cmax   	/* load Cmax in BX */
			sub ax, WORD PTR Nmin   	/* AX = Nmax - Nmin */
			sub bx, WORD PTR Cmin   	/* BX = Cmax - Cmin */
			jz             L10311    	/* check division by zero */
			xor dx, dx   	/* prepare for division, zero DX */
			div               bx    	/* AX = AX/BX */
			jmp            L10312
L10311:
		mov ax, 255   	/* if div by zero, assume result max byte value */
L10312:                  	/* ** Duplicate AX in 4 words of MM0 ** */
		mov bx, ax   	/* copy AX into BX */
			shl eax, 16   	/* shift 2 bytes of EAX left */
			mov ax, bx   	/* copy BX into AX */
			movd mm0, eax   	/* copy EAX into MM0 */
			movd mm1, eax   	/* copy EAX into MM1 */
			punpckldq mm0, mm1   	/* fill higher words of MM0 with AX */
			/* ** Duplicate Cmin in 4 words of MM1 ** */
			mov ax, WORD PTR Cmin   	/* load Cmin into AX */
			mov bx, ax   	/* copy AX into BX */
			shl eax, 16   	/* shift 2 bytes of EAX left */
			mov ax, bx   	/* copy BX into AX */
			movd mm1, eax   	/* copy EAX into MM1 */
			movd mm2, eax   	/* copy EAX into MM2 */
			punpckldq mm1, mm2   	/* fill higher words of MM1 with Cmin */
			/* ** Duplicate Nmin in 4 words of MM2 ** */
			mov ax, WORD PTR Nmin   	/* load Nmin into AX */
			mov bx, ax   	/* copy AX into BX */
			shl eax, 16   	/* shift 2 bytes of EAX left */
			mov ax, bx   	/* copy BX into AX */
			movd mm2, eax   	/* copy EAX into MM2 */
			movd mm3, eax   	/* copy EAX into MM3 */
			punpckldq mm2, mm3   	/* fill higher words of MM2 with Nmin */
			pxor mm7, mm7   	/* zero MM7 register */
			mov eax, Src1   	/* load Src1 address into eax */
			mov edi, Dest   	/* load Dest address into edi */
			mov ecx, SrcLength   	/* load loop counter (SIZE) into ecx */
			shr ecx, 3   	/* counter/8 (MMX loads 8 bytes at a time) */
			align 16                 	/* 16 byte alignment of the loop entry */
L1031:
		movq mm3, [eax]   	/* load 8 bytes from Src1 into MM3 */
		movq mm4, mm3   	/* copy MM3 into MM4  */
			punpcklbw mm3, mm7   	/* unpack low  bytes of SrcDest into words */
			punpckhbw mm4, mm7   	/* unpack high bytes of SrcDest into words */
			psubusb mm3, mm1   	/* S-Cmin, low  bytes */
			psubusb mm4, mm1   	/* S-Cmin, high bytes */
			pmullw mm3, mm0   	/* MM0*(S-Cmin), low  bytes */
			pmullw mm4, mm0   	/* MM0*(S-Cmin), high bytes */
			paddusb mm3, mm2   	/* MM0*(S-Cmin)+Nmin, low  bytes */
			paddusb mm4, mm2   	/* MM0*(S-Cmin)+Nmin, high bytes */
			/* ** Take abs value of the signed words ** */
			movq mm5, mm3   	/* copy mm3 into mm5 */
			movq mm6, mm4   	/* copy mm4 into mm6 */
			psraw mm5, 15   	/* fill mm5 words with word sign bit */
			psraw mm6, 15   	/* fill mm6 words with word sign bit */
			pxor mm3, mm5   	/* take 1's compliment of only neg words */
			pxor mm4, mm6   	/* take 1's compliment of only neg words */
			psubsw mm3, mm5   	/* add 1 to only neg words, W-(-1) or W-0 */
			psubsw mm4, mm6   	/* add 1 to only neg words, W-(-1) or W-0 */
			packuswb mm3, mm4   	/* pack words back into bytes with saturation */
			movq [edi], mm3   	/* store result in Dest */
			add eax, 8   	/* increase Src1 register pointer by 8 */
			add edi, 8   	/* increase Dest register pointer by 8 */
			dec              ecx    	/* decrease loop counter */
			jnz             L1031    	/* check loop termination, proceed if required */
			emms                      	/* exit MMX state */
			popa
	}
#else
	/* i386 and x86_64 */
	__m64 *mSrc1 = (__m64*)Src1;
	__m64 *mDest = (__m64*)Dest;
	__m64 mm0, mm1, mm2, mm3;

	int i;
	/* Duplicate (Nmax-Nmin)/(Cmax-Cmin) in 4 words of MM0 */
	unsigned short a = Nmax - Nmin;
	unsigned short b = Cmax - Cmin;
	if (b == 0) {
	    a = 255;
	} else {
	    a /= b;
	}
	i = (a<<16)|a;
	mm0 = _m_from_int(i);
	mm1 = _m_from_int(i);
	mm0 = _m_punpckldq(mm0, mm1);			/* fill higher words of MM0 with AX */
	/* Duplicate Cmin in 4 words of MM1 */
	i = (Cmin<<16)|(short)Cmin;
	mm1 = _m_from_int(i);
	mm2 = _m_from_int(i);
	mm1 = _m_punpckldq(mm1, mm2);			/* fill higher words of MM1 with Cmin */
	/* Duplicate Nmin in 4 words of MM2 */
	i = (Nmin<<16)|(short)Nmin;
	mm2 = _m_from_int(i);
	mm3 = _m_from_int(i);
	mm2 = _m_punpckldq(mm2, mm3);			/* fill higher words of MM2 with Nmin */
	__m64 mm7 = _m_from_int(0);			/* zero mm0 register */
	for (i = 0; i < SrcLength/8; i++) {
		__m64 mm3, mm4, mm5, mm6;
		mm3 = _m_punpcklbw(*mSrc1, mm7);	/* unpack low  bytes of Src1 into words */
		mm4 = _m_punpckhbw(*mSrc1, mm7);	/* unpack high bytes of Src1 into words */
		mm3 = _m_psubusb(mm3, mm1);		/* S-Cmin, low	bytes */
		mm4 = _m_psubusb(mm4, mm1);		/* S-Cmin, high bytes */
		mm3 = _m_pmullw(mm3, mm0);		/* MM0*(S-Cmin), low  bytes */
		mm4 = _m_pmullw(mm4, mm0);		/* MM0*(S-Cmin), high bytes */
		mm3 = _m_paddusb(mm3, mm2);		/* MM0*(S-Cmin)+Nmin, low  bytes */
		mm4 = _m_paddusb(mm4, mm2);		/* MM0*(S-Cmin)+Nmin, high bytes */
		/* Take abs value of the signed words */
		mm5 = _m_psrawi(mm3, 15);		/* fill mm5 words with word sign bit */
		mm6 = _m_psrawi(mm4, 15);		/* fill mm6 words with word sign bit */
		mm3 = _m_pxor(mm3, mm5);		/* take 1's compliment of only neg. words */
		mm4 = _m_pxor(mm4, mm6);		/* take 1's compliment of only neg. words */
		mm3 = _m_psubsw(mm3, mm5);		/* add 1 to only neg. words, W-(-1) or W-0 */
		mm4 = _m_psubsw(mm4, mm6);		/* add 1 to only neg. words, W-(-1) or W-0 */
		*mDest = _m_packuswb(mm3, mm4);		/* pack words back into bytes with saturation */
		mSrc1++;
		mDest++;
	}
	_m_empty();					/* clean MMX state */
#endif
	return (0);
#else
	return (-1);
#endif
}

/*!
\brief Filter using NormalizeLinear: D = saturation255((Nmax - Nmin)/(Cmax - Cmin)*(S - Cmin) + Nmin)

\param Src Pointer to the start of the source byte array (S).
\param Dest Pointer to the start of the destination byte array (D).
\param length The number of bytes in the source array.
\param Cmin Normalization constant.
\param Cmax Normalization constant.
\param Nmin Normalization constant.
\param Nmax Normalization constant.

\return Returns 0 for success or -1 for error.
*/
int SDL_imageFilterNormalizeLinear(unsigned char *Src, unsigned char *Dest, unsigned int length, int Cmin, int Cmax, int Nmin,
								   int Nmax)
{
	unsigned int i, istart;
	unsigned char *cursrc;
	unsigned char *curdest;
	int dN, dC, factor;
	int result;

	/* Validate input parameters */
	if ((Src == NULL) || (Dest == NULL))
		return(-1);
	if (length == 0)
		return(0);

	if ((SDL_imageFilterMMXdetect()) && (length > 7)) {

		SDL_imageFilterNormalizeLinearMMX(Src, Dest, length, Cmin, Cmax, Nmin, Nmax);

		/* Check for unaligned bytes */
		if ((length & 7) > 0) {
			/* Setup to process unaligned bytes */
			istart = length & 0xfffffff8;
			cursrc = &Src[istart];
			curdest = &Dest[istart];
		} else {
			/* No unaligned bytes - we are done */
			return (0);
		}
	} else {
		/* Setup to process whole image */
		istart = 0;
		cursrc = Src;
		curdest = Dest;
	}

	/* C routine to process image */
	dC = Cmax - Cmin;
	if (dC == 0)
		return (0);
	dN = Nmax - Nmin;
	factor = dN / dC;
	for (i = istart; i < length; i++) {
		result = factor * ((int) (*cursrc) - Cmin) + Nmin;
		if (result > 255)
			result = 255;
		*curdest = (unsigned char) result;
		/* Advance pointers */
		cursrc++;
		curdest++;
	}

	return (0);
}

/* ------------------------------------------------------------------------------------ */

/*!
\brief Filter using ConvolveKernel3x3Divide: Dij = saturation0and255( ... ) 

\param Src The source 2D byte array to convolve. Should be different from destination.
\param Dest The destination 2D byte array to store the result in. Should be different from source.
\param rows Number of rows in source/destination array. Must be >2.
\param columns Number of columns in source/destination array. Must be >2.
\param Kernel The 2D convolution kernel of size 3x3.
\param Divisor The divisor of the convolution sum. Must be >0.

Note: Non-MMX implementation not available for this function.

\return Returns 1 if filter was applied, 0 otherwise.
*/
int SDL_imageFilterConvolveKernel3x3Divide(unsigned char *Src, unsigned char *Dest, int rows, int columns,
										   signed short *Kernel, unsigned char Divisor)
{
	/* Validate input parameters */
	if ((Src == NULL) || (Dest == NULL) || (Kernel == NULL))
		return(-1);

	if ((columns < 3) || (rows < 3) || (Divisor == 0))
		return (-1);

	if ((SDL_imageFilterMMXdetect())) {
//#ifdef USE_MMX
#if defined(USE_MMX) && defined(i386)
#if !defined(GCC__)
		__asm
		{
			pusha
				pxor mm0, mm0   	/* zero MM0 */
				xor ebx, ebx   	/* zero EBX */
				mov bl, Divisor   	/* load Divisor into BL */
				mov edx, Kernel   	/* load Kernel address into EDX */
				movq mm5, [edx]   	/* MM5 = {0,K2,K1,K0} */
			add edx, 8   	/* second row              |K0 K1 K2 0| */
				movq mm6, [edx]   	/* MM6 = {0,K5,K4,K3}  K = |K3 K4 K5 0| */
			add edx, 8   	/* third row               |K6 K7 K8 0| */
				movq mm7, [edx]   	/* MM7 = {0,K8,K7,K6} */
			/* ---, */
			mov eax, columns   	/* load columns into EAX */
				mov esi, Src   	/* ESI = Src row 0 address */
				mov edi, Dest   	/* load Dest address to EDI */
				add edi, eax   	/* EDI = EDI + columns */
				inc              edi    	/* 1 byte offset from the left edge */
				mov edx, rows   	/* initialize ROWS counter */
				sub edx, 2   	/* do not use first and last row */
				/* ---, */
L10320:
			mov ecx, eax   	/* initialize COLUMS counter */
				sub ecx, 2   	/* do not use first and last column */
				align 16                 	/* 16 byte alignment of the loop entry */
L10322:
			/* ---, */
			movq mm1, [esi]   	/* load 8 bytes of the image first row */
			add esi, eax   	/* move one row below */
				movq mm2, [esi]   	/* load 8 bytes of the image second row */
			add esi, eax   	/* move one row below */
				movq mm3, [esi]   	/* load 8 bytes of the image third row */
			punpcklbw mm1, mm0   	/* unpack first 4 bytes into words */
				punpcklbw mm2, mm0   	/* unpack first 4 bytes into words */
				punpcklbw mm3, mm0   	/* unpack first 4 bytes into words */
				pmullw mm1, mm5   	/* multiply words first row  image*Kernel */
				pmullw mm2, mm6   	/* multiply words second row image*Kernel */
				pmullw mm3, mm7   	/* multiply words third row  image*Kernel */
				paddsw mm1, mm2   	/* add 4 words of the first and second rows */
				paddsw mm1, mm3   	/* add 4 words of the third row and result */
				movq mm2, mm1   	/* copy MM1 into MM2 */
				psrlq mm1, 32   	/* shift 2 left words to the right */
				paddsw mm1, mm2   	/* add 2 left and 2 right result words */
				movq mm3, mm1   	/* copy MM1 into MM3 */
				psrlq mm1, 16   	/* shift 1 left word to the right */
				paddsw mm1, mm3   	/* add 1 left and 1 right result words */
				/* --, */
				movd mm2, eax   	/* save EAX in MM2 */
				movd mm3, edx   	/* save EDX in MM3 */
				movd eax, mm1   	/* copy MM1 into EAX */
				psraw mm1, 15   	/* spread sign bit of the result */
				movd edx, mm1   	/* fill EDX with a sign bit */
				idiv bx    	/* IDIV - VERY EXPENSIVE */
				movd mm1, eax   	/* move result of division into MM1 */
				packuswb mm1, mm0   	/* pack division result with saturation */
				movd eax, mm1   	/* copy saturated result into EAX */
				mov [edi], al   	/* copy a byte result into Dest */
				movd edx, mm3   	/* restore saved EDX */
				movd eax, mm2   	/* restore saved EAX */
				/* --, */
				sub esi, eax   	/* move two rows up */
				sub esi, eax   	/* */
				inc              esi    	/* move Src  pointer to the next pixel */
				inc              edi    	/* move Dest pointer to the next pixel */
				/* ---, */
				dec              ecx    	/* decrease loop counter COLUMNS */
				jnz            L10322    	/* check loop termination, proceed if required */
				add esi, 2   	/* move to the next row in Src */
				add edi, 2   	/* move to the next row in Dest */
				dec              edx    	/* decrease loop counter ROWS */
				jnz            L10320    	/* check loop termination, proceed if required */
				/* ---, */
				emms                      	/* exit MMX state */
				popa
		}
#else
		asm volatile
			("pusha		     \n\t" "pxor      %%mm0, %%mm0 \n\t"	/* zero MM0 */
			"xor       %%ebx, %%ebx \n\t"	/* zero EBX */
			"mov           %5, %%bl \n\t"	/* load Divisor into BL */
			"mov          %4, %%edx \n\t"	/* load Kernel address into EDX */
			"movq    (%%edx), %%mm5 \n\t"	/* MM5 = {0,K2,K1,K0} */
			"add          $8, %%edx \n\t"	/* second row              |K0 K1 K2 0| */
			"movq    (%%edx), %%mm6 \n\t"	/* MM6 = {0,K5,K4,K3}  K = |K3 K4 K5 0| */
			"add          $8, %%edx \n\t"	/* third row               |K6 K7 K8 0| */
			"movq    (%%edx), %%mm7 \n\t"	/* MM7 = {0,K8,K7,K6} */
			/* --- */
			"mov          %3, %%eax \n\t"	/* load columns into EAX */
			"mov          %1, %%esi \n\t"	/* ESI = Src row 0 address */
			"mov          %0, %%edi \n\t"	/* load Dest address to EDI */
			"add       %%eax, %%edi \n\t"	/* EDI = EDI + columns */
			"inc              %%edi \n\t"	/* 1 byte offset from the left edge */
			"mov          %2, %%edx \n\t"	/* initialize ROWS counter */
			"sub          $2, %%edx \n\t"	/* do not use first and last row */
			/* --- */
			".L10320:               \n\t" "mov       %%eax, %%ecx \n\t"	/* initialize COLUMS counter */
			"sub          $2, %%ecx \n\t"	/* do not use first and last column */
			".align 16              \n\t"	/* 16 byte alignment of the loop entry */
			".L10322:               \n\t"
			/* --- */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the image first row */
			"add       %%eax, %%esi \n\t"	/* move one row below */
			"movq    (%%esi), %%mm2 \n\t"	/* load 8 bytes of the image second row */
			"add       %%eax, %%esi \n\t"	/* move one row below */
			"movq    (%%esi), %%mm3 \n\t"	/* load 8 bytes of the image third row */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first 4 bytes into words */
			"punpcklbw %%mm0, %%mm2 \n\t"	/* unpack first 4 bytes into words */
			"punpcklbw %%mm0, %%mm3 \n\t"	/* unpack first 4 bytes into words */
			"pmullw    %%mm5, %%mm1 \n\t"	/* multiply words first row  image*Kernel */
			"pmullw    %%mm6, %%mm2 \n\t"	/* multiply words second row image*Kernel */
			"pmullw    %%mm7, %%mm3 \n\t"	/* multiply words third row  image*Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the first and second rows */
			"paddsw    %%mm3, %%mm1 \n\t"	/* add 4 words of the third row and result */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"psrlq       $32, %%mm1 \n\t"	/* shift 2 left words to the right */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 2 left and 2 right result words */
			"movq      %%mm1, %%mm3 \n\t"	/* copy MM1 into MM3 */
			"psrlq       $16, %%mm1 \n\t"	/* shift 1 left word to the right */
			"paddsw    %%mm3, %%mm1 \n\t"	/* add 1 left and 1 right result words */
			/* -- */
			"movd      %%eax, %%mm2 \n\t"	/* save EAX in MM2 */
			"movd      %%edx, %%mm3 \n\t"	/* save EDX in MM3 */
			"movd      %%mm1, %%eax \n\t"	/* copy MM1 into EAX */
			"psraw       $15, %%mm1 \n\t"	/* spread sign bit of the result */
			"movd      %%mm1, %%edx \n\t"	/* fill EDX with a sign bit */
			"idivw             %%bx \n\t"	/* IDIV - VERY EXPENSIVE */
			"movd      %%eax, %%mm1 \n\t"	/* move result of division into MM1 */
			"packuswb  %%mm0, %%mm1 \n\t"	/* pack division result with saturation */
			"movd      %%mm1, %%eax \n\t"	/* copy saturated result into EAX */
			"mov      %%al, (%%edi) \n\t"	/* copy a byte result into Dest */
			"movd      %%mm3, %%edx \n\t"	/* restore saved EDX */
			"movd      %%mm2, %%eax \n\t"	/* restore saved EAX */
			/* -- */
			"sub       %%eax, %%esi \n\t"	/* move two rows up */
			"sub       %%eax, %%esi \n\t"	/* */
			"inc              %%esi \n\t"	/* move Src  pointer to the next pixel */
			"inc              %%edi \n\t"	/* move Dest pointer to the next pixel */
			/* --- */
			"dec              %%ecx \n\t"	/* decrease loop counter COLUMNS */
			"jnz            .L10322 \n\t"	/* check loop termination, proceed if required */
			"add          $2, %%esi \n\t"	/* move to the next row in Src */
			"add          $2, %%edi \n\t"	/* move to the next row in Dest */
			"dec              %%edx \n\t"	/* decrease loop counter ROWS */
			"jnz            .L10320 \n\t"	/* check loop termination, proceed if required */
			/* --- */
			"emms                   \n\t"	/* exit MMX state */
			"popa                   \n\t":"=m" (Dest)	/* %0 */
			:"m"(Src),		/* %1 */
			"m"(rows),		/* %2 */
			"m"(columns),		/* %3 */
			"m"(Kernel),		/* %4 */
			"m"(Divisor)		/* %5 */
			);
#endif
#endif
		return (0);
	} else {
		/* No non-MMX implementation yet */
		return (-1);
	}
}

/*!
\brief Filter using ConvolveKernel5x5Divide: Dij = saturation0and255( ... ) 

\param Src The source 2D byte array to convolve. Should be different from destination.
\param Dest The destination 2D byte array to store the result in. Should be different from source.
\param rows Number of rows in source/destination array. Must be >4.
\param columns Number of columns in source/destination array. Must be >4.
\param Kernel The 2D convolution kernel of size 5x5.
\param Divisor The divisor of the convolution sum. Must be >0.

Note: Non-MMX implementation not available for this function.

\return Returns 1 if filter was applied, 0 otherwise.
*/
int SDL_imageFilterConvolveKernel5x5Divide(unsigned char *Src, unsigned char *Dest, int rows, int columns,
										   signed short *Kernel, unsigned char Divisor)
{
	/* Validate input parameters */
	if ((Src == NULL) || (Dest == NULL) || (Kernel == NULL))
		return(-1);

	if ((columns < 5) || (rows < 5) || (Divisor == 0))
		return (-1);

	if ((SDL_imageFilterMMXdetect())) {
//#ifdef USE_MMX
#if defined(USE_MMX) && defined(i386)
#if !defined(GCC__)
		__asm
		{
			pusha
				pxor mm0, mm0   	/* zero MM0 */
				xor ebx, ebx   	/* zero EBX */
				mov bl, Divisor   	/* load Divisor into BL */
				movd mm5, ebx   	/* copy Divisor into MM5 */
				mov edx, Kernel   	/* load Kernel address into EDX */
				mov esi, Src   	/* load Src  address to ESI */
				mov edi, Dest   	/* load Dest address to EDI */
				add edi, 2   	/* 2 column offset from the left edge */
				mov eax, columns   	/* load columns into EAX */
				shl eax, 1   	/* EAX = columns * 2 */
				add edi, eax   	/* 2 row offset from the top edge */
				shr eax, 1   	/* EAX = columns */
				mov ebx, rows   	/* initialize ROWS counter */
				sub ebx, 4   	/* do not use first 2 and last 2 rows */
				/* ---, */
L10330:
			mov ecx, eax   	/* initialize COLUMNS counter */
				sub ecx, 4   	/* do not use first 2 and last 2 columns */
				align 16                 	/* 16 byte alignment of the loop entry */
L10332:
			pxor mm7, mm7   	/* zero MM7 (accumulator) */
				movd mm6, esi   	/* save ESI in MM6 */
				/* --- 1 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm2, mm1   	/* copy MM1 into MM2 */
				add esi, eax   	/* move Src pointer 1 row below */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				movq mm4, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				punpckhbw mm2, mm0   	/* unpack second 4 bytes into words */
				pmullw mm1, mm3   	/* mult 4 low  words of Src and Kernel */
				pmullw mm2, mm4   	/* mult 4 high words of Src and Kernel */
				paddsw mm1, mm2   	/* add 4 words of the high and low bytes */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				/* --- 2 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm2, mm1   	/* copy MM1 into MM2 */
				add esi, eax   	/* move Src pointer 1 row below */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				movq mm4, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				punpckhbw mm2, mm0   	/* unpack second 4 bytes into words */
				pmullw mm1, mm3   	/* mult 4 low  words of Src and Kernel */
				pmullw mm2, mm4   	/* mult 4 high words of Src and Kernel */
				paddsw mm1, mm2   	/* add 4 words of the high and low bytes */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				/* --- 3 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm2, mm1   	/* copy MM1 into MM2 */
				add esi, eax   	/* move Src pointer 1 row below */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				movq mm4, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				punpckhbw mm2, mm0   	/* unpack second 4 bytes into words */
				pmullw mm1, mm3   	/* mult 4 low  words of Src and Kernel */
				pmullw mm2, mm4   	/* mult 4 high words of Src and Kernel */
				paddsw mm1, mm2   	/* add 4 words of the high and low bytes */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				/* --- 4 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm2, mm1   	/* copy MM1 into MM2 */
				add esi, eax   	/* move Src pointer 1 row below */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				movq mm4, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				punpckhbw mm2, mm0   	/* unpack second 4 bytes into words */
				pmullw mm1, mm3   	/* mult 4 low  words of Src and Kernel */
				pmullw mm2, mm4   	/* mult 4 high words of Src and Kernel */
				paddsw mm1, mm2   	/* add 4 words of the high and low bytes */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				/* --- 5 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm2, mm1   	/* copy MM1 into MM2 */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				movq mm4, [edx]   	/* load 4 words of Kernel */
			punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				punpckhbw mm2, mm0   	/* unpack second 4 bytes into words */
				pmullw mm1, mm3   	/* mult 4 low  words of Src and Kernel */
				pmullw mm2, mm4   	/* mult 4 high words of Src and Kernel */
				paddsw mm1, mm2   	/* add 4 words of the high and low bytes */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				/* ---, */
				movq mm3, mm7   	/* copy MM7 into MM3 */
				psrlq mm7, 32   	/* shift 2 left words to the right */
				paddsw mm7, mm3   	/* add 2 left and 2 right result words */
				movq mm2, mm7   	/* copy MM7 into MM2 */
				psrlq mm7, 16   	/* shift 1 left word to the right */
				paddsw mm7, mm2   	/* add 1 left and 1 right result words */
				/* ---, */
				movd mm1, eax   	/* save EDX in MM1 */
				movd mm2, ebx   	/* save EDX in MM2 */
				movd mm3, edx   	/* save EDX in MM3 */
				movd eax, mm7   	/* load summation result into EAX */
				psraw mm7, 15   	/* spread sign bit of the result */
				movd ebx, mm5   	/* load Divisor into EBX */
				movd edx, mm7   	/* fill EDX with a sign bit */
				idiv bx    	/* IDIV - VERY EXPENSIVE */
				movd mm7, eax   	/* move result of division into MM7 */
				packuswb mm7, mm0   	/* pack division result with saturation */
				movd eax, mm7   	/* copy saturated result into EAX */
				mov [edi], al   	/* copy a byte result into Dest */
				movd edx, mm3   	/* restore saved EDX */
				movd ebx, mm2   	/* restore saved EBX */
				movd eax, mm1   	/* restore saved EAX */
				/* --, */
				movd esi, mm6   	/* move Src pointer to the top pixel */
				sub edx, 72   	/* EDX = Kernel address */
				inc              esi    	/* move Src  pointer to the next pixel */
				inc              edi    	/* move Dest pointer to the next pixel */
				/* ---, */
				dec              ecx    	/* decrease loop counter COLUMNS */
				jnz            L10332    	/* check loop termination, proceed if required */
				add esi, 4   	/* move to the next row in Src */
				add edi, 4   	/* move to the next row in Dest */
				dec              ebx    	/* decrease loop counter ROWS */
				jnz            L10330    	/* check loop termination, proceed if required */
				/* ---, */
				emms                      	/* exit MMX state */
				popa
		}
#else
		asm volatile
			("pusha		     \n\t" "pxor      %%mm0, %%mm0 \n\t"	/* zero MM0 */
			"xor       %%ebx, %%ebx \n\t"	/* zero EBX */
			"mov           %5, %%bl \n\t"	/* load Divisor into BL */
			"movd      %%ebx, %%mm5 \n\t"	/* copy Divisor into MM5 */
			"mov          %4, %%edx \n\t"	/* load Kernel address into EDX */
			"mov          %1, %%esi \n\t"	/* load Src  address to ESI */
			"mov          %0, %%edi \n\t"	/* load Dest address to EDI */
			"add          $2, %%edi \n\t"	/* 2 column offset from the left edge */
			"mov          %3, %%eax \n\t"	/* load columns into EAX */
			"shl          $1, %%eax \n\t"	/* EAX = columns * 2 */
			"add       %%eax, %%edi \n\t"	/* 2 row offset from the top edge */
			"shr          $1, %%eax \n\t"	/* EAX = columns */
			"mov          %2, %%ebx \n\t"	/* initialize ROWS counter */
			"sub          $4, %%ebx \n\t"	/* do not use first 2 and last 2 rows */
			/* --- */
			".L10330:               \n\t" "mov       %%eax, %%ecx \n\t"	/* initialize COLUMNS counter */
			"sub          $4, %%ecx \n\t"	/* do not use first 2 and last 2 columns */
			".align 16              \n\t"	/* 16 byte alignment of the loop entry */
			".L10332:               \n\t" "pxor      %%mm7, %%mm7 \n\t"	/* zero MM7 (accumulator) */
			"movd      %%esi, %%mm6 \n\t"	/* save ESI in MM6 */
			/* --- 1 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"add       %%eax, %%esi \n\t"	/* move Src pointer 1 row below */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"movq    (%%edx), %%mm4 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"punpckhbw %%mm0, %%mm2 \n\t"	/* unpack second 4 bytes into words */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"pmullw    %%mm4, %%mm2 \n\t"	/* mult. 4 high words of Src and Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the high and low bytes */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			/* --- 2 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"add       %%eax, %%esi \n\t"	/* move Src pointer 1 row below */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"movq    (%%edx), %%mm4 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"punpckhbw %%mm0, %%mm2 \n\t"	/* unpack second 4 bytes into words */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"pmullw    %%mm4, %%mm2 \n\t"	/* mult. 4 high words of Src and Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the high and low bytes */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			/* --- 3 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"add       %%eax, %%esi \n\t"	/* move Src pointer 1 row below */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"movq    (%%edx), %%mm4 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"punpckhbw %%mm0, %%mm2 \n\t"	/* unpack second 4 bytes into words */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"pmullw    %%mm4, %%mm2 \n\t"	/* mult. 4 high words of Src and Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the high and low bytes */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			/* --- 4 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"add       %%eax, %%esi \n\t"	/* move Src pointer 1 row below */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"movq    (%%edx), %%mm4 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"punpckhbw %%mm0, %%mm2 \n\t"	/* unpack second 4 bytes into words */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"pmullw    %%mm4, %%mm2 \n\t"	/* mult. 4 high words of Src and Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the high and low bytes */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			/* --- 5 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"movq    (%%edx), %%mm4 \n\t"	/* load 4 words of Kernel */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"punpckhbw %%mm0, %%mm2 \n\t"	/* unpack second 4 bytes into words */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"pmullw    %%mm4, %%mm2 \n\t"	/* mult. 4 high words of Src and Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the high and low bytes */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			/* --- */
			"movq      %%mm7, %%mm3 \n\t"	/* copy MM7 into MM3 */
			"psrlq       $32, %%mm7 \n\t"	/* shift 2 left words to the right */
			"paddsw    %%mm3, %%mm7 \n\t"	/* add 2 left and 2 right result words */
			"movq      %%mm7, %%mm2 \n\t"	/* copy MM7 into MM2 */
			"psrlq       $16, %%mm7 \n\t"	/* shift 1 left word to the right */
			"paddsw    %%mm2, %%mm7 \n\t"	/* add 1 left and 1 right result words */
			/* --- */
			"movd      %%eax, %%mm1 \n\t"	/* save EDX in MM1 */
			"movd      %%ebx, %%mm2 \n\t"	/* save EDX in MM2 */
			"movd      %%edx, %%mm3 \n\t"	/* save EDX in MM3 */
			"movd      %%mm7, %%eax \n\t"	/* load summation result into EAX */
			"psraw       $15, %%mm7 \n\t"	/* spread sign bit of the result */
			"movd      %%mm5, %%ebx \n\t"	/* load Divisor into EBX */
			"movd      %%mm7, %%edx \n\t"	/* fill EDX with a sign bit */
			"idivw             %%bx \n\t"	/* IDIV - VERY EXPENSIVE */
			"movd      %%eax, %%mm7 \n\t"	/* move result of division into MM7 */
			"packuswb  %%mm0, %%mm7 \n\t"	/* pack division result with saturation */
			"movd      %%mm7, %%eax \n\t"	/* copy saturated result into EAX */
			"mov      %%al, (%%edi) \n\t"	/* copy a byte result into Dest */
			"movd      %%mm3, %%edx \n\t"	/* restore saved EDX */
			"movd      %%mm2, %%ebx \n\t"	/* restore saved EBX */
			"movd      %%mm1, %%eax \n\t"	/* restore saved EAX */
			/* -- */
			"movd      %%mm6, %%esi \n\t"	/* move Src pointer to the top pixel */
			"sub         $72, %%edx \n\t"	/* EDX = Kernel address */
			"inc              %%esi \n\t"	/* move Src  pointer to the next pixel */
			"inc              %%edi \n\t"	/* move Dest pointer to the next pixel */
			/* --- */
			"dec              %%ecx \n\t"	/* decrease loop counter COLUMNS */
			"jnz            .L10332 \n\t"	/* check loop termination, proceed if required */
			"add          $4, %%esi \n\t"	/* move to the next row in Src */
			"add          $4, %%edi \n\t"	/* move to the next row in Dest */
			"dec              %%ebx \n\t"	/* decrease loop counter ROWS */
			"jnz            .L10330 \n\t"	/* check loop termination, proceed if required */
			/* --- */
			"emms                   \n\t"	/* exit MMX state */
			"popa                   \n\t":"=m" (Dest)	/* %0 */
			:"m"(Src),		/* %1 */
			"m"(rows),		/* %2 */
			"m"(columns),		/* %3 */
			"m"(Kernel),		/* %4 */
			"m"(Divisor)		/* %5 */
			);
#endif
#endif
		return (0);
	} else {
		/* No non-MMX implementation yet */
		return (-1);
	}
}

/*!
\brief Filter using ConvolveKernel7x7Divide: Dij = saturation0and255( ... ) 

\param Src The source 2D byte array to convolve. Should be different from destination.
\param Dest The destination 2D byte array to store the result in. Should be different from source.
\param rows Number of rows in source/destination array. Must be >6.
\param columns Number of columns in source/destination array. Must be >6.
\param Kernel The 2D convolution kernel of size 7x7.
\param Divisor The divisor of the convolution sum. Must be >0.

Note: Non-MMX implementation not available for this function.

\return Returns 1 if filter was applied, 0 otherwise.
*/
int SDL_imageFilterConvolveKernel7x7Divide(unsigned char *Src, unsigned char *Dest, int rows, int columns,
										   signed short *Kernel, unsigned char Divisor)
{
	/* Validate input parameters */
	if ((Src == NULL) || (Dest == NULL) || (Kernel == NULL))
		return(-1);

	if ((columns < 7) || (rows < 7) || (Divisor == 0))
		return (-1);

	if ((SDL_imageFilterMMXdetect())) {
//#ifdef USE_MMX
#if defined(USE_MMX) && defined(i386)
#if !defined(GCC__)
		__asm
		{
			pusha
				pxor mm0, mm0   	/* zero MM0 */
				xor ebx, ebx   	/* zero EBX */
				mov bl, Divisor   	/* load Divisor into BL */
				movd mm5, ebx   	/* copy Divisor into MM5 */
				mov edx, Kernel  	/* load Kernel address into EDX */
				mov esi, Src   	/* load Src  address to ESI */
				mov edi, Dest   	/* load Dest address to EDI */
				add edi, 3   	/* 3 column offset from the left edge */
				mov eax, columns   	/* load columns into EAX */
				add edi, eax   	/* 3 row offset from the top edge */
				add edi, eax
				add edi, eax
				mov ebx, rows   	/* initialize ROWS counter */
				sub ebx, 6   	/* do not use first 3 and last 3 rows */
				/* ---, */
L10340:
			mov ecx, eax   	/* initialize COLUMNS counter */
				sub ecx, 6   	/* do not use first 3 and last 3 columns */
				align 16                 	/* 16 byte alignment of the loop entry */
L10342:
			pxor mm7, mm7   	/* zero MM7 (accumulator) */
				movd mm6, esi   	/* save ESI in MM6 */
				/* --- 1 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm2, mm1   	/* copy MM1 into MM2 */
				add esi, eax   	/* move Src pointer 1 row below */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				movq mm4, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				punpckhbw mm2, mm0   	/* unpack second 4 bytes into words */
				pmullw mm1, mm3   	/* mult 4 low  words of Src and Kernel */
				pmullw mm2, mm4   	/* mult 4 high words of Src and Kernel */
				paddsw mm1, mm2   	/* add 4 words of the high and low bytes */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				/* --- 2 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm2, mm1   	/* copy MM1 into MM2 */
				add esi, eax   	/* move Src pointer 1 row below */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				movq mm4, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				punpckhbw mm2, mm0   	/* unpack second 4 bytes into words */
				pmullw mm1, mm3   	/* mult 4 low  words of Src and Kernel */
				pmullw mm2, mm4   	/* mult 4 high words of Src and Kernel */
				paddsw mm1, mm2   	/* add 4 words of the high and low bytes */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				/* --- 3 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm2, mm1   	/* copy MM1 into MM2 */
				add esi, eax   	/* move Src pointer 1 row below */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				movq mm4, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				punpckhbw mm2, mm0   	/* unpack second 4 bytes into words */
				pmullw mm1, mm3   	/* mult 4 low  words of Src and Kernel */
				pmullw mm2, mm4   	/* mult 4 high words of Src and Kernel */
				paddsw mm1, mm2   	/* add 4 words of the high and low bytes */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				/* --- 4 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm2, mm1   	/* copy MM1 into MM2 */
				add esi, eax   	/* move Src pointer 1 row below */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				movq mm4, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				punpckhbw mm2, mm0   	/* unpack second 4 bytes into words */
				pmullw mm1, mm3   	/* mult 4 low  words of Src and Kernel */
				pmullw mm2, mm4   	/* mult 4 high words of Src and Kernel */
				paddsw mm1, mm2   	/* add 4 words of the high and low bytes */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				/* --- 5 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm2, mm1   	/* copy MM1 into MM2 */
				add esi, eax   	/* move Src pointer 1 row below */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				movq mm4, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				punpckhbw mm2, mm0   	/* unpack second 4 bytes into words */
				pmullw mm1, mm3   	/* mult 4 low  words of Src and Kernel */
				pmullw mm2, mm4   	/* mult 4 high words of Src and Kernel */
				paddsw mm1, mm2   	/* add 4 words of the high and low bytes */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				/* --- 6 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm2, mm1   	/* copy MM1 into MM2 */
				add esi, eax   	/* move Src pointer 1 row below */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				movq mm4, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				punpckhbw mm2, mm0   	/* unpack second 4 bytes into words */
				pmullw mm1, mm3   	/* mult 4 low  words of Src and Kernel */
				pmullw mm2, mm4   	/* mult 4 high words of Src and Kernel */
				paddsw mm1, mm2   	/* add 4 words of the high and low bytes */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				/* --- 7 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm2, mm1   	/* copy MM1 into MM2 */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				movq mm4, [edx]   	/* load 4 words of Kernel */
			punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				punpckhbw mm2, mm0   	/* unpack second 4 bytes into words */
				pmullw mm1, mm3   	/* mult 4 low  words of Src and Kernel */
				pmullw mm2, mm4   	/* mult 4 high words of Src and Kernel */
				paddsw mm1, mm2   	/* add 4 words of the high and low bytes */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				/* ---, */
				movq mm3, mm7   	/* copy MM7 into MM3 */
				psrlq mm7, 32   	/* shift 2 left words to the right */
				paddsw mm7, mm3   	/* add 2 left and 2 right result words */
				movq mm2, mm7   	/* copy MM7 into MM2 */
				psrlq mm7, 16   	/* shift 1 left word to the right */
				paddsw mm7, mm2   	/* add 1 left and 1 right result words */
				/* ---, */
				movd mm1, eax   	/* save EDX in MM1 */
				movd mm2, ebx   	/* save EDX in MM2 */
				movd mm3, edx   	/* save EDX in MM3 */
				movd eax, mm7   	/* load summation result into EAX */
				psraw mm7, 15   	/* spread sign bit of the result */
				movd ebx, mm5   	/* load Divisor into EBX */
				movd edx, mm7   	/* fill EDX with a sign bit */
				idiv bx    	/* IDIV - VERY EXPENSIVE */
				movd mm7, eax   	/* move result of division into MM7 */
				packuswb mm7, mm0   	/* pack division result with saturation */
				movd eax, mm7   	/* copy saturated result into EAX */
				mov [edi], al   	/* copy a byte result into Dest */
				movd edx, mm3   	/* restore saved EDX */
				movd ebx, mm2   	/* restore saved EBX */
				movd eax, mm1   	/* restore saved EAX */
				/* --, */
				movd esi, mm6   	/* move Src pointer to the top pixel */
				sub edx, 104   	/* EDX = Kernel address */
				inc              esi    	/* move Src  pointer to the next pixel */
				inc              edi    	/* move Dest pointer to the next pixel */
				/* ---, */
				dec              ecx    	/* decrease loop counter COLUMNS */
				jnz            L10342    	/* check loop termination, proceed if required */
				add esi, 6   	/* move to the next row in Src */
				add edi, 6   	/* move to the next row in Dest */
				dec              ebx    	/* decrease loop counter ROWS */
				jnz            L10340    	/* check loop termination, proceed if required */
				/* ---, */
				emms                      	/* exit MMX state */
				popa
		}
#else
		asm volatile
			("pusha		     \n\t" "pxor      %%mm0, %%mm0 \n\t"	/* zero MM0 */
			"xor       %%ebx, %%ebx \n\t"	/* zero EBX */
			"mov           %5, %%bl \n\t"	/* load Divisor into BL */
			"movd      %%ebx, %%mm5 \n\t"	/* copy Divisor into MM5 */
			"mov          %4, %%edx \n\t"	/* load Kernel address into EDX */
			"mov          %1, %%esi \n\t"	/* load Src  address to ESI */
			"mov          %0, %%edi \n\t"	/* load Dest address to EDI */
			"add          $3, %%edi \n\t"	/* 3 column offset from the left edge */
			"mov          %3, %%eax \n\t"	/* load columns into EAX */
			"add       %%eax, %%edi \n\t"	/* 3 row offset from the top edge */
			"add       %%eax, %%edi \n\t" "add       %%eax, %%edi \n\t" "mov          %2, %%ebx \n\t"	/* initialize ROWS counter */
			"sub          $6, %%ebx \n\t"	/* do not use first 3 and last 3 rows */
			/* --- */
			".L10340:               \n\t" "mov       %%eax, %%ecx \n\t"	/* initialize COLUMNS counter */
			"sub          $6, %%ecx \n\t"	/* do not use first 3 and last 3 columns */
			".align 16              \n\t"	/* 16 byte alignment of the loop entry */
			".L10342:               \n\t" "pxor      %%mm7, %%mm7 \n\t"	/* zero MM7 (accumulator) */
			"movd      %%esi, %%mm6 \n\t"	/* save ESI in MM6 */
			/* --- 1 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"add       %%eax, %%esi \n\t"	/* move Src pointer 1 row below */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"movq    (%%edx), %%mm4 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"punpckhbw %%mm0, %%mm2 \n\t"	/* unpack second 4 bytes into words */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"pmullw    %%mm4, %%mm2 \n\t"	/* mult. 4 high words of Src and Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the high and low bytes */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			/* --- 2 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"add       %%eax, %%esi \n\t"	/* move Src pointer 1 row below */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"movq    (%%edx), %%mm4 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"punpckhbw %%mm0, %%mm2 \n\t"	/* unpack second 4 bytes into words */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"pmullw    %%mm4, %%mm2 \n\t"	/* mult. 4 high words of Src and Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the high and low bytes */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			/* --- 3 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"add       %%eax, %%esi \n\t"	/* move Src pointer 1 row below */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"movq    (%%edx), %%mm4 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"punpckhbw %%mm0, %%mm2 \n\t"	/* unpack second 4 bytes into words */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"pmullw    %%mm4, %%mm2 \n\t"	/* mult. 4 high words of Src and Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the high and low bytes */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			/* --- 4 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"add       %%eax, %%esi \n\t"	/* move Src pointer 1 row below */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"movq    (%%edx), %%mm4 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"punpckhbw %%mm0, %%mm2 \n\t"	/* unpack second 4 bytes into words */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"pmullw    %%mm4, %%mm2 \n\t"	/* mult. 4 high words of Src and Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the high and low bytes */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			/* --- 5 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"add       %%eax, %%esi \n\t"	/* move Src pointer 1 row below */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"movq    (%%edx), %%mm4 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"punpckhbw %%mm0, %%mm2 \n\t"	/* unpack second 4 bytes into words */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"pmullw    %%mm4, %%mm2 \n\t"	/* mult. 4 high words of Src and Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the high and low bytes */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			/* --- 6 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"add       %%eax, %%esi \n\t"	/* move Src pointer 1 row below */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"movq    (%%edx), %%mm4 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"punpckhbw %%mm0, %%mm2 \n\t"	/* unpack second 4 bytes into words */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"pmullw    %%mm4, %%mm2 \n\t"	/* mult. 4 high words of Src and Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the high and low bytes */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			/* --- 7 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"movq    (%%edx), %%mm4 \n\t"	/* load 4 words of Kernel */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"punpckhbw %%mm0, %%mm2 \n\t"	/* unpack second 4 bytes into words */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"pmullw    %%mm4, %%mm2 \n\t"	/* mult. 4 high words of Src and Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the high and low bytes */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			/* --- */
			"movq      %%mm7, %%mm3 \n\t"	/* copy MM7 into MM3 */
			"psrlq       $32, %%mm7 \n\t"	/* shift 2 left words to the right */
			"paddsw    %%mm3, %%mm7 \n\t"	/* add 2 left and 2 right result words */
			"movq      %%mm7, %%mm2 \n\t"	/* copy MM7 into MM2 */
			"psrlq       $16, %%mm7 \n\t"	/* shift 1 left word to the right */
			"paddsw    %%mm2, %%mm7 \n\t"	/* add 1 left and 1 right result words */
			/* --- */
			"movd      %%eax, %%mm1 \n\t"	/* save EDX in MM1 */
			"movd      %%ebx, %%mm2 \n\t"	/* save EDX in MM2 */
			"movd      %%edx, %%mm3 \n\t"	/* save EDX in MM3 */
			"movd      %%mm7, %%eax \n\t"	/* load summation result into EAX */
			"psraw       $15, %%mm7 \n\t"	/* spread sign bit of the result */
			"movd      %%mm5, %%ebx \n\t"	/* load Divisor into EBX */
			"movd      %%mm7, %%edx \n\t"	/* fill EDX with a sign bit */
			"idivw             %%bx \n\t"	/* IDIV - VERY EXPENSIVE */
			"movd      %%eax, %%mm7 \n\t"	/* move result of division into MM7 */
			"packuswb  %%mm0, %%mm7 \n\t"	/* pack division result with saturation */
			"movd      %%mm7, %%eax \n\t"	/* copy saturated result into EAX */
			"mov      %%al, (%%edi) \n\t"	/* copy a byte result into Dest */
			"movd      %%mm3, %%edx \n\t"	/* restore saved EDX */
			"movd      %%mm2, %%ebx \n\t"	/* restore saved EBX */
			"movd      %%mm1, %%eax \n\t"	/* restore saved EAX */
			/* -- */
			"movd      %%mm6, %%esi \n\t"	/* move Src pointer to the top pixel */
			"sub        $104, %%edx \n\t"	/* EDX = Kernel address */
			"inc              %%esi \n\t"	/* move Src  pointer to the next pixel */
			"inc              %%edi \n\t"	/* move Dest pointer to the next pixel */
			/* --- */
			"dec              %%ecx \n\t"	/* decrease loop counter COLUMNS */
			"jnz            .L10342 \n\t"	/* check loop termination, proceed if required */
			"add          $6, %%esi \n\t"	/* move to the next row in Src */
			"add          $6, %%edi \n\t"	/* move to the next row in Dest */
			"dec              %%ebx \n\t"	/* decrease loop counter ROWS */
			"jnz            .L10340 \n\t"	/* check loop termination, proceed if required */
			/* --- */
			"emms                   \n\t"	/* exit MMX state */
			"popa                   \n\t":"=m" (Dest)	/* %0 */
			:"m"(Src),		/* %1 */
			"m"(rows),		/* %2 */
			"m"(columns),		/* %3 */
			"m"(Kernel),		/* %4 */
			"m"(Divisor)		/* %5 */
			);
#endif
#endif
		return (0);
	} else {
		/* No non-MMX implementation yet */
		return (-1);
	}
}

/*!
\brief Filter using ConvolveKernel9x9Divide: Dij = saturation0and255( ... ) 

\param Src The source 2D byte array to convolve. Should be different from destination.
\param Dest The destination 2D byte array to store the result in. Should be different from source.
\param rows Number of rows in source/destination array. Must be >8.
\param columns Number of columns in source/destination array. Must be >8.
\param Kernel The 2D convolution kernel of size 9x9.
\param Divisor The divisor of the convolution sum. Must be >0.

Note: Non-MMX implementation not available for this function.

\return Returns 1 if filter was applied, 0 otherwise.
*/
int SDL_imageFilterConvolveKernel9x9Divide(unsigned char *Src, unsigned char *Dest, int rows, int columns,
										   signed short *Kernel, unsigned char Divisor)
{
	/* Validate input parameters */
	if ((Src == NULL) || (Dest == NULL) || (Kernel == NULL))
		return(-1);

	if ((columns < 9) || (rows < 9) || (Divisor == 0))
		return (-1);

	if ((SDL_imageFilterMMXdetect())) {
//#ifdef USE_MMX
#if defined(USE_MMX) && defined(i386)
#if !defined(GCC__)
		__asm
		{
			pusha
				pxor mm0, mm0   	/* zero MM0 */
				xor ebx, ebx   	/* zero EBX */
				mov bl, Divisor   	/* load Divisor into BL */
				movd mm5, ebx   	/* copy Divisor into MM5 */
				mov edx, Kernel   	/* load Kernel address into EDX */
				mov esi, Src   	/* load Src  address to ESI */
				mov edi, Dest   	/* load Dest address to EDI */
				add edi, 4   	/* 4 column offset from the left edge */
				mov eax, columns   	/* load columns into EAX */
				add edi, eax   	/* 4 row offset from the top edge */
				add edi, eax
				add edi, eax
				add edi, eax
				mov ebx, rows   	/* initialize ROWS counter */
				sub ebx, 8   	/* do not use first 4 and last 4 rows */
				/* ---, */
L10350:
			mov ecx, eax   	/* initialize COLUMNS counter */
				sub ecx, 8   	/* do not use first 4 and last 4 columns */
				align 16                 	/* 16 byte alignment of the loop entry */
L10352:
			pxor mm7, mm7   	/* zero MM7 (accumulator) */
				movd mm6, esi   	/* save ESI in MM6 */
				/* --- 1 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm2, mm1   	/* copy MM1 into MM2 */
				inc              esi    	/* move pointer to the next 8 bytes of Src */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				movq mm4, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				punpckhbw mm2, mm0   	/* unpack second 4 bytes into words */
				pmullw mm1, mm3   	/* mult. 4 low  words of Src and Kernel */
				pmullw mm2, mm4   	/* mult. 4 high words of Src and Kernel */
				paddsw mm1, mm2   	/* add 4 words of the high and low bytes */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			dec              esi
				add esi, eax   	/* move Src pointer 1 row below */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				pmullw mm1, mm3   	/* mult. 4 low  words of Src and Kernel */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				/* --- 2 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm2, mm1   	/* copy MM1 into MM2 */
				inc              esi    	/* move pointer to the next 8 bytes of Src */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				movq mm4, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				punpckhbw mm2, mm0   	/* unpack second 4 bytes into words */
				pmullw mm1, mm3   	/* mult. 4 low  words of Src and Kernel */
				pmullw mm2, mm4   	/* mult. 4 high words of Src and Kernel */
				paddsw mm1, mm2   	/* add 4 words of the high and low bytes */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			dec              esi
				add esi, eax   	/* move Src pointer 1 row below */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				pmullw mm1, mm3   	/* mult. 4 low  words of Src and Kernel */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				/* --- 3 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm2, mm1   	/* copy MM1 into MM2 */
				inc              esi    	/* move pointer to the next 8 bytes of Src */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				movq mm4, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				punpckhbw mm2, mm0   	/* unpack second 4 bytes into words */
				pmullw mm1, mm3   	/* mult. 4 low  words of Src and Kernel */
				pmullw mm2, mm4   	/* mult. 4 high words of Src and Kernel */
				paddsw mm1, mm2   	/* add 4 words of the high and low bytes */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			dec              esi
				add esi, eax   	/* move Src pointer 1 row below */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				pmullw mm1, mm3   	/* mult. 4 low  words of Src and Kernel */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				/* --- 4 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm2, mm1   	/* copy MM1 into MM2 */
				inc              esi    	/* move pointer to the next 8 bytes of Src */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				movq mm4, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				punpckhbw mm2, mm0   	/* unpack second 4 bytes into words */
				pmullw mm1, mm3   	/* mult. 4 low  words of Src and Kernel */
				pmullw mm2, mm4   	/* mult. 4 high words of Src and Kernel */
				paddsw mm1, mm2   	/* add 4 words of the high and low bytes */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			dec              esi
				add esi, eax   	/* move Src pointer 1 row below */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				pmullw mm1, mm3   	/* mult. 4 low  words of Src and Kernel */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				/* --- 5 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm2, mm1   	/* copy MM1 into MM2 */
				inc              esi    	/* move pointer to the next 8 bytes of Src */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				movq mm4, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				punpckhbw mm2, mm0   	/* unpack second 4 bytes into words */
				pmullw mm1, mm3   	/* mult. 4 low  words of Src and Kernel */
				pmullw mm2, mm4   	/* mult. 4 high words of Src and Kernel */
				paddsw mm1, mm2   	/* add 4 words of the high and low bytes */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			dec              esi
				add esi, eax   	/* move Src pointer 1 row below */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				pmullw mm1, mm3   	/* mult. 4 low  words of Src and Kernel */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				/* --- 6 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm2, mm1   	/* copy MM1 into MM2 */
				inc              esi    	/* move pointer to the next 8 bytes of Src */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				movq mm4, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				punpckhbw mm2, mm0   	/* unpack second 4 bytes into words */
				pmullw mm1, mm3   	/* mult. 4 low  words of Src and Kernel */
				pmullw mm2, mm4   	/* mult. 4 high words of Src and Kernel */
				paddsw mm1, mm2   	/* add 4 words of the high and low bytes */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			dec              esi
				add esi, eax   	/* move Src pointer 1 row below */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				pmullw mm1, mm3   	/* mult. 4 low  words of Src and Kernel */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				/* --- 7 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm2, mm1   	/* copy MM1 into MM2 */
				inc              esi    	/* move pointer to the next 8 bytes of Src */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				movq mm4, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				punpckhbw mm2, mm0   	/* unpack second 4 bytes into words */
				pmullw mm1, mm3   	/* mult. 4 low  words of Src and Kernel */
				pmullw mm2, mm4   	/* mult. 4 high words of Src and Kernel */
				paddsw mm1, mm2   	/* add 4 words of the high and low bytes */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			dec              esi
				add esi, eax   	/* move Src pointer 1 row below */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				pmullw mm1, mm3   	/* mult. 4 low  words of Src and Kernel */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				/* --- 8 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm2, mm1   	/* copy MM1 into MM2 */
				inc              esi    	/* move pointer to the next 8 bytes of Src */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				movq mm4, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				punpckhbw mm2, mm0   	/* unpack second 4 bytes into words */
				pmullw mm1, mm3   	/* mult. 4 low  words of Src and Kernel */
				pmullw mm2, mm4   	/* mult. 4 high words of Src and Kernel */
				paddsw mm1, mm2   	/* add 4 words of the high and low bytes */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			dec              esi
				add esi, eax   	/* move Src pointer 1 row below */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				pmullw mm1, mm3   	/* mult. 4 low  words of Src and Kernel */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				/* --- 9 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm2, mm1   	/* copy MM1 into MM2 */
				inc              esi    	/* move pointer to the next 8 bytes of Src */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				movq mm4, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				punpckhbw mm2, mm0   	/* unpack second 4 bytes into words */
				pmullw mm1, mm3   	/* mult. 4 low  words of Src and Kernel */
				pmullw mm2, mm4   	/* mult. 4 high words of Src and Kernel */
				paddsw mm1, mm2   	/* add 4 words of the high and low bytes */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm3, [edx]   	/* load 4 words of Kernel */
			punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				pmullw mm1, mm3   	/* mult. 4 low  words of Src and Kernel */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				/* ---, */
				movq mm3, mm7   	/* copy MM7 into MM3 */
				psrlq mm7, 32   	/* shift 2 left words to the right */
				paddsw mm7, mm3   	/* add 2 left and 2 right result words */
				movq mm2, mm7   	/* copy MM7 into MM2 */
				psrlq mm7, 16   	/* shift 1 left word to the right */
				paddsw mm7, mm2   	/* add 1 left and 1 right result words */
				/* ---, */
				movd mm1, eax   	/* save EDX in MM1 */
				movd mm2, ebx   	/* save EDX in MM2 */
				movd mm3, edx   	/* save EDX in MM3 */
				movd eax, mm7   	/* load summation result into EAX */
				psraw mm7, 15   	/* spread sign bit of the result */
				movd ebx, mm5   	/* load Divisor into EBX */
				movd edx, mm7   	/* fill EDX with a sign bit */
				idiv bx    	/* IDIV - VERY EXPENSIVE */
				movd mm7, eax   	/* move result of division into MM7 */
				packuswb mm7, mm0   	/* pack division result with saturation */
				movd eax, mm7   	/* copy saturated result into EAX */
				mov [edi], al   	/* copy a byte result into Dest */
				movd edx, mm3   	/* restore saved EDX */
				movd ebx, mm2   	/* restore saved EBX */
				movd eax, mm1   	/* restore saved EAX */
				/* --, */
				movd esi, mm6   	/* move Src pointer to the top pixel */
				sub edx, 208   	/* EDX = Kernel address */
				inc              esi    	/* move Src  pointer to the next pixel */
				inc              edi    	/* move Dest pointer to the next pixel */
				/* ---, */
				dec              ecx    	/* decrease loop counter COLUMNS */
				jnz            L10352    	/* check loop termination, proceed if required */
				add esi, 8   	/* move to the next row in Src */
				add edi, 8   	/* move to the next row in Dest */
				dec              ebx    	/* decrease loop counter ROWS */
				jnz            L10350    	/* check loop termination, proceed if required */
				/* ---, */
				emms                      	/* exit MMX state */
				popa
		}
#else
		asm volatile
			("pusha		     \n\t" "pxor      %%mm0, %%mm0 \n\t"	/* zero MM0 */
			"xor       %%ebx, %%ebx \n\t"	/* zero EBX */
			"mov           %5, %%bl \n\t"	/* load Divisor into BL */
			"movd      %%ebx, %%mm5 \n\t"	/* copy Divisor into MM5 */
			"mov          %4, %%edx \n\t"	/* load Kernel address into EDX */
			"mov          %1, %%esi \n\t"	/* load Src  address to ESI */
			"mov          %0, %%edi \n\t"	/* load Dest address to EDI */
			"add          $4, %%edi \n\t"	/* 4 column offset from the left edge */
			"mov          %3, %%eax \n\t"	/* load columns into EAX */
			"add       %%eax, %%edi \n\t"	/* 4 row offset from the top edge */
			"add       %%eax, %%edi \n\t" "add       %%eax, %%edi \n\t" "add       %%eax, %%edi \n\t" "mov          %2, %%ebx \n\t"	/* initialize ROWS counter */
			"sub          $8, %%ebx \n\t"	/* do not use first 4 and last 4 rows */
			/* --- */
			".L10350:               \n\t" "mov       %%eax, %%ecx \n\t"	/* initialize COLUMNS counter */
			"sub          $8, %%ecx \n\t"	/* do not use first 4 and last 4 columns */
			".align 16              \n\t"	/* 16 byte alignment of the loop entry */
			".L10352:               \n\t" "pxor      %%mm7, %%mm7 \n\t"	/* zero MM7 (accumulator) */
			"movd      %%esi, %%mm6 \n\t"	/* save ESI in MM6 */
			/* --- 1 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"inc              %%esi \n\t"	/* move pointer to the next 8 bytes of Src */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"movq    (%%edx), %%mm4 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"punpckhbw %%mm0, %%mm2 \n\t"	/* unpack second 4 bytes into words */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"pmullw    %%mm4, %%mm2 \n\t"	/* mult. 4 high words of Src and Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the high and low bytes */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"dec              %%esi \n\t" "add       %%eax, %%esi \n\t"	/* move Src pointer 1 row below */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			/* --- 2 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"inc              %%esi \n\t"	/* move pointer to the next 8 bytes of Src */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"movq    (%%edx), %%mm4 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"punpckhbw %%mm0, %%mm2 \n\t"	/* unpack second 4 bytes into words */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"pmullw    %%mm4, %%mm2 \n\t"	/* mult. 4 high words of Src and Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the high and low bytes */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"dec              %%esi \n\t" "add       %%eax, %%esi \n\t"	/* move Src pointer 1 row below */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			/* --- 3 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"inc              %%esi \n\t"	/* move pointer to the next 8 bytes of Src */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"movq    (%%edx), %%mm4 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"punpckhbw %%mm0, %%mm2 \n\t"	/* unpack second 4 bytes into words */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"pmullw    %%mm4, %%mm2 \n\t"	/* mult. 4 high words of Src and Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the high and low bytes */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"dec              %%esi \n\t" "add       %%eax, %%esi \n\t"	/* move Src pointer 1 row below */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			/* --- 4 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"inc              %%esi \n\t"	/* move pointer to the next 8 bytes of Src */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"movq    (%%edx), %%mm4 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"punpckhbw %%mm0, %%mm2 \n\t"	/* unpack second 4 bytes into words */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"pmullw    %%mm4, %%mm2 \n\t"	/* mult. 4 high words of Src and Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the high and low bytes */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"dec              %%esi \n\t" "add       %%eax, %%esi \n\t"	/* move Src pointer 1 row below */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			/* --- 5 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"inc              %%esi \n\t"	/* move pointer to the next 8 bytes of Src */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"movq    (%%edx), %%mm4 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"punpckhbw %%mm0, %%mm2 \n\t"	/* unpack second 4 bytes into words */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"pmullw    %%mm4, %%mm2 \n\t"	/* mult. 4 high words of Src and Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the high and low bytes */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"dec              %%esi \n\t" "add       %%eax, %%esi \n\t"	/* move Src pointer 1 row below */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			/* --- 6 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"inc              %%esi \n\t"	/* move pointer to the next 8 bytes of Src */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"movq    (%%edx), %%mm4 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"punpckhbw %%mm0, %%mm2 \n\t"	/* unpack second 4 bytes into words */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"pmullw    %%mm4, %%mm2 \n\t"	/* mult. 4 high words of Src and Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the high and low bytes */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"dec              %%esi \n\t" "add       %%eax, %%esi \n\t"	/* move Src pointer 1 row below */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			/* --- 7 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"inc              %%esi \n\t"	/* move pointer to the next 8 bytes of Src */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"movq    (%%edx), %%mm4 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"punpckhbw %%mm0, %%mm2 \n\t"	/* unpack second 4 bytes into words */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"pmullw    %%mm4, %%mm2 \n\t"	/* mult. 4 high words of Src and Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the high and low bytes */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"dec              %%esi \n\t" "add       %%eax, %%esi \n\t"	/* move Src pointer 1 row below */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			/* --- 8 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"inc              %%esi \n\t"	/* move pointer to the next 8 bytes of Src */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"movq    (%%edx), %%mm4 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"punpckhbw %%mm0, %%mm2 \n\t"	/* unpack second 4 bytes into words */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"pmullw    %%mm4, %%mm2 \n\t"	/* mult. 4 high words of Src and Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the high and low bytes */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"dec              %%esi \n\t" "add       %%eax, %%esi \n\t"	/* move Src pointer 1 row below */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			/* --- 9 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"inc              %%esi \n\t"	/* move pointer to the next 8 bytes of Src */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"movq    (%%edx), %%mm4 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"punpckhbw %%mm0, %%mm2 \n\t"	/* unpack second 4 bytes into words */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"pmullw    %%mm4, %%mm2 \n\t"	/* mult. 4 high words of Src and Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the high and low bytes */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			/* --- */
			"movq      %%mm7, %%mm3 \n\t"	/* copy MM7 into MM3 */
			"psrlq       $32, %%mm7 \n\t"	/* shift 2 left words to the right */
			"paddsw    %%mm3, %%mm7 \n\t"	/* add 2 left and 2 right result words */
			"movq      %%mm7, %%mm2 \n\t"	/* copy MM7 into MM2 */
			"psrlq       $16, %%mm7 \n\t"	/* shift 1 left word to the right */
			"paddsw    %%mm2, %%mm7 \n\t"	/* add 1 left and 1 right result words */
			/* --- */
			"movd      %%eax, %%mm1 \n\t"	/* save EDX in MM1 */
			"movd      %%ebx, %%mm2 \n\t"	/* save EDX in MM2 */
			"movd      %%edx, %%mm3 \n\t"	/* save EDX in MM3 */
			"movd      %%mm7, %%eax \n\t"	/* load summation result into EAX */
			"psraw       $15, %%mm7 \n\t"	/* spread sign bit of the result */
			"movd      %%mm5, %%ebx \n\t"	/* load Divisor into EBX */
			"movd      %%mm7, %%edx \n\t"	/* fill EDX with a sign bit */
			"idivw             %%bx \n\t"	/* IDIV - VERY EXPENSIVE */
			"movd      %%eax, %%mm7 \n\t"	/* move result of division into MM7 */
			"packuswb  %%mm0, %%mm7 \n\t"	/* pack division result with saturation */
			"movd      %%mm7, %%eax \n\t"	/* copy saturated result into EAX */
			"mov      %%al, (%%edi) \n\t"	/* copy a byte result into Dest */
			"movd      %%mm3, %%edx \n\t"	/* restore saved EDX */
			"movd      %%mm2, %%ebx \n\t"	/* restore saved EBX */
			"movd      %%mm1, %%eax \n\t"	/* restore saved EAX */
			/* -- */
			"movd      %%mm6, %%esi \n\t"	/* move Src pointer to the top pixel */
			"sub        $208, %%edx \n\t"	/* EDX = Kernel address */
			"inc              %%esi \n\t"	/* move Src  pointer to the next pixel */
			"inc              %%edi \n\t"	/* move Dest pointer to the next pixel */
			/* --- */
			"dec              %%ecx \n\t"	/* decrease loop counter COLUMNS */
			"jnz            .L10352 \n\t"	/* check loop termination, proceed if required */
			"add          $8, %%esi \n\t"	/* move to the next row in Src */
			"add          $8, %%edi \n\t"	/* move to the next row in Dest */
			"dec              %%ebx \n\t"	/* decrease loop counter ROWS */
			"jnz            .L10350 \n\t"	/* check loop termination, proceed if required */
			/* --- */
			"emms                   \n\t"	/* exit MMX state */
			"popa                   \n\t":"=m" (Dest)	/* %0 */
			:"m"(Src),		/* %1 */
			"m"(rows),		/* %2 */
			"m"(columns),		/* %3 */
			"m"(Kernel),		/* %4 */
			"m"(Divisor)		/* %5 */
			);
#endif
#endif
		return (0);
	} else {
		/* No non-MMX implementation yet */
		return (-1);
	}
}

/*!
\brief Filter using ConvolveKernel3x3ShiftRight: Dij = saturation0and255( ... ) 

\param Src The source 2D byte array to convolve. Should be different from destination.
\param Dest The destination 2D byte array to store the result in. Should be different from source.
\param rows Number of rows in source/destination array. Must be >2.
\param columns Number of columns in source/destination array. Must be >2.
\param Kernel The 2D convolution kernel of size 3x3.
\param NRightShift The number of right bit shifts to apply to the convolution sum. Must be <7.

Note: Non-MMX implementation not available for this function.

\return Returns 1 if filter was applied, 0 otherwise.
*/
int SDL_imageFilterConvolveKernel3x3ShiftRight(unsigned char *Src, unsigned char *Dest, int rows, int columns,
											   signed short *Kernel, unsigned char NRightShift)
{
	/* Validate input parameters */
	if ((Src == NULL) || (Dest == NULL) || (Kernel == NULL))
		return(-1);

	if ((columns < 3) || (rows < 3) || (NRightShift > 7))
		return (-1);

	if ((SDL_imageFilterMMXdetect())) {
//#ifdef USE_MMX
#if defined(USE_MMX) && defined(i386)
#if !defined(GCC__)
		__asm
		{
			pusha
				pxor mm0, mm0   	/* zero MM0 */
				xor ebx, ebx   	/* zero EBX */
				mov bl, NRightShift   	/* load NRightShift into BL */
				movd mm4, ebx   	/* copy NRightShift into MM4 */
				mov edx, Kernel   	/* load Kernel address into EDX */
				movq mm5, [edx]   	/* MM5 = {0,K2,K1,K0} */
			add edx, 8   	/* second row              |K0 K1 K2 0| */
				movq mm6, [edx]   	/* MM6 = {0,K5,K4,K3}  K = |K3 K4 K5 0| */
			add edx, 8   	/* third row               |K6 K7 K8 0| */
				movq mm7, [edx]   	/* MM7 = {0,K8,K7,K6} */
			/* ---, */
			mov eax, columns   	/* load columns into EAX */
				mov esi, Src   	/* ESI = Src row 0 address */
				mov edi, Dest   	/* load Dest address to EDI */
				add edi, eax   	/* EDI = EDI + columns */
				inc              edi    	/* 1 byte offset from the left edge */
				mov edx, rows   	/* initialize ROWS counter */
				sub edx, 2   	/* do not use first and last row */
				/* ---, */
L10360:
			mov ecx, eax   	/* initialize COLUMS counter */
				sub ecx, 2   	/* do not use first and last column */
				align 16                 	/* 16 byte alignment of the loop entry */
L10362:
			/* ---, */
			movq mm1, [esi]   	/* load 8 bytes of the image first row */
			add esi, eax   	/* move one row below */
				movq mm2, [esi]   	/* load 8 bytes of the image second row */
			add esi, eax   	/* move one row below */
				movq mm3, [esi]   	/* load 8 bytes of the image third row */
			punpcklbw mm1, mm0   	/* unpack first 4 bytes into words */
				punpcklbw mm2, mm0   	/* unpack first 4 bytes into words */
				punpcklbw mm3, mm0   	/* unpack first 4 bytes into words */
				psrlw mm1, mm4   	/* shift right each pixel NshiftRight times */
				psrlw mm2, mm4   	/* shift right each pixel NshiftRight times */
				psrlw mm3, mm4   	/* shift right each pixel NshiftRight times */
				pmullw mm1, mm5   	/* multiply words first row  image*Kernel */
				pmullw mm2, mm6   	/* multiply words second row image*Kernel */
				pmullw mm3, mm7   	/* multiply words third row  image*Kernel */
				paddsw mm1, mm2   	/* add 4 words of the first and second rows */
				paddsw mm1, mm3   	/* add 4 words of the third row and result */
				movq mm2, mm1   	/* copy MM1 into MM2 */
				psrlq mm1, 32   	/* shift 2 left words to the right */
				paddsw mm1, mm2   	/* add 2 left and 2 right result words */
				movq mm3, mm1   	/* copy MM1 into MM3 */
				psrlq mm1, 16   	/* shift 1 left word to the right */
				paddsw mm1, mm3   	/* add 1 left and 1 right result words */
				packuswb mm1, mm0   	/* pack shift result with saturation */
				movd ebx, mm1   	/* copy saturated result into EBX */
				mov [edi], bl   	/* copy a byte result into Dest */
				/* --, */
				sub esi, eax   	/* move two rows up */
				sub esi, eax
				inc              esi    	/* move Src  pointer to the next pixel */
				inc              edi    	/* move Dest pointer to the next pixel */
				/* ---, */
				dec              ecx    	/* decrease loop counter COLUMNS */
				jnz            L10362    	/* check loop termination, proceed if required */
				add esi, 2   	/* move to the next row in Src */
				add edi, 2   	/* move to the next row in Dest */
				dec              edx    	/* decrease loop counter ROWS */
				jnz            L10360    	/* check loop termination, proceed if required */
				/* ---, */
				emms                      	/* exit MMX state */
				popa
		}
#else
		asm volatile
			("pusha		     \n\t" "pxor      %%mm0, %%mm0 \n\t"	/* zero MM0 */
			"xor       %%ebx, %%ebx \n\t"	/* zero EBX */
			"mov           %5, %%bl \n\t"	/* load NRightShift into BL */
			"movd      %%ebx, %%mm4 \n\t"	/* copy NRightShift into MM4 */
			"mov          %4, %%edx \n\t"	/* load Kernel address into EDX */
			"movq    (%%edx), %%mm5 \n\t"	/* MM5 = {0,K2,K1,K0} */
			"add          $8, %%edx \n\t"	/* second row              |K0 K1 K2 0| */
			"movq    (%%edx), %%mm6 \n\t"	/* MM6 = {0,K5,K4,K3}  K = |K3 K4 K5 0| */
			"add          $8, %%edx \n\t"	/* third row               |K6 K7 K8 0| */
			"movq    (%%edx), %%mm7 \n\t"	/* MM7 = {0,K8,K7,K6} */
			/* --- */
			"mov          %3, %%eax \n\t"	/* load columns into EAX */
			"mov          %1, %%esi \n\t"	/* ESI = Src row 0 address */
			"mov          %0, %%edi \n\t"	/* load Dest address to EDI */
			"add       %%eax, %%edi \n\t"	/* EDI = EDI + columns */
			"inc              %%edi \n\t"	/* 1 byte offset from the left edge */
			"mov          %2, %%edx \n\t"	/* initialize ROWS counter */
			"sub          $2, %%edx \n\t"	/* do not use first and last row */
			/* --- */
			".L10360:               \n\t" "mov       %%eax, %%ecx \n\t"	/* initialize COLUMS counter */
			"sub          $2, %%ecx \n\t"	/* do not use first and last column */
			".align 16              \n\t"	/* 16 byte alignment of the loop entry */
			".L10362:               \n\t"
			/* --- */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the image first row */
			"add       %%eax, %%esi \n\t"	/* move one row below */
			"movq    (%%esi), %%mm2 \n\t"	/* load 8 bytes of the image second row */
			"add       %%eax, %%esi \n\t"	/* move one row below */
			"movq    (%%esi), %%mm3 \n\t"	/* load 8 bytes of the image third row */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first 4 bytes into words */
			"punpcklbw %%mm0, %%mm2 \n\t"	/* unpack first 4 bytes into words */
			"punpcklbw %%mm0, %%mm3 \n\t"	/* unpack first 4 bytes into words */
			"psrlw     %%mm4, %%mm1 \n\t"	/* shift right each pixel NshiftRight times */
			"psrlw     %%mm4, %%mm2 \n\t"	/* shift right each pixel NshiftRight times */
			"psrlw     %%mm4, %%mm3 \n\t"	/* shift right each pixel NshiftRight times */
			"pmullw    %%mm5, %%mm1 \n\t"	/* multiply words first row  image*Kernel */
			"pmullw    %%mm6, %%mm2 \n\t"	/* multiply words second row image*Kernel */
			"pmullw    %%mm7, %%mm3 \n\t"	/* multiply words third row  image*Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the first and second rows */
			"paddsw    %%mm3, %%mm1 \n\t"	/* add 4 words of the third row and result */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"psrlq       $32, %%mm1 \n\t"	/* shift 2 left words to the right */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 2 left and 2 right result words */
			"movq      %%mm1, %%mm3 \n\t"	/* copy MM1 into MM3 */
			"psrlq       $16, %%mm1 \n\t"	/* shift 1 left word to the right */
			"paddsw    %%mm3, %%mm1 \n\t"	/* add 1 left and 1 right result words */
			"packuswb  %%mm0, %%mm1 \n\t"	/* pack shift result with saturation */
			"movd      %%mm1, %%ebx \n\t"	/* copy saturated result into EBX */
			"mov      %%bl, (%%edi) \n\t"	/* copy a byte result into Dest */
			/* -- */
			"sub       %%eax, %%esi \n\t"	/* move two rows up */
			"sub       %%eax, %%esi \n\t" "inc              %%esi \n\t"	/* move Src  pointer to the next pixel */
			"inc              %%edi \n\t"	/* move Dest pointer to the next pixel */
			/* --- */
			"dec              %%ecx \n\t"	/* decrease loop counter COLUMNS */
			"jnz            .L10362 \n\t"	/* check loop termination, proceed if required */
			"add          $2, %%esi \n\t"	/* move to the next row in Src */
			"add          $2, %%edi \n\t"	/* move to the next row in Dest */
			"dec              %%edx \n\t"	/* decrease loop counter ROWS */
			"jnz            .L10360 \n\t"	/* check loop termination, proceed if required */
			/* --- */
			"emms                   \n\t"	/* exit MMX state */
			"popa                   \n\t":"=m" (Dest)	/* %0 */
			:"m"(Src),		/* %1 */
			"m"(rows),		/* %2 */
			"m"(columns),		/* %3 */
			"m"(Kernel),		/* %4 */
			"m"(NRightShift)	/* %5 */
			);
#endif
#endif
		return (0);
	} else {
		/* No non-MMX implementation yet */
		return (-1);
	}
}

/*!
\brief Filter using ConvolveKernel5x5ShiftRight: Dij = saturation0and255( ... ) 

\param Src The source 2D byte array to convolve. Should be different from destination.
\param Dest The destination 2D byte array to store the result in. Should be different from source.
\param rows Number of rows in source/destination array. Must be >4.
\param columns Number of columns in source/destination array. Must be >4.
\param Kernel The 2D convolution kernel of size 5x5.
\param NRightShift The number of right bit shifts to apply to the convolution sum. Must be <7.

Note: Non-MMX implementation not available for this function.

\return Returns 1 if filter was applied, 0 otherwise.
*/
int SDL_imageFilterConvolveKernel5x5ShiftRight(unsigned char *Src, unsigned char *Dest, int rows, int columns,
											   signed short *Kernel, unsigned char NRightShift)
{
	/* Validate input parameters */
	if ((Src == NULL) || (Dest == NULL) || (Kernel == NULL))
		return(-1);

	if ((columns < 5) || (rows < 5) || (NRightShift > 7))
		return (-1);

	if ((SDL_imageFilterMMXdetect())) {
//#ifdef USE_MMX
#if defined(USE_MMX) && defined(i386)
#if !defined(GCC__)
		__asm
		{
			pusha
				pxor mm0, mm0   	/* zero MM0 */
				xor ebx, ebx   	/* zero EBX */
				mov bl, NRightShift   	/* load NRightShift into BL */
				movd mm5, ebx   	/* copy NRightShift into MM5 */
				mov edx, Kernel   	/* load Kernel address into EDX */
				mov esi, Src   	/* load Src  address to ESI */
				mov edi, Dest   	/* load Dest address to EDI */
				add edi, 2   	/* 2 column offset from the left edge */
				mov eax, columns   	/* load columns into EAX */
				shl eax, 1   	/* EAX = columns * 2 */
				add edi, eax   	/* 2 row offset from the top edge */
				shr eax, 1   	/* EAX = columns */
				mov ebx, rows   	/* initialize ROWS counter */
				sub ebx, 4   	/* do not use first 2 and last 2 rows */
				/* ---, */
L10370:
			mov ecx, eax   	/* initialize COLUMNS counter */
				sub ecx, 4   	/* do not use first 2 and last 2 columns */
				align 16                 	/* 16 byte alignment of the loop entry */
L10372:
			pxor mm7, mm7   	/* zero MM7 (accumulator) */
				movd mm6, esi   	/* save ESI in MM6 */
				/* --- 1 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm2, mm1   	/* copy MM1 into MM2 */
				add esi, eax   	/* move Src pointer 1 row below */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				movq mm4, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				punpckhbw mm2, mm0   	/* unpack second 4 bytes into words */
				psrlw mm1, mm5   	/* shift right each pixel NshiftRight times */
				psrlw mm2, mm5   	/* shift right each pixel NshiftRight times */
				pmullw mm1, mm3   	/* mult 4 low  words of Src and Kernel */
				pmullw mm2, mm4   	/* mult 4 high words of Src and Kernel */
				paddsw mm1, mm2   	/* add 4 words of the high and low bytes */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				/* --- 2 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm2, mm1   	/* copy MM1 into MM2 */
				add esi, eax   	/* move Src pointer 1 row below */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				movq mm4, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				punpckhbw mm2, mm0   	/* unpack second 4 bytes into words */
				psrlw mm1, mm5   	/* shift right each pixel NshiftRight times */
				psrlw mm2, mm5   	/* shift right each pixel NshiftRight times */
				pmullw mm1, mm3   	/* mult 4 low  words of Src and Kernel */
				pmullw mm2, mm4   	/* mult 4 high words of Src and Kernel */
				paddsw mm1, mm2   	/* add 4 words of the high and low bytes */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				/* --- 3 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm2, mm1   	/* copy MM1 into MM2 */
				add esi, eax   	/* move Src pointer 1 row below */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				movq mm4, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				punpckhbw mm2, mm0   	/* unpack second 4 bytes into words */
				psrlw mm1, mm5   	/* shift right each pixel NshiftRight times */
				psrlw mm2, mm5   	/* shift right each pixel NshiftRight times */
				pmullw mm1, mm3   	/* mult 4 low  words of Src and Kernel */
				pmullw mm2, mm4   	/* mult 4 high words of Src and Kernel */
				paddsw mm1, mm2   	/* add 4 words of the high and low bytes */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				/* --- 4 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm2, mm1   	/* copy MM1 into MM2 */
				add esi, eax   	/* move Src pointer 1 row below */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				movq mm4, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				punpckhbw mm2, mm0   	/* unpack second 4 bytes into words */
				psrlw mm1, mm5   	/* shift right each pixel NshiftRight times */
				psrlw mm2, mm5   	/* shift right each pixel NshiftRight times */
				pmullw mm1, mm3   	/* mult 4 low  words of Src and Kernel */
				pmullw mm2, mm4   	/* mult 4 high words of Src and Kernel */
				paddsw mm1, mm2   	/* add 4 words of the high and low bytes */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				/* --- 5 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm2, mm1   	/* copy MM1 into MM2 */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				movq mm4, [edx]   	/* load 4 words of Kernel */
			punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				punpckhbw mm2, mm0   	/* unpack second 4 bytes into words */
				psrlw mm1, mm5   	/* shift right each pixel NshiftRight times */
				psrlw mm2, mm5   	/* shift right each pixel NshiftRight times */
				pmullw mm1, mm3   	/* mult 4 low  words of Src and Kernel */
				pmullw mm2, mm4   	/* mult 4 high words of Src and Kernel */
				paddsw mm1, mm2   	/* add 4 words of the high and low bytes */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				/* ---, */
				movq mm3, mm7   	/* copy MM7 into MM3 */
				psrlq mm7, 32   	/* shift 2 left words to the right */
				paddsw mm7, mm3   	/* add 2 left and 2 right result words */
				movq mm2, mm7   	/* copy MM7 into MM2 */
				psrlq mm7, 16   	/* shift 1 left word to the right */
				paddsw mm7, mm2   	/* add 1 left and 1 right result words */
				movd mm1, eax   	/* save EAX in MM1 */
				packuswb mm7, mm0   	/* pack division result with saturation */
				movd eax, mm7   	/* copy saturated result into EAX */
				mov [edi], al   	/* copy a byte result into Dest */
				movd eax, mm1   	/* restore saved EAX */
				/* --, */
				movd esi, mm6   	/* move Src pointer to the top pixel */
				sub edx, 72   	/* EDX = Kernel address */
				inc              esi    	/* move Src  pointer to the next pixel */
				inc              edi    	/* move Dest pointer to the next pixel */
				/* ---, */
				dec              ecx    	/* decrease loop counter COLUMNS */
				jnz            L10372    	/* check loop termination, proceed if required */
				add esi, 4   	/* move to the next row in Src */
				add edi, 4   	/* move to the next row in Dest */
				dec              ebx    	/* decrease loop counter ROWS */
				jnz            L10370    	/* check loop termination, proceed if required */
				/* ---, */
				emms                      	/* exit MMX state */
				popa
		}
#else
		asm volatile
			("pusha		     \n\t" "pxor      %%mm0, %%mm0 \n\t"	/* zero MM0 */
			"xor       %%ebx, %%ebx \n\t"	/* zero EBX */
			"mov           %5, %%bl \n\t"	/* load NRightShift into BL */
			"movd      %%ebx, %%mm5 \n\t"	/* copy NRightShift into MM5 */
			"mov          %4, %%edx \n\t"	/* load Kernel address into EDX */
			"mov          %1, %%esi \n\t"	/* load Src  address to ESI */
			"mov          %0, %%edi \n\t"	/* load Dest address to EDI */
			"add          $2, %%edi \n\t"	/* 2 column offset from the left edge */
			"mov          %3, %%eax \n\t"	/* load columns into EAX */
			"shl          $1, %%eax \n\t"	/* EAX = columns * 2 */
			"add       %%eax, %%edi \n\t"	/* 2 row offset from the top edge */
			"shr          $1, %%eax \n\t"	/* EAX = columns */
			"mov          %2, %%ebx \n\t"	/* initialize ROWS counter */
			"sub          $4, %%ebx \n\t"	/* do not use first 2 and last 2 rows */
			/* --- */
			".L10370:               \n\t" "mov       %%eax, %%ecx \n\t"	/* initialize COLUMNS counter */
			"sub          $4, %%ecx \n\t"	/* do not use first 2 and last 2 columns */
			".align 16              \n\t"	/* 16 byte alignment of the loop entry */
			".L10372:               \n\t" "pxor      %%mm7, %%mm7 \n\t"	/* zero MM7 (accumulator) */
			"movd      %%esi, %%mm6 \n\t"	/* save ESI in MM6 */
			/* --- 1 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"add       %%eax, %%esi \n\t"	/* move Src pointer 1 row below */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"movq    (%%edx), %%mm4 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"punpckhbw %%mm0, %%mm2 \n\t"	/* unpack second 4 bytes into words */
			"psrlw     %%mm5, %%mm1 \n\t"	/* shift right each pixel NshiftRight times */
			"psrlw     %%mm5, %%mm2 \n\t"	/* shift right each pixel NshiftRight times */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"pmullw    %%mm4, %%mm2 \n\t"	/* mult. 4 high words of Src and Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the high and low bytes */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			/* --- 2 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"add       %%eax, %%esi \n\t"	/* move Src pointer 1 row below */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"movq    (%%edx), %%mm4 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"punpckhbw %%mm0, %%mm2 \n\t"	/* unpack second 4 bytes into words */
			"psrlw     %%mm5, %%mm1 \n\t"	/* shift right each pixel NshiftRight times */
			"psrlw     %%mm5, %%mm2 \n\t"	/* shift right each pixel NshiftRight times */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"pmullw    %%mm4, %%mm2 \n\t"	/* mult. 4 high words of Src and Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the high and low bytes */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			/* --- 3 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"add       %%eax, %%esi \n\t"	/* move Src pointer 1 row below */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"movq    (%%edx), %%mm4 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"punpckhbw %%mm0, %%mm2 \n\t"	/* unpack second 4 bytes into words */
			"psrlw     %%mm5, %%mm1 \n\t"	/* shift right each pixel NshiftRight times */
			"psrlw     %%mm5, %%mm2 \n\t"	/* shift right each pixel NshiftRight times */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"pmullw    %%mm4, %%mm2 \n\t"	/* mult. 4 high words of Src and Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the high and low bytes */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			/* --- 4 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"add       %%eax, %%esi \n\t"	/* move Src pointer 1 row below */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"movq    (%%edx), %%mm4 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"punpckhbw %%mm0, %%mm2 \n\t"	/* unpack second 4 bytes into words */
			"psrlw     %%mm5, %%mm1 \n\t"	/* shift right each pixel NshiftRight times */
			"psrlw     %%mm5, %%mm2 \n\t"	/* shift right each pixel NshiftRight times */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"pmullw    %%mm4, %%mm2 \n\t"	/* mult. 4 high words of Src and Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the high and low bytes */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			/* --- 5 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"movq    (%%edx), %%mm4 \n\t"	/* load 4 words of Kernel */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"punpckhbw %%mm0, %%mm2 \n\t"	/* unpack second 4 bytes into words */
			"psrlw     %%mm5, %%mm1 \n\t"	/* shift right each pixel NshiftRight times */
			"psrlw     %%mm5, %%mm2 \n\t"	/* shift right each pixel NshiftRight times */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"pmullw    %%mm4, %%mm2 \n\t"	/* mult. 4 high words of Src and Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the high and low bytes */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			/* --- */
			"movq      %%mm7, %%mm3 \n\t"	/* copy MM7 into MM3 */
			"psrlq       $32, %%mm7 \n\t"	/* shift 2 left words to the right */
			"paddsw    %%mm3, %%mm7 \n\t"	/* add 2 left and 2 right result words */
			"movq      %%mm7, %%mm2 \n\t"	/* copy MM7 into MM2 */
			"psrlq       $16, %%mm7 \n\t"	/* shift 1 left word to the right */
			"paddsw    %%mm2, %%mm7 \n\t"	/* add 1 left and 1 right result words */
			"movd      %%eax, %%mm1 \n\t"	/* save EAX in MM1 */
			"packuswb  %%mm0, %%mm7 \n\t"	/* pack division result with saturation */
			"movd      %%mm7, %%eax \n\t"	/* copy saturated result into EAX */
			"mov      %%al, (%%edi) \n\t"	/* copy a byte result into Dest */
			"movd      %%mm1, %%eax \n\t"	/* restore saved EAX */
			/* -- */
			"movd      %%mm6, %%esi \n\t"	/* move Src pointer to the top pixel */
			"sub         $72, %%edx \n\t"	/* EDX = Kernel address */
			"inc              %%esi \n\t"	/* move Src  pointer to the next pixel */
			"inc              %%edi \n\t"	/* move Dest pointer to the next pixel */
			/* --- */
			"dec              %%ecx \n\t"	/* decrease loop counter COLUMNS */
			"jnz            .L10372 \n\t"	/* check loop termination, proceed if required */
			"add          $4, %%esi \n\t"	/* move to the next row in Src */
			"add          $4, %%edi \n\t"	/* move to the next row in Dest */
			"dec              %%ebx \n\t"	/* decrease loop counter ROWS */
			"jnz            .L10370 \n\t"	/* check loop termination, proceed if required */
			/* --- */
			"emms                   \n\t"	/* exit MMX state */
			"popa                   \n\t":"=m" (Dest)	/* %0 */
			:"m"(Src),		/* %1 */
			"m"(rows),		/* %2 */
			"m"(columns),		/* %3 */
			"m"(Kernel),		/* %4 */
			"m"(NRightShift)	/* %5 */
			);
#endif
#endif
		return (0);
	} else {
		/* No non-MMX implementation yet */
		return (-1);
	}
}

/*!
\brief Filter using ConvolveKernel7x7ShiftRight: Dij = saturation0and255( ... ) 

\param Src The source 2D byte array to convolve. Should be different from destination.
\param Dest The destination 2D byte array to store the result in. Should be different from source.
\param rows Number of rows in source/destination array. Must be >6.
\param columns Number of columns in source/destination array. Must be >6.
\param Kernel The 2D convolution kernel of size 7x7.
\param NRightShift The number of right bit shifts to apply to the convolution sum. Must be <7.

Note: Non-MMX implementation not available for this function.

\return Returns 1 if filter was applied, 0 otherwise.
*/
int SDL_imageFilterConvolveKernel7x7ShiftRight(unsigned char *Src, unsigned char *Dest, int rows, int columns,
											   signed short *Kernel, unsigned char NRightShift)
{
	/* Validate input parameters */
	if ((Src == NULL) || (Dest == NULL) || (Kernel == NULL))
		return(-1);

	if ((columns < 7) || (rows < 7) || (NRightShift > 7))
		return (-1);

	if ((SDL_imageFilterMMXdetect())) {
//#ifdef USE_MMX
#if defined(USE_MMX) && defined(i386)
#if !defined(GCC__)
		__asm
		{
			pusha
				pxor mm0, mm0   	/* zero MM0 */
				xor ebx, ebx   	/* zero EBX */
				mov bl, NRightShift   	/* load NRightShift into BL */
				movd mm5, ebx   	/* copy NRightShift into MM5 */
				mov edx, Kernel   	/* load Kernel address into EDX */
				mov esi, Src   	/* load Src  address to ESI */
				mov edi, Dest   	/* load Dest address to EDI */
				add edi, 3   	/* 3 column offset from the left edge */
				mov eax, columns   	/* load columns into EAX */
				add edi, eax   	/* 3 row offset from the top edge */
				add edi, eax
				add edi, eax
				mov ebx, rows   	/* initialize ROWS counter */
				sub ebx, 6   	/* do not use first 3 and last 3 rows */
				/* ---, */
L10380:
			mov ecx, eax   	/* initialize COLUMNS counter */
				sub ecx, 6   	/* do not use first 3 and last 3 columns */
				align 16                 	/* 16 byte alignment of the loop entry */
L10382:
			pxor mm7, mm7   	/* zero MM7 (accumulator) */
				movd mm6, esi   	/* save ESI in MM6 */
				/* --- 1 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm2, mm1   	/* copy MM1 into MM2 */
				add esi, eax   	/* move Src pointer 1 row below */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				movq mm4, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				punpckhbw mm2, mm0   	/* unpack second 4 bytes into words */
				psrlw mm1, mm5   	/* shift right each pixel NshiftRight times */
				psrlw mm2, mm5   	/* shift right each pixel NshiftRight times */
				pmullw mm1, mm3   	/* mult 4 low  words of Src and Kernel */
				pmullw mm2, mm4   	/* mult 4 high words of Src and Kernel */
				paddsw mm1, mm2   	/* add 4 words of the high and low bytes */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				/* --- 2 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm2, mm1   	/* copy MM1 into MM2 */
				add esi, eax   	/* move Src pointer 1 row below */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				movq mm4, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				punpckhbw mm2, mm0   	/* unpack second 4 bytes into words */
				psrlw mm1, mm5   	/* shift right each pixel NshiftRight times */
				psrlw mm2, mm5   	/* shift right each pixel NshiftRight times */
				pmullw mm1, mm3   	/* mult 4 low  words of Src and Kernel */
				pmullw mm2, mm4   	/* mult 4 high words of Src and Kernel */
				paddsw mm1, mm2   	/* add 4 words of the high and low bytes */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				/* --- 3 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm2, mm1   	/* copy MM1 into MM2 */
				add esi, eax   	/* move Src pointer 1 row below */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				movq mm4, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				punpckhbw mm2, mm0   	/* unpack second 4 bytes into words */
				psrlw mm1, mm5   	/* shift right each pixel NshiftRight times */
				psrlw mm2, mm5   	/* shift right each pixel NshiftRight times */
				pmullw mm1, mm3   	/* mult 4 low  words of Src and Kernel */
				pmullw mm2, mm4   	/* mult 4 high words of Src and Kernel */
				paddsw mm1, mm2   	/* add 4 words of the high and low bytes */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				/* --- 4 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm2, mm1   	/* copy MM1 into MM2 */
				add esi, eax   	/* move Src pointer 1 row below */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				movq mm4, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				punpckhbw mm2, mm0   	/* unpack second 4 bytes into words */
				psrlw mm1, mm5   	/* shift right each pixel NshiftRight times */
				psrlw mm2, mm5   	/* shift right each pixel NshiftRight times */
				pmullw mm1, mm3   	/* mult 4 low  words of Src and Kernel */
				pmullw mm2, mm4   	/* mult 4 high words of Src and Kernel */
				paddsw mm1, mm2   	/* add 4 words of the high and low bytes */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				/* --- 5 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm2, mm1   	/* copy MM1 into MM2 */
				add esi, eax   	/* move Src pointer 1 row below */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				movq mm4, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				punpckhbw mm2, mm0   	/* unpack second 4 bytes into words */
				psrlw mm1, mm5   	/* shift right each pixel NshiftRight times */
				psrlw mm2, mm5   	/* shift right each pixel NshiftRight times */
				pmullw mm1, mm3   	/* mult 4 low  words of Src and Kernel */
				pmullw mm2, mm4   	/* mult 4 high words of Src and Kernel */
				paddsw mm1, mm2   	/* add 4 words of the high and low bytes */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				/* --- 6 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm2, mm1   	/* copy MM1 into MM2 */
				add esi, eax   	/* move Src pointer 1 row below */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				movq mm4, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				punpckhbw mm2, mm0   	/* unpack second 4 bytes into words */
				psrlw mm1, mm5   	/* shift right each pixel NshiftRight times */
				psrlw mm2, mm5   	/* shift right each pixel NshiftRight times */
				pmullw mm1, mm3   	/* mult 4 low  words of Src and Kernel */
				pmullw mm2, mm4   	/* mult 4 high words of Src and Kernel */
				paddsw mm1, mm2   	/* add 4 words of the high and low bytes */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				/* --- 7 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm2, mm1   	/* copy MM1 into MM2 */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				movq mm4, [edx]   	/* load 4 words of Kernel */
			punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				punpckhbw mm2, mm0   	/* unpack second 4 bytes into words */
				psrlw mm1, mm5   	/* shift right each pixel NshiftRight times */
				psrlw mm2, mm5   	/* shift right each pixel NshiftRight times */
				pmullw mm1, mm3   	/* mult 4 low  words of Src and Kernel */
				pmullw mm2, mm4   	/* mult 4 high words of Src and Kernel */
				paddsw mm1, mm2   	/* add 4 words of the high and low bytes */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				/* ---, */
				movq mm3, mm7   	/* copy MM7 into MM3 */
				psrlq mm7, 32   	/* shift 2 left words to the right */
				paddsw mm7, mm3   	/* add 2 left and 2 right result words */
				movq mm2, mm7   	/* copy MM7 into MM2 */
				psrlq mm7, 16   	/* shift 1 left word to the right */
				paddsw mm7, mm2   	/* add 1 left and 1 right result words */
				movd mm1, eax   	/* save EAX in MM1 */
				packuswb mm7, mm0   	/* pack division result with saturation */
				movd eax, mm7   	/* copy saturated result into EAX */
				mov [edi], al   	/* copy a byte result into Dest */
				movd eax, mm1   	/* restore saved EAX */
				/* --, */
				movd esi, mm6   	/* move Src pointer to the top pixel */
				sub edx, 104   	/* EDX = Kernel address */
				inc              esi    	/* move Src  pointer to the next pixel */
				inc              edi    	/* move Dest pointer to the next pixel */
				/* ---, */
				dec              ecx    	/* decrease loop counter COLUMNS */
				jnz            L10382    	/* check loop termination, proceed if required */
				add esi, 6   	/* move to the next row in Src */
				add edi, 6   	/* move to the next row in Dest */
				dec              ebx    	/* decrease loop counter ROWS */
				jnz            L10380    	/* check loop termination, proceed if required */
				/* ---, */
				emms                      	/* exit MMX state */
				popa
		}
#else
		asm volatile
			("pusha		     \n\t" "pxor      %%mm0, %%mm0 \n\t"	/* zero MM0 */
			"xor       %%ebx, %%ebx \n\t"	/* zero EBX */
			"mov           %5, %%bl \n\t"	/* load NRightShift into BL */
			"movd      %%ebx, %%mm5 \n\t"	/* copy NRightShift into MM5 */
			"mov          %4, %%edx \n\t"	/* load Kernel address into EDX */
			"mov          %1, %%esi \n\t"	/* load Src  address to ESI */
			"mov          %0, %%edi \n\t"	/* load Dest address to EDI */
			"add          $3, %%edi \n\t"	/* 3 column offset from the left edge */
			"mov          %3, %%eax \n\t"	/* load columns into EAX */
			"add       %%eax, %%edi \n\t"	/* 3 row offset from the top edge */
			"add       %%eax, %%edi \n\t" "add       %%eax, %%edi \n\t" "mov          %2, %%ebx \n\t"	/* initialize ROWS counter */
			"sub          $6, %%ebx \n\t"	/* do not use first 3 and last 3 rows */
			/* --- */
			".L10380:               \n\t" "mov       %%eax, %%ecx \n\t"	/* initialize COLUMNS counter */
			"sub          $6, %%ecx \n\t"	/* do not use first 3 and last 3 columns */
			".align 16              \n\t"	/* 16 byte alignment of the loop entry */
			".L10382:               \n\t" "pxor      %%mm7, %%mm7 \n\t"	/* zero MM7 (accumulator) */
			"movd      %%esi, %%mm6 \n\t"	/* save ESI in MM6 */
			/* --- 1 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"add       %%eax, %%esi \n\t"	/* move Src pointer 1 row below */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"movq    (%%edx), %%mm4 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"punpckhbw %%mm0, %%mm2 \n\t"	/* unpack second 4 bytes into words */
			"psrlw     %%mm5, %%mm1 \n\t"	/* shift right each pixel NshiftRight times */
			"psrlw     %%mm5, %%mm2 \n\t"	/* shift right each pixel NshiftRight times */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"pmullw    %%mm4, %%mm2 \n\t"	/* mult. 4 high words of Src and Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the high and low bytes */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			/* --- 2 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"add       %%eax, %%esi \n\t"	/* move Src pointer 1 row below */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"movq    (%%edx), %%mm4 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"punpckhbw %%mm0, %%mm2 \n\t"	/* unpack second 4 bytes into words */
			"psrlw     %%mm5, %%mm1 \n\t"	/* shift right each pixel NshiftRight times */
			"psrlw     %%mm5, %%mm2 \n\t"	/* shift right each pixel NshiftRight times */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"pmullw    %%mm4, %%mm2 \n\t"	/* mult. 4 high words of Src and Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the high and low bytes */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			/* --- 3 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"add       %%eax, %%esi \n\t"	/* move Src pointer 1 row below */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"movq    (%%edx), %%mm4 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"punpckhbw %%mm0, %%mm2 \n\t"	/* unpack second 4 bytes into words */
			"psrlw     %%mm5, %%mm1 \n\t"	/* shift right each pixel NshiftRight times */
			"psrlw     %%mm5, %%mm2 \n\t"	/* shift right each pixel NshiftRight times */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"pmullw    %%mm4, %%mm2 \n\t"	/* mult. 4 high words of Src and Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the high and low bytes */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			/* --- 4 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"add       %%eax, %%esi \n\t"	/* move Src pointer 1 row below */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"movq    (%%edx), %%mm4 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"punpckhbw %%mm0, %%mm2 \n\t"	/* unpack second 4 bytes into words */
			"psrlw     %%mm5, %%mm1 \n\t"	/* shift right each pixel NshiftRight times */
			"psrlw     %%mm5, %%mm2 \n\t"	/* shift right each pixel NshiftRight times */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"pmullw    %%mm4, %%mm2 \n\t"	/* mult. 4 high words of Src and Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the high and low bytes */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			/* --- 5 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"add       %%eax, %%esi \n\t"	/* move Src pointer 1 row below */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"movq    (%%edx), %%mm4 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"punpckhbw %%mm0, %%mm2 \n\t"	/* unpack second 4 bytes into words */
			"psrlw     %%mm5, %%mm1 \n\t"	/* shift right each pixel NshiftRight times */
			"psrlw     %%mm5, %%mm2 \n\t"	/* shift right each pixel NshiftRight times */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"pmullw    %%mm4, %%mm2 \n\t"	/* mult. 4 high words of Src and Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the high and low bytes */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			/* --- 6 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"add       %%eax, %%esi \n\t"	/* move Src pointer 1 row below */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"movq    (%%edx), %%mm4 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"punpckhbw %%mm0, %%mm2 \n\t"	/* unpack second 4 bytes into words */
			"psrlw     %%mm5, %%mm1 \n\t"	/* shift right each pixel NshiftRight times */
			"psrlw     %%mm5, %%mm2 \n\t"	/* shift right each pixel NshiftRight times */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"pmullw    %%mm4, %%mm2 \n\t"	/* mult. 4 high words of Src and Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the high and low bytes */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			/* --- 7 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"movq    (%%edx), %%mm4 \n\t"	/* load 4 words of Kernel */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"punpckhbw %%mm0, %%mm2 \n\t"	/* unpack second 4 bytes into words */
			"psrlw     %%mm5, %%mm1 \n\t"	/* shift right each pixel NshiftRight times */
			"psrlw     %%mm5, %%mm2 \n\t"	/* shift right each pixel NshiftRight times */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"pmullw    %%mm4, %%mm2 \n\t"	/* mult. 4 high words of Src and Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the high and low bytes */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			/* --- */
			"movq      %%mm7, %%mm3 \n\t"	/* copy MM7 into MM3 */
			"psrlq       $32, %%mm7 \n\t"	/* shift 2 left words to the right */
			"paddsw    %%mm3, %%mm7 \n\t"	/* add 2 left and 2 right result words */
			"movq      %%mm7, %%mm2 \n\t"	/* copy MM7 into MM2 */
			"psrlq       $16, %%mm7 \n\t"	/* shift 1 left word to the right */
			"paddsw    %%mm2, %%mm7 \n\t"	/* add 1 left and 1 right result words */
			"movd      %%eax, %%mm1 \n\t"	/* save EAX in MM1 */
			"packuswb  %%mm0, %%mm7 \n\t"	/* pack division result with saturation */
			"movd      %%mm7, %%eax \n\t"	/* copy saturated result into EAX */
			"mov      %%al, (%%edi) \n\t"	/* copy a byte result into Dest */
			"movd      %%mm1, %%eax \n\t"	/* restore saved EAX */
			/* -- */
			"movd      %%mm6, %%esi \n\t"	/* move Src pointer to the top pixel */
			"sub        $104, %%edx \n\t"	/* EDX = Kernel address */
			"inc              %%esi \n\t"	/* move Src  pointer to the next pixel */
			"inc              %%edi \n\t"	/* move Dest pointer to the next pixel */
			/* --- */
			"dec              %%ecx \n\t"	/* decrease loop counter COLUMNS */
			"jnz            .L10382 \n\t"	/* check loop termination, proceed if required */
			"add          $6, %%esi \n\t"	/* move to the next row in Src */
			"add          $6, %%edi \n\t"	/* move to the next row in Dest */
			"dec              %%ebx \n\t"	/* decrease loop counter ROWS */
			"jnz            .L10380 \n\t"	/* check loop termination, proceed if required */
			/* --- */
			"emms                   \n\t"	/* exit MMX state */
			"popa                   \n\t":"=m" (Dest)	/* %0 */
			:"m"(Src),		/* %1 */
			"m"(rows),		/* %2 */
			"m"(columns),		/* %3 */
			"m"(Kernel),		/* %4 */
			"m"(NRightShift)	/* %5 */
			);
#endif
#endif
		return (0);
	} else {
		/* No non-MMX implementation yet */
		return (-1);
	}
}

/*!
\brief Filter using ConvolveKernel9x9ShiftRight: Dij = saturation255( ... ) 

\param Src The source 2D byte array to convolve. Should be different from destination.
\param Dest The destination 2D byte array to store the result in. Should be different from source.
\param rows Number of rows in source/destination array. Must be >8.
\param columns Number of columns in source/destination array. Must be >8.
\param Kernel The 2D convolution kernel of size 9x9.
\param NRightShift The number of right bit shifts to apply to the convolution sum. Must be <7.

Note: Non-MMX implementation not available for this function.

\return Returns 1 if filter was applied, 0 otherwise.
*/
int SDL_imageFilterConvolveKernel9x9ShiftRight(unsigned char *Src, unsigned char *Dest, int rows, int columns,
											   signed short *Kernel, unsigned char NRightShift)
{
	/* Validate input parameters */
	if ((Src == NULL) || (Dest == NULL) || (Kernel == NULL))
		return(-1);

	if ((columns < 9) || (rows < 9) || (NRightShift > 7))
		return (-1);

	if ((SDL_imageFilterMMXdetect())) {
//#ifdef USE_MMX
#if defined(USE_MMX) && defined(i386)
#if !defined(GCC__)
		__asm
		{
			pusha
				pxor mm0, mm0   	/* zero MM0 */
				xor ebx, ebx   	/* zero EBX */
				mov bl, NRightShift   	/* load NRightShift into BL */
				movd mm5, ebx   	/* copy NRightShift into MM5 */
				mov edx, Kernel   	/* load Kernel address into EDX */
				mov esi, Src   	/* load Src  address to ESI */
				mov edi, Dest   	/* load Dest address to EDI */
				add edi, 4   	/* 4 column offset from the left edge */
				mov eax, columns   	/* load columns into EAX */
				add edi, eax   	/* 4 row offset from the top edge */
				add edi, eax
				add edi, eax
				add edi, eax
				mov ebx, rows   	/* initialize ROWS counter */
				sub ebx, 8   	/* do not use first 4 and last 4 rows */
				/* ---, */
L10390:
			mov ecx, eax   	/* initialize COLUMNS counter */
				sub ecx, 8   	/* do not use first 4 and last 4 columns */
				align 16                 	/* 16 byte alignment of the loop entry */
L10392:
			pxor mm7, mm7   	/* zero MM7 (accumulator) */
				movd mm6, esi   	/* save ESI in MM6 */
				/* --- 1 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm2, mm1   	/* copy MM1 into MM2 */
				inc              esi    	/* move pointer to the next 8 bytes of Src */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				movq mm4, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				punpckhbw mm2, mm0   	/* unpack second 4 bytes into words */
				psrlw mm1, mm5   	/* shift right each pixel NshiftRight times */
				psrlw mm2, mm5   	/* shift right each pixel NshiftRight times */
				pmullw mm1, mm3   	/* mult 4 low  words of Src and Kernel */
				pmullw mm2, mm4   	/* mult 4 high words of Src and Kernel */
				paddsw mm1, mm2   	/* add 4 words of the high and low bytes */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			dec              esi
				add esi, eax   	/* move Src pointer 1 row below */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				psrlw mm1, mm5   	/* shift right each pixel NshiftRight times */
				pmullw mm1, mm3   	/* mult 4 low  words of Src and Kernel */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				/* --- 2 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm2, mm1   	/* copy MM1 into MM2 */
				inc              esi    	/* move pointer to the next 8 bytes of Src */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				movq mm4, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				punpckhbw mm2, mm0   	/* unpack second 4 bytes into words */
				psrlw mm1, mm5   	/* shift right each pixel NshiftRight times */
				psrlw mm2, mm5   	/* shift right each pixel NshiftRight times */
				pmullw mm1, mm3   	/* mult 4 low  words of Src and Kernel */
				pmullw mm2, mm4   	/* mult 4 high words of Src and Kernel */
				paddsw mm1, mm2   	/* add 4 words of the high and low bytes */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			dec              esi
				add esi, eax   	/* move Src pointer 1 row below */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				psrlw mm1, mm5   	/* shift right each pixel NshiftRight times */
				pmullw mm1, mm3   	/* mult 4 low  words of Src and Kernel */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				/* --- 3 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm2, mm1   	/* copy MM1 into MM2 */
				inc              esi    	/* move pointer to the next 8 bytes of Src */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				movq mm4, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				punpckhbw mm2, mm0   	/* unpack second 4 bytes into words */
				psrlw mm1, mm5   	/* shift right each pixel NshiftRight times */
				psrlw mm2, mm5   	/* shift right each pixel NshiftRight times */
				pmullw mm1, mm3   	/* mult 4 low  words of Src and Kernel */
				pmullw mm2, mm4   	/* mult 4 high words of Src and Kernel */
				paddsw mm1, mm2   	/* add 4 words of the high and low bytes */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			dec              esi
				add esi, eax   	/* move Src pointer 1 row below */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				psrlw mm1, mm5   	/* shift right each pixel NshiftRight times */
				pmullw mm1, mm3   	/* mult 4 low  words of Src and Kernel */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				/* --- 4 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm2, mm1   	/* copy MM1 into MM2 */
				inc              esi    	/* move pointer to the next 8 bytes of Src */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				movq mm4, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				punpckhbw mm2, mm0   	/* unpack second 4 bytes into words */
				psrlw mm1, mm5   	/* shift right each pixel NshiftRight times */
				psrlw mm2, mm5   	/* shift right each pixel NshiftRight times */
				pmullw mm1, mm3   	/* mult 4 low  words of Src and Kernel */
				pmullw mm2, mm4   	/* mult 4 high words of Src and Kernel */
				paddsw mm1, mm2   	/* add 4 words of the high and low bytes */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			dec              esi
				add esi, eax   	/* move Src pointer 1 row below */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				psrlw mm1, mm5   	/* shift right each pixel NshiftRight times */
				pmullw mm1, mm3   	/* mult 4 low  words of Src and Kernel */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				/* --- 5 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm2, mm1   	/* copy MM1 into MM2 */
				inc              esi    	/* move pointer to the next 8 bytes of Src */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				movq mm4, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				punpckhbw mm2, mm0   	/* unpack second 4 bytes into words */
				psrlw mm1, mm5   	/* shift right each pixel NshiftRight times */
				psrlw mm2, mm5   	/* shift right each pixel NshiftRight times */
				pmullw mm1, mm3   	/* mult 4 low  words of Src and Kernel */
				pmullw mm2, mm4   	/* mult 4 high words of Src and Kernel */
				paddsw mm1, mm2   	/* add 4 words of the high and low bytes */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			dec              esi
				add esi, eax   	/* move Src pointer 1 row below */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				psrlw mm1, mm5   	/* shift right each pixel NshiftRight times */
				pmullw mm1, mm3   	/* mult 4 low  words of Src and Kernel */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				/* --- 6 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm2, mm1   	/* copy MM1 into MM2 */
				inc              esi    	/* move pointer to the next 8 bytes of Src */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				movq mm4, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				punpckhbw mm2, mm0   	/* unpack second 4 bytes into words */
				psrlw mm1, mm5   	/* shift right each pixel NshiftRight times */
				psrlw mm2, mm5   	/* shift right each pixel NshiftRight times */
				pmullw mm1, mm3   	/* mult 4 low  words of Src and Kernel */
				pmullw mm2, mm4   	/* mult 4 high words of Src and Kernel */
				paddsw mm1, mm2   	/* add 4 words of the high and low bytes */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			dec              esi
				add esi, eax   	/* move Src pointer 1 row below */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				psrlw mm1, mm5   	/* shift right each pixel NshiftRight times */
				pmullw mm1, mm3   	/* mult 4 low  words of Src and Kernel */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				/* --- 7 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm2, mm1   	/* copy MM1 into MM2 */
				inc              esi    	/* move pointer to the next 8 bytes of Src */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				movq mm4, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				punpckhbw mm2, mm0   	/* unpack second 4 bytes into words */
				psrlw mm1, mm5   	/* shift right each pixel NshiftRight times */
				psrlw mm2, mm5   	/* shift right each pixel NshiftRight times */
				pmullw mm1, mm3   	/* mult 4 low  words of Src and Kernel */
				pmullw mm2, mm4   	/* mult 4 high words of Src and Kernel */
				paddsw mm1, mm2   	/* add 4 words of the high and low bytes */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			dec              esi
				add esi, eax   	/* move Src pointer 1 row below */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				psrlw mm1, mm5   	/* shift right each pixel NshiftRight times */
				pmullw mm1, mm3   	/* mult 4 low  words of Src and Kernel */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				/* --- 8 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm2, mm1   	/* copy MM1 into MM2 */
				inc              esi    	/* move pointer to the next 8 bytes of Src */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				movq mm4, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				punpckhbw mm2, mm0   	/* unpack second 4 bytes into words */
				psrlw mm1, mm5   	/* shift right each pixel NshiftRight times */
				psrlw mm2, mm5   	/* shift right each pixel NshiftRight times */
				pmullw mm1, mm3   	/* mult 4 low  words of Src and Kernel */
				pmullw mm2, mm4   	/* mult 4 high words of Src and Kernel */
				paddsw mm1, mm2   	/* add 4 words of the high and low bytes */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			dec              esi
				add esi, eax   	/* move Src pointer 1 row below */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				psrlw mm1, mm5   	/* shift right each pixel NshiftRight times */
				pmullw mm1, mm3   	/* mult 4 low  words of Src and Kernel */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				/* --- 9 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm2, mm1   	/* copy MM1 into MM2 */
				inc              esi    	/* move pointer to the next 8 bytes of Src */
				movq mm3, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				movq mm4, [edx]   	/* load 4 words of Kernel */
			add edx, 8   	/* move pointer to other 4 words */
				punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				punpckhbw mm2, mm0   	/* unpack second 4 bytes into words */
				psrlw mm1, mm5   	/* shift right each pixel NshiftRight times */
				psrlw mm2, mm5   	/* shift right each pixel NshiftRight times */
				pmullw mm1, mm3   	/* mult 4 low  words of Src and Kernel */
				pmullw mm2, mm4   	/* mult 4 high words of Src and Kernel */
				paddsw mm1, mm2   	/* add 4 words of the high and low bytes */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				movq mm1, [esi]   	/* load 8 bytes of the Src */
			movq mm3, [edx]   	/* load 4 words of Kernel */
			punpcklbw mm1, mm0   	/* unpack first  4 bytes into words */
				psrlw mm1, mm5   	/* shift right each pixel NshiftRight times */
				pmullw mm1, mm3   	/* mult 4 low  words of Src and Kernel */
				paddsw mm7, mm1   	/* add MM1 to accumulator MM7 */
				/* ---, */
				movq mm3, mm7   	/* copy MM7 into MM3 */
				psrlq mm7, 32   	/* shift 2 left words to the right */
				paddsw mm7, mm3   	/* add 2 left and 2 right result words */
				movq mm2, mm7   	/* copy MM7 into MM2 */
				psrlq mm7, 16   	/* shift 1 left word to the right */
				paddsw mm7, mm2   	/* add 1 left and 1 right result words */
				movd mm1, eax   	/* save EAX in MM1 */
				packuswb mm7, mm0   	/* pack division result with saturation */
				movd eax, mm7   	/* copy saturated result into EAX */
				mov [edi], al   	/* copy a byte result into Dest */
				movd eax, mm1   	/* restore saved EAX */
				/* --, */
				movd esi, mm6   	/* move Src pointer to the top pixel */
				sub edx, 208   	/* EDX = Kernel address */
				inc              esi    	/* move Src  pointer to the next pixel */
				inc              edi    	/* move Dest pointer to the next pixel */
				/* ---, */
				dec              ecx    	/* decrease loop counter COLUMNS */
				jnz            L10392    	/* check loop termination, proceed if required */
				add esi, 8   	/* move to the next row in Src */
				add edi, 8   	/* move to the next row in Dest */
				dec              ebx    	/* decrease loop counter ROWS */
				jnz            L10390    	/* check loop termination, proceed if required */
				/* ---, */
				emms                      	/* exit MMX state */
				popa
		}
#else
		asm volatile
			("pusha		     \n\t" "pxor      %%mm0, %%mm0 \n\t"	/* zero MM0 */
			"xor       %%ebx, %%ebx \n\t"	/* zero EBX */
			"mov           %5, %%bl \n\t"	/* load NRightShift into BL */
			"movd      %%ebx, %%mm5 \n\t"	/* copy NRightShift into MM5 */
			"mov          %4, %%edx \n\t"	/* load Kernel address into EDX */
			"mov          %1, %%esi \n\t"	/* load Src  address to ESI */
			"mov          %0, %%edi \n\t"	/* load Dest address to EDI */
			"add          $4, %%edi \n\t"	/* 4 column offset from the left edge */
			"mov          %3, %%eax \n\t"	/* load columns into EAX */
			"add       %%eax, %%edi \n\t"	/* 4 row offset from the top edge */
			"add       %%eax, %%edi \n\t" "add       %%eax, %%edi \n\t" "add       %%eax, %%edi \n\t" "mov          %2, %%ebx \n\t"	/* initialize ROWS counter */
			"sub          $8, %%ebx \n\t"	/* do not use first 4 and last 4 rows */
			/* --- */
			".L10390:               \n\t" "mov       %%eax, %%ecx \n\t"	/* initialize COLUMNS counter */
			"sub          $8, %%ecx \n\t"	/* do not use first 4 and last 4 columns */
			".align 16              \n\t"	/* 16 byte alignment of the loop entry */
			".L10392:               \n\t" "pxor      %%mm7, %%mm7 \n\t"	/* zero MM7 (accumulator) */
			"movd      %%esi, %%mm6 \n\t"	/* save ESI in MM6 */
			/* --- 1 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"inc              %%esi \n\t"	/* move pointer to the next 8 bytes of Src */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"movq    (%%edx), %%mm4 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"punpckhbw %%mm0, %%mm2 \n\t"	/* unpack second 4 bytes into words */
			"psrlw     %%mm5, %%mm1 \n\t"	/* shift right each pixel NshiftRight times */
			"psrlw     %%mm5, %%mm2 \n\t"	/* shift right each pixel NshiftRight times */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"pmullw    %%mm4, %%mm2 \n\t"	/* mult. 4 high words of Src and Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the high and low bytes */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"dec              %%esi \n\t" "add       %%eax, %%esi \n\t"	/* move Src pointer 1 row below */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"psrlw     %%mm5, %%mm1 \n\t"	/* shift right each pixel NshiftRight times */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			/* --- 2 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"inc              %%esi \n\t"	/* move pointer to the next 8 bytes of Src */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"movq    (%%edx), %%mm4 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"punpckhbw %%mm0, %%mm2 \n\t"	/* unpack second 4 bytes into words */
			"psrlw     %%mm5, %%mm1 \n\t"	/* shift right each pixel NshiftRight times */
			"psrlw     %%mm5, %%mm2 \n\t"	/* shift right each pixel NshiftRight times */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"pmullw    %%mm4, %%mm2 \n\t"	/* mult. 4 high words of Src and Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the high and low bytes */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"dec              %%esi \n\t" "add       %%eax, %%esi \n\t"	/* move Src pointer 1 row below */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"psrlw     %%mm5, %%mm1 \n\t"	/* shift right each pixel NshiftRight times */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			/* --- 3 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"inc              %%esi \n\t"	/* move pointer to the next 8 bytes of Src */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"movq    (%%edx), %%mm4 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"punpckhbw %%mm0, %%mm2 \n\t"	/* unpack second 4 bytes into words */
			"psrlw     %%mm5, %%mm1 \n\t"	/* shift right each pixel NshiftRight times */
			"psrlw     %%mm5, %%mm2 \n\t"	/* shift right each pixel NshiftRight times */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"pmullw    %%mm4, %%mm2 \n\t"	/* mult. 4 high words of Src and Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the high and low bytes */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"dec              %%esi \n\t" "add       %%eax, %%esi \n\t"	/* move Src pointer 1 row below */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"psrlw     %%mm5, %%mm1 \n\t"	/* shift right each pixel NshiftRight times */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			/* --- 4 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"inc              %%esi \n\t"	/* move pointer to the next 8 bytes of Src */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"movq    (%%edx), %%mm4 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"punpckhbw %%mm0, %%mm2 \n\t"	/* unpack second 4 bytes into words */
			"psrlw     %%mm5, %%mm1 \n\t"	/* shift right each pixel NshiftRight times */
			"psrlw     %%mm5, %%mm2 \n\t"	/* shift right each pixel NshiftRight times */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"pmullw    %%mm4, %%mm2 \n\t"	/* mult. 4 high words of Src and Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the high and low bytes */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"dec              %%esi \n\t" "add       %%eax, %%esi \n\t"	/* move Src pointer 1 row below */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"psrlw     %%mm5, %%mm1 \n\t"	/* shift right each pixel NshiftRight times */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			/* --- 5 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"inc              %%esi \n\t"	/* move pointer to the next 8 bytes of Src */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"movq    (%%edx), %%mm4 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"punpckhbw %%mm0, %%mm2 \n\t"	/* unpack second 4 bytes into words */
			"psrlw     %%mm5, %%mm1 \n\t"	/* shift right each pixel NshiftRight times */
			"psrlw     %%mm5, %%mm2 \n\t"	/* shift right each pixel NshiftRight times */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"pmullw    %%mm4, %%mm2 \n\t"	/* mult. 4 high words of Src and Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the high and low bytes */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"dec              %%esi \n\t" "add       %%eax, %%esi \n\t"	/* move Src pointer 1 row below */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"psrlw     %%mm5, %%mm1 \n\t"	/* shift right each pixel NshiftRight times */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			/* --- 6 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"inc              %%esi \n\t"	/* move pointer to the next 8 bytes of Src */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"movq    (%%edx), %%mm4 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"punpckhbw %%mm0, %%mm2 \n\t"	/* unpack second 4 bytes into words */
			"psrlw     %%mm5, %%mm1 \n\t"	/* shift right each pixel NshiftRight times */
			"psrlw     %%mm5, %%mm2 \n\t"	/* shift right each pixel NshiftRight times */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"pmullw    %%mm4, %%mm2 \n\t"	/* mult. 4 high words of Src and Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the high and low bytes */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"dec              %%esi \n\t" "add       %%eax, %%esi \n\t"	/* move Src pointer 1 row below */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"psrlw     %%mm5, %%mm1 \n\t"	/* shift right each pixel NshiftRight times */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			/* --- 7 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"inc              %%esi \n\t"	/* move pointer to the next 8 bytes of Src */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"movq    (%%edx), %%mm4 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"punpckhbw %%mm0, %%mm2 \n\t"	/* unpack second 4 bytes into words */
			"psrlw     %%mm5, %%mm1 \n\t"	/* shift right each pixel NshiftRight times */
			"psrlw     %%mm5, %%mm2 \n\t"	/* shift right each pixel NshiftRight times */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"pmullw    %%mm4, %%mm2 \n\t"	/* mult. 4 high words of Src and Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the high and low bytes */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"dec              %%esi \n\t" "add       %%eax, %%esi \n\t"	/* move Src pointer 1 row below */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"psrlw     %%mm5, %%mm1 \n\t"	/* shift right each pixel NshiftRight times */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			/* --- 8 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"inc              %%esi \n\t"	/* move pointer to the next 8 bytes of Src */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"movq    (%%edx), %%mm4 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"punpckhbw %%mm0, %%mm2 \n\t"	/* unpack second 4 bytes into words */
			"psrlw     %%mm5, %%mm1 \n\t"	/* shift right each pixel NshiftRight times */
			"psrlw     %%mm5, %%mm2 \n\t"	/* shift right each pixel NshiftRight times */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"pmullw    %%mm4, %%mm2 \n\t"	/* mult. 4 high words of Src and Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the high and low bytes */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"dec              %%esi \n\t" "add       %%eax, %%esi \n\t"	/* move Src pointer 1 row below */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"psrlw     %%mm5, %%mm1 \n\t"	/* shift right each pixel NshiftRight times */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			/* --- 9 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq      %%mm1, %%mm2 \n\t"	/* copy MM1 into MM2 */
			"inc              %%esi \n\t"	/* move pointer to the next 8 bytes of Src */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"movq    (%%edx), %%mm4 \n\t"	/* load 4 words of Kernel */
			"add          $8, %%edx \n\t"	/* move pointer to other 4 words */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"punpckhbw %%mm0, %%mm2 \n\t"	/* unpack second 4 bytes into words */
			"psrlw     %%mm5, %%mm1 \n\t"	/* shift right each pixel NshiftRight times */
			"psrlw     %%mm5, %%mm2 \n\t"	/* shift right each pixel NshiftRight times */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"pmullw    %%mm4, %%mm2 \n\t"	/* mult. 4 high words of Src and Kernel */
			"paddsw    %%mm2, %%mm1 \n\t"	/* add 4 words of the high and low bytes */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			"movq    (%%esi), %%mm1 \n\t"	/* load 8 bytes of the Src */
			"movq    (%%edx), %%mm3 \n\t"	/* load 4 words of Kernel */
			"punpcklbw %%mm0, %%mm1 \n\t"	/* unpack first  4 bytes into words */
			"psrlw     %%mm5, %%mm1 \n\t"	/* shift right each pixel NshiftRight times */
			"pmullw    %%mm3, %%mm1 \n\t"	/* mult. 4 low  words of Src and Kernel */
			"paddsw    %%mm1, %%mm7 \n\t"	/* add MM1 to accumulator MM7 */
			/* --- */
			"movq      %%mm7, %%mm3 \n\t"	/* copy MM7 into MM3 */
			"psrlq       $32, %%mm7 \n\t"	/* shift 2 left words to the right */
			"paddsw    %%mm3, %%mm7 \n\t"	/* add 2 left and 2 right result words */
			"movq      %%mm7, %%mm2 \n\t"	/* copy MM7 into MM2 */
			"psrlq       $16, %%mm7 \n\t"	/* shift 1 left word to the right */
			"paddsw    %%mm2, %%mm7 \n\t"	/* add 1 left and 1 right result words */
			"movd      %%eax, %%mm1 \n\t"	/* save EAX in MM1 */
			"packuswb  %%mm0, %%mm7 \n\t"	/* pack division result with saturation */
			"movd      %%mm7, %%eax \n\t"	/* copy saturated result into EAX */
			"mov      %%al, (%%edi) \n\t"	/* copy a byte result into Dest */
			"movd      %%mm1, %%eax \n\t"	/* restore saved EAX */
			/* -- */
			"movd      %%mm6, %%esi \n\t"	/* move Src pointer to the top pixel */
			"sub        $208, %%edx \n\t"	/* EDX = Kernel address */
			"inc              %%esi \n\t"	/* move Src  pointer to the next pixel */
			"inc              %%edi \n\t"	/* move Dest pointer to the next pixel */
			/* --- */
			"dec              %%ecx \n\t"	/* decrease loop counter COLUMNS */
			"jnz            .L10392 \n\t"	/* check loop termination, proceed if required */
			"add          $8, %%esi \n\t"	/* move to the next row in Src */
			"add          $8, %%edi \n\t"	/* move to the next row in Dest */
			"dec              %%ebx \n\t"	/* decrease loop counter ROWS */
			"jnz            .L10390 \n\t"	/* check loop termination, proceed if required */
			/* --- */
			"emms                   \n\t"	/* exit MMX state */
			"popa                   \n\t":"=m" (Dest)	/* %0 */
			:"m"(Src),		/* %1 */
			"m"(rows),		/* %2 */
			"m"(columns),		/* %3 */
			"m"(Kernel),		/* %4 */
			"m"(NRightShift)	/* %5 */
			);
#endif
#endif
		return (0);
	} else {
		/* No non-MMX implementation yet */
		return (-1);
	}
}

/* ------------------------------------------------------------------------------------ */

/*!
\brief Filter using SobelX: Dij = saturation255( ... ) 

\param Src The source 2D byte array to sobel-filter. Should be different from destination.
\param Dest The destination 2D byte array to store the result in. Should be different from source.
\param rows Number of rows in source/destination array. Must be >2.
\param columns Number of columns in source/destination array. Must be >7.

Note: Non-MMX implementation not available for this function.

\return Returns 1 if filter was applied, 0 otherwise.
*/
int SDL_imageFilterSobelX(unsigned char *Src, unsigned char *Dest, int rows, int columns)
{
	/* Validate input parameters */
	if ((Src == NULL) || (Dest == NULL))
		return(-1);

	if ((columns < 8) || (rows < 3))
		return (-1);

	if ((SDL_imageFilterMMXdetect())) {
//#ifdef USE_MMX
#if defined(USE_MMX) && defined(i386)
#if !defined(GCC__)
		__asm
		{
			pusha
				pxor mm0, mm0   	/* zero MM0 */
				mov eax, columns   	/* load columns into EAX */
				/* ---, */
				mov esi, Src   	/* ESI = Src row 0 address */
				mov edi, Dest   	/* load Dest address to EDI */
				add edi, eax   	/* EDI = EDI + columns */
				inc              edi    	/* 1 byte offset from the left edge */
				mov edx, rows   	/* initialize ROWS counter */
				sub edx, 2   	/* do not use first and last rows */
				/* ---, */
L10400:
			mov ecx, eax   	/* initialize COLUMS counter */
				shr ecx, 3   	/* EBX/8 (MMX loads 8 bytes at a time) */
				mov ebx, esi   	/* save ESI in EBX */
				movd mm1, edi   	/* save EDI in MM1 */
				align 16                 	/* 16 byte alignment of the loop entry */
L10402:
			/* ---, */
			movq mm4, [esi]   	/* load 8 bytes from Src */
			movq mm5, mm4   	/* save MM4 in MM5 */
				add esi, 2   	/* move ESI pointer 2 bytes right */
				punpcklbw mm4, mm0   	/* unpack 4 low  bytes into words */
				punpckhbw mm5, mm0   	/* unpack 4 high bytes into words */
				movq mm6, [esi]   	/* load 8 bytes from Src */
			movq mm7, mm6   	/* save MM6 in MM7 */
				sub esi, 2   	/* move ESI pointer back 2 bytes left */
				punpcklbw mm6, mm0   	/* unpack 4 low  bytes into words */
				punpckhbw mm7, mm0   	/* unpack 4 high bytes into words */
				add esi, eax   	/* move to the next row of Src */
				movq mm2, [esi]   	/* load 8 bytes from Src */
			movq mm3, mm2   	/* save MM2 in MM3 */
				add esi, 2   	/* move ESI pointer 2 bytes right */
				punpcklbw mm2, mm0   	/* unpack 4 low  bytes into words */
				punpckhbw mm3, mm0   	/* unpack 4 high bytes into words */
				paddw mm4, mm2   	/* add 4 low  bytes to accumolator MM4 */
				paddw mm5, mm3   	/* add 4 high bytes to accumolator MM5 */
				paddw mm4, mm2   	/* add 4 low  bytes to accumolator MM4 */
				paddw mm5, mm3   	/* add 4 high bytes to accumolator MM5 */
				movq mm2, [esi]   	/* load 8 bytes from Src */
			movq mm3, mm2   	/* save MM2 in MM3 */
				sub esi, 2   	/* move ESI pointer back 2 bytes left */
				punpcklbw mm2, mm0   	/* unpack 4 low  bytes into words */
				punpckhbw mm3, mm0   	/* unpack 4 high bytes into words */
				paddw mm6, mm2   	/* add 4 low  bytes to accumolator MM6 */
				paddw mm7, mm3   	/* add 4 high bytes to accumolator MM7 */
				paddw mm6, mm2   	/* add 4 low  bytes to accumolator MM6 */
				paddw mm7, mm3   	/* add 4 high bytes to accumolator MM7 */
				add esi, eax   	/* move to the next row of Src */
				movq mm2, [esi]   	/* load 8 bytes from Src */
			movq mm3, mm2   	/* save MM2 in MM3 */
				add esi, 2   	/* move ESI pointer 2 bytes right */
				punpcklbw mm2, mm0   	/* unpack 4 low  bytes into words */
				punpckhbw mm3, mm0   	/* unpack 4 high bytes into words */
				paddw mm4, mm2   	/* add 4 low  bytes to accumolator MM4 */
				paddw mm5, mm3   	/* add 4 high bytes to accumolator MM5 */
				movq mm2, [esi]   	/* load 8 bytes from Src */
			movq mm3, mm2   	/* save MM2 in MM3 */
				sub esi, 2   	/* move ESI pointer back 2 bytes left */
				punpcklbw mm2, mm0   	/* unpack 4 low  bytes into words */
				punpckhbw mm3, mm0   	/* unpack 4 high bytes into words */
				paddw mm6, mm2   	/* add 4 low  bytes to accumolator MM6 */
				paddw mm7, mm3   	/* add 4 high bytes to accumolator MM7 */
				/* ---, */
				movq mm2, mm4   	/* copy MM4 into MM2 */
				psrlq mm4, 32   	/* shift 2 left words to the right */
				psubw mm4, mm2   	/* MM4 = MM4 - MM2 */
				movq mm3, mm6   	/* copy MM6 into MM3 */
				psrlq mm6, 32   	/* shift 2 left words to the right */
				psubw mm6, mm3   	/* MM6 = MM6 - MM3 */
				punpckldq mm4, mm6   	/* combine 2 words of MM6 and 2 words of MM4 */
				movq mm2, mm5   	/* copy MM6 into MM2 */
				psrlq mm5, 32   	/* shift 2 left words to the right */
				psubw mm5, mm2   	/* MM5 = MM5 - MM2 */
				movq mm3, mm7   	/* copy MM7 into MM3 */
				psrlq mm7, 32   	/* shift 2 left words to the right */
				psubw mm7, mm3   	/* MM7 = MM7 - MM3 */
				punpckldq mm5, mm7   	/* combine 2 words of MM7 and 2 words of MM5 */
				/* Take abs values of MM4 and MM5 */
				movq mm6, mm4   	/* copy MM4 into MM6 */
				movq mm7, mm5   	/* copy MM5 into MM7 */
				psraw mm6, 15   	/* fill MM6 words with word sign bit */
				psraw mm7, 15   	/* fill MM7 words with word sign bit */
				pxor mm4, mm6   	/* take 1's compliment of only neg words */
				pxor mm5, mm7   	/* take 1's compliment of only neg words */
				psubsw mm4, mm6   	/* add 1 to only neg words, W-(-1) or W-0 */
				psubsw mm5, mm7   	/* add 1 to only neg words, W-(-1) or W-0 */
				packuswb mm4, mm5   	/* combine and pack/saturate MM5 and MM4 */
				movq [edi], mm4   	/* store result in Dest */
				/* ---, */
				sub esi, eax   	/* move to the current top row in Src */
				sub esi, eax
				add esi, 8   	/* move Src  pointer to the next 8 pixels */
				add edi, 8   	/* move Dest pointer to the next 8 pixels */
				/* ---, */
				dec              ecx    	/* decrease loop counter COLUMNS */
				jnz            L10402    	/* check loop termination, proceed if required */
				mov esi, ebx   	/* restore most left current row Src  address */
				movd edi, mm1   	/* restore most left current row Dest address */
				add esi, eax   	/* move to the next row in Src */
				add edi, eax   	/* move to the next row in Dest */
				dec              edx    	/* decrease loop counter ROWS */
				jnz            L10400    	/* check loop termination, proceed if required */
				/* ---, */
				emms                      	/* exit MMX state */
				popa
		}
#else
		asm volatile
			("pusha		     \n\t" "pxor      %%mm0, %%mm0 \n\t"	/* zero MM0 */
			"mov          %3, %%eax \n\t"	/* load columns into EAX */
			/* --- */
			"mov          %1, %%esi \n\t"	/* ESI = Src row 0 address */
			"mov          %0, %%edi \n\t"	/* load Dest address to EDI */
			"add       %%eax, %%edi \n\t"	/* EDI = EDI + columns */
			"inc              %%edi \n\t"	/* 1 byte offset from the left edge */
			"mov          %2, %%edx \n\t"	/* initialize ROWS counter */
			"sub          $2, %%edx \n\t"	/* do not use first and last rows */
			/* --- */
			".L10400:                \n\t" "mov       %%eax, %%ecx \n\t"	/* initialize COLUMS counter */
			"shr          $3, %%ecx \n\t"	/* EBX/8 (MMX loads 8 bytes at a time) */
			"mov       %%esi, %%ebx \n\t"	/* save ESI in EBX */
			"movd      %%edi, %%mm1 \n\t"	/* save EDI in MM1 */
			".align 16              \n\t"	/* 16 byte alignment of the loop entry */
			".L10402:               \n\t"
			/* --- */
			"movq    (%%esi), %%mm4 \n\t"	/* load 8 bytes from Src */
			"movq      %%mm4, %%mm5 \n\t"	/* save MM4 in MM5 */
			"add          $2, %%esi \n\t"	/* move ESI pointer 2 bytes right */
			"punpcklbw %%mm0, %%mm4 \n\t"	/* unpack 4 low  bytes into words */
			"punpckhbw %%mm0, %%mm5 \n\t"	/* unpack 4 high bytes into words */
			"movq    (%%esi), %%mm6 \n\t"	/* load 8 bytes from Src */
			"movq      %%mm6, %%mm7 \n\t"	/* save MM6 in MM7 */
			"sub          $2, %%esi \n\t"	/* move ESI pointer back 2 bytes left */
			"punpcklbw %%mm0, %%mm6 \n\t"	/* unpack 4 low  bytes into words */
			"punpckhbw %%mm0, %%mm7 \n\t"	/* unpack 4 high bytes into words */
			"add       %%eax, %%esi \n\t"	/* move to the next row of Src */
			"movq    (%%esi), %%mm2 \n\t"	/* load 8 bytes from Src */
			"movq      %%mm2, %%mm3 \n\t"	/* save MM2 in MM3 */
			"add          $2, %%esi \n\t"	/* move ESI pointer 2 bytes right */
			"punpcklbw %%mm0, %%mm2 \n\t"	/* unpack 4 low  bytes into words */
			"punpckhbw %%mm0, %%mm3 \n\t"	/* unpack 4 high bytes into words */
			"paddw     %%mm2, %%mm4 \n\t"	/* add 4 low  bytes to accumolator MM4 */
			"paddw     %%mm3, %%mm5 \n\t"	/* add 4 high bytes to accumolator MM5 */
			"paddw     %%mm2, %%mm4 \n\t"	/* add 4 low  bytes to accumolator MM4 */
			"paddw     %%mm3, %%mm5 \n\t"	/* add 4 high bytes to accumolator MM5 */
			"movq    (%%esi), %%mm2 \n\t"	/* load 8 bytes from Src */
			"movq      %%mm2, %%mm3 \n\t"	/* save MM2 in MM3 */
			"sub          $2, %%esi \n\t"	/* move ESI pointer back 2 bytes left */
			"punpcklbw %%mm0, %%mm2 \n\t"	/* unpack 4 low  bytes into words */
			"punpckhbw %%mm0, %%mm3 \n\t"	/* unpack 4 high bytes into words */
			"paddw     %%mm2, %%mm6 \n\t"	/* add 4 low  bytes to accumolator MM6 */
			"paddw     %%mm3, %%mm7 \n\t"	/* add 4 high bytes to accumolator MM7 */
			"paddw     %%mm2, %%mm6 \n\t"	/* add 4 low  bytes to accumolator MM6 */
			"paddw     %%mm3, %%mm7 \n\t"	/* add 4 high bytes to accumolator MM7 */
			"add       %%eax, %%esi \n\t"	/* move to the next row of Src */
			"movq    (%%esi), %%mm2 \n\t"	/* load 8 bytes from Src */
			"movq      %%mm2, %%mm3 \n\t"	/* save MM2 in MM3 */
			"add          $2, %%esi \n\t"	/* move ESI pointer 2 bytes right */
			"punpcklbw %%mm0, %%mm2 \n\t"	/* unpack 4 low  bytes into words */
			"punpckhbw %%mm0, %%mm3 \n\t"	/* unpack 4 high bytes into words */
			"paddw     %%mm2, %%mm4 \n\t"	/* add 4 low  bytes to accumolator MM4 */
			"paddw     %%mm3, %%mm5 \n\t"	/* add 4 high bytes to accumolator MM5 */
			"movq    (%%esi), %%mm2 \n\t"	/* load 8 bytes from Src */
			"movq      %%mm2, %%mm3 \n\t"	/* save MM2 in MM3 */
			"sub          $2, %%esi \n\t"	/* move ESI pointer back 2 bytes left */
			"punpcklbw %%mm0, %%mm2 \n\t"	/* unpack 4 low  bytes into words */
			"punpckhbw %%mm0, %%mm3 \n\t"	/* unpack 4 high bytes into words */
			"paddw     %%mm2, %%mm6 \n\t"	/* add 4 low  bytes to accumolator MM6 */
			"paddw     %%mm3, %%mm7 \n\t"	/* add 4 high bytes to accumolator MM7 */
			/* --- */
			"movq      %%mm4, %%mm2 \n\t"	/* copy MM4 into MM2 */
			"psrlq       $32, %%mm4 \n\t"	/* shift 2 left words to the right */
			"psubw     %%mm2, %%mm4 \n\t"	/* MM4 = MM4 - MM2 */
			"movq      %%mm6, %%mm3 \n\t"	/* copy MM6 into MM3 */
			"psrlq       $32, %%mm6 \n\t"	/* shift 2 left words to the right */
			"psubw     %%mm3, %%mm6 \n\t"	/* MM6 = MM6 - MM3 */
			"punpckldq %%mm6, %%mm4 \n\t"	/* combine 2 words of MM6 and 2 words of MM4 */
			"movq      %%mm5, %%mm2 \n\t"	/* copy MM6 into MM2 */
			"psrlq       $32, %%mm5 \n\t"	/* shift 2 left words to the right */
			"psubw     %%mm2, %%mm5 \n\t"	/* MM5 = MM5 - MM2 */
			"movq      %%mm7, %%mm3 \n\t"	/* copy MM7 into MM3 */
			"psrlq       $32, %%mm7 \n\t"	/* shift 2 left words to the right */
			"psubw     %%mm3, %%mm7 \n\t"	/* MM7 = MM7 - MM3 */
			"punpckldq %%mm7, %%mm5 \n\t"	/* combine 2 words of MM7 and 2 words of MM5 */
			/* Take abs values of MM4 and MM5 */
			"movq      %%mm4, %%mm6 \n\t"	/* copy MM4 into MM6 */
			"movq      %%mm5, %%mm7 \n\t"	/* copy MM5 into MM7 */
			"psraw       $15, %%mm6 \n\t"	/* fill MM6 words with word sign bit */
			"psraw       $15, %%mm7 \n\t"	/* fill MM7 words with word sign bit */
			"pxor      %%mm6, %%mm4 \n\t"	/* take 1's compliment of only neg. words */
			"pxor      %%mm7, %%mm5 \n\t"	/* take 1's compliment of only neg. words */
			"psubsw    %%mm6, %%mm4 \n\t"	/* add 1 to only neg. words, W-(-1) or W-0 */
			"psubsw    %%mm7, %%mm5 \n\t"	/* add 1 to only neg. words, W-(-1) or W-0 */
			"packuswb  %%mm5, %%mm4 \n\t"	/* combine and pack/saturate MM5 and MM4 */
			"movq    %%mm4, (%%edi) \n\t"	/* store result in Dest */
			/* --- */
			"sub       %%eax, %%esi \n\t"	/* move to the current top row in Src */
			"sub       %%eax, %%esi \n\t" "add $8,          %%esi \n\t"	/* move Src  pointer to the next 8 pixels */
			"add $8,          %%edi \n\t"	/* move Dest pointer to the next 8 pixels */
			/* --- */
			"dec              %%ecx \n\t"	/* decrease loop counter COLUMNS */
			"jnz            .L10402 \n\t"	/* check loop termination, proceed if required */
			"mov       %%ebx, %%esi \n\t"	/* restore most left current row Src  address */
			"movd      %%mm1, %%edi \n\t"	/* restore most left current row Dest address */
			"add       %%eax, %%esi \n\t"	/* move to the next row in Src */
			"add       %%eax, %%edi \n\t"	/* move to the next row in Dest */
			"dec              %%edx \n\t"	/* decrease loop counter ROWS */
			"jnz            .L10400 \n\t"	/* check loop termination, proceed if required */
			/* --- */
			"emms                   \n\t"	/* exit MMX state */
			"popa                   \n\t":"=m" (Dest)	/* %0 */
			:"m"(Src),		/* %1 */
			"m"(rows),		/* %2 */
			"m"(columns)		/* %3 */
			);
#endif
#endif
		return (0);
	} else {
		/* No non-MMX implementation yet */
		return (-1);
	}
}

/*!
\brief Filter using SobelXShiftRight: Dij = saturation255( ... ) 

\param Src The source 2D byte array to sobel-filter. Should be different from destination.
\param Dest The destination 2D byte array to store the result in. Should be different from source.
\param rows Number of rows in source/destination array. Must be >2.
\param columns Number of columns in source/destination array. Must be >8.
\param NRightShift The number of right bit shifts to apply to the filter sum. Must be <7.

Note: Non-MMX implementation not available for this function.

\return Returns 1 if filter was applied, 0 otherwise.
*/
int SDL_imageFilterSobelXShiftRight(unsigned char *Src, unsigned char *Dest, int rows, int columns,
									unsigned char NRightShift)
{
	/* Validate input parameters */
	if ((Src == NULL) || (Dest == NULL))
		return(-1);
	if ((columns < 8) || (rows < 3) || (NRightShift > 7))
		return (-1);

	if ((SDL_imageFilterMMXdetect())) {
//#ifdef USE_MMX
#if defined(USE_MMX) && defined(i386)
#if !defined(GCC__)
		__asm
		{
			pusha
				pxor mm0, mm0   	/* zero MM0 */
				mov eax, columns   	/* load columns into EAX */
				xor ebx, ebx   	/* zero EBX */
				mov bl, NRightShift   	/* load NRightShift into BL */
				movd mm1, ebx   	/* copy NRightShift into MM1 */
				/* ---, */
				mov esi, Src   	/* ESI = Src row 0 address */
				mov edi, Dest   	/* load Dest address to EDI */
				add edi, eax   	/* EDI = EDI + columns */
				inc              edi    	/* 1 byte offset from the left edge */
				/* initialize ROWS counter */
				sub rows, 2   	/* do not use first and last rows */
				/* ---, */
L10410:
			mov ecx, eax   	/* initialize COLUMS counter */
				shr ecx, 3   	/* EBX/8 (MMX loads 8 bytes at a time) */
				mov ebx, esi   	/* save ESI in EBX */
				mov edx, edi   	/* save EDI in EDX */
				align 16                 	/* 16 byte alignment of the loop entry */
L10412:
			/* ---, */
			movq mm4, [esi]   	/* load 8 bytes from Src */
			movq mm5, mm4   	/* save MM4 in MM5 */
				add esi, 2   	/* move ESI pointer 2 bytes right */
				punpcklbw mm4, mm0   	/* unpack 4 low  bytes into words */
				punpckhbw mm5, mm0   	/* unpack 4 high bytes into words */
				psrlw mm4, mm1   	/* shift right each pixel NshiftRight times */
				psrlw mm5, mm1   	/* shift right each pixel NshiftRight times */
				movq mm6, [esi]   	/* load 8 bytes from Src */
			movq mm7, mm6   	/* save MM6 in MM7 */
				sub esi, 2   	/* move ESI pointer back 2 bytes left */
				punpcklbw mm6, mm0   	/* unpack 4 low  bytes into words */
				punpckhbw mm7, mm0   	/* unpack 4 high bytes into words */
				psrlw mm6, mm1   	/* shift right each pixel NshiftRight times */
				psrlw mm7, mm1   	/* shift right each pixel NshiftRight times */
				add esi, eax   	/* move to the next row of Src */
				movq mm2, [esi]   	/* load 8 bytes from Src */
			movq mm3, mm2   	/* save MM2 in MM3 */
				add esi, 2   	/* move ESI pointer 2 bytes right */
				punpcklbw mm2, mm0   	/* unpack 4 low  bytes into words */
				punpckhbw mm3, mm0   	/* unpack 4 high bytes into words */
				psrlw mm2, mm1   	/* shift right each pixel NshiftRight times */
				psrlw mm3, mm1   	/* shift right each pixel NshiftRight times */
				paddw mm4, mm2   	/* add 4 low  bytes to accumolator MM4 */
				paddw mm5, mm3   	/* add 4 high bytes to accumolator MM5 */
				paddw mm4, mm2   	/* add 4 low  bytes to accumolator MM4 */
				paddw mm5, mm3   	/* add 4 high bytes to accumolator MM5 */
				movq mm2, [esi]   	/* load 8 bytes from Src */
			movq mm3, mm2   	/* save MM2 in MM3 */
				sub esi, 2   	/* move ESI pointer back 2 bytes left */
				punpcklbw mm2, mm0   	/* unpack 4 low  bytes into words */
				punpckhbw mm3, mm0   	/* unpack 4 high bytes into words */
				psrlw mm2, mm1   	/* shift right each pixel NshiftRight times */
				psrlw mm3, mm1   	/* shift right each pixel NshiftRight times */
				paddw mm6, mm2   	/* add 4 low  bytes to accumolator MM6 */
				paddw mm7, mm3   	/* add 4 high bytes to accumolator MM7 */
				paddw mm6, mm2   	/* add 4 low  bytes to accumolator MM6 */
				paddw mm7, mm3   	/* add 4 high bytes to accumolator MM7 */
				add esi, eax   	/* move to the next row of Src */
				movq mm2, [esi]   	/* load 8 bytes from Src */
			movq mm3, mm2   	/* save MM2 in MM3 */
				add esi, 2   	/* move ESI pointer 2 bytes right */
				punpcklbw mm2, mm0   	/* unpack 4 low  bytes into words */
				punpckhbw mm3, mm0   	/* unpack 4 high bytes into words */
				psrlw mm2, mm1   	/* shift right each pixel NshiftRight times */
				psrlw mm3, mm1   	/* shift right each pixel NshiftRight times */
				paddw mm4, mm2   	/* add 4 low  bytes to accumolator MM4 */
				paddw mm5, mm3   	/* add 4 high bytes to accumolator MM5 */
				movq mm2, [esi]   	/* load 8 bytes from Src */
			movq mm3, mm2   	/* save MM2 in MM3 */
				sub esi, 2   	/* move ESI pointer back 2 bytes left */
				punpcklbw mm2, mm0   	/* unpack 4 low  bytes into words */
				punpckhbw mm3, mm0   	/* unpack 4 high bytes into words */
				psrlw mm2, mm1   	/* shift right each pixel NshiftRight times */
				psrlw mm3, mm1   	/* shift right each pixel NshiftRight times */
				paddw mm6, mm2   	/* add 4 low  bytes to accumolator MM6 */
				paddw mm7, mm3   	/* add 4 high bytes to accumolator MM7 */
				/* ---, */
				movq mm2, mm4   	/* copy MM4 into MM2 */
				psrlq mm4, 32   	/* shift 2 left words to the right */
				psubw mm4, mm2   	/* MM4 = MM4 - MM2 */
				movq mm3, mm6   	/* copy MM6 into MM3 */
				psrlq mm6, 32   	/* shift 2 left words to the right */
				psubw mm6, mm3   	/* MM6 = MM6 - MM3 */
				punpckldq mm4, mm6   	/* combine 2 words of MM6 and 2 words of MM4 */
				movq mm2, mm5   	/* copy MM6 into MM2 */
				psrlq mm5, 32   	/* shift 2 left words to the right */
				psubw mm5, mm2   	/* MM5 = MM5 - MM2 */
				movq mm3, mm7   	/* copy MM7 into MM3 */
				psrlq mm7, 32   	/* shift 2 left words to the right */
				psubw mm7, mm3   	/* MM7 = MM7 - MM3 */
				punpckldq mm5, mm7   	/* combine 2 words of MM7 and 2 words of MM5 */
				/* Take abs values of MM4 and MM5 */
				movq mm6, mm4   	/* copy MM4 into MM6 */
				movq mm7, mm5   	/* copy MM5 into MM7 */
				psraw mm6, 15   	/* fill MM6 words with word sign bit */
				psraw mm7, 15   	/* fill MM7 words with word sign bit */
				pxor mm4, mm6   	/* take 1's compliment of only neg words */
				pxor mm5, mm7   	/* take 1's compliment of only neg words */
				psubsw mm4, mm6   	/* add 1 to only neg words, W-(-1) or W-0 */
				psubsw mm5, mm7   	/* add 1 to only neg words, W-(-1) or W-0 */
				packuswb mm4, mm5   	/* combine and pack/saturate MM5 and MM4 */
				movq [edi], mm4   	/* store result in Dest */
				/* ---, */
				sub esi, eax   	/* move to the current top row in Src */
				sub esi, eax
				add esi, 8   	/* move Src  pointer to the next 8 pixels */
				add edi, 8   	/* move Dest pointer to the next 8 pixels */
				/* ---, */
				dec              ecx    	/* decrease loop counter COLUMNS */
				jnz            L10412    	/* check loop termination, proceed if required */
				mov esi, ebx   	/* restore most left current row Src  address */
				mov edi, edx   	/* restore most left current row Dest address */
				add esi, eax   	/* move to the next row in Src */
				add edi, eax   	/* move to the next row in Dest */
				dec rows    	/* decrease loop counter ROWS */
				jnz            L10410    	/* check loop termination, proceed if required */
				/* ---, */
				emms                      	/* exit MMX state */
				popa
		}
#else
		asm volatile
			("pusha		     \n\t" "pxor      %%mm0, %%mm0 \n\t"	/* zero MM0 */
			"mov          %3, %%eax \n\t"	/* load columns into EAX */
			"xor       %%ebx, %%ebx \n\t"	/* zero EBX */
			"mov           %4, %%bl \n\t"	/* load NRightShift into BL */
			"movd      %%ebx, %%mm1 \n\t"	/* copy NRightShift into MM1 */
			/* --- */
			"mov          %1, %%esi \n\t"	/* ESI = Src row 0 address */
			"mov          %0, %%edi \n\t"	/* load Dest address to EDI */
			"add       %%eax, %%edi \n\t"	/* EDI = EDI + columns */
			"inc              %%edi \n\t"	/* 1 byte offset from the left edge */
			/* initialize ROWS counter */
			"subl            $2, %2 \n\t"	/* do not use first and last rows */
			/* --- */
			".L10410:                \n\t" "mov       %%eax, %%ecx \n\t"	/* initialize COLUMS counter */
			"shr          $3, %%ecx \n\t"	/* EBX/8 (MMX loads 8 bytes at a time) */
			"mov       %%esi, %%ebx \n\t"	/* save ESI in EBX */
			"mov       %%edi, %%edx \n\t"	/* save EDI in EDX */
			".align 16              \n\t"	/* 16 byte alignment of the loop entry */
			".L10412:               \n\t"
			/* --- */
			"movq    (%%esi), %%mm4 \n\t"	/* load 8 bytes from Src */
			"movq      %%mm4, %%mm5 \n\t"	/* save MM4 in MM5 */
			"add          $2, %%esi \n\t"	/* move ESI pointer 2 bytes right */
			"punpcklbw %%mm0, %%mm4 \n\t"	/* unpack 4 low  bytes into words */
			"punpckhbw %%mm0, %%mm5 \n\t"	/* unpack 4 high bytes into words */
			"psrlw     %%mm1, %%mm4 \n\t"	/* shift right each pixel NshiftRight times */
			"psrlw     %%mm1, %%mm5 \n\t"	/* shift right each pixel NshiftRight times */
			"movq    (%%esi), %%mm6 \n\t"	/* load 8 bytes from Src */
			"movq      %%mm6, %%mm7 \n\t"	/* save MM6 in MM7 */
			"sub          $2, %%esi \n\t"	/* move ESI pointer back 2 bytes left */
			"punpcklbw %%mm0, %%mm6 \n\t"	/* unpack 4 low  bytes into words */
			"punpckhbw %%mm0, %%mm7 \n\t"	/* unpack 4 high bytes into words */
			"psrlw     %%mm1, %%mm6 \n\t"	/* shift right each pixel NshiftRight times */
			"psrlw     %%mm1, %%mm7 \n\t"	/* shift right each pixel NshiftRight times */
			"add       %%eax, %%esi \n\t"	/* move to the next row of Src */
			"movq    (%%esi), %%mm2 \n\t"	/* load 8 bytes from Src */
			"movq      %%mm2, %%mm3 \n\t"	/* save MM2 in MM3 */
			"add          $2, %%esi \n\t"	/* move ESI pointer 2 bytes right */
			"punpcklbw %%mm0, %%mm2 \n\t"	/* unpack 4 low  bytes into words */
			"punpckhbw %%mm0, %%mm3 \n\t"	/* unpack 4 high bytes into words */
			"psrlw     %%mm1, %%mm2 \n\t"	/* shift right each pixel NshiftRight times */
			"psrlw     %%mm1, %%mm3 \n\t"	/* shift right each pixel NshiftRight times */
			"paddw     %%mm2, %%mm4 \n\t"	/* add 4 low  bytes to accumolator MM4 */
			"paddw     %%mm3, %%mm5 \n\t"	/* add 4 high bytes to accumolator MM5 */
			"paddw     %%mm2, %%mm4 \n\t"	/* add 4 low  bytes to accumolator MM4 */
			"paddw     %%mm3, %%mm5 \n\t"	/* add 4 high bytes to accumolator MM5 */
			"movq    (%%esi), %%mm2 \n\t"	/* load 8 bytes from Src */
			"movq      %%mm2, %%mm3 \n\t"	/* save MM2 in MM3 */
			"sub          $2, %%esi \n\t"	/* move ESI pointer back 2 bytes left */
			"punpcklbw %%mm0, %%mm2 \n\t"	/* unpack 4 low  bytes into words */
			"punpckhbw %%mm0, %%mm3 \n\t"	/* unpack 4 high bytes into words */
			"psrlw     %%mm1, %%mm2 \n\t"	/* shift right each pixel NshiftRight times */
			"psrlw     %%mm1, %%mm3 \n\t"	/* shift right each pixel NshiftRight times */
			"paddw     %%mm2, %%mm6 \n\t"	/* add 4 low  bytes to accumolator MM6 */
			"paddw     %%mm3, %%mm7 \n\t"	/* add 4 high bytes to accumolator MM7 */
			"paddw     %%mm2, %%mm6 \n\t"	/* add 4 low  bytes to accumolator MM6 */
			"paddw     %%mm3, %%mm7 \n\t"	/* add 4 high bytes to accumolator MM7 */
			"add       %%eax, %%esi \n\t"	/* move to the next row of Src */
			"movq    (%%esi), %%mm2 \n\t"	/* load 8 bytes from Src */
			"movq      %%mm2, %%mm3 \n\t"	/* save MM2 in MM3 */
			"add          $2, %%esi \n\t"	/* move ESI pointer 2 bytes right */
			"punpcklbw %%mm0, %%mm2 \n\t"	/* unpack 4 low  bytes into words */
			"punpckhbw %%mm0, %%mm3 \n\t"	/* unpack 4 high bytes into words */
			"psrlw     %%mm1, %%mm2 \n\t"	/* shift right each pixel NshiftRight times */
			"psrlw     %%mm1, %%mm3 \n\t"	/* shift right each pixel NshiftRight times */
			"paddw     %%mm2, %%mm4 \n\t"	/* add 4 low  bytes to accumolator MM4 */
			"paddw     %%mm3, %%mm5 \n\t"	/* add 4 high bytes to accumolator MM5 */
			"movq    (%%esi), %%mm2 \n\t"	/* load 8 bytes from Src */
			"movq      %%mm2, %%mm3 \n\t"	/* save MM2 in MM3 */
			"sub          $2, %%esi \n\t"	/* move ESI pointer back 2 bytes left */
			"punpcklbw %%mm0, %%mm2 \n\t"	/* unpack 4 low  bytes into words */
			"punpckhbw %%mm0, %%mm3 \n\t"	/* unpack 4 high bytes into words */
			"psrlw     %%mm1, %%mm2 \n\t"	/* shift right each pixel NshiftRight times */
			"psrlw     %%mm1, %%mm3 \n\t"	/* shift right each pixel NshiftRight times */
			"paddw     %%mm2, %%mm6 \n\t"	/* add 4 low  bytes to accumolator MM6 */
			"paddw     %%mm3, %%mm7 \n\t"	/* add 4 high bytes to accumolator MM7 */
			/* --- */
			"movq      %%mm4, %%mm2 \n\t"	/* copy MM4 into MM2 */
			"psrlq       $32, %%mm4 \n\t"	/* shift 2 left words to the right */
			"psubw     %%mm2, %%mm4 \n\t"	/* MM4 = MM4 - MM2 */
			"movq      %%mm6, %%mm3 \n\t"	/* copy MM6 into MM3 */
			"psrlq       $32, %%mm6 \n\t"	/* shift 2 left words to the right */
			"psubw     %%mm3, %%mm6 \n\t"	/* MM6 = MM6 - MM3 */
			"punpckldq %%mm6, %%mm4 \n\t"	/* combine 2 words of MM6 and 2 words of MM4 */
			"movq      %%mm5, %%mm2 \n\t"	/* copy MM6 into MM2 */
			"psrlq       $32, %%mm5 \n\t"	/* shift 2 left words to the right */
			"psubw     %%mm2, %%mm5 \n\t"	/* MM5 = MM5 - MM2 */
			"movq      %%mm7, %%mm3 \n\t"	/* copy MM7 into MM3 */
			"psrlq       $32, %%mm7 \n\t"	/* shift 2 left words to the right */
			"psubw     %%mm3, %%mm7 \n\t"	/* MM7 = MM7 - MM3 */
			"punpckldq %%mm7, %%mm5 \n\t"	/* combine 2 words of MM7 and 2 words of MM5 */
			/* Take abs values of MM4 and MM5 */
			"movq      %%mm4, %%mm6 \n\t"	/* copy MM4 into MM6 */
			"movq      %%mm5, %%mm7 \n\t"	/* copy MM5 into MM7 */
			"psraw       $15, %%mm6 \n\t"	/* fill MM6 words with word sign bit */
			"psraw       $15, %%mm7 \n\t"	/* fill MM7 words with word sign bit */
			"pxor      %%mm6, %%mm4 \n\t"	/* take 1's compliment of only neg. words */
			"pxor      %%mm7, %%mm5 \n\t"	/* take 1's compliment of only neg. words */
			"psubsw    %%mm6, %%mm4 \n\t"	/* add 1 to only neg. words, W-(-1) or W-0 */
			"psubsw    %%mm7, %%mm5 \n\t"	/* add 1 to only neg. words, W-(-1) or W-0 */
			"packuswb  %%mm5, %%mm4 \n\t"	/* combine and pack/saturate MM5 and MM4 */
			"movq    %%mm4, (%%edi) \n\t"	/* store result in Dest */
			/* --- */
			"sub       %%eax, %%esi \n\t"	/* move to the current top row in Src */
			"sub       %%eax, %%esi \n\t" "add $8,          %%esi \n\t"	/* move Src  pointer to the next 8 pixels */
			"add $8,          %%edi \n\t"	/* move Dest pointer to the next 8 pixels */
			/* --- */
			"dec              %%ecx \n\t"	/* decrease loop counter COLUMNS */
			"jnz            .L10412 \n\t"	/* check loop termination, proceed if required */
			"mov       %%ebx, %%esi \n\t"	/* restore most left current row Src  address */
			"mov       %%edx, %%edi \n\t"	/* restore most left current row Dest address */
			"add       %%eax, %%esi \n\t"	/* move to the next row in Src */
			"add       %%eax, %%edi \n\t"	/* move to the next row in Dest */
			"decl                %2 \n\t"	/* decrease loop counter ROWS */
			"jnz            .L10410 \n\t"	/* check loop termination, proceed if required */
			/* --- */
			"emms                   \n\t"	/* exit MMX state */
			"popa                   \n\t":"=m" (Dest)	/* %0 */
			:"m"(Src),		/* %1 */
			"m"(rows),		/* %2 */
			"m"(columns),		/* %3 */
			"m"(NRightShift)	/* %4 */
			);
#endif
#endif
		return (0);
	} else {
		/* No non-MMX implementation yet */
		return (-1);
	}
}

/*!
\brief Align stack to 32 byte boundary,
*/
void SDL_imageFilterAlignStack(void)
{
#ifdef USE_MMX
#if !defined(GCC__)
	__asm
	{				/* --- stack alignment --- */
		mov ebx, esp   	/* load ESP into EBX */
			sub ebx, 4   	/* reserve space on stack for old value of ESP */
			and ebx, -32   	/* align EBX along a 32 byte boundary */
			mov [ebx], esp   	/* save old value of ESP in stack, behind the bndry */
			mov esp, ebx   	/* align ESP along a 32 byte boundary */
	}
#else
	asm volatile
		(				/* --- stack alignment --- */
		"mov       %%esp, %%ebx \n\t"	/* load ESP into EBX */
		"sub          $4, %%ebx \n\t"	/* reserve space on stack for old value of ESP */
		"and        $-32, %%ebx \n\t"	/* align EBX along a 32 byte boundary */
		"mov     %%esp, (%%ebx) \n\t"	/* save old value of ESP in stack, behind the bndry */
		"mov       %%ebx, %%esp \n\t"	/* align ESP along a 32 byte boundary */
		::);
#endif
#endif
}

/*!
\brief Restore previously aligned stack.
*/
void SDL_imageFilterRestoreStack(void)
{
#ifdef USE_MMX
#if !defined(GCC__)
	__asm
	{				/* --- restoring old stack --- */
		mov ebx, [esp]   	/* load old value of ESP */
		mov esp, ebx   	/* restore old value of ESP */
	}
#else
	asm volatile
		(				/* --- restoring old stack --- */
		"mov     (%%esp), %%ebx \n\t"	/* load old value of ESP */
		"mov       %%ebx, %%esp \n\t"	/* restore old value of ESP */
		::);
#endif
#endif
}
