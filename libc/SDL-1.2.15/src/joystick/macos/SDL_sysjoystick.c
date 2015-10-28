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

#ifdef SDL_JOYSTICK_MACOS

/*  SDL stuff  --  "SDL_sysjoystick.c"
    MacOS joystick functions by Frederick Reitberger

    The code that follows is meant for SDL.  Use at your own risk.
*/

#include <InputSprocket.h>

#include "SDL_joystick.h"
#include "../SDL_sysjoystick.h"
#include "../SDL_joystick_c.h"


/*  The max number of joysticks we will detect  */
#define     MAX_JOYSTICKS       16 
/*  Limit ourselves to 32 elements per device  */
#define     kMaxReferences      32 

#define		ISpSymmetricAxisToFloat(axis)	((((float) axis) - kISpAxisMiddle) / (kISpAxisMaximum-kISpAxisMiddle))
#define		ISpAsymmetricAxisToFloat(axis)	(((float) axis) / (kISpAxisMaximum))


static  ISpDeviceReference  SYS_Joysticks[MAX_JOYSTICKS];
static  ISpElementListReference SYS_Elements[MAX_JOYSTICKS];
static  ISpDeviceDefinition     SYS_DevDef[MAX_JOYSTICKS];

struct joystick_hwdata 
{
    char name[64];
/*    Uint8   id;*/
    ISpElementReference refs[kMaxReferences];
    /*  gonna need some sort of mapping info  */
}; 


/* Function to scan the system for joysticks.
 * Joystick 0 should be the system default joystick.
 * This function should return the number of available joysticks, or -1
 * on an unrecoverable fatal error.
 */
int SDL_SYS_JoystickInit(void)
{
    static ISpDeviceClass classes[4] = {
        kISpDeviceClass_Joystick,
    #if kISpDeviceClass_Gamepad
        kISpDeviceClass_Gamepad,
    #endif
        kISpDeviceClass_Wheel,
        0
    };
    OSErr   err;
    int     i;
    UInt32  count, numJoysticks;

    if ( (Ptr)0 == (Ptr)ISpStartup ) {
        SDL_SetError("InputSprocket not installed");
        return -1;  //  InputSprocket not installed
    }

    if( (Ptr)0 == (Ptr)ISpGetVersion ) {
        SDL_SetError("InputSprocket not version 1.1 or newer");
        return -1;  //  old version of ISp (not at least 1.1)
    }

    ISpStartup();

    /* Get all the joysticks */
    numJoysticks = 0;
    for ( i=0; classes[i]; ++i ) {
        count = 0;
        err = ISpDevices_ExtractByClass(
            classes[i],
            MAX_JOYSTICKS-numJoysticks,
            &count,
            &SYS_Joysticks[numJoysticks]);
        numJoysticks += count;
    }

    for(i = 0; i < numJoysticks; i++)
    {
        ISpDevice_GetDefinition(
            SYS_Joysticks[i], sizeof(ISpDeviceDefinition),
            &SYS_DevDef[i]);
        
        err = ISpElementList_New(
            0, NULL,
            &SYS_Elements[i], 0);
        
        if (err) {
            SDL_OutOfMemory();
            return -1;
        }

        ISpDevice_GetElementList(
            SYS_Joysticks[i],
            &SYS_Elements[i]);
    }

    ISpDevices_Deactivate(numJoysticks, SYS_Joysticks);

    return numJoysticks;
}

/* Function to get the device-dependent name of a joystick */
const char *SDL_SYS_JoystickName(int index)
{
    static char name[64];
    int len;

    /*  convert pascal string to c-string  */
    len = SYS_DevDef[index].deviceName[0];
    if ( len >= sizeof(name) ) {
        len = (sizeof(name) - 1);
    }
    SDL_memcpy(name, &SYS_DevDef[index].deviceName[1], len);
    name[len] = '\0';

    return name;
}

/* Function to open a joystick for use.
   The joystick to open is specified by the index field of the joystick.
   This should fill the nbuttons and naxes fields of the joystick structure.
   It returns 0, or -1 if there is an error.
 */
