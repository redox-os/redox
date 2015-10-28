/*
    SDL - Simple DirectMedia Layer
    Copyright (C) 1997-2012 Sam Lantinga

    This library is free software; you can redistribute it and/or
    modify it under the terms of the GNU Lesser General Public
    License as published by the Free Software Foundation; either
    version 2.1 of the License, or (at your option) any later version.

    This library is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
    Lesser General Public License for more details.

    You should have received a copy of the GNU Lesser General Public
    License along with this library; if not, write to the Free Software
    Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA  02110-1301  USA

    Sam Lantinga
    slouken@libsdl.org
*/

/*
	Chunky to planar conversion routine
	1 byte/pixel -> 4 or 8 bit planes

	Patrice Mandin
	Xavier Joubert
	Mikael Kalms
*/

	.globl	_SDL_Atari_C2pConvert
	.globl	_SDL_Atari_C2pConvert8
	.globl	_SDL_Atari_C2pConvert4
	.globl	_SDL_Atari_C2pConvert4_pal

/* ------------	Conversion C2P, 8 bits ------------ */

	.text
_SDL_Atari_C2pConvert8:
#if !defined(__mcoldfire__)
	movel	sp@(4),c2p_source
	movel	sp@(8),c2p_dest
	movel	sp@(12),c2p_width
	movel	sp@(16),c2p_height
	movel	sp@(20),c2p_dblligne
	movel	sp@(24),c2p_srcpitch
	movel	sp@(28),c2p_dstpitch

	moveml	d2-d7/a2-a6,sp@-

	movel	c2p_source,c2p_cursrc
	movel	c2p_dest,c2p_curdst
	movel	#0x0f0f0f0f,d4
	movel	#0x00ff00ff,d5
	movel	#0x55555555,d6
	movew	c2p_height+2,c2p_row
	movew	c2p_width+2,d0
	andw	#-8,d0
	movew	d0,c2p_rowlen

SDL_Atari_C2p8_rowloop:

	movel	c2p_cursrc,a0
	movel	c2p_curdst,a1

	movel	a0,a2
	addw	c2p_rowlen,a2

	movel	a0@+,d0
	movel	a0@+,d1
	movel	a0@+,d2
	movel	a0@+,d3
/*
	d0 = a7a6a5a4a3a2a1a0 b7b6b5b4b3b2b1b0 c7c6c5c4c3c2c1c0 d7d6d5d4d3d2d1d0
	d1 = e7e6e5e4e3e2e1e0 f7f6f5f4f3f2f1f0 g7g6g5g4g3g2g1g0 h7h6h5h4h3h2h1h0
	d2 = i7i6i5i4i3i2i1i0 j7j6j5j4j3j2j1j0 k7k6k5k4k3k2k1k0 l7l6l5l4l3l2l1l0
	d3 = m7m6m5m4m3m2m1m0 n7n6n5n4n3n2n1n0 o7o6o5o4o3o2o1o0 p7p6p5p4p3p2p1p0
*/
	movel	d1,d7
	lsrl	#4,d7
	eorl	d0,d7
	andl	d4,d7
	eorl	d7,d0
	lsll	#4,d7
	eorl	d7,d1

	movel	d3,d7
	lsrl	#4,d7
	eorl	d2,d7
	andl	d4,d7
	eorl	d7,d2
	lsll	#4,d7
	eorl	d7,d3

	movel	d2,d7
	lsrl	#8,d7
	eorl	d0,d7
 	andl	d5,d7
	eorl	d7,d0
	lsll	#8,d7
	eorl	d7,d2

	movel	d3,d7
	lsrl	#8,d7
	eorl	d1,d7
 	andl	d5,d7
	eorl	d7,d1
	lsll	#8,d7
	eorl	d7,d3
