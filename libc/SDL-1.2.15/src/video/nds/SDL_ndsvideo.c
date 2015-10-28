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

#include <nds.h>
#include <nds/registers_alt.h>
#include "SDL.h"
#include "SDL_error.h"
#include "SDL_video.h"
#include "SDL_mouse.h"
#include "../SDL_sysvideo.h"
#include "../SDL_pixels_c.h"
#include "../../events/SDL_events_c.h"

#include "SDL_ndsvideo.h"
#include "SDL_ndsevents_c.h"
#include "SDL_ndsmouse_c.h"

#define NDSVID_DRIVER_NAME "nds"

/* Initialization/Query functions */
static int NDS_VideoInit(_THIS, SDL_PixelFormat *vformat);
static SDL_Rect **NDS_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags);
static SDL_Surface *NDS_SetVideoMode(_THIS, SDL_Surface *current, int width, int height, int bpp, Uint32 flags);
static int NDS_SetColors(_THIS, int firstcolor, int ncolors, SDL_Color *colors);
static void NDS_VideoQuit(_THIS);

/* Hardware surface functions */
static int NDS_AllocHWSurface(_THIS, SDL_Surface *surface);
static int NDS_LockHWSurface(_THIS, SDL_Surface *surface);
static int NDS_FlipHWSurface(_THIS, SDL_Surface *surface);
static void NDS_UnlockHWSurface(_THIS, SDL_Surface *surface);
static void NDS_FreeHWSurface(_THIS, SDL_Surface *surface);

/* etc. */
static void NDS_UpdateRects(_THIS, int numrects, SDL_Rect *rects);

/* NDS driver bootstrap functions */

static int NDS_Available(void)
{
	return(1);
}

static void NDS_DeleteDevice(SDL_VideoDevice *device)
{
	SDL_free(device->hidden);
	SDL_free(device);
}

void on_irq_vblank() 
{	
  // Disable interrupts
  //REG_IME = 0;
  scanKeys();

  //  VBLANK_INTR_WAIT_FLAGS |= IRQ_VBLANK; 
  //  REG_IF |= IRQ_VBLANK; 
  //REG_IF = REG_IF;

  // Enable interrupts
  //REG_IME = 1;
}

static int HWAccelBlit(SDL_Surface *src, SDL_Rect *srcrect,
                        SDL_Surface *dst, SDL_Rect *dstrect)
 {
	return 0;
 }
 
static int CheckHWBlit(_THIS, SDL_Surface *src, SDL_Surface *dst)
{
 	if (src->flags & SDL_SRCALPHA) return false;
 	if (src->flags & SDL_SRCCOLORKEY) return false;
 	if (src->flags & SDL_HWPALETTE ) return false;
 	if (dst->flags & SDL_SRCALPHA) return false;
 	if (dst->flags & SDL_SRCCOLORKEY) return false;
 	if (dst->flags & SDL_HWPALETTE ) return false;

 	if (src->format->BitsPerPixel != dst->format->BitsPerPixel) return false;
 	if (src->format->BytesPerPixel != dst->format->BytesPerPixel) return false;
 		
        src->map->hw_blit = HWAccelBlit;
        return true;
}

static SDL_VideoDevice *NDS_CreateDevice(int devindex)
{
	SDL_VideoDevice *device=0;


	/* Initialize all variables that we clean on shutdown */
	device = (SDL_VideoDevice *)SDL_malloc(sizeof(SDL_VideoDevice));
	if ( device ) {
		SDL_memset(device, 0, (sizeof *device));
		device->hidden = (struct SDL_PrivateVideoData *)
				SDL_malloc((sizeof *device->hidden));
	}
	if ( (device == NULL) || (device->hidden == NULL) ) {
		SDL_OutOfMemory();
		if ( device ) {
			SDL_free(device);
		}
		return(0);
	} 
	SDL_memset(device->hidden, 0, (sizeof *device->hidden));

	/* Set the function pointers */
	device->VideoInit = NDS_VideoInit;
	device->ListModes = NDS_ListModes;
	device->SetVideoMode = NDS_SetVideoMode;
	device->CreateYUVOverlay = NULL;
	device->SetColors = NDS_SetColors;
	device->UpdateRects = NDS_UpdateRects;
	device->VideoQuit = NDS_VideoQuit;
	device->AllocHWSurface = NDS_AllocHWSurface;
	device->CheckHWBlit = CheckHWBlit;
	device->FillHWRect = NULL;
	device->SetHWColorKey = NULL;
	device->SetHWAlpha = NULL;
	device->LockHWSurface = NDS_LockHWSurface;
	device->UnlockHWSurface = NDS_UnlockHWSurface;
	device->FlipHWSurface = NDS_FlipHWSurface;
	device->FreeHWSurface = NDS_FreeHWSurface;
	device->SetCaption = NULL;
	device->SetIcon = NULL;
	device->IconifyWindow = NULL;
	device->GrabInput = NULL;
	device->GetWMInfo = NULL;
	device->InitOSKeymap = NDS_InitOSKeymap;
	device->PumpEvents = NDS_PumpEvents;
	device->info.blit_hw=1;

	device->free = NDS_DeleteDevice;
	return device;
}

