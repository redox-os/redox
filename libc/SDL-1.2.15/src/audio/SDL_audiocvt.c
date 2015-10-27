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
#include "SDL_config.h"

/* Functions for audio drivers to perform runtime conversion of audio format */

#include "SDL_audio.h"


/* Effectively mix right and left channels into a single channel */
void SDLCALL SDL_ConvertMono(SDL_AudioCVT *cvt, Uint16 format)
{
	int i;
	Sint32 sample;

#ifdef DEBUG_CONVERT
	fprintf(stderr, "Converting to mono\n");
#endif
	switch (format&0x8018) {

		case AUDIO_U8: {
			Uint8 *src, *dst;

			src = cvt->buf;
			dst = cvt->buf;
			for ( i=cvt->len_cvt/2; i; --i ) {
				sample = src[0] + src[1];
				*dst = (Uint8)(sample / 2);
				src += 2;
				dst += 1;
			}
		}
		break;

		case AUDIO_S8: {
			Sint8 *src, *dst;

			src = (Sint8 *)cvt->buf;
			dst = (Sint8 *)cvt->buf;
			for ( i=cvt->len_cvt/2; i; --i ) {
				sample = src[0] + src[1];
				*dst = (Sint8)(sample / 2);
				src += 2;
				dst += 1;
			}
		}
		break;

		case AUDIO_U16: {
			Uint8 *src, *dst;

			src = cvt->buf;
			dst = cvt->buf;
			if ( (format & 0x1000) == 0x1000 ) {
				for ( i=cvt->len_cvt/4; i; --i ) {
					sample = (Uint16)((src[0]<<8)|src[1])+
					         (Uint16)((src[2]<<8)|src[3]);
					sample /= 2;
					dst[1] = (sample&0xFF);
					sample >>= 8;
					dst[0] = (sample&0xFF);
					src += 4;
					dst += 2;
				}
			} else {
				for ( i=cvt->len_cvt/4; i; --i ) {
					sample = (Uint16)((src[1]<<8)|src[0])+
					         (Uint16)((src[3]<<8)|src[2]);
					sample /= 2;
					dst[0] = (sample&0xFF);
					sample >>= 8;
					dst[1] = (sample&0xFF);
					src += 4;
					dst += 2;
				}
			}
		}
		break;

		case AUDIO_S16: {
			Uint8 *src, *dst;

			src = cvt->buf;
			dst = cvt->buf;
			if ( (format & 0x1000) == 0x1000 ) {
				for ( i=cvt->len_cvt/4; i; --i ) {
					sample = (Sint16)((src[0]<<8)|src[1])+
					         (Sint16)((src[2]<<8)|src[3]);
					sample /= 2;
					dst[1] = (sample&0xFF);
					sample >>= 8;
					dst[0] = (sample&0xFF);
					src += 4;
					dst += 2;
				}
			} else {
				for ( i=cvt->len_cvt/4; i; --i ) {
					sample = (Sint16)((src[1]<<8)|src[0])+
					         (Sint16)((src[3]<<8)|src[2]);
					sample /= 2;
					dst[0] = (sample&0xFF);
					sample >>= 8;
					dst[1] = (sample&0xFF);
					src += 4;
					dst += 2;
				}
			}
		}
		break;
	}
	cvt->len_cvt /= 2;
	if ( cvt->filters[++cvt->filter_index] ) {
		cvt->filters[cvt->filter_index](cvt, format);
	}
}

/* Discard top 4 channels */
void SDLCALL SDL_ConvertStrip(SDL_AudioCVT *cvt, Uint16 format)
{
	int i;
	Sint32 lsample, rsample;

#ifdef DEBUG_CONVERT
	fprintf(stderr, "Converting down to stereo\n");
#endif
	switch (format&0x8018) {

		case AUDIO_U8: {
			Uint8 *src, *dst;

			src = cvt->buf;
			dst = cvt->buf;
			for ( i=cvt->len_cvt/6; i; --i ) {
				dst[0] = src[0];
				dst[1] = src[1];
				src += 6;
				dst += 2;
			}
		}
		break;

		case AUDIO_S8: {
			Sint8 *src, *dst;

			src = (Sint8 *)cvt->buf;
			dst = (Sint8 *)cvt->buf;
			for ( i=cvt->len_cvt/6; i; --i ) {
				dst[0] = src[0];
				dst[1] = src[1];
				src += 6;
				dst += 2;
			}
		}
		break;

		case AUDIO_U16: {
			Uint8 *src, *dst;

			src = cvt->buf;
			dst = cvt->buf;
			if ( (format & 0x1000) == 0x1000 ) {
				for ( i=cvt->len_cvt/12; i; --i ) {
					lsample = (Uint16)((src[0]<<8)|src[1]);
					rsample = (Uint16)((src[2]<<8)|src[3]);
						dst[1] = (lsample&0xFF);
						lsample >>= 8;
						dst[0] = (lsample&0xFF);
						dst[3] = (rsample&0xFF);
						rsample >>= 8;
						dst[2] = (rsample&0xFF);
					src += 12;
					dst += 4;
				}
			} else {
				for ( i=cvt->len_cvt/12; i; --i ) {
					lsample = (Uint16)((src[1]<<8)|src[0]);
					rsample = (Uint16)((src[3]<<8)|src[2]);
						dst[0] = (lsample&0xFF);
						lsample >>= 8;
						dst[1] = (lsample&0xFF);
						dst[2] = (rsample&0xFF);
						rsample >>= 8;
						dst[3] = (rsample&0xFF);
					src += 12;
					dst += 4;
				}
			}
		}
		break;

		case AUDIO_S16: {
			Uint8 *src, *dst;

			src = cvt->buf;
			dst = cvt->buf;
			if ( (format & 0x1000) == 0x1000 ) {
				for ( i=cvt->len_cvt/12; i; --i ) {
					lsample = (Sint16)((src[0]<<8)|src[1]);
					rsample = (Sint16)((src[2]<<8)|src[3]);
						dst[1] = (lsample&0xFF);
						lsample >>= 8;
						dst[0] = (lsample&0xFF);
						dst[3] = (rsample&0xFF);
						rsample >>= 8;
						dst[2] = (rsample&0xFF);
					src += 12;
					dst += 4;
				}
			} else {
				for ( i=cvt->len_cvt/12; i; --i ) {
					lsample = (Sint16)((src[1]<<8)|src[0]);
					rsample = (Sint16)((src[3]<<8)|src[2]);
						dst[0] = (lsample&0xFF);
						lsample >>= 8;
						dst[1] = (lsample&0xFF);
						dst[2] = (rsample&0xFF);
						rsample >>= 8;
						dst[3] = (rsample&0xFF);
					src += 12;
					dst += 4;
				}
			}
		}
		break;
	}
	cvt->len_cvt /= 3;
	if ( cvt->filters[++cvt->filter_index] ) {
		cvt->filters[cvt->filter_index](cvt, format);
	}
}