int SDL_SYS_JoystickOpen(SDL_Joystick *joystick)
{
    int     index;
    UInt32  count, gotCount, count2;
    long    numAxis, numButtons, numHats, numBalls;

    count = kMaxReferences;
    count2 = 0;
    numAxis = numButtons = numHats = numBalls = 0;

    index = joystick->index;

    /* allocate memory for system specific hardware data */
    joystick->hwdata = (struct joystick_hwdata *) SDL_malloc(sizeof(*joystick->hwdata));
    if (joystick->hwdata == NULL)
    {
		SDL_OutOfMemory();
		return(-1);
    }
    SDL_memset(joystick->hwdata, 0, sizeof(*joystick->hwdata));
    SDL_strlcpy(joystick->hwdata->name, SDL_SYS_JoystickName(index), SDL_arraysize(joystick->hwdata->name));
    joystick->name = joystick->hwdata->name;

    ISpElementList_ExtractByKind(
        SYS_Elements[index],
        kISpElementKind_Axis,
        count,
        &gotCount,
        joystick->hwdata->refs);

    numAxis = gotCount;
    count -= gotCount;
    count2 += gotCount;

    ISpElementList_ExtractByKind(
        SYS_Elements[index],
        kISpElementKind_DPad,
        count,
        &gotCount,
        &(joystick->hwdata->refs[count2]));

    numHats = gotCount;
    count -= gotCount;
    count2 += gotCount;

    ISpElementList_ExtractByKind(
        SYS_Elements[index],
        kISpElementKind_Button,
        count,
        &gotCount,
        &(joystick->hwdata->refs[count2]));

    numButtons = gotCount;
    count -= gotCount;
    count2 += gotCount;

    joystick->naxes = numAxis;
    joystick->nhats = numHats;
    joystick->nballs = numBalls;
    joystick->nbuttons = numButtons;

    ISpDevices_Activate(
        1,
        &SYS_Joysticks[index]);

    return 0;
}

/* Function to update the state of a joystick - called as a device poll.
 * This function shouldn't update the joystick structure directly,
 * but instead should call SDL_PrivateJoystick*() to deliver events
 * and update joystick device state.
 */
void SDL_SYS_JoystickUpdate(SDL_Joystick *joystick)
{
    int     i, j;
    ISpAxisData     a;
    ISpDPadData     b;
    //ISpDeltaData    c;
    ISpButtonData   d;

    for(i = 0, j = 0; i < joystick->naxes; i++, j++)
    {
        Sint16 value;

        ISpElement_GetSimpleState(
            joystick->hwdata->refs[j],
            &a);
        value = (ISpSymmetricAxisToFloat(a)* 32767.0);
        if ( value != joystick->axes[i] ) {
            SDL_PrivateJoystickAxis(joystick, i, value);
        }
    }

    for(i = 0; i < joystick->nhats; i++, j++)
    {
        Uint8 pos;

        ISpElement_GetSimpleState(
            joystick->hwdata->refs[j],
            &b);
        switch(b) {
            case kISpPadIdle:
                pos = SDL_HAT_CENTERED;
                break;
            case kISpPadLeft:
                pos = SDL_HAT_LEFT;
                break;
            case kISpPadUpLeft:
                pos = SDL_HAT_LEFTUP;
                break;
            case kISpPadUp:
                pos = SDL_HAT_UP;
                break;
            case kISpPadUpRight:
                pos = SDL_HAT_RIGHTUP;
                break;
            case kISpPadRight:
                pos = SDL_HAT_RIGHT;
                break;
            case kISpPadDownRight:
                pos = SDL_HAT_RIGHTDOWN;
                break;
            case kISpPadDown:
                pos = SDL_HAT_DOWN;
                break;
            case kISpPadDownLeft:
                pos = SDL_HAT_LEFTDOWN;
                break;
        }
        if ( pos != joystick->hats[i] ) {
            SDL_PrivateJoystickHat(joystick, i, pos);
        }
    }

    for(i = 0; i < joystick->nballs; i++, j++)
    {
        /*  ignore balls right now  */
    }

    for(i = 0; i < joystick->nbuttons; i++, j++)
    {
        ISpElement_GetSimpleState(
            joystick->hwdata->refs[j],
            &d);
        if ( d != joystick->buttons[i] ) {
            SDL_PrivateJoystickButton(joystick, i, d);
        }
    }
}

/* Function to close a joystick after use */
void SDL_SYS_JoystickClose(SDL_Joystick *joystick)
{
    int index;

    index = joystick->index;

    ISpDevices_Deactivate(
        1,
        &SYS_Joysticks[index]);
}

/* Function to perform any system-specific joystick related cleanup */
void SDL_SYS_JoystickQuit(void)
{
    ISpShutdown();
}

#endif /* SDL_JOYSTICK_MACOS */
