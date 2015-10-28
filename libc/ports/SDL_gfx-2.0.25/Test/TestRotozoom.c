/* 

TestRotozoom.c: test program for rotozoom routines

(C) A. Schiffler, 2001-2011, zlib License

*/

#include <stdlib.h>
#include <stdio.h>
#include <string.h>

#include "SDL.h"

#ifdef WIN32
#include <windows.h>
#include "SDL_gfxPrimitives.h"
#include "SDL_rotozoom.h"
#else
#include "SDL/SDL_gfxPrimitives.h"
#include "SDL/SDL_rotozoom.h"
#endif

/* Pause flag */
int pause = 0;

/* Custom rotation setup */
double custom_angle=0.0;
double custom_fx=1.0;
double custom_fy=1.0;
int custom_smooth=0;

/* Delay between frames */
int delay;

/* Curren message */
char *messageText;

void HandleEvent()
{
	SDL_Event event; 

	/* Check for events */
	while ( SDL_PollEvent(&event) || pause ) {
		switch (event.type) {
			 case SDL_KEYDOWN:
			        /* Space pauses/unpauses */
			 	if ((event.key.state==SDL_PRESSED) && 
			 	    (event.key.keysym.sym==SDLK_SPACE)) {
			 		pause = !pause;
			 		if (pause) {
			 			printf ("Paused ...\n");
			 		}
			 	} else {
			 		exit(0);
			 	}			 	
			 	break;
			 case SDL_QUIT:
				 exit(0);
				 break;
		}
		
		if (pause) {
			SDL_Delay(100);
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

#define POSITION_CENTER		1
#define POSITION_BOTTOMRIGHT	2

void RotatePicture (SDL_Surface *screen, SDL_Surface *picture, int rotate, int flip, int smooth, int position) 
{
	SDL_Surface *rotozoom_picture;
	SDL_Rect dest;
	int framecount, framemax, frameinc;
	double angle, zoomf, zoomfx, zoomfy;

	printf("%s\n", messageText);

	/* Rotate and display the picture */
	framemax=4*360; 
	frameinc=1;
	for (framecount=-360; framecount<framemax; framecount += frameinc) {
		if ((framecount % 360)==0) frameinc++;
		HandleEvent();
		ClearScreen(screen);
		zoomf=(float)(framecount+2*360)/(float)framemax;
		zoomf=1.5*zoomf*zoomf;
		/* Are we in flipping mode? */
		if (flip) {
			/* Flip X factor */
			if (flip & 1) {
				zoomfx=-zoomf;
			} else {
				zoomfx=zoomf;
			}
			/* Flip Y factor */
			if (flip & 2) {
				zoomfy=-zoomf;
			} else {
				zoomfy=zoomf;
			}
			angle=framecount*rotate;
			if (((framecount % 120)==0) || (delay>0)) {
				printf ("  Frame: %i   Rotate: angle=%.2f  Zoom: x=%.2f y=%.2f\n",framecount,angle,zoomfx,zoomfy);
			}
			if ((rotozoom_picture=rotozoomSurfaceXY (picture, angle, zoomfx, zoomfy, smooth))!=NULL) {
				switch (position) {
					case POSITION_CENTER:
						dest.x = (screen->w - rotozoom_picture->w)/2;
						dest.y = (screen->h - rotozoom_picture->h)/2;
						break;
					case POSITION_BOTTOMRIGHT:
						dest.x = (screen->w/2) - rotozoom_picture->w;
						dest.y = (screen->h/2) - rotozoom_picture->h;
						break;
				}
				dest.w = rotozoom_picture->w;
				dest.h = rotozoom_picture->h;
				if ( SDL_BlitSurface(rotozoom_picture, NULL, screen, &dest) < 0 ) {
					fprintf(stderr, "Blit failed: %s\n", SDL_GetError());
					break;
				}
				SDL_FreeSurface(rotozoom_picture);
		 }
		} else {
			angle=framecount*rotate;
			if ((framecount % 120)==0) {
				printf ("  Frame: %i   Rotate: angle=%.2f  Zoom: f=%.2f \n",framecount,angle,zoomf);
			}
			if ((rotozoom_picture=rotozoomSurface (picture, angle, zoomf, smooth))!=NULL) {
				switch (position) {
					case POSITION_CENTER:
						dest.x = (screen->w - rotozoom_picture->w)/2;
						dest.y = (screen->h - rotozoom_picture->h)/2;
						break;
					case POSITION_BOTTOMRIGHT:
						dest.x = (screen->w/2) - rotozoom_picture->w;
						dest.y = (screen->h/2) - rotozoom_picture->h;
						break;
				}
				dest.w = rotozoom_picture->w;
				dest.h = rotozoom_picture->h;
				if ( SDL_BlitSurface(rotozoom_picture, NULL, screen, &dest) < 0 ) {
					fprintf(stderr, "Blit failed: %s\n", SDL_GetError());
					break;
				}
				SDL_FreeSurface(rotozoom_picture);
		 }
		}

		stringRGBA(screen, 8, 8, messageText, 255, 255, 255, 255);

		/* Display by flipping screens */
		SDL_Flip(screen);

		/* Maybe delay */
		if (delay>0) {
			SDL_Delay(delay);
		}
	}

	if (rotate) {
		/* Final display with angle=0 */
		HandleEvent();
		ClearScreen(screen);
		if (flip) {
			if ((rotozoom_picture=rotozoomSurfaceXY (picture, 0.01, zoomfx, zoomfy, smooth))!=NULL) {
				dest.x = (screen->w - rotozoom_picture->w)/2;;
				dest.y = (screen->h - rotozoom_picture->h)/2;
				dest.w = rotozoom_picture->w;
				dest.h = rotozoom_picture->h;
				if ( SDL_BlitSurface(rotozoom_picture, NULL, screen, &dest) < 0 ) {
					fprintf(stderr, "Blit failed: %s\n", SDL_GetError());
					return;
				}
				SDL_FreeSurface(rotozoom_picture);
			}		
		} else {
			if ((rotozoom_picture=rotozoomSurface (picture, 0.01, zoomf, smooth))!=NULL) {
				dest.x = (screen->w - rotozoom_picture->w)/2;;
				dest.y = (screen->h - rotozoom_picture->h)/2;
				dest.w = rotozoom_picture->w;
				dest.h = rotozoom_picture->h;
				if ( SDL_BlitSurface(rotozoom_picture, NULL, screen, &dest) < 0 ) {
					fprintf(stderr, "Blit failed: %s\n", SDL_GetError());
					return;
				}
				SDL_FreeSurface(rotozoom_picture);
			}		
		}

		stringRGBA(screen, 8, 8, messageText, 255, 255, 255, 255);

		/* Display by flipping screens */
		SDL_Flip(screen);

		/* Maybe delay */
		if (delay>0) {
			SDL_Delay(delay);
		}
	}

	/* Pause for a sec */
	SDL_Delay(1000);
}

void ZoomPicture (SDL_Surface *screen, SDL_Surface *picture, int smooth) 
{
	SDL_Surface *rotozoom_picture;
	SDL_Rect dest;
	int framecount, framemax, frameinc;
	double zoomxf,zoomyf;

	printf("%s\n", messageText);

	/* Zoom and display the picture */
	framemax=4*360; frameinc=1;
	for (framecount=360; framecount<framemax; framecount += frameinc) {
		if ((framecount % 360)==0) frameinc++;
		HandleEvent();
		ClearScreen(screen);
		zoomxf=(float)framecount/(float)framemax;
		zoomxf=1.5*zoomxf*zoomxf;
		zoomyf=0.5+fabs(1.0*sin((double)framecount/80.0));
		if ((framecount % 120)==0) {
			printf ("  Frame: %i   Zoom: x=%.2f y=%.2f\n",framecount,zoomxf,zoomyf);
		}
		if ((rotozoom_picture=zoomSurface (picture, zoomxf, zoomyf, smooth))!=NULL) {
			dest.x = (screen->w - rotozoom_picture->w)/2;;
			dest.y = (screen->h - rotozoom_picture->h)/2;
			dest.w = rotozoom_picture->w;
			dest.h = rotozoom_picture->h;
			if ( SDL_BlitSurface(rotozoom_picture, NULL, screen, &dest) < 0 ) {
				fprintf(stderr, "Blit failed: %s\n", SDL_GetError());
				break;
			}
			SDL_FreeSurface(rotozoom_picture);
		}

		stringRGBA(screen, 8, 8, messageText, 255, 255, 255, 255);

		/* Display by flipping screens */
		SDL_Flip(screen);

		/* Maybe delay */
		if (delay>0) {
			SDL_Delay(delay);
		}
	}

	/* Pause for a sec */
	SDL_Delay(1000);
}

void RotatePicture90Degrees (SDL_Surface *screen, SDL_Surface *picture) 
{
	SDL_Surface *rotozoom_picture;
	SDL_Rect dest;
	int framecount, framemax, frameinc;
	int numClockwiseTurns;

	printf("%s\n", messageText);

	/* Rotate and display the picture */
	framemax = 21;
	frameinc = 1;
	numClockwiseTurns = -4;
	for (framecount=0; framecount<framemax; framecount += frameinc) {
		HandleEvent();
		ClearScreen(screen);
		printf ("  Frame: %i   Rotate90: %i clockwise turns\n",framecount,numClockwiseTurns+4);
		if ((rotozoom_picture=rotateSurface90Degrees(picture, numClockwiseTurns))!=NULL) {
			dest.x = (screen->w - rotozoom_picture->w)/2;;
			dest.y = (screen->h - rotozoom_picture->h)/2;
			dest.w = rotozoom_picture->w;
			dest.h = rotozoom_picture->h;
			if (SDL_BlitSurface(rotozoom_picture, NULL, screen, &dest) < 0 ) {
				fprintf(stderr, "Blit failed: %s\n", SDL_GetError());
				break;
			}
			SDL_FreeSurface(rotozoom_picture);
		}

		stringRGBA(screen, 8, 8, messageText, 255, 255, 255, 255);

		/* Display by flipping screens */
		SDL_Flip(screen);

		/* Always delay */
		SDL_Delay(333);
		if (delay>0) {
			SDL_Delay(delay);
		}

		numClockwiseTurns++;
	}

	/* Pause for a sec */
	SDL_Delay(1000);
}

#define ROTATE_OFF	0
#define ROTATE_ON	1

#define FLIP_OFF	0
#define FLIP_X		1
#define FLIP_Y		2
#define FLIP_XY		3

void CustomTest(SDL_Surface *screen, SDL_Surface *picture, double a, double x, double y, int smooth){
	SDL_Surface *rotozoom_picture;
	SDL_Rect dest;

	printf("%s\n", messageText);
	printf ("  Frame: C   Rotate: angle=%.2f  Zoom: fx=%.2f fy=%.2f \n",a,x,y);

	HandleEvent();
	ClearScreen(screen);
	if ((rotozoom_picture=rotozoomSurfaceXY (picture, a, x, y, smooth))!=NULL) {
		dest.x = (screen->w - rotozoom_picture->w)/2;;
		dest.y = (screen->h - rotozoom_picture->h)/2;
		dest.w = rotozoom_picture->w;
		dest.h = rotozoom_picture->h;
		if ( SDL_BlitSurface(rotozoom_picture, NULL, screen, &dest) < 0 ) {
			fprintf(stderr, "Blit failed: %s\n", SDL_GetError());
			return;
		}
		SDL_FreeSurface(rotozoom_picture);
	}

	/* Display by flipping screens */
	SDL_Flip(screen);

	/* Maybe delay */
	if (delay>0) {
		SDL_Delay(delay);
	}

	SDL_Delay(1000);		
}


void AccuracyTest1(SDL_Surface *screen)
{
	SDL_Surface* testx1;
	SDL_Surface* testx2;
	SDL_Rect r;
	SDL_Rect target;
	SDL_Surface* ref;
	int size, halfsize, doublesize;

	printf("%s\n", messageText);
	for (size = 10; size < 200; size += 2)
	{		
		HandleEvent();
		ClearScreen(screen);

		stringRGBA(screen, 8, 8, messageText, 255, 255, 255, 255);

		halfsize = size / 2;
		doublesize = size * 2;
		printf ("  zoom from %i to %i\n", size, doublesize);

		// Set up test surfaces
		testx1 = SDL_CreateRGBSurface(SDL_SWSURFACE, size, size, 24, 0, 0, 0, 0);

		r.x = 0;
		r.y = 0;
		r.w = halfsize;
		r.h = halfsize;
		SDL_FillRect(testx1, &r, 0x339933);

		r.x = halfsize;
		r.y = halfsize;
		SDL_FillRect(testx1, &r, 0x993399);

		r.x = 0;
		r.y = halfsize;
		SDL_FillRect(testx1, &r, 0x999933);

		r.x = halfsize;
		r.y = 0;
		SDL_FillRect(testx1, &r, 0x333399);

		testx2 = zoomSurface(testx1, 2.0, 2.0, 0);

		ref = SDL_CreateRGBSurface(SDL_SWSURFACE, size, size, 24, 0, 0, 0, 0);
		r.w = size;
		r.h = size;
		r.x = 0;
		r.y = 0;
		SDL_FillRect(ref, &r, 0xFFFFFF);

		/* Validation display */

		target.x = 0;
		target.y = 20;
		target.w = 0;
		target.h = 0;
		SDL_BlitSurface(testx1, 0, screen, &target);

		target.x = size;
		target.y = 20;
		SDL_BlitSurface(ref, 0, screen, &target);

		target.x = 0;
		target.y = 20 + size;
		SDL_BlitSurface(ref, 0, screen, &target);

		target.x = doublesize + 20;
		target.y = 20;
		SDL_BlitSurface(testx2, 0, screen, &target);

		target.x = doublesize + doublesize + 20;
		target.y = 20;
		SDL_BlitSurface(ref, 0, screen, &target);

		target.x = doublesize + 20;
		target.y = 20 + doublesize;
		SDL_BlitSurface(ref, 0, screen, &target);

		SDL_FreeSurface(testx1);
		SDL_FreeSurface(testx2);
		SDL_FreeSurface(ref);

		/* Display by flipping screens */
		SDL_Flip(screen);

		/* Always delay */
		SDL_Delay(250);

		/* Maybe add extra delay */
		if (delay>0) {
			SDL_Delay(delay);
		}
	}

	SDL_Delay(1000);
}

void AccuracyTest2(SDL_Surface *screen, SDL_Surface *picture)
{
	SDL_Surface *zoomed1, *zoomed2, *zoomed3, *zoomed4;
	int factor;
	int neww, newh;
	SDL_Rect target;

	printf("%s\n", messageText);
	for (factor = 2; factor < 64; factor += 1)
	{		
		HandleEvent();
		ClearScreen(screen);

		stringRGBA(screen, 8, 8, messageText, 255, 255, 255, 255);

		neww = picture->w * factor;
		newh = picture->h * factor;
		printf ("  zoom %ix%i to %ix%i\n", picture->w, picture->h, neww, newh);

		zoomed1 = zoomSurface(picture,  (float)factor,  (float)factor, 0);
		zoomed2 = zoomSurface(picture,  (float)factor,  (float)factor, 1);
		zoomed3 = zoomSurface(picture,  (float)factor, -(float)factor, 1);
		zoomed4 = zoomSurface(picture, -(float)factor,  (float)factor, 1);

		target.x = screen->w/2 - zoomed1->w;
		target.y = screen->h/2 - zoomed1->h;
		target.w = zoomed1->w;
		target.h = zoomed1->h;
		SDL_BlitSurface(zoomed1, 0, screen, &target);
		target.x = screen->w/2;
		target.y = screen->h/2;
		SDL_BlitSurface(zoomed2, 0, screen, &target);
		target.x = screen->w/2 - zoomed3->w;
		target.y = screen->h/2;
		SDL_BlitSurface(zoomed4, 0, screen, &target);
		target.x = screen->w/2;
		target.y = screen->h/2 - zoomed4->h;
		SDL_BlitSurface(zoomed3, 0, screen, &target);

		SDL_FreeSurface(zoomed1);
		SDL_FreeSurface(zoomed2);
		SDL_FreeSurface(zoomed3);
		SDL_FreeSurface(zoomed4);

		/* Display by flipping screens */
		SDL_Flip(screen);

		/* Always delay */
		SDL_Delay(250);

		/* Maybe add extra delay */
		if (delay>0) {
			SDL_Delay(delay);
		}
	}

	SDL_Delay(1000);
}


void Draw (SDL_Surface *screen, int start, int end)
{
	SDL_Surface *picture, *picture_again;
	char *bmpfile;

	/* Define masking bytes */
#if SDL_BYTEORDER == SDL_BIG_ENDIAN
	Uint32 rmask = 0xff000000; 
	Uint32 gmask = 0x00ff0000;
	Uint32 bmask = 0x0000ff00; 
	Uint32 amask = 0x000000ff;
#else
	Uint32 amask = 0xff000000; 
	Uint32 bmask = 0x00ff0000;
	Uint32 gmask = 0x0000ff00; 
	Uint32 rmask = 0x000000ff;
#endif

	/* --------- 8 bit tests -------- */

	if (start<=6) {

		/* Message */
		printf("8 bit tests ...\n");

		/* Load the image into a surface */
		bmpfile = "sample8.bmp";
		printf("Loading picture: %s\n", bmpfile);
		picture = SDL_LoadBMP(bmpfile);
		if ( picture == NULL ) {
			fprintf(stderr, "Couldn't load %s: %s\n", bmpfile, SDL_GetError());
			return;
		}
		
		/* Add white frame */
		rectangleColor(picture, 0, 0, picture->w-1, picture->h-1, 0xffffffff);

                if (start <= 1) {
			sprintf(messageText, "1.  rotozoom: Rotating and zooming");
			RotatePicture(screen,picture,ROTATE_ON,FLIP_OFF,SMOOTHING_OFF,POSITION_CENTER);
		}
		if (end == 1) goto done8bit;

                if (start <= 2) {
			sprintf(messageText, "2.  rotozoom: Just zooming (angle=0)");
			RotatePicture(screen,picture,ROTATE_OFF,FLIP_OFF,SMOOTHING_OFF,POSITION_CENTER);
			RotatePicture(screen,picture,ROTATE_OFF,FLIP_OFF,SMOOTHING_OFF,POSITION_BOTTOMRIGHT);
		}
		if (end == 2) goto done8bit;

                if (start <= 3) {
			sprintf(messageText, "3.  zoom: Just zooming");
			ZoomPicture(screen,picture,SMOOTHING_OFF);
		}
		if (end == 3) goto done8bit;

                if (start <= 4) {
			sprintf(messageText, "4.  rotozoom: Rotating and zooming, interpolation on but unused");
			RotatePicture(screen,picture,ROTATE_ON,FLIP_OFF,SMOOTHING_ON,POSITION_CENTER);
		}
		if (end == 4) goto done8bit;

                if (start <= 5) {
			sprintf(messageText, "5.  rotozoom: Just zooming (angle=0), interpolation on but unused");
			RotatePicture(screen,picture,ROTATE_OFF,FLIP_OFF,SMOOTHING_ON,POSITION_CENTER);
			RotatePicture(screen,picture,ROTATE_OFF,FLIP_OFF,SMOOTHING_ON,POSITION_BOTTOMRIGHT);
		}
		if (end == 5) goto done8bit;

                if (start <= 6) {
			sprintf(messageText, "6.  zoom: Just zooming, interpolation on but unused");
			ZoomPicture(screen,picture,SMOOTHING_ON);
		}
		if (end == 6) goto done8bit;

		done8bit:
		
		/* Free the picture */
		SDL_FreeSurface(picture);
		
		if (end <= 6) return;
	}

	/* -------- 24 bit test --------- */

	if (start<=12) {

		/* Message */
		printf("24 bit tests ...\n");

		/* Load the image into a surface */
		bmpfile = "sample24.bmp";
		printf("Loading picture: %s\n", bmpfile);
		picture = SDL_LoadBMP(bmpfile);
		if ( picture == NULL ) {
			fprintf(stderr, "Couldn't load %s: %s\n", bmpfile, SDL_GetError());
			return;
		}
		
		/* Add white frame */
		rectangleColor(picture, 0, 0, picture->w-1, picture->h-1, 0xffffffff);

                if (start <= 7) {
			sprintf(messageText, "7.  rotozoom: Rotating and zooming, no interpolation");
		  	RotatePicture(screen,picture,ROTATE_ON,FLIP_OFF,SMOOTHING_OFF,POSITION_CENTER);
		}
		if (end == 7) goto done24bit;

                if (start <= 8) {
			sprintf(messageText, "8a.  rotozoom: Just zooming (angle=0), no interpolation, centered");
			RotatePicture(screen,picture,ROTATE_OFF,FLIP_OFF,SMOOTHING_OFF,POSITION_CENTER);
			sprintf(messageText, "8b.  rotozoom: Just zooming (angle=0), no interpolation, corner");
			RotatePicture(screen,picture,ROTATE_OFF,FLIP_OFF,SMOOTHING_OFF,POSITION_BOTTOMRIGHT);
			sprintf(messageText, "8c.  rotozoom: Just zooming (angle=0), X flip, no interpolation, centered");
			RotatePicture(screen,picture,ROTATE_OFF,FLIP_X,SMOOTHING_OFF,POSITION_CENTER);
			sprintf(messageText, "8d.  rotozoom: Just zooming (angle=0), Y flip, no interpolation, centered");
			RotatePicture(screen,picture,ROTATE_OFF,FLIP_Y,SMOOTHING_OFF,POSITION_CENTER);
			sprintf(messageText, "8e.  rotozoom: Just zooming (angle=0), XY flip, no interpolation, centered");
			RotatePicture(screen,picture,ROTATE_OFF,FLIP_XY,SMOOTHING_OFF,POSITION_CENTER);
		}
		if (end == 8) goto done24bit;

                if (start <= 9) {
  			sprintf(messageText, "9.  zoom: Just zooming, no interpolation");
  			ZoomPicture(screen,picture,SMOOTHING_OFF);
		}
		if (end == 9) goto done24bit;

                if (start <= 10) {
			sprintf(messageText, "10. rotozoom: Rotating and zooming, with interpolation");
			RotatePicture(screen,picture,ROTATE_ON,FLIP_OFF,SMOOTHING_ON,POSITION_CENTER);
		}
		if (end == 10) goto done24bit;

                if (start <= 11) {
			sprintf(messageText, "11a. rotozoom: Just zooming (angle=0), with interpolation, centered");
			RotatePicture(screen,picture,ROTATE_OFF,FLIP_OFF,SMOOTHING_ON,POSITION_CENTER);
			sprintf(messageText, "11b. rotozoom: Just zooming (angle=0), with interpolation, corner");
			RotatePicture(screen,picture,ROTATE_OFF,FLIP_OFF,SMOOTHING_ON,POSITION_BOTTOMRIGHT);
			sprintf(messageText, "11c. rotozoom: Just zooming (angle=0), X flip, with interpolation, corner");
			RotatePicture(screen,picture,ROTATE_OFF,FLIP_X,SMOOTHING_ON,POSITION_CENTER);
			sprintf(messageText, "11d. rotozoom: Just zooming (angle=0), Y flip, with interpolation, corner");
			RotatePicture(screen,picture,ROTATE_OFF,FLIP_Y,SMOOTHING_ON,POSITION_CENTER);
			sprintf(messageText, "11e. rotozoom: Just zooming (angle=0), XY flip, with interpolation, corner");
			RotatePicture(screen,picture,ROTATE_OFF,FLIP_XY,SMOOTHING_ON,POSITION_CENTER);
		}
		if (end == 11) goto done24bit;

                if (start <= 12) {
			sprintf(messageText, "12. zoom: Just zooming, with interpolation");
			ZoomPicture(screen,picture,SMOOTHING_ON);
		}
		if (end == 12) goto done24bit;
		
		done24bit:

		/* Free the picture */
		SDL_FreeSurface(picture);
		
		if (end <= 12) return;
	}

	/* -------- 32 bit test --------- */

	if (start<=16) {

		/* Message */
		printf("32 bit tests ...\n");

		/* Load the image into a surface */
		bmpfile = "sample24.bmp";
		printf("Loading picture: %s\n", bmpfile);
		picture = SDL_LoadBMP(bmpfile);
		if ( picture == NULL ) {
			fprintf(stderr, "Couldn't load %s: %s\n", bmpfile, SDL_GetError());
			return;
		}

		/* Add white frame */
		rectangleColor(picture, 0, 0, picture->w-1, picture->h-1, 0xffffffff);

		/* New source surface is 32bit with defined RGBA ordering */
		/* Much faster to do this once rather than the routine on the fly */
		fprintf(stderr,"Converting 24bit image into 32bit RGBA surface ...\n");
		picture_again = SDL_CreateRGBSurface(SDL_SWSURFACE, picture->w, picture->h, 32, rmask, gmask, bmask, amask);
		if (picture_again == NULL) goto done32bit;		
		SDL_BlitSurface(picture,NULL,picture_again,NULL);

                if (start <= 13) {
			sprintf(messageText, "13. Rotating and zooming, with interpolation (RGBA source)");
			RotatePicture(screen,picture_again,ROTATE_ON,FLIP_OFF,SMOOTHING_ON,POSITION_CENTER);
		}
		if (end == 13) goto done32bit;

                if (start <= 14) {
			sprintf(messageText, "14. Just zooming (angle=0), with interpolation (RGBA source)");
			RotatePicture(screen,picture_again,ROTATE_OFF,FLIP_OFF,SMOOTHING_ON,POSITION_CENTER);
			RotatePicture(screen,picture_again,ROTATE_OFF,FLIP_OFF,SMOOTHING_ON,POSITION_BOTTOMRIGHT);
		}
		if (end == 14) goto done32bit;

		SDL_FreeSurface(picture_again);
		picture_again=NULL;

		/* New source surface is 32bit with defined ABGR ordering */
		/* Much faster to do this once rather than the routine on the fly */
		fprintf(stderr,"Converting 24bit image into 32bit ABGR surface ...\n");
		picture_again = SDL_CreateRGBSurface(SDL_SWSURFACE, picture->w, picture->h, 32, amask, bmask, gmask, rmask);
		if (picture_again == NULL) goto done32bit;		
		SDL_BlitSurface(picture,NULL,picture_again,NULL);

                if (start <= 14) {
			sprintf(messageText, "15. Rotating and zooming, with interpolation (ABGR source)");
			RotatePicture(screen,picture_again,ROTATE_ON,FLIP_OFF,SMOOTHING_ON,POSITION_CENTER);
		}
		if (end == 14) goto done32bit;

                if (start <= 14) {
			sprintf(messageText, "16. Just zooming (angle=0), with interpolation (ABGR source)");
			RotatePicture(screen,picture_again,ROTATE_OFF,FLIP_OFF,SMOOTHING_ON,POSITION_CENTER);
			RotatePicture(screen,picture_again,ROTATE_OFF,FLIP_OFF,SMOOTHING_ON,POSITION_BOTTOMRIGHT);
		}
		if (end == 14) goto done32bit;


		done32bit:
		
		/* Free the picture */
		SDL_FreeSurface(picture);
		if (picture_again) SDL_FreeSurface(picture_again);
		
		if (end <= 16) return;
	}

	/* -------- 32 bit flip test --------- */

	if (start<=22) {

		/* Message */
		printf("32 bit flip tests ...\n");

		/* Load the image into a surface */
		bmpfile = "sample24.bmp";
		printf("Loading picture: %s\n", bmpfile);
		picture = SDL_LoadBMP(bmpfile);
		if ( picture == NULL ) {
			fprintf(stderr, "Couldn't load %s: %s\n", bmpfile, SDL_GetError());
			return;
		}

		/* Add white frame */
		rectangleColor(picture, 0, 0, picture->w-1, picture->h-1, 0xffffffff);

		/* Excercise flipping functions on 32bit RGBA */
		printf("Converting 24bit image into 32bit RGBA surface ...\n");
		picture_again = SDL_CreateRGBSurface(SDL_SWSURFACE, picture->w, picture->h, 32, rmask, gmask, bmask, amask);
		if (picture_again == NULL) goto doneflip;
		SDL_BlitSurface(picture,NULL,picture_again,NULL);
			
                if (start <= 17) {
			sprintf(messageText, "17. Rotating with x-flip, no interpolation (RGBA source)");
			RotatePicture(screen,picture_again,ROTATE_ON,FLIP_X,SMOOTHING_OFF,POSITION_CENTER);
		}
		if (end == 17) goto doneflip;

                if (start <= 18) {
			sprintf(messageText, "18. Rotating with y-flip, no interpolation");
			RotatePicture(screen,picture_again,ROTATE_ON,FLIP_Y,SMOOTHING_OFF,POSITION_CENTER);
		}
		if (end == 18) goto doneflip;

                if (start <= 19) {
			sprintf(messageText, "19. Rotating with xy-flip, no interpolation");
			RotatePicture(screen,picture_again,ROTATE_ON,FLIP_XY,SMOOTHING_OFF,POSITION_CENTER);
		}
		if (end == 19) goto doneflip;

                if (start <= 20) {
			sprintf(messageText, "20. Rotating with x-flip, with interpolation");
			RotatePicture(screen,picture_again,ROTATE_ON,FLIP_X,SMOOTHING_ON,POSITION_CENTER);
		}
		if (end == 20) goto doneflip;

                if (start <= 21) {
			sprintf(messageText, "21. Rotating with y-flip, with interpolation");
			RotatePicture(screen,picture_again,ROTATE_ON,FLIP_Y,SMOOTHING_ON,POSITION_CENTER);
		}
		if (end == 21) goto doneflip;

                if (start <= 22) {
			sprintf(messageText, "22. Rotating with xy-flip, with interpolation");
			RotatePicture(screen,picture_again,ROTATE_ON,FLIP_XY,SMOOTHING_ON,POSITION_CENTER);
		}
		if (end == 22) goto doneflip;

		doneflip:
		
		/* Free the pictures */
		SDL_FreeSurface(picture);
		if (picture_again) SDL_FreeSurface(picture_again);
		
		if (end <= 22) return;
	}

	if (start<=24) {

		/* Message */
		printf("Loading 24bit image\n");

		/* Load the image into a surface */
		bmpfile = "sample24.bmp";
		printf("Loading picture: %s\n", bmpfile);
		picture = SDL_LoadBMP(bmpfile);
		if ( picture == NULL ) {
			fprintf(stderr, "Couldn't load %s: %s\n", bmpfile, SDL_GetError());
			return;
		}

		/* Add white frame */
		rectangleColor(picture, 0, 0, picture->w-1, picture->h-1, 0xffffffff);

		/* Excercise flipping functions on 32bit RGBA */
		fprintf(stderr,"Converting 24bit image into 32bit RGBA surface ...\n");
		picture_again = SDL_CreateRGBSurface(SDL_SWSURFACE, picture->w, picture->h, 32, rmask, gmask, bmask, amask);
		SDL_BlitSurface(picture,NULL,picture_again,NULL);

		sprintf(messageText, "23. CustomTest, values from commandline (32bit)");
		CustomTest(screen, picture_again, custom_angle, custom_fx, custom_fy, custom_smooth);

		SDL_FreeSurface(picture_again);

		/* Free the picture */
		SDL_FreeSurface(picture);

		/* Message */
		printf("Loading 8bit image\n");

		/* Load the image into a surface */
		bmpfile = "sample8.bmp";
		printf("Loading picture: %s\n", bmpfile);
		picture = SDL_LoadBMP(bmpfile);
		if ( picture == NULL ) {
			fprintf(stderr, "Couldn't load %s: %s\n", bmpfile, SDL_GetError());
			return;
		}

		sprintf(messageText, "24. CustomTest, values from commandline (8bit)");
		CustomTest(screen, picture, custom_angle, custom_fx, custom_fy, custom_smooth);

		/* Free the picture */
		SDL_FreeSurface(picture);
		
		if (end <= 24) return;
	}

	if (start<=25) {

		/* Message */
		printf("Loading 24bit image\n");

		/* Load the image into a surface */
		bmpfile = "sample24.bmp";
		printf("Loading picture: %s\n", bmpfile);
		picture = SDL_LoadBMP(bmpfile);
		if ( picture == NULL ) {
			fprintf(stderr, "Couldn't load %s: %s\n", bmpfile, SDL_GetError());
			return;
		}

		/* Add white frame */
		rectangleColor(picture, 0, 0, picture->w-1, picture->h-1, 0xffffffff);

		/* New source surface is 32bit with defined RGBA ordering */
		printf("Converting 24bit image into 32bit RGBA surface ...\n");
		picture_again = SDL_CreateRGBSurface(SDL_SWSURFACE, picture->w, picture->h, 32, rmask, gmask, bmask, amask);
		if (picture_again == NULL) goto donerotate90;
		SDL_BlitSurface(picture,NULL,picture_again,NULL);

		/* Excercise rotate90 function on 32bit RGBA */
		sprintf(messageText, "25.  rotate90: Rotate 90 degrees clockwise (32bit)");
		RotatePicture90Degrees(screen, picture_again);

		donerotate90:
		
		/* Free the pictures */
		SDL_FreeSurface(picture);
		if (picture_again) SDL_FreeSurface(picture_again);

		if (end <= 25) return;
	}

	if (start<=26) {
		/* Run accuracy test */
		sprintf(messageText, "26.  accuracy: zoom by factor of 2");
		AccuracyTest1(screen);

		if (end <= 26) return;
	}

	if (start <= 27) {
		/* Load the image into a surface */
		bmpfile = "sample2x2.bmp";
		printf("Loading picture: %s\n", bmpfile);
		picture = SDL_LoadBMP(bmpfile);
		if ( picture == NULL ) {
			fprintf(stderr, "Couldn't load %s: %s\n", bmpfile, SDL_GetError());
			return;
		}

		sprintf(messageText, "27a.  zoom accuracy: zoom 2x2 bitmap");		
		AccuracyTest2(screen, picture);

		/* Free the pictures */
		SDL_FreeSurface(picture);

		/* Load the image into a surface */
		bmpfile = "sample3x3.bmp";
		printf("Loading picture: %s\n", bmpfile);
		picture = SDL_LoadBMP(bmpfile);
		if ( picture == NULL ) {
			fprintf(stderr, "Couldn't load %s: %s\n", bmpfile, SDL_GetError());
			return;
		}		
		
		sprintf(messageText, "27b.  zoom accuracy: zoom 3x3 bitmap");		
		AccuracyTest2(screen, picture);

		/* Free the pictures */
		SDL_FreeSurface(picture);

		/* Load the image into a surface */
		bmpfile = "sample16x16.bmp";
		printf("Loading picture: %s\n", bmpfile);
		picture = SDL_LoadBMP(bmpfile);
		if ( picture == NULL ) {
			fprintf(stderr, "Couldn't load %s: %s\n", bmpfile, SDL_GetError());
			return;
		}		
		
		sprintf(messageText, "27c.  zoom accuracy: zoom 16x16 bitmap");		
		AccuracyTest2(screen, picture);

		/* Free the pictures */
		SDL_FreeSurface(picture);
	
		if (end <= 27) return;
	}
	
	return;
}

/*!
 \brief SDL_rotozoom test
*/
int main ( int argc, char *argv[] )
{
	SDL_Surface *screen;
	int w, h;
	int desired_bpp;
	Uint32 video_flags;
	int start, end;

	/* Title */
	fprintf(stderr,"SDL_rotozoom test\n");
	messageText = (char *)malloc(128);

	/* Set default options and check command-line */
	w = 640;
	h = 480;
	desired_bpp = 0;
	video_flags = 0;
	start = 1;
	end = 9999;
	delay = 0;
	while ( argc > 1 ) {
		if ( strcmp(argv[1], "-start") == 0 ) {
			if ( argv[2] && ((start = atoi(argv[2])) > 0) ) {
				argv += 2;
				argc -= 2;
			} else {
				fprintf(stderr,
					"The -start option requires a numeric argument\n");
				exit(1);
			}
		}
		else if ( strcmp(argv[1], "-end") == 0 ) {
			if ( argv[2] && ((end = atoi(argv[2])) > 0) ) {
				argv += 2;
				argc -= 2;
			} else {
				fprintf(stderr,
					"The -end option requires a numeric argument\n");
				exit(1);
			}

		} 
		else 
		if ( strcmp(argv[1], "-delay") == 0 ) {
				if ( argv[2] && ((delay = atoi(argv[2])) > 0) ) {
					argv += 2;
					argc -= 2;
				} else {
					fprintf(stderr,
						"The -delay option requires an argument\n");
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
										if ( strcmp(argv[1], "-custom") == 0 ) {
											if (( argv[2] ) && ( argv[3] ) && ( argv[4] ) && (argv[5] )) {
												custom_angle = atof(argv[2]);
												custom_fx = atof(argv[3]);
												custom_fy = atof(argv[4]);
												custom_smooth = atoi(argv[5]);
												argv += 5;
												argc -= 5;
											} else {
												fprintf(stderr,
													"The -custom option requires 4 arguments\n");
												exit(1);
											}
										} else
											if (( strcmp(argv[1], "-help") == 0 ) || (strcmp(argv[1], "--help") == 0)) {
												printf ("Usage:\n");
												printf (" -start #	  Set starting test number\n");
												printf ("             1=8bit, 7=24bit, 13=32bit, 19=32bit flip, 23=custom, 25=rotate90\n");
												printf ("             26=zoom accuracy\n");
												printf (" -delay #        Set delay between frames in ms (default: 0=off)\n");
												printf ("                 (if >0, enables verbose frame logging\n");
												printf (" -width #	  Screen width (Default: %i)\n",w);
												printf (" -height #	  Screen height (Default: %i)\n",h);
												printf (" -bpp #	  Screen bpp\n");
												printf (" -warp		  Enable hardware palette\n");
												printf (" -hw		  Enable hardware surface\n");
												printf (" -fullscreen	  Enable fullscreen mode\n");
												printf (" -custom # # #	# Custom: angle scalex scaley smooth\n");
												printf ("                  scalex/scaley<0, enables flip on axis\n");
												printf ("                  smooth=0/1\n");
												exit(0);
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
	SDL_WM_SetCaption("SDL_rotozoom test", "rotozoom");

	/* Do all the drawing work */
	Draw(screen, start, end);	
	free(messageText);

	return(0);
}