/* Discard top 2 channels of 6 */
void SDLCALL SDL_ConvertStrip_2(SDL_AudioCVT *cvt, Uint16 format)
{
	int i;
	Sint32 lsample, rsample;

#ifdef DEBUG_CONVERT
	fprintf(stderr, "Converting 6 down to quad\n");
#endif
	switch (format&0x8018) {

		case AUDIO_U8: {
			Uint8 *src, *dst;

			src = cvt->buf;
			dst = cvt->buf;
			for ( i=cvt->len_cvt/4; i; --i ) {
				dst[0] = src[0];
				dst[1] = src[1];
				src += 4;
				dst += 2;
			}
		}
		break;

		case AUDIO_S8: {
			Sint8 *src, *dst;

			src = (Sint8 *)cvt->buf;
			dst = (Sint8 *)cvt->buf;
			for ( i=cvt->len_cvt/4; i; --i ) {
				dst[0] = src[0];
				dst[1] = src[1];
				src += 4;
				dst += 2;
			}
		}
		break;

		case AUDIO_U16: {
			Uint8 *src, *dst;

			src = cvt->buf;
			dst = cvt->buf;
			if ( (format & 0x1000) == 0x1000 ) {
				for ( i=cvt->len_cvt/8; i; --i ) {
					lsample = (Uint16)((src[0]<<8)|src[1]);
					rsample = (Uint16)((src[2]<<8)|src[3]);
						dst[1] = (lsample&0xFF);
						lsample >>= 8;
						dst[0] = (lsample&0xFF);
						dst[3] = (rsample&0xFF);
						rsample >>= 8;
						dst[2] = (rsample&0xFF);
					src += 8;
					dst += 4;
				}
			} else {
				for ( i=cvt->len_cvt/8; i; --i ) {
					lsample = (Uint16)((src[1]<<8)|src[0]);
					rsample = (Uint16)((src[3]<<8)|src[2]);
						dst[0] = (lsample&0xFF);
						lsample >>= 8;
						dst[1] = (lsample&0xFF);
						dst[2] = (rsample&0xFF);
						rsample >>= 8;
						dst[3] = (rsample&0xFF);
					src += 8;
					dst += 4;
				}
			}
		}
		break;

		case AUDIO_S16: {
			Uint8 *src, *dst;

			src = cvt->buf;
			dst = cvt->buf;
			if ( (format & 0x1000) == 0x1000 ) {
				for ( i=cvt->len_cvt/8; i; --i ) {
					lsample = (Sint16)((src[0]<<8)|src[1]);
					rsample = (Sint16)((src[2]<<8)|src[3]);
						dst[1] = (lsample&0xFF);
						lsample >>= 8;
						dst[0] = (lsample&0xFF);
						dst[3] = (rsample&0xFF);
						rsample >>= 8;
						dst[2] = (rsample&0xFF);
					src += 8;
					dst += 4;
				}
			} else {
				for ( i=cvt->len_cvt/8; i; --i ) {
					lsample = (Sint16)((src[1]<<8)|src[0]);
					rsample = (Sint16)((src[3]<<8)|src[2]);
						dst[0] = (lsample&0xFF);
						lsample >>= 8;
						dst[1] = (lsample&0xFF);
						dst[2] = (rsample&0xFF);
						rsample >>= 8;
						dst[3] = (rsample&0xFF);
					src += 8;
					dst += 4;
				}
			}
		}
		break;
	}
	cvt->len_cvt /= 2;
	if ( cvt->filters[++cvt->filter_index] ) {
		cvt->filters[cvt->filter_index](cvt, format);
	}
}

/* Duplicate a mono channel to both stereo channels */
void SDLCALL SDL_ConvertStereo(SDL_AudioCVT *cvt, Uint16 format)
{
	int i;

#ifdef DEBUG_CONVERT
	fprintf(stderr, "Converting to stereo\n");
#endif
	if ( (format & 0xFF) == 16 ) {
		Uint16 *src, *dst;

		src = (Uint16 *)(cvt->buf+cvt->len_cvt);
		dst = (Uint16 *)(cvt->buf+cvt->len_cvt*2);
		for ( i=cvt->len_cvt/2; i; --i ) {
			dst -= 2;
			src -= 1;
			dst[0] = src[0];
			dst[1] = src[0];
		}
	} else {
		Uint8 *src, *dst;

		src = cvt->buf+cvt->len_cvt;
		dst = cvt->buf+cvt->len_cvt*2;
		for ( i=cvt->len_cvt; i; --i ) {
			dst -= 2;
			src -= 1;
			dst[0] = src[0];
			dst[1] = src[0];
		}
	}
	cvt->len_cvt *= 2;
	if ( cvt->filters[++cvt->filter_index] ) {
		cvt->filters[cvt->filter_index](cvt, format);
	}
}


