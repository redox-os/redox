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

/*
	GEM video driver

	Patrice Mandin
	and work from
	Olivier Landemarre, Johan Klockars, Xavier Joubert, Claude Attard
*/

/* Mint includes */
#include <gem.h>
#include <gemx.h>
#include <mint/osbind.h>
#include <mint/cookie.h>

#include "SDL_endian.h"
#include "SDL_video.h"
#include "SDL_mouse.h"
#include "../SDL_sysvideo.h"
#include "../SDL_pixels_c.h"
#include "../../events/SDL_events_c.h"
#include "../SDL_cursor_c.h"

#include "../ataricommon/SDL_ataric2p_s.h"
#include "../ataricommon/SDL_atarieddi_s.h"
#include "../ataricommon/SDL_atarimxalloc_c.h"
#include "../ataricommon/SDL_atarigl_c.h"

#include "SDL_gemvideo.h"
#include "SDL_gemevents_c.h"
#include "SDL_gemmouse_c.h"
#include "SDL_gemwm_c.h"
#include "../ataricommon/SDL_xbiosevents_c.h"
#include "../ataricommon/SDL_ataridevmouse_c.h"

/* Defines */

/*#define DEBUG_VIDEO_GEM	1*/

#define GEM_VID_DRIVER_NAME "gem"

#undef MIN
#define MIN(a,b) (((a)<(b)) ? (a) : (b))
#undef MAX
#define MAX(a,b) (((a)>(b)) ? (a) : (b))

/* Variables */

static unsigned char vdi_index[256] = {
	0,  2,  3,  6,  4,  7,  5,   8,
	9, 10, 11, 14, 12, 15, 13, 255
};

static const char empty_name[]="";

/* Initialization/Query functions */
static int GEM_VideoInit(_THIS, SDL_PixelFormat *vformat);
static SDL_Rect **GEM_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags);
static SDL_Surface *GEM_SetVideoMode(_THIS, SDL_Surface *current, int width, int height, int bpp, Uint32 flags);
static int GEM_SetColors(_THIS, int firstcolor, int ncolors, SDL_Color *colors);
static void GEM_VideoQuit(_THIS);

/* Hardware surface functions */
static int GEM_AllocHWSurface(_THIS, SDL_Surface *surface);
static int GEM_LockHWSurface(_THIS, SDL_Surface *surface);
static int GEM_FlipHWSurface(_THIS, SDL_Surface *surface);
static void GEM_UnlockHWSurface(_THIS, SDL_Surface *surface);
static void GEM_FreeHWSurface(_THIS, SDL_Surface *surface);
static void GEM_UpdateRects(_THIS, int numrects, SDL_Rect *rects);
#if 0
static int GEM_ToggleFullScreen(_THIS, int on);
#endif

/* Internal functions */
static void GEM_FreeBuffers(_THIS);
static void GEM_ClearScreen(_THIS);
static void GEM_ClearRect(_THIS, short *rect);
static void GEM_SetNewPalette(_THIS, Uint16 newpal[256][3]);
static void GEM_LockScreen(_THIS);
static void GEM_UnlockScreen(_THIS);
static void refresh_window(_THIS, int winhandle, short *rect);

#if SDL_VIDEO_OPENGL
/* OpenGL functions */
static void GEM_GL_SwapBuffers(_THIS);
#endif

/* GEM driver bootstrap functions */

static int GEM_Available(void)
{
	/* Test if AES available */
	if (appl_init() == -1)
		return 0;

	appl_exit();
	return 1;
}

static void GEM_DeleteDevice(SDL_VideoDevice *device)
{
	SDL_free(device->hidden);
	SDL_free(device);
}

static SDL_VideoDevice *GEM_CreateDevice(int devindex)
{
	SDL_VideoDevice *device;
	int vectors_mask;
/*	unsigned long dummy;*/

	/* Initialize all variables that we clean on shutdown */
	device = (SDL_VideoDevice *)SDL_malloc(sizeof(SDL_VideoDevice));
	if ( device ) {
		SDL_memset(device, 0, (sizeof *device));
		device->hidden = (struct SDL_PrivateVideoData *)
				SDL_malloc((sizeof *device->hidden));
		device->gl_data = (struct SDL_PrivateGLData *)
				SDL_malloc((sizeof *device->gl_data));
	}
	if ( (device == NULL) || (device->hidden == NULL) ) {
		SDL_OutOfMemory();
		if ( device ) {
			SDL_free(device);
		}
		return(0);
	}
	SDL_memset(device->hidden, 0, (sizeof *device->hidden));
	SDL_memset(device->gl_data, 0, sizeof(*device->gl_data));

	/* Set the function pointers */
	device->VideoInit = GEM_VideoInit;
	device->ListModes = GEM_ListModes;
	device->SetVideoMode = GEM_SetVideoMode;
	device->SetColors = GEM_SetColors;
	device->UpdateRects = NULL /*GEM_UpdateRects*/;
	device->VideoQuit = GEM_VideoQuit;
	device->AllocHWSurface = GEM_AllocHWSurface;
	device->LockHWSurface = GEM_LockHWSurface;
	device->UnlockHWSurface = GEM_UnlockHWSurface;
	device->FlipHWSurface = GEM_FlipHWSurface;
	device->FreeHWSurface = GEM_FreeHWSurface;
	device->ToggleFullScreen = NULL /*GEM_ToggleFullScreen*/;

	/* Window manager */
	device->SetCaption = GEM_SetCaption;
	device->SetIcon = GEM_SetIcon;
	device->IconifyWindow = GEM_IconifyWindow;
	device->GrabInput = GEM_GrabInput;

	/* Events */
	device->InitOSKeymap = GEM_InitOSKeymap;
	device->PumpEvents = GEM_PumpEvents;

	/* Mouse */
	device->FreeWMCursor = GEM_FreeWMCursor;
	device->CreateWMCursor = GEM_CreateWMCursor;
	device->ShowWMCursor = GEM_ShowWMCursor;
	device->WarpWMCursor = NULL /*GEM_WarpWMCursor*/;
	device->CheckMouseMode = GEM_CheckMouseMode;

#if SDL_VIDEO_OPENGL
	/* OpenGL functions */
	device->GL_LoadLibrary = SDL_AtariGL_LoadLibrary;
	device->GL_GetProcAddress = SDL_AtariGL_GetProcAddress;
	device->GL_GetAttribute = SDL_AtariGL_GetAttribute;
	device->GL_MakeCurrent = SDL_AtariGL_MakeCurrent;
	device->GL_SwapBuffers = GEM_GL_SwapBuffers;
#endif

	device->hidden->use_dev_mouse =
		(SDL_AtariDevMouse_Open()!=0) ? SDL_TRUE : SDL_FALSE;

	vectors_mask = ATARI_XBIOS_JOYSTICKEVENTS;	/* XBIOS joystick events */
	if (!(device->hidden->use_dev_mouse)) {
		vectors_mask |= ATARI_XBIOS_MOUSEEVENTS;	/* XBIOS mouse events */
	}
/*	if (Getcookie(C_MiNT, &dummy)==C_FOUND) {
		vectors_mask = 0;
	}*/

	SDL_AtariXbios_InstallVectors(vectors_mask);

	device->free = GEM_DeleteDevice;

	return device;
}

