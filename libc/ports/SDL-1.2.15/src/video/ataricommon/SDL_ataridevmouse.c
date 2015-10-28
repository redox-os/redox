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

/*
	MiNT /dev/mouse driver

	Patrice Mandin
*/

#include <fcntl.h>
#include <unistd.h>

#include "../../events/SDL_events_c.h"
#include "SDL_ataridevmouse_c.h"

/* Defines */

#define DEVICE_NAME	"/dev/mouse"

/* Local variables */

static int handle = -1;
static int mouseb, prev_mouseb;

/* Functions */

int SDL_AtariDevMouse_Open(void)
{
	int r;
	const char *mousedev;

	/*
		TODO: Fix the MiNT device driver, that locks mouse for other
		applications, so this is disabled till fixed
	 */
	return 0;

	/* First, try SDL_MOUSEDEV device */
	mousedev = SDL_getenv("SDL_MOUSEDEV");
	if (!mousedev) {
		handle = open(mousedev, 0);
	}

	/* Failed, try default device */
	if (handle<0) {
		handle = open(DEVICE_NAME, 0);
	}

	if (handle<0) {
		handle = -1;
		return 0;
	}

	/* Set non blocking mode */
	r = fcntl(handle, F_GETFL, 0);
	if (r<0) {
		close(handle);
		handle = -1;
		return 0;
	}

	r |= O_NDELAY;

	r = fcntl(handle, F_SETFL, r);
	if (r<0) {
		close(handle);
		handle = -1;
		return 0;
	}

	prev_mouseb = 7;
	return 1;
}

void SDL_AtariDevMouse_Close(void)
{
	if (handle>0) {
		close(handle);
		handle = -1;
	}
}

static int atari_GetButton(int button)
{
	switch(button)
	{
		case 0:
			return SDL_BUTTON_RIGHT;
		case 1:
			return SDL_BUTTON_MIDDLE;
		default:
			break;
	}

	return SDL_BUTTON_LEFT;
}

void SDL_AtariDevMouse_PostMouseEvents(_THIS, SDL_bool buttonEvents)
{
	unsigned char buffer[3];
	int mousex, mousey;

	if (handle<0) {
		return;
	}

	mousex = mousey = 0;
	while (read(handle, buffer, sizeof(buffer))==sizeof(buffer)) {
		mouseb = buffer[0] & 7;
		mousex += (char) buffer[1];
		mousey += (char) buffer[2];

		/* Mouse button events */
		if (buttonEvents && (mouseb != prev_mouseb)) {
			int i;

			for (i=0;i<3;i++) {
				int curbutton, prevbutton;

				curbutton = mouseb & (1<<i);
				prevbutton = prev_mouseb & (1<<i);
			
				if (curbutton && !prevbutton) {
					SDL_PrivateMouseButton(SDL_RELEASED, atari_GetButton(i), 0, 0);
				}
				if (!curbutton && prevbutton) {
					SDL_PrivateMouseButton(SDL_PRESSED, atari_GetButton(i), 0, 0);
				}
			}

			prev_mouseb = mouseb;
		}
	}

	/* Mouse motion event */
	if (mousex || mousey) {
		SDL_PrivateMouseMotion(0, 1, mousex, -mousey);
	}
}