/*
	d0 = a7a6a5a4e7e6e5e4 i7i6i5i4m7m6m5m4 c7c6c5c4g7g6g5g4 k7k6k5k4o7o6o5o4
	d1 = a3a2a1a0e3e2e1e0 i3i2i1i0m3m2m1m0 c3c2c1c0g3g2g1g0 k3k2k1k0o3o2o1o0
	d2 = b7b6b5b4f7f6f5f4 j7j6j5j4n7n6n5n4 d7d6d5d4h7h6h5h4 l7l6l5l4p7p6p5p4
	d3 = b3b2b1b0f3f2f1f0 j3j2j1j0n3n2n1n0 d3d2d1d0h3h2h1h0 l3l2l1l0p3p2p1p0
*/
	bras	SDL_Atari_C2p8_start

SDL_Atari_C2p8_pix16:

	movel	a0@+,d0
	movel	a0@+,d1
	movel	a0@+,d2
	movel	a0@+,d3
/*
	d0 = a7a6a5a4a3a2a1a0 b7b6b5b4b3b2b1b0 c7c6c5c4c3c2c1c0 d7d6d5d4d3d2d1d0
	d1 = e7e6e5e4e3e2e1e0 f7f6f5f4f3f2f1f0 g7g6g5g4g3g2g1g0 h7h6h5h4h3h2h1h0
	d2 = i7i6i5i4i3i2i1i0 j7j6j5j4j3j2j1j0 k7k6k5k4k3k2k1k0 l7l6l5l4l3l2l1l0
	d3 = m7m6m5m4m3m2m1m0 n7n6n5n4n3n2n1n0 o7o6o5o4o3o2o1o0 p7p6p5p4p3p2p1p0
*/
	movel	d1,d7
	lsrl	#4,d7
	movel	a3,a1@+
	eorl	d0,d7
	andl	d4,d7
	eorl	d7,d0
	lsll	#4,d7
	eorl	d7,d1

	movel	d3,d7
	lsrl	#4,d7
	eorl	d2,d7
	andl	d4,d7
	eorl	d7,d2
	movel	a4,a1@+
	lsll	#4,d7
	eorl	d7,d3

	movel	d2,d7
	lsrl	#8,d7
	eorl	d0,d7
	andl	d5,d7
	eorl	d7,d0
	movel	a5,a1@+
	lsll	#8,d7
	eorl	d7,d2

	movel	d3,d7
	lsrl	#8,d7
	eorl	d1,d7
	andl	d5,d7
	eorl	d7,d1
	movel	a6,a1@+
	lsll	#8,d7
	eorl	d7,d3
/*
	d0 = a7a6a5a4e7e6e5e4 i7i6i5i4m7m6m5m4 c7c6c5c4g7g6g5g4 k7k6k5k4o7o6o5o4
	d1 = a3a2a1a0e3e2e1e0 i3i2i1i0m3m2m1m0 c3c2c1c0g3g2g1g0 k3k2k1k0o3o2o1o0
	d2 = b7b6b5b4f7f6f5f4 j7j6j5j4n7n6n5n4 d7d6d5d4h7h6h5h4 l7l6l5l4p7p6p5p4
	d3 = b3b2b1b0f3f2f1f0 j3j2j1j0n3n2n1n0 d3d2d1d0h3h2h1h0 l3l2l1l0p3p2p1p0
*/

SDL_Atari_C2p8_start:

	movel	d2,d7
	lsrl	#1,d7
	eorl	d0,d7
	andl	d6,d7
	eorl	d7,d0
	addl	d7,d7
	eorl	d7,d2

	movel	d3,d7
	lsrl	#1,d7
	eorl	d1,d7
	andl	d6,d7
	eorl	d7,d1
	addl	d7,d7
	eorl	d7,d3
/*
	d0 = a7b7a5b5e7f7e5f5 i7j7i5j5m7n7m5n5 c7d7c5d5g7h7g5h5 k7l7k5l5o7p7o5p5
	d1 = a3b3a1b1e3f3e1f1 i3j3i1j1m3n3m1n1 c3d3c1d1g3h3g1h1 k3l3k1l1o3p3o1p1
	d2 = a6b6a4b4e6f6e4f4 i6j6i4j4m6n6m4n4 c6d6c4d4g6h6g4h4 k6l6k4l4o6p6o4p4
	d3 = a2b2a0b0e2f2e0f0 i2j2i0j0m2n2m0n0 c2d2c0d0g2h2g0h0 k2l2k0l0o2p2o0p0
*/
	movew	d2,d7
	movew	d0,d2
	swap	d2
	movew	d2,d0
	movew	d7,d2

	movew	d3,d7
	movew	d1,d3
	swap	d3
	movew	d3,d1
	movew	d7,d3