VideoBootStrap GEM_bootstrap = {
	GEM_VID_DRIVER_NAME, "Atari GEM video driver",
	GEM_Available, GEM_CreateDevice
};

static void VDI_ReadExtInfo(_THIS, short *work_out)
{
	unsigned long EdDI_version;
	long cookie_EdDI;
	Uint16 clut_type;

	/* Read EdDI informations */
	if  (Getcookie(C_EdDI, &cookie_EdDI) == C_NOTFOUND) {
		return;
	}
	
	EdDI_version = Atari_get_EdDI_version( (void *)cookie_EdDI);

	vq_scrninfo(VDI_handle, work_out);

	VDI_format = work_out[0];
	clut_type = work_out[1];

	/* With EdDI>=1.1, we can have screen pitch, address and format
	 * so we can directly write to screen without using vro_cpyfm
	 */
	if (EdDI_version >= EDDI_11) {
		VDI_pitch = work_out[5];
		VDI_screen = (void *) *((unsigned long *) &work_out[6]);
	}

	switch(clut_type) {
		case VDI_CLUT_HARDWARE:
			{
				int i;
				Uint16 *tmp_p;

				tmp_p = (Uint16 *)&work_out[16];

				for (i=0;i<256;i++) {
					vdi_index[*tmp_p++] = i;
				}
			}
			break;
		case VDI_CLUT_SOFTWARE:
			{
				int component; /* red, green, blue, alpha, overlay */
				int num_bit;
				unsigned short *tmp_p;

				/* We can build masks with info here */
				tmp_p = (unsigned short *) &work_out[16];
				for (component=0;component<5;component++) {
					for (num_bit=0;num_bit<16;num_bit++) {
						unsigned short valeur;

						valeur = *tmp_p++;

						if (valeur == 0xffff) {
							continue;
						}

						switch(component) {
							case 0:
								VDI_redmask |= 1<< valeur;
								break;
							case 1:
								VDI_greenmask |= 1<< valeur;
								break;
							case 2:
								VDI_bluemask |= 1<< valeur;
								break;
							case 3:
								VDI_alphamask |= 1<< valeur;
								break;
						}
					}
				}
			}

			/* Remove lower green bits for Intel endian screen */
			if ((VDI_greenmask == ((7<<13)|3)) || (VDI_greenmask == ((7<<13)|7))) {
				VDI_greenmask &= ~(7<<13);
			}
			break;
		case VDI_CLUT_NONE:
			break;
	}
}

