/* 

TestGfxTexture.c: test program for textured polygon routine
                  (Contributed by Kees Jongenburger)

(C) A. Schiffler, December 2006, zlib license

*/

#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <time.h>

#include "SDL.h"

#ifdef WIN32
#include <windows.h>
#include "SDL_gfxPrimitives.h"
#include "SDL_framerate.h"
#else
#include "SDL/SDL_gfxPrimitives.h"
#include "SDL/SDL_framerate.h"
#endif

void HandleEvent()
{
	SDL_Event event; 

	/* Check for events */
	while ( SDL_PollEvent(&event) ) {
		switch (event.type) {
			 case SDL_KEYDOWN:
			 case SDL_QUIT:
				 exit(0);
				 break;
		}
	}
}

void ClearScreen(SDL_Surface *screen)
{
	int i;
	/* Set the screen to black */
	if ( SDL_LockSurface(screen) == 0 ) {
		Uint32 black;
		Uint8 *pixels;
		black = SDL_MapRGB(screen->format, 0, 0, 0);
		pixels = (Uint8 *)screen->pixels;
		for ( i=0; i<screen->h; ++i ) {
			memset(pixels, black,
				screen->w*screen->format->BytesPerPixel);
			pixels += screen->pitch;
		}
		SDL_UnlockSurface(screen);
	}
}

#define NUM_POINTS 150

void Draw(SDL_Surface *screen)
{
	int i,rate,x,y,dx,dy;
	int psize = NUM_POINTS; 
	double sin_start = 0;
	double sin_amp = 100;
	Sint16 polygon_x[NUM_POINTS], polygon_y[NUM_POINTS];
	Sint16 polygon_alpha_x[4], polygon_alpha_y[4];
	SDL_Surface *texture;
	SDL_Surface *texture_alpha;
	FPSmanager fpsm;
	int width_half = screen->w/2;
	int height_half = screen->h/2;

	/* Load texture surfaces */
	texture = SDL_LoadBMP("texture.bmp");
	texture_alpha = SDL_LoadBMP("texture_alpha.bmp");
	SDL_SetAlpha(texture_alpha, SDL_SRCALPHA, 128);

	/* Initialize variables */
	srand((int)time(NULL));
	i=0;
	x=width_half;
	y=height_half;
	dx=7;
	dy=5;

	/* Initialize Framerate manager */  
	SDL_initFramerate(&fpsm);

	/* Polygon for blended texture */
	polygon_alpha_x[0]= 0;
	polygon_alpha_y[0]= 0;
	polygon_alpha_x[1]= width_half;
	polygon_alpha_y[1]= 0;
	polygon_alpha_x[2]= screen->w*2 / 3;
	polygon_alpha_y[2]= screen->h;
	polygon_alpha_x[3]= 0;
	polygon_alpha_y[3]= screen->h;

	/* Set/switch framerate */
	rate=25;
	SDL_setFramerate(&fpsm,rate);

	/* Drawing loop */
	while (1) {

		/* Generate wave polygon */
		sin_start++;
		polygon_x[0]= 0;
		polygon_y[0]= screen->h;
		polygon_x[1]= 0;
		polygon_y[1]= height_half;    
		for (i=2; i < psize -2 ; i++){
			polygon_x[i]= (screen->w  * (i-2)) / (psize -5) ;
			polygon_y[i]= (Sint16)(sin(sin_start/100) * 200) + height_half - (Sint16)(sin((i + sin_start) / 20) * sin_amp);
		}

		polygon_x[psize-2]= screen->w;
		polygon_y[psize-2]= height_half;
		polygon_x[psize-1]= screen->w;
		polygon_y[psize-1]= screen->h;

		/* Event handler */
		HandleEvent();

		/* Black screen */
		ClearScreen(screen);

		/* Move */
		x += dx;
		y += dy;

		/* Reflect */
		if ((x<0) || (x>screen->w)) { dx=-dx; }
		if ((y<0) || (y>screen->h)) { dy=-dy; }

		/* Draw */
		texturedPolygon(screen,polygon_x,polygon_y,psize,texture, -(screen->w  * (Sint16)(sin_start-2)) / (psize - 5), -(Sint16)(sin(sin_start/100) * 200));
		texturedPolygon(screen,polygon_alpha_x,polygon_alpha_y,4,texture_alpha,(Sint16)sin_start,-(Sint16)sin_start);

		/* Display by flipping screens */
		SDL_Flip(screen);

		/* Delay to fix rate */                   
		SDL_framerateDelay(&fpsm);  
	}
}

/* ======== */

int main ( int argc, char *argv[] )
{
	SDL_Surface *screen;
	int w, h;
	int desired_bpp;
	Uint32 video_flags;

	/* Title */
	fprintf (stderr,"texturedPolygon test\n");

	/* Set default options and check command-line */
	w = 640;
	h = 480;
	desired_bpp = 0;
	video_flags = 0;
	while ( argc > 1 ) {
		if ( strcmp(argv[1], "-width") == 0 ) {
			if ( argv[2] && ((w = atoi(argv[2])) > 0) ) {
				argv += 2;
				argc -= 2;
			} else {
				fprintf(stderr,
					"The -width option requires an argument\n");
				exit(1);
			}
		} else
			if ( strcmp(argv[1], "-height") == 0 ) {
				if ( argv[2] && ((h = atoi(argv[2])) > 0) ) {
					argv += 2;
					argc -= 2;
				} else {
					fprintf(stderr,
						"The -height option requires an argument\n");
					exit(1);
				}
			} else
				if ( strcmp(argv[1], "-bpp") == 0 ) {
					if ( argv[2] ) {
						desired_bpp = atoi(argv[2]);
						argv += 2;
						argc -= 2;
					} else {
						fprintf(stderr,
							"The -bpp option requires an argument\n");
						exit(1);
					}
				} else
					if ( strcmp(argv[1], "-warp") == 0 ) {
						video_flags |= SDL_HWPALETTE;
						argv += 1;
						argc -= 1;
					} else
						if ( strcmp(argv[1], "-hw") == 0 ) {
							video_flags |= SDL_HWSURFACE;
							argv += 1;
							argc -= 1;
						} else
							if ( strcmp(argv[1], "-fullscreen") == 0 ) {
								video_flags |= SDL_FULLSCREEN;
								argv += 1;
								argc -= 1;
							} else
								break;
	}

	/* Force double buffering */
	video_flags |= SDL_DOUBLEBUF;

	/* Initialize SDL */
	if ( SDL_Init(SDL_INIT_VIDEO) < 0 ) {
		fprintf(stderr,
			"Couldn't initialize SDL: %s\n", SDL_GetError());
		exit(1);
	}
	atexit(SDL_Quit);			/* Clean up on exit */

	/* Initialize the display */
	screen = SDL_SetVideoMode(w, h, desired_bpp, video_flags);
	if ( screen == NULL ) {
		fprintf(stderr, "Couldn't set %dx%dx%d video mode: %s\n",
			w, h, desired_bpp, SDL_GetError());
		exit(1);
	}

	/* Show some info */
	printf("Set %dx%dx%d mode\n",
		screen->w, screen->h, screen->format->BitsPerPixel);
	printf("Video surface located in %s memory.\n",
		(screen->flags&SDL_HWSURFACE) ? "video" : "system");

	/* Check for double buffering */
	if ( screen->flags & SDL_DOUBLEBUF ) {
		printf("Double-buffering enabled - good!\n");
	}

	/* Set the window manager title bar */
	SDL_WM_SetCaption("texturedPolygon test", "texturedPolygon");

	/* Do all the drawing work */
	Draw (screen);

	return(0);
}
