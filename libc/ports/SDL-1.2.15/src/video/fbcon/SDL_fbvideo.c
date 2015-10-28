/*
    SDL - Simple DirectMedia Layer
    Copyright (C) 1997-2012 Sam Lantinga

    This library is free software; you can redistribute it and/or
    modify it under the terms of the GNU Lesser General Public
    License as published by the Free Software Foundation; either
    version 2.1 of the License, or (at your option) any later version.

    This library is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
    Lesser General Public License for more details.

    You should have received a copy of the GNU Lesser General Public
    License along with this library; if not, write to the Free Software
    Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA  02110-1301  USA

    Sam Lantinga
    slouken@libsdl.org
*/
#include "SDL_config.h"

/* Framebuffer console based SDL video driver implementation.
*/

#include <stdio.h>
#include <fcntl.h>
#include <unistd.h>
#include <sys/ioctl.h>
#include <sys/mman.h>

#ifndef HAVE_GETPAGESIZE
#include <asm/page.h>		/* For definition of PAGE_SIZE */
#endif

#include <linux/vt.h>

#include "SDL_video.h"
#include "SDL_mouse.h"
#include "../SDL_sysvideo.h"
#include "../SDL_pixels_c.h"
#include "../../events/SDL_events_c.h"
#include "SDL_fbvideo.h"
#include "SDL_fbmouse_c.h"
#include "SDL_fbevents_c.h"
#include "SDL_fb3dfx.h"
#include "SDL_fbmatrox.h"
#include "SDL_fbriva.h"

/*#define FBCON_DEBUG*/

#if defined(i386) && defined(FB_TYPE_VGA_PLANES)
#define VGA16_FBCON_SUPPORT
#include <sys/io.h>		/* For ioperm() */
#ifndef FB_AUX_VGA_PLANES_VGA4
#define FB_AUX_VGA_PLANES_VGA4	0
#endif
/*
static inline void outb (unsigned char value, unsigned short port)
{
  __asm__ __volatile__ ("outb %b0,%w1"::"a" (value), "Nd" (port));
} 
*/
#endif /* FB_TYPE_VGA_PLANES */

/* A list of video resolutions that we query for (sorted largest to smallest) */
static const SDL_Rect checkres[] = {
	{  0, 0, 1600, 1200 },		/* 16 bpp: 0x11E, or 286 */
	{  0, 0, 1408, 1056 },		/* 16 bpp: 0x19A, or 410 */
	{  0, 0, 1280, 1024 },		/* 16 bpp: 0x11A, or 282 */
	{  0, 0, 1152,  864 },		/* 16 bpp: 0x192, or 402 */
	{  0, 0, 1024,  768 },		/* 16 bpp: 0x117, or 279 */
	{  0, 0,  960,  720 },		/* 16 bpp: 0x18A, or 394 */
	{  0, 0,  800,  600 },		/* 16 bpp: 0x114, or 276 */
	{  0, 0,  768,  576 },		/* 16 bpp: 0x182, or 386 */
	{  0, 0,  720,  576 },		/* PAL */
	{  0, 0,  720,  480 },		/* NTSC */
	{  0, 0,  640,  480 },		/* 16 bpp: 0x111, or 273 */
	{  0, 0,  640,  400 },		/*  8 bpp: 0x100, or 256 */
	{  0, 0,  512,  384 },
	{  0, 0,  320,  240 },
	{  0, 0,  320,  200 }
};
static const struct {
	int xres;
	int yres;
	int pixclock;
	int left;
	int right;
	int upper;
	int lower;
	int hslen;
	int vslen;
	int sync;
	int vmode;
} vesa_timings[] = {
#ifdef USE_VESA_TIMINGS	/* Only tested on Matrox Millenium I */
	{  640,  400, 39771,  48, 16, 39,  8,  96, 2, 2, 0 },	/* 70 Hz */
	{  640,  480, 39683,  48, 16, 33, 10,  96, 2, 0, 0 },	/* 60 Hz */
	{  768,  576, 26101, 144, 16, 28,  6, 112, 4, 0, 0 },	/* 60 Hz */
	{  800,  600, 24038, 144, 24, 28,  8, 112, 6, 0, 0 },	/* 60 Hz */
	{  960,  720, 17686, 144, 24, 28,  8, 112, 4, 0, 0 },	/* 60 Hz */
	{ 1024,  768, 15386, 160, 32, 30,  4, 128, 4, 0, 0 },	/* 60 Hz */
	{ 1152,  864, 12286, 192, 32, 30,  4, 128, 4, 0, 0 },	/* 60 Hz */
	{ 1280, 1024,  9369, 224, 32, 32,  4, 136, 4, 0, 0 },	/* 60 Hz */
	{ 1408, 1056,  8214, 256, 40, 32,  5, 144, 5, 0, 0 },	/* 60 Hz */
	{ 1600, 1200,/*?*/0, 272, 48, 32,  5, 152, 5, 0, 0 },	/* 60 Hz */
#else
	/* You can generate these timings from your XF86Config file using
	   the 'modeline2fb' perl script included with the fbset package.
	   These timings were generated for Matrox Millenium I, 15" monitor.
	*/
	{  320,  200, 79440,  16, 16, 20,  4,  48, 1, 0, 2 },	/* 70 Hz */
	{  320,  240, 63492,  16, 16, 16,  4,  48, 2, 0, 2 },	/* 72 Hz */
	{  512,  384, 49603,  48, 16, 16,  1,  64, 3, 0, 0 },	/* 78 Hz */
	{  640,  400, 31746,  96, 32, 41,  1,  64, 3, 2, 0 },	/* 85 Hz */
	{  640,  480, 31746, 120, 16, 16,  1,  64, 3, 0, 0 },	/* 75 Hz */
	{  768,  576, 26101, 144, 16, 28,  6, 112, 4, 0, 0 },	/* 60 Hz */
	{  800,  600, 20000,  64, 56, 23, 37, 120, 6, 3, 0 },	/* 72 Hz */
	{  960,  720, 17686, 144, 24, 28,  8, 112, 4, 0, 0 },	/* 60 Hz */
	{ 1024,  768, 13333, 144, 24, 29,  3, 136, 6, 0, 0 },	/* 70 Hz */
	{ 1152,  864, 12286, 192, 32, 30,  4, 128, 4, 0, 0 },	/* 60 Hz */
	{ 1280, 1024,  9369, 224, 32, 32,  4, 136, 4, 0, 0 },	/* 60 Hz */
	{ 1408, 1056,  8214, 256, 40, 32,  5, 144, 5, 0, 0 },	/* 60 Hz */
	{ 1600, 1200,/*?*/0, 272, 48, 32,  5, 152, 5, 0, 0 },	/* 60 Hz */
#endif
};
enum {
	FBCON_ROTATE_NONE = 0,
	FBCON_ROTATE_CCW = 90,
	FBCON_ROTATE_UD = 180,
	FBCON_ROTATE_CW = 270
};

#define min(a,b) ((a)<(b)?(a):(b))

/* Initialization/Query functions */
static int FB_VideoInit(_THIS, SDL_PixelFormat *vformat);
static SDL_Rect **FB_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags);
static SDL_Surface *FB_SetVideoMode(_THIS, SDL_Surface *current, int width, int height, int bpp, Uint32 flags);
#ifdef VGA16_FBCON_SUPPORT
static SDL_Surface *FB_SetVGA16Mode(_THIS, SDL_Surface *current, int width, int height, int bpp, Uint32 flags);
#endif
static int FB_SetColors(_THIS, int firstcolor, int ncolors, SDL_Color *colors);
static void FB_VideoQuit(_THIS);

/* Hardware surface functions */
static int FB_InitHWSurfaces(_THIS, SDL_Surface *screen, char *base, int size);
static void FB_FreeHWSurfaces(_THIS);
static int FB_AllocHWSurface(_THIS, SDL_Surface *surface);
static int FB_LockHWSurface(_THIS, SDL_Surface *surface);
static void FB_UnlockHWSurface(_THIS, SDL_Surface *surface);
static void FB_FreeHWSurface(_THIS, SDL_Surface *surface);
static void FB_WaitVBL(_THIS);
static void FB_WaitIdle(_THIS);
static int FB_FlipHWSurface(_THIS, SDL_Surface *surface);

/* Internal palette functions */
static void FB_SavePalette(_THIS, struct fb_fix_screeninfo *finfo,
                                  struct fb_var_screeninfo *vinfo);
static void FB_RestorePalette(_THIS);

/* Shadow buffer functions */
static FB_bitBlit FB_blit16;
static FB_bitBlit FB_blit16blocked;

static int SDL_getpagesize(void)
{
#ifdef HAVE_GETPAGESIZE
	return getpagesize();
#elif defined(PAGE_SIZE)
	return PAGE_SIZE;
#else
#error Can not determine system page size.
	return 4096;  /* this is what it USED to be in Linux... */
#endif
}


/* Small wrapper for mmap() so we can play nicely with no-mmu hosts
 * (non-mmu hosts disallow the MAP_SHARED flag) */

static void *do_mmap(void *start, size_t length, int prot, int flags, int fd, off_t offset)
{
	void *ret;
	ret = mmap(start, length, prot, flags, fd, offset);
	if ( ret == (char *)-1 && (flags & MAP_SHARED) ) {
		ret = mmap(start, length, prot,
		           (flags & ~MAP_SHARED) | MAP_PRIVATE, fd, offset);
	}
	return ret;
}

/* FB driver bootstrap functions */