/* Duplicate a stereo channel to a pseudo-5.1 stream */
void SDLCALL SDL_ConvertSurround(SDL_AudioCVT *cvt, Uint16 format)
{
	int i;

#ifdef DEBUG_CONVERT
	fprintf(stderr, "Converting stereo to surround\n");
#endif
	switch (format&0x8018) {

		case AUDIO_U8: {
			Uint8 *src, *dst, lf, rf, ce;

			src = (Uint8 *)(cvt->buf+cvt->len_cvt);
			dst = (Uint8 *)(cvt->buf+cvt->len_cvt*3);
			for ( i=cvt->len_cvt; i; --i ) {
				dst -= 6;
				src -= 2;
				lf = src[0];
				rf = src[1];
				ce = (lf/2) + (rf/2);
				dst[0] = lf;
				dst[1] = rf;
				dst[2] = lf - ce;
				dst[3] = rf - ce;
				dst[4] = ce;
				dst[5] = ce;
			}
		}
		break;

		case AUDIO_S8: {
			Sint8 *src, *dst, lf, rf, ce;

			src = (Sint8 *)cvt->buf+cvt->len_cvt;
			dst = (Sint8 *)cvt->buf+cvt->len_cvt*3;
			for ( i=cvt->len_cvt; i; --i ) {
				dst -= 6;
				src -= 2;
				lf = src[0];
				rf = src[1];
				ce = (lf/2) + (rf/2);
				dst[0] = lf;
				dst[1] = rf;
				dst[2] = lf - ce;
				dst[3] = rf - ce;
				dst[4] = ce;
				dst[5] = ce;
			}
		}
		break;

		case AUDIO_U16: {
			Uint8 *src, *dst;
			Uint16 lf, rf, ce, lr, rr;

			src = cvt->buf+cvt->len_cvt;
			dst = cvt->buf+cvt->len_cvt*3;

			if ( (format & 0x1000) == 0x1000 ) {
				for ( i=cvt->len_cvt/4; i; --i ) {
					dst -= 12;
					src -= 4;
					lf = (Uint16)((src[0]<<8)|src[1]);
					rf = (Uint16)((src[2]<<8)|src[3]);
					ce = (lf/2) + (rf/2);
					rr = lf - ce;
					lr = rf - ce;
						dst[1] = (lf&0xFF);
						dst[0] = ((lf>>8)&0xFF);
						dst[3] = (rf&0xFF);
						dst[2] = ((rf>>8)&0xFF);

						dst[1+4] = (lr&0xFF);
						dst[0+4] = ((lr>>8)&0xFF);
						dst[3+4] = (rr&0xFF);
						dst[2+4] = ((rr>>8)&0xFF);

						dst[1+8] = (ce&0xFF);
						dst[0+8] = ((ce>>8)&0xFF);
						dst[3+8] = (ce&0xFF);
						dst[2+8] = ((ce>>8)&0xFF);
				}
			} else {
				for ( i=cvt->len_cvt/4; i; --i ) {
					dst -= 12;
					src -= 4;
					lf = (Uint16)((src[1]<<8)|src[0]);
					rf = (Uint16)((src[3]<<8)|src[2]);
					ce = (lf/2) + (rf/2);
					rr = lf - ce;
					lr = rf - ce;
						dst[0] = (lf&0xFF);
						dst[1] = ((lf>>8)&0xFF);
						dst[2] = (rf&0xFF);
						dst[3] = ((rf>>8)&0xFF);

						dst[0+4] = (lr&0xFF);
						dst[1+4] = ((lr>>8)&0xFF);
						dst[2+4] = (rr&0xFF);
						dst[3+4] = ((rr>>8)&0xFF);

						dst[0+8] = (ce&0xFF);
						dst[1+8] = ((ce>>8)&0xFF);
						dst[2+8] = (ce&0xFF);
						dst[3+8] = ((ce>>8)&0xFF);
				}
			}
		}
		break;

		case AUDIO_S16: {
			Uint8 *src, *dst;
			Sint16 lf, rf, ce, lr, rr;

			src = cvt->buf+cvt->len_cvt;
			dst = cvt->buf+cvt->len_cvt*3;

			if ( (format & 0x1000) == 0x1000 ) {
				for ( i=cvt->len_cvt/4; i; --i ) {
					dst -= 12;
					src -= 4;
					lf = (Sint16)((src[0]<<8)|src[1]);
					rf = (Sint16)((src[2]<<8)|src[3]);
					ce = (lf/2) + (rf/2);
					rr = lf - ce;
					lr = rf - ce;
						dst[1] = (lf&0xFF);
						dst[0] = ((lf>>8)&0xFF);
						dst[3] = (rf&0xFF);
						dst[2] = ((rf>>8)&0xFF);

						dst[1+4] = (lr&0xFF);
						dst[0+4] = ((lr>>8)&0xFF);
						dst[3+4] = (rr&0xFF);
						dst[2+4] = ((rr>>8)&0xFF);

						dst[1+8] = (ce&0xFF);
						dst[0+8] = ((ce>>8)&0xFF);
						dst[3+8] = (ce&0xFF);
						dst[2+8] = ((ce>>8)&0xFF);
				}
			} else {
				for ( i=cvt->len_cvt/4; i; --i ) {
					dst -= 12;
					src -= 4;
					lf = (Sint16)((src[1]<<8)|src[0]);
					rf = (Sint16)((src[3]<<8)|src[2]);
					ce = (lf/2) + (rf/2);
					rr = lf - ce;
					lr = rf - ce;
						dst[0] = (lf&0xFF);
						dst[1] = ((lf>>8)&0xFF);
						dst[2] = (rf&0xFF);
						dst[3] = ((rf>>8)&0xFF);

						dst[0+4] = (lr&0xFF);
						dst[1+4] = ((lr>>8)&0xFF);
						dst[2+4] = (rr&0xFF);
						dst[3+4] = ((rr>>8)&0xFF);

						dst[0+8] = (ce&0xFF);
						dst[1+8] = ((ce>>8)&0xFF);
						dst[2+8] = (ce&0xFF);
						dst[3+8] = ((ce>>8)&0xFF);
				}
			}
		}
		break;
	}
	cvt->len_cvt *= 3;
	if ( cvt->filters[++cvt->filter_index] ) {
		cvt->filters[cvt->filter_index](cvt, format);
	}
}


