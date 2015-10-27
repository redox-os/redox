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
#include "SDL_stdinc.h"

#include "soundcommon.h"

void SoundSystemInit(u32 rate,u32 buffersize,u8 channel,u8 format)
{
	soundsystem->rate = rate;
	
	if(format == 8) 
		soundsystem->buffersize = buffersize;
	else if(format == 16)
		soundsystem->buffersize = buffersize * sizeof(short);

	soundsystem->mixbuffer = (s8*)SDL_malloc(soundsystem->buffersize);
	//soundsystem->soundbuffer = soundsystem->mixbuffer;
	soundsystem->format = format;
	soundsystem->channel = channel;
	soundsystem->prevtimer = 0;
	soundsystem->soundcursor = 0;
	soundsystem->numsamples = 0;
	soundsystem->period = 0x1000000 / rate;
	soundsystem->cmd = INIT;
}

void SoundStartMixer(void)
{
	soundsystem->cmd |= MIX;
}

void SendCommandToArm7(u32 command)
{
    while (REG_IPC_FIFO_CR & IPC_FIFO_SEND_FULL);
    if (REG_IPC_FIFO_CR & IPC_FIFO_ERROR)
    {
        REG_IPC_FIFO_CR |= IPC_FIFO_SEND_CLEAR;
    } 
    
    REG_IPC_FIFO_TX = command;
}
