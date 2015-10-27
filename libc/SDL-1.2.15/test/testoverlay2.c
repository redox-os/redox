/********************************************************************************
 *                                                                              *
 * Test of the overlay used for moved pictures, test more closed to real life.  *
 * Running trojan moose :) Coded by Mike Gorchak.                               *
 *                                                                              *
 ********************************************************************************/

#include <stdlib.h>
#include <stdio.h>
#include <string.h>

#include "SDL.h"

#define MOOSEPIC_W 64
#define MOOSEPIC_H 88

#define MOOSEFRAME_SIZE (MOOSEPIC_W * MOOSEPIC_H)
#define MOOSEFRAMES_COUNT 10

SDL_Color MooseColors[84]={
    { 49,  49,  49}, { 66,  24,   0}, { 66,  33,   0}, { 66,  66,  66},
    { 66, 115,  49}, { 74,  33,   0}, { 74,  41,  16}, { 82,  33,   8},
    { 82,  41,   8}, { 82,  49,  16}, { 82,  82,  82}, { 90,  41,   8},
    { 90,  41,  16}, { 90,  57,  24}, { 99,  49,  16}, { 99,  66,  24},
    { 99,  66,  33}, { 99,  74,  33}, {107,  57,  24}, {107,  82,  41},
    {115,  57,  33}, {115,  66,  33}, {115,  66,  41}, {115,  74,   0},
    {115,  90,  49}, {115, 115, 115}, {123,  82,   0}, {123,  99,  57},
    {132,  66,  41}, {132,  74,  41}, {132,  90,   8}, {132,  99,  33},
    {132,  99,  66}, {132, 107,  66}, {140,  74,  49}, {140,  99,  16},
    {140, 107,  74}, {140, 115,  74}, {148, 107,  24}, {148, 115,  82},
    {148, 123,  74}, {148, 123,  90}, {156, 115,  33}, {156, 115,  90},
    {156, 123,  82}, {156, 132,  82}, {156, 132,  99}, {156, 156, 156},
    {165, 123,  49}, {165, 123,  90}, {165, 132,  82}, {165, 132,  90},
    {165, 132,  99}, {165, 140,  90}, {173, 132,  57}, {173, 132,  99},
    {173, 140, 107}, {173, 140, 115}, {173, 148,  99}, {173, 173, 173},
    {181, 140,  74}, {181, 148, 115}, {181, 148, 123}, {181, 156, 107},
    {189, 148, 123}, {189, 156,  82}, {189, 156, 123}, {189, 156, 132},
    {189, 189, 189}, {198, 156, 123}, {198, 165, 132}, {206, 165,  99},
    {206, 165, 132}, {206, 173, 140}, {206, 206, 206}, {214, 173, 115},
    {214, 173, 140}, {222, 181, 148}, {222, 189, 132}, {222, 189, 156},
    {222, 222, 222}, {231, 198, 165}, {231, 231, 231}, {239, 206, 173}
};


/* Call this instead of exit(), so we can clean up SDL: atexit() is evil. */
static void quit(int rc)
{
	SDL_Quit();
	exit(rc);
}

/* All RGB2YUV conversion code and some other parts of code has been taken from testoverlay.c */

/* NOTE: These RGB conversion functions are not intended for speed,
         only as examples.
*/

void RGBtoYUV(Uint8 *rgb, int *yuv, int monochrome, int luminance)
{
    if (monochrome)
    {
#if 1 /* these are the two formulas that I found on the FourCC site... */
        yuv[0] = 0.299*rgb[0] + 0.587*rgb[1] + 0.114*rgb[2];
        yuv[1] = 128;
        yuv[2] = 128;
#else
        yuv[0] = (0.257 * rgb[0]) + (0.504 * rgb[1]) + (0.098 * rgb[2]) + 16;
        yuv[1] = 128;
        yuv[2] = 128;
#endif
    }
    else
    {
#if 1 /* these are the two formulas that I found on the FourCC site... */
        yuv[0] = 0.299*rgb[0] + 0.587*rgb[1] + 0.114*rgb[2];
        yuv[1] = (rgb[2]-yuv[0])*0.565 + 128;
        yuv[2] = (rgb[0]-yuv[0])*0.713 + 128;
#else
        yuv[0] = (0.257 * rgb[0]) + (0.504 * rgb[1]) + (0.098 * rgb[2]) + 16;
        yuv[1] = 128 - (0.148 * rgb[0]) - (0.291 * rgb[1]) + (0.439 * rgb[2]);
        yuv[2] = 128 + (0.439 * rgb[0]) - (0.368 * rgb[1]) - (0.071 * rgb[2]);
#endif
    }

    if (luminance!=100)
    {
        yuv[0]=yuv[0]*luminance/100;
        if (yuv[0]>255)
            yuv[0]=255;
    }
}

