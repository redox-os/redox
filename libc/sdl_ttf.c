#include <stdio.h>
#include <stdlib.h>
#include <SDL/SDL_ttf.h>
#include <SDL/SDL.h>

typedef int BOOL;

#define TRUE 1
#define FALSE 0

/* Some of this code was adapted from the SDL introductionl at
   http://www.libsdl.org/intro.en and from the test code downloaded
   with the SDL sources from http://www.libsdl.org .
*/

/* All SDL types and procedures used here are described in
   /usr/local/include/SDL/SDL_video.h
*/

/* This procedure takes an image, the surface (in this case, the screen)
   and the x and y coordinates where the image is to be written to the
   surface (screen) */

void ShowBMP(SDL_Surface *image, SDL_Surface *screen, int x, int y)
{

  SDL_Rect dest;  // SDL_Rect is a record type for defining the location
                  // and size of a rectangle.


  dest.x = x;
  dest.y = y;
  dest.w = image->w;
  dest.h = image->h;


  // This call writes the image to the screen surface, at the
  // location specified by dest.  The second argument being
  // NULL specifies that the entire image should be written.

  SDL_BlitSurface(image, NULL, screen, &dest);
}

main(int argc, char *argv[]){
  if ( SDL_Init(SDL_INIT_VIDEO) < 0 ) {
    fprintf(stderr, "Unable to init SDL: %s\n", SDL_GetError());
    exit(1);
  }
  atexit(SDL_Quit);
  if ( TTF_Init() < 0 ) {
    fprintf(stderr, "Unable to init TTF: %s\n", TTF_GetError());
    exit(1);
  }
  atexit(TTF_Quit);

  const SDL_VideoInfo *video_info = SDL_GetVideoInfo();

  SDL_Surface *screen = SDL_SetVideoMode(640, 480, video_info->vfmt->BitsPerPixel, SDL_HWSURFACE);

  //Read in a font from the specified font file of the specified size
  TTF_Font *text_font =  TTF_OpenFont("/ui/fonts/DroidSans.ttf", 36);

  if (text_font == NULL) {
    printf("Could not load font\n");
    exit(1);
  }

  SDL_FillRect(screen, NULL, SDL_MapRGB(screen->format, 255, 255, 255));

  SDL_Color text_color;
  text_color.r = 0;
  text_color.g = 0;
  text_color.b = 0;

  SDL_Surface *text_image =  TTF_RenderText_Blended(text_font, "Example Text", text_color);

  ShowBMP(text_image, screen, 10, 10);

  SDL_FreeSurface(text_image);

  SDL_Flip(screen);

  int done = FALSE;

  //SDL_Event, defined in SDL_events.h, is a type used to store
  //events -- such as mouseclicks, key presses, mouse movements,
  //etc. that can occur while the program is running.
  SDL_Event event;
  while (!done) {
    while ( SDL_PollEvent(&event) ) {
      switch (event.type) {
      case SDL_KEYDOWN:
      case SDL_MOUSEBUTTONDOWN:
      case SDL_QUIT:
	done = TRUE;
	break;
      default:
	break;
      }
    }
  }
}
