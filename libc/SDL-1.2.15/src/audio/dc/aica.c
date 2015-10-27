/* This file is part of the Dreamcast function library.
 * Please see libdream.c for further details.
 *
 * (c)2000 Dan Potter
 * modify BERO
 */
#include "aica.h"

#include <arch/irq.h>
#include <dc/spu.h>

/* #define dc_snd_base ((volatile unsigned char *)0x00800000) */ /* arm side */
#define dc_snd_base ((volatile unsigned char *)0xa0700000) /* dc side */

/* Some convienence macros */
#define	SNDREGADDR(x)	(0xa0700000 + (x))
#define	CHNREGADDR(ch,x)	SNDREGADDR(0x80*(ch)+(x))


#define SNDREG32(x)	(*(volatile unsigned long *)SNDREGADDR(x))
#define SNDREG8(x)	(*(volatile unsigned char *)SNDREGADDR(x))
#define CHNREG32(ch, x) (*(volatile unsigned long *)CHNREGADDR(ch,x))
#define CHNREG8(ch, x)	(*(volatile unsigned long *)CHNREGADDR(ch,x))

#define G2_LOCK(OLD) \
	do { \
		if (!irq_inside_int()) \
			OLD = irq_disable(); \
		/* suspend any G2 DMA here... */ \
		while((*(volatile unsigned int *)0xa05f688c) & 0x20) \
			; \
	} while(0)

#define G2_UNLOCK(OLD) \
	do { \
		/* resume any G2 DMA here... */ \
		if (!irq_inside_int()) \
			irq_restore(OLD); \
	} while(0)


void aica_init() {
	int i, j, old = 0;
	
	/* Initialize AICA channels */	
	G2_LOCK(old);
	SNDREG32(0x2800) = 0x0000;
	
	for (i=0; i<64; i++) {
		for (j=0; j<0x80; j+=4) {
			if ((j&31)==0) g2_fifo_wait();
			CHNREG32(i, j) = 0;
		}
		g2_fifo_wait();
		CHNREG32(i,0) = 0x8000;
		CHNREG32(i,20) = 0x1f;
	}

	SNDREG32(0x2800) = 0x000f;
	g2_fifo_wait();
	G2_UNLOCK(old);
}

/* Translates a volume from linear form to logarithmic form (required by
   the AICA chip */
/* int logs[] = {

0, 40, 50, 58, 63, 68, 73, 77, 80, 83, 86, 89, 92, 94, 97, 99, 101, 103,
105, 107, 109, 111, 112, 114, 116, 117, 119, 120, 122, 123, 125, 126, 127,
129, 130, 131, 133, 134, 135, 136, 137, 139, 140, 141, 142, 143, 144, 145,
146, 147, 148, 149, 150, 151, 152, 153, 154, 155, 156, 156, 157, 158, 159,
160, 161, 162, 162, 163, 164, 165, 166, 166, 167, 168, 169, 170, 170, 171,
172, 172, 173, 174, 175, 175, 176, 177, 177, 178, 179, 180, 180, 181, 182,
182, 183, 183, 184, 185, 185, 186, 187, 187, 188, 188, 189, 190, 190, 191,
191, 192, 193, 193, 194, 194, 195, 196, 196, 197, 197, 198, 198, 199, 199,
200, 201, 201, 202, 202, 203, 203, 204, 204, 205, 205, 206, 206, 207, 207,
208, 208, 209, 209, 210, 210, 211, 211, 212, 212, 213, 213, 214, 214, 215,
215, 216, 216, 217, 217, 217, 218, 218, 219, 219, 220, 220, 221, 221, 222,
222, 222, 223, 223, 224, 224, 225, 225, 225, 226, 226, 227, 227, 228, 228,
228, 229, 229, 230, 230, 230, 231, 231, 232, 232, 232, 233, 233, 234, 234,
234, 235, 235, 236, 236, 236, 237, 237, 238, 238, 238, 239, 239, 240, 240,
240, 241, 241, 241, 242, 242, 243, 243, 243, 244, 244, 244, 245, 245, 245,
246, 246, 247, 247, 247, 248, 248, 248, 249, 249, 249, 250, 250, 250, 251,
251, 251, 252, 252, 252, 253, 253, 253, 254, 254, 254, 255

}; */

