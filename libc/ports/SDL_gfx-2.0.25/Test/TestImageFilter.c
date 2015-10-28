/* 

TestImageFilter.c: test program for MMX filter routines

(C) A. Schiffler, 2006, zlib license
(C) Sylvain Beucler, 2013, zlib license

*/

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#include "SDL.h"

#ifdef WIN32
#include <windows.h>
#include "SDL_imageFilter.h"
#ifndef bcmp
#define bcmp(s1, s2, n) memcmp ((s1), (s2), (n))
#endif
#else
#include "SDL/SDL_imageFilter.h"
#endif

#define SRC_SIZE 23

int total_count = 0;
int ok_count = 0;

void setup_src(unsigned char *src1, unsigned char *src2)
{
	int i;

	src1[0]=1;
	src1[2]=1; 
	src1[1]=4;
	src1[3]=3;
	src1[4]=33;
	for (i=5; i<14; i++) src1[i]=i;
	src1[14]=8;
	for (i=15; i<SRC_SIZE; i++) src1[i]=rand();

	src2[0]=1;
	src2[1]=3;
	src2[2]=3; 
	src2[3]=2;
	src2[4]=44;
	for (i=5; i<14; i++) src2[i]=14-i;
	src2[14]=10;
	for (i=15; i<SRC_SIZE; i++) src2[i]=src1[i];
}

void print_result(int mmx, char *label, unsigned char *src1, unsigned char *src2, unsigned char *dst) 
{
	char blabel[80];
	int i;
	memset((void *)blabel, ' ', 80);
	blabel[strlen(label)+4]=0;

	printf("\n");
	printf ("%s   pos   ", blabel);
	for (i = 0; i < SRC_SIZE; i++)
		printf("%2d ", i);
	printf("\n");

	printf ("%s   src1  ", blabel);
	for (i = 0; i < SRC_SIZE; i++)
		printf("%02x ", src1[i]);
	printf("\n");

	if (src2) {
		printf ("%s   src2  ", blabel);
		for (i = 0; i < SRC_SIZE; i++)
			printf("%02x ", src2[i]);
	}
	printf("\n");

	printf ("%s %s   dest  ",mmx?"MMX":" C ",label);
	for (i = 0; i < SRC_SIZE; i++)
		printf("%02x ", dst[i]);
	printf("\n");
}

void print_compare(unsigned char *dst1, unsigned char *dst2) 
{ 
	total_count++;
	if (bcmp(dst1,dst2,SRC_SIZE)==0) {
		printf ("OK\n");
		ok_count++;
	} else {
		printf ("ERROR\n");
	}
}

void print_line() 
{
	printf ("------------------------------------------------------------------------\n\n\n");
}

void pause()
{
	char ch;
	do {
		ch = getchar();
		putchar('.');
	} while (ch != '\n');
}

/* ----------- main ---------- */