int GEM_VideoInit(_THIS, SDL_PixelFormat *vformat)
{
	int i, menubar_size;
	short work_in[12], work_out[272], dummy;

	/* Open AES (Application Environment Services) */
	if (appl_init() == -1) {
		fprintf(stderr,"Can not open AES\n");
		return 1;
	}

	/* Read version and features */
	GEM_version = aes_global[0];
	if (GEM_version >= 0x0410) {
		short ap_gout[4], errorcode;
		
		GEM_wfeatures=0;
		errorcode=appl_getinfo(AES_WINDOW, &ap_gout[0], &ap_gout[1], &ap_gout[2], &ap_gout[3]);

		if (errorcode==0) {
			GEM_wfeatures=ap_gout[0];			
		}
	}	

	/* Ask VDI physical workstation handle opened by AES */
	VDI_handle = graf_handle(&dummy, &dummy, &dummy, &dummy);
	if (VDI_handle<1) {
		fprintf(stderr,"Wrong VDI handle %d returned by AES\n",VDI_handle);
		return 1;
	}

	/* Open virtual VDI workstation */
	work_in[0]=Getrez()+2;
	for(i = 1; i < 10; i++)
		work_in[i] = 1;
	work_in[10] = 2;

	v_opnvwk(work_in, &VDI_handle, work_out);
	if (VDI_handle == 0) {
		fprintf(stderr,"Can not open VDI virtual workstation\n");
		return 1;
	}

	/* Read fullscreen size */
	VDI_w = work_out[0] + 1;
	VDI_h = work_out[1] + 1;

	/* Read desktop size and position */
	if (!wind_get(DESKTOP_HANDLE, WF_WORKXYWH, &GEM_desk_x, &GEM_desk_y, &GEM_desk_w, &GEM_desk_h)) {
		fprintf(stderr,"Can not read desktop properties\n");
		return 1;
	}

	/* Read bit depth */
	vq_extnd(VDI_handle, 1, work_out);
	VDI_bpp = work_out[4];
	VDI_oldnumcolors=0;

	switch(VDI_bpp) {
		case 8:
			VDI_pixelsize=1;
			break;
		case 15:
		case 16:
			VDI_pixelsize=2;
			break;
		case 24:
			VDI_pixelsize=3;
			break;
		case 32:
			VDI_pixelsize=4;
			break;
		default:
			fprintf(stderr,"%d bits colour depth not supported\n",VDI_bpp);
			return 1;
	}

	/* Setup hardware -> VDI palette mapping */
	for(i = 16; i < 255; i++) {
		vdi_index[i] = i;
	}
	vdi_index[255] = 1;

	/* Save current palette */
	if (VDI_bpp>8) {
		VDI_oldnumcolors=1<<8;
	} else {
		VDI_oldnumcolors=1<<VDI_bpp;
	}
	
	for(i = 0; i < VDI_oldnumcolors; i++) {
		short rgb[3];

		vq_color(VDI_handle, i, 0, rgb);

		VDI_oldpalette[i][0] = rgb[0];
		VDI_oldpalette[i][1] = rgb[1];
		VDI_oldpalette[i][2] = rgb[2];
	}
	VDI_setpalette = GEM_SetNewPalette;
	SDL_memcpy(VDI_curpalette,VDI_oldpalette,sizeof(VDI_curpalette));

	/* Setup screen info */
	GEM_title_name = empty_name;
	GEM_icon_name = empty_name;

	GEM_handle = -1;
	GEM_locked = SDL_FALSE;
	GEM_win_fulled = SDL_FALSE;
	GEM_fullscreen = SDL_FALSE;
	GEM_lock_redraw = SDL_TRUE;	/* Prevent redraw till buffers are setup */

	VDI_screen = NULL;
	VDI_pitch = VDI_w * VDI_pixelsize;
	VDI_format = ( (VDI_bpp <= 8) ? VDI_FORMAT_INTER : VDI_FORMAT_PACK);
	VDI_redmask = VDI_greenmask = VDI_bluemask = VDI_alphamask = 0;
	VDI_ReadExtInfo(this, work_out);

#ifdef DEBUG_VIDEO_GEM
	printf("sdl:video:gem: screen: address=0x%08x, pitch=%d\n", VDI_screen, VDI_pitch);
	printf("sdl:video:gem: format=%d\n", VDI_format);
	printf("sdl:video:gem: masks: 0x%08x, 0x%08x, 0x%08x, 0x%08x\n",
		VDI_alphamask, VDI_redmask, VDI_greenmask, VDI_bluemask
	);
#endif

	/* Setup destination mfdb */
	VDI_dst_mfdb.fd_addr = NULL;

	/* Determine the current screen size */
	this->info.current_w = VDI_w;
	this->info.current_h = VDI_h;

	/* Determine the screen depth */
	/* we change this during the SDL_SetVideoMode implementation... */
	vformat->BitsPerPixel = VDI_bpp;

	/* Set mouse cursor to arrow */
	graf_mouse(ARROW, NULL);
	GEM_cursor = NULL;

	/* Init chunky to planar routine */
	SDL_Atari_C2pConvert = SDL_Atari_C2pConvert8;

	/* Setup VDI fill functions */
	vsf_color(VDI_handle,0);
	vsf_interior(VDI_handle,1);
	vsf_perimeter(VDI_handle,0);

	/* Menu bar save buffer */
	menubar_size = GEM_desk_w * GEM_desk_y * VDI_pixelsize;
	GEM_menubar=Atari_SysMalloc(menubar_size,MX_PREFTTRAM);

	/* Fill video modes list */
	SDL_modelist[0] = SDL_malloc(sizeof(SDL_Rect));
	SDL_modelist[0]->x = 0;
	SDL_modelist[0]->y = 0;
	SDL_modelist[0]->w = VDI_w;
	SDL_modelist[0]->h = VDI_h;

	SDL_modelist[1] = NULL;

#if SDL_VIDEO_OPENGL
	SDL_AtariGL_InitPointers(this);
#endif

	this->info.wm_available = 1;

	/* We're done! */
	return(0);
}

SDL_Rect **GEM_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags)
{
	if (format->BitsPerPixel != VDI_bpp) {
		return ((SDL_Rect **)NULL);
	}

	if (flags & SDL_FULLSCREEN) {
		return (SDL_modelist);
	}

	return((SDL_Rect **)-1);
}

static void GEM_FreeBuffers(_THIS)
{
	/* Release buffer */
	if ( GEM_buffer2 ) {
		Mfree( GEM_buffer2 );
		GEM_buffer2=NULL;
	}

	if ( GEM_buffer1 ) {
		Mfree( GEM_buffer1 );
		GEM_buffer1=NULL;
	}
}

static void GEM_ClearRect(_THIS, short *rect)
{
	short oldrgb[3], rgb[3]={0,0,0};

	vq_color(VDI_handle, vdi_index[0], 0, oldrgb);
	vs_color(VDI_handle, vdi_index[0], rgb);

	vsf_color(VDI_handle,0);
	vsf_interior(VDI_handle,1);
	vsf_perimeter(VDI_handle,0);
	v_bar(VDI_handle, rect);

	vs_color(VDI_handle, vdi_index[0], oldrgb);
}