static int FB_Available(void)
{
	int console = -1;
	/* Added check for /fb/0 (devfs) */
	/* but - use environment variable first... if it fails, still check defaults */
	int idx = 0;
	const char *SDL_fbdevs[4] = { NULL, "/dev/fb0", "/dev/fb/0", NULL };

	SDL_fbdevs[0] = SDL_getenv("SDL_FBDEV");
	if( !SDL_fbdevs[0] )
		idx++;
	for( ; SDL_fbdevs[idx]; idx++ )
	{
		console = open(SDL_fbdevs[idx], O_RDWR, 0);
		if ( console >= 0 ) {
			close(console);
			break;
		}
	}
	return(console >= 0);
}

static void FB_DeleteDevice(SDL_VideoDevice *device)
{
	SDL_free(device->hidden);
	SDL_free(device);
}

static SDL_VideoDevice *FB_CreateDevice(int devindex)
{
	SDL_VideoDevice *this;

	/* Initialize all variables that we clean on shutdown */
	this = (SDL_VideoDevice *)SDL_malloc(sizeof(SDL_VideoDevice));
	if ( this ) {
		SDL_memset(this, 0, (sizeof *this));
		this->hidden = (struct SDL_PrivateVideoData *)
				SDL_malloc((sizeof *this->hidden));
	}
	if ( (this == NULL) || (this->hidden == NULL) ) {
		SDL_OutOfMemory();
		if ( this ) {
			SDL_free(this);
		}
		return(0);
	}
	SDL_memset(this->hidden, 0, (sizeof *this->hidden));
	wait_vbl = FB_WaitVBL;
	wait_idle = FB_WaitIdle;
	mouse_fd = -1;
	keyboard_fd = -1;

	/* Set the function pointers */
	this->VideoInit = FB_VideoInit;
	this->ListModes = FB_ListModes;
	this->SetVideoMode = FB_SetVideoMode;
	this->SetColors = FB_SetColors;
	this->UpdateRects = NULL;
	this->VideoQuit = FB_VideoQuit;
	this->AllocHWSurface = FB_AllocHWSurface;
	this->CheckHWBlit = NULL;
	this->FillHWRect = NULL;
	this->SetHWColorKey = NULL;
	this->SetHWAlpha = NULL;
	this->LockHWSurface = FB_LockHWSurface;
	this->UnlockHWSurface = FB_UnlockHWSurface;
	this->FlipHWSurface = FB_FlipHWSurface;
	this->FreeHWSurface = FB_FreeHWSurface;
	this->SetCaption = NULL;
	this->SetIcon = NULL;
	this->IconifyWindow = NULL;
	this->GrabInput = NULL;
	this->GetWMInfo = NULL;
	this->InitOSKeymap = FB_InitOSKeymap;
	this->PumpEvents = FB_PumpEvents;

	this->free = FB_DeleteDevice;

	return this;
}

VideoBootStrap FBCON_bootstrap = {
	"fbcon", "Linux Framebuffer Console",
	FB_Available, FB_CreateDevice
};

#define FB_MODES_DB	"/etc/fb.modes"

static int read_fbmodes_line(FILE*f, char* line, int length)
{
	int blank;
	char* c;
	int i;
	
	blank=0;
	/* find a relevant line */
	do
	{
		if (!fgets(line,length,f))
			return 0;
		c=line;
		while(((*c=='\t')||(*c==' '))&&(*c!=0))
			c++;
		
		if ((*c=='\n')||(*c=='#')||(*c==0))
			blank=1;
		else
			blank=0;
	}
	while(blank);
	/* remove whitespace at the begining of the string */
	i=0;
	do
	{
		line[i]=c[i];
		i++;
	}
	while(c[i]!=0);
	return 1;
}

static int read_fbmodes_mode(FILE *f, struct fb_var_screeninfo *vinfo)
{
	char line[1024];
	char option[256];

	/* Find a "geometry" */
	do {
		if (read_fbmodes_line(f, line, sizeof(line))==0)
			return 0;
		if (SDL_strncmp(line,"geometry",8)==0)
			break;
	}
	while(1);

	SDL_sscanf(line, "geometry %d %d %d %d %d", &vinfo->xres, &vinfo->yres, 
			&vinfo->xres_virtual, &vinfo->yres_virtual, &vinfo->bits_per_pixel);
	if (read_fbmodes_line(f, line, sizeof(line))==0)
		return 0;
			
	SDL_sscanf(line, "timings %d %d %d %d %d %d %d", &vinfo->pixclock, 
			&vinfo->left_margin, &vinfo->right_margin, &vinfo->upper_margin, 
			&vinfo->lower_margin, &vinfo->hsync_len, &vinfo->vsync_len);
		
	vinfo->sync=0;
	vinfo->vmode=FB_VMODE_NONINTERLACED;
				
	/* Parse misc options */
	do {
		if (read_fbmodes_line(f, line, sizeof(line))==0)
			return 0;

		if (SDL_strncmp(line,"hsync",5)==0) {
			SDL_sscanf(line,"hsync %s",option);
			if (SDL_strncmp(option,"high",4)==0)
				vinfo->sync |= FB_SYNC_HOR_HIGH_ACT;
		}
		else if (SDL_strncmp(line,"vsync",5)==0) {
			SDL_sscanf(line,"vsync %s",option);
			if (SDL_strncmp(option,"high",4)==0)
				vinfo->sync |= FB_SYNC_VERT_HIGH_ACT;
		}
		else if (SDL_strncmp(line,"csync",5)==0) {
			SDL_sscanf(line,"csync %s",option);
			if (SDL_strncmp(option,"high",4)==0)
				vinfo->sync |= FB_SYNC_COMP_HIGH_ACT;
		}
		else if (SDL_strncmp(line,"extsync",5)==0) {
			SDL_sscanf(line,"extsync %s",option);
			if (SDL_strncmp(option,"true",4)==0)
				vinfo->sync |= FB_SYNC_EXT;
		}
		else if (SDL_strncmp(line,"laced",5)==0) {
			SDL_sscanf(line,"laced %s",option);
			if (SDL_strncmp(option,"true",4)==0)
				vinfo->vmode |= FB_VMODE_INTERLACED;
		}
		else if (SDL_strncmp(line,"double",6)==0) {
			SDL_sscanf(line,"double %s",option);
			if (SDL_strncmp(option,"true",4)==0)
				vinfo->vmode |= FB_VMODE_DOUBLE;
		}
	}
	while(SDL_strncmp(line,"endmode",7)!=0);

	return 1;
}

static int FB_CheckMode(_THIS, struct fb_var_screeninfo *vinfo,
                        int index, unsigned int *w, unsigned int *h)
{
	int mode_okay;

	mode_okay = 0;
	vinfo->bits_per_pixel = (index+1)*8;
	vinfo->xres = *w;
	vinfo->xres_virtual = *w;
	vinfo->yres = *h;
	vinfo->yres_virtual = *h;
	vinfo->activate = FB_ACTIVATE_TEST;
	if ( ioctl(console_fd, FBIOPUT_VSCREENINFO, vinfo) == 0 ) {
#ifdef FBCON_DEBUG
		fprintf(stderr, "Checked mode %dx%d at %d bpp, got mode %dx%d at %d bpp\n", *w, *h, (index+1)*8, vinfo->xres, vinfo->yres, vinfo->bits_per_pixel);
#endif
		if ( (((vinfo->bits_per_pixel+7)/8)-1) == index ) {
			*w = vinfo->xres;
			*h = vinfo->yres;
			mode_okay = 1;
		}
	}
	return mode_okay;
}

static int FB_AddMode(_THIS, int index, unsigned int w, unsigned int h, int check_timings)
{
	SDL_Rect *mode;
	int i;
	int next_mode;

	/* Check to see if we already have this mode */
	if ( SDL_nummodes[index] > 0 ) {
		mode = SDL_modelist[index][SDL_nummodes[index]-1];
		if ( (mode->w == w) && (mode->h == h) ) {
#ifdef FBCON_DEBUG
			fprintf(stderr, "We already have mode %dx%d at %d bytes per pixel\n", w, h, index+1);
#endif
			return(0);
		}
	}

	/* Only allow a mode if we have a valid timing for it */
	if ( check_timings ) {
		int found_timing = 0;
		for ( i=0; i<(sizeof(vesa_timings)/sizeof(vesa_timings[0])); ++i ) {
			if ( (w == vesa_timings[i].xres) &&
			     (h == vesa_timings[i].yres) && vesa_timings[i].pixclock ) {
				found_timing = 1;
				break;
			}
		}
		if ( !found_timing ) {
#ifdef FBCON_DEBUG
			fprintf(stderr, "No valid timing line for mode %dx%d\n", w, h);
#endif
			return(0);
		}
	}

	/* Set up the new video mode rectangle */
	mode = (SDL_Rect *)SDL_malloc(sizeof *mode);
	if ( mode == NULL ) {
		SDL_OutOfMemory();
		return(-1);
	}
	mode->x = 0;
	mode->y = 0;
	mode->w = w;
	mode->h = h;
#ifdef FBCON_DEBUG
	fprintf(stderr, "Adding mode %dx%d at %d bytes per pixel\n", w, h, index+1);
#endif

	/* Allocate the new list of modes, and fill in the new mode */
	next_mode = SDL_nummodes[index];
	SDL_modelist[index] = (SDL_Rect **)
	       SDL_realloc(SDL_modelist[index], (1+next_mode+1)*sizeof(SDL_Rect *));
	if ( SDL_modelist[index] == NULL ) {
		SDL_OutOfMemory();
		SDL_nummodes[index] = 0;
		SDL_free(mode);
		return(-1);
	}
	SDL_modelist[index][next_mode] = mode;
	SDL_modelist[index][next_mode+1] = NULL;
	SDL_nummodes[index]++;

	return(0);
}

