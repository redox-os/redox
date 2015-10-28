/* 

LaplaceRelaxation.c : Laplacian relaxation demo

Demo Program that
- loads an image
- converts it to a brightness map
- binarize brightness map
- performs a Laplace relaxation on it
- displays the solution using a sinusoidally modulated lookup table

Very slow! :-) TODO: use imagefilter MMX routines

(C) A. Schiffler, Sep 2010, zlib License

*/

#include <stdlib.h>
#include <stdio.h>
#include <string.h>

#include "SDL.h"

#ifdef WIN32
#include <windows.h>
#include "SDL_gfxPrimitives.h"
#include "SDL_rotozoom.h"
#include "SDL_imageFilter.h"
#ifndef bcmp
#define bcmp(s1, s2, n) memcmp ((s1), (s2), (n))
#endif
#else
#include "SDL/SDL_gfxPrimitives.h"
#include "SDL/SDL_rotozoom.h"
#include "SDL/SDL_imageFilter.h"
#endif

int iterations = 400;
int contours = 20;
int threshold = 4;
int w = 640, h = 480;

void HandleEvent()
{
	SDL_Event event; 

	/* Check for events */
	SDL_PollEvent(&event);
	switch (event.type) {
		 case SDL_KEYDOWN:
		 case SDL_QUIT:
			 exit(0);
			 break;
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

/* 
Returns a potential value from either the static or the dynamic charge map based 
on the binarized map at coordinates x,y for a grid of size w,h clamping to 0 at the edges. 
*/
double GetPotential(Uint8 *map, Uint8 *greyscale, double *relaxed, int x, int y, int w, int h)
{
	int offset;
	double v;

	if ((x<0) || (y<0) || (x>(w-1)) || (y>(h-1))) {
		v = 0.0;
	} else {
		offset = x+w*y;
		if ((Uint8)map[offset]!=0) {
			v = (double)greyscale[offset];
		} else {
			v = relaxed[offset];
		}
	}

	return v;
}

/*
* Return the pixel value at (x, y)
*/
Uint32 getPixel(SDL_Surface *surface, int x, int y)
{
	int bpp = surface->format->BytesPerPixel;

	/* Here p is the address to the pixel we want to retrieve */
	Uint8 *p = (Uint8 *)surface->pixels + y * surface->pitch + x * bpp;

	switch (bpp) {
	case 1:
		return *p;

	case 2:
		return *(Uint16 *)p;

	case 3:
		if (SDL_BYTEORDER == SDL_BIG_ENDIAN)
			return p[0] << 16 | p[1] << 8 | p[2];
		else
			return p[0] | p[1] << 8 | p[2] << 16;

	case 4:
		return *(Uint32 *)p;

	default:
		return 0;       /* shouldn't happen, but avoids warnings */
	} // switch
}

void Draw (SDL_Surface *screen, char *bmpfile)
{
	char messageText[128];
	Uint32 rmask, gmask, bmask, amask;
	SDL_Surface *picture;
	SDL_Surface *mapped_picture;
	SDL_Surface *rotozoom_picture;
	SDL_PixelFormat *pixelFormat;
	Uint8 *grayscale, *map, *curmap;
	double *unrelaxed, *relaxed, *currelax;
	int mapsize, relaxsize;
	int rowskip;
	char *pixel;
	Uint32 p;
	int x, y;
	Uint8 r, g, b, a;
	double dy;
	double t, s;
	int i;
	double c1, c2, c3, c4, ca;
	Uint8 lookupTable[256];
	double zw, zh, zf;

	/* SDL interprets each pixel as a 32-bit number, so our masks must depend
	on the endianness (byte order) of the machine */
#if SDL_BYTEORDER == SDL_BIG_ENDIAN
	rmask = 0xff000000;
	gmask = 0x00ff0000;
	bmask = 0x0000ff00;
	amask = 0x000000ff;
#else
	rmask = 0x000000ff;
	gmask = 0x0000ff00;
	bmask = 0x00ff0000;
	amask = 0xff000000;
#endif

	/* Load the image into a surface */
	printf("Loading picture: %s\n", bmpfile);
	picture = SDL_LoadBMP(bmpfile);
	if ( picture == NULL ) {
		fprintf(stderr, "Couldn't load %s: %s\n", bmpfile, SDL_GetError());
		return;
	}

	/* Convert image to a brightness map */
	printf("Allocating workbuffers\n");
	mapsize = picture->w * picture->h;
	relaxsize = mapsize * sizeof(double);
	grayscale = (Uint8 *)malloc(mapsize);
	map = (Uint8 *)malloc(mapsize);
	unrelaxed = (double *)malloc(relaxsize);
	relaxed = (double *)malloc(relaxsize);
	if ((grayscale == NULL) || (map == NULL) || (unrelaxed == NULL) || (relaxed == NULL))
	{
		fprintf(stderr, "Out of memory\n");
		return;
	}
	memset(grayscale, 0, mapsize);
	memset(map, 0, mapsize);
	memset(unrelaxed, 0, relaxsize);
	memset(relaxed, 0, relaxsize);

	printf("Converting image to brightness map\n");
	pixel = picture->pixels;
	pixelFormat = picture->format;
	rowskip = (picture->pitch - picture->w * picture->format->BytesPerPixel);
	curmap = grayscale;
	for (y=0; y < picture->h; y++) {
		for (x=0; x < picture->w; x++) {
			// Get RGB
			p = getPixel(picture, x, y);
			SDL_GetRGBA(p, pixelFormat, &r, &g, &b, &a);

			// Calculate luminance (Y = 0.3R + 0.59G + 0.11B;)
			dy  = (0.30 * r) + (0.59 * g) + (0.11 * b);
			if (dy<0.0) {
				dy=0.0;
			} else if (dy>255.0) {
				dy=255.0;
			}
			*curmap = (Uint8)dy;

			// Next pixel
			pixel += picture->format->BytesPerPixel;
			curmap++;
		}

		// Next row
		pixel += rowskip;
	}

	/* --- Prepare relaxation loop --- */

	/* Binarize luminance map */
	SDL_imageFilterBinarizeUsingThreshold(grayscale, map, mapsize, threshold);

	/* Create cos^5 based lookup table */
	t = 0.0;
	for (i = 0; i < 256; i++)
	{
		s = 1.0 - 0.5 * (1.0 + cos(t));
		s = 255.0 * s * s * s * s * s;
		lookupTable[i] = (Uint8)s;
		t += ((double)contours*2.0*3.141592653589/128.0);
	}

	/* Create new surface for relaxed image */
	mapped_picture = SDL_CreateRGBSurface(SDL_SWSURFACE, picture->w, picture->h, 32,
		rmask, gmask, bmask, amask);
	if (mapped_picture == NULL) {
		fprintf(stderr, "CreateRGBSurface failed: %s\n", SDL_GetError());
		return;
	}

	/* Apply relaxation algorithm */
	printf("Applying Laplacian relaxation: %i iterations\n", iterations);
	// Iterate to relax
	for (i = 0; i <= iterations; i++)
	{
		// Backup original
		memcpy(unrelaxed, relaxed, relaxsize);
		// Average into relaxed
		for (x=0; x < picture->w; x++) {		
			for (y=0; y < picture->h; y++) {
				// up
				c1 = GetPotential(map, grayscale, unrelaxed, x, y-1, picture->w, picture->h);
				// down
				c2 = GetPotential(map, grayscale, unrelaxed, x, y+1, picture->w, picture->h);
				// left
				c3 = GetPotential(map, grayscale, unrelaxed, x-1, y, picture->w, picture->h);
				// right
				c4 = GetPotential(map, grayscale, unrelaxed, x+1, y, picture->w, picture->h);
				// average and store
				ca = ((c1 + c2 + c3 + c4)/4.0);
				relaxed[x+y*picture->w] = ca;
			}
		}

		// Draw only sometimes
		if (((i % 10)==0) || (i==iterations)) {

			/* --- Create image with contour map --- */

			/* Fill output surface via colormap */
			currelax = relaxed;
			for (y=0; y<mapped_picture->h; y++) {
				for (x=0; x<mapped_picture->w; x++) {
					if (map[x+y*picture->w]!=0) {
						r = g = b = grayscale[x+y*picture->w];
					} else {
						r = g = b = lookupTable[(Uint8)*currelax];
					}
					pixelRGBA(mapped_picture, x, y, r, g, b, 255);
					currelax++;
				}
			}

			/* --- Scale and draw to screen --- */

			/* Scale to screen size */
			zw = (double)screen->w/(double)mapped_picture->w; 
			zh = (double)screen->h/(double)mapped_picture->h; 
			zf = (zw < zh) ? zw : zh;
			if ((rotozoom_picture=zoomSurface(mapped_picture, zf, zf, 1))==NULL) {
				fprintf(stderr, "Rotozoom failed: %s\n", SDL_GetError());
				return;
			}	

			/* Draw surface to screen */
			if ( SDL_BlitSurface(rotozoom_picture, NULL, screen, NULL) < 0 ) {
				fprintf(stderr, "Blit failed: %s\n", SDL_GetError());
				return;
			}
			SDL_FreeSurface(rotozoom_picture);

			/* Info */
			if (i != iterations) {
				sprintf(messageText,"%i", i);
				stringRGBA(screen, 8, 8, messageText, 255, 255, 255, 255);
			}

			/* Display by flipping screens */
			SDL_Flip(screen);
		}

		/* Maybe quit */
		HandleEvent();
	}

	/* Save final picture */
	if (SDL_SaveBMP(mapped_picture, "result.bmp") <0) {
		fprintf(stderr, "Save BMP failed: %s\n", SDL_GetError());
		return;
	}
	free(map);
	free(grayscale);
	free(unrelaxed);
	free(relaxed);
	SDL_FreeSurface(picture);
	SDL_FreeSurface(mapped_picture);

	return;
}

int main ( int argc, char *argv[] )
{
	SDL_Surface *screen;
	int desired_bpp;
	Uint32 video_flags;
	char *bmpfile = NULL;

	/* Title */
	fprintf(stderr,"Laplace relaxation demo\n");

	/* Set default options and check command-line */
	desired_bpp = 32;
	video_flags = SDL_SWSURFACE | SDL_SRCALPHA | SDL_RESIZABLE | SDL_DOUBLEBUF;
	while ( argc > 1 ) {
		if ( strcmp(argv[1], "-iterations") == 0 ) {
			if ( argv[2] && ((iterations = atoi(argv[2])) > 0) ) {
				argv += 2;
				argc -= 2;
			} else {
				fprintf(stderr,
					"The -iterations option requires an argument\n");
				exit(1);
			}
		} else if ( strcmp(argv[1], "-contours") == 0 ) {
			if ( argv[2] && ((contours = atoi(argv[2])) > 0) ) {
				argv += 2;
				argc -= 2;
			} else {
				fprintf(stderr,
					"The -contours option requires an argument\n");
				exit(1);
			}
		} else if ( strcmp(argv[1], "-threshold") == 0 ) {
			if ( argv[2] && ((threshold = atoi(argv[2])) > 0) ) {
				argv += 2;
				argc -= 2;
			} else {
				fprintf(stderr,
					"The -threshold option requires an argument\n");
				exit(1);
			}
		} else if (( strcmp(argv[1], "-help") == 0 ) || (strcmp(argv[1], "--help") == 0)) {
			printf ("Usage:\n%s [options] filename\n");
			printf ("Options:\n");
			printf (" -iterations #	  Number of relaxation iterations to perform, default: 400\n");
			printf (" -contours #	  Number of contours over range, default: 8\n");
			printf (" -threshold #	  Binarization threshold for Y, default: 20\n");
			exit(0);
		} else {
			bmpfile = argv[1];
			break;
		}
	}

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
		(screen->flags & SDL_HWSURFACE) ? "video" : "system");

	/* Check for double buffering */
	if ( screen->flags & SDL_DOUBLEBUF ) {
		printf("Double-buffering enabled - good!\n");
	}

	/* Set the window manager title bar */
	SDL_WM_SetCaption("Laplace relaxation demo", "laplacerelaxation");

	/* Do all the drawing work */
	Draw(screen, bmpfile);	

	/* Wait for keypress */
	while(1) {
		HandleEvent();
		SDL_Delay(100);
	}

	return(0);
}