/* Duplicate a stereo channel to a pseudo-4.0 stream */
void SDLCALL SDL_ConvertSurround_4(SDL_AudioCVT *cvt, Uint16 format)
{
	int i;

#ifdef DEBUG_CONVERT
	fprintf(stderr, "Converting stereo to quad\n");
#endif
	switch (format&0x8018) {

		case AUDIO_U8: {
			Uint8 *src, *dst, lf, rf, ce;

			src = (Uint8 *)(cvt->buf+cvt->len_cvt);
			dst = (Uint8 *)(cvt->buf+cvt->len_cvt*2);
			for ( i=cvt->len_cvt; i; --i ) {
				dst -= 4;
				src -= 2;
				lf = src[0];
				rf = src[1];
				ce = (lf/2) + (rf/2);
				dst[0] = lf;
				dst[1] = rf;
				dst[2] = lf - ce;
				dst[3] = rf - ce;
			}
		}
		break;

		case AUDIO_S8: {
			Sint8 *src, *dst, lf, rf, ce;

			src = (Sint8 *)cvt->buf+cvt->len_cvt;
			dst = (Sint8 *)cvt->buf+cvt->len_cvt*2;
			for ( i=cvt->len_cvt; i; --i ) {
				dst -= 4;
				src -= 2;
				lf = src[0];
				rf = src[1];
				ce = (lf/2) + (rf/2);
				dst[0] = lf;
				dst[1] = rf;
				dst[2] = lf - ce;
				dst[3] = rf - ce;
			}
		}
		break;

		case AUDIO_U16: {
			Uint8 *src, *dst;
			Uint16 lf, rf, ce, lr, rr;

			src = cvt->buf+cvt->len_cvt;
			dst = cvt->buf+cvt->len_cvt*2;

			if ( (format & 0x1000) == 0x1000 ) {
				for ( i=cvt->len_cvt/4; i; --i ) {
					dst -= 8;
					src -= 4;
					lf = (Uint16)((src[0]<<8)|src[1]);
					rf = (Uint16)((src[2]<<8)|src[3]);
					ce = (lf/2) + (rf/2);
					rr = lf - ce;
					lr = rf - ce;
						dst[1] = (lf&0xFF);
						dst[0] = ((lf>>8)&0xFF);
						dst[3] = (rf&0xFF);
						dst[2] = ((rf>>8)&0xFF);

						dst[1+4] = (lr&0xFF);
						dst[0+4] = ((lr>>8)&0xFF);
						dst[3+4] = (rr&0xFF);
						dst[2+4] = ((rr>>8)&0xFF);
				}
			} else {
				for ( i=cvt->len_cvt/4; i; --i ) {
					dst -= 8;
					src -= 4;
					lf = (Uint16)((src[1]<<8)|src[0]);
					rf = (Uint16)((src[3]<<8)|src[2]);
					ce = (lf/2) + (rf/2);
					rr = lf - ce;
					lr = rf - ce;
						dst[0] = (lf&0xFF);
						dst[1] = ((lf>>8)&0xFF);
						dst[2] = (rf&0xFF);
						dst[3] = ((rf>>8)&0xFF);

						dst[0+4] = (lr&0xFF);
						dst[1+4] = ((lr>>8)&0xFF);
						dst[2+4] = (rr&0xFF);
						dst[3+4] = ((rr>>8)&0xFF);
				}
			}
		}
		break;

		case AUDIO_S16: {
			Uint8 *src, *dst;
			Sint16 lf, rf, ce, lr, rr;

			src = cvt->buf+cvt->len_cvt;
			dst = cvt->buf+cvt->len_cvt*2;

			if ( (format & 0x1000) == 0x1000 ) {
				for ( i=cvt->len_cvt/4; i; --i ) {
					dst -= 8;
					src -= 4;
					lf = (Sint16)((src[0]<<8)|src[1]);
					rf = (Sint16)((src[2]<<8)|src[3]);
					ce = (lf/2) + (rf/2);
					rr = lf - ce;
					lr = rf - ce;
						dst[1] = (lf&0xFF);
						dst[0] = ((lf>>8)&0xFF);
						dst[3] = (rf&0xFF);
						dst[2] = ((rf>>8)&0xFF);

						dst[1+4] = (lr&0xFF);
						dst[0+4] = ((lr>>8)&0xFF);
						dst[3+4] = (rr&0xFF);
						dst[2+4] = ((rr>>8)&0xFF);
				}
			} else {
				for ( i=cvt->len_cvt/4; i; --i ) {
					dst -= 8;
					src -= 4;
					lf = (Sint16)((src[1]<<8)|src[0]);
					rf = (Sint16)((src[3]<<8)|src[2]);
					ce = (lf/2) + (rf/2);
					rr = lf - ce;
					lr = rf - ce;
						dst[0] = (lf&0xFF);
						dst[1] = ((lf>>8)&0xFF);
						dst[2] = (rf&0xFF);
						dst[3] = ((rf>>8)&0xFF);

						dst[0+4] = (lr&0xFF);
						dst[1+4] = ((lr>>8)&0xFF);
						dst[2+4] = (rr&0xFF);
						dst[3+4] = ((rr>>8)&0xFF);
				}
			}
		}
		break;
	}
	cvt->len_cvt *= 2;
	if ( cvt->filters[++cvt->filter_index] ) {
		cvt->filters[cvt->filter_index](cvt, format);
	}
}