int main(int argc, char *argv[])
{
	unsigned char src1[SRC_SIZE], src2[SRC_SIZE], dstm[SRC_SIZE], dstc[SRC_SIZE];
	int size = 2*1024*1024;
	unsigned char *t1 = (unsigned char *)malloc(size), *t2 = (unsigned char *)malloc(size), *d = (unsigned char *)malloc(size);
	int i;

	// Interestingly, C tests are about 4x faster
	// on malloc(size) than on char[size]

	printf("src1:\t%s (%p)\tsrc2:\t%s (%p)\tdstm:\t%s (%p)\tdstc:\t%s (%p)\n",
		((long long)src1%8) ? "not aligned" : "aligned", src1,
		((long long)src2%8) ? "not aligned" : "aligned", src2,
		((long long)dstm%8) ? "not aligned" : "aligned", dstm,
		((long long)dstc%8) ? "not aligned" : "aligned", dstc);

	printf("t1:\t%s (%p)\tt2:\t%s (%p)\td:\t%s (%p)\n",
		((long long)t1%8) ? "not aligned" : "aligned", t1,
		((long long)t2%8) ? "not aligned" : "aligned", t2,
		((long long) d%8) ? "not aligned" : "aligned",  d);

	{
		/* Initialize to make valgrind happy */
		srand((unsigned int)time(NULL));
		for (i = 0; i < size; i++) {
			/* use more random lower-order bits (int->char) */
			t1[i] = rand(); t2[i] = rand(); d[i] = rand();
		}
	}

	SDL_Init(SDL_INIT_TIMER);

	/* SDL_imageFilter Test */

	printf ("TestImageFilter\n\n");
	printf ("Testing an array of 23 bytes - first 16 bytes should be processed\n");
	printf ("by MMX or C code, the last 7 bytes only by C code.\n\n");

	print_line();


#define	TEST_C   0
#define	TEST_MMX 1
	{
#define FUNC(f) { #f, SDL_imageFilter ## f }
		struct func {
			char* name;
			int (*f)(unsigned char*, unsigned char*, unsigned char*, unsigned int);
		};
		struct func funcs[] = {
			FUNC(BitAnd),
			FUNC(BitOr),
			FUNC(Add),
			FUNC(AbsDiff),
			FUNC(Mean),
			FUNC(Sub),
			FUNC(Mult),
			FUNC(MultNor),
			FUNC(MultDivby2),
			FUNC(MultDivby4),
			FUNC(Div),
		};

		int k;
		for (k = 0; k < sizeof(funcs)/sizeof(struct func); k++) {
			Uint32 start;
			int i;

			setup_src(src1, src2);

			SDL_imageFilterMMXon();
			funcs[k].f(src1, src2, dstm, SRC_SIZE);
			print_result(TEST_MMX, funcs[k].name, src1, src2, dstm);
			start = SDL_GetTicks();
			for (i = 0; i < 50; i++) {
				funcs[k].f(t1, t2, d, size);
			}
			printf("MMX %dx%dk: %dms\n", i, size/1024, SDL_GetTicks() - start);

			SDL_imageFilterMMXoff();
			funcs[k].f(src1, src2, dstc, SRC_SIZE);
			print_result(TEST_C, funcs[k].name, src1, src2, dstc);
			start = SDL_GetTicks();
			for (i = 0; i < 50; i++) {
				funcs[k].f(t1, t2, d, size);
			}
			printf(" C  %dx%dk: %dms\n", i, size/1024, SDL_GetTicks() - start);

			print_compare(dstm,dstc);
			print_line();
		}
	}

	{
		Uint32 start;
		int i;
		char call[1024];
		sprintf(call, "BitNegation");

		setup_src(src1, src2);

		SDL_imageFilterMMXon();
		SDL_imageFilterBitNegation(src1, dstm, SRC_SIZE);
		print_result(TEST_MMX, call, src1, NULL, dstm);
		start = SDL_GetTicks();
		for (i = 0; i < 50; i++) {
			SDL_imageFilterBitNegation(t1, d, size);
		}
		printf("MMX %dx%dk: %dms\n", i, size/1024, SDL_GetTicks() - start);

		SDL_imageFilterMMXoff();
		SDL_imageFilterBitNegation(src1, dstc, SRC_SIZE);
		print_result(TEST_C, call, src1, NULL, dstc);
		start = SDL_GetTicks();
		for (i = 0; i < 50; i++) {
			SDL_imageFilterBitNegation(t1, d, size);
		}
		printf(" C  %dx%dk: %dms\n", i, size/1024, SDL_GetTicks() - start);

		print_compare(dstm,dstc);
		print_line();
	}


	{
#undef FUNC
#define FUNC(f, c) { #f, SDL_imageFilter ## f, c }
		struct func {
			char* name;
			int (*f)(unsigned char*, unsigned char*, unsigned int, unsigned char);
			unsigned char arg;
		};
		struct func funcs[] = {
			FUNC(AddByte,                3),
			FUNC(AddByteToHalf,          3),
			FUNC(SubByte,                3),
			FUNC(ShiftRight,             1),
			FUNC(ShiftRightUint,         4),
			FUNC(MultByByte,             3),
			FUNC(ShiftLeftByte,          3),
			FUNC(ShiftLeft,              3),
			FUNC(ShiftLeftUint,          4),
			FUNC(BinarizeUsingThreshold, 9),
		};

		int k;
		for (k = 0; k < sizeof(funcs)/sizeof(struct func); k++) {
			Uint32 start;
			int i;
			char call[1024];
			sprintf(call, "%s(%u)", funcs[k].name, funcs[k].arg);

			setup_src(src1, src2);

			SDL_imageFilterMMXon();
			funcs[k].f(src1, dstm, SRC_SIZE, funcs[k].arg);
			print_result(TEST_MMX, call, src1, NULL, dstm);
			start = SDL_GetTicks();
			for (i = 0; i < 50; i++) {
				funcs[k].f(t1, d, size, funcs[k].arg);
			}
			printf("MMX %dx%dk: %dms\n", i, size/1024, SDL_GetTicks() - start);

			SDL_imageFilterMMXoff();
			funcs[k].f(src1, dstc, SRC_SIZE, funcs[k].arg);
			print_result(TEST_C, call, src1, NULL, dstc);
			start = SDL_GetTicks();
			for (i = 0; i < 50; i++) {
				funcs[k].f(t1, d, size, funcs[k].arg);
			}
			printf(" C  %dx%dk: %dms\n", i, size/1024, SDL_GetTicks() - start);

			print_compare(dstm,dstc);
			print_line();
		}
	}


	{
#undef FUNC
#define FUNC(f, c1, c2) { #f, SDL_imageFilter ## f, c1, c2 }
		struct func {
			char* name;
			int (*f)(unsigned char*, unsigned char*, unsigned int, unsigned char, unsigned char);
			unsigned char arg1, arg2;
		};
		struct func funcs[] = {
			FUNC(ShiftRightAndMultByByte, 1, 3),
			FUNC(ClipToRange, 3, 8),
		};

		int k;
		for (k = 0; k < sizeof(funcs)/sizeof(struct func); k++) {
			Uint32 start;
			int i;
			char call[1024];
			sprintf(call, "%s(%u,%u)", funcs[k].name, funcs[k].arg1, funcs[k].arg2);

			setup_src(src1, src2);

			SDL_imageFilterMMXon();
			funcs[k].f(src1, dstm, SRC_SIZE, funcs[k].arg1, funcs[k].arg2);
			print_result(TEST_MMX, call, src1, NULL, dstm);
			start = SDL_GetTicks();
			for (i = 0; i < 50; i++) {
				funcs[k].f(t1, d, size, funcs[k].arg1, funcs[k].arg2);
			}
			printf("MMX %dx%dk: %dms\n", i, size/1024, SDL_GetTicks() - start);

			SDL_imageFilterMMXoff();
			funcs[k].f(src1, dstc, SRC_SIZE, funcs[k].arg1, funcs[k].arg2);
			print_result(TEST_C, call, src1, NULL, dstc);
			start = SDL_GetTicks();
			for (i = 0; i < 50; i++) {
				funcs[k].f(t1, d, size, funcs[k].arg1, funcs[k].arg2);
			}
			printf(" C  %dx%dk: %dms\n", i, size/1024, SDL_GetTicks() - start);

			print_compare(dstm,dstc);
			print_line();
		}
	}


	{
		Uint32 start;
		int i;
		char call[1024];
		sprintf(call, "NormalizeLinear(0,33,0,255)");

		setup_src(src1, src2);

		SDL_imageFilterMMXon();
		SDL_imageFilterNormalizeLinear(src1, dstm, SRC_SIZE, 0,33, 0,255);
		print_result(TEST_MMX, call, src1, NULL, dstm);
		start = SDL_GetTicks();
		for (i = 0; i < 50; i++) {
			SDL_imageFilterNormalizeLinear(t1, d, size, 0,33, 0,255);
		}
		printf("MMX %dx%dk: %dms\n", i, size/1024, SDL_GetTicks() - start);

		SDL_imageFilterMMXoff();
		SDL_imageFilterNormalizeLinear(src1, dstc, SRC_SIZE, 0,33, 0,255);
		print_result(TEST_C, call, src1, NULL, dstc);
		start = SDL_GetTicks();
		for (i = 0; i < 50; i++) {
			SDL_imageFilterNormalizeLinear(t1, d, size, 0,33, 0,255);
		}
		printf(" C  %dx%dk: %dms\n", i, size/1024, SDL_GetTicks() - start);

		print_compare(dstm,dstc);
		print_line();
	}


	/* Uint functions */
	/* Disabled, since broken *//* ??? */
	{
#undef FUNC
#define FUNC(f, c) { #f, SDL_imageFilter ## f, c }
		struct func {
			char* name;
			int (*f)(unsigned char*, unsigned char*, unsigned int, unsigned int);
			unsigned int arg;
		};
		struct func funcs[] = {
			FUNC(AddUint,       0x01020304),
			FUNC(SubUint,       0x01020304),
		};

		int k;
		for (k = 0; k < sizeof(funcs)/sizeof(struct func); k++) {
			Uint32 start;
			int i;
			char call[1024];
			sprintf(call, "%s(%u)", funcs[k].name, funcs[k].arg);

			setup_src(src1, src2);

			SDL_imageFilterMMXon();
			funcs[k].f(src1, dstm, SRC_SIZE, funcs[k].arg);
			print_result(TEST_MMX, call, src1, NULL, dstm);
			start = SDL_GetTicks();
			for (i = 0; i < 50; i++) {
				funcs[k].f(t1, d, size, funcs[k].arg);
			}
			printf("MMX %dx%dk: %dms\n", i, size/1024, SDL_GetTicks() - start);

			SDL_imageFilterMMXoff();
			funcs[k].f(src1, dstc, SRC_SIZE, funcs[k].arg);
			print_result(TEST_C, call, src1, NULL, dstc);
			start = SDL_GetTicks();
			for (i = 0; i < 50; i++) {
				funcs[k].f(t1, d, size, funcs[k].arg);
			}
			printf(" C  %dx%dk: %dms\n", i, size/1024, SDL_GetTicks() - start);

			print_compare(dstm,dstc);
			print_line();
		}
	}


	SDL_imageFilterMMXon();
	if (SDL_imageFilterMMXdetect())
	{
		printf("MMX was detected\n\n");
	}
	else
	{
		printf("MMX was NOT detected\n\n");
	}

	printf ("Result: %i of %i passed OK.\n", ok_count, total_count);

#ifdef WIN32 
	printf("Press Enter to continue ...");
	pause();
#endif

	SDL_Quit();
	free(d);
	free(t2);
	free(t1);
	exit(0);
}
