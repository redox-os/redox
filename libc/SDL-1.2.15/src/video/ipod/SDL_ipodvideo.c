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

#include <sys/types.h>
#include <sys/ioctl.h>

#include <unistd.h>
#include <fcntl.h>
#include <string.h>
#include <termios.h>
#include <ctype.h>

#include <linux/vt.h>
#include <linux/kd.h>
#include <linux/keyboard.h>
#include <linux/fb.h>

#include "SDL_video.h"
#include "SDL_mouse.h"
#include "../SDL_sysvideo.h"
#include "../SDL_pixels_c.h"
#include "../../events/SDL_events_c.h"
#include "SDL_sysevents.h"
#include "SDL_ipodvideo.h"

#define _THIS SDL_VideoDevice *this

static int iPod_VideoInit (_THIS, SDL_PixelFormat *vformat);
static SDL_Rect **iPod_ListModes (_THIS, SDL_PixelFormat *format, Uint32 flags);
static SDL_Surface *iPod_SetVideoMode (_THIS, SDL_Surface *current, int width, int height, int bpp, Uint32 flags);
static int iPod_SetColors (_THIS, int firstcolor, int ncolors, SDL_Color *colors);
static void iPod_UpdateRects (_THIS, int nrects, SDL_Rect *rects);
static void iPod_VideoQuit (_THIS);
static void iPod_PumpEvents (_THIS);

static long iPod_GetGeneration();

static int initd = 0;
static int kbfd = -1;
static int fbfd = -1;
static int oldvt = -1;
static int curvt = -1;
static int old_kbmode = -1;
static long generation = 0;
static struct termios old_termios, cur_termios;

FILE *dbgout;

#define LCD_DATA          0x10
#define LCD_CMD           0x08
#define IPOD_OLD_LCD_BASE 0xc0001000
#define IPOD_OLD_LCD_RTC  0xcf001110
#define IPOD_NEW_LCD_BASE 0x70003000
#define IPOD_NEW_LCD_RTC  0x60005010

static unsigned long lcd_base, lcd_rtc, lcd_width, lcd_height;

static long iPod_GetGeneration() 
{
    int i;
    char cpuinfo[256];
    char *ptr;
    FILE *file;
    
    if ((file = fopen("/proc/cpuinfo", "r")) != NULL) {
	while (fgets(cpuinfo, sizeof(cpuinfo), file) != NULL)
	    if (SDL_strncmp(cpuinfo, "Revision", 8) == 0)
		break;
	fclose(file);
    }
    for (i = 0; !isspace(cpuinfo[i]); i++);
    for (; isspace(cpuinfo[i]); i++);
    ptr = cpuinfo + i + 2;
    
    return SDL_strtol(ptr, NULL, 10);
}

static int iPod_Available() 
{
    return 1;
}

static void iPod_DeleteDevice (SDL_VideoDevice *device)
{
    free (device->hidden);
    free (device);
}

void iPod_InitOSKeymap (_THIS) {}

static SDL_VideoDevice *iPod_CreateDevice (int devindex)
{
    SDL_VideoDevice *this;
    
    this = (SDL_VideoDevice *)SDL_malloc (sizeof(SDL_VideoDevice));
    if (this) {
	memset (this, 0, sizeof *this);
	this->hidden = (struct SDL_PrivateVideoData *) SDL_malloc (sizeof(struct SDL_PrivateVideoData));
    }
    if (!this || !this->hidden) {
	SDL_OutOfMemory();
	if (this)
	    SDL_free (this);
	return 0;
    }
    memset (this->hidden, 0, sizeof(struct SDL_PrivateVideoData));
    
    generation = iPod_GetGeneration();

    this->VideoInit = iPod_VideoInit;
    this->ListModes = iPod_ListModes;
    this->SetVideoMode = iPod_SetVideoMode;
    this->SetColors = iPod_SetColors;
    this->UpdateRects = iPod_UpdateRects;
    this->VideoQuit = iPod_VideoQuit;
    this->AllocHWSurface = 0;
    this->CheckHWBlit = 0;
    this->FillHWRect = 0;
    this->SetHWColorKey = 0;
    this->SetHWAlpha = 0;
    this->LockHWSurface = 0;
    this->UnlockHWSurface = 0;
    this->FlipHWSurface = 0;
    this->FreeHWSurface = 0;
    this->SetCaption = 0;
    this->SetIcon = 0;
    this->IconifyWindow = 0;
    this->GrabInput = 0;
    this->GetWMInfo = 0;
    this->InitOSKeymap = iPod_InitOSKeymap;
    this->PumpEvents = iPod_PumpEvents;
    this->free = iPod_DeleteDevice;

    return this;
}

