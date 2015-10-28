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

#include "SDL.h"
#include "SDL_events.h"
#include "../../events/SDL_events_c.h"
#include "SDL_x11video.h"

/* From the X server sources... */
#define MAX_GAMMA 10.0
#define MIN_GAMMA (1.0/MAX_GAMMA)

static int X11_SetGammaNoLock(_THIS, float red, float green, float blue)
{
#if SDL_VIDEO_DRIVER_X11_VIDMODE
    if (use_vidmode >= 200) {
        SDL_NAME(XF86VidModeGamma) gamma;
        Bool succeeded;

	/* Clamp the gamma values */
	if ( red < MIN_GAMMA ) {
		gamma.red = MIN_GAMMA;
	} else
	if ( red > MAX_GAMMA ) {
		gamma.red = MAX_GAMMA;
	} else {
        	gamma.red = red;
	}
	if ( green < MIN_GAMMA ) {
		gamma.green = MIN_GAMMA;
	} else
	if ( green > MAX_GAMMA ) {
		gamma.green = MAX_GAMMA;
	} else {
        	gamma.green = green;
	}
	if ( blue < MIN_GAMMA ) {
		gamma.blue = MIN_GAMMA;
	} else
	if ( blue > MAX_GAMMA ) {
		gamma.blue = MAX_GAMMA;
	} else {
        	gamma.blue = blue;
	}
        if ( SDL_GetAppState() & SDL_APPACTIVE ) {
            succeeded = SDL_NAME(XF86VidModeSetGamma)(SDL_Display, SDL_Screen, &gamma);
            XSync(SDL_Display, False);
        } else {
            gamma_saved[0] = gamma.red;
            gamma_saved[1] = gamma.green;
            gamma_saved[2] = gamma.blue;
            succeeded = True;
        }
        if ( succeeded ) {
            ++gamma_changed;
        }
        return succeeded ? 0 : -1;
    }
#endif
    SDL_SetError("Gamma correction not supported");
    return -1;
}
int X11_SetVidModeGamma(_THIS, float red, float green, float blue)
{
    int result;

    SDL_Lock_EventThread();
    result = X11_SetGammaNoLock(this, red, green, blue);
    SDL_Unlock_EventThread();

    return(result);
}

static int X11_GetGammaNoLock(_THIS, float *red, float *green, float *blue)
{
#if SDL_VIDEO_DRIVER_X11_VIDMODE
    if (use_vidmode >= 200) {
        SDL_NAME(XF86VidModeGamma) gamma;
        if (SDL_NAME(XF86VidModeGetGamma)(SDL_Display, SDL_Screen, &gamma)) {
            *red   = gamma.red;
            *green = gamma.green;
            *blue  = gamma.blue;
            return 0;
        }
        return -1;
    }
#endif
    return -1;
}
int X11_GetVidModeGamma(_THIS, float *red, float *green, float *blue)
{
    int result;

    SDL_Lock_EventThread();
    result = X11_GetGammaNoLock(this, red, green, blue);
    SDL_Unlock_EventThread();

    return(result);
}

void X11_SaveVidModeGamma(_THIS)
{
    /* Try to save the current gamma, otherwise disable gamma control */
    if ( X11_GetGammaNoLock(this,
              &gamma_saved[0], &gamma_saved[1], &gamma_saved[2]) < 0 ) {
        this->SetGamma = 0;
        this->GetGamma = 0;
    }
    gamma_changed = 0;
}
void X11_SwapVidModeGamma(_THIS)
{
    float new_gamma[3];

    if ( gamma_changed ) {
        new_gamma[0] = gamma_saved[0];
        new_gamma[1] = gamma_saved[1];
        new_gamma[2] = gamma_saved[2];
        X11_GetGammaNoLock(this, &gamma_saved[0], &gamma_saved[1], &gamma_saved[2]);
        X11_SetGammaNoLock(this, new_gamma[0], new_gamma[1], new_gamma[2]);
    }
}