static int cmpmodes(const void *va, const void *vb)
{
    const SDL_Rect *a = *(const SDL_Rect**)va;
    const SDL_Rect *b = *(const SDL_Rect**)vb;
    if ( a->h == b->h )
        return b->w - a->w;
    else
        return b->h - a->h;
}

static void FB_SortModes(_THIS)
{
	int i;
	for ( i=0; i<NUM_MODELISTS; ++i ) {
		if ( SDL_nummodes[i] > 0 ) {
			SDL_qsort(SDL_modelist[i], SDL_nummodes[i], sizeof *SDL_modelist[i], cmpmodes);
		}
	}
}

static int FB_VideoInit(_THIS, SDL_PixelFormat *vformat)
{
	const int pagesize = SDL_getpagesize();
	struct fb_fix_screeninfo finfo;
	struct fb_var_screeninfo vinfo;
	int i, j;
	int current_index;
	unsigned int current_w;
	unsigned int current_h;
	const char *SDL_fbdev;
	const char *rotation;
	FILE *modesdb;

	/* Initialize the library */
	SDL_fbdev = SDL_getenv("SDL_FBDEV");
	if ( SDL_fbdev == NULL ) {
		SDL_fbdev = "/dev/fb0";
	}
	console_fd = open(SDL_fbdev, O_RDWR, 0);
	if ( console_fd < 0 ) {
		SDL_SetError("Unable to open %s", SDL_fbdev);
		return(-1);
	}

#if !SDL_THREADS_DISABLED
	/* Create the hardware surface lock mutex */
	hw_lock = SDL_CreateMutex();
	if ( hw_lock == NULL ) {
		SDL_SetError("Unable to create lock mutex");
		FB_VideoQuit(this);
		return(-1);
	}
#endif

	/* Get the type of video hardware */
	if ( ioctl(console_fd, FBIOGET_FSCREENINFO, &finfo) < 0 ) {
		SDL_SetError("Couldn't get console hardware info");
		FB_VideoQuit(this);
		return(-1);
	}
	switch (finfo.type) {
		case FB_TYPE_PACKED_PIXELS:
			/* Supported, no worries.. */
			break;
#ifdef VGA16_FBCON_SUPPORT
		case FB_TYPE_VGA_PLANES:
			/* VGA16 is supported, but that's it */
			if ( finfo.type_aux == FB_AUX_VGA_PLANES_VGA4 ) {
				if ( ioperm(0x3b4, 0x3df - 0x3b4 + 1, 1) < 0 ) {
					SDL_SetError("No I/O port permissions");
					FB_VideoQuit(this);
					return(-1);
				}
				this->SetVideoMode = FB_SetVGA16Mode;
				break;
			}
			/* Fall through to unsupported case */
#endif /* VGA16_FBCON_SUPPORT */
		default:
			SDL_SetError("Unsupported console hardware");
			FB_VideoQuit(this);
			return(-1);
	}
	switch (finfo.visual) {
		case FB_VISUAL_TRUECOLOR:
		case FB_VISUAL_PSEUDOCOLOR:
		case FB_VISUAL_STATIC_PSEUDOCOLOR:
		case FB_VISUAL_DIRECTCOLOR:
			break;
		default:
			SDL_SetError("Unsupported console hardware");
			FB_VideoQuit(this);
			return(-1);
	}

	/* Check if the user wants to disable hardware acceleration */
	{ const char *fb_accel;
		fb_accel = SDL_getenv("SDL_FBACCEL");
		if ( fb_accel ) {
			finfo.accel = SDL_atoi(fb_accel);
		}
	}

	/* Memory map the device, compensating for buggy PPC mmap() */
	mapped_offset = (((long)finfo.smem_start) -
	                (((long)finfo.smem_start)&~(pagesize-1)));
	mapped_memlen = finfo.smem_len+mapped_offset;
	mapped_mem = do_mmap(NULL, mapped_memlen,
	                  PROT_READ|PROT_WRITE, MAP_SHARED, console_fd, 0);
	if ( mapped_mem == (char *)-1 ) {
		SDL_SetError("Unable to memory map the video hardware");
		mapped_mem = NULL;
		FB_VideoQuit(this);
		return(-1);
	}

	/* Determine the current screen depth */
	if ( ioctl(console_fd, FBIOGET_VSCREENINFO, &vinfo) < 0 ) {
		SDL_SetError("Couldn't get console pixel format");
		FB_VideoQuit(this);
		return(-1);
	}
	vformat->BitsPerPixel = vinfo.bits_per_pixel;
	if ( vformat->BitsPerPixel < 8 ) {
		/* Assuming VGA16, we handle this via a shadow framebuffer */
		vformat->BitsPerPixel = 8;
	}
	for ( i=0; i<vinfo.red.length; ++i ) {
		vformat->Rmask <<= 1;
		vformat->Rmask |= (0x00000001<<vinfo.red.offset);
	}
	for ( i=0; i<vinfo.green.length; ++i ) {
		vformat->Gmask <<= 1;
		vformat->Gmask |= (0x00000001<<vinfo.green.offset);
	}
	for ( i=0; i<vinfo.blue.length; ++i ) {
		vformat->Bmask <<= 1;
		vformat->Bmask |= (0x00000001<<vinfo.blue.offset);
	}
	saved_vinfo = vinfo;

	/* Save hardware palette, if needed */
	FB_SavePalette(this, &finfo, &vinfo);

	/* If the I/O registers are available, memory map them so we
	   can take advantage of any supported hardware acceleration.
	 */
	vinfo.accel_flags = 0;	/* Temporarily reserve registers */
	ioctl(console_fd, FBIOPUT_VSCREENINFO, &vinfo);
	if ( finfo.accel && finfo.mmio_len ) {
		mapped_iolen = finfo.mmio_len;
		mapped_io = do_mmap(NULL, mapped_iolen, PROT_READ|PROT_WRITE,
		                 MAP_SHARED, console_fd, mapped_memlen);
		if ( mapped_io == (char *)-1 ) {
			/* Hmm, failed to memory map I/O registers */
			mapped_io = NULL;
		}
	}

	rotate = FBCON_ROTATE_NONE;
	rotation = SDL_getenv("SDL_VIDEO_FBCON_ROTATION");
	if (rotation != NULL) {
		if (SDL_strlen(rotation) == 0) {
			shadow_fb = 0;
			rotate = FBCON_ROTATE_NONE;
#ifdef FBCON_DEBUG
			printf("Not rotating, no shadow\n");
#endif
		} else if (!SDL_strcmp(rotation, "NONE")) {
			shadow_fb = 1;
			rotate = FBCON_ROTATE_NONE;
#ifdef FBCON_DEBUG
			printf("Not rotating, but still using shadow\n");
#endif
		} else if (!SDL_strcmp(rotation, "CW")) {
			shadow_fb = 1;
			rotate = FBCON_ROTATE_CW;
#ifdef FBCON_DEBUG
			printf("Rotating screen clockwise\n");
#endif
		} else if (!SDL_strcmp(rotation, "CCW")) {
			shadow_fb = 1;
			rotate = FBCON_ROTATE_CCW;
#ifdef FBCON_DEBUG
			printf("Rotating screen counter clockwise\n");
#endif
		} else if (!SDL_strcmp(rotation, "UD")) {
			shadow_fb = 1;
			rotate = FBCON_ROTATE_UD;
#ifdef FBCON_DEBUG
			printf("Rotating screen upside down\n");
#endif
		} else {
			SDL_SetError("\"%s\" is not a valid value for "
				 "SDL_VIDEO_FBCON_ROTATION", rotation);
			return(-1);
		}
	}

	if (rotate == FBCON_ROTATE_CW || rotate == FBCON_ROTATE_CCW) {
		current_w = vinfo.yres;
		current_h = vinfo.xres;
	} else {
		current_w = vinfo.xres;
		current_h = vinfo.yres;
	}

	/* Query for the list of available video modes */
	current_index = ((vinfo.bits_per_pixel+7)/8)-1;
	modesdb = fopen(FB_MODES_DB, "r");
	for ( i=0; i<NUM_MODELISTS; ++i ) {
		SDL_nummodes[i] = 0;
		SDL_modelist[i] = NULL;
	}
	if ( SDL_getenv("SDL_FB_BROKEN_MODES") != NULL ) {
		FB_AddMode(this, current_index, current_w, current_h, 0);
	} else if(modesdb) {
		while ( read_fbmodes_mode(modesdb, &vinfo) ) {
			for ( i=0; i<NUM_MODELISTS; ++i ) {
				unsigned int w, h;

				if (rotate == FBCON_ROTATE_CW || rotate == FBCON_ROTATE_CCW) {
					w = vinfo.yres;
					h = vinfo.xres;
				} else {
					w = vinfo.xres;
					h = vinfo.yres;
				}
				/* See if we are querying for the current mode */
				if ( i == current_index ) {
					if ( (current_w > w) || (current_h > h) ) {
						/* Only check once */
						FB_AddMode(this, i, current_w, current_h, 0);
						current_index = -1;
					}
				}
				if ( FB_CheckMode(this, &vinfo, i, &w, &h) ) {
					FB_AddMode(this, i, w, h, 0);
				}
			}
		}
		fclose(modesdb);
		FB_SortModes(this);
	} else {
		for ( i=0; i<NUM_MODELISTS; ++i ) {
			for ( j=0; j<(sizeof(checkres)/sizeof(checkres[0])); ++j ) {
				unsigned int w, h;

				if (rotate == FBCON_ROTATE_CW || rotate == FBCON_ROTATE_CCW) {
					w = checkres[j].h;
					h = checkres[j].w;
				} else {
					w = checkres[j].w;
					h = checkres[j].h;
				}
				/* See if we are querying for the current mode */
				if ( i == current_index ) {
					if ( (current_w > w) || (current_h > h) ) {
						/* Only check once */
						FB_AddMode(this, i, current_w, current_h, 0);
						current_index = -1;
					}
				}
				if ( FB_CheckMode(this, &vinfo, i, &w, &h) ) {
					FB_AddMode(this, i, w, h, 1);
				}
			}
		}
	}

	this->info.current_w = current_w;
	this->info.current_h = current_h;
	this->info.wm_available = 0;
	this->info.hw_available = !shadow_fb;
	this->info.video_mem = shadow_fb ? 0 : finfo.smem_len/1024;
	/* Fill in our hardware acceleration capabilities */
	if ( mapped_io ) {
		switch (finfo.accel) {
		    case FB_ACCEL_MATROX_MGA2064W:
		    case FB_ACCEL_MATROX_MGA1064SG:
		    case FB_ACCEL_MATROX_MGA2164W:
		    case FB_ACCEL_MATROX_MGA2164W_AGP:
		    case FB_ACCEL_MATROX_MGAG100:
		    /*case FB_ACCEL_MATROX_MGAG200: G200 acceleration broken! */
		    case FB_ACCEL_MATROX_MGAG400:
#ifdef FBACCEL_DEBUG
			printf("Matrox hardware accelerator!\n");
#endif
			FB_MatroxAccel(this, finfo.accel);
			break;
		    case FB_ACCEL_3DFX_BANSHEE:
#ifdef FBACCEL_DEBUG
			printf("3DFX hardware accelerator!\n");
#endif
			FB_3DfxAccel(this, finfo.accel);
			break;
		    case FB_ACCEL_NV3:
		    case FB_ACCEL_NV4:
#ifdef FBACCEL_DEBUG
			printf("NVidia hardware accelerator!\n");
#endif
			FB_RivaAccel(this, finfo.accel);
			break;
		    default:
#ifdef FBACCEL_DEBUG
			printf("Unknown hardware accelerator.\n");
#endif
			break;
		}
	}

	if (shadow_fb) {
		shadow_mem = (char *)SDL_malloc(mapped_memlen);
		if (shadow_mem == NULL) {
			SDL_SetError("No memory for shadow");
			return (-1);
		} 
	}

	/* Enable mouse and keyboard support */
	if ( FB_OpenKeyboard(this) < 0 ) {
		FB_VideoQuit(this);
		return(-1);
	}
	if ( FB_OpenMouse(this) < 0 ) {
		const char *sdl_nomouse;

		sdl_nomouse = SDL_getenv("SDL_NOMOUSE");
		if ( ! sdl_nomouse ) {
			SDL_SetError("Unable to open mouse");
			FB_VideoQuit(this);
			return(-1);
		}
	}

	/* We're done! */
	return(0);
}

