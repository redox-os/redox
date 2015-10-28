/* 

TestShrink.c: test program for shrink routines

(C) A. Schiffler, July 2006, zlib License

*/

#include <stdlib.h>
#include <stdio.h>
#include <string.h>

#include "SDL.h"

#ifdef WIN32
#include <windows.h>
#include "SDL_rotozoom.h"
#else
#include "SDL/SDL_rotozoom.h"
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
		Uint8 *pixels;
		pixels = (Uint8 *)screen->pixels;
		for ( i=0; i<screen->h; ++i ) {
			memset(pixels, 0,
				screen->w*screen->format->BytesPerPixel);
			pixels += screen->pitch;
		}
		SDL_UnlockSurface(screen);
	}
}


void ShrinkPicture (SDL_Surface *screen, SDL_Surface *picture) 
{
	SDL_Surface *shrink_picture;
	SDL_Rect dest;
	int factorx, factory;

	for (factorx=1; factorx<6; factorx++) {
		for (factory=1; factory<6; factory++) {

			HandleEvent();

			ClearScreen(screen);
			if ((shrink_picture=shrinkSurface (picture, factorx, factory))!=NULL) {
				dest.x = (screen->w - shrink_picture->w)/2;;
				dest.y = (screen->h - shrink_picture->h)/2;
				dest.w = shrink_picture->w;
				dest.h = shrink_picture->h;
				if ( SDL_BlitSurface(shrink_picture, NULL, screen, &dest) < 0 ) {
					fprintf(stderr, "Blit failed: %s\n", SDL_GetError());
					break;
				}
				SDL_FreeSurface(shrink_picture);
			}

			/* Display by flipping screens */
			SDL_Flip(screen);

			/* Pause for 0.25 sec */
			SDL_Delay(250);

		}
	}

}