VideoBootStrap NDS_bootstrap = {
	NDSVID_DRIVER_NAME, "SDL NDS video driver",
	NDS_Available, NDS_CreateDevice
};

	u16* frontBuffer;// = (u16*)(0x06000000);
	u16* backBuffer;// =  (u16*)(0x06000000 + 256 * 256 * 2); 
int NDS_VideoInit(_THIS, SDL_PixelFormat *vformat)
{
	//printf("WARNING: You are using the SDL NDS video driver!\n");

	/* Determine the screen depth (use default 8-bit depth) */
	/* we change this during the SDL_SetVideoMode implementation... */
	vformat->BitsPerPixel = 16;	// mode 3
	vformat->BytesPerPixel = 2;
	vformat->Rmask = 0x0000f800;
	vformat->Gmask = 0x000007e0;
	vformat->Bmask = 0x0000001f; 
    powerON(POWER_ALL);
	irqInit();
	irqSet(IRQ_VBLANK, on_irq_vblank); 
	irqEnable(IRQ_VBLANK);

    //set the mode for 2 text layers and two extended background layers
	//videoSetMode(MODE_5_2D | DISPLAY_BG3_ACTIVE); 
	videoSetMode(MODE_6_2D| DISPLAY_BG2_ACTIVE); 
	
	//set the sub background up for text display (we could just print to one
	//of the main display text backgrounds just as easily
	videoSetModeSub(MODE_0_2D | DISPLAY_BG0_ACTIVE); //sub bg 0 will be used to print text
	
    //set the first two banks as background memory and the third as sub background memory
    //D is not used..if you need a bigger background then you will need to map
    //more vram banks consecutivly (VRAM A-D are all 0x20000 bytes in size)
    //vramSetMainBanks(VRAM_A_MAIN_BG_0x6000000, VRAM_B_MAIN_BG_0x6020000,VRAM_C_SUB_BG , VRAM_D_LCD); 
	vramSetMainBanks(VRAM_A_MAIN_BG,VRAM_B_MAIN_BG,VRAM_C_MAIN_BG,VRAM_D_MAIN_BG);
	//vramSetBankA(VRAM_A_MAIN_BG);
	//vramSetBankB(VRAM_B_MAIN_BG);
	//vramSetBankC(VRAM_C_MAIN_BG);
	//vramSetBankD(VRAM_D_MAIN_BG);
	//vramSetBankE(VRAM_E_MAIN_BG);
	//vramSetBankF(VRAM_F_MAIN_BG);
	//vramSetBankG(VRAM_G_MAIN_BG);
	vramSetBankH(VRAM_H_SUB_BG);
	vramSetBankI(VRAM_I_LCD);
    
	////////////////set up text background for text/////////////////////
    SUB_BG0_CR = BG_MAP_BASE(8);
	
	BG_PALETTE_SUB[255] = RGB15(31,31,31);//by default font will be rendered with color 255
	///////////////set up our bitmap background///////////////////////

	//BG3_CR = BG_BMP16_512x512;
	
	//these are rotation backgrounds so you must set the rotation attributes:
    //these are fixed point numbers with the low 8 bits the fractional part
    //this basicaly gives it a 1:1 translation in x and y so you get a nice flat bitmap
      /*  BG3_XDX = 1<<8;
        BG3_XDY = 0; 
        BG3_YDX = 0;
        BG3_YDY = 1<<8;
    //our bitmap looks a bit better if we center it so scroll down (256 - 192) / 2 
        BG3_CX = 0;
        BG3_CY = 0; 	
		*/
	//consoleInit() is a lot more flexible but this gets you up and running quick
	consoleInitDefault((u16*)SCREEN_BASE_BLOCK_SUB(8), (u16*)CHAR_BASE_BLOCK_SUB(0), 16); 
	

	frontBuffer =(u16*)(0x06000000);
	//backBuffer  =(u16*)(0x06000000 + 1024 * 512*2); 

	//lcdSwap();
	/* We're done! */
	return(0); 
}

