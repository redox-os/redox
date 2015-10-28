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
#include "SDL_config.h"

/*
	DMA 8bits and Falcon Codec audio definitions

	Patrice Mandin, Didier Méquignon
*/

#ifndef _SDL_mintaudio_dma8_h
#define _SDL_mintaudio_dma8_h

#define DMAAUDIO_IO_BASE (0xffff8900)
struct DMAAUDIO_IO_S {
	unsigned char int_ctrl;
	unsigned char control;

	unsigned char dummy1;
	unsigned char start_high;
	unsigned char dummy2;
	unsigned char start_mid;
	unsigned char dummy3;
	unsigned char start_low;

	unsigned char dummy4;
	unsigned char cur_high;
	unsigned char dummy5;
	unsigned char cur_mid;
	unsigned char dummy6;
	unsigned char cur_low;

	unsigned char dummy7;
	unsigned char end_high;
	unsigned char dummy8;
	unsigned char end_mid;
	unsigned char dummy9;
	unsigned char end_low;

	unsigned char dummy10[12];

	unsigned char track_ctrl; /* CODEC only */
	unsigned char sound_ctrl;
	unsigned short sound_data;
	unsigned short sound_mask;

	unsigned char dummy11[10];
	
	unsigned short dev_ctrl;
	unsigned short dest_ctrl;
	unsigned short sync_div;
	unsigned char track_rec;
	unsigned char adderin_input;
	unsigned char channel_input;
	unsigned char channel_amplification;
	unsigned char channel_reduction;
	
	unsigned char dummy12[6];

	unsigned char data_direction;
	unsigned char dummy13;
	unsigned char dev_data;
};
#define DMAAUDIO_IO ((*(volatile struct DMAAUDIO_IO_S *)DMAAUDIO_IO_BASE))

#endif /* _SDL_mintaudio_dma8_h */
