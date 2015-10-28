#include <SDL/SDL.h>
#include <SDL/SDL_image.h>
#include <SDL/SDL_ttf.h>
#include <stdio.h>
#include <time.h>

int log_events(){
    SDL_Event event;
    if(SDL_PollEvent(&event)){  /* Loop until there are no events left on the queue */
        switch(event.type){  /* Process the appropiate event type */
            case SDL_KEYDOWN:  /* Handle a KEYDOWN event */
                printf("Key down %d\n", event.key.keysym.sym);
                if(event.key.keysym.sym == SDLK_ESCAPE){
                    return 0;
                }
                break;
            case SDL_KEYUP:
                printf("Key up: %d\n", event.key.keysym.sym);
                break;
            case SDL_MOUSEMOTION:
                printf("Mouse motion\n");
                break;
            default: /* Report an unhandled event */
                printf("Unknown Event: %d\n", event.type);
                break;
        }
    }
    return 1;
}

int main(int argc, char* argv[]) {
    SDL_Init(SDL_INIT_VIDEO);
    IMG_Init(IMG_INIT_PNG);
    TTF_Init();

    SDL_Surface * surface = SDL_SetVideoMode(640, 480, 32, 0);

    if (surface == NULL) {
        printf("Could not create surface: %s\n", SDL_GetError());
        return 1;
    }

    time_t start = time(NULL);

    SDL_FillRect(surface, NULL, SDL_MapRGB(surface->format, 0, 0, 0));
    SDL_Flip(surface);

    while(time(NULL) - start < 1){
        log_events();
    }

    SDL_FillRect(surface, NULL, SDL_MapRGB(surface->format, 255, 0, 0));
    SDL_Flip(surface);

    while(time(NULL) - start < 2){
        log_events();
    }

    SDL_FillRect(surface, NULL, SDL_MapRGB(surface->format, 0, 255, 0));
    SDL_Flip(surface);

    while(time(NULL) - start < 3){
        log_events();
    }

    SDL_FillRect(surface, NULL, SDL_MapRGB(surface->format, 0, 0, 255));
    SDL_Flip(surface);

    while(time(NULL) - start < 4){
        log_events();
    }

    SDL_FillRect(surface, NULL, SDL_MapRGB(surface->format, 255, 255, 255));
    SDL_Flip(surface);

    while(log_events()){}

    TTF_Quit();
    IMG_Quit();
    SDL_Quit();

    return 0;
}