SDL_Rect **NDS_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags)
{
	return (SDL_Rect **) -1;
}

SDL_Surface *NDS_SetVideoMode(_THIS, SDL_Surface *current,
				int width, int height, int bpp, Uint32 flags)
{
	Uint32 Rmask, Gmask, Bmask, Amask; 

	//if(width > 1024 || height > 512 || bpp > 16)
	//	return(NULL);

	if(bpp >8) {
		bpp=16;
 		Rmask = 0x0000001F;
		Gmask = 0x000003E0;
		Bmask = 0x00007C00;
		Amask = 0x00008000;

		videoSetMode(MODE_5_2D| DISPLAY_BG2_ACTIVE); 

		vramSetMainBanks(VRAM_A_MAIN_BG,VRAM_B_MAIN_BG,VRAM_C_MAIN_BG,VRAM_D_MAIN_BG);

		BG2_CR = BG_BMP16_512x512;
	    BG2_XDX = ((width / 256) << 8) | (width % 256) ; 
        BG2_XDY = 0; 
        BG2_YDX = 0;	
        BG2_YDY = ((height / 192) << 8) | ((height % 192) + (height % 192) / 3) ;
        BG2_CX = 0;
        BG2_CY = 0; 
//        for (i=0;i<256*192;i++)
//	        frontBuffer[i] = RGB15(31,0,0)|BIT(15);
	}
	else
	if(bpp <= 8) {
		bpp=8;
		Rmask = 0x00000000;
		Gmask = 0x00000000; 
		Bmask = 0x00000000;
		BG2_CR = BG_BMP8_1024x512;
        BG2_XDX = ((width / 256) << 8) | (width % 256) ;
        BG2_XDY = 0; 
        BG2_YDX = 0;
        BG2_YDY = ((height / 192) << 8) | ((height % 192) + (height % 192) / 3) ;

	}
	else
		if(bpp < 15) bpp=15;
	if(width<=256) width=256;
	else
		if(width<256) width=256;
	if(height<=192) height=192;
	else
		if(height<192) height=192;
	
	if(bpp==8)
	{
		if(width<256) width=256;
		if(height<192) height=192;
		this->hidden->ndsmode=4;
	}
	
	if(bpp==15)
	{
		if(width<256) this->hidden->ndsmode=5;
		else this->hidden->ndsmode=3; 
	}

	this->hidden->buffer= frontBuffer;//NDS_VRAM_BASE;
	
	//NDS_DISPCNT = NDS_DISP_MODE(this->hidden->ndsmode)|NDS_DISP_BG2;
	
 	//fprintf(stderr,"Setting mode %dx%d (ndsmode %d)\n", width, height,this->hidden->ndsmode);

	// FIXME: How do I tell that 15 bits mode is 555?

	SDL_memset(this->hidden->buffer, 0, 1024 * 512* ((this->hidden->ndsmode==4 || this->hidden->ndsmode==5) ? 2 : 1 ) * ((bpp+7) / 8));

	/* Allocate the new pixel format for the screen */
	if ( ! SDL_ReallocFormat(current, bpp, Rmask, Gmask, Bmask, Amask) ) {
		this->hidden->buffer = NULL;
		SDL_SetError("Couldn't allocate new pixel format for requested mode");
		return(NULL);
	}

	/* Set up the new mode framebuffer */
	current->flags = flags | SDL_FULLSCREEN | SDL_HWSURFACE | (this->hidden->ndsmode > 0 ? SDL_DOUBLEBUF : 0);
	this->hidden->w = current->w = width;
	this->hidden->h = current->h = height;
	current->pixels = frontBuffer;

	if (flags & SDL_DOUBLEBUF) { 
		this->hidden->secondbufferallocd=1;
		backBuffer=(u16*)SDL_malloc(1024*512*2);
		current->pixels = backBuffer; 
	}
	if(bpp==8)
		current->pitch =1024;
	else
		current->pitch =512*2;

	/* We're done */
	return(current);
}