VideoBootStrap iPod_bootstrap = {
    "ipod", "iPod Framebuffer Driver",
    iPod_Available, iPod_CreateDevice
};

//--//

static int iPod_VideoInit (_THIS, SDL_PixelFormat *vformat)
{
    if (!initd) {
	/*** Code adapted/copied from SDL fbcon driver. ***/

	static const char * const tty0[] = { "/dev/tty0", "/dev/vc/0", 0 };
	static const char * const vcs[] = { "/dev/vc/%d", "/dev/tty%d", 0 };
	int i, tty0_fd;

	dbgout = fdopen (open ("/etc/sdlpod.log", O_WRONLY | O_SYNC | O_APPEND), "a");
	if (dbgout) {
	    setbuf (dbgout, 0);
	    fprintf (dbgout, "--> Started SDL <--\n");
	}

	// Try to query for a free VT
	tty0_fd = -1;
	for ( i=0; tty0[i] && (tty0_fd < 0); ++i ) {
	    tty0_fd = open(tty0[i], O_WRONLY, 0);
	}
	if ( tty0_fd < 0 ) {
	    tty0_fd = dup(0); /* Maybe stdin is a VT? */
	}
	ioctl(tty0_fd, VT_OPENQRY, &curvt);
	close(tty0_fd);

	tty0_fd = open("/dev/tty", O_RDWR, 0);
	if ( tty0_fd >= 0 ) {
	    ioctl(tty0_fd, TIOCNOTTY, 0);
	    close(tty0_fd);
	}

	if ( (geteuid() == 0) && (curvt > 0) ) {
	    for ( i=0; vcs[i] && (kbfd < 0); ++i ) {
		char vtpath[12];
		
		SDL_snprintf(vtpath, SDL_arraysize(vtpath), vcs[i], curvt);
		kbfd = open(vtpath, O_RDWR);
	    }
	}
	if ( kbfd < 0 ) {
	    if (dbgout) fprintf (dbgout, "Couldn't open any VC\n");
	    return -1;
	}
	if (dbgout) fprintf (stderr, "Current VT: %d\n", curvt);

	if (kbfd >= 0) {
	    /* Switch to the correct virtual terminal */
	    if ( curvt > 0 ) {
		struct vt_stat vtstate;
		
		if ( ioctl(kbfd, VT_GETSTATE, &vtstate) == 0 ) {
		    oldvt = vtstate.v_active;
		}
		if ( ioctl(kbfd, VT_ACTIVATE, curvt) == 0 ) {
		    if (dbgout) fprintf (dbgout, "Waiting for switch to this VT... ");
		    ioctl(kbfd, VT_WAITACTIVE, curvt);
		    if (dbgout) fprintf (dbgout, "done!\n");
		}
	    }

	    // Set terminal input mode
	    if (tcgetattr (kbfd, &old_termios) < 0) {
		if (dbgout) fprintf (dbgout, "Can't get termios\n");
		return -1;
	    }
	    cur_termios = old_termios;
	    //	    cur_termios.c_iflag &= ~(ICRNL | INPCK | ISTRIP | IXON);
	    //	    cur_termios.c_iflag |= (BRKINT);
	    //	    cur_termios.c_lflag &= ~(ICANON | ECHO | ISIG | IEXTEN);
	    //	    cur_termios.c_oflag &= ~(OPOST);
	    //	    cur_termios.c_oflag |= (ONOCR | ONLRET);
	    cur_termios.c_lflag &= ~(ICANON | ECHO | ISIG);
	    cur_termios.c_iflag &= ~(ISTRIP | IGNCR | ICRNL | INLCR | IXOFF | IXON);
	    cur_termios.c_cc[VMIN] = 0;
	    cur_termios.c_cc[VTIME] = 0;
	    
	    if (tcsetattr (kbfd, TCSAFLUSH, &cur_termios) < 0) {
		if (dbgout) fprintf (dbgout, "Can't set termios\n");
		return -1;
	    }
	    if (ioctl (kbfd, KDSKBMODE, K_MEDIUMRAW) < 0) {
		if (dbgout) fprintf (dbgout, "Can't set medium-raw mode\n");
		return -1;
	    }
	    if (ioctl (kbfd, KDSETMODE, KD_GRAPHICS) < 0) {
		if (dbgout) fprintf (dbgout, "Can't set graphics\n");
		return -1;
	    }
	}

	// Open the framebuffer
	if ((fbfd = open ("/dev/fb0", O_RDWR)) < 0) {
	    if (dbgout) fprintf (dbgout, "Can't open framebuffer\n");
	    return -1;
	} else {
	    struct fb_var_screeninfo vinfo;

	    if (dbgout) fprintf (dbgout, "Generation: %ld\n", generation);

	    if (generation >= 40000) {
		lcd_base = IPOD_NEW_LCD_BASE;
	    } else {
		lcd_base = IPOD_OLD_LCD_BASE;
	    }
	    
	    ioctl (fbfd, FBIOGET_VSCREENINFO, &vinfo);
	    close (fbfd);

	    if (lcd_base == IPOD_OLD_LCD_BASE)
		lcd_rtc = IPOD_OLD_LCD_RTC;
	    else if (lcd_base == IPOD_NEW_LCD_BASE)
		lcd_rtc = IPOD_NEW_LCD_RTC;
	    else {
		SDL_SetError ("Unknown iPod version");
		return -1;
	    }

	    lcd_width = vinfo.xres;
	    lcd_height = vinfo.yres;

	    if (dbgout) fprintf (dbgout, "LCD is %dx%d\n", lcd_width, lcd_height);
	}

	fcntl (kbfd, F_SETFL, O_RDWR | O_NONBLOCK);

	/* Determine the current screen size */
	this->info.current_w = lcd_width;
	this->info.current_h = lcd_height;

	if ((generation >= 60000) && (generation < 70000)) {
	    vformat->BitsPerPixel = 16;
	    vformat->Rmask = 0xF800;
	    vformat->Gmask = 0x07E0;
	    vformat->Bmask = 0x001F;
	} else {
	    vformat->BitsPerPixel = 8;
	    vformat->Rmask = vformat->Gmask = vformat->Bmask = 0;
	}

	initd = 1;
	if (dbgout) fprintf (dbgout, "Initialized.\n\n");
    }
    return 0;
}

