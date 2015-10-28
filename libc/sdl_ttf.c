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


int window_w, window_h;

main(int argc, char *argv[])
{

  //This initializes the SDL system for audio and video.

  if ( SDL_Init(SDL_INIT_VIDEO) < 0 ) {
    fprintf(stderr, "Unable to init SDL: %s\n", SDL_GetError());
    exit(1);
  }


  //Whenever the program exits, SDL_Quit() will be called.
  //SDL_Quit() is defined in SDL.h

  atexit(SDL_Quit);

  //Also, TTF_Quit() should be called upon program exit.

  atexit(TTF_Quit);

  // Initialize the TrueType font system.

  TTF_Init();

  // This call retrieves information about your screen,
  // including width and height (in pixels) and the
  // number of bits per pixels (i.e. the pixel depth).
  // See the SDL_VideoInfo type defined in SDL_video.h

  const SDL_VideoInfo *video_info = SDL_GetVideoInfo();

  // Let's have a window of 600x400 pixels

  window_w = 600;
  window_h = 400;


  // This creates a graphics window with the specified
  // width, height, and bits per pixel. You are best
  // off matching the number of pits per pixel to that
  // of your screen.  The last argument is a flags argument,
  // the various choices are described in SDL_video.h .
  // SDL_HWSURFACE seems to work fine.

  SDL_Surface *screen = SDL_SetVideoMode(window_w,
					 window_h,
					 video_info->vfmt->BitsPerPixel,
					 SDL_HWSURFACE);


  //This generates the number representing the background color.

  unsigned int background = SDL_MapRGB(screen->format, 0x90, 0x90, 0xff);

  //Read in a font from the specified font file of the specified size

  //This one for Mac OS X.

  TTF_Font *text_font =  TTF_OpenFont("file:///ui/fonts/DroidSans.ttf", 36);

  //This one is for Windows/Cygwin, assuming there is a font named
  //times.ttf that you copied into the current working directory.
  //
  //    TTF_Font *text_font =  TTF_OpenFont("times.ttf", 36);

  //This one is for Linux
  //
  //    TTF_Font *text_font =  TTF_OpenFont("/usr/share/fonts/truetype/freefont/FreeMonoBold.ttf", 36);

  if (text_font == NULL) {
    printf("Could not load font\n");
    exit(1);
  }


  //This writes the background color to the entirety of the screen surface.
  //It won't actually appear on the visible screen until
  //SDL_Flip() is called. The NULL second argument indicates
  //that the backgorund color should be written to the entire screen
  //screen surface.
  SDL_FillRect(screen, NULL, background);


  //This declares a variable of type SDL_Color, which is a struct type
  //containing three 8-bit fields, r, g, and b. It is used to specify
  //the font color for the displayed text image.

  SDL_Color font_color;
  font_color.r = 0;
  font_color.g = 0xff;  //very green.  If you want black, make this 0.
  font_color.b = 0;

  //This is the character array that will contain the string to turn into
  //the text image to be displayed.

  char textbuf[80];


  //This is the count of the number of times the text bounces off the side
  //of the window. It will be displayed as part of the text image.

  int bouncecount = 0;


  //sprintf is like printf, but instead of writing the output to the screen, it
  //writes the output to a string which is passed as the first parameter to
  //sprintf.
  sprintf(textbuf, "Bounces = %d", bouncecount);


  //This converts a string to an SDL_Surface (image) that can be displayed
  //like any other image. TTF_RenderText_Solid takes as parameters the
  //font, the text string, and the font color.

  SDL_Surface *text_image =  TTF_RenderText_Solid(text_font,
						  textbuf,
						  font_color);



  //This causes the actual visible screen to be updated with the
  //contents of the screen surface.

  SDL_Flip(screen);


  //The text image will bounce back and forth horizontally.
  int x_pos = 1;
  int y_pos = (window_h - text_image->h)/2;


  //This variables are used to determine how far the text_image moves,
  //in pixels, each time through the loop, below.
  int delta_x = 2;

  int done = FALSE;

  //SDL_Event, defined in SDL_events.h, is a type used to store
  //events -- such as mouseclicks, key presses, mouse movements,
  //etc. that can occur while the program is running.
  SDL_Event event;


  SDLKey current_key;

  //This loop will keep going until the user presses a mouse key
  //or a key on the keyboard.

  while (!done) {


    //erase any text_image on the screen by writing the background
    //color to the entire screen.
    SDL_FillRect(screen, NULL, background);

    //Call the ShowBMP procedure, above, to write the text text_image
    //to the screen at the specified x and y locations.
    ShowBMP(text_image, screen, x_pos, y_pos);

    /* This will actually cause the visible screen to be updated */

    SDL_Flip(screen);

    //SDL_Delay, defined in SDL_timer.h, allows you to specify
    //a delay in milliseconds, in order to help control the speed of
    //motion in your program.
    SDL_Delay(100);

    //Add the delta's to the x and y directions.
    x_pos += delta_x;

    //Check if the text_image, in the next iteration, will be past the
    //right or left edge of the window. If so, reverse direction.
    if ((x_pos >  screen->w - text_image->w) || (x_pos < 0)) {
      delta_x = -delta_x;

      //put the text_image at the edge of the window
      x_pos = (x_pos < 0) ? 0 : screen->w - text_image->w;

      //The text has bounced, so increment the bounce count.
      bouncecount++;

      //Create a new text image, incorporating the new value
      //of bouncecount:

      //Free up the memory used by the old SDL_Surface image
      SDL_FreeSurface(text_image);

      //new string containing the new bouncecount
      sprintf(textbuf, "Bounces = %d", bouncecount);

      //new SDL_Surface image
      text_image =  TTF_RenderText_Solid(text_font,
					 textbuf,
					 font_color);

    }

    //Now poll to see if the user has hit a key or a
    //mouse button. If so, set done to true so the loop
    //will exit.

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
