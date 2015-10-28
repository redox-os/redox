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

#ifndef __SOUNDCOMMON_H
#define __SOUNDCOMMON_H

#include <nds.h>

#define CLOCK (1 << 25)

#ifdef __cplusplus
extern "C" {
#endif

typedef enum
{
	NONE = 0,
	INIT = 1,
	MIX = 2,
	MIXING = 4,
	STOP = 8
}CommandType;

typedef enum
{
	FIFO_NONE = 0,
	UPDATEON_ARM9 = 1,
	MIXCOMPLETE_ONARM9 = 2,
}FifoType; 

typedef struct
{
	s8 *mixbuffer;//,*soundbuffer;
	u32 rate;
	u32 buffersize;
	u32 cmd;
	u8 channel,format;
	u32 soundcursor,numsamples;
	s32 prevtimer;
	s16 period;
}S_SoundSystem;

#define soundsystem ((S_SoundSystem*)((u32)(IPC)+sizeof(TransferRegion)))

#ifdef ARM9
extern void SoundSystemInit(u32 rate,u32 buffersize,u8 channel,u8 format);
extern void SoundStartMixer(void);
extern void SendCommandToArm7(u32 command);
#else
extern void SoundVBlankIrq(void);
extern void SoundSwapAndMix(void);
extern void SoundSetTimer(int period);
extern void SoundFifoHandler(void);
extern void SendCommandToArm9(u32 command);
#endif

#ifdef __cplusplus
}
#endif
#endif