static SDL_Rect **FB_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags)
{
	return(SDL_modelist[((format->BitsPerPixel+7)/8)-1]);
}

/* Various screen update functions available */
static void FB_DirectUpdate(_THIS, int numrects, SDL_Rect *rects);
#ifdef VGA16_FBCON_SUPPORT
static void FB_VGA16Update(_THIS, int numrects, SDL_Rect *rects);
#endif

#ifdef FBCON_DEBUG
static void print_vinfo(struct fb_var_screeninfo *vinfo)
{
	fprintf(stderr, "Printing vinfo:\n");
	fprintf(stderr, "\txres: %d\n", vinfo->xres);
	fprintf(stderr, "\tyres: %d\n", vinfo->yres);
	fprintf(stderr, "\txres_virtual: %d\n", vinfo->xres_virtual);
	fprintf(stderr, "\tyres_virtual: %d\n", vinfo->yres_virtual);
	fprintf(stderr, "\txoffset: %d\n", vinfo->xoffset);
	fprintf(stderr, "\tyoffset: %d\n", vinfo->yoffset);
	fprintf(stderr, "\tbits_per_pixel: %d\n", vinfo->bits_per_pixel);
	fprintf(stderr, "\tgrayscale: %d\n", vinfo->grayscale);
	fprintf(stderr, "\tnonstd: %d\n", vinfo->nonstd);
	fprintf(stderr, "\tactivate: %d\n", vinfo->activate);
	fprintf(stderr, "\theight: %d\n", vinfo->height);
	fprintf(stderr, "\twidth: %d\n", vinfo->width);
	fprintf(stderr, "\taccel_flags: %d\n", vinfo->accel_flags);
	fprintf(stderr, "\tpixclock: %d\n", vinfo->pixclock);
	fprintf(stderr, "\tleft_margin: %d\n", vinfo->left_margin);
	fprintf(stderr, "\tright_margin: %d\n", vinfo->right_margin);
	fprintf(stderr, "\tupper_margin: %d\n", vinfo->upper_margin);
	fprintf(stderr, "\tlower_margin: %d\n", vinfo->lower_margin);
	fprintf(stderr, "\thsync_len: %d\n", vinfo->hsync_len);
	fprintf(stderr, "\tvsync_len: %d\n", vinfo->vsync_len);
	fprintf(stderr, "\tsync: %d\n", vinfo->sync);
	fprintf(stderr, "\tvmode: %d\n", vinfo->vmode);
	fprintf(stderr, "\tred: %d/%d\n", vinfo->red.length, vinfo->red.offset);
	fprintf(stderr, "\tgreen: %d/%d\n", vinfo->green.length, vinfo->green.offset);
	fprintf(stderr, "\tblue: %d/%d\n", vinfo->blue.length, vinfo->blue.offset);
	fprintf(stderr, "\talpha: %d/%d\n", vinfo->transp.length, vinfo->transp.offset);
}
static void print_finfo(struct fb_fix_screeninfo *finfo)
{
	fprintf(stderr, "Printing finfo:\n");
	fprintf(stderr, "\tsmem_start = %p\n", (char *)finfo->smem_start);
	fprintf(stderr, "\tsmem_len = %d\n", finfo->smem_len);
	fprintf(stderr, "\ttype = %d\n", finfo->type);
	fprintf(stderr, "\ttype_aux = %d\n", finfo->type_aux);
	fprintf(stderr, "\tvisual = %d\n", finfo->visual);
	fprintf(stderr, "\txpanstep = %d\n", finfo->xpanstep);
	fprintf(stderr, "\typanstep = %d\n", finfo->ypanstep);
	fprintf(stderr, "\tywrapstep = %d\n", finfo->ywrapstep);
	fprintf(stderr, "\tline_length = %d\n", finfo->line_length);
	fprintf(stderr, "\tmmio_start = %p\n", (char *)finfo->mmio_start);
	fprintf(stderr, "\tmmio_len = %d\n", finfo->mmio_len);
	fprintf(stderr, "\taccel = %d\n", finfo->accel);
}
#endif

static int choose_fbmodes_mode(struct fb_var_screeninfo *vinfo)
{
	int matched;
	FILE *modesdb;
	struct fb_var_screeninfo cinfo;

	matched = 0;
	modesdb = fopen(FB_MODES_DB, "r");
	if ( modesdb ) {
		/* Parse the mode definition file */
		while ( read_fbmodes_mode(modesdb, &cinfo) ) {
			if ( (vinfo->xres == cinfo.xres && vinfo->yres == cinfo.yres) &&
			     (!matched || (vinfo->bits_per_pixel == cinfo.bits_per_pixel)) ) {
				vinfo->pixclock = cinfo.pixclock;
				vinfo->left_margin = cinfo.left_margin;
				vinfo->right_margin = cinfo.right_margin;
				vinfo->upper_margin = cinfo.upper_margin;
				vinfo->lower_margin = cinfo.lower_margin;
				vinfo->hsync_len = cinfo.hsync_len;
				vinfo->vsync_len = cinfo.vsync_len;
				if ( matched ) {
					break;
				}
				matched = 1;
			}
		}
		fclose(modesdb);
	}
	return(matched);
}

static int choose_vesa_mode(struct fb_var_screeninfo *vinfo)
{
	int matched;
	int i;

	/* Check for VESA timings */
	matched = 0;
	for ( i=0; i<(sizeof(vesa_timings)/sizeof(vesa_timings[0])); ++i ) {
		if ( (vinfo->xres == vesa_timings[i].xres) &&
		     (vinfo->yres == vesa_timings[i].yres) ) {
#ifdef FBCON_DEBUG
			fprintf(stderr, "Using VESA timings for %dx%d\n",
						vinfo->xres, vinfo->yres);
#endif
			if ( vesa_timings[i].pixclock ) {
				vinfo->pixclock = vesa_timings[i].pixclock;
			}
			vinfo->left_margin = vesa_timings[i].left;
			vinfo->right_margin = vesa_timings[i].right;
			vinfo->upper_margin = vesa_timings[i].upper;
			vinfo->lower_margin = vesa_timings[i].lower;
			vinfo->hsync_len = vesa_timings[i].hslen;
			vinfo->vsync_len = vesa_timings[i].vslen;
			vinfo->sync = vesa_timings[i].sync;
			vinfo->vmode = vesa_timings[i].vmode;
			matched = 1;
			break;
		}
	}
	return(matched);
}