void ConvertRGBtoYV12(SDL_Surface *s, SDL_Overlay *o, int monochrome, int luminance)
{
	int x,y;
	int yuv[3];
	Uint8 *p,*op[3];

	SDL_LockSurface(s);
	SDL_LockYUVOverlay(o);

	/* Convert */
	for(y=0; y<s->h && y<o->h; y++)
	{
		p=((Uint8 *) s->pixels)+s->pitch*y;
		op[0]=o->pixels[0]+o->pitches[0]*y;
		op[1]=o->pixels[1]+o->pitches[1]*(y/2);
		op[2]=o->pixels[2]+o->pitches[2]*(y/2);
		for(x=0; x<s->w && x<o->w; x++)
		{
			RGBtoYUV(p, yuv, monochrome, luminance);
			*(op[0]++)=yuv[0];
			if(x%2==0 && y%2==0)
			{
				*(op[1]++)=yuv[2];
				*(op[2]++)=yuv[1];
			}
			p+=s->format->BytesPerPixel;
		}
	}

	SDL_UnlockYUVOverlay(o);
	SDL_UnlockSurface(s);
}

void ConvertRGBtoIYUV(SDL_Surface *s, SDL_Overlay *o, int monochrome, int luminance)
{
	int x,y;
	int yuv[3];
	Uint8 *p,*op[3];

	SDL_LockSurface(s);
	SDL_LockYUVOverlay(o);

	/* Convert */
	for(y=0; y<s->h && y<o->h; y++)
	{
		p=((Uint8 *) s->pixels)+s->pitch*y;
		op[0]=o->pixels[0]+o->pitches[0]*y;
		op[1]=o->pixels[1]+o->pitches[1]*(y/2);
		op[2]=o->pixels[2]+o->pitches[2]*(y/2);
		for(x=0; x<s->w && x<o->w; x++)
		{
			RGBtoYUV(p,yuv, monochrome, luminance);
			*(op[0]++)=yuv[0];
			if(x%2==0 && y%2==0)
			{
				*(op[1]++)=yuv[1];
				*(op[2]++)=yuv[2];
			}
			p+=s->format->BytesPerPixel;
		}
	}

	SDL_UnlockYUVOverlay(o);
	SDL_UnlockSurface(s);
}

void ConvertRGBtoUYVY(SDL_Surface *s, SDL_Overlay *o, int monochrome, int luminance)
{
	int x,y;
	int yuv[3];
	Uint8 *p,*op;

	SDL_LockSurface(s);
	SDL_LockYUVOverlay(o);

	for(y=0; y<s->h && y<o->h; y++)
	{
		p=((Uint8 *) s->pixels)+s->pitch*y;
		op=o->pixels[0]+o->pitches[0]*y;
		for(x=0; x<s->w && x<o->w; x++)
		{
			RGBtoYUV(p, yuv, monochrome, luminance);
			if(x%2==0)
			{
				*(op++)=yuv[1];
				*(op++)=yuv[0];
				*(op++)=yuv[2];
			}
			else
				*(op++)=yuv[0];

			p+=s->format->BytesPerPixel;
		}
	}

	SDL_UnlockYUVOverlay(o);
	SDL_UnlockSurface(s);
}

void ConvertRGBtoYVYU(SDL_Surface *s, SDL_Overlay *o, int monochrome, int luminance)
{
	int x,y;
	int yuv[3];
	Uint8 *p,*op;

	SDL_LockSurface(s);
	SDL_LockYUVOverlay(o);

	for(y=0; y<s->h && y<o->h; y++)
	{
		p=((Uint8 *) s->pixels)+s->pitch*y;
		op=o->pixels[0]+o->pitches[0]*y;
		for(x=0; x<s->w && x<o->w; x++)
		{
			RGBtoYUV(p,yuv, monochrome, luminance);
			if(x%2==0)
			{
				*(op++)=yuv[0];
				*(op++)=yuv[2];
				op[1]=yuv[1];
			}
			else
			{
				*op=yuv[0];
				op+=2;
			}

			p+=s->format->BytesPerPixel;
		}
	}

	SDL_UnlockYUVOverlay(o);
	SDL_UnlockSurface(s);
}