/* Convert 8-bit to 16-bit - LSB */
void SDLCALL SDL_Convert16LSB(SDL_AudioCVT *cvt, Uint16 format)
{
	int i;
	Uint8 *src, *dst;

#ifdef DEBUG_CONVERT
	fprintf(stderr, "Converting to 16-bit LSB\n");
#endif
	src = cvt->buf+cvt->len_cvt;
	dst = cvt->buf+cvt->len_cvt*2;
	for ( i=cvt->len_cvt; i; --i ) {
		src -= 1;
		dst -= 2;
		dst[1] = *src;
		dst[0] = 0;
	}
	format = ((format & ~0x0008) | AUDIO_U16LSB);
	cvt->len_cvt *= 2;
	if ( cvt->filters[++cvt->filter_index] ) {
		cvt->filters[cvt->filter_index](cvt, format);
	}
}
/* Convert 8-bit to 16-bit - MSB */
void SDLCALL SDL_Convert16MSB(SDL_AudioCVT *cvt, Uint16 format)
{
	int i;
	Uint8 *src, *dst;

#ifdef DEBUG_CONVERT
	fprintf(stderr, "Converting to 16-bit MSB\n");
#endif
	src = cvt->buf+cvt->len_cvt;
	dst = cvt->buf+cvt->len_cvt*2;
	for ( i=cvt->len_cvt; i; --i ) {
		src -= 1;
		dst -= 2;
		dst[0] = *src;
		dst[1] = 0;
	}
	format = ((format & ~0x0008) | AUDIO_U16MSB);
	cvt->len_cvt *= 2;
	if ( cvt->filters[++cvt->filter_index] ) {
		cvt->filters[cvt->filter_index](cvt, format);
	}
}

/* Convert 16-bit to 8-bit */
void SDLCALL SDL_Convert8(SDL_AudioCVT *cvt, Uint16 format)
{
	int i;
	Uint8 *src, *dst;

#ifdef DEBUG_CONVERT
	fprintf(stderr, "Converting to 8-bit\n");
#endif
	src = cvt->buf;
	dst = cvt->buf;
	if ( (format & 0x1000) != 0x1000 ) { /* Little endian */
		++src;
	}
	for ( i=cvt->len_cvt/2; i; --i ) {
		*dst = *src;
		src += 2;
		dst += 1;
	}
	format = ((format & ~0x9010) | AUDIO_U8);
	cvt->len_cvt /= 2;
	if ( cvt->filters[++cvt->filter_index] ) {
		cvt->filters[cvt->filter_index](cvt, format);
	}
}

/* Toggle signed/unsigned */
void SDLCALL SDL_ConvertSign(SDL_AudioCVT *cvt, Uint16 format)
{
	int i;
	Uint8 *data;

#ifdef DEBUG_CONVERT
	fprintf(stderr, "Converting audio signedness\n");
#endif
	data = cvt->buf;
	if ( (format & 0xFF) == 16 ) {
		if ( (format & 0x1000) != 0x1000 ) { /* Little endian */
			++data;
		}
		for ( i=cvt->len_cvt/2; i; --i ) {
			*data ^= 0x80;
			data += 2;
		}
	} else {
		for ( i=cvt->len_cvt; i; --i ) {
			*data++ ^= 0x80;
		}
	}
	format = (format ^ 0x8000);
	if ( cvt->filters[++cvt->filter_index] ) {
		cvt->filters[cvt->filter_index](cvt, format);
	}
}

/* Toggle endianness */
void SDLCALL SDL_ConvertEndian(SDL_AudioCVT *cvt, Uint16 format)
{
	int i;
	Uint8 *data, tmp;

#ifdef DEBUG_CONVERT
	fprintf(stderr, "Converting audio endianness\n");
#endif
	data = cvt->buf;
	for ( i=cvt->len_cvt/2; i; --i ) {
		tmp = data[0];
		data[0] = data[1];
		data[1] = tmp;
		data += 2;
	}
	format = (format ^ 0x1000);
	if ( cvt->filters[++cvt->filter_index] ) {
		cvt->filters[cvt->filter_index](cvt, format);
	}
}

/* Convert rate up by multiple of 2 */
void SDLCALL SDL_RateMUL2(SDL_AudioCVT *cvt, Uint16 format)
{
	int i;
	Uint8 *src, *dst;

#ifdef DEBUG_CONVERT
	fprintf(stderr, "Converting audio rate * 2\n");
#endif
	src = cvt->buf+cvt->len_cvt;
	dst = cvt->buf+cvt->len_cvt*2;
	switch (format & 0xFF) {
		case 8:
			for ( i=cvt->len_cvt; i; --i ) {
				src -= 1;
				dst -= 2;
				dst[0] = src[0];
				dst[1] = src[0];
			}
			break;
		case 16:
			for ( i=cvt->len_cvt/2; i; --i ) {
				src -= 2;
				dst -= 4;
				dst[0] = src[0];
				dst[1] = src[1];
				dst[2] = src[0];
				dst[3] = src[1];
			}
			break;
	}
	cvt->len_cvt *= 2;
	if ( cvt->filters[++cvt->filter_index] ) {
		cvt->filters[cvt->filter_index](cvt, format);
	}
}


/* Convert rate up by multiple of 2, for stereo */
void SDLCALL SDL_RateMUL2_c2(SDL_AudioCVT *cvt, Uint16 format)
{
	int i;
	Uint8 *src, *dst;

#ifdef DEBUG_CONVERT
	fprintf(stderr, "Converting audio rate * 2\n");
#endif
	src = cvt->buf+cvt->len_cvt;
	dst = cvt->buf+cvt->len_cvt*2;
	switch (format & 0xFF) {
		case 8:
			for ( i=cvt->len_cvt/2; i; --i ) {
				src -= 2;
				dst -= 4;
				dst[0] = src[0];
				dst[1] = src[1];
				dst[2] = src[0];
				dst[3] = src[1];
			}
			break;
		case 16:
			for ( i=cvt->len_cvt/4; i; --i ) {
				src -= 4;
				dst -= 8;
				dst[0] = src[0];
				dst[1] = src[1];
				dst[2] = src[2];
				dst[3] = src[3];
				dst[4] = src[0];
				dst[5] = src[1];
				dst[6] = src[2];
				dst[7] = src[3];
			}
			break;
	}
	cvt->len_cvt *= 2;
	if ( cvt->filters[++cvt->filter_index] ) {
		cvt->filters[cvt->filter_index](cvt, format);
	}
}