const static unsigned char logs[] = {
	0, 15, 22, 27, 31, 35, 39, 42, 45, 47, 50, 52, 55, 57, 59, 61,
	63, 65, 67, 69, 71, 73, 74, 76, 78, 79, 81, 82, 84, 85, 87, 88,
	90, 91, 92, 94, 95, 96, 98, 99, 100, 102, 103, 104, 105, 106,
	108, 109, 110, 111, 112, 113, 114, 116, 117, 118, 119, 120, 121,
	122, 123, 124, 125, 126, 127, 128, 129, 130, 131, 132, 133, 134,
	135, 136, 137, 138, 138, 139, 140, 141, 142, 143, 144, 145, 146,
	146, 147, 148, 149, 150, 151, 152, 152, 153, 154, 155, 156, 156,
	157, 158, 159, 160, 160, 161, 162, 163, 164, 164, 165, 166, 167,
	167, 168, 169, 170, 170, 171, 172, 173, 173, 174, 175, 176, 176,
	177, 178, 178, 179, 180, 181, 181, 182, 183, 183, 184, 185, 185,
	186, 187, 187, 188, 189, 189, 190, 191, 191, 192, 193, 193, 194,
	195, 195, 196, 197, 197, 198, 199, 199, 200, 200, 201, 202, 202,
	203, 204, 204, 205, 205, 206, 207, 207, 208, 209, 209, 210, 210,
	211, 212, 212, 213, 213, 214, 215, 215, 216, 216, 217, 217, 218,
	219, 219, 220, 220, 221, 221, 222, 223, 223, 224, 224, 225, 225,
	226, 227, 227, 228, 228, 229, 229, 230, 230, 231, 232, 232, 233,
	233, 234, 234, 235, 235, 236, 236, 237, 237, 238, 239, 239, 240,
	240, 241, 241, 242, 242, 243, 243, 244, 244, 245, 245, 246, 246,
	247, 247, 248, 248, 249, 249, 250, 250, 251, 251, 252, 252, 253, 254, 255
};

/* For the moment this is going to have to suffice, until we really
   figure out what these mean. */
#define AICA_PAN(x) ((x)==0x80?(0):((x)<0x80?(0x1f):(0x0f)))
#define AICA_VOL(x) (0xff - logs[128 + (((x) & 0xff) / 2)])
//#define AICA_VOL(x) (0xff - logs[x&255])

static inline unsigned  AICA_FREQ(unsigned freq)	{
	unsigned long freq_lo, freq_base = 5644800;
	int freq_hi = 7;

	/* Need to convert frequency to floating point format
	   (freq_hi is exponent, freq_lo is mantissa)
	   Formula is ferq = 44100*2^freq_hi*(1+freq_lo/1024) */
	while (freq < freq_base && freq_hi > -8) {
		freq_base >>= 1;
		--freq_hi;
	}
	while (freq < freq_base && freq_hi > -8) {
		freq_base >>= 1;
		freq_hi--;
	}
	freq_lo = (freq<<10) / freq_base;
	return (freq_hi << 11) | (freq_lo & 1023);
}

/* Sets up a sound channel completely. This is generally good if you want
   a quick and dirty way to play notes. If you want a more comprehensive
   set of routines (more like PC wavetable cards) see below.
   
   ch is the channel to play on (0 - 63)
   smpptr is the pointer to the sound data; if you're running off the
     SH4, then this ought to be (ptr - 0xa0800000); otherwise it's just
     ptr. Basically, it's an offset into sound ram.
   mode is one of the mode constants (16 bit, 8 bit, ADPCM)
   nsamp is the number of samples to play (not number of bytes!)
   freq is the sampling rate of the sound
   vol is the volume, 0 to 0xff (0xff is louder)
   pan is a panning constant -- 0 is left, 128 is center, 255 is right.

   This routine (and the similar ones) owe a lot to Marcus' sound example -- 
   I hadn't gotten quite this far into dissecting the individual regs yet. */