void ConvertRGBtoYUY2(SDL_Surface *s, SDL_Overlay *o, int monochrome, int luminance)
{
	int x,y;
	int yuv[3];
	Uint8 *p,*op;

	SDL_LockSurface(s);
	SDL_LockYUVOverlay(o);
        
	for(y=0; y<s->h && y<o->h; y++)
	{
		p=((Uint8 *) s->pixels)+s->pitch*y;
		op=o->pixels[0]+o->pitches[0]*y;
		for(x=0; x<s->w && x<o->w; x++)
		{
			RGBtoYUV(p,yuv, monochrome, luminance);
			if(x%2==0)
			{
				*(op++)=yuv[0];
				*(op++)=yuv[1];
				op[1]=yuv[2];
			}
			else
			{
				*op=yuv[0];
				op+=2;
			}

			p+=s->format->BytesPerPixel;
		}
	}

	SDL_UnlockYUVOverlay(o);
	SDL_UnlockSurface(s);
}

static void PrintUsage(char *argv0)
{
    fprintf(stderr, "Usage: %s [arg] [arg] [arg] ...\n", argv0);
    fprintf(stderr, "\n");
    fprintf(stderr, "Where 'arg' is any of the following options:\n");
    fprintf(stderr, "\n");
    fprintf(stderr, "	-fps <frames per second>\n");
    fprintf(stderr, "	-format <fmt> (one of the: YV12, IYUV, YUY2, UYVY, YVYU)\n");
    fprintf(stderr, "	-scale <scale factor> (initial scale of the overlay)\n");
    fprintf(stderr, "	-help (shows this help)\n");
    fprintf(stderr, "\n");
    fprintf(stderr, "Press ESC to exit, or SPACE to freeze the movie while application running.\n");
    fprintf(stderr, "\n");
}