/* Convert rate up by multiple of 2, for quad */
void SDLCALL SDL_RateMUL2_c4(SDL_AudioCVT *cvt, Uint16 format)
{
	int i;
	Uint8 *src, *dst;

#ifdef DEBUG_CONVERT
	fprintf(stderr, "Converting audio rate * 2\n");
#endif
	src = cvt->buf+cvt->len_cvt;
	dst = cvt->buf+cvt->len_cvt*2;
	switch (format & 0xFF) {
		case 8:
			for ( i=cvt->len_cvt/4; i; --i ) {
				src -= 4;
				dst -= 8;
				dst[0] = src[0];
				dst[1] = src[1];
				dst[2] = src[2];
				dst[3] = src[3];
				dst[4] = src[0];
				dst[5] = src[1];
				dst[6] = src[2];
				dst[7] = src[3];
			}
			break;
		case 16:
			for ( i=cvt->len_cvt/8; i; --i ) {
				src -= 8;
				dst -= 16;
				dst[0] = src[0];
				dst[1] = src[1];
				dst[2] = src[2];
				dst[3] = src[3];
				dst[4] = src[4];
				dst[5] = src[5];
				dst[6] = src[6];
				dst[7] = src[7];
				dst[8] = src[0];
				dst[9] = src[1];
				dst[10] = src[2];
				dst[11] = src[3];
				dst[12] = src[4];
				dst[13] = src[5];
				dst[14] = src[6];
				dst[15] = src[7];
			}
			break;
	}
	cvt->len_cvt *= 2;
	if ( cvt->filters[++cvt->filter_index] ) {
		cvt->filters[cvt->filter_index](cvt, format);
	}
}


/* Convert rate up by multiple of 2, for 5.1 */
void SDLCALL SDL_RateMUL2_c6(SDL_AudioCVT *cvt, Uint16 format)
{
	int i;
	Uint8 *src, *dst;

#ifdef DEBUG_CONVERT
	fprintf(stderr, "Converting audio rate * 2\n");
#endif
	src = cvt->buf+cvt->len_cvt;
	dst = cvt->buf+cvt->len_cvt*2;
	switch (format & 0xFF) {
		case 8:
			for ( i=cvt->len_cvt/6; i; --i ) {
				src -= 6;
				dst -= 12;
				dst[0] = src[0];
				dst[1] = src[1];
				dst[2] = src[2];
				dst[3] = src[3];
				dst[4] = src[4];
				dst[5] = src[5];
				dst[6] = src[0];
				dst[7] = src[1];
				dst[8] = src[2];
				dst[9] = src[3];
				dst[10] = src[4];
				dst[11] = src[5];
			}
			break;
		case 16:
			for ( i=cvt->len_cvt/12; i; --i ) {
				src -= 12;
				dst -= 24;
				dst[0] = src[0];
				dst[1] = src[1];
				dst[2] = src[2];
				dst[3] = src[3];
				dst[4] = src[4];
				dst[5] = src[5];
				dst[6] = src[6];
				dst[7] = src[7];
				dst[8] = src[8];
				dst[9] = src[9];
				dst[10] = src[10];
				dst[11] = src[11];
				dst[12] = src[0];
				dst[13] = src[1];
				dst[14] = src[2];
				dst[15] = src[3];
				dst[16] = src[4];
				dst[17] = src[5];
				dst[18] = src[6];
				dst[19] = src[7];
				dst[20] = src[8];
				dst[21] = src[9];
				dst[22] = src[10];
				dst[23] = src[11];
			}
			break;
	}
	cvt->len_cvt *= 2;
	if ( cvt->filters[++cvt->filter_index] ) {
		cvt->filters[cvt->filter_index](cvt, format);
	}
}

/* Convert rate down by multiple of 2 */
void SDLCALL SDL_RateDIV2(SDL_AudioCVT *cvt, Uint16 format)
{
	int i;
	Uint8 *src, *dst;

#ifdef DEBUG_CONVERT
	fprintf(stderr, "Converting audio rate / 2\n");
#endif
	src = cvt->buf;
	dst = cvt->buf;
	switch (format & 0xFF) {
		case 8:
			for ( i=cvt->len_cvt/2; i; --i ) {
				dst[0] = src[0];
				src += 2;
				dst += 1;
			}
			break;
		case 16:
			for ( i=cvt->len_cvt/4; i; --i ) {
				dst[0] = src[0];
				dst[1] = src[1];
				src += 4;
				dst += 2;
			}
			break;
	}
	cvt->len_cvt /= 2;
	if ( cvt->filters[++cvt->filter_index] ) {
		cvt->filters[cvt->filter_index](cvt, format);
	}
}


/* Convert rate down by multiple of 2, for stereo */
void SDLCALL SDL_RateDIV2_c2(SDL_AudioCVT *cvt, Uint16 format)
{
	int i;
	Uint8 *src, *dst;

#ifdef DEBUG_CONVERT
	fprintf(stderr, "Converting audio rate / 2\n");
#endif
	src = cvt->buf;
	dst = cvt->buf;
	switch (format & 0xFF) {
		case 8:
			for ( i=cvt->len_cvt/4; i; --i ) {
				dst[0] = src[0];
				dst[1] = src[1];
				src += 4;
				dst += 2;
			}
			break;
		case 16:
			for ( i=cvt->len_cvt/8; i; --i ) {
				dst[0] = src[0];
				dst[1] = src[1];
				dst[2] = src[2];
				dst[3] = src[3];
				src += 8;
				dst += 4;
			}
			break;
	}
	cvt->len_cvt /= 2;
	if ( cvt->filters[++cvt->filter_index] ) {
		cvt->filters[cvt->filter_index](cvt, format);
	}
}