/*
	d0 = a7b7a5b5e7f7e5f5 i7j7i5j5m7n7m5n5 a6b6a4b4e6f6e4f4 i6j6i4j4m6n6m4n4
	d1 = a3b3a1b1e3f3e1f1 i3j3i1j1m3n3m1n1 a2b2a0b0e2f2e0f0 i2j2i0j0m2n2m0n0
	d2 = c7d7c5d5g7h7g5h5 k7l7k5l5o7p7o5p5 c6d6c4d4g6h6g4h4 k6l6k4l4o6p6o4p4
	d3 = c3d3c1d1g3h3g1h1 k3l3k1l1o3p3o1p1 c2d2c0d0g2h2g0h0 k2l2k0l0o2p2o0p0
*/
	movel	d2,d7
	lsrl	#2,d7
	eorl	d0,d7
	andl	#0x33333333,d7
	eorl	d7,d0
	lsll	#2,d7
	eorl	d7,d2

	movel	d3,d7
	lsrl	#2,d7
	eorl	d1,d7
	andl	#0x33333333,d7
	eorl	d7,d1
	lsll	#2,d7
	eorl	d7,d3
/*
	d0 = a7b7c7d7e7f7g7h7 i7j7k7l7m7n7o7p7 a6b6c6d6e6f6g6h6 i6j6k6l6m6n6o6p6
	d1 = a3b3c3d3e3f3g3h3 i3j3k3l3m3n3o3p3 a2b2c2d2e2f2g2h2 i2j2k2l2m2n2o2p2
	d2 = a5b5c5d5e5f5g5h5 i5j5k5l5m5n5o5p5 a4b4c4d4e4f4g4h4 i4j4k4l4m4n4o4p4
	d3 = a1b1c1d1e1f1g1h1 i1j1k1l1m1n1o1p1 a0b0c0d0e0f0g0h0 i0j0k0l0m0n0o0p0
*/
	swap	d0
	swap	d1
	swap	d2
	swap	d3

	movel	d0,a6
	movel	d2,a5
	movel	d1,a4
	movel	d3,a3

	cmpl	a0,a2
	bgt		SDL_Atari_C2p8_pix16

	movel	a3,a1@+
	movel	a4,a1@+
	movel	a5,a1@+
	movel	a6,a1@+

	/* Double the line ? */

	movel	c2p_srcpitch,d0
	movel	c2p_dstpitch,d1

	tstl	c2p_dblligne
	beqs	SDL_Atari_C2p8_nodblline

	movel	c2p_curdst,a0
	movel	a0,a1
	addl	d1,a1

	movew	c2p_width+2,d7
	lsrw	#4,d7
	subql	#1,d7
SDL_Atari_C2p8_dblloop:
	movel	a0@+,a1@+
	movel	a0@+,a1@+
	movel	a0@+,a1@+
	movel	a0@+,a1@+
	dbra	d7,SDL_Atari_C2p8_dblloop

	addl	d1,c2p_curdst

SDL_Atari_C2p8_nodblline:

	/* Next line */

	addl	d0,c2p_cursrc
	addl	d1,c2p_curdst

	subqw	#1,c2p_row
	bne		SDL_Atari_C2p8_rowloop	

	moveml	sp@+,d2-d7/a2-a6
#endif
	rts

/* ------------	Conversion C2P, 4 bits ------------ */

_SDL_Atari_C2pConvert4:
#if !defined(__mcoldfire__)
	movel	sp@(4),c2p_source
	movel	sp@(8),c2p_dest
	movel	sp@(12),c2p_width
	movel	sp@(16),c2p_height
	movel	sp@(20),c2p_dblligne
	movel	sp@(24),c2p_srcpitch
	movel	sp@(28),c2p_dstpitch

	moveml	d2-d7/a2-a6,sp@-

	movel	c2p_source,a0
	movel	c2p_dest,a1
	lea	_SDL_Atari_table_c2p,a2
	movel	#0x00070001,d3