#ifdef VGA16_FBCON_SUPPORT
static SDL_Surface *FB_SetVGA16Mode(_THIS, SDL_Surface *current,
				int width, int height, int bpp, Uint32 flags)
{
	struct fb_fix_screeninfo finfo;
	struct fb_var_screeninfo vinfo;

	/* Set the terminal into graphics mode */
	if ( FB_EnterGraphicsMode(this) < 0 ) {
		return(NULL);
	}

	/* Restore the original palette */
	FB_RestorePalette(this);

	/* Set the video mode and get the final screen format */
	if ( ioctl(console_fd, FBIOGET_VSCREENINFO, &vinfo) < 0 ) {
		SDL_SetError("Couldn't get console screen info");
		return(NULL);
	}
	cache_vinfo = vinfo;
#ifdef FBCON_DEBUG
	fprintf(stderr, "Printing actual vinfo:\n");
	print_vinfo(&vinfo);
#endif
	if ( ! SDL_ReallocFormat(current, bpp, 0, 0, 0, 0) ) {
		return(NULL);
	}
	current->format->palette->ncolors = 16;

	/* Get the fixed information about the console hardware.
	   This is necessary since finfo.line_length changes.
	 */
	if ( ioctl(console_fd, FBIOGET_FSCREENINFO, &finfo) < 0 ) {
		SDL_SetError("Couldn't get console hardware info");
		return(NULL);
	}
#ifdef FBCON_DEBUG
	fprintf(stderr, "Printing actual finfo:\n");
	print_finfo(&finfo);
#endif

	/* Save hardware palette, if needed */
	FB_SavePalette(this, &finfo, &vinfo);

	/* Set up the new mode framebuffer */
	current->flags = SDL_FULLSCREEN;
	current->w = vinfo.xres;
	current->h = vinfo.yres;
	current->pitch = current->w;
	current->pixels = SDL_malloc(current->h*current->pitch);

	/* Set the update rectangle function */
	this->UpdateRects = FB_VGA16Update;

	/* We're done */
	return(current);
}
#endif /* VGA16_FBCON_SUPPORT */

static SDL_Surface *FB_SetVideoMode(_THIS, SDL_Surface *current,
				int width, int height, int bpp, Uint32 flags)
{
	struct fb_fix_screeninfo finfo;
	struct fb_var_screeninfo vinfo;
	int i;
	Uint32 Rmask;
	Uint32 Gmask;
	Uint32 Bmask;
	char *surfaces_mem;
	int surfaces_len;

	/* Set the terminal into graphics mode */
	if ( FB_EnterGraphicsMode(this) < 0 ) {
		return(NULL);
	}

	/* Restore the original palette */
	FB_RestorePalette(this);

	/* Set the video mode and get the final screen format */
	if ( ioctl(console_fd, FBIOGET_VSCREENINFO, &vinfo) < 0 ) {
		SDL_SetError("Couldn't get console screen info");
		return(NULL);
	}
#ifdef FBCON_DEBUG
	fprintf(stderr, "Printing original vinfo:\n");
	print_vinfo(&vinfo);
#endif
	/* Do not use double buffering with shadow buffer */
	if (shadow_fb) {
		flags &= ~SDL_DOUBLEBUF;
	}

	if ( (vinfo.xres != width) || (vinfo.yres != height) ||
	     (vinfo.bits_per_pixel != bpp) || (flags & SDL_DOUBLEBUF) ) {
		vinfo.activate = FB_ACTIVATE_NOW;
		vinfo.accel_flags = 0;
		vinfo.bits_per_pixel = bpp;
		vinfo.xres = width;
		vinfo.xres_virtual = width;
		vinfo.yres = height;
		if ( flags & SDL_DOUBLEBUF ) {
			vinfo.yres_virtual = height*2;
		} else {
			vinfo.yres_virtual = height;
		}
		vinfo.xoffset = 0;
		vinfo.yoffset = 0;
		vinfo.red.length = vinfo.red.offset = 0;
		vinfo.green.length = vinfo.green.offset = 0;
		vinfo.blue.length = vinfo.blue.offset = 0;
		vinfo.transp.length = vinfo.transp.offset = 0;
		if ( ! choose_fbmodes_mode(&vinfo) ) {
			choose_vesa_mode(&vinfo);
		}
#ifdef FBCON_DEBUG
		fprintf(stderr, "Printing wanted vinfo:\n");
		print_vinfo(&vinfo);
#endif
		if ( !shadow_fb &&
				ioctl(console_fd, FBIOPUT_VSCREENINFO, &vinfo) < 0 ) {
			vinfo.yres_virtual = height;
			if ( ioctl(console_fd, FBIOPUT_VSCREENINFO, &vinfo) < 0 ) {
				SDL_SetError("Couldn't set console screen info");
				return(NULL);
			}
		}
	} else {
		int maxheight;

		/* Figure out how much video memory is available */
		if ( flags & SDL_DOUBLEBUF ) {
			maxheight = height*2;
		} else {
			maxheight = height;
		}
		if ( vinfo.yres_virtual > maxheight ) {
			vinfo.yres_virtual = maxheight;
		}
	}
	cache_vinfo = vinfo;
#ifdef FBCON_DEBUG
	fprintf(stderr, "Printing actual vinfo:\n");
	print_vinfo(&vinfo);
#endif
	Rmask = 0;
	for ( i=0; i<vinfo.red.length; ++i ) {
		Rmask <<= 1;
		Rmask |= (0x00000001<<vinfo.red.offset);
	}
	Gmask = 0;
	for ( i=0; i<vinfo.green.length; ++i ) {
		Gmask <<= 1;
		Gmask |= (0x00000001<<vinfo.green.offset);
	}
	Bmask = 0;
	for ( i=0; i<vinfo.blue.length; ++i ) {
		Bmask <<= 1;
		Bmask |= (0x00000001<<vinfo.blue.offset);
	}
	if ( ! SDL_ReallocFormat(current, vinfo.bits_per_pixel,
	                                  Rmask, Gmask, Bmask, 0) ) {
		return(NULL);
	}

	/* Get the fixed information about the console hardware.
	   This is necessary since finfo.line_length changes.
	 */
	if ( ioctl(console_fd, FBIOGET_FSCREENINFO, &finfo) < 0 ) {
		SDL_SetError("Couldn't get console hardware info");
		return(NULL);
	}

	/* Save hardware palette, if needed */
	FB_SavePalette(this, &finfo, &vinfo);

	if (shadow_fb) {
		if (vinfo.bits_per_pixel == 16) {
			blitFunc = (rotate == FBCON_ROTATE_NONE ||
					rotate == FBCON_ROTATE_UD) ?
				FB_blit16 : FB_blit16blocked;
		} else {
#ifdef FBCON_DEBUG
			fprintf(stderr, "Init vinfo:\n");
			print_vinfo(&vinfo);
#endif
			SDL_SetError("Using software buffer, but no blitter "
					"function is available for %d bpp.",
					vinfo.bits_per_pixel);
			return(NULL);
		}
	}

	/* Set up the new mode framebuffer */
	current->flags &= SDL_FULLSCREEN;
	if (shadow_fb) {
		current->flags |= SDL_SWSURFACE;
	} else {
		current->flags |= SDL_HWSURFACE;
	}
	current->w = vinfo.xres;
	current->h = vinfo.yres;
	if (shadow_fb) {
		current->pitch = current->w * ((vinfo.bits_per_pixel + 7) / 8);
		current->pixels = shadow_mem;
		physlinebytes = finfo.line_length;
	} else {
		current->pitch = finfo.line_length;
		current->pixels = mapped_mem+mapped_offset;
	}

	/* Set up the information for hardware surfaces */
	surfaces_mem = (char *)current->pixels +
		vinfo.yres_virtual*current->pitch;
	surfaces_len = (shadow_fb) ?
		0 : (mapped_memlen-(surfaces_mem-mapped_mem));

	FB_FreeHWSurfaces(this);
	FB_InitHWSurfaces(this, current, surfaces_mem, surfaces_len);

	/* Let the application know we have a hardware palette */
	switch (finfo.visual) {
		case FB_VISUAL_PSEUDOCOLOR:
		current->flags |= SDL_HWPALETTE;
		break;
		default:
		break;
	}

	/* Update for double-buffering, if we can */
	if ( flags & SDL_DOUBLEBUF ) {
		if ( vinfo.yres_virtual == (height*2) ) {
			current->flags |= SDL_DOUBLEBUF;
			flip_page = 0;
			flip_address[0] = (char *)current->pixels;
			flip_address[1] = (char *)current->pixels+
				current->h*current->pitch;
			this->screen = current;
			FB_FlipHWSurface(this, current);
			this->screen = NULL;
		}
	}

	/* Set the update rectangle function */
	this->UpdateRects = FB_DirectUpdate;

	/* We're done */
	return(current);
}