static SDL_Rect **iPod_ListModes (_THIS, SDL_PixelFormat *format, Uint32 flags)
{
    int width, height, fd;
    static SDL_Rect r;
    static SDL_Rect *rs[2] = { &r, 0 };

    if ((fd = open ("/dev/fb0", O_RDWR)) < 0) {
	return 0;
    } else {
	struct fb_var_screeninfo vinfo;
	
	ioctl (fbfd, FBIOGET_VSCREENINFO, &vinfo);
	close (fbfd);
	
	width = vinfo.xres;
	height = vinfo.yres;
    }
    r.x = r.y = 0;
    r.w = width;
    r.h = height;
    return rs;
}


static SDL_Surface *iPod_SetVideoMode (_THIS, SDL_Surface *current, int width, int height, int bpp,
				       Uint32 flags)
{
    Uint32 Rmask, Gmask, Bmask;
    if (bpp > 8) {
	Rmask = 0xF800;
	Gmask = 0x07E0;
	Bmask = 0x001F;	
    } else {
	Rmask = Gmask = Bmask = 0;
    }

    if (this->hidden->buffer) SDL_free (this->hidden->buffer);
    this->hidden->buffer = SDL_malloc (width * height * (bpp / 8));
    if (!this->hidden->buffer) {
	SDL_SetError ("Couldn't allocate buffer for requested mode");
	return 0;
    }

    memset (this->hidden->buffer, 0, width * height * (bpp / 8));

    if (!SDL_ReallocFormat (current, bpp, Rmask, Gmask, Bmask, 0)) {
	SDL_SetError ("Couldn't allocate new pixel format");
	SDL_free (this->hidden->buffer);
	this->hidden->buffer = 0;
	return 0;
    }

    if (bpp <= 8) {
	int i, j;
	for (i = 0; i < 256; i += 4) {
	    for (j = 0; j < 4; j++) {
		current->format->palette->colors[i+j].r = 85 * j;
		current->format->palette->colors[i+j].g = 85 * j;
		current->format->palette->colors[i+j].b = 85 * j;
	    }
	}
    }

    current->flags = flags & SDL_FULLSCREEN;
    this->hidden->w = current->w = width;
    this->hidden->h = current->h = height;
    current->pitch = current->w * (bpp / 8);
    current->pixels = this->hidden->buffer;

    return current;
}