static void GEM_ClearScreen(_THIS)
{
	short pxy[4];

	v_hide_c(VDI_handle);

	pxy[0] = pxy[1] = 0;
	pxy[2] = VDI_w - 1;
	pxy[3] = VDI_h - 1;
	GEM_ClearRect(this, pxy);

	v_show_c(VDI_handle, 1);
}

static void GEM_SetNewPalette(_THIS, Uint16 newpal[256][3])
{
	int i;
	short rgb[3];

	if (VDI_oldnumcolors==0)
		return;

	for(i = 0; i < VDI_oldnumcolors; i++) {
		rgb[0] = newpal[i][0];
		rgb[1] = newpal[i][1];
		rgb[2] = newpal[i][2];

		vs_color(VDI_handle, i, rgb);
	}
}

static void GEM_LockScreen(_THIS)
{
	if (!GEM_locked) {
		/* Lock AES */
		wind_update(BEG_UPDATE);
		wind_update(BEG_MCTRL);
		/* Reserve memory space, used to be sure of compatibility */
		form_dial( FMD_START, 0,0,0,0, 0,0,VDI_w,VDI_h);

		/* Save menu bar */
		if (GEM_menubar) {
			MFDB mfdb_src;
			short blitcoords[8];

			mfdb_src.fd_addr=GEM_menubar;
			mfdb_src.fd_w=GEM_desk_w;
			mfdb_src.fd_h=GEM_desk_y;
			mfdb_src.fd_wdwidth=GEM_desk_w>>4;
			mfdb_src.fd_nplanes=VDI_bpp;
			mfdb_src.fd_stand=
				mfdb_src.fd_r1=
				mfdb_src.fd_r2=
				mfdb_src.fd_r3= 0;

			blitcoords[0] = blitcoords[4] = 0;
			blitcoords[1] = blitcoords[5] = 0;
			blitcoords[2] = blitcoords[6] = GEM_desk_w-1;
			blitcoords[3] = blitcoords[7] = GEM_desk_y-1;

			vro_cpyfm(VDI_handle, S_ONLY, blitcoords, &VDI_dst_mfdb, &mfdb_src);
		}

		GEM_locked=SDL_TRUE;
	}
}

static void GEM_UnlockScreen(_THIS)
{
	if (GEM_locked) {
		/* Restore menu bar */
		if (GEM_menubar) {
			MFDB mfdb_src;
			short blitcoords[8];

			mfdb_src.fd_addr=GEM_menubar;
			mfdb_src.fd_w=GEM_desk_w;
			mfdb_src.fd_h=GEM_desk_y;
			mfdb_src.fd_wdwidth=GEM_desk_w>>4;
			mfdb_src.fd_nplanes=VDI_bpp;
			mfdb_src.fd_stand=
				mfdb_src.fd_r1=
				mfdb_src.fd_r2=
				mfdb_src.fd_r3= 0;

			blitcoords[0] = blitcoords[4] = 0;
			blitcoords[1] = blitcoords[5] = 0;
			blitcoords[2] = blitcoords[6] = GEM_desk_w-1;
			blitcoords[3] = blitcoords[7] = GEM_desk_y-1;

			vro_cpyfm(VDI_handle, S_ONLY, blitcoords, &mfdb_src, &VDI_dst_mfdb);
		}

		/* Restore screen memory, and send REDRAW to all apps */
		form_dial( FMD_FINISH, 0,0,0,0, 0,0,VDI_w,VDI_h);
		/* Unlock AES */
		wind_update(END_MCTRL);
		wind_update(END_UPDATE);

		GEM_locked=SDL_FALSE;
	}
}