/* Convert rate down by multiple of 2, for quad */
void SDLCALL SDL_RateDIV2_c4(SDL_AudioCVT *cvt, Uint16 format)
{
	int i;
	Uint8 *src, *dst;

#ifdef DEBUG_CONVERT
	fprintf(stderr, "Converting audio rate / 2\n");
#endif
	src = cvt->buf;
	dst = cvt->buf;
	switch (format & 0xFF) {
		case 8:
			for ( i=cvt->len_cvt/8; i; --i ) {
				dst[0] = src[0];
				dst[1] = src[1];
				dst[2] = src[2];
				dst[3] = src[3];
				src += 8;
				dst += 4;
			}
			break;
		case 16:
			for ( i=cvt->len_cvt/16; i; --i ) {
				dst[0] = src[0];
				dst[1] = src[1];
				dst[2] = src[2];
				dst[3] = src[3];
				dst[4] = src[4];
				dst[5] = src[5];
				dst[6] = src[6];
				dst[7] = src[7];
				src += 16;
				dst += 8;
			}
			break;
	}
	cvt->len_cvt /= 2;
	if ( cvt->filters[++cvt->filter_index] ) {
		cvt->filters[cvt->filter_index](cvt, format);
	}
}

/* Convert rate down by multiple of 2, for 5.1 */
void SDLCALL SDL_RateDIV2_c6(SDL_AudioCVT *cvt, Uint16 format)
{
	int i;
	Uint8 *src, *dst;

#ifdef DEBUG_CONVERT
	fprintf(stderr, "Converting audio rate / 2\n");
#endif
	src = cvt->buf;
	dst = cvt->buf;
	switch (format & 0xFF) {
		case 8:
			for ( i=cvt->len_cvt/12; i; --i ) {
				dst[0] = src[0];
				dst[1] = src[1];
				dst[2] = src[2];
				dst[3] = src[3];
				dst[4] = src[4];
				dst[5] = src[5];
				src += 12;
				dst += 6;
			}
			break;
		case 16:
			for ( i=cvt->len_cvt/24; i; --i ) {
				dst[0] = src[0];
				dst[1] = src[1];
				dst[2] = src[2];
				dst[3] = src[3];
				dst[4] = src[4];
				dst[5] = src[5];
				dst[6] = src[6];
				dst[7] = src[7];
				dst[8] = src[8];
				dst[9] = src[9];
				dst[10] = src[10];
				dst[11] = src[11];
				src += 24;
				dst += 12;
			}
			break;
	}
	cvt->len_cvt /= 2;
	if ( cvt->filters[++cvt->filter_index] ) {
		cvt->filters[cvt->filter_index](cvt, format);
	}
}

/* Very slow rate conversion routine */
void SDLCALL SDL_RateSLOW(SDL_AudioCVT *cvt, Uint16 format)
{
	double ipos;
	int i, clen;

#ifdef DEBUG_CONVERT
	fprintf(stderr, "Converting audio rate * %4.4f\n", 1.0/cvt->rate_incr);
#endif
	clen = (int)((double)cvt->len_cvt / cvt->rate_incr);
	if ( cvt->rate_incr > 1.0 ) {
		switch (format & 0xFF) {
			case 8: {
				Uint8 *output;

				output = cvt->buf;
				ipos = 0.0;
				for ( i=clen; i; --i ) {
					*output = cvt->buf[(int)ipos];
					ipos += cvt->rate_incr;
					output += 1;
				}
			}
			break;

			case 16: {
				Uint16 *output;

				clen &= ~1;
				output = (Uint16 *)cvt->buf;
				ipos = 0.0;
				for ( i=clen/2; i; --i ) {
					*output=((Uint16 *)cvt->buf)[(int)ipos];
					ipos += cvt->rate_incr;
					output += 1;
				}
			}
			break;
		}
	} else {
		switch (format & 0xFF) {
			case 8: {
				Uint8 *output;

				output = cvt->buf+clen;
				ipos = (double)cvt->len_cvt;
				for ( i=clen; i; --i ) {
					ipos -= cvt->rate_incr;
					output -= 1;
					*output = cvt->buf[(int)ipos];
				}
			}
			break;

			case 16: {
				Uint16 *output;

				clen &= ~1;
				output = (Uint16 *)(cvt->buf+clen);
				ipos = (double)cvt->len_cvt/2;
				for ( i=clen/2; i; --i ) {
					ipos -= cvt->rate_incr;
					output -= 1;
					*output=((Uint16 *)cvt->buf)[(int)ipos];
				}
			}
			break;
		}
	}
	cvt->len_cvt = clen;
	if ( cvt->filters[++cvt->filter_index] ) {
		cvt->filters[cvt->filter_index](cvt, format);
	}
}

int SDL_ConvertAudio(SDL_AudioCVT *cvt)
{
	/* Make sure there's data to convert */
	if ( cvt->buf == NULL ) {
		SDL_SetError("No buffer allocated for conversion");
		return(-1);
	}
	/* Return okay if no conversion is necessary */
	cvt->len_cvt = cvt->len;
	if ( cvt->filters[0] == NULL ) {
		return(0);
	}

	/* Set up the conversion and go! */
	cvt->filter_index = 0;
	cvt->filters[0](cvt, cvt->src_format);
	return(0);
}

/* Creates a set of audio filters to convert from one format to another. 
   Returns -1 if the format conversion is not supported, or 1 if the
   audio filter is set up.
*/
  