#if defined(__mc68020__)
	moveq	#0,d0
#endif
	
	movel	c2p_height,d7
	subql	#1,d7
c2p4_bcly:
	movel	a0,a4	| Save start address of source
	movel	a1,a5	| Save start address of dest

	| Conversion
                 			
	movel	c2p_width,d6
	lsrw	#4,d6
	subql	#1,d6
c2p4_bclx:
	| Octets 0-7
	
	moveq	#0,d1
	moveq	#7,d5
c2p4_bcl07:
#if defined(__mc68020__)
	moveb	a0@+,d0
	lea		a2@(0,d0:w:4),a3
#else
	moveq	#0,d0
	moveb	a0@+,d0
	lslw	#2,d0
	lea		a2@(0,d0:w),a3
#endif
	lsll	#1,d1
	orl		a3@,d1
	dbra	d5,c2p4_bcl07

	movepl	d1,a1@(0)
	addw	d3,a1
	swap	d3
	
	| Octets 8-15

	moveq	#0,d1
	moveq	#7,d5
c2p4_bcl815:
#if defined(__mc68020__)
	moveb	a0@+,d0
	lea		a2@(0,d0:w:4),a3
#else
	moveq	#0,d0
	moveb	a0@+,d0
	lslw	#2,d0
	lea		a2@(0,d0:w),a3
#endif
	lsll	#1,d1
	orl		a3@,d1
	dbra	d5,c2p4_bcl815

	movepl	d1,a1@(0)
	addw	d3,a1
	swap	d3

	dbra	d6,c2p4_bclx

	| Double line ?

	tstl	c2p_dblligne
	beqs	c2p4_nodblligne

	movel	a5,a6			| src line
	movel	a5,a1			| dest line
	addl	c2p_dstpitch,a1

	movel	c2p_width,d6
	lsrw	#3,d6
	subql	#1,d6
c2p4_copydbl:
	movel	a6@+,a1@+
	dbra	d6,c2p4_copydbl

	addl	c2p_dstpitch,a5
c2p4_nodblligne:

	| Next line

	movel	a4,a0		
	addl	c2p_srcpitch,a0
	movel	a5,a1
	addl	c2p_dstpitch,a1

	dbra	d7,c2p4_bcly

	moveml	sp@+,d2-d7/a2-a6
#endif
	rts

/* ------------	Conversion of a light palette in 4 bits ------------ */

_SDL_Atari_C2pConvert4_pal:
#if !defined(__mcoldfire__)
	/* a0 is a 256-word light palette */
	movel	sp@(4),a0

	moveml	d2-d3,sp@-

	lea		_SDL_Atari_table_c2p,a1
	movew	#255,d3
c2p_pal_initbcl:
	movew	a0@+,d0
	lsrw	#4,d0
	andw	#15,d0

	moveq	#3,d1
c2p_pal_initbyte:
	btst	d1,d0
	sne		d2
	negw	d2
	moveb	d2,a1@(0,d1:w)

	dbra	d1,c2p_pal_initbyte

	addql	#4,a1
	dbra	d3,c2p_pal_initbcl

	moveml	sp@+,d2-d3
#endif
	rts

/* ------------	Buffers ------------ */

	.bss

	.even
	.comm	_SDL_Atari_C2pConvert,4
	.comm	_SDL_Atari_table_c2p,1024

	.comm	c2p_source,4	/* Source framebuffer */
	.comm	c2p_dest,4		/* Destination framebuffer */
	.comm	c2p_width,4		/* Width of zone to convert */
	.comm	c2p_height,4	/* Height of zone to convert */
	.comm	c2p_dblligne,4	/* Double the lines while converting ? */
	.comm	c2p_srcpitch,4	/* Source pitch */
	.comm	c2p_dstpitch,4	/* Destination pitch */
	.comm	c2p_cursrc,4	/* Current source line */
	.comm	c2p_curdst,4	/* Current destination line */
	.comm	c2p_rowlen,2	/* Line length in bytes */
	.comm	c2p_row,2		/* Current line number */