SDL_Surface *GEM_SetVideoMode(_THIS, SDL_Surface *current,
				int width, int height, int bpp, Uint32 flags)
{
	Uint32 modeflags, screensize;
	SDL_bool use_shadow1, use_shadow2;

	/* width must be multiple of 16, for vro_cpyfm() and c2p_convert() */
	if ((width & 15) != 0) {
		width = (width | 15) +1;
	}

	/*--- Verify if asked mode can be used ---*/
	if (VDI_bpp != bpp) {
		SDL_SetError("%d bpp mode not supported", bpp);
		return(NULL);
	}

	if (flags & SDL_FULLSCREEN) {
		if ((VDI_w < width) || (VDI_h < height)) {
			SDL_SetError("%dx%d mode is too large", width, height);
			return(NULL);
		}
	}

	/*--- Allocate the new pixel format for the screen ---*/
	if ( ! SDL_ReallocFormat(current, VDI_bpp, VDI_redmask, VDI_greenmask, VDI_bluemask, VDI_alphamask) ) {
		SDL_SetError("Couldn't allocate new pixel format for requested mode");
		return(NULL);
	}

	screensize = width * height * VDI_pixelsize;

#ifdef DEBUG_VIDEO_GEM
	printf("sdl:video:gem: setvideomode(): %dx%dx%d = %d\n", width, height, bpp, screensize);
#endif

	/*--- Allocate shadow buffers if needed, and conversion operations ---*/
	GEM_FreeBuffers(this);

	GEM_bufops=0;
	use_shadow1=use_shadow2=SDL_FALSE;
	if (VDI_screen && (flags & SDL_FULLSCREEN)) {
		if (VDI_format==VDI_FORMAT_INTER) {
			use_shadow1=SDL_TRUE;
			GEM_bufops = B2S_C2P_1TOS;
		}
	} else {
		use_shadow1=SDL_TRUE;
		if (VDI_format==VDI_FORMAT_PACK) {
			GEM_bufops = B2S_VROCPYFM_1TOS;
		} else {
			use_shadow2=SDL_TRUE;
			GEM_bufops = B2S_C2P_1TO2|B2S_VROCPYFM_2TOS;
		}
	}

	if (use_shadow1) {
		GEM_buffer1 = Atari_SysMalloc(screensize, MX_PREFTTRAM);
		if (GEM_buffer1==NULL) {
			SDL_SetError("Can not allocate %d KB for frame buffer", screensize>>10);
			return NULL;
		}
		SDL_memset(GEM_buffer1, 0, screensize);
#ifdef DEBUG_VIDEO_GEM
		printf("sdl:video:gem: setvideomode(): allocated buffer 1\n");
#endif
	}

	if (use_shadow2) {
		GEM_buffer2 = Atari_SysMalloc(screensize, MX_PREFTTRAM);
		if (GEM_buffer2==NULL) {
			SDL_SetError("Can not allocate %d KB for shadow buffer", screensize>>10);
			return NULL;
		}
		SDL_memset(GEM_buffer2, 0, screensize);
#ifdef DEBUG_VIDEO_GEM
		printf("sdl:video:gem: setvideomode(): allocated buffer 2\n");
#endif
	}

	/*--- Initialize screen ---*/
	modeflags = SDL_PREALLOC;
	if (VDI_bpp == 8) {
		modeflags |= SDL_HWPALETTE;
	}

	if (flags & SDL_FULLSCREEN) {
		GEM_LockScreen(this);

		GEM_ClearScreen(this);

		modeflags |= SDL_FULLSCREEN;
		if (VDI_screen && (VDI_format==VDI_FORMAT_PACK) && !use_shadow1) {
			modeflags |= SDL_HWSURFACE;
		} else {
			modeflags |= SDL_SWSURFACE;
		}

		GEM_fullscreen = SDL_TRUE;
	} else {
		int old_win_type;
		short x2,y2,w2,h2;

		GEM_UnlockScreen(this);

		/* Set window gadgets */
		old_win_type = GEM_win_type;
		if (!(flags & SDL_NOFRAME)) {
			GEM_win_type=NAME|MOVER|CLOSER|SMALLER;
			if (flags & SDL_RESIZABLE) {
				GEM_win_type |= FULLER|SIZER;
				modeflags |= SDL_RESIZABLE;
			}
		} else {
			GEM_win_type=0;
			modeflags |= SDL_NOFRAME;
		}
		modeflags |= SDL_SWSURFACE;

		/* Recreate window ? only for different widget or non-created window */
		if ((old_win_type != GEM_win_type) || (GEM_handle < 0)) {
			/* Calculate window size */
			if (!wind_calc(WC_BORDER, GEM_win_type, 0,0,width,height, &x2,&y2,&w2,&h2)) {
				GEM_FreeBuffers(this);
				SDL_SetError("Can not calculate window attributes");
				return NULL;
			}

			/* Center window */
			x2 = (GEM_desk_w-w2)>>1;
			y2 = (GEM_desk_h-h2)>>1;
			if (x2<0) {
				x2 = 0;
			}
			if (y2<0) {
				y2 = 0;
			}
			x2 += GEM_desk_x;
			y2 += GEM_desk_y;

			/* Destroy existing window */
			if (GEM_handle >= 0) {
				wind_close(GEM_handle);
				wind_delete(GEM_handle);
			}

			/* Create window */
			GEM_handle=wind_create(GEM_win_type, x2,y2,w2,h2);
			if (GEM_handle<0) {
				GEM_FreeBuffers(this);
				SDL_SetError("Can not create window");
				return NULL;
			}

#ifdef DEBUG_VIDEO_GEM
			printf("sdl:video:gem: handle=%d\n", GEM_handle);
#endif

			/* Setup window name */
			wind_set(GEM_handle,WF_NAME,(short)(((unsigned long)GEM_title_name)>>16),(short)(((unsigned long)GEM_title_name) & 0xffff),0,0);
			GEM_refresh_name = SDL_FALSE;

			/* Open the window */
			wind_open(GEM_handle,x2,y2,w2,h2);
		} else {
			/* Resize window to fit asked video mode */
			wind_get (GEM_handle, WF_WORKXYWH, &x2,&y2,&w2,&h2);
			if (wind_calc(WC_BORDER, GEM_win_type, x2,y2,width,height, &x2,&y2,&w2,&h2)) {
				wind_set (GEM_handle, WF_CURRXYWH, x2,y2,w2,h2);
			}
		}

		GEM_fullscreen = SDL_FALSE;
	}

	/* Set up the new mode framebuffer */
	current->w = width;
	current->h = height;
	if (use_shadow1) {
		current->pixels = GEM_buffer1;
		current->pitch = width * VDI_pixelsize;
	} else {
		current->pixels = VDI_screen;
		current->pitch = VDI_pitch;
	}

#if SDL_VIDEO_OPENGL
	if (flags & SDL_OPENGL) {
		if (!SDL_AtariGL_Init(this, current)) {
			GEM_FreeBuffers(this);
			SDL_SetError("Can not create OpenGL context");
			return NULL;
		}

		modeflags |= SDL_OPENGL;
	}
#endif

	current->flags = modeflags;

#ifdef DEBUG_VIDEO_GEM
	printf("sdl:video:gem: surface: %dx%d\n", current->w, current->h);
#endif

	this->UpdateRects = GEM_UpdateRects;
	GEM_lock_redraw = SDL_FALSE;	/* Enable redraw */

	/* We're done */
	return(current);
}