#ifdef FBCON_DEBUG
void FB_DumpHWSurfaces(_THIS)
{
	vidmem_bucket *bucket;

	printf("Memory left: %d (%d total)\n", surfaces_memleft, surfaces_memtotal);
	printf("\n");
	printf("         Base  Size\n");
	for ( bucket=&surfaces; bucket; bucket=bucket->next ) {
		printf("Bucket:  %p, %d (%s)\n", bucket->base, bucket->size, bucket->used ? "used" : "free");
		if ( bucket->prev ) {
			if ( bucket->base != bucket->prev->base+bucket->prev->size ) {
				printf("Warning, corrupt bucket list! (prev)\n");
			}
		} else {
			if ( bucket != &surfaces ) {
				printf("Warning, corrupt bucket list! (!prev)\n");
			}
		}
		if ( bucket->next ) {
			if ( bucket->next->base != bucket->base+bucket->size ) {
				printf("Warning, corrupt bucket list! (next)\n");
			}
		}
	}
	printf("\n");
}
#endif

static int FB_InitHWSurfaces(_THIS, SDL_Surface *screen, char *base, int size)
{
	vidmem_bucket *bucket;

	surfaces_memtotal = size;
	surfaces_memleft = size;

	if ( surfaces_memleft > 0 ) {
		bucket = (vidmem_bucket *)SDL_malloc(sizeof(*bucket));
		if ( bucket == NULL ) {
			SDL_OutOfMemory();
			return(-1);
		}
		bucket->prev = &surfaces;
		bucket->used = 0;
		bucket->dirty = 0;
		bucket->base = base;
		bucket->size = size;
		bucket->next = NULL;
	} else {
		bucket = NULL;
	}

	surfaces.prev = NULL;
	surfaces.used = 1;
	surfaces.dirty = 0;
	surfaces.base = screen->pixels;
	surfaces.size = (unsigned int)((long)base - (long)surfaces.base);
	surfaces.next = bucket;
	screen->hwdata = (struct private_hwdata *)&surfaces;
	return(0);
}
static void FB_FreeHWSurfaces(_THIS)
{
	vidmem_bucket *bucket, *freeable;

	bucket = surfaces.next;
	while ( bucket ) {
		freeable = bucket;
		bucket = bucket->next;
		SDL_free(freeable);
	}
	surfaces.next = NULL;
}

static int FB_AllocHWSurface(_THIS, SDL_Surface *surface)
{
	vidmem_bucket *bucket;
	int size;
	int extra;

/* Temporarily, we only allow surfaces the same width as display.
   Some blitters require the pitch between two hardware surfaces
   to be the same.  Others have interesting alignment restrictions.
   Until someone who knows these details looks at the code...
*/
if ( surface->pitch > SDL_VideoSurface->pitch ) {
	SDL_SetError("Surface requested wider than screen");
	return(-1);
}
surface->pitch = SDL_VideoSurface->pitch;
	size = surface->h * surface->pitch;
#ifdef FBCON_DEBUG
	fprintf(stderr, "Allocating bucket of %d bytes\n", size);
#endif

	/* Quick check for available mem */
	if ( size > surfaces_memleft ) {
		SDL_SetError("Not enough video memory");
		return(-1);
	}

	/* Search for an empty bucket big enough */
	for ( bucket=&surfaces; bucket; bucket=bucket->next ) {
		if ( ! bucket->used && (size <= bucket->size) ) {
			break;
		}
	}
	if ( bucket == NULL ) {
		SDL_SetError("Video memory too fragmented");
		return(-1);
	}

	/* Create a new bucket for left-over memory */
	extra = (bucket->size - size);
	if ( extra ) {
		vidmem_bucket *newbucket;

#ifdef FBCON_DEBUG
	fprintf(stderr, "Adding new free bucket of %d bytes\n", extra);
#endif
		newbucket = (vidmem_bucket *)SDL_malloc(sizeof(*newbucket));
		if ( newbucket == NULL ) {
			SDL_OutOfMemory();
			return(-1);
		}
		newbucket->prev = bucket;
		newbucket->used = 0;
		newbucket->base = bucket->base+size;
		newbucket->size = extra;
		newbucket->next = bucket->next;
		if ( bucket->next ) {
			bucket->next->prev = newbucket;
		}
		bucket->next = newbucket;
	}

	/* Set the current bucket values and return it! */
	bucket->used = 1;
	bucket->size = size;
	bucket->dirty = 0;
#ifdef FBCON_DEBUG
	fprintf(stderr, "Allocated %d bytes at %p\n", bucket->size, bucket->base);
#endif
	surfaces_memleft -= size;
	surface->flags |= SDL_HWSURFACE;
	surface->pixels = bucket->base;
	surface->hwdata = (struct private_hwdata *)bucket;
	return(0);
}
static void FB_FreeHWSurface(_THIS, SDL_Surface *surface)
{
	vidmem_bucket *bucket, *freeable;

	/* Look for the bucket in the current list */
	for ( bucket=&surfaces; bucket; bucket=bucket->next ) {
		if ( bucket == (vidmem_bucket *)surface->hwdata ) {
			break;
		}
	}
	if ( bucket && bucket->used ) {
		/* Add the memory back to the total */
#ifdef DGA_DEBUG
	printf("Freeing bucket of %d bytes\n", bucket->size);
#endif
		surfaces_memleft += bucket->size;

		/* Can we merge the space with surrounding buckets? */
		bucket->used = 0;
		if ( bucket->next && ! bucket->next->used ) {
#ifdef DGA_DEBUG
	printf("Merging with next bucket, for %d total bytes\n", bucket->size+bucket->next->size);
#endif
			freeable = bucket->next;
			bucket->size += bucket->next->size;
			bucket->next = bucket->next->next;
			if ( bucket->next ) {
				bucket->next->prev = bucket;
			}
			SDL_free(freeable);
		}
		if ( bucket->prev && ! bucket->prev->used ) {
#ifdef DGA_DEBUG
	printf("Merging with previous bucket, for %d total bytes\n", bucket->prev->size+bucket->size);
#endif
			freeable = bucket;
			bucket->prev->size += bucket->size;
			bucket->prev->next = bucket->next;
			if ( bucket->next ) {
				bucket->next->prev = bucket->prev;
			}
			SDL_free(freeable);
		}
	}
	surface->pixels = NULL;
	surface->hwdata = NULL;
}

static int FB_LockHWSurface(_THIS, SDL_Surface *surface)
{
	if ( switched_away ) {
		return -2; /* no hardware access */
	}
	if ( surface == this->screen ) {
		SDL_mutexP(hw_lock);
		if ( FB_IsSurfaceBusy(surface) ) {
			FB_WaitBusySurfaces(this);
		}
	} else {
		if ( FB_IsSurfaceBusy(surface) ) {
			FB_WaitBusySurfaces(this);
		}
	}
	return(0);
}
static void FB_UnlockHWSurface(_THIS, SDL_Surface *surface)
{
	if ( surface == this->screen ) {
		SDL_mutexV(hw_lock);
	}
}

static void FB_WaitVBL(_THIS)
{
#ifdef FBIOWAITRETRACE /* Heheh, this didn't make it into the main kernel */
	ioctl(console_fd, FBIOWAITRETRACE, 0);
#endif
	return;
}

static void FB_WaitIdle(_THIS)
{
	return;
}

static int FB_FlipHWSurface(_THIS, SDL_Surface *surface)
{
	if ( switched_away ) {
		return -2; /* no hardware access */
	}

	/* Wait for vertical retrace and then flip display */
	cache_vinfo.yoffset = flip_page*surface->h;
	if ( FB_IsSurfaceBusy(this->screen) ) {
		FB_WaitBusySurfaces(this);
	}
	wait_vbl(this);
	if ( ioctl(console_fd, FBIOPAN_DISPLAY, &cache_vinfo) < 0 ) {
		SDL_SetError("ioctl(FBIOPAN_DISPLAY) failed");
		return(-1);
	}
	flip_page = !flip_page;

	surface->pixels = flip_address[flip_page];
	return(0);
}

static void FB_blit16(Uint8 *byte_src_pos, int src_right_delta, int src_down_delta,
		Uint8 *byte_dst_pos, int dst_linebytes, int width, int height)
{
	int w;
	Uint16 *src_pos = (Uint16 *)byte_src_pos;
	Uint16 *dst_pos = (Uint16 *)byte_dst_pos;

	while (height) {
		Uint16 *src = src_pos;
		Uint16 *dst = dst_pos;
		for (w = width; w != 0; w--) {
			*dst = *src;
			src += src_right_delta;
			dst++;
		}
		dst_pos = (Uint16 *)((Uint8 *)dst_pos + dst_linebytes);
		src_pos += src_down_delta;
		height--;
	}
}

#define BLOCKSIZE_W 32
#define BLOCKSIZE_H 32

static void FB_blit16blocked(Uint8 *byte_src_pos, int src_right_delta, int src_down_delta, 
		Uint8 *byte_dst_pos, int dst_linebytes, int width, int height)
{
	int w;
	Uint16 *src_pos = (Uint16 *)byte_src_pos;
	Uint16 *dst_pos = (Uint16 *)byte_dst_pos;

	while (height > 0) {
		Uint16 *src = src_pos;
		Uint16 *dst = dst_pos;
		for (w = width; w > 0; w -= BLOCKSIZE_W) {
			FB_blit16((Uint8 *)src,
					src_right_delta,
					src_down_delta,
					(Uint8 *)dst,
					dst_linebytes,
					min(w, BLOCKSIZE_W),
					min(height, BLOCKSIZE_H));
			src += src_right_delta * BLOCKSIZE_W;
			dst += BLOCKSIZE_W;
		}
		dst_pos = (Uint16 *)((Uint8 *)dst_pos + dst_linebytes * BLOCKSIZE_H);
		src_pos += src_down_delta * BLOCKSIZE_H;
		height -= BLOCKSIZE_H;
	}
}

