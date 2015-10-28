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

#ifdef SDL_JOYSTICK_RISCOS

/*
   RISC OS - Joystick support by Alan Buckley (alan_baa@hotmail.com) - 10 April 2003

   Note: Currently assumes joystick is present if joystick module is loaded
   and that there is one joystick with four buttons.
*/

/* This is the system specific header for the SDL joystick API */

#include "SDL_events.h"
#include "SDL_joystick.h"
#include "../SDL_sysjoystick.h"
#include "../SDL_joystick_c.h"

#include "kernel.h"

#define JOYSTICK_READ 0x43F40

struct joystick_hwdata 
{
	int joystate;
};


/* Function to scan the system for joysticks.
 * This function should set SDL_numjoysticks to the number of available
 * joysticks.  Joystick 0 should be the system default joystick.
 * It should return number of joysticks, or -1 on an unrecoverable fatal error.
 */
int SDL_SYS_JoystickInit(void)
{
	_kernel_swi_regs regs;

	 /* Try to read joystick 0 */
	regs.r[0] = 0;
	if (_kernel_swi(JOYSTICK_READ, &regs, &regs) == NULL)
	{
		/* Switch works so assume we've got a joystick */
		return 1;
	}
	/* Switch fails so it looks like there's no joystick here */

	return(0);
}

/* Function to get the device-dependent name of a joystick */
const char *SDL_SYS_JoystickName(int index)
{
	if (index == 0)
	{
		return "RISC OS Joystick 0";
	}

	SDL_SetError("No joystick available with that index");
	return(NULL);
}

/* Function to open a joystick for use.
   The joystick to open is specified by the index field of the joystick.
   This should fill the nbuttons and naxes fields of the joystick structure.
   It returns 0, or -1 if there is an error.
 */
int SDL_SYS_JoystickOpen(SDL_Joystick *joystick)
{
	_kernel_swi_regs regs;

	if(!(joystick->hwdata=SDL_malloc(sizeof(struct joystick_hwdata))))
		return -1;

	regs.r[0] = joystick->index;

	/* Don't know how to get exact count of buttons so assume max of 4 for now */
	joystick->nbuttons=4;

	joystick->nhats=0;
	joystick->nballs=0;
	joystick->naxes=2;
	joystick->hwdata->joystate=0;

	return 0;

}

/* Function to update the state of a joystick - called as a device poll.
 * This function shouldn't update the joystick structure directly,
 * but instead should call SDL_PrivateJoystick*() to deliver events
 * and update joystick device state.
 */
void SDL_SYS_JoystickUpdate(SDL_Joystick *joystick)
{
	_kernel_swi_regs regs;
	regs.r[0] = joystick->index;

	if (_kernel_swi(JOYSTICK_READ, &regs, &regs) == NULL)
	{
		int newstate = regs.r[0];
		int oldstate = joystick->hwdata->joystate;
		if (newstate != oldstate)
		{
			if ((newstate & 0xFF) != (oldstate & 0xFF))
			{
				int y = regs.r[0] & 0xFF;
				/* Convert to signed values */
				if (y >= 128) y -= 256;
				SDL_PrivateJoystickAxis(joystick,1,-y * 256); /* Up and down opposite to result in SDL */
			}
			if ((newstate & 0xFF00) != (oldstate & 0xFF00))
			{
				int x = (regs.r[0] & 0xFF00) >> 8;
				if (x >= 128) x -= 256;
				SDL_PrivateJoystickAxis(joystick,0,x * 256);
			}

			if ((newstate & 0xFF0000) != (oldstate & 0xFF0000))
			{
				int buttons = (regs.r[0] & 0xFF0000) >> 16;
				int oldbuttons = (oldstate & 0xFF0000) >> 16;
				int i;
				for (i = 0; i < joystick->nbuttons; i++)
				{
					if ((buttons & (1<<i)) != (oldbuttons & (1<<i)))
					{
						if (buttons & (1<<i)) SDL_PrivateJoystickButton(joystick,i,SDL_PRESSED);
						else SDL_PrivateJoystickButton(joystick,i,SDL_RELEASED);
					}
				}
			}
			joystick->hwdata->joystate = newstate;
		}		
	}

	return;
}

/* Function to close a joystick after use */
void SDL_SYS_JoystickClose(SDL_Joystick *joystick)
{
	if(joystick->hwdata)
		SDL_free(joystick->hwdata);
	return;
}

/* Function to perform any system-specific joystick related cleanup */
void SDL_SYS_JoystickQuit(void)
{
	SDL_numjoysticks=0;

	return;
}

#endif /* SDL_JOYSTICK_RISCOS */
