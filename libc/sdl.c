#include "SDL.h"
#include <stdio.h>
#include <time.h>

int main(int argc, char* argv[]) {
    SDL_Init(SDL_INIT_VIDEO);

    SDL_Surface * surface = SDL_SetVideoMode(640, 480, 32, 0);

    if (surface == NULL) {
        printf("Could not create surface: %s\n", SDL_GetError());
        return 1;
    }

    time_t start = time(NULL);

    SDL_FillRect(surface, NULL, SDL_MapRGB(surface->format, 255, 255, 255));
    SDL_Flip(surface);

    while(time(NULL) - start < 1){}

    SDL_FillRect(surface, NULL, SDL_MapRGB(surface->format, 255, 0, 0));
    SDL_Flip(surface);

    while(time(NULL) - start < 2){}

    SDL_FillRect(surface, NULL, SDL_MapRGB(surface->format, 0, 255, 0));
    SDL_Flip(surface);

    while(time(NULL) - start < 3){}

    SDL_FillRect(surface, NULL, SDL_MapRGB(surface->format, 0, 0, 255));
    SDL_Flip(surface);

    while(time(NULL) - start < 4){}

    SDL_Quit();

    return 0;
}