static int iPod_SetColors (_THIS, int firstcolor, int ncolors, SDL_Color *colors)
{
    if (SDL_VideoSurface && SDL_VideoSurface->format && SDL_VideoSurface->format->palette) {
	int i, j;
	for (i = 0; i < 256; i += 4) {
	    for (j = 0; j < 4; j++) {
		SDL_VideoSurface->format->palette->colors[i+j].r = 85 * j;
		SDL_VideoSurface->format->palette->colors[i+j].g = 85 * j;
		SDL_VideoSurface->format->palette->colors[i+j].b = 85 * j;
	    }
	}
    }
    return 0;
}

static void iPod_VideoQuit (_THIS)
{
    ioctl (kbfd, KDSETMODE, KD_TEXT);
    tcsetattr (kbfd, TCSAFLUSH, &old_termios);
    old_kbmode = -1;

    if (oldvt > 0)
	ioctl (kbfd, VT_ACTIVATE, oldvt);
    
    if (kbfd > 0)
	close (kbfd);

    if (dbgout) {
	fprintf (dbgout, "<-- Ended SDL -->\n");
	fclose (dbgout);
    }
    
    kbfd = -1;
}

static char iPod_SC_keymap[] = {
    0,				/* 0 - no key */
    '[' - 0x40,			/* ESC (Ctrl+[) */
    '1', '2', '3', '4', '5', '6', '7', '8', '9',
    '-', '=',
    '\b', '\t',			/* Backspace, Tab (Ctrl+H,Ctrl+I) */
    'q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '[', ']',
    '\n', 0,			/* Enter, Left CTRL */
    'a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', ';', '\'', '`',
    0, '\\',			/* left shift, backslash */
    'z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/',
    0, '*', 0, ' ', 0,		/* right shift, KP mul, left alt, space, capslock */
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, /* F1-10 */
    0, 0,			/* numlock, scrollock */
    '7', '8', '9', '-', '4', '5', '6', '+', '1', '2', '3', '0', '.', /* numeric keypad */
    0, 0,			/* padding */
    0, 0, 0,			/* "less" (?), F11, F12 */
    0, 0, 0, 0, 0, 0, 0,	/* padding */
    '\n', 0, '/', 0, 0,	/* KP enter, Rctrl, Ctrl, KP div, PrtSc, RAlt */
    0, 0, 0, 0, 0, 0, 0, 0, 0,	/* Break, Home, Up, PgUp, Left, Right, End, Down, PgDn */
    0, 0,			/* Ins, Del */
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, /* padding */
    0, 0,			/* RWin, LWin */
    0				/* no key */
};
    