int SDL_BuildAudioCVT(SDL_AudioCVT *cvt,
	Uint16 src_format, Uint8 src_channels, int src_rate,
	Uint16 dst_format, Uint8 dst_channels, int dst_rate)
{
/*printf("Build format %04x->%04x, channels %u->%u, rate %d->%d\n",
		src_format, dst_format, src_channels, dst_channels, src_rate, dst_rate);*/
	/* Start off with no conversion necessary */
	cvt->needed = 0;
	cvt->filter_index = 0;
	cvt->filters[0] = NULL;
	cvt->len_mult = 1;
	cvt->len_ratio = 1.0;

	/* First filter:  Endian conversion from src to dst */
	if ( (src_format & 0x1000) != (dst_format & 0x1000)
	     && ((src_format & 0xff) == 16) && ((dst_format & 0xff) == 16)) {
		cvt->filters[cvt->filter_index++] = SDL_ConvertEndian;
	}
	
	/* Second filter: Sign conversion -- signed/unsigned */
	if ( (src_format & 0x8000) != (dst_format & 0x8000) ) {
		cvt->filters[cvt->filter_index++] = SDL_ConvertSign;
	}

	/* Next filter:  Convert 16 bit <--> 8 bit PCM */
	if ( (src_format & 0xFF) != (dst_format & 0xFF) ) {
		switch (dst_format&0x10FF) {
			case AUDIO_U8:
				cvt->filters[cvt->filter_index++] =
							 SDL_Convert8;
				cvt->len_ratio /= 2;
				break;
			case AUDIO_U16LSB:
				cvt->filters[cvt->filter_index++] =
							SDL_Convert16LSB;
				cvt->len_mult *= 2;
				cvt->len_ratio *= 2;
				break;
			case AUDIO_U16MSB:
				cvt->filters[cvt->filter_index++] =
							SDL_Convert16MSB;
				cvt->len_mult *= 2;
				cvt->len_ratio *= 2;
				break;
		}
	}

	/* Last filter:  Mono/Stereo conversion */
	if ( src_channels != dst_channels ) {
		if ( (src_channels == 1) && (dst_channels > 1) ) {
			cvt->filters[cvt->filter_index++] = 
						SDL_ConvertStereo;
			cvt->len_mult *= 2;
			src_channels = 2;
			cvt->len_ratio *= 2;
		}
		if ( (src_channels == 2) &&
				(dst_channels == 6) ) {
			cvt->filters[cvt->filter_index++] =
						 SDL_ConvertSurround;
			src_channels = 6;
			cvt->len_mult *= 3;
			cvt->len_ratio *= 3;
		}
		if ( (src_channels == 2) &&
				(dst_channels == 4) ) {
			cvt->filters[cvt->filter_index++] =
						 SDL_ConvertSurround_4;
			src_channels = 4;
			cvt->len_mult *= 2;
			cvt->len_ratio *= 2;
		}
		while ( (src_channels*2) <= dst_channels ) {
			cvt->filters[cvt->filter_index++] = 
						SDL_ConvertStereo;
			cvt->len_mult *= 2;
			src_channels *= 2;
			cvt->len_ratio *= 2;
		}
		if ( (src_channels == 6) &&
				(dst_channels <= 2) ) {
			cvt->filters[cvt->filter_index++] =
						 SDL_ConvertStrip;
			src_channels = 2;
			cvt->len_ratio /= 3;
		}
		if ( (src_channels == 6) &&
				(dst_channels == 4) ) {
			cvt->filters[cvt->filter_index++] =
						 SDL_ConvertStrip_2;
			src_channels = 4;
			cvt->len_ratio /= 2;
		}
		/* This assumes that 4 channel audio is in the format:
		     Left {front/back} + Right {front/back}
		   so converting to L/R stereo works properly.
		 */
		while ( ((src_channels%2) == 0) &&
				((src_channels/2) >= dst_channels) ) {
			cvt->filters[cvt->filter_index++] =
						 SDL_ConvertMono;
			src_channels /= 2;
			cvt->len_ratio /= 2;
		}
		if ( src_channels != dst_channels ) {
			/* Uh oh.. */;
		}
	}

	/* Do rate conversion */
	cvt->rate_incr = 0.0;
	if ( (src_rate/100) != (dst_rate/100) ) {
		Uint32 hi_rate, lo_rate;
		int len_mult;
		double len_ratio;
		void (SDLCALL *rate_cvt)(SDL_AudioCVT *cvt, Uint16 format);

		if ( src_rate > dst_rate ) {
			hi_rate = src_rate;
			lo_rate = dst_rate;
			switch (src_channels) {
				case 1: rate_cvt = SDL_RateDIV2; break;
				case 2: rate_cvt = SDL_RateDIV2_c2; break;
				case 4: rate_cvt = SDL_RateDIV2_c4; break;
				case 6: rate_cvt = SDL_RateDIV2_c6; break;
				default: return -1;
			}
			len_mult = 1;
			len_ratio = 0.5;
		} else {
			hi_rate = dst_rate;
			lo_rate = src_rate;
			switch (src_channels) {
				case 1: rate_cvt = SDL_RateMUL2; break;
				case 2: rate_cvt = SDL_RateMUL2_c2; break;
				case 4: rate_cvt = SDL_RateMUL2_c4; break;
				case 6: rate_cvt = SDL_RateMUL2_c6; break;
				default: return -1;
			}
			len_mult = 2;
			len_ratio = 2.0;
		}
		/* If hi_rate = lo_rate*2^x then conversion is easy */
		while ( ((lo_rate*2)/100) <= (hi_rate/100) ) {
			cvt->filters[cvt->filter_index++] = rate_cvt;
			cvt->len_mult *= len_mult;
			lo_rate *= 2;
			cvt->len_ratio *= len_ratio;
		}
		/* We may need a slow conversion here to finish up */
		if ( (lo_rate/100) != (hi_rate/100) ) {
#if 1
			/* The problem with this is that if the input buffer is
			   say 1K, and the conversion rate is say 1.1, then the
			   output buffer is 1.1K, which may not be an acceptable
			   buffer size for the audio driver (not a power of 2)
			*/
			/* For now, punt and hope the rate distortion isn't great.
			*/
#else
			if ( src_rate < dst_rate ) {
				cvt->rate_incr = (double)lo_rate/hi_rate;
				cvt->len_mult *= 2;
				cvt->len_ratio /= cvt->rate_incr;
			} else {
				cvt->rate_incr = (double)hi_rate/lo_rate;
				cvt->len_ratio *= cvt->rate_incr;
			}
			cvt->filters[cvt->filter_index++] = SDL_RateSLOW;
#endif
		}
	}

	/* Set up the filter information */
	if ( cvt->filter_index != 0 ) {
		cvt->needed = 1;
		cvt->src_format = src_format;
		cvt->dst_format = dst_format;
		cvt->len = 0;
		cvt->buf = NULL;
		cvt->filters[cvt->filter_index] = NULL;
	}
	return(cvt->needed);
}