static void FB_DirectUpdate(_THIS, int numrects, SDL_Rect *rects)
{
	int width = cache_vinfo.xres;
	int height = cache_vinfo.yres;
	int bytes_per_pixel = (cache_vinfo.bits_per_pixel + 7) / 8;
	int i;

	if (!shadow_fb) {
		/* The application is already updating the visible video memory */
		return;
	}

	if (cache_vinfo.bits_per_pixel != 16) {
		SDL_SetError("Shadow copy only implemented for 16 bpp");
		return;
	}

	for (i = 0; i < numrects; i++) {
		int x1, y1, x2, y2;
		int scr_x1, scr_y1, scr_x2, scr_y2;
		int sha_x1, sha_y1;
		int shadow_right_delta;  /* Address change when moving right in dest */
		int shadow_down_delta;   /* Address change when moving down in dest */
		char *src_start;
		char *dst_start;

		x1 = rects[i].x; 
		y1 = rects[i].y;
		x2 = x1 + rects[i].w; 
		y2 = y1 + rects[i].h;

		if (x1 < 0) {
			x1 = 0;
		} else if (x1 > width) {
			x1 = width;
		}
		if (x2 < 0) {
			x2 = 0;
		} else if (x2 > width) {
			x2 = width;
		}
		if (y1 < 0) {
			y1 = 0;
		} else if (y1 > height) {
			y1 = height;
		}
		if (y2 < 0) {
			y2 = 0;
		} else if (y2 > height) {
			y2 = height;
		}
		if (x2 <= x1 || y2 <= y1) {
			continue;
		}

		switch (rotate) {
			case FBCON_ROTATE_NONE:
				sha_x1 = scr_x1 = x1;
				sha_y1 = scr_y1 = y1;
				scr_x2 = x2;
				scr_y2 = y2;
				shadow_right_delta = 1;
				shadow_down_delta = width;
				break;
			case FBCON_ROTATE_CCW:
				scr_x1 = y1;
				scr_y1 = width - x2;
				scr_x2 = y2;
				scr_y2 = width - x1;
				sha_x1 = x2 - 1;
				sha_y1 = y1;
				shadow_right_delta = width;
				shadow_down_delta = -1;
				break;
			case FBCON_ROTATE_UD:
				scr_x1 = width - x2;
				scr_y1 = height - y2;
				scr_x2 = width - x1;
				scr_y2 = height - y1;
				sha_x1 = x2 - 1;
				sha_y1 = y2 - 1;
				shadow_right_delta = -1;
				shadow_down_delta = -width;
				break;
			case FBCON_ROTATE_CW:
				scr_x1 = height - y2;
				scr_y1 = x1;
				scr_x2 = height - y1;
				scr_y2 = x2;
				sha_x1 = x1;
				sha_y1 = y2 - 1;
				shadow_right_delta = -width;
				shadow_down_delta = 1;
				break;
			default:
				SDL_SetError("Unknown rotation");
				return;
		}

		src_start = shadow_mem +
			(sha_y1 * width + sha_x1) * bytes_per_pixel;
		dst_start = mapped_mem + mapped_offset + scr_y1 * physlinebytes + 
			scr_x1 * bytes_per_pixel;

		blitFunc((Uint8 *) src_start,
				shadow_right_delta, 
				shadow_down_delta, 
				(Uint8 *) dst_start,
				physlinebytes,
				scr_x2 - scr_x1,
				scr_y2 - scr_y1);
	}
}

#ifdef VGA16_FBCON_SUPPORT
/* Code adapted with thanks from the XFree86 VGA16 driver! :) */
#define writeGr(index, value) \
outb(index, 0x3CE);           \
outb(value, 0x3CF);
#define writeSeq(index, value) \
outb(index, 0x3C4);            \
outb(value, 0x3C5);

static void FB_VGA16Update(_THIS, int numrects, SDL_Rect *rects)
{
    SDL_Surface *screen;
    int width, height, FBPitch, left, i, j, SRCPitch, phase;
    register Uint32 m;
    Uint8  s1, s2, s3, s4;
    Uint32 *src, *srcPtr;
    Uint8  *dst, *dstPtr;

    if ( switched_away ) {
        return; /* no hardware access */
    }

    screen = this->screen;
    FBPitch = screen->w >> 3;
    SRCPitch = screen->pitch >> 2;

    writeGr(0x03, 0x00);
    writeGr(0x05, 0x00);
    writeGr(0x01, 0x00);
    writeGr(0x08, 0xFF);

    while(numrects--) {
	left = rects->x & ~7;
        width = (rects->w + 7) >> 3;
        height = rects->h;
        src = (Uint32*)screen->pixels + (rects->y * SRCPitch) + (left >> 2); 
        dst = (Uint8*)mapped_mem + (rects->y * FBPitch) + (left >> 3);

	if((phase = (long)dst & 3L)) {
	    phase = 4 - phase;
	    if(phase > width) phase = width;
	    width -= phase;
	}

        while(height--) {
	    writeSeq(0x02, 1 << 0);
	    dstPtr = dst;
	    srcPtr = src;
	    i = width;
	    j = phase;
	    while(j--) {
		m = (srcPtr[1] & 0x01010101) | ((srcPtr[0] & 0x01010101) << 4);
 		*dstPtr++ = (m >> 24) | (m >> 15) | (m >> 6) | (m << 3);
		srcPtr += 2;
	    }
	    while(i >= 4) {
		m = (srcPtr[1] & 0x01010101) | ((srcPtr[0] & 0x01010101) << 4);
 		s1 = (m >> 24) | (m >> 15) | (m >> 6) | (m << 3);
		m = (srcPtr[3] & 0x01010101) | ((srcPtr[2] & 0x01010101) << 4);
 		s2 = (m >> 24) | (m >> 15) | (m >> 6) | (m << 3);
		m = (srcPtr[5] & 0x01010101) | ((srcPtr[4] & 0x01010101) << 4);
 		s3 = (m >> 24) | (m >> 15) | (m >> 6) | (m << 3);
		m = (srcPtr[7] & 0x01010101) | ((srcPtr[6] & 0x01010101) << 4);
 		s4 = (m >> 24) | (m >> 15) | (m >> 6) | (m << 3);
		*((Uint32*)dstPtr) = s1 | (s2 << 8) | (s3 << 16) | (s4 << 24);
		srcPtr += 8;
		dstPtr += 4;
		i -= 4;
	    }
	    while(i--) {
		m = (srcPtr[1] & 0x01010101) | ((srcPtr[0] & 0x01010101) << 4);
 		*dstPtr++ = (m >> 24) | (m >> 15) | (m >> 6) | (m << 3);
		srcPtr += 2;
	    }

	    writeSeq(0x02, 1 << 1);
	    dstPtr = dst;
	    srcPtr = src;
	    i = width;
	    j = phase;
	    while(j--) {
		m = (srcPtr[1] & 0x02020202) | ((srcPtr[0] & 0x02020202) << 4);
 		*dstPtr++ = (m >> 25) | (m >> 16) | (m >> 7) | (m << 2);
		srcPtr += 2;
	    }
	    while(i >= 4) {
		m = (srcPtr[1] & 0x02020202) | ((srcPtr[0] & 0x02020202) << 4);
 		s1 = (m >> 25) | (m >> 16) | (m >> 7) | (m << 2);
		m = (srcPtr[3] & 0x02020202) | ((srcPtr[2] & 0x02020202) << 4);
 		s2 = (m >> 25) | (m >> 16) | (m >> 7) | (m << 2);
		m = (srcPtr[5] & 0x02020202) | ((srcPtr[4] & 0x02020202) << 4);
 		s3 = (m >> 25) | (m >> 16) | (m >> 7) | (m << 2);
		m = (srcPtr[7] & 0x02020202) | ((srcPtr[6] & 0x02020202) << 4);
 		s4 = (m >> 25) | (m >> 16) | (m >> 7) | (m << 2);
		*((Uint32*)dstPtr) = s1 | (s2 << 8) | (s3 << 16) | (s4 << 24);
		srcPtr += 8;
		dstPtr += 4;
		i -= 4;
	    }
	    while(i--) {
		m = (srcPtr[1] & 0x02020202) | ((srcPtr[0] & 0x02020202) << 4);
 		*dstPtr++ = (m >> 25) | (m >> 16) | (m >> 7) | (m << 2);
		srcPtr += 2;
	    }

	    writeSeq(0x02, 1 << 2);
	    dstPtr = dst;
	    srcPtr = src;
	    i = width;
	    j = phase;
	    while(j--) {
		m = (srcPtr[1] & 0x04040404) | ((srcPtr[0] & 0x04040404) << 4);
 		*dstPtr++ = (m >> 26) | (m >> 17) | (m >> 8) | (m << 1);
		srcPtr += 2;
	    }
	    while(i >= 4) {
		m = (srcPtr[1] & 0x04040404) | ((srcPtr[0] & 0x04040404) << 4);
 		s1 = (m >> 26) | (m >> 17) | (m >> 8) | (m << 1);
		m = (srcPtr[3] & 0x04040404) | ((srcPtr[2] & 0x04040404) << 4);
 		s2 = (m >> 26) | (m >> 17) | (m >> 8) | (m << 1);
		m = (srcPtr[5] & 0x04040404) | ((srcPtr[4] & 0x04040404) << 4);
 		s3 = (m >> 26) | (m >> 17) | (m >> 8) | (m << 1);
		m = (srcPtr[7] & 0x04040404) | ((srcPtr[6] & 0x04040404) << 4);
 		s4 = (m >> 26) | (m >> 17) | (m >> 8) | (m << 1);
		*((Uint32*)dstPtr) = s1 | (s2 << 8) | (s3 << 16) | (s4 << 24);
		srcPtr += 8;
		dstPtr += 4;
		i -= 4;
	    }
	    while(i--) {
		m = (srcPtr[1] & 0x04040404) | ((srcPtr[0] & 0x04040404) << 4);
 		*dstPtr++ = (m >> 26) | (m >> 17) | (m >> 8) | (m << 1);
		srcPtr += 2;
	    }
	    
	    writeSeq(0x02, 1 << 3);
	    dstPtr = dst;
	    srcPtr = src;
	    i = width;
	    j = phase;
	    while(j--) {
		m = (srcPtr[1] & 0x08080808) | ((srcPtr[0] & 0x08080808) << 4);
 		*dstPtr++ = (m >> 27) | (m >> 18) | (m >> 9) | m;
		srcPtr += 2;
	    }
	    while(i >= 4) {
		m = (srcPtr[1] & 0x08080808) | ((srcPtr[0] & 0x08080808) << 4);
 		s1 = (m >> 27) | (m >> 18) | (m >> 9) | m;
		m = (srcPtr[3] & 0x08080808) | ((srcPtr[2] & 0x08080808) << 4);
 		s2 = (m >> 27) | (m >> 18) | (m >> 9) | m;
		m = (srcPtr[5] & 0x08080808) | ((srcPtr[4] & 0x08080808) << 4);
 		s3 = (m >> 27) | (m >> 18) | (m >> 9) | m;
		m = (srcPtr[7] & 0x08080808) | ((srcPtr[6] & 0x08080808) << 4);
 		s4 = (m >> 27) | (m >> 18) | (m >> 9) | m;
		*((Uint32*)dstPtr) = s1 | (s2 << 8) | (s3 << 16) | (s4 << 24);
		srcPtr += 8;
		dstPtr += 4;
		i -= 4;
	    }
	    while(i--) {
		m = (srcPtr[1] & 0x08080808) | ((srcPtr[0] & 0x08080808) << 4);
 		*dstPtr++ = (m >> 27) | (m >> 18) | (m >> 9) | m;
		srcPtr += 2;
	    }

            dst += FBPitch;
            src += SRCPitch;
        }
        rects++;
    }
}
#endif /* VGA16_FBCON_SUPPORT */