static int GEM_AllocHWSurface(_THIS, SDL_Surface *surface)
{
	return -1;
}

static void GEM_FreeHWSurface(_THIS, SDL_Surface *surface)
{
	return;
}

static int GEM_LockHWSurface(_THIS, SDL_Surface *surface)
{
	return(0);
}

static void GEM_UnlockHWSurface(_THIS, SDL_Surface *surface)
{
	return;
}

static void GEM_UpdateRectsFullscreen(_THIS, int numrects, SDL_Rect *rects)
{
	SDL_Surface *surface;
	int i, surf_width;

	surface = this->screen;
	/* Need to be a multiple of 16 pixels */
	surf_width=surface->w;
	if ((surf_width & 15) != 0) {
		surf_width = (surf_width | 15) + 1;
	}

	if (GEM_bufops & (B2S_C2P_1TO2|B2S_C2P_1TOS)) {
		void *destscr;
		int destpitch;

		if (GEM_bufops & B2S_C2P_1TOS) {
			destscr = VDI_screen;
			destpitch = VDI_pitch;
		} else {
			destscr = GEM_buffer2;
			destpitch = surface->pitch;
		}

		for (i=0;i<numrects;i++) {
			void *source,*destination;
			int x1,x2;

			x1 = rects[i].x & ~15;
			x2 = rects[i].x+rects[i].w;
			if (x2 & 15) {
				x2 = (x2 | 15) +1;
			}

			source = surface->pixels;
			source += surface->pitch * rects[i].y;
			source += x1;

			destination = destscr;
			destination += destpitch * rects[i].y;
			destination += x1;

			SDL_Atari_C2pConvert(
				source, destination,
				x2-x1, rects[i].h,
				SDL_FALSE,
				surface->pitch, destpitch
			);
		}
	}

	if (GEM_bufops & (B2S_VROCPYFM_1TOS|B2S_VROCPYFM_2TOS)) {
		MFDB mfdb_src;
		short blitcoords[8];

		mfdb_src.fd_addr=surface->pixels;
		mfdb_src.fd_w=surf_width;
		mfdb_src.fd_h=surface->h;
		mfdb_src.fd_wdwidth= (surface->pitch/VDI_pixelsize) >> 4;
		mfdb_src.fd_nplanes=surface->format->BitsPerPixel;
		mfdb_src.fd_stand=
			mfdb_src.fd_r1=
			mfdb_src.fd_r2=
			mfdb_src.fd_r3= 0;
		if (GEM_bufops & B2S_VROCPYFM_2TOS) {
			mfdb_src.fd_addr=GEM_buffer2;
		}

		for ( i=0; i<numrects; ++i ) {
			blitcoords[0] = blitcoords[4] = rects[i].x;
			blitcoords[1] = blitcoords[5] = rects[i].y;
			blitcoords[2] = blitcoords[6] = rects[i].x + rects[i].w - 1;
			blitcoords[3] = blitcoords[7] = rects[i].y + rects[i].h - 1;

			vro_cpyfm(VDI_handle, S_ONLY, blitcoords, &mfdb_src, &VDI_dst_mfdb);
		}
	}
}

static void GEM_UpdateRectsWindowed(_THIS, int numrects, SDL_Rect *rects)
{
	short pxy[4], wind_pxy[4];
	int i;

	if (wind_get(GEM_handle, WF_WORKXYWH, &wind_pxy[0], &wind_pxy[1], &wind_pxy[2], &wind_pxy[3])==0) {
		return;
	}

	for ( i=0; i<numrects; ++i ) {
		pxy[0] = wind_pxy[0] + rects[i].x;
		pxy[1] = wind_pxy[1] + rects[i].y;
		pxy[2] = rects[i].w;
		pxy[3] = rects[i].h;

		GEM_wind_redraw(this, GEM_handle, pxy);
	}
}

static void GEM_UpdateRects(_THIS, int numrects, SDL_Rect *rects)
{
	SDL_Surface *surface;

	if (GEM_lock_redraw) {
		return;
	}

	surface = this->screen;

	if (surface->flags & SDL_FULLSCREEN) {
		GEM_UpdateRectsFullscreen(this, numrects, rects);
	} else {
		GEM_UpdateRectsWindowed(this, numrects, rects);
	}
}

static int GEM_FlipHWSurfaceFullscreen(_THIS, SDL_Surface *surface)
{
	int surf_width;

	/* Need to be a multiple of 16 pixels */
	surf_width=surface->w;
	if ((surf_width & 15) != 0) {
		surf_width = (surf_width | 15) + 1;
	}

	if (GEM_bufops & (B2S_C2P_1TO2|B2S_C2P_1TOS)) {
		void *destscr;
		int destpitch;

		if (GEM_bufops & B2S_C2P_1TOS) {
			destscr = VDI_screen;
			destpitch = VDI_pitch;
		} else {
			destscr = GEM_buffer2;
			destpitch = surface->pitch;
		}

		SDL_Atari_C2pConvert(
			surface->pixels, destscr,
			surf_width, surface->h,
			SDL_FALSE,
			surface->pitch, destpitch
		);
	}

	if (GEM_bufops & (B2S_VROCPYFM_1TOS|B2S_VROCPYFM_2TOS)) {
		MFDB mfdb_src;
		short blitcoords[8];

		mfdb_src.fd_w=surf_width;
		mfdb_src.fd_h=surface->h;
		mfdb_src.fd_wdwidth=mfdb_src.fd_w >> 4;
		mfdb_src.fd_nplanes=surface->format->BitsPerPixel;
		mfdb_src.fd_stand=
			mfdb_src.fd_r1=
			mfdb_src.fd_r2=
			mfdb_src.fd_r3= 0;
		if (GEM_bufops & B2S_VROCPYFM_1TOS) {
			mfdb_src.fd_addr=surface->pixels;
		} else {
			mfdb_src.fd_addr=GEM_buffer2;
		}

		blitcoords[0] = blitcoords[4] = 0;
		blitcoords[1] = blitcoords[5] = 0;
		blitcoords[2] = blitcoords[6] = surface->w - 1;
		blitcoords[3] = blitcoords[7] = surface->h - 1;

		vro_cpyfm(VDI_handle, S_ONLY, blitcoords, &mfdb_src, &VDI_dst_mfdb);
	}

	return(0);
}