int main(int argc, char **argv)
{
    Uint8* RawMooseData;
    SDL_RWops* handle;
    SDL_Surface* screen;
    SDL_Surface* MooseFrame[MOOSEFRAMES_COUNT];
    SDL_Overlay* overlay;
    SDL_Rect overlayrect;
    SDL_Event event;
    Uint32 lastftick;
    int paused=0;
    int resized=0;
    int i;
    int fps=12;
    int fpsdelay;
    int overlay_format=SDL_YUY2_OVERLAY;
    int scale=5;

    if (SDL_Init(SDL_INIT_VIDEO | SDL_INIT_NOPARACHUTE) < 0)
    {
        fprintf(stderr, "Couldn't initialize SDL: %s\n", SDL_GetError());
        return 3;
    }

    while ( argc > 1 )
    {
        if (strcmp(argv[1], "-fps")== 0)
        {
            if (argv[2])
            {
                fps = atoi(argv[2]);
                if (fps==0)
                {
                    fprintf(stderr, "The -fps option requires an argument [from 1 to 1000], default is 12.\n");
                    quit(10);
                }
                if ((fps<0) || (fps>1000))
                {
                    fprintf(stderr, "The -fps option must be in range from 1 to 1000, default is 12.\n");
                    quit(10);
                }
                argv += 2;
                argc -= 2;
            }
            else
            {
                fprintf(stderr, "The -fps option requires an argument [from 1 to 1000], default is 12.\n");
                quit(10);
            }
        } else
        if (strcmp(argv[1], "-format") == 0)
        {
            if (argv[2])
            {
                if (!strcmp(argv[2],"YV12"))
                    overlay_format = SDL_YV12_OVERLAY;
                else if(!strcmp(argv[2],"IYUV"))
                    overlay_format = SDL_IYUV_OVERLAY;
                else if(!strcmp(argv[2],"YUY2"))
                    overlay_format = SDL_YUY2_OVERLAY;
                else if(!strcmp(argv[2],"UYVY"))
                    overlay_format = SDL_UYVY_OVERLAY;
                else if(!strcmp(argv[2],"YVYU"))
                    overlay_format = SDL_YVYU_OVERLAY;
                else
                {
                    fprintf(stderr, "The -format option %s is not recognized, see help for info.\n", argv[2]);
                    quit(10);
                }
                argv += 2;
                argc -= 2;
            }
            else
            {
                fprintf(stderr, "The -format option requires an argument, default is YUY2.\n");
                quit(10);
            }
        } else
        if (strcmp(argv[1], "-scale") == 0)
        {
            if (argv[2])
            {
                scale = atoi(argv[2]);
                if (scale==0)
                {
                    fprintf(stderr, "The -scale option requires an argument [from 1 to 50], default is 5.\n");
                    quit(10);
                }
                if ((scale<0) || (scale>50))
                {
                    fprintf(stderr, "The -scale option must be in range from 1 to 50, default is 5.\n");
                    quit(10);
                }
                argv += 2;
                argc -= 2;
            }
            else
            {
                fprintf(stderr, "The -fps option requires an argument [from 1 to 1000], default is 12.\n");
                quit(10);
            }
        } else
        if ((strcmp(argv[1], "-help") == 0 ) || (strcmp(argv[1], "-h") == 0))
        {
            PrintUsage(argv[0]);
            quit(0);
        } else
        {
            fprintf(stderr, "Unrecognized option: %s.\n", argv[1]);
            quit(10);
        }
        break;
    }
   
    RawMooseData=(Uint8*)malloc(MOOSEFRAME_SIZE * MOOSEFRAMES_COUNT);
    if (RawMooseData==NULL)
    {
        fprintf(stderr, "Can't allocate memory for movie !\n");
        free(RawMooseData);
        quit(1);
    }

    /* load the trojan moose images */
    handle=SDL_RWFromFile("moose.dat", "rb");
    if (handle==NULL)
    {
        fprintf(stderr, "Can't find the file moose.dat !\n");
        free(RawMooseData);
        quit(2);
    }
   
    SDL_RWread(handle, RawMooseData, MOOSEFRAME_SIZE, MOOSEFRAMES_COUNT);

    SDL_RWclose(handle);

    /* Set video mode */
    if ( (screen=SDL_SetVideoMode(MOOSEPIC_W*scale, MOOSEPIC_H*scale, 0, SDL_RESIZABLE | SDL_SWSURFACE)) == NULL )
    {
        fprintf(stderr, "Couldn't set video mode: %s\n", SDL_GetError());
        free(RawMooseData);
        quit(4);
    }

    /* Set the window manager title bar */
    SDL_WM_SetCaption("SDL test overlay: running moose", "testoverlay2");

    for (i=0; i<MOOSEFRAMES_COUNT; i++)
    {
        MooseFrame[i]=SDL_CreateRGBSurfaceFrom(RawMooseData+i*MOOSEFRAME_SIZE, MOOSEPIC_W,
                                               MOOSEPIC_H, 8, MOOSEPIC_W, 0, 0, 0, 0);
        if (MooseFrame[i]==NULL)
        {
            fprintf(stderr, "Couldn't create SDL_Surfaces:%s\n", SDL_GetError());
            free(RawMooseData);
            quit(5);
        }
        SDL_SetColors(MooseFrame[i], MooseColors, 0, 84);

	{
		SDL_Surface *newsurf;
		SDL_PixelFormat format;

		format.palette=NULL;
		format.BitsPerPixel=32;
		format.BytesPerPixel=4;
#if SDL_BYTEORDER == SDL_LIL_ENDIAN
		format.Rshift=0;
		format.Gshift=8;
		format.Bshift=16;
#else
		format.Rshift=24;
		format.Gshift=16;
		format.Bshift=8;
#endif
		format.Ashift=0;
		format.Rmask=0xff<<format.Rshift;
		format.Gmask=0xff<<format.Gshift;
		format.Bmask=0xff<<format.Bshift;
		format.Amask=0;
		format.Rloss=0;
		format.Gloss=0;
		format.Bloss=0;
		format.Aloss=8;
		format.colorkey=0;
		format.alpha=0;

		newsurf=SDL_ConvertSurface(MooseFrame[i], &format, SDL_SWSURFACE);
		if(!newsurf)
		{
                    fprintf(stderr, "Couldn't convert picture to 32bits RGB: %s\n", SDL_GetError());
                    quit(6);
		}
		SDL_FreeSurface(MooseFrame[i]);
		MooseFrame[i]=newsurf;
	}
    }

    free(RawMooseData);

    overlay=SDL_CreateYUVOverlay(MOOSEPIC_W, MOOSEPIC_H, overlay_format, screen);
    if (!overlay)
    {
        fprintf(stderr, "Couldn't create overlay: %s\n", SDL_GetError());
        quit(7);
    }

    printf("Created %dx%dx%d %s %s overlay\n",overlay->w,overlay->h,overlay->planes,
           overlay->hw_overlay?"hardware":"software",
           overlay->format==SDL_YV12_OVERLAY?"YV12":
           overlay->format==SDL_IYUV_OVERLAY?"IYUV":
           overlay->format==SDL_YUY2_OVERLAY?"YUY2":
           overlay->format==SDL_UYVY_OVERLAY?"UYVY":
           overlay->format==SDL_YVYU_OVERLAY?"YVYU":
           "Unknown");

    for(i=0; i<overlay->planes; i++)
    {
        printf("  plane %d: pitch=%d\n", i, overlay->pitches[i]);
    }

    overlayrect.x=0;
    overlayrect.y=0;
    overlayrect.w=MOOSEPIC_W*scale;
    overlayrect.h=MOOSEPIC_H*scale;

    /* set the start frame */
    i=0;
    fpsdelay=1000/fps;

    /* Ignore key up events, they don't even get filtered */
    SDL_EventState(SDL_KEYUP, SDL_IGNORE);

    lastftick=SDL_GetTicks();

    /* Loop, waiting for QUIT or RESIZE */
    while (1)
    {
        if (SDL_PollEvent(&event))
        {
            switch (event.type)
            {
                case SDL_VIDEORESIZE:
                     screen=SDL_SetVideoMode(event.resize.w, event.resize.h, 0, SDL_RESIZABLE | SDL_SWSURFACE);
                     overlayrect.w=event.resize.w;
                     overlayrect.h=event.resize.h;
                     if (paused)
                     {
                         resized=1;
                     }
                     break;
                case SDL_MOUSEBUTTONDOWN:
                     overlayrect.x = event.button.x - overlayrect.w/2;
                     overlayrect.y = event.button.y - overlayrect.h/2;
                     break;
                case SDL_KEYDOWN:
                     if (event.key.keysym.sym == SDLK_SPACE)
                     {
                         paused=!paused;
                         break;
                     }
                     if (event.key.keysym.sym != SDLK_ESCAPE)
                     {
                         break;
                     }
                case SDL_QUIT:
                     SDL_FreeYUVOverlay(overlay);
                     for (i=0; i<MOOSEFRAMES_COUNT; i++)
                     {
                         SDL_FreeSurface(MooseFrame[i]);
                     }
                     quit(0);
            }
        }

        if ((!paused)||(resized))
        {
            if (((SDL_GetTicks()-lastftick)>fpsdelay)||(resized))
            {
                lastftick=SDL_GetTicks();

                switch (overlay_format)
                {
                    case SDL_YUY2_OVERLAY:
                         ConvertRGBtoYUY2(MooseFrame[i], overlay, 0, 100);
                         break;
                    case SDL_YV12_OVERLAY:
                         ConvertRGBtoYV12(MooseFrame[i], overlay, 0, 100);
                         break;
                    case SDL_UYVY_OVERLAY:
                         ConvertRGBtoUYVY(MooseFrame[i], overlay, 0, 100);
                         break;
                    case SDL_YVYU_OVERLAY:
                         ConvertRGBtoYVYU(MooseFrame[i], overlay, 0, 100);
                         break;
                    case SDL_IYUV_OVERLAY:
                         ConvertRGBtoIYUV(MooseFrame[i], overlay, 0, 100);
                         break;
                }

                SDL_DisplayYUVOverlay(overlay, &overlayrect);
                if (!resized)
                {
                    i++;
                    if (i==10)
                    {
                        i=0;
                    }
                }
                else
                {
                    resized=0;
                }
            }
        }
        /* kind of timeslice to OS */
        SDL_Delay(1);
    }

	SDL_Quit();
    return 0;
}