void Draw (SDL_Surface *screen, int start)
{
	SDL_Surface *picture, *picture_again;
	char *bmpfile;

	/* --------- 8 bit test -------- */

	if (start<=1) {

		/* Message */
		fprintf (stderr,"Loading 8bit square image\n");

		/* Load the image into a surface */
		bmpfile = "sample8-box.bmp";
		fprintf(stderr, "Loading picture: %s\n", bmpfile);
		picture = SDL_LoadBMP(bmpfile);
		if ( picture == NULL ) {
			fprintf(stderr, "Couldn't load %s: %s\n", bmpfile, SDL_GetError());
			return;
		}


		fprintf (stderr,"1. shrink 8bit\n");
		ShrinkPicture(screen,picture);

		/* Free the picture */
		SDL_FreeSurface(picture);

	}

	if (start<=2) {

		/* Message */
		fprintf (stderr,"Loading 8bit image\n");

		/* Load the image into a surface */
		bmpfile = "sample8.bmp";
		fprintf(stderr, "Loading picture: %s\n", bmpfile);
		picture = SDL_LoadBMP(bmpfile);
		if ( picture == NULL ) {
			fprintf(stderr, "Couldn't load %s: %s\n", bmpfile, SDL_GetError());
			return;
		}


		fprintf (stderr,"1. shrink 8bit\n");
		ShrinkPicture(screen,picture);

		/* Free the picture */
		SDL_FreeSurface(picture);

	}

	/* -------- 24 bit test --------- */


	if (start<=3) {

		/* Message */
		fprintf (stderr,"Loading 24bit square image\n");

		/* Load the image into a surface */
		bmpfile = "sample24-box.bmp";
		fprintf(stderr, "Loading picture: %s\n", bmpfile);
		picture = SDL_LoadBMP(bmpfile);
		if ( picture == NULL ) {
			fprintf(stderr, "Couldn't load %s: %s\n", bmpfile, SDL_GetError());
			return;
		}

		fprintf (stderr,"2. shrink 24bit\n");
		ShrinkPicture(screen,picture);

		/* Free the picture */
		SDL_FreeSurface(picture);

	}

	if (start<=4) {

		/* Message */
		fprintf (stderr,"Loading 24bit image\n");

		/* Load the image into a surface */
		bmpfile = "sample24.bmp";
		fprintf(stderr, "Loading picture: %s\n", bmpfile);
		picture = SDL_LoadBMP(bmpfile);
		if ( picture == NULL ) {
			fprintf(stderr, "Couldn't load %s: %s\n", bmpfile, SDL_GetError());
			return;
		}

		fprintf (stderr,"2. shrink 24bit\n");
		ShrinkPicture(screen,picture);

		/* Free the picture */
		SDL_FreeSurface(picture);

	}

	/* -------- 32 bit test --------- */

	if (start<=5) {

		/* Message */
		fprintf (stderr,"Loading 24bit square image\n");

		/* Load the image into a surface */
		bmpfile = "sample24-box.bmp";
		fprintf(stderr, "Loading picture: %s\n", bmpfile);
		picture = SDL_LoadBMP(bmpfile);
		if ( picture == NULL ) {
			fprintf(stderr, "Couldn't load %s: %s\n", bmpfile, SDL_GetError());
			return;
		}

		/* New source surface is 32bit with defined RGBA ordering */
		/* Much faster to do this once rather than the routine on the fly */
		fprintf (stderr,"Converting 24bit image into 32bit RGBA surface ...\n");
		picture_again = SDL_CreateRGBSurface(SDL_SWSURFACE, picture->w, picture->h, 32, 0x000000ff, 0x0000ff00, 0x00ff0000, 0xff000000);
		SDL_BlitSurface(picture,NULL,picture_again,NULL);

		/* Message */
		fprintf (stderr,"3. shrink 32bit \n");
		ShrinkPicture(screen,picture_again);

		/* Free the picture2 */
		SDL_FreeSurface(picture_again);
		SDL_FreeSurface(picture);

	}

	if (start<=6) {

		/* Message */
		fprintf (stderr,"Loading 24bit image\n");

		/* Load the image into a surface */
		bmpfile = "sample24.bmp";
		fprintf(stderr, "Loading picture: %s\n", bmpfile);
		picture = SDL_LoadBMP(bmpfile);
		if ( picture == NULL ) {
			fprintf(stderr, "Couldn't load %s: %s\n", bmpfile, SDL_GetError());
			return;
		}

		/* New source surface is 32bit with defined RGBA ordering */
		/* Much faster to do this once rather than the routine on the fly */
		fprintf (stderr,"Converting 24bit image into 32bit RGBA surface ...\n");
		picture_again = SDL_CreateRGBSurface(SDL_SWSURFACE, picture->w, picture->h, 32, 0x000000ff, 0x0000ff00, 0x00ff0000, 0xff000000);
		SDL_BlitSurface(picture,NULL,picture_again,NULL);

		/* Message */
		fprintf (stderr,"3. shrink 32bit \n");
		ShrinkPicture(screen,picture_again);

		/* Free the picture2 */
		SDL_FreeSurface(picture_again);
		SDL_FreeSurface(picture);

	}

	return;
}

int main ( int argc, char *argv[] )
{
	SDL_Surface *screen;
	int w, h;
	int desired_bpp;
	Uint32 video_flags;
	int start;

	/* Title */
	fprintf (stderr,"SDL_rotozoom test\n");

	/* Set default options and check command-line */
	w = 640;
	h = 480;
	desired_bpp = 0;
	video_flags = 0;
	start = 1;
	while ( argc > 1 ) {
		if ( strcmp(argv[1], "-start") == 0 ) {
			if ( argv[2] && ((start = atoi(argv[2])) > 0) ) {
				argv += 2;
				argc -= 2;
			} else {
				fprintf(stderr,
					"The -start option requires an argument\n");
				exit(1);
			}
		} else
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
									if (( strcmp(argv[1], "-help") == 0 ) || (strcmp(argv[1], "--help") == 0)) {
										printf ("Usage:\n");
										printf (" -start #	Set starting test number (1=8bit, 3=24bit, 5=32bit)\n");
										printf (" -width #	Screen width (Default: %i)\n",w);
										printf (" -height #	Screen height (Default: %i)\n",h);
										printf (" -bpp #	Screen bpp\n");
										printf (" -warp		Enable hardware palette\n");
										printf (" -hw		Enable hardware surface\n");
										printf (" -fullscreen	Enable fullscreen mode\n");

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
	SDL_WM_SetCaption("SDL_rotozoom shrink test", "shrink");

	/* Do all the drawing work */
	Draw (screen, start);

	return(0);
}