static int GEM_FlipHWSurfaceWindowed(_THIS, SDL_Surface *surface)
{
	short	pxy[8];

	/* Update the whole window */
	wind_get(GEM_handle, WF_WORKXYWH, &pxy[0], &pxy[1], &pxy[2], &pxy[3]);

	GEM_wind_redraw(this, GEM_handle, pxy);

	return(0);
}

static int GEM_FlipHWSurface(_THIS, SDL_Surface *surface)
{
	if (GEM_lock_redraw) {
		return(0);
	}

	if (surface->flags & SDL_FULLSCREEN) {
		return GEM_FlipHWSurfaceFullscreen(this, surface);
	} else {
		return GEM_FlipHWSurfaceWindowed(this, surface);
	}
}

static int GEM_SetColors(_THIS, int firstcolor, int ncolors, SDL_Color *colors)
{
	int i;
	SDL_Surface *surface;

#ifdef DEBUG_VIDEO_GEM
	printf("sdl:video:gem: setcolors()\n");
#endif

	/* Do not change palette in True Colour */
	surface = this->screen;
	if (surface->format->BitsPerPixel > 8) {
		return 1;
	}

	for(i = 0; i < ncolors; i++)
	{
		int		r, g, b;
		short	rgb[3];

		r = colors[i].r;
		g = colors[i].g;
		b = colors[i].b;

		rgb[0] = VDI_curpalette[i][0] = (1000 * r) / 255;
		rgb[1] = VDI_curpalette[i][1] =(1000 * g) / 255;
		rgb[2] = VDI_curpalette[i][2] =(1000 * b) / 255;

		vs_color(VDI_handle, vdi_index[firstcolor+i], rgb);
	}

	return(1);
}

#if 0
static int GEM_ToggleFullScreen(_THIS, int on)
{
	if (on) {
		GEM_LockScreen(this);
	} else {
		GEM_UnlockScreen(this);
	}

	return(1);
}
#endif

/* Note:  If we are terminated, this could be called in the middle of
   another SDL video routine -- notably UpdateRects.
*/
void GEM_VideoQuit(_THIS)
{
	SDL_AtariXbios_RestoreVectors();
	if (GEM_usedevmouse) {
		SDL_AtariDevMouse_Close();
	}

	GEM_FreeBuffers(this);

#if SDL_VIDEO_OPENGL
	if (gl_active) {
		SDL_AtariGL_Quit(this, SDL_TRUE);
	}
#endif

	/* Destroy window */
	if (GEM_handle>=0) {
		wind_close(GEM_handle);
		wind_delete(GEM_handle);
		GEM_handle=-1;
	}

	GEM_UnlockScreen(this);
	if (GEM_menubar) {
		Mfree(GEM_menubar);
		GEM_menubar=NULL;
	}

	appl_exit();

	GEM_SetNewPalette(this, VDI_oldpalette);

	/* Close VDI workstation */
	if (VDI_handle) {
		v_clsvwk(VDI_handle);
	}

	/* Free mode list */
	if (SDL_modelist[0]) {
		SDL_free(SDL_modelist[0]);
		SDL_modelist[0]=NULL;
	}

	this->screen->pixels = NULL;	
}

void GEM_wind_redraw(_THIS, int winhandle, short *inside)
{
	short todo[4];

	/* Tell AES we are going to update */
	wind_update(BEG_UPDATE);

	v_hide_c(VDI_handle);

	/* Browse the rectangle list to redraw */
	if (wind_get(winhandle, WF_FIRSTXYWH, &todo[0], &todo[1], &todo[2], &todo[3])!=0) {

		while (todo[2] && todo[3]) {

			if (rc_intersect((GRECT *)inside,(GRECT *)todo)) {
				todo[2] += todo[0]-1;
				todo[3] += todo[1]-1;
				refresh_window(this, winhandle, todo);
			}

			if (wind_get(winhandle, WF_NEXTXYWH, &todo[0], &todo[1], &todo[2], &todo[3])==0) {
				break;
			}
		}

	}

	/* Update finished */
	wind_update(END_UPDATE);

	v_show_c(VDI_handle,1);
}