static int NDS_AllocHWSurface(_THIS, SDL_Surface *surface)
{
	if(this->hidden->secondbufferallocd) {
		//printf("double double buffer alloc\n");
		return -1;
	}
	//if(this->hidden->ndsmode==3)
	//{
	//	printf("no 2nd buffer in mode3\n");
	//	return -1;
	//}
	//printf("second buffer\n");
	//this->hidden->secondbufferallocd=1;
	//backBuffer=(u16*)malloc(1024*512*2);
	//surface->pixels = backBuffer; 

	return(0);
}
static void NDS_FreeHWSurface(_THIS, SDL_Surface *surface)
{
	//free(backBuffer);
	this->hidden->secondbufferallocd=0;
}
int z=0;
/* We need to wait for vertical retrace on page flipped displays */
static int NDS_LockHWSurface(_THIS, SDL_Surface *surface)
{
/*
	uint8* a = surface->pixels;
  int i,j;
  a += 5 * SCREEN_WIDTH + 5;
  for( i = 0; i < 195; ++i) {
    uint16* line = a + (SCREEN_WIDTH * i);
    for( j = 0; j < 158; ++j) {
      *line++ = RGB15(155,155,25);
    }
  }
*/
	//if (z <256)
	// BG_PALETTE[z++]=RGB15(255-z,z,255-z);

 
	return(0);
}

static void NDS_UnlockHWSurface(_THIS, SDL_Surface *surface)
{
	return;
}

static int NDS_FlipHWSurface(_THIS, SDL_Surface *surface)
{
	if(this->hidden->secondbufferallocd){
		while(DISP_Y!=192);
	    while(DISP_Y==192); 
		//printf("flip");

		dmaCopyAsynch(backBuffer,frontBuffer,1024*512);
	}
		//printf("flip\n");
        //u16* temp = surface->pixels;
        //surface->pixels = frontBuffer;
        //frontBuffer = temp;
	/*	u8* vram=BG_GFX;
	int x,y;
	for(y = 0; y < 512; y++)
		dmaCopy(&frontBuffer[y*rects->w], &vram[y*512],512);
	//unsigned char buf;
	
	//printf("NDS_FlipHWSurface\n");
	//printf("ptr now: 0x%x\n",surface->pixels);

	    while(DISP_Y!=192);
	    while(DISP_Y==192); 
        //swap
        u16* temp = frontBuffer;
        frontBuffer = backBuffer;
        backBuffer = temp;
        
        //flip 
        //base is 16KB and screen size is 256x256x2 (128KB)
        BG2_CR ^= BG_BMP_BASE( 512 / 16 ); */
/*
	if(surface->pixels == frontBuffer)//NDS_VRAM_BASE)
	{
			while(DISP_Y!=192);
	while(DISP_Y==192); 
        //swap
        u16* temp = backBuffer;
        backBuffer = frontBuffer;
        frontBuffer = temp;
        
        //flip 
        //base is 16KB and screen size is 256x256x2 (128KB)
        BG3_CR ^= BG_BMP_BASE( 128 / 16 ); 
	}
	else
	{

		while(DISP_Y!=192);
	while(DISP_Y==192); 
        //swap
        u16* temp = frontBuffer;
        frontBuffer = backBuffer;
        backBuffer = temp;
        
        //flip 
        //base is 16KB and screen size is 256x256x2 (128KB)
        BG3_CR ^= BG_BMP_BASE( 128 / 16 ); 
		
	}
	*/
	//printf("ptr then: 0x%x\n",surface->pixels);

	//printf("setting dispcnt to 0x%x\n",NDS_DISPCNT = NDS_DISP_MODE(this->hidden->ndsmode)|NDS_DISP_BG2| buf);
	return(0);
}

static void NDS_UpdateRects(_THIS, int numrects, SDL_Rect *rects)
{
	//fprintf(stderr,"update\n");
	/* do nothing. */
	//dmaCopy(frontBuffer,BG_GFX,512*512);
	 /*
	u8* vram=(u8*)BG_GFX;
	int x,y;
	for(y = 0; y < 512; y++)
		dmaCopy(&frontBuffer[y*rects->w], &vram[y*512],512);
	 */

}

int NDS_SetColors(_THIS, int firstcolor, int ncolors, SDL_Color *colors)
{
	//printf("SetColors\n");
	short r,g,b;
	
	if(this->hidden->ndsmode != 4)
	{
		printf("This is not a palettized mode\n");
		return -1;
	}

	int i,j=firstcolor+ncolors;
	for(i=firstcolor;i<j;i++)
	{
		r=colors[i].r>>3;
		g=colors[i].g>>3;
		b=colors[i].b>>3;
		BG_PALETTE[i]=RGB15(r, g, b);
	} 

	return(0);
}

/* Note:  If we are terminated, this could be called in the middle of
   another SDL video routine -- notably UpdateRects.
*/
void NDS_VideoQuit(_THIS)
{
}