static void iPod_keyboard() 
{
    unsigned char keybuf[128];
    int i, nread;
    SDL_keysym keysym;
    SDL_Event ev;

    keysym.mod = 0;
    keysym.scancode = 0xff;
    memset (&ev, 0, sizeof(SDL_Event));

    nread = read (kbfd, keybuf, 128);
    for (i = 0; i < nread; i++) {
	char ascii = iPod_SC_keymap[keybuf[i] & 0x7f];

	if (dbgout) fprintf (dbgout, "Key! %02x is %c %s", keybuf[i], ascii, (keybuf[i] & 0x80)? "up" : "down");

	keysym.sym = keysym.unicode = ascii;
	ev.type = (keybuf[i] & 0x80)? SDL_KEYUP : SDL_KEYDOWN;
	ev.key.state = 0;
	ev.key.keysym = keysym;
	SDL_PushEvent (&ev);
    }
}

static void iPod_PumpEvents (_THIS) 
{
    fd_set fdset;
    int max_fd = 0;
    static struct timeval zero;
    int posted;

    do {
	posted = 0;

	FD_ZERO (&fdset);
	if (kbfd >= 0) {
	    FD_SET (kbfd, &fdset);
	    max_fd = kbfd;
	}
	if (dbgout) fprintf (dbgout, "Selecting");
	if (select (max_fd + 1, &fdset, 0, 0, &zero) > 0) {
	    if (dbgout) fprintf (dbgout, " -> match!\n");
	    iPod_keyboard();
	    posted++;
	}
	if (dbgout) fprintf (dbgout, "\n");
    } while (posted);
}

// enough space for 160x128x2
static char ipod_scr[160 * (128/4)];

#define outl(datum,addr) (*(volatile unsigned long *)(addr) = (datum))
#define inl(addr) (*(volatile unsigned long *)(addr))

/*** The following LCD code is taken from Linux kernel uclinux-2.4.24-uc0-ipod2,
     file arch/armnommu/mach-ipod/fb.c. A few modifications have been made. ***/

/* get current usec counter */
static int M_timer_get_current(void)
{
	return inl(lcd_rtc);
}

/* check if number of useconds has past */
static int M_timer_check(int clock_start, int usecs)
{
	unsigned long clock;
	clock = inl(lcd_rtc);
	
	if ( (clock - clock_start) >= usecs ) {
		return 1;
	} else {
		return 0;
	}
}

/* wait for LCD with timeout */
static void M_lcd_wait_write(void)
{
	if ( (inl(lcd_base) & 0x8000) != 0 ) {
		int start = M_timer_get_current();
			
		do {
			if ( (inl(lcd_base) & (unsigned int)0x8000) == 0 ) 
				break;
		} while ( M_timer_check(start, 1000) == 0 );
	}
}


/* send LCD data */
static void M_lcd_send_data(int data_lo, int data_hi)
{
	M_lcd_wait_write();
	
	outl(data_lo, lcd_base + LCD_DATA);
		
	M_lcd_wait_write();
	
	outl(data_hi, lcd_base + LCD_DATA);

}

/* send LCD command */
static void
M_lcd_prepare_cmd(int cmd)
{
	M_lcd_wait_write();

	outl(0x0, lcd_base + LCD_CMD);

	M_lcd_wait_write();
	
	outl(cmd, lcd_base + LCD_CMD);
	
}

/* send LCD command and data */
static void M_lcd_cmd_and_data(int cmd, int data_lo, int data_hi)
{
	M_lcd_prepare_cmd(cmd);

	M_lcd_send_data(data_lo, data_hi);
}

// Copied from uW
static void M_update_display(int sx, int sy, int mx, int my)
{
	int y;
	unsigned short cursor_pos;

	sx >>= 3;
	mx >>= 3;

	cursor_pos = sx + (sy << 5);

	for ( y = sy; y <= my; y++ ) {
		unsigned char *img_data;
		int x;

		/* move the cursor */
		M_lcd_cmd_and_data(0x11, cursor_pos >> 8, cursor_pos & 0xff);

		/* setup for printing */
		M_lcd_prepare_cmd(0x12);

		img_data = ipod_scr + (sx << 1) + (y * (lcd_width/4));

		/* loops up to 160 times */
		for ( x = sx; x <= mx; x++ ) {
		        /* display eight pixels */
			M_lcd_send_data(*(img_data + 1), *img_data);

			img_data += 2;
		}

		/* update cursor pos counter */
		cursor_pos += 0x20;
	}
}