void FB_SavePaletteTo(_THIS, int palette_len, __u16 *area)
{
	struct fb_cmap cmap;

	cmap.start = 0;
	cmap.len = palette_len;
	cmap.red = &area[0*palette_len];
	cmap.green = &area[1*palette_len];
	cmap.blue = &area[2*palette_len];
	cmap.transp = NULL;
	ioctl(console_fd, FBIOGETCMAP, &cmap);
}

void FB_RestorePaletteFrom(_THIS, int palette_len, __u16 *area)
{
	struct fb_cmap cmap;

	cmap.start = 0;
	cmap.len = palette_len;
	cmap.red = &area[0*palette_len];
	cmap.green = &area[1*palette_len];
	cmap.blue = &area[2*palette_len];
	cmap.transp = NULL;
	ioctl(console_fd, FBIOPUTCMAP, &cmap);
}

static void FB_SavePalette(_THIS, struct fb_fix_screeninfo *finfo,
                                  struct fb_var_screeninfo *vinfo)
{
	int i;

	/* Save hardware palette, if needed */
	if ( finfo->visual == FB_VISUAL_PSEUDOCOLOR ) {
		saved_cmaplen = 1<<vinfo->bits_per_pixel;
		saved_cmap=(__u16 *)SDL_malloc(3*saved_cmaplen*sizeof(*saved_cmap));
		if ( saved_cmap != NULL ) {
			FB_SavePaletteTo(this, saved_cmaplen, saved_cmap);
		}
	}

	/* Added support for FB_VISUAL_DIRECTCOLOR.
	   With this mode pixel information is passed through the palette...
	   Neat fading and gamma correction effects can be had by simply
	   fooling around with the palette instead of changing the pixel
	   values themselves... Very neat!

	   Adam Meyerowitz 1/19/2000
	   ameyerow@optonline.com
	*/
	if ( finfo->visual == FB_VISUAL_DIRECTCOLOR ) {
		__u16 new_entries[3*256];

		/* Save the colormap */
		saved_cmaplen = 256;
		saved_cmap=(__u16 *)SDL_malloc(3*saved_cmaplen*sizeof(*saved_cmap));
		if ( saved_cmap != NULL ) {
			FB_SavePaletteTo(this, saved_cmaplen, saved_cmap);
		}

		/* Allocate new identity colormap */
		for ( i=0; i<256; ++i ) {
	      		new_entries[(0*256)+i] =
			new_entries[(1*256)+i] =
			new_entries[(2*256)+i] = (i<<8)|i;
		}
		FB_RestorePaletteFrom(this, 256, new_entries);
	}
}

static void FB_RestorePalette(_THIS)
{
	/* Restore the original palette */
	if ( saved_cmap ) {
		FB_RestorePaletteFrom(this, saved_cmaplen, saved_cmap);
		SDL_free(saved_cmap);
		saved_cmap = NULL;
	}
}

static int FB_SetColors(_THIS, int firstcolor, int ncolors, SDL_Color *colors)
{
	int i;
	__u16 r[256];
	__u16 g[256];
	__u16 b[256];
	struct fb_cmap cmap;

	/* Set up the colormap */
	for (i = 0; i < ncolors; i++) {
		r[i] = colors[i].r << 8;
		g[i] = colors[i].g << 8;
		b[i] = colors[i].b << 8;
	}
	cmap.start = firstcolor;
	cmap.len = ncolors;
	cmap.red = r;
	cmap.green = g;
	cmap.blue = b;
	cmap.transp = NULL;

	if( (ioctl(console_fd, FBIOPUTCMAP, &cmap) < 0) ||
	    !(this->screen->flags & SDL_HWPALETTE) ) {
	        colors = this->screen->format->palette->colors;
		ncolors = this->screen->format->palette->ncolors;
		cmap.start = 0;
		cmap.len = ncolors;
		SDL_memset(r, 0, sizeof(r));
		SDL_memset(g, 0, sizeof(g));
		SDL_memset(b, 0, sizeof(b));
		if ( ioctl(console_fd, FBIOGETCMAP, &cmap) == 0 ) {
			for ( i=ncolors-1; i>=0; --i ) {
				colors[i].r = (r[i]>>8);
				colors[i].g = (g[i]>>8);
				colors[i].b = (b[i]>>8);
			}
		}
		return(0);
	}
	return(1);
}

/* Note:  If we are terminated, this could be called in the middle of
   another SDL video routine -- notably UpdateRects.
*/
static void FB_VideoQuit(_THIS)
{
	int i, j;

	if ( this->screen ) {
		/* Clear screen and tell SDL not to free the pixels */

		const char *dontClearPixels = SDL_getenv("SDL_FBCON_DONT_CLEAR");

		/* If the framebuffer is not to be cleared, make sure that we won't
		 * display the previous frame when disabling double buffering. */
		if ( dontClearPixels && flip_page == 0 ) {
			SDL_memcpy(flip_address[0], flip_address[1], this->screen->pitch * this->screen->h);
		}

		if ( !dontClearPixels && this->screen->pixels && FB_InGraphicsMode(this) ) {
#if defined(__powerpc__) || defined(__ia64__)	/* SIGBUS when using SDL_memset() ?? */
			Uint8 *rowp = (Uint8 *)this->screen->pixels;
			int left = this->screen->pitch*this->screen->h;
			while ( left-- ) { *rowp++ = 0; }
#else
			SDL_memset(this->screen->pixels,0,this->screen->h*this->screen->pitch);
#endif
		}
		/* This test fails when using the VGA16 shadow memory */
		if ( ((char *)this->screen->pixels >= mapped_mem) &&
		     ((char *)this->screen->pixels < (mapped_mem+mapped_memlen)) ) {
			this->screen->pixels = NULL;
		}
	}

	/* Clear the lock mutex */
	if ( hw_lock ) {
		SDL_DestroyMutex(hw_lock);
		hw_lock = NULL;
	}

	/* Clean up defined video modes */
	for ( i=0; i<NUM_MODELISTS; ++i ) {
		if ( SDL_modelist[i] != NULL ) {
			for ( j=0; SDL_modelist[i][j]; ++j ) {
				SDL_free(SDL_modelist[i][j]);
			}
			SDL_free(SDL_modelist[i]);
			SDL_modelist[i] = NULL;
		}
	}

	/* Clean up the memory bucket list */
	FB_FreeHWSurfaces(this);

	/* Close console and input file descriptors */
	if ( console_fd > 0 ) {
		/* Unmap the video framebuffer and I/O registers */
		if ( mapped_mem ) {
			munmap(mapped_mem, mapped_memlen);
			mapped_mem = NULL;
		}
		if ( mapped_io ) {
			munmap(mapped_io, mapped_iolen);
			mapped_io = NULL;
		}

		/* Restore the original video mode and palette */
		if ( FB_InGraphicsMode(this) ) {
			FB_RestorePalette(this);
			ioctl(console_fd, FBIOPUT_VSCREENINFO, &saved_vinfo);
		}

		/* We're all done with the framebuffer */
		close(console_fd);
		console_fd = -1;
	}
	FB_CloseMouse(this);
	FB_CloseKeyboard(this);
}
