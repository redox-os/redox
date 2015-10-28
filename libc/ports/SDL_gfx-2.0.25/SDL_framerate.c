/*

SDL_framerate.c: framerate manager

Copyright (C) 2001-2012  Andreas Schiffler

This software is provided 'as-is', without any express or implied
warranty. In no event will the authors be held liable for any damages
arising from the use of this software.

Permission is granted to anyone to use this software for any purpose,
including commercial applications, and to alter it and redistribute it
freely, subject to the following restrictions:

1. The origin of this software must not be misrepresented; you must not
claim that you wrote the original software. If you use this software
in a product, an acknowledgment in the product documentation would be
appreciated but is not required.

2. Altered source versions must be plainly marked as such, and must not be
misrepresented as being the original software.

3. This notice may not be removed or altered from any source
distribution.

Andreas Schiffler -- aschiffler at ferzkopp dot net

*/

#include "SDL_framerate.h"

/*!
\brief Internal wrapper to SDL_GetTicks that ensures a non-zero return value.

\return The tick count.
*/
Uint32 _getTicks()
{
	Uint32 ticks = SDL_GetTicks();

	/* 
	* Since baseticks!=0 is used to track initialization
	* we need to ensure that the tick count is always >0 
	* since SDL_GetTicks may not have incremented yet and
	* return 0 depending on the timing of the calls.
	*/
	if (ticks == 0) {
		return 1;
	} else {
		return ticks;
	}
}

/*!
\brief Initialize the framerate manager.

Initialize the framerate manager, set default framerate of 30Hz and
reset delay interpolation.

\param manager Pointer to the framerate manager.
*/
void SDL_initFramerate(FPSmanager * manager)
{
	/*
	* Store some sane values 
	*/
	manager->framecount = 0;
	manager->rate = FPS_DEFAULT;
	manager->rateticks = (1000.0f / (float) FPS_DEFAULT);
	manager->baseticks = _getTicks();
	manager->lastticks = manager->baseticks;

}

/*!
\brief Set the framerate in Hz 

Sets a new framerate for the manager and reset delay interpolation.
Rate values must be between FPS_LOWER_LIMIT and FPS_UPPER_LIMIT inclusive to be accepted.

\param manager Pointer to the framerate manager.
\param rate The new framerate in Hz (frames per second).

\return 0 for sucess and -1 for error.
*/
int SDL_setFramerate(FPSmanager * manager, Uint32 rate)
{
	if ((rate >= FPS_LOWER_LIMIT) && (rate <= FPS_UPPER_LIMIT)) {
		manager->framecount = 0;
		manager->rate = rate;
		manager->rateticks = (1000.0f / (float) rate);
		return (0);
	} else {
		return (-1);
	}
}

/*!
\brief Return the current target framerate in Hz 

Get the currently set framerate of the manager.

\param manager Pointer to the framerate manager.

\return Current framerate in Hz or -1 for error.
*/
int SDL_getFramerate(FPSmanager * manager)
{
	if (manager == NULL) {
		return (-1);
	} else {
		return ((int)manager->rate);
	}
}

/*!
\brief Return the current framecount.

Get the current framecount from the framerate manager. 
A frame is counted each time SDL_framerateDelay is called.

\param manager Pointer to the framerate manager.

\return Current frame count or -1 for error.
*/
int SDL_getFramecount(FPSmanager * manager)
{
	if (manager == NULL) {
		return (-1);
	} else {
		return ((int)manager->framecount);
	}
}

/*!
\brief Delay execution to maintain a constant framerate and calculate fps.

Generate a delay to accomodate currently set framerate. Call once in the
graphics/rendering loop. If the computer cannot keep up with the rate (i.e.
drawing too slow), the delay is zero and the delay interpolation is reset.

\param manager Pointer to the framerate manager.

\return The time that passed since the last call to the function in ms. May return 0.
*/
Uint32 SDL_framerateDelay(FPSmanager * manager)
{
	Uint32 current_ticks;
	Uint32 target_ticks;
	Uint32 the_delay;
	Uint32 time_passed = 0;

	/*
	* No manager, no delay
	*/
	if (manager == NULL) {
		return 0;
	}

	/*
	* Initialize uninitialized manager 
	*/
	if (manager->baseticks == 0) {
		SDL_initFramerate(manager);
	}

	/*
	* Next frame 
	*/
	manager->framecount++;

	/*
	* Get/calc ticks 
	*/
	current_ticks = _getTicks();
	time_passed = current_ticks - manager->lastticks;
	manager->lastticks = current_ticks;
	target_ticks = manager->baseticks + (Uint32) ((float) manager->framecount * manager->rateticks);

	if (current_ticks <= target_ticks) {
		the_delay = target_ticks - current_ticks;
		SDL_Delay(the_delay);
	} else {
		manager->framecount = 0;
		manager->baseticks = _getTicks();
	}

	return time_passed;
}