/* get current usec counter */
static int C_timer_get_current(void)
{
	return inl(0x60005010);
}

/* check if number of useconds has past */
static int C_timer_check(int clock_start, int usecs)
{
	unsigned long clock;
	clock = inl(0x60005010);
	
	if ( (clock - clock_start) >= usecs ) {
		return 1;
	} else {
		return 0;
	}
}

/* wait for LCD with timeout */
static void C_lcd_wait_write(void)
{
	if ((inl(0x70008A0C) & 0x80000000) != 0) {
		int start = C_timer_get_current();
			
		do {
			if ((inl(0x70008A0C) & 0x80000000) == 0) 
				break;
		} while (C_timer_check(start, 1000) == 0);
	}
}
static void C_lcd_cmd_data(int cmd, int data)
{
	C_lcd_wait_write();
	outl(cmd | 0x80000000, 0x70008A0C);

	C_lcd_wait_write();
	outl(data | 0x80000000, 0x70008A0C);
}

static void C_update_display(int sx, int sy, int mx, int my)
{
	int height = (my - sy) + 1;
	int width = (mx - sx) + 1;

	char *addr = SDL_VideoSurface->pixels;

	if (width & 1) width++;

	/* start X and Y */
	C_lcd_cmd_data(0x12, (sy & 0xff));
	C_lcd_cmd_data(0x13, (((SDL_VideoSurface->w - 1) - sx) & 0xff));

	/* max X and Y */
	C_lcd_cmd_data(0x15, (((sy + height) - 1) & 0xff));
	C_lcd_cmd_data(0x16, (((((SDL_VideoSurface->w - 1) - sx) - width) + 1) & 0xff));

	addr += sx + sy * SDL_VideoSurface->pitch;

	while (height > 0) {
		int h, x, y, pixels_to_write;

		pixels_to_write = (width * height) * 2;

		/* calculate how much we can do in one go */
		h = height;
		if (pixels_to_write > 64000) {
			h = (64000/2) / width;
			pixels_to_write = (width * h) * 2;
		}

		outl(0x10000080, 0x70008A20);
		outl((pixels_to_write - 1) | 0xC0010000, 0x70008A24);
		outl(0x34000000, 0x70008A20);

		/* for each row */
		for (x = 0; x < h; x++)
		{
			/* for each column */
			for (y = 0; y < width; y += 2) {
				unsigned two_pixels;

				two_pixels = addr[0] | (addr[1] << 16);
				addr += 2;

				while ((inl(0x70008A20) & 0x1000000) == 0);

				/* output 2 pixels */
				outl(two_pixels, 0x70008B00);
			}

			addr += SDL_VideoSurface->w - width;
		}

		while ((inl(0x70008A20) & 0x4000000) == 0);

		outl(0x0, 0x70008A24);

		height = height - h;
	}
}

// Should work with photo. However, I don't have one, so I'm not sure.
static void iPod_UpdateRects (_THIS, int nrects, SDL_Rect *rects) 
{
    if (SDL_VideoSurface->format->BitsPerPixel == 16) {
	C_update_display (0, 0, lcd_width, lcd_height);
    } else {
	int i, y, x;
	for (i = 0; i < nrects; i++) {
	    SDL_Rect *r = rects + i;
	    if (!r) {
		continue;
	    }
	    
	    for (y = r->y; (y < r->y + r->h) && y < lcd_height; y++) {
		for (x = r->x; (x < r->x + r->w) && x < lcd_width; x++) {
		    ipod_scr[y*(lcd_width/4) + x/4] &= ~(3 << (2 * (x%4)));
		    ipod_scr[y*(lcd_width/4) + x/4] |=
			(((Uint8*)(SDL_VideoSurface->pixels))[ y*SDL_VideoSurface->pitch + x ] & 3) << (2 * (x%4));
		}
	    }
	}
	
	M_update_display (0, 0, lcd_width, lcd_height);
    }
}
