/* 

TestABGR.c: test GFX behavior on byteordering

(C) A. Schiffler, 2005, zlib License

*/

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <time.h>

#include "SDL.h"

#ifdef WIN32
 #include <windows.h>
 #include "SDL_gfxPrimitives.h"
#else
 #include "SDL/SDL_gfxPrimitives.h"
#endif

#define WIDTH	400
#define HEIGHT	424

void WaitForEvent()
{
	int done;
	SDL_Event event; 

	/* Check for events */
	done = 0;
	while (!done) {
		SDL_PollEvent(&event);
		switch (event.type) {
			 case SDL_KEYDOWN:
			 case SDL_QUIT:
				 done = 1;
				 break;
		}
		SDL_Delay(100);
	}
}

int main(int argc, char *argv[])
{
	SDL_Surface *screen;
	Uint8  video_bpp;
	Uint32 videoflags;
	char title[64];
	char message[64];
	SDL_Rect r;
	int y;
	SDL_Surface* s;

	/* Define masking bytes */
	Uint32 mask1 = 0xff000000; 
	Uint32 mask2 = 0x00ff0000;
	Uint32 mask3 = 0x0000ff00; 
	Uint32 mask4 = 0x000000ff;

	/* Generate title+message strings */
	sprintf (title, "TestABGR - v%i.%i.%i", SDL_GFXPRIMITIVES_MAJOR, SDL_GFXPRIMITIVES_MINOR, SDL_GFXPRIMITIVES_MICRO);
	sprintf (message, "%s", "Left:RGBA Right:ABGR Top:Solid Bottom:Transparent");

	/* Initialize SDL */
	if ( SDL_Init(SDL_INIT_VIDEO) < 0 ) {
		fprintf(stderr, "Couldn't initialize SDL: %s\n",SDL_GetError());
		exit(1);
	}
	atexit(SDL_Quit);

	video_bpp = 32;
	videoflags = SDL_SWSURFACE | SDL_SRCALPHA | SDL_RESIZABLE;
	while ( argc > 1 ) {
		--argc;
		if ( strcmp(argv[argc-1], "-bpp") == 0 ) {
			video_bpp = atoi(argv[argc]);
			--argc;
		} else
			if ( strcmp(argv[argc], "-hw") == 0 ) {
				videoflags |= SDL_HWSURFACE;
			} else
				if ( strcmp(argv[argc], "-warp") == 0 ) {
					videoflags |= SDL_HWPALETTE;
				} else
					if ( strcmp(argv[argc], "-fullscreen") == 0 ) {
						videoflags |= SDL_FULLSCREEN;
					} else {
						fprintf(stderr, 
							"Usage: %s [-bpp N] [-warp] [-hw] [-fullscreen]\n",
							argv[0]);
						exit(1);
					}
	}

	/* Set video mode */
	if ( (screen=SDL_SetVideoMode(WIDTH, HEIGHT, video_bpp, videoflags)) == NULL ) {
		fprintf(stderr, "Couldn't set %ix%i %i bpp video mode: %s\n",WIDTH,HEIGHT,video_bpp,SDL_GetError());
		exit(2);
	}

	/* Use alpha blending */
	SDL_SetAlpha(screen, SDL_SRCALPHA, 0);

	/* Set title for window */
	SDL_WM_SetCaption(title,title);

	/* Draw some white stripes as background */
	for (y=0; y<400; y += 20) {
		boxRGBA(SDL_GetVideoSurface(), 0, y+10, 400, y+20, 255, 255, 255, 255);
	}

	/* Solid color test */  
	s = SDL_CreateRGBSurface(0, 200, 200, 32, mask1, mask2, mask3, mask4);
	filledEllipseRGBA(s, 0, 0, 100, 100, 255, 0, 0, 255); // red
	r.x = 0; r.y = 0;
	SDL_BlitSurface(s, 0, SDL_GetVideoSurface(), &r);

	s = SDL_CreateRGBSurface(0, 200, 200, 32, mask1, mask2, mask3, mask4);
	filledEllipseRGBA(s, 0, 0, 100, 100, 0, 255, 0, 255); // green
	r.x = 0; r.y = 100;
	SDL_BlitSurface(s, 0, SDL_GetVideoSurface(), &r);

	s = SDL_CreateRGBSurface(0, 200, 200, 32, mask1, mask2, mask3, mask4);
	filledEllipseRGBA(s, 0, 0, 100, 100, 0, 0, 255, 255); // blue
	r.x = 100; r.y = 0;
	SDL_BlitSurface(s, 0, SDL_GetVideoSurface(), &r);

	s = SDL_CreateRGBSurface(0, 200, 200, 32, mask1, mask2, mask3, mask4);
	filledEllipseRGBA(s, 0, 0, 100, 100, 255, 255, 255, 255); // white
	r.x = 100; r.y = 100;
	SDL_BlitSurface(s, 0, SDL_GetVideoSurface(), &r);

	s = SDL_CreateRGBSurface(0, 200, 200, 32, mask4, mask3, mask2, mask1);
	filledEllipseRGBA(s, 0, 0, 100, 100, 255, 0, 0, 255); // red
	r.x = 200; r.y = 0;
	SDL_BlitSurface(s, 0, SDL_GetVideoSurface(), &r);

	s = SDL_CreateRGBSurface(0, 200, 200, 32, mask4, mask3, mask2, mask1);
	filledEllipseRGBA(s, 0, 0, 100, 100, 0, 255, 0, 255); // green
	r.x = 200; r.y = 100;
	SDL_BlitSurface(s, 0, SDL_GetVideoSurface(), &r);

	s = SDL_CreateRGBSurface(0, 200, 200, 32, mask4, mask3, mask2, mask1);
	filledEllipseRGBA(s, 0, 0, 100, 100, 0, 0, 255, 255); // blue
	r.x = 300; r.y = 0;
	SDL_BlitSurface(s, 0, SDL_GetVideoSurface(), &r);

	s = SDL_CreateRGBSurface(0, 200, 200, 32, mask4, mask3, mask2, mask1);
	filledEllipseRGBA(s, 0, 0, 100, 100, 255, 255, 255, 255); // white
	r.x = 300; r.y = 100;
	SDL_BlitSurface(s, 0, SDL_GetVideoSurface(), &r);


	/* Transparent color test */
	s = SDL_CreateRGBSurface(0, 200, 200, 32, mask1, mask2, mask3, mask4);
	filledEllipseRGBA(s, 0, 0, 100, 100, 255, 0, 0, 200); // red+trans
	r.x = 0; r.y = 200;
	SDL_BlitSurface(s, 0, SDL_GetVideoSurface(), &r);

	s = SDL_CreateRGBSurface(0, 200, 200, 32, mask1, mask2, mask3, mask4);
	filledEllipseRGBA(s, 0, 0, 100, 100, 0, 255, 0, 200); // green+trans
	r.x = 0; r.y = 300;
	SDL_BlitSurface(s, 0, SDL_GetVideoSurface(), &r);

	s = SDL_CreateRGBSurface(0, 200, 200, 32, mask1, mask2, mask3, mask4);
	filledEllipseRGBA(s, 0, 0, 100, 100, 0, 0, 255, 200); // blue+trans
	r.x = 100; r.y = 200;
	SDL_BlitSurface(s, 0, SDL_GetVideoSurface(), &r);

	s = SDL_CreateRGBSurface(0, 200, 200, 32, mask1, mask2, mask3, mask4);
	filledEllipseRGBA(s, 0, 0, 100, 100, 255, 255, 255, 200); // white+trans
	r.x = 100; r.y = 300;
	SDL_BlitSurface(s, 0, SDL_GetVideoSurface(), &r);

	s = SDL_CreateRGBSurface(0, 200, 200, 32, mask4, mask3, mask2, mask1);
	filledEllipseRGBA(s, 0, 0, 100, 100, 255, 0, 0, 200); // red+trans
	r.x = 200; r.y = 200;
	SDL_BlitSurface(s, 0, SDL_GetVideoSurface(), &r);

	s = SDL_CreateRGBSurface(0, 200, 200, 32, mask4, mask3, mask2, mask1);
	filledEllipseRGBA(s, 0, 0, 100, 100, 0, 255, 0, 200); // green+trans
	r.x = 200; r.y = 300;
	SDL_BlitSurface(s, 0, SDL_GetVideoSurface(), &r);

	s = SDL_CreateRGBSurface(0, 200, 200, 32, mask4, mask3, mask2, mask1);
	filledEllipseRGBA(s, 0, 0, 100, 100, 0, 0, 255, 200); // blue+trans
	r.x = 300; r.y = 200;
	SDL_BlitSurface(s, 0, SDL_GetVideoSurface(), &r);

	s = SDL_CreateRGBSurface(0, 200, 200, 32, mask4, mask3, mask2, mask1);
	filledEllipseRGBA(s, 0, 0, 100, 100, 255, 255, 255, 200); // white+trans
	r.x = 300; r.y = 300;
	SDL_BlitSurface(s, 0, SDL_GetVideoSurface(), &r);

	stringRGBA (screen, WIDTH/2-4*strlen(message),HEIGHT-12,message,255,255,255,255);

	SDL_Flip(SDL_GetVideoSurface());
	
	WaitForEvent();

	return 0;
}