static void refresh_window(_THIS, int winhandle, short *rect)
{
	MFDB mfdb_src;
	short pxy[8],wind_pxy[8];
	SDL_Surface *surface;
	int iconified;

	/* Is window iconified ? */
	iconified = 0;
/*	if (GEM_wfeatures & (1<<WF_ICONIFY))*/ {
		if (wind_get(winhandle, WF_ICONIFY, &wind_pxy[0], &wind_pxy[1], &wind_pxy[2], &wind_pxy[3])!=0) {
			iconified = wind_pxy[0];
		}
	}

	if (wind_get(winhandle, WF_WORKXYWH, &wind_pxy[0], &wind_pxy[1], &wind_pxy[2], &wind_pxy[3])==0) {
		return;
	}

	if (iconified && GEM_icon) {
		short icon_rect[4], dst_rect[4];
		short iconx,icony;
		
		surface = GEM_icon;

		GEM_ClearRect(this, rect);

		/* Calculate centered icon(x,y,w,h) relative to window */
		iconx = (wind_pxy[2]-surface->w)>>1;
		icony = (wind_pxy[3]-surface->h)>>1;

		icon_rect[0] = iconx;
		icon_rect[1] = icony;
		icon_rect[2] = surface->w;
		icon_rect[3] = surface->h;

		/* Calculate redraw rectangle(x,y,w,h) relative to window */
		dst_rect[0] = rect[0]-wind_pxy[0];
		dst_rect[1] = rect[1]-wind_pxy[1];
		dst_rect[2] = rect[2]-rect[0]+1;
		dst_rect[3] = rect[3]-rect[1]+1;

		/* Does the icon rectangle must be redrawn ? */
		if (!rc_intersect((GRECT *)icon_rect, (GRECT *)dst_rect)) {
			return;
		}

#if DEBUG_VIDEO_GEM
		printf("sdl:video:gem:  clip(0,0,%d,%d) to (%d,%d,%d,%d)\n",
			surface->w-1,surface->h-1, dst_rect[0],dst_rect[1],dst_rect[2],dst_rect[3]);
		printf("sdl:video:gem:  icon(%d,%d,%d,%d)\n",
			icon_rect[0], icon_rect[1], icon_rect[2], icon_rect[3]);
		printf("sdl:video:gem: refresh_window(): draw icon\n");
#endif

		/* Calculate icon(x1,y1,x2,y2) relative to screen */
		icon_rect[0] += wind_pxy[0];
		icon_rect[1] += wind_pxy[1];
		icon_rect[2] += icon_rect[0]-1;
		icon_rect[3] += icon_rect[1]-1;

		/* Calculate intersection rectangle to redraw */
		pxy[4]=pxy[0]=MAX(icon_rect[0],rect[0]);
		pxy[5]=pxy[1]=MAX(icon_rect[1],rect[1]);
 		pxy[6]=pxy[2]=MIN(icon_rect[2],rect[2]);
	 	pxy[7]=pxy[3]=MIN(icon_rect[3],rect[3]);

		/* Calculate icon source image pos relative to window */
		pxy[0] -= wind_pxy[0]+iconx;
		pxy[1] -= wind_pxy[1]+icony;
		pxy[2] -= wind_pxy[0]+iconx;
		pxy[3] -= wind_pxy[1]+icony;

	} else {
		surface = this->screen;

#if DEBUG_VIDEO_GEM
		printf("sdl:video:gem: refresh_window(): draw frame buffer\n");
#endif

		/* Redraw all window content */
		pxy[0] = rect[0]-wind_pxy[0];
		pxy[1] = rect[1]-wind_pxy[1];
	 	pxy[2] = rect[2]-wind_pxy[0];   
	 	pxy[3] = rect[3]-wind_pxy[1];  

		pxy[4] = rect[0];
		pxy[5] = rect[1];
		pxy[6] = rect[2];  
		pxy[7] = rect[3];
	}

	if (GEM_bufops & B2S_C2P_1TO2) {
		void *src, *dest;
		int x1,x2;

		x1 = (rect[0]-wind_pxy[0]) & ~15;
		x2 = rect[2]-wind_pxy[0];
		if (x2 & 15) {
			x2 = (x2 | 15) +1;
		}

		src = surface->pixels;
		src += surface->pitch * (rect[1]-wind_pxy[1]);
		src += x1;

		dest = GEM_buffer2;
		dest += surface->pitch * (rect[1]-wind_pxy[1]);
		dest += x1;

		SDL_Atari_C2pConvert(
			src, dest,
			x2-x1, rect[3]-rect[1]+1,
			SDL_FALSE,
			surface->pitch, surface->pitch
		);
	}

	mfdb_src.fd_addr=surface->pixels;
	{
		int width;

		/* Need to be a multiple of 16 pixels */
		width=surface->w;
		if ((width & 15) != 0) {
			width = (width | 15) + 1;
		}
		mfdb_src.fd_w=width;
	}
	mfdb_src.fd_h=surface->h;
  	mfdb_src.fd_nplanes=surface->format->BitsPerPixel;
	mfdb_src.fd_wdwidth=mfdb_src.fd_w>>4;
	mfdb_src.fd_stand=
		mfdb_src.fd_r1=
  		mfdb_src.fd_r2=
	  	mfdb_src.fd_r3= 0;

	if (GEM_bufops & B2S_VROCPYFM_2TOS) {
		mfdb_src.fd_addr=GEM_buffer2;
	}

#if DEBUG_VIDEO_GEM
	printf("sdl:video:gem: redraw %dx%d: (%d,%d,%d,%d) to (%d,%d,%d,%d)\n",
		surface->w, surface->h,
		pxy[0],pxy[1],pxy[2],pxy[3],
		pxy[4],pxy[5],pxy[6],pxy[7]
	);
#endif

	vro_cpyfm( VDI_handle, S_ONLY, pxy, &mfdb_src, &VDI_dst_mfdb);
}

#if SDL_VIDEO_OPENGL

static void GEM_GL_SwapBuffers(_THIS)
{
	SDL_AtariGL_SwapBuffers(this);
	GEM_FlipHWSurface(this, this->screen);
}

#endif
