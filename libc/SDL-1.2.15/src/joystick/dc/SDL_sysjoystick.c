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

#ifdef SDL_JOYSTICK_DC

#include "SDL_events.h"
#include "SDL_joystick.h"
#include "../SDL_sysjoystick.h"
#include "../SDL_joystick_c.h"

#include <dc/maple.h>
#include <dc/maple/controller.h>

#define MAX_JOYSTICKS	8	/* only 2 are supported in the multimedia API */
#define MAX_AXES	6	/* each joystick can have up to 6 axes */
#define MAX_BUTTONS	8	/* and 8 buttons                      */
#define	MAX_HATS	2

#define	JOYNAMELEN	8

/* array to hold joystick ID values */
static uint8	SYS_Joystick_addr[MAX_JOYSTICKS];

/* The private structure used to keep track of a joystick */
struct joystick_hwdata
{
	cont_cond_t prev_cond;
	int prev_buttons;
};

/* Function to scan the system for joysticks.
 * This function should set SDL_numjoysticks to the number of available
 * joysticks.  Joystick 0 should be the system default joystick.
 * It should return 0, or -1 on an unrecoverable fatal error.
 */
int SDL_SYS_JoystickInit(void)
{
	int numdevs;

	int p,u;

	numdevs = 0;
	for(p=0;p<MAPLE_PORT_COUNT;p++) {
		for(u=0;u<MAPLE_UNIT_COUNT;u++) {
			if (maple_device_func(p,u)&MAPLE_FUNC_CONTROLLER) {
				SYS_Joystick_addr[numdevs] = maple_addr(p,u);
				numdevs++;
			}
		}
	}

	return(numdevs);
}

/* Function to get the device-dependent name of a joystick */
const char *SDL_SYS_JoystickName(int index)
{
	maple_device_t *dev;
	if (maple_compat_resolve(SYS_Joystick_addr[index],&dev,MAPLE_FUNC_CONTROLLER)!=0) return NULL;
	return dev->info.product_name;
}

/* Function to open a joystick for use.
   The joystick to open is specified by the index field of the joystick.
   This should fill the nbuttons and naxes fields of the joystick structure.
   It returns 0, or -1 if there is an error.
 */
int SDL_SYS_JoystickOpen(SDL_Joystick *joystick)
{
	/* allocate memory for system specific hardware data */
	joystick->hwdata = (struct joystick_hwdata *) SDL_malloc(sizeof(*joystick->hwdata));
	if (joystick->hwdata == NULL)
	{
		SDL_OutOfMemory();
		return(-1);
	}
	SDL_memset(joystick->hwdata, 0, sizeof(*joystick->hwdata));

	/* fill nbuttons, naxes, and nhats fields */
	joystick->nbuttons = MAX_BUTTONS;
	joystick->naxes = MAX_AXES;
	joystick->nhats = MAX_HATS;
	return(0);
}


/* Function to update the state of a joystick - called as a device poll.
 * This function shouldn't update the joystick structure directly,
 * but instead should call SDL_PrivateJoystick*() to deliver events
 * and update joystick device state.
 */

void SDL_SYS_JoystickUpdate(SDL_Joystick *joystick)
{
const	int sdl_buttons[] = {
	CONT_C,
	CONT_B,
	CONT_A,
	CONT_START,
	CONT_Z,
	CONT_Y,
	CONT_X,
	CONT_D
};

	uint8 addr;
	cont_cond_t cond,*prev_cond;
	int buttons,prev_buttons,i,changed;

	addr = SYS_Joystick_addr[joystick->index];
	if (cont_get_cond(addr,&cond)<0) return;

	buttons = cond.buttons;
	prev_buttons = joystick->hwdata->prev_buttons;
	changed = buttons^prev_buttons;

	if ((changed)&(CONT_DPAD_UP|CONT_DPAD_DOWN|CONT_DPAD_LEFT|CONT_DPAD_RIGHT)) {
		int hat = SDL_HAT_CENTERED;
		if (buttons&CONT_DPAD_UP) hat|=SDL_HAT_UP;
		if (buttons&CONT_DPAD_DOWN) hat|=SDL_HAT_DOWN;
		if (buttons&CONT_DPAD_LEFT) hat|=SDL_HAT_LEFT;
		if (buttons&CONT_DPAD_RIGHT) hat|=SDL_HAT_RIGHT;
		SDL_PrivateJoystickHat(joystick, 0, hat);
	}
	if ((changed)&(CONT_DPAD2_UP|CONT_DPAD2_DOWN|CONT_DPAD2_LEFT|CONT_DPAD2_RIGHT)) {
		int hat = SDL_HAT_CENTERED;
		if (buttons&CONT_DPAD2_UP) hat|=SDL_HAT_UP;
		if (buttons&CONT_DPAD2_DOWN) hat|=SDL_HAT_DOWN;
		if (buttons&CONT_DPAD2_LEFT) hat|=SDL_HAT_LEFT;
		if (buttons&CONT_DPAD2_RIGHT) hat|=SDL_HAT_RIGHT;
		SDL_PrivateJoystickHat(joystick, 1, hat);
	}

	for(i=0;i<sizeof(sdl_buttons)/sizeof(sdl_buttons[0]);i++) {
		if (changed & sdl_buttons[i]) {
			SDL_PrivateJoystickButton(joystick, i, (buttons & sdl_buttons[i])?SDL_PRESSED:SDL_RELEASED);
		}
	}

	prev_cond = &joystick->hwdata->prev_cond;
	if (cond.joyx!=prev_cond->joyx)
		SDL_PrivateJoystickAxis(joystick, 0, cond.joyx-128);
	if (cond.joyy!=prev_cond->joyy)
		SDL_PrivateJoystickAxis(joystick, 1, cond.joyy-128);
	if (cond.rtrig!=prev_cond->rtrig)
		SDL_PrivateJoystickAxis(joystick, 2, cond.rtrig);
	if (cond.ltrig!=prev_cond->ltrig)
		SDL_PrivateJoystickAxis(joystick, 3, cond.ltrig);
	if (cond.joy2x!=prev_cond->joy2x)
		SDL_PrivateJoystickAxis(joystick, 4, cond.joy2x-128);
	if (cond.joy2y!=prev_cond->joy2y)
		SDL_PrivateJoystickAxis(joystick, 5, cond.joy2y-128);

	joystick->hwdata->prev_buttons = buttons;
	joystick->hwdata->prev_cond = cond;
}

/* Function to close a joystick after use */
void SDL_SYS_JoystickClose(SDL_Joystick *joystick)
{
	if (joystick->hwdata != NULL) {
		/* free system specific hardware data */
		SDL_free(joystick->hwdata);
	}
}

/* Function to perform any system-specific joystick related cleanup */
void SDL_SYS_JoystickQuit(void)
{
	return;
}

#endif /* SDL_JOYSTICK_DC */