void aica_play(int ch,int mode,unsigned long smpptr,int loopst,int loopend,int freq,int vol,int pan,int loopflag) {
/*	int i;
*/
	int val;
	int old = 0;

	/* Stop the channel (if it's already playing) */
	aica_stop(ch);
	/* doesn't seem to be needed, but it's here just in case */
/*
	for (i=0; i<256; i++) {
		asm("nop");
		asm("nop");
		asm("nop");
		asm("nop");
	}
*/
	G2_LOCK(old);
	/* Envelope setup. The first of these is the loop point,
	   e.g., where the sample starts over when it loops. The second
	   is the loop end. This is the full length of the sample when
	   you are not looping, or the loop end point when you are (though
	   storing more than that is a waste of memory if you're not doing
	   volume enveloping). */
	CHNREG32(ch, 8) = loopst & 0xffff;
	CHNREG32(ch, 12) = loopend & 0xffff;
	
	/* Write resulting values */
	CHNREG32(ch, 24) = AICA_FREQ(freq);
	
	/* Set volume, pan, and some other things that we don't know what
	   they do =) */
	CHNREG32(ch, 36) = AICA_PAN(pan) | (0xf<<8);
	/* Convert the incoming volume and pan into hardware values */
	/* Vol starts at zero so we can ramp */
	vol = AICA_VOL(vol);
	CHNREG32(ch, 40) = 0x24 | (vol<<8);
	/* Convert the incoming volume and pan into hardware values */
	/* Vol starts at zero so we can ramp */

	/* If we supported volume envelopes (which we don't yet) then
	   this value would set that up. The top 4 bits determine the
	   envelope speed. f is the fastest, 1 is the slowest, and 0
	   seems to be an invalid value and does weird things). The
	   default (below) sets it into normal mode (play and terminate/loop).
	CHNREG32(ch, 16) = 0xf010;
	*/
	CHNREG32(ch, 16) = 0x1f;	/* No volume envelope */
	
	
	/* Set sample format, buffer address, and looping control. If
	   0x0200 mask is set on reg 0, the sample loops infinitely. If
	   it's not set, the sample plays once and terminates. We'll
	   also set the bits to start playback here. */
	CHNREG32(ch, 4) = smpptr & 0xffff;
	val = 0xc000 | 0x0000 | (mode<<7) | (smpptr >> 16);
	if (loopflag) val|=0x200;
	
	CHNREG32(ch, 0) = val;
	
	G2_UNLOCK(old);

	/* Enable playback */
	/* CHNREG32(ch, 0) |= 0xc000; */
	g2_fifo_wait();

#if 0
	for (i=0xff; i>=vol; i--) {
		if ((i&7)==0) g2_fifo_wait();
		CHNREG32(ch, 40) =  0x24 | (i<<8);;
	}

	g2_fifo_wait();
#endif
}

/* Stop the sound on a given channel */
void aica_stop(int ch) {
	g2_write_32(CHNREGADDR(ch, 0),(g2_read_32(CHNREGADDR(ch, 0)) & ~0x4000) | 0x8000);
	g2_fifo_wait();
}


/* The rest of these routines can change the channel in mid-stride so you
   can do things like vibrato and panning effects. */
   
/* Set channel volume */
void aica_vol(int ch,int vol) {
//	g2_write_8(CHNREGADDR(ch, 41),AICA_VOL(vol));
	g2_write_32(CHNREGADDR(ch, 40),(g2_read_32(CHNREGADDR(ch, 40))&0xffff00ff)|(AICA_VOL(vol)<<8) );
	g2_fifo_wait();
}

/* Set channel pan */
void aica_pan(int ch,int pan) {
//	g2_write_8(CHNREGADDR(ch, 36),AICA_PAN(pan));
	g2_write_32(CHNREGADDR(ch, 36),(g2_read_32(CHNREGADDR(ch, 36))&0xffffff00)|(AICA_PAN(pan)) );
	g2_fifo_wait();
}

/* Set channel frequency */
void aica_freq(int ch,int freq) {
	g2_write_32(CHNREGADDR(ch, 24),AICA_FREQ(freq));
	g2_fifo_wait();
}

/* Get channel position */
int aica_get_pos(int ch) {
#if 1
	/* Observe channel ch */
	g2_write_32(SNDREGADDR(0x280c),(g2_read_32(SNDREGADDR(0x280c))&0xffff00ff) | (ch<<8));
	g2_fifo_wait();
	/* Update position counters */
	return g2_read_32(SNDREGADDR(0x2814)) & 0xffff;
#else
	/* Observe channel ch */
	g2_write_8(SNDREGADDR(0x280d),ch);
	/* Update position counters */
	return g2_read_32(SNDREGADDR(0x2814)) & 0xffff;
#endif
}
