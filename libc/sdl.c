#include "SDL.h"
#include <stdio.h>

int main(int argc, char* argv[]) {

    SDL_Surface *surface;                    // Declare a pointer

    SDL_Init(SDL_INIT_VIDEO);              // Initialize SDL2

    // Create an application window with the following settings:
    surface = SDL_SetVideoMode(
        640,                               // width, in pixels
        480,                               // height, in pixels
        32,
        0                  // flags - see below
    );

    // Check that the window was successfully created
    if (surface == NULL) {
        // In the case that the window could not be made...
        printf("Could not create window: %s\n", SDL_GetError());
        return 1;
    }

    // The window is open: could enter program loop here (see SDL_PollEvent())

    SDL_Delay(3000);  // Pause execution for 3000 milliseconds, for example

    // Clean up
    SDL_Quit();
    return 0;
}
