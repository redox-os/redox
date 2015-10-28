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

#include "directx.h"

/* Not yet in the mingw32 cross-compile headers */
#ifndef CDS_FULLSCREEN
#define CDS_FULLSCREEN	4
#endif

#include "SDL_timer.h"
#include "SDL_events.h"
#include "SDL_syswm.h"
#include "../SDL_sysvideo.h"
#include "../SDL_blit.h"
#include "../SDL_pixels_c.h"
#include "SDL_dx5video.h"
#include "../wincommon/SDL_syswm_c.h"
#include "../wincommon/SDL_sysmouse_c.h"
#include "SDL_dx5events_c.h"
#include "SDL_dx5yuv_c.h"
#include "../wincommon/SDL_wingl_c.h"

#ifdef _WIN32_WCE
#define NO_CHANGEDISPLAYSETTINGS
#endif
#ifndef WS_MAXIMIZE
#define WS_MAXIMIZE		0
#endif
#ifndef SWP_NOCOPYBITS
#define SWP_NOCOPYBITS	0
#endif
#ifndef PC_NOCOLLAPSE
#define PC_NOCOLLAPSE	0
#endif


/* DirectX function pointers for video and events */
HRESULT (WINAPI *DDrawCreate)( GUID FAR *lpGUID, LPDIRECTDRAW FAR *lplpDD, IUnknown FAR *pUnkOuter );
HRESULT (WINAPI *DInputCreate)(HINSTANCE hinst, DWORD dwVersion, LPDIRECTINPUT *ppDI, LPUNKNOWN punkOuter);

/* This is the rect EnumModes2 uses */
struct DX5EnumRect {
	SDL_Rect r;
	int refreshRate;
	struct DX5EnumRect* next;
};
static struct DX5EnumRect *enumlists[NUM_MODELISTS];

/*
 * Experimentally determined values for c_cfDI* constants used in DirectX 5.0
 */

/* Keyboard */

static DIOBJECTDATAFORMAT KBD_fmt[] = {
	{ &GUID_Key, 0, 0x8000000C, 0x00000000 },
	{ &GUID_Key, 1, 0x8000010C, 0x00000000 },
	{ &GUID_Key, 2, 0x8000020C, 0x00000000 },
	{ &GUID_Key, 3, 0x8000030C, 0x00000000 },
	{ &GUID_Key, 4, 0x8000040C, 0x00000000 },
	{ &GUID_Key, 5, 0x8000050C, 0x00000000 },
	{ &GUID_Key, 6, 0x8000060C, 0x00000000 },
	{ &GUID_Key, 7, 0x8000070C, 0x00000000 },
	{ &GUID_Key, 8, 0x8000080C, 0x00000000 },
	{ &GUID_Key, 9, 0x8000090C, 0x00000000 },
	{ &GUID_Key, 10, 0x80000A0C, 0x00000000 },
	{ &GUID_Key, 11, 0x80000B0C, 0x00000000 },
	{ &GUID_Key, 12, 0x80000C0C, 0x00000000 },
	{ &GUID_Key, 13, 0x80000D0C, 0x00000000 },
	{ &GUID_Key, 14, 0x80000E0C, 0x00000000 },
	{ &GUID_Key, 15, 0x80000F0C, 0x00000000 },
	{ &GUID_Key, 16, 0x8000100C, 0x00000000 },
	{ &GUID_Key, 17, 0x8000110C, 0x00000000 },
	{ &GUID_Key, 18, 0x8000120C, 0x00000000 },
	{ &GUID_Key, 19, 0x8000130C, 0x00000000 },
	{ &GUID_Key, 20, 0x8000140C, 0x00000000 },
	{ &GUID_Key, 21, 0x8000150C, 0x00000000 },
	{ &GUID_Key, 22, 0x8000160C, 0x00000000 },
	{ &GUID_Key, 23, 0x8000170C, 0x00000000 },
	{ &GUID_Key, 24, 0x8000180C, 0x00000000 },
	{ &GUID_Key, 25, 0x8000190C, 0x00000000 },
	{ &GUID_Key, 26, 0x80001A0C, 0x00000000 },
	{ &GUID_Key, 27, 0x80001B0C, 0x00000000 },
	{ &GUID_Key, 28, 0x80001C0C, 0x00000000 },
	{ &GUID_Key, 29, 0x80001D0C, 0x00000000 },
	{ &GUID_Key, 30, 0x80001E0C, 0x00000000 },
	{ &GUID_Key, 31, 0x80001F0C, 0x00000000 },
	{ &GUID_Key, 32, 0x8000200C, 0x00000000 },
	{ &GUID_Key, 33, 0x8000210C, 0x00000000 },
	{ &GUID_Key, 34, 0x8000220C, 0x00000000 },
	{ &GUID_Key, 35, 0x8000230C, 0x00000000 },
	{ &GUID_Key, 36, 0x8000240C, 0x00000000 },
	{ &GUID_Key, 37, 0x8000250C, 0x00000000 },
	{ &GUID_Key, 38, 0x8000260C, 0x00000000 },
	{ &GUID_Key, 39, 0x8000270C, 0x00000000 },
	{ &GUID_Key, 40, 0x8000280C, 0x00000000 },
	{ &GUID_Key, 41, 0x8000290C, 0x00000000 },
	{ &GUID_Key, 42, 0x80002A0C, 0x00000000 },
	{ &GUID_Key, 43, 0x80002B0C, 0x00000000 },
	{ &GUID_Key, 44, 0x80002C0C, 0x00000000 },
	{ &GUID_Key, 45, 0x80002D0C, 0x00000000 },
	{ &GUID_Key, 46, 0x80002E0C, 0x00000000 },
	{ &GUID_Key, 47, 0x80002F0C, 0x00000000 },
	{ &GUID_Key, 48, 0x8000300C, 0x00000000 },
	{ &GUID_Key, 49, 0x8000310C, 0x00000000 },
	{ &GUID_Key, 50, 0x8000320C, 0x00000000 },
	{ &GUID_Key, 51, 0x8000330C, 0x00000000 },
	{ &GUID_Key, 52, 0x8000340C, 0x00000000 },
	{ &GUID_Key, 53, 0x8000350C, 0x00000000 },
	{ &GUID_Key, 54, 0x8000360C, 0x00000000 },
	{ &GUID_Key, 55, 0x8000370C, 0x00000000 },
	{ &GUID_Key, 56, 0x8000380C, 0x00000000 },
	{ &GUID_Key, 57, 0x8000390C, 0x00000000 },
	{ &GUID_Key, 58, 0x80003A0C, 0x00000000 },
	{ &GUID_Key, 59, 0x80003B0C, 0x00000000 },
	{ &GUID_Key, 60, 0x80003C0C, 0x00000000 },
	{ &GUID_Key, 61, 0x80003D0C, 0x00000000 },
	{ &GUID_Key, 62, 0x80003E0C, 0x00000000 },
	{ &GUID_Key, 63, 0x80003F0C, 0x00000000 },
	{ &GUID_Key, 64, 0x8000400C, 0x00000000 },
	{ &GUID_Key, 65, 0x8000410C, 0x00000000 },
	{ &GUID_Key, 66, 0x8000420C, 0x00000000 },
	{ &GUID_Key, 67, 0x8000430C, 0x00000000 },
	{ &GUID_Key, 68, 0x8000440C, 0x00000000 },
	{ &GUID_Key, 69, 0x8000450C, 0x00000000 },
	{ &GUID_Key, 70, 0x8000460C, 0x00000000 },
	{ &GUID_Key, 71, 0x8000470C, 0x00000000 },
	{ &GUID_Key, 72, 0x8000480C, 0x00000000 },
	{ &GUID_Key, 73, 0x8000490C, 0x00000000 },
	{ &GUID_Key, 74, 0x80004A0C, 0x00000000 },
	{ &GUID_Key, 75, 0x80004B0C, 0x00000000 },
	{ &GUID_Key, 76, 0x80004C0C, 0x00000000 },
	{ &GUID_Key, 77, 0x80004D0C, 0x00000000 },
	{ &GUID_Key, 78, 0x80004E0C, 0x00000000 },
	{ &GUID_Key, 79, 0x80004F0C, 0x00000000 },
	{ &GUID_Key, 80, 0x8000500C, 0x00000000 },
	{ &GUID_Key, 81, 0x8000510C, 0x00000000 },
	{ &GUID_Key, 82, 0x8000520C, 0x00000000 },
	{ &GUID_Key, 83, 0x8000530C, 0x00000000 },
	{ &GUID_Key, 84, 0x8000540C, 0x00000000 },
	{ &GUID_Key, 85, 0x8000550C, 0x00000000 },
	{ &GUID_Key, 86, 0x8000560C, 0x00000000 },
	{ &GUID_Key, 87, 0x8000570C, 0x00000000 },
	{ &GUID_Key, 88, 0x8000580C, 0x00000000 },
	{ &GUID_Key, 89, 0x8000590C, 0x00000000 },
	{ &GUID_Key, 90, 0x80005A0C, 0x00000000 },
	{ &GUID_Key, 91, 0x80005B0C, 0x00000000 },
	{ &GUID_Key, 92, 0x80005C0C, 0x00000000 },
	{ &GUID_Key, 93, 0x80005D0C, 0x00000000 },
	{ &GUID_Key, 94, 0x80005E0C, 0x00000000 },
	{ &GUID_Key, 95, 0x80005F0C, 0x00000000 },
	{ &GUID_Key, 96, 0x8000600C, 0x00000000 },
	{ &GUID_Key, 97, 0x8000610C, 0x00000000 },
	{ &GUID_Key, 98, 0x8000620C, 0x00000000 },
	{ &GUID_Key, 99, 0x8000630C, 0x00000000 },
	{ &GUID_Key, 100, 0x8000640C, 0x00000000 },
	{ &GUID_Key, 101, 0x8000650C, 0x00000000 },
	{ &GUID_Key, 102, 0x8000660C, 0x00000000 },
	{ &GUID_Key, 103, 0x8000670C, 0x00000000 },
	{ &GUID_Key, 104, 0x8000680C, 0x00000000 },
	{ &GUID_Key, 105, 0x8000690C, 0x00000000 },
	{ &GUID_Key, 106, 0x80006A0C, 0x00000000 },
	{ &GUID_Key, 107, 0x80006B0C, 0x00000000 },
	{ &GUID_Key, 108, 0x80006C0C, 0x00000000 },
	{ &GUID_Key, 109, 0x80006D0C, 0x00000000 },
	{ &GUID_Key, 110, 0x80006E0C, 0x00000000 },
	{ &GUID_Key, 111, 0x80006F0C, 0x00000000 },
	{ &GUID_Key, 112, 0x8000700C, 0x00000000 },
	{ &GUID_Key, 113, 0x8000710C, 0x00000000 },
	{ &GUID_Key, 114, 0x8000720C, 0x00000000 },
	{ &GUID_Key, 115, 0x8000730C, 0x00000000 },
	{ &GUID_Key, 116, 0x8000740C, 0x00000000 },
	{ &GUID_Key, 117, 0x8000750C, 0x00000000 },
	{ &GUID_Key, 118, 0x8000760C, 0x00000000 },
	{ &GUID_Key, 119, 0x8000770C, 0x00000000 },
	{ &GUID_Key, 120, 0x8000780C, 0x00000000 },
	{ &GUID_Key, 121, 0x8000790C, 0x00000000 },
	{ &GUID_Key, 122, 0x80007A0C, 0x00000000 },
	{ &GUID_Key, 123, 0x80007B0C, 0x00000000 },
	{ &GUID_Key, 124, 0x80007C0C, 0x00000000 },
	{ &GUID_Key, 125, 0x80007D0C, 0x00000000 },
	{ &GUID_Key, 126, 0x80007E0C, 0x00000000 },
	{ &GUID_Key, 127, 0x80007F0C, 0x00000000 },
	{ &GUID_Key, 128, 0x8000800C, 0x00000000 },
	{ &GUID_Key, 129, 0x8000810C, 0x00000000 },
	{ &GUID_Key, 130, 0x8000820C, 0x00000000 },
	{ &GUID_Key, 131, 0x8000830C, 0x00000000 },
	{ &GUID_Key, 132, 0x8000840C, 0x00000000 },
	{ &GUID_Key, 133, 0x8000850C, 0x00000000 },
	{ &GUID_Key, 134, 0x8000860C, 0x00000000 },
	{ &GUID_Key, 135, 0x8000870C, 0x00000000 },
	{ &GUID_Key, 136, 0x8000880C, 0x00000000 },
	{ &GUID_Key, 137, 0x8000890C, 0x00000000 },
	{ &GUID_Key, 138, 0x80008A0C, 0x00000000 },
	{ &GUID_Key, 139, 0x80008B0C, 0x00000000 },
	{ &GUID_Key, 140, 0x80008C0C, 0x00000000 },
	{ &GUID_Key, 141, 0x80008D0C, 0x00000000 },
	{ &GUID_Key, 142, 0x80008E0C, 0x00000000 },
	{ &GUID_Key, 143, 0x80008F0C, 0x00000000 },
	{ &GUID_Key, 144, 0x8000900C, 0x00000000 },
	{ &GUID_Key, 145, 0x8000910C, 0x00000000 },
	{ &GUID_Key, 146, 0x8000920C, 0x00000000 },
	{ &GUID_Key, 147, 0x8000930C, 0x00000000 },
	{ &GUID_Key, 148, 0x8000940C, 0x00000000 },
	{ &GUID_Key, 149, 0x8000950C, 0x00000000 },
	{ &GUID_Key, 150, 0x8000960C, 0x00000000 },
	{ &GUID_Key, 151, 0x8000970C, 0x00000000 },
	{ &GUID_Key, 152, 0x8000980C, 0x00000000 },
	{ &GUID_Key, 153, 0x8000990C, 0x00000000 },
	{ &GUID_Key, 154, 0x80009A0C, 0x00000000 },
	{ &GUID_Key, 155, 0x80009B0C, 0x00000000 },
	{ &GUID_Key, 156, 0x80009C0C, 0x00000000 },
	{ &GUID_Key, 157, 0x80009D0C, 0x00000000 },
	{ &GUID_Key, 158, 0x80009E0C, 0x00000000 },
	{ &GUID_Key, 159, 0x80009F0C, 0x00000000 },
	{ &GUID_Key, 160, 0x8000A00C, 0x00000000 },
	{ &GUID_Key, 161, 0x8000A10C, 0x00000000 },
	{ &GUID_Key, 162, 0x8000A20C, 0x00000000 },
	{ &GUID_Key, 163, 0x8000A30C, 0x00000000 },
	{ &GUID_Key, 164, 0x8000A40C, 0x00000000 },
	{ &GUID_Key, 165, 0x8000A50C, 0x00000000 },
	{ &GUID_Key, 166, 0x8000A60C, 0x00000000 },
	{ &GUID_Key, 167, 0x8000A70C, 0x00000000 },
	{ &GUID_Key, 168, 0x8000A80C, 0x00000000 },
	{ &GUID_Key, 169, 0x8000A90C, 0x00000000 },
	{ &GUID_Key, 170, 0x8000AA0C, 0x00000000 },
	{ &GUID_Key, 171, 0x8000AB0C, 0x00000000 },
	{ &GUID_Key, 172, 0x8000AC0C, 0x00000000 },
	{ &GUID_Key, 173, 0x8000AD0C, 0x00000000 },
	{ &GUID_Key, 174, 0x8000AE0C, 0x00000000 },
	{ &GUID_Key, 175, 0x8000AF0C, 0x00000000 },
	{ &GUID_Key, 176, 0x8000B00C, 0x00000000 },
	{ &GUID_Key, 177, 0x8000B10C, 0x00000000 },
	{ &GUID_Key, 178, 0x8000B20C, 0x00000000 },
	{ &GUID_Key, 179, 0x8000B30C, 0x00000000 },
	{ &GUID_Key, 180, 0x8000B40C, 0x00000000 },
	{ &GUID_Key, 181, 0x8000B50C, 0x00000000 },
	{ &GUID_Key, 182, 0x8000B60C, 0x00000000 },
	{ &GUID_Key, 183, 0x8000B70C, 0x00000000 },
	{ &GUID_Key, 184, 0x8000B80C, 0x00000000 },
	{ &GUID_Key, 185, 0x8000B90C, 0x00000000 },
	{ &GUID_Key, 186, 0x8000BA0C, 0x00000000 },
	{ &GUID_Key, 187, 0x8000BB0C, 0x00000000 },
	{ &GUID_Key, 188, 0x8000BC0C, 0x00000000 },
	{ &GUID_Key, 189, 0x8000BD0C, 0x00000000 },
	{ &GUID_Key, 190, 0x8000BE0C, 0x00000000 },
	{ &GUID_Key, 191, 0x8000BF0C, 0x00000000 },
	{ &GUID_Key, 192, 0x8000C00C, 0x00000000 },
	{ &GUID_Key, 193, 0x8000C10C, 0x00000000 },
	{ &GUID_Key, 194, 0x8000C20C, 0x00000000 },
	{ &GUID_Key, 195, 0x8000C30C, 0x00000000 },
	{ &GUID_Key, 196, 0x8000C40C, 0x00000000 },
	{ &GUID_Key, 197, 0x8000C50C, 0x00000000 },
	{ &GUID_Key, 198, 0x8000C60C, 0x00000000 },
	{ &GUID_Key, 199, 0x8000C70C, 0x00000000 },
	{ &GUID_Key, 200, 0x8000C80C, 0x00000000 },
	{ &GUID_Key, 201, 0x8000C90C, 0x00000000 },
	{ &GUID_Key, 202, 0x8000CA0C, 0x00000000 },
	{ &GUID_Key, 203, 0x8000CB0C, 0x00000000 },
	{ &GUID_Key, 204, 0x8000CC0C, 0x00000000 },
	{ &GUID_Key, 205, 0x8000CD0C, 0x00000000 },
	{ &GUID_Key, 206, 0x8000CE0C, 0x00000000 },
	{ &GUID_Key, 207, 0x8000CF0C, 0x00000000 },
	{ &GUID_Key, 208, 0x8000D00C, 0x00000000 },
	{ &GUID_Key, 209, 0x8000D10C, 0x00000000 },
	{ &GUID_Key, 210, 0x8000D20C, 0x00000000 },
	{ &GUID_Key, 211, 0x8000D30C, 0x00000000 },
	{ &GUID_Key, 212, 0x8000D40C, 0x00000000 },
	{ &GUID_Key, 213, 0x8000D50C, 0x00000000 },
	{ &GUID_Key, 214, 0x8000D60C, 0x00000000 },
	{ &GUID_Key, 215, 0x8000D70C, 0x00000000 },
	{ &GUID_Key, 216, 0x8000D80C, 0x00000000 },
	{ &GUID_Key, 217, 0x8000D90C, 0x00000000 },
	{ &GUID_Key, 218, 0x8000DA0C, 0x00000000 },
	{ &GUID_Key, 219, 0x8000DB0C, 0x00000000 },
	{ &GUID_Key, 220, 0x8000DC0C, 0x00000000 },
	{ &GUID_Key, 221, 0x8000DD0C, 0x00000000 },
	{ &GUID_Key, 222, 0x8000DE0C, 0x00000000 },
	{ &GUID_Key, 223, 0x8000DF0C, 0x00000000 },
	{ &GUID_Key, 224, 0x8000E00C, 0x00000000 },
	{ &GUID_Key, 225, 0x8000E10C, 0x00000000 },
	{ &GUID_Key, 226, 0x8000E20C, 0x00000000 },
	{ &GUID_Key, 227, 0x8000E30C, 0x00000000 },
	{ &GUID_Key, 228, 0x8000E40C, 0x00000000 },
	{ &GUID_Key, 229, 0x8000E50C, 0x00000000 },
	{ &GUID_Key, 230, 0x8000E60C, 0x00000000 },
	{ &GUID_Key, 231, 0x8000E70C, 0x00000000 },
	{ &GUID_Key, 232, 0x8000E80C, 0x00000000 },
	{ &GUID_Key, 233, 0x8000E90C, 0x00000000 },
	{ &GUID_Key, 234, 0x8000EA0C, 0x00000000 },
	{ &GUID_Key, 235, 0x8000EB0C, 0x00000000 },
	{ &GUID_Key, 236, 0x8000EC0C, 0x00000000 },
	{ &GUID_Key, 237, 0x8000ED0C, 0x00000000 },
	{ &GUID_Key, 238, 0x8000EE0C, 0x00000000 },
	{ &GUID_Key, 239, 0x8000EF0C, 0x00000000 },
	{ &GUID_Key, 240, 0x8000F00C, 0x00000000 },
	{ &GUID_Key, 241, 0x8000F10C, 0x00000000 },
	{ &GUID_Key, 242, 0x8000F20C, 0x00000000 },
	{ &GUID_Key, 243, 0x8000F30C, 0x00000000 },
	{ &GUID_Key, 244, 0x8000F40C, 0x00000000 },
	{ &GUID_Key, 245, 0x8000F50C, 0x00000000 },
	{ &GUID_Key, 246, 0x8000F60C, 0x00000000 },
	{ &GUID_Key, 247, 0x8000F70C, 0x00000000 },
	{ &GUID_Key, 248, 0x8000F80C, 0x00000000 },
	{ &GUID_Key, 249, 0x8000F90C, 0x00000000 },
	{ &GUID_Key, 250, 0x8000FA0C, 0x00000000 },
	{ &GUID_Key, 251, 0x8000FB0C, 0x00000000 },
	{ &GUID_Key, 252, 0x8000FC0C, 0x00000000 },
	{ &GUID_Key, 253, 0x8000FD0C, 0x00000000 },
	{ &GUID_Key, 254, 0x8000FE0C, 0x00000000 },
	{ &GUID_Key, 255, 0x8000FF0C, 0x00000000 },
};

const DIDATAFORMAT c_dfDIKeyboard = { sizeof(DIDATAFORMAT), sizeof(DIOBJECTDATAFORMAT), 0x00000002, 256, 256, KBD_fmt };


/* Mouse */

static DIOBJECTDATAFORMAT PTR_fmt[] = {
	{ &GUID_XAxis, 0, 0x00FFFF03, 0x00000000 },
	{ &GUID_YAxis, 4, 0x00FFFF03, 0x00000000 },
	{ &GUID_ZAxis, 8, 0x80FFFF03, 0x00000000 },
	{ NULL,       12, 0x00FFFF0C, 0x00000000 },
	{ NULL,       13, 0x00FFFF0C, 0x00000000 },
	{ NULL,       14, 0x80FFFF0C, 0x00000000 },
	{ NULL,       15, 0x80FFFF0C, 0x00000000 },
};

const DIDATAFORMAT c_dfDIMouse = { sizeof(DIDATAFORMAT), sizeof(DIOBJECTDATAFORMAT), 0x00000002, 16, 7, PTR_fmt };

static DIOBJECTDATAFORMAT PTR2_fmt[] = {
	{ &GUID_XAxis, 0, 0x00FFFF03, 0x00000000 },
	{ &GUID_YAxis, 4, 0x00FFFF03, 0x00000000 },
	{ &GUID_ZAxis, 8, 0x80FFFF03, 0x00000000 },
	{ NULL,       12, 0x00FFFF0C, 0x00000000 },
	{ NULL,       13, 0x00FFFF0C, 0x00000000 },
	{ NULL,       14, 0x80FFFF0C, 0x00000000 },
	{ NULL,       15, 0x80FFFF0C, 0x00000000 },
	{ NULL,       16, 0x80FFFF0C, 0x00000000 },
	{ NULL,       17, 0x80FFFF0C, 0x00000000 },
	{ NULL,       18, 0x80FFFF0C, 0x00000000 },
	{ NULL,       19, 0x80FFFF0C, 0x00000000 }
};

const DIDATAFORMAT c_dfDIMouse2 = { sizeof(DIDATAFORMAT), sizeof(DIOBJECTDATAFORMAT), 0x00000002, 20, 11, PTR2_fmt };


/* Joystick */

static DIOBJECTDATAFORMAT JOY_fmt[] = {
	{ &GUID_XAxis, 0, 0x80FFFF03, 0x00000100 },
	{ &GUID_YAxis, 4, 0x80FFFF03, 0x00000100 },
	{ &GUID_ZAxis, 8, 0x80FFFF03, 0x00000100 },
	{ &GUID_RxAxis, 12, 0x80FFFF03, 0x00000100 },
	{ &GUID_RyAxis, 16, 0x80FFFF03, 0x00000100 },
	{ &GUID_RzAxis, 20, 0x80FFFF03, 0x00000100 },
	{ &GUID_Slider, 24, 0x80FFFF03, 0x00000100 },
	{ &GUID_Slider, 28, 0x80FFFF03, 0x00000100 },
	{ &GUID_POV, 32, 0x80FFFF10, 0x00000000 },
	{ &GUID_POV, 36, 0x80FFFF10, 0x00000000 },
	{ &GUID_POV, 40, 0x80FFFF10, 0x00000000 },
	{ &GUID_POV, 44, 0x80FFFF10, 0x00000000 },
	{ NULL, 48, 0x80FFFF0C, 0x00000000 },
	{ NULL, 49, 0x80FFFF0C, 0x00000000 },
	{ NULL, 50, 0x80FFFF0C, 0x00000000 },
	{ NULL, 51, 0x80FFFF0C, 0x00000000 },
	{ NULL, 52, 0x80FFFF0C, 0x00000000 },
	{ NULL, 53, 0x80FFFF0C, 0x00000000 },
	{ NULL, 54, 0x80FFFF0C, 0x00000000 },
	{ NULL, 55, 0x80FFFF0C, 0x00000000 },
	{ NULL, 56, 0x80FFFF0C, 0x00000000 },
	{ NULL, 57, 0x80FFFF0C, 0x00000000 },
	{ NULL, 58, 0x80FFFF0C, 0x00000000 },
	{ NULL, 59, 0x80FFFF0C, 0x00000000 },
	{ NULL, 60, 0x80FFFF0C, 0x00000000 },
	{ NULL, 61, 0x80FFFF0C, 0x00000000 },
	{ NULL, 62, 0x80FFFF0C, 0x00000000 },
	{ NULL, 63, 0x80FFFF0C, 0x00000000 },
	{ NULL, 64, 0x80FFFF0C, 0x00000000 },
	{ NULL, 65, 0x80FFFF0C, 0x00000000 },
	{ NULL, 66, 0x80FFFF0C, 0x00000000 },
	{ NULL, 67, 0x80FFFF0C, 0x00000000 },
	{ NULL, 68, 0x80FFFF0C, 0x00000000 },
	{ NULL, 69, 0x80FFFF0C, 0x00000000 },
	{ NULL, 70, 0x80FFFF0C, 0x00000000 },
	{ NULL, 71, 0x80FFFF0C, 0x00000000 },
	{ NULL, 72, 0x80FFFF0C, 0x00000000 },
	{ NULL, 73, 0x80FFFF0C, 0x00000000 },
	{ NULL, 74, 0x80FFFF0C, 0x00000000 },
	{ NULL, 75, 0x80FFFF0C, 0x00000000 },
	{ NULL, 76, 0x80FFFF0C, 0x00000000 },
	{ NULL, 77, 0x80FFFF0C, 0x00000000 },
	{ NULL, 78, 0x80FFFF0C, 0x00000000 },
	{ NULL, 79, 0x80FFFF0C, 0x00000000 },
};

const DIDATAFORMAT c_dfDIJoystick = { sizeof(DIDATAFORMAT), sizeof(DIOBJECTDATAFORMAT), 0x00000001, 80, 44, JOY_fmt };


/* Initialization/Query functions */
static int DX5_VideoInit(_THIS, SDL_PixelFormat *vformat);
static SDL_Rect **DX5_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags);
static SDL_Surface *DX5_SetVideoMode(_THIS, SDL_Surface *current, int width, int height, int bpp, Uint32 flags);
static int DX5_SetColors(_THIS, int firstcolor, int ncolors,
			 SDL_Color *colors);
static int DX5_SetGammaRamp(_THIS, Uint16 *ramp);
static int DX5_GetGammaRamp(_THIS, Uint16 *ramp);
static void DX5_VideoQuit(_THIS);

/* Hardware surface functions */
static int DX5_AllocHWSurface(_THIS, SDL_Surface *surface);
static int DX5_CheckHWBlit(_THIS, SDL_Surface *src, SDL_Surface *dst);
static int DX5_FillHWRect(_THIS, SDL_Surface *dst, SDL_Rect *dstrect, Uint32 color);
static int DX5_SetHWColorKey(_THIS, SDL_Surface *surface, Uint32 key);
static int DX5_SetHWAlpha(_THIS, SDL_Surface *surface, Uint8 alpha);
static int DX5_LockHWSurface(_THIS, SDL_Surface *surface);
static void DX5_UnlockHWSurface(_THIS, SDL_Surface *surface);
static int DX5_FlipHWSurface(_THIS, SDL_Surface *surface);
static void DX5_FreeHWSurface(_THIS, SDL_Surface *surface);

static int DX5_AllocDDSurface(_THIS, SDL_Surface *surface, 
				LPDIRECTDRAWSURFACE3 requested, Uint32 flag);

/* Windows message handling functions */
static void DX5_Activate(_THIS, BOOL active, BOOL minimized);
static void DX5_RealizePalette(_THIS);
static void DX5_PaletteChanged(_THIS, HWND window);
static void DX5_WinPAINT(_THIS, HDC hdc);

/* WinDIB driver functions for manipulating gamma ramps */
extern int DIB_SetGammaRamp(_THIS, Uint16 *ramp);
extern int DIB_GetGammaRamp(_THIS, Uint16 *ramp);
extern void DIB_QuitGamma(_THIS);

/* DX5 driver bootstrap functions */

static int DX5_Available(void)
{
	HINSTANCE DInputDLL;
	HINSTANCE DDrawDLL;
	int dinput_ok;
	int ddraw_ok;

	/* Version check DINPUT.DLL and DDRAW.DLL (Is DirectX okay?) */
	dinput_ok = 0;
	DInputDLL = LoadLibrary(TEXT("DINPUT.DLL"));
	if ( DInputDLL != NULL ) {
		dinput_ok = 1;
	  	FreeLibrary(DInputDLL);
	}
	ddraw_ok = 0;
	DDrawDLL = LoadLibrary(TEXT("DDRAW.DLL"));
	if ( DDrawDLL != NULL ) {
	  HRESULT (WINAPI *DDrawCreate)(GUID *,LPDIRECTDRAW *,IUnknown *);
	  LPDIRECTDRAW DDraw;

	  /* Try to create a valid DirectDraw object */
	  DDrawCreate = (void *)GetProcAddress(DDrawDLL, TEXT("DirectDrawCreate"));
	  if ( (DDrawCreate != NULL)
			&& !FAILED(DDrawCreate(NULL, &DDraw, NULL)) ) {
	    if ( !FAILED(IDirectDraw_SetCooperativeLevel(DDraw,
							NULL, DDSCL_NORMAL)) ) {
	      DDSURFACEDESC desc;
	      LPDIRECTDRAWSURFACE  DDrawSurf;
	      LPDIRECTDRAWSURFACE3 DDrawSurf3;

	      /* Try to create a DirectDrawSurface3 object */
	      SDL_memset(&desc, 0, sizeof(desc));
	      desc.dwSize = sizeof(desc);
	      desc.dwFlags = DDSD_CAPS;
	      desc.ddsCaps.dwCaps = DDSCAPS_PRIMARYSURFACE|DDSCAPS_VIDEOMEMORY;
	      if ( !FAILED(IDirectDraw_CreateSurface(DDraw, &desc,
							&DDrawSurf, NULL)) ) {
	        if ( !FAILED(IDirectDrawSurface_QueryInterface(DDrawSurf,
			&IID_IDirectDrawSurface3, (LPVOID *)&DDrawSurf3)) ) {
	          /* Yay! */
		  ddraw_ok = 1;

	          /* Clean up.. */
	          IDirectDrawSurface3_Release(DDrawSurf3);
	        }
	        IDirectDrawSurface_Release(DDrawSurf);
	      }
	    }
	    IDirectDraw_Release(DDraw);
	  }
	  FreeLibrary(DDrawDLL);
	}
	return(dinput_ok && ddraw_ok);
}

/* Functions for loading the DirectX functions dynamically */
static HINSTANCE DDrawDLL = NULL;
static HINSTANCE DInputDLL = NULL;

static void DX5_Unload(void)
{
	if ( DDrawDLL != NULL ) {
		FreeLibrary(DDrawDLL);
		DDrawCreate = NULL;
		DDrawDLL = NULL;
	}
	if ( DInputDLL != NULL ) {
		FreeLibrary(DInputDLL);
		DInputCreate = NULL;
		DInputDLL = NULL;
	}
}
static int DX5_Load(void)
{
	int status;

	DX5_Unload();
	DDrawDLL = LoadLibrary(TEXT("DDRAW.DLL"));
	if ( DDrawDLL != NULL ) {
		DDrawCreate = (void *)GetProcAddress(DDrawDLL,
					TEXT("DirectDrawCreate"));
	}
	DInputDLL = LoadLibrary(TEXT("DINPUT.DLL"));
	if ( DInputDLL != NULL ) {
		DInputCreate = (void *)GetProcAddress(DInputDLL,
					TEXT("DirectInputCreateA"));
	}
	if ( DDrawDLL && DDrawCreate && DInputDLL && DInputCreate ) {
		status = 0;
	} else {
		DX5_Unload();
		status = -1;
	}
	return status;
}

static void DX5_DeleteDevice(SDL_VideoDevice *this)
{
	/* Free DirectDraw object */
	if ( ddraw2 != NULL ) {
		IDirectDraw2_Release(ddraw2);
	}
	DX5_Unload();
	if ( this ) {
		if ( this->hidden ) {
			SDL_free(this->hidden);
		}
		if ( this->gl_data ) {
			SDL_free(this->gl_data);
		}
		SDL_free(this);
	}
}

static SDL_VideoDevice *DX5_CreateDevice(int devindex)
{
	SDL_VideoDevice *device;

	/* Load DirectX */
	if ( DX5_Load() < 0 ) {
		return(NULL);
	}

	/* Initialize all variables that we clean on shutdown */
	device = (SDL_VideoDevice *)SDL_malloc(sizeof(SDL_VideoDevice));
	if ( device ) {
		SDL_memset(device, 0, (sizeof *device));
		device->hidden = (struct SDL_PrivateVideoData *)
				SDL_malloc((sizeof *device->hidden));
		device->gl_data = (struct SDL_PrivateGLData *)
				SDL_malloc((sizeof *device->gl_data));
	}
	if ( (device == NULL) || (device->hidden == NULL) ||
		                 (device->gl_data == NULL) ) {
		SDL_OutOfMemory();
		DX5_DeleteDevice(device);
		return(NULL);
	}
	SDL_memset(device->hidden, 0, (sizeof *device->hidden));
	SDL_memset(device->gl_data, 0, (sizeof *device->gl_data));

	/* Set the function pointers */
	device->VideoInit = DX5_VideoInit;
	device->ListModes = DX5_ListModes;
	device->SetVideoMode = DX5_SetVideoMode;
	device->UpdateMouse = WIN_UpdateMouse;
	device->CreateYUVOverlay = DX5_CreateYUVOverlay;
	device->SetColors = DX5_SetColors;
	device->UpdateRects = NULL;
	device->VideoQuit = DX5_VideoQuit;
	device->AllocHWSurface = DX5_AllocHWSurface;
	device->CheckHWBlit = DX5_CheckHWBlit;
	device->FillHWRect = DX5_FillHWRect;
	device->SetHWColorKey = DX5_SetHWColorKey;
	device->SetHWAlpha = DX5_SetHWAlpha;
	device->LockHWSurface = DX5_LockHWSurface;
	device->UnlockHWSurface = DX5_UnlockHWSurface;
	device->FlipHWSurface = DX5_FlipHWSurface;
	device->FreeHWSurface = DX5_FreeHWSurface;
	device->SetGammaRamp = DX5_SetGammaRamp;
	device->GetGammaRamp = DX5_GetGammaRamp;
#if SDL_VIDEO_OPENGL
	device->GL_LoadLibrary = WIN_GL_LoadLibrary;
	device->GL_GetProcAddress = WIN_GL_GetProcAddress;
	device->GL_GetAttribute = WIN_GL_GetAttribute;
	device->GL_MakeCurrent = WIN_GL_MakeCurrent;
	device->GL_SwapBuffers = WIN_GL_SwapBuffers;
#endif
	device->SetCaption = WIN_SetWMCaption;
	device->SetIcon = WIN_SetWMIcon;
	device->IconifyWindow = WIN_IconifyWindow;
	device->GrabInput = WIN_GrabInput;
	device->GetWMInfo = WIN_GetWMInfo;
	device->FreeWMCursor = WIN_FreeWMCursor;
	device->CreateWMCursor = WIN_CreateWMCursor;
	device->ShowWMCursor = WIN_ShowWMCursor;
	device->WarpWMCursor = WIN_WarpWMCursor;
	device->CheckMouseMode = WIN_CheckMouseMode;
	device->InitOSKeymap = DX5_InitOSKeymap;
	device->PumpEvents = DX5_PumpEvents;

	/* Set up the windows message handling functions */
	WIN_Activate = DX5_Activate;
	WIN_RealizePalette = DX5_RealizePalette;
	WIN_PaletteChanged = DX5_PaletteChanged;
	WIN_WinPAINT = DX5_WinPAINT;
	HandleMessage = DX5_HandleMessage;

	device->free = DX5_DeleteDevice;

	/* We're finally ready */
	return device;
}

VideoBootStrap DIRECTX_bootstrap = {
	"directx", "Win95/98/2000 DirectX",
	DX5_Available, DX5_CreateDevice
};

static int cmpmodes(const void *va, const void *vb)
{
    SDL_Rect *a = *(SDL_Rect **)va;
    SDL_Rect *b = *(SDL_Rect **)vb;
    if ( a->w == b->w )
        return b->h - a->h;
    else
        return b->w - a->w;
}

static HRESULT WINAPI EnumModes2(DDSURFACEDESC *desc, VOID *udata)
{
	SDL_VideoDevice *this = (SDL_VideoDevice *)udata;
	struct DX5EnumRect *enumrect;
#if defined(NONAMELESSUNION)
	int bpp = desc->ddpfPixelFormat.u1.dwRGBBitCount;
	int refreshRate = desc->u2.dwRefreshRate;
#else
	int bpp = desc->ddpfPixelFormat.dwRGBBitCount;
	int refreshRate = desc->dwRefreshRate;
#endif
	int maxRefreshRate;

	if ( desc->dwWidth <= SDL_desktop_mode.dmPelsWidth &&
	     desc->dwHeight <= SDL_desktop_mode.dmPelsHeight ) {
		maxRefreshRate = SDL_desktop_mode.dmDisplayFrequency;
	} else {
		maxRefreshRate = 85;	/* safe value? */
	}

	switch (bpp)  {
		case 8:
		case 16:
		case 24:
		case 32:
			bpp /= 8; --bpp;
			if ( enumlists[bpp] &&
			     enumlists[bpp]->r.w == (Uint16)desc->dwWidth &&
			     enumlists[bpp]->r.h == (Uint16)desc->dwHeight ) {
				if ( refreshRate > enumlists[bpp]->refreshRate &&
				     refreshRate <= maxRefreshRate ) {
					enumlists[bpp]->refreshRate = refreshRate;
#ifdef DDRAW_DEBUG
 fprintf(stderr, "New refresh rate for %d bpp: %dx%d at %d Hz\n", (bpp+1)*8, (int)desc->dwWidth, (int)desc->dwHeight, refreshRate);
#endif
				}
				break;
			}
			++SDL_nummodes[bpp];
			enumrect = (struct DX5EnumRect*)SDL_malloc(sizeof(struct DX5EnumRect));
			if ( !enumrect ) {
				SDL_OutOfMemory();
				return(DDENUMRET_CANCEL);
			}
			enumrect->refreshRate = refreshRate;
			enumrect->r.x = 0;
			enumrect->r.y = 0;
			enumrect->r.w = (Uint16)desc->dwWidth;
			enumrect->r.h = (Uint16)desc->dwHeight;
			enumrect->next = enumlists[bpp];
			enumlists[bpp] = enumrect;
#ifdef DDRAW_DEBUG
 fprintf(stderr, "New mode for %d bpp: %dx%d at %d Hz\n", (bpp+1)*8, (int)desc->dwWidth, (int)desc->dwHeight, refreshRate);
#endif
			break;
	}

	return(DDENUMRET_OK);
}

void SetDDerror(const char *function, int code)
{
	static char *error;
	static char  errbuf[1024];

	errbuf[0] = 0;
	switch (code) {
		case DDERR_GENERIC:
			error = "Undefined error!";
			break;
		case DDERR_EXCEPTION:
			error = "Exception encountered";
			break;
		case DDERR_INVALIDOBJECT:
			error = "Invalid object";
			break;
		case DDERR_INVALIDPARAMS:
			error = "Invalid parameters";
			break;
		case DDERR_NOTFOUND:
			error = "Object not found";
			break;
		case DDERR_INVALIDRECT:
			error = "Invalid rectangle";
			break;
		case DDERR_INVALIDCAPS:
			error = "Invalid caps member";
			break;
		case DDERR_INVALIDPIXELFORMAT:
			error = "Invalid pixel format";
			break;
		case DDERR_OUTOFMEMORY:
			error = "Out of memory";
			break;
		case DDERR_OUTOFVIDEOMEMORY:
			error = "Out of video memory";
			break;
		case DDERR_SURFACEBUSY:
			error = "Surface busy";
			break;
		case DDERR_SURFACELOST:
			error = "Surface was lost";
			break;
		case DDERR_WASSTILLDRAWING:
			error = "DirectDraw is still drawing";
			break;
		case DDERR_INVALIDSURFACETYPE:
			error = "Invalid surface type";
			break;
		case DDERR_NOEXCLUSIVEMODE:
			error = "Not in exclusive access mode";
			break;
		case DDERR_NOPALETTEATTACHED:
			error = "No palette attached";
			break;
		case DDERR_NOPALETTEHW:
			error = "No palette hardware";
			break;
		case DDERR_NOT8BITCOLOR:
			error = "Not 8-bit color";
			break;
		case DDERR_EXCLUSIVEMODEALREADYSET:
			error = "Exclusive mode was already set";
			break;
		case DDERR_HWNDALREADYSET:
			error = "Window handle already set";
			break;
		case DDERR_HWNDSUBCLASSED:
			error = "Window handle is subclassed";
			break;
		case DDERR_NOBLTHW:
			error = "No blit hardware";
			break;
		case DDERR_IMPLICITLYCREATED:
			error = "Surface was implicitly created";
			break;
		case DDERR_INCOMPATIBLEPRIMARY:
			error = "Incompatible primary surface";
			break;
		case DDERR_NOCOOPERATIVELEVELSET:
			error = "No cooperative level set";
			break;
		case DDERR_NODIRECTDRAWHW:
			error = "No DirectDraw hardware";
			break;
		case DDERR_NOEMULATION:
			error = "No emulation available";
			break;
		case DDERR_NOFLIPHW:
			error = "No flip hardware";
			break;
		case DDERR_NOTFLIPPABLE:
			error = "Surface not flippable";
			break;
		case DDERR_PRIMARYSURFACEALREADYEXISTS:
			error = "Primary surface already exists";
			break;
		case DDERR_UNSUPPORTEDMODE:
			error = "Unsupported mode";
			break;
		case DDERR_WRONGMODE:
			error = "Surface created in different mode";
			break;
		case DDERR_UNSUPPORTED:
			error = "Operation not supported";
			break;
		case E_NOINTERFACE:
			error = "Interface not present";
			break;
		default:
			SDL_snprintf(errbuf, SDL_arraysize(errbuf),
			         "%s: Unknown DirectDraw error: 0x%x",
								function, code);
			break;
	}
	if ( ! errbuf[0] ) {
		SDL_snprintf(errbuf, SDL_arraysize(errbuf), "%s: %s", function, error);
	}
	SDL_SetError("%s", errbuf);
	return;
}


static int DX5_UpdateVideoInfo(_THIS)
{
	/* This needs to be DDCAPS_DX5 for the DirectDraw2 interface */
#if DIRECTDRAW_VERSION <= 0x300
#error Your version of DirectX must be greater than or equal to 5.0
#endif
#ifndef IDirectDrawGammaControl_SetGammaRamp
	/*if gamma is undefined then we really have directx <= 0x500*/
	DDCAPS DDCaps;
#else
	DDCAPS_DX5 DDCaps;
#endif
	HRESULT result;

	/* Fill in our hardware acceleration capabilities */
	SDL_memset(&DDCaps, 0, sizeof(DDCaps));
	DDCaps.dwSize = sizeof(DDCaps);
	result = IDirectDraw2_GetCaps(ddraw2, (DDCAPS *)&DDCaps, NULL);
	if ( result != DD_OK ) {
		SetDDerror("DirectDraw2::GetCaps", result);
		return(-1);
	}
	this->info.hw_available = 1;
	if ( (DDCaps.dwCaps & DDCAPS_BLT) == DDCAPS_BLT ) {
		this->info.blit_hw = 1;
	}
	if ( ((DDCaps.dwCaps & DDCAPS_COLORKEY) == DDCAPS_COLORKEY) &&
	     ((DDCaps.dwCKeyCaps & DDCKEYCAPS_SRCBLT) == DDCKEYCAPS_SRCBLT) ) {
		this->info.blit_hw_CC = 1;
	}
	if ( (DDCaps.dwCaps & DDCAPS_ALPHA) == DDCAPS_ALPHA ) {
		/* This is only for alpha channel, and DirectX 6
		   doesn't support 2D alpha blits yet, so set it 0
		 */
		this->info.blit_hw_A = 0;
	}
	if ( (DDCaps.dwCaps & DDCAPS_CANBLTSYSMEM) == DDCAPS_CANBLTSYSMEM ) {
		this->info.blit_sw = 1;
		/* This isn't necessarily true, but the HEL will cover us */
		this->info.blit_sw_CC = this->info.blit_hw_CC;
		this->info.blit_sw_A = this->info.blit_hw_A;
	}
	if ( (DDCaps.dwCaps & DDCAPS_BLTCOLORFILL) == DDCAPS_BLTCOLORFILL ) {
		this->info.blit_fill = 1;
	}

	/* Find out how much video memory is available */
	{ DDSCAPS ddsCaps;
	  DWORD total_mem;
		ddsCaps.dwCaps = DDSCAPS_VIDEOMEMORY;
		result = IDirectDraw2_GetAvailableVidMem(ddraw2,
						&ddsCaps, &total_mem, NULL);
		if ( result != DD_OK ) {
			total_mem = DDCaps.dwVidMemTotal; 
		}
		this->info.video_mem = total_mem/1024;
	}
	return(0);
}

int DX5_VideoInit(_THIS, SDL_PixelFormat *vformat)
{
	HRESULT result;
	LPDIRECTDRAW ddraw;
	int i, j;
	HDC hdc;

	/* Intialize everything */
	ddraw2 = NULL;
	SDL_primary = NULL;
	SDL_clipper = NULL;
	SDL_palette = NULL;
	for ( i=0; i<NUM_MODELISTS; ++i ) {
		SDL_nummodes[i] = 0;
		SDL_modelist[i] = NULL;
		SDL_modeindex[i] = 0;
	}
	colorchange_expected = 0;

	/* Create the window */
	if ( DX5_CreateWindow(this) < 0 ) {
		return(-1);
	}

#if !SDL_AUDIO_DISABLED
	DX5_SoundFocus(SDL_Window);
#endif

	/* Create the DirectDraw object */
	result = DDrawCreate(NULL, &ddraw, NULL);
	if ( result != DD_OK ) {
		SetDDerror("DirectDrawCreate", result);
		return(-1);
	}
	result = IDirectDraw_QueryInterface(ddraw, &IID_IDirectDraw2,
							(LPVOID *)&ddraw2);
	IDirectDraw_Release(ddraw);
	if ( result != DD_OK ) {
		SetDDerror("DirectDraw::QueryInterface", result);
		return(-1);
	}

	/* Determine the screen depth */
	hdc = GetDC(SDL_Window);
	vformat->BitsPerPixel = GetDeviceCaps(hdc,PLANES) *
					GetDeviceCaps(hdc,BITSPIXEL);
	ReleaseDC(SDL_Window, hdc);

#ifndef NO_CHANGEDISPLAYSETTINGS
	/* Query for the desktop resolution */
	EnumDisplaySettings(NULL, ENUM_CURRENT_SETTINGS, &SDL_desktop_mode);
	this->info.current_w = SDL_desktop_mode.dmPelsWidth;
	this->info.current_h = SDL_desktop_mode.dmPelsHeight;
#endif

	/* Enumerate the available fullscreen modes */
	for ( i=0; i<NUM_MODELISTS; ++i )
		enumlists[i] = NULL;

	result = IDirectDraw2_EnumDisplayModes(ddraw2,DDEDM_REFRESHRATES,NULL,this,EnumModes2);
	if ( result != DD_OK ) {
		SetDDerror("DirectDraw2::EnumDisplayModes", result);
		return(-1);
	}
	for ( i=0; i<NUM_MODELISTS; ++i ) {
		struct DX5EnumRect *rect;
		SDL_modelist[i] = (SDL_Rect **)
				SDL_malloc((SDL_nummodes[i]+1)*sizeof(SDL_Rect *));
		if ( SDL_modelist[i] == NULL ) {
			SDL_OutOfMemory();
			return(-1);
		}
		for ( j = 0, rect = enumlists[i]; rect; ++j, rect = rect->next ) {
			SDL_modelist[i][j] = &rect->r;
		}
		SDL_modelist[i][j] = NULL;

		if ( SDL_nummodes[i] > 0 ) {
			SDL_qsort(SDL_modelist[i], SDL_nummodes[i], sizeof *SDL_modelist[i], cmpmodes);
		}
	}
	
	/* Fill in some window manager capabilities */
	this->info.wm_available = 1;

	/* Fill in the video hardware capabilities */
	DX5_UpdateVideoInfo(this);

	return(0);
}

SDL_Rect **DX5_ListModes(_THIS, SDL_PixelFormat *format, Uint32 flags)
{
	int bpp;

	bpp = format->BitsPerPixel;
	if ( (flags & SDL_FULLSCREEN) == SDL_FULLSCREEN ) {
		/* FIXME:  No support for 1 bpp or 4 bpp formats */
		switch (bpp) {  /* Does windows support other BPP? */
			case 8:
			case 16:
			case 24:
			case 32:
				bpp = (bpp/8)-1;
				if ( SDL_nummodes[bpp] > 0 )
					return(SDL_modelist[bpp]);
				/* Fall through */
			default:
				return((SDL_Rect **)0);
		}
	} else {
		if ( this->screen->format->BitsPerPixel == bpp ) {
			return((SDL_Rect **)-1);
		} else {
			return((SDL_Rect **)0);
		}
	}
}

/* Various screen update functions available */
static void DX5_WindowUpdate(_THIS, int numrects, SDL_Rect *rects);
static void DX5_DirectUpdate(_THIS, int numrects, SDL_Rect *rects);

SDL_Surface *DX5_SetVideoMode(_THIS, SDL_Surface *current,
				int width, int height, int bpp, Uint32 flags)
{
	SDL_Surface *video;
	int prev_w = -1;
	int prev_h = -1;
	HRESULT result;
	DWORD sharemode;
	DWORD style;
	const DWORD directstyle =
			(WS_POPUP);
	const DWORD windowstyle = 
			(WS_OVERLAPPED|WS_CAPTION|WS_SYSMENU|WS_MINIMIZEBOX);
	const DWORD resizestyle =
			(WS_THICKFRAME|WS_MAXIMIZEBOX);
	DDSURFACEDESC ddsd;
	LPDIRECTDRAWSURFACE  dd_surface1;
	LPDIRECTDRAWSURFACE3 dd_surface3;

	SDL_resizing = 1;
#ifdef DDRAW_DEBUG
 fprintf(stderr, "Setting %dx%dx%d video mode\n", width, height, bpp);
#endif
	/* Clean up any previous DirectDraw surfaces */
	if ( current->hwdata ) {
		this->FreeHWSurface(this, current);
		current->hwdata = NULL;
	}
	if ( SDL_primary != NULL ) {
		IDirectDrawSurface3_Release(SDL_primary);
		SDL_primary = NULL;
	}

#ifndef NO_CHANGEDISPLAYSETTINGS
	/* Unset any previous OpenGL fullscreen mode */
	if ( (current->flags & (SDL_OPENGL|SDL_FULLSCREEN)) ==
	                       (SDL_OPENGL|SDL_FULLSCREEN) ) {
		ChangeDisplaySettings(NULL, 0);
	}
#endif

	/* Clean up any GL context that may be hanging around */
	if ( current->flags & SDL_OPENGL ) {
		WIN_GL_ShutDown(this);
	}

	/* If we are setting a GL mode, use GDI, not DirectX (yuck) */
	if ( flags & SDL_OPENGL ) {
		Uint32 Rmask, Gmask, Bmask;

		/* Recalculate the bitmasks if necessary */
		if ( bpp == current->format->BitsPerPixel ) {
			video = current;
		} else {
			switch (bpp) {
			    case 15:
			    case 16:
				if ( 0 /*DIB_SussScreenDepth() == 15*/ ) {
					/* 5-5-5 */
					Rmask = 0x00007c00;
					Gmask = 0x000003e0;
					Bmask = 0x0000001f;
				} else {
					/* 5-6-5 */
					Rmask = 0x0000f800;
					Gmask = 0x000007e0;
					Bmask = 0x0000001f;
				}
				break;
			    case 24:
			    case 32:
				/* GDI defined as 8-8-8 */
				Rmask = 0x00ff0000;
				Gmask = 0x0000ff00;
				Bmask = 0x000000ff;
				break;
			    default:
				Rmask = 0x00000000;
				Gmask = 0x00000000;
				Bmask = 0x00000000;
				break;
			}
			video = SDL_CreateRGBSurface(SDL_SWSURFACE, 0, 0, bpp,
			                             Rmask, Gmask, Bmask, 0);
			if ( video == NULL ) {
				SDL_OutOfMemory();
				return(NULL);
			}
		}

		/* Fill in part of the video surface */
		prev_w = video->w;
		prev_h = video->h;
		video->flags = 0;	/* Clear flags */
		video->w = width;
		video->h = height;
		video->pitch = SDL_CalculatePitch(video);

#ifndef NO_CHANGEDISPLAYSETTINGS
		/* Set fullscreen mode if appropriate.
		   Ugh, since our list of valid video modes comes from
		   the DirectX driver, we may not actually be able to
		   change to the desired resolution here.
		   FIXME: Should we do a closest match?
		 */
		if ( (flags & SDL_FULLSCREEN) == SDL_FULLSCREEN ) {
			DEVMODE settings;
			BOOL changed;

			SDL_memset(&settings, 0, sizeof(DEVMODE));
			settings.dmSize = sizeof(DEVMODE);
			settings.dmBitsPerPel = video->format->BitsPerPixel;
			settings.dmPelsWidth = width;
			settings.dmPelsHeight = height;
			settings.dmFields = DM_PELSWIDTH | DM_PELSHEIGHT | DM_BITSPERPEL;
			if ( width <= (int)SDL_desktop_mode.dmPelsWidth &&
			     height <= (int)SDL_desktop_mode.dmPelsHeight ) {
				settings.dmDisplayFrequency = SDL_desktop_mode.dmDisplayFrequency;
				settings.dmFields |= DM_DISPLAYFREQUENCY;
			}
			changed = (ChangeDisplaySettings(&settings, CDS_FULLSCREEN) == DISP_CHANGE_SUCCESSFUL);
			if ( ! changed && (settings.dmFields & DM_DISPLAYFREQUENCY) ) {
				settings.dmFields &= ~DM_DISPLAYFREQUENCY;
				changed = (ChangeDisplaySettings(&settings, CDS_FULLSCREEN) == DISP_CHANGE_SUCCESSFUL);
			}
			if ( changed ) {
				video->flags |= SDL_FULLSCREEN;
				SDL_fullscreen_mode = settings;
			}
		}
#endif /* !NO_CHANGEDISPLAYSETTINGS */

		style = GetWindowLong(SDL_Window, GWL_STYLE);
		style &= ~(resizestyle|WS_MAXIMIZE);
		if ( video->flags & SDL_FULLSCREEN ) {
			style &= ~windowstyle;
			style |= directstyle;
		} else {
			if ( flags & SDL_NOFRAME ) {
				style &= ~windowstyle;
				style |= directstyle;
				video->flags |= SDL_NOFRAME;
			} else {
				style &= ~directstyle;
				style |= windowstyle;
				if ( flags & SDL_RESIZABLE ) {
					style |= resizestyle;
					video->flags |= SDL_RESIZABLE;
				}
			}
#if WS_MAXIMIZE
			if (IsZoomed(SDL_Window)) style |= WS_MAXIMIZE;
#endif
		}

		/* DJM: Don't piss of anyone who has setup his own window */
		if ( !SDL_windowid )
			SetWindowLong(SDL_Window, GWL_STYLE, style);

		/* Resize the window (copied from SDL WinDIB driver) */
		if ( !SDL_windowid && !IsZoomed(SDL_Window) ) {
			RECT bounds;
			int x, y;
			HWND top;
			UINT swp_flags;
			const char *window = NULL;
			const char *center = NULL;

			if ( video->w != prev_w || video->h != prev_h ) {
				window = SDL_getenv("SDL_VIDEO_WINDOW_POS");
				center = SDL_getenv("SDL_VIDEO_CENTERED");
				if ( window ) {
					if ( SDL_sscanf(window, "%d,%d", &x, &y) == 2 ) {
						SDL_windowX = x;
						SDL_windowY = y;
					}
					if ( SDL_strcmp(window, "center") == 0 ) {
						center = window;
					}
				}
			}
			swp_flags = (SWP_NOCOPYBITS | SWP_SHOWWINDOW);

			bounds.left = SDL_windowX;
			bounds.top = SDL_windowY;
			bounds.right = SDL_windowX+video->w;
			bounds.bottom = SDL_windowY+video->h;
			AdjustWindowRectEx(&bounds, GetWindowLong(SDL_Window, GWL_STYLE), (GetMenu(SDL_Window) != NULL), 0);
			width = bounds.right-bounds.left;
			height = bounds.bottom-bounds.top;
			if ( (flags & SDL_FULLSCREEN) ) {
				x = (GetSystemMetrics(SM_CXSCREEN)-width)/2;
				y = (GetSystemMetrics(SM_CYSCREEN)-height)/2;
			} else if ( center ) {
				x = (GetSystemMetrics(SM_CXSCREEN)-width)/2;
				y = (GetSystemMetrics(SM_CYSCREEN)-height)/2;
			} else if ( SDL_windowX || SDL_windowY || window ) {
				x = bounds.left;
				y = bounds.top;
			} else {
				x = y = -1;
				swp_flags |= SWP_NOMOVE;
			}
			if ( flags & SDL_FULLSCREEN ) {
				top = HWND_TOPMOST;
			} else {
				top = HWND_NOTOPMOST;
			}
			SetWindowPos(SDL_Window, top, x, y, width, height, swp_flags);
			if ( !(flags & SDL_FULLSCREEN) ) {
				SDL_windowX = SDL_bounds.left;
				SDL_windowY = SDL_bounds.top;
			}
			SetForegroundWindow(SDL_Window);
		}
		SDL_resizing = 0;

		/* Set up for OpenGL */
		if ( WIN_GL_SetupWindow(this) < 0 ) {
			return(NULL);
		}
		video->flags |= SDL_OPENGL;
		return(video);
	}

	/* Set the appropriate window style */
	style = GetWindowLong(SDL_Window, GWL_STYLE);
	style &= ~(resizestyle|WS_MAXIMIZE);
	if ( (flags & SDL_FULLSCREEN) == SDL_FULLSCREEN ) {
		style &= ~windowstyle;
		style |= directstyle;
	} else {
		if ( flags & SDL_NOFRAME ) {
			style &= ~windowstyle;
			style |= directstyle;
		} else {
			style &= ~directstyle;
			style |= windowstyle;
			if ( flags & SDL_RESIZABLE ) {
				style |= resizestyle;
			}
		}
#if WS_MAXIMIZE
		if (IsZoomed(SDL_Window)) style |= WS_MAXIMIZE;
#endif
	}
	/* DJM: Don't piss of anyone who has setup his own window */
	if ( !SDL_windowid )
		SetWindowLong(SDL_Window, GWL_STYLE, style);

	/* Set DirectDraw sharing mode.. exclusive when fullscreen */
	if ( (flags & SDL_FULLSCREEN) == SDL_FULLSCREEN ) {
		sharemode = DDSCL_FULLSCREEN|DDSCL_EXCLUSIVE|DDSCL_ALLOWREBOOT;
	} else {
		sharemode = DDSCL_NORMAL;
	}
	result = IDirectDraw2_SetCooperativeLevel(ddraw2,SDL_Window,sharemode);
	if ( result != DD_OK ) {
		SetDDerror("DirectDraw2::SetCooperativeLevel", result);
		return(NULL);
	}

	/* Set the display mode, if we are in fullscreen mode */
	if ( (flags & SDL_FULLSCREEN) == SDL_FULLSCREEN ) {
		RECT bounds;
		struct DX5EnumRect *rect;
		int maxRefreshRate;

		/* Cover up desktop during mode change */
		bounds.left = 0;
		bounds.top = 0;
		bounds.right = GetSystemMetrics(SM_CXSCREEN);
		bounds.bottom = GetSystemMetrics(SM_CYSCREEN);
		AdjustWindowRectEx(&bounds, GetWindowLong(SDL_Window, GWL_STYLE), (GetMenu(SDL_Window) != NULL), 0);
		SetWindowPos(SDL_Window, HWND_TOPMOST,
			bounds.left, bounds.top, 
			bounds.right - bounds.left,
			bounds.bottom - bounds.top, SWP_NOCOPYBITS);
		ShowWindow(SDL_Window, SW_SHOW);
		while ( GetForegroundWindow() != SDL_Window ) {
			SetForegroundWindow(SDL_Window);
			SDL_Delay(100);
		}

		/* find maximum monitor refresh rate for this resolution */
		/* Dmitry Yakimov ftech@tula.net */
		maxRefreshRate = 0; /* system default */
		for ( rect = enumlists[bpp / 8 - 1]; rect; rect = rect->next ) {
			if ( (width == rect->r.w) && (height == rect->r.h) ) {
				maxRefreshRate = rect->refreshRate;
				break;
			}
		}
#ifdef DDRAW_DEBUG
 fprintf(stderr, "refresh rate = %d Hz\n", maxRefreshRate);
#endif

		result = IDirectDraw2_SetDisplayMode(ddraw2, width, height, bpp, maxRefreshRate, 0);
		if ( result != DD_OK ) {
			result = IDirectDraw2_SetDisplayMode(ddraw2, width, height, bpp, 0, 0);
			if ( result != DD_OK ) {
				/* We couldn't set fullscreen mode, try window */
				return(DX5_SetVideoMode(this, current, width, height, bpp, flags & ~SDL_FULLSCREEN)); 
			}
		}
		DX5_DInputReset(this, 1);
	} else {
		DX5_DInputReset(this, 0);
	}
	DX5_UpdateVideoInfo(this);

	/* Create a primary DirectDraw surface */
	SDL_memset(&ddsd, 0, sizeof(ddsd));
	ddsd.dwSize = sizeof(ddsd);
	ddsd.dwFlags = DDSD_CAPS;
	ddsd.ddsCaps.dwCaps = (DDSCAPS_PRIMARYSURFACE|DDSCAPS_VIDEOMEMORY);
	if ( (flags & SDL_FULLSCREEN) != SDL_FULLSCREEN ) {
		/* There's no windowed double-buffering */
		flags &= ~SDL_DOUBLEBUF;
	}
	if ( (flags & SDL_DOUBLEBUF) == SDL_DOUBLEBUF ) {
		ddsd.dwFlags |= DDSD_BACKBUFFERCOUNT;
		ddsd.ddsCaps.dwCaps |= (DDSCAPS_COMPLEX|DDSCAPS_FLIP);
		ddsd.dwBackBufferCount = 1;
	}
	result = IDirectDraw2_CreateSurface(ddraw2, &ddsd, &dd_surface1, NULL); 
	if ( (result != DD_OK) && ((flags & SDL_DOUBLEBUF) == SDL_DOUBLEBUF) ) {
		ddsd.dwFlags &= ~DDSD_BACKBUFFERCOUNT;
		ddsd.ddsCaps.dwCaps &= ~(DDSCAPS_COMPLEX|DDSCAPS_FLIP);
		ddsd.dwBackBufferCount = 0;
		result = IDirectDraw2_CreateSurface(ddraw2,
						&ddsd, &dd_surface1, NULL); 
	}
	if ( result != DD_OK ) {
		SetDDerror("DirectDraw2::CreateSurface(PRIMARY)", result);
		return(NULL);
	}
	result = IDirectDrawSurface_QueryInterface(dd_surface1,
			&IID_IDirectDrawSurface3, (LPVOID *)&SDL_primary);
	if ( result != DD_OK ) {
		SetDDerror("DirectDrawSurface::QueryInterface", result);
		return(NULL);
	}
	IDirectDrawSurface_Release(dd_surface1);

	/* Get the format of the primary DirectDraw surface */
	SDL_memset(&ddsd, 0, sizeof(ddsd));
	ddsd.dwSize = sizeof(ddsd);
	ddsd.dwFlags = DDSD_PIXELFORMAT|DDSD_CAPS;
	result = IDirectDrawSurface3_GetSurfaceDesc(SDL_primary, &ddsd);
	if ( result != DD_OK ) {
		SetDDerror("DirectDrawSurface::GetSurfaceDesc", result);
		return(NULL);
	}
	if ( ! (ddsd.ddpfPixelFormat.dwFlags&DDPF_RGB) ) {
		SDL_SetError("Primary DDRAW surface is not RGB format");
		return(NULL);
	}

	/* Free old palette and create a new one if we're in 8-bit mode */
	if ( SDL_palette != NULL ) {
		IDirectDrawPalette_Release(SDL_palette);
		SDL_palette = NULL;
	}
#if defined(NONAMELESSUNION)
	if ( ddsd.ddpfPixelFormat.u1.dwRGBBitCount == 8 ) {
#else
	if ( ddsd.ddpfPixelFormat.dwRGBBitCount == 8 ) {
#endif
		int i;

		if ( (flags & SDL_FULLSCREEN) == SDL_FULLSCREEN ) {
			/* We have access to the entire palette */
			for ( i=0; i<256; ++i ) {
				SDL_colors[i].peFlags =
						(PC_NOCOLLAPSE|PC_RESERVED);
				SDL_colors[i].peRed = 0;
				SDL_colors[i].peGreen = 0;
				SDL_colors[i].peBlue = 0;
			}
		} else {
			/* First 10 colors are reserved by Windows */
			for ( i=0; i<10; ++i ) {
				SDL_colors[i].peFlags = PC_EXPLICIT;
				SDL_colors[i].peRed = i;
				SDL_colors[i].peGreen = 0;
				SDL_colors[i].peBlue = 0;
			}
			for ( i=10; i<(10+236); ++i ) {
				SDL_colors[i].peFlags = PC_NOCOLLAPSE;
				SDL_colors[i].peRed = 0;
				SDL_colors[i].peGreen = 0;
				SDL_colors[i].peBlue = 0;
			}
			/* Last 10 colors are reserved by Windows */
			for ( i=246; i<256; ++i ) {
				SDL_colors[i].peFlags = PC_EXPLICIT;
				SDL_colors[i].peRed = i;
				SDL_colors[i].peGreen = 0;
				SDL_colors[i].peBlue = 0;
			}
		}
		result = IDirectDraw2_CreatePalette(ddraw2,
		     			(DDPCAPS_8BIT|DDPCAPS_ALLOW256),
						SDL_colors, &SDL_palette, NULL);
		if ( result != DD_OK ) {
			SetDDerror("DirectDraw2::CreatePalette", result);
			return(NULL);
		}
		result = IDirectDrawSurface3_SetPalette(SDL_primary,
								SDL_palette);
		if ( result != DD_OK ) {
			SetDDerror("DirectDrawSurface3::SetPalette", result);
			return(NULL);
		}
	}

	/* Create our video surface using the same pixel format */
	video = current;
	if ( (width != video->w) || (height != video->h)
			|| (video->format->BitsPerPixel != 
#if defined(NONAMELESSUNION)
				ddsd.ddpfPixelFormat.u1.dwRGBBitCount) ) {
#else
				ddsd.ddpfPixelFormat.dwRGBBitCount) ) {
#endif
		SDL_FreeSurface(video);
		video = SDL_CreateRGBSurface(SDL_SWSURFACE, 0, 0,
#if defined(NONAMELESSUNION)
				ddsd.ddpfPixelFormat.u1.dwRGBBitCount,
					ddsd.ddpfPixelFormat.u2.dwRBitMask,
					ddsd.ddpfPixelFormat.u3.dwGBitMask,
					ddsd.ddpfPixelFormat.u4.dwBBitMask,
#else
				ddsd.ddpfPixelFormat.dwRGBBitCount,
					ddsd.ddpfPixelFormat.dwRBitMask,
					ddsd.ddpfPixelFormat.dwGBitMask,
					ddsd.ddpfPixelFormat.dwBBitMask,
#endif
								0);
		if ( video == NULL ) {
			SDL_OutOfMemory();
			return(NULL);
		}
		prev_w = video->w;
		prev_h = video->h;
		video->w = width;
		video->h = height;
		video->pitch = 0;
	}
	video->flags = 0;	/* Clear flags */

	/* If not fullscreen, locking is possible, but it doesn't do what 
	   the caller really expects -- if the locked surface is written to,
	   the appropriate portion of the entire screen is modified, not 
	   the application window, as we would like.
	   Note that it is still possible to write directly to display
	   memory, but the application must respect the clip list of
	   the surface.  There might be some odd timing interactions
	   involving clip list updates and background refreshing as
	   Windows moves other windows across our window.
	   We currently don't support this, even though it might be a
	   good idea since BeOS has an implementation of BDirectWindow
	   that does the same thing.  This would be most useful for
	   applications that do complete screen updates every frame.
	    -- Fixme?
	*/
	if ( (flags & SDL_FULLSCREEN) != SDL_FULLSCREEN ) {
		/* Necessary if we're going from fullscreen to window */
		if ( video->pixels == NULL ) {
			video->pitch = (width*video->format->BytesPerPixel);
			/* Pitch needs to be QWORD (8-byte) aligned */
			video->pitch = (video->pitch + 7) & ~7;
			video->pixels = (void *)SDL_malloc(video->h*video->pitch);
			if ( video->pixels == NULL ) {
				if ( video != current ) {
					SDL_FreeSurface(video);
				}
				SDL_OutOfMemory();
				return(NULL);
			}
		}
		dd_surface3 = NULL;
#if 0 /* FIXME: enable this when SDL consistently reports lost surfaces */
		if ( (flags & SDL_HWSURFACE) == SDL_HWSURFACE ) {
			video->flags |= SDL_HWSURFACE;
		} else {
			video->flags |= SDL_SWSURFACE;
		}
#else
		video->flags |= SDL_SWSURFACE;
#endif
		if ( (flags & SDL_RESIZABLE) && !(flags & SDL_NOFRAME) ) {
			video->flags |= SDL_RESIZABLE;
		}
		if ( flags & SDL_NOFRAME ) {
			video->flags |= SDL_NOFRAME;
		}
	} else {
		/* Necessary if we're going from window to fullscreen */
		if ( video->pixels != NULL ) {
			SDL_free(video->pixels);
			video->pixels = NULL;
		}
		dd_surface3 = SDL_primary;
		video->flags |= SDL_HWSURFACE;
	}

	/* See if the primary surface has double-buffering enabled */
	if ( (ddsd.ddsCaps.dwCaps & DDSCAPS_FLIP) == DDSCAPS_FLIP ) {
		video->flags |= SDL_DOUBLEBUF;
	}

	/* Allocate the SDL surface associated with the primary surface */
	if ( DX5_AllocDDSurface(this, video, dd_surface3,
	                        video->flags&SDL_HWSURFACE) < 0 ) {
		if ( video != current ) {
			SDL_FreeSurface(video);
		}
		return(NULL);
	}

	/* Use the appropriate blitting function */
	if ( (flags & SDL_FULLSCREEN) == SDL_FULLSCREEN ) {
		video->flags |= SDL_FULLSCREEN;
		if ( video->format->palette != NULL ) {
			video->flags |= SDL_HWPALETTE;
		}
		this->UpdateRects = DX5_DirectUpdate;
	} else {
		this->UpdateRects = DX5_WindowUpdate;
	}

	/* Make our window the proper size, set the clipper, then show it */
	if ( (flags & SDL_FULLSCREEN) != SDL_FULLSCREEN ) {
		/* Create and set a clipper on our primary surface */
		if ( SDL_clipper == NULL ) {
			result = IDirectDraw2_CreateClipper(ddraw2,
							0, &SDL_clipper, NULL);
			if ( result != DD_OK ) {
				if ( video != current ) {
					SDL_FreeSurface(video);
				}
				SetDDerror("DirectDraw2::CreateClipper",result);
				return(NULL);
			}
		}
		result = IDirectDrawClipper_SetHWnd(SDL_clipper, 0, SDL_Window);
		if ( result != DD_OK ) {
			if ( video != current ) {
				SDL_FreeSurface(video);
			}
			SetDDerror("DirectDrawClipper::SetHWnd", result);
			return(NULL);
		}
		result = IDirectDrawSurface3_SetClipper(SDL_primary,
								SDL_clipper);
		if ( result != DD_OK ) {
			if ( video != current ) {
				SDL_FreeSurface(video);
			}
			SetDDerror("DirectDrawSurface3::SetClipper", result);
			return(NULL);
		}

		/* Resize the window (copied from SDL WinDIB driver) */
		if ( !SDL_windowid && !IsZoomed(SDL_Window) ) {
			RECT bounds;
			int  x, y;
			UINT swp_flags;
			const char *window = NULL;
			const char *center = NULL;

			if ( video->w != prev_w || video->h != prev_h ) {
				window = SDL_getenv("SDL_VIDEO_WINDOW_POS");
				center = SDL_getenv("SDL_VIDEO_CENTERED");
				if ( window ) {
					if ( SDL_sscanf(window, "%d,%d", &x, &y) == 2 ) {
						SDL_windowX = x;
						SDL_windowY = y;
					}
					if ( SDL_strcmp(window, "center") == 0 ) {
						center = window;
					}
				}
			}
			swp_flags = SWP_NOCOPYBITS;

			bounds.left = SDL_windowX;
			bounds.top = SDL_windowY;
			bounds.right = SDL_windowX+video->w;
			bounds.bottom = SDL_windowY+video->h;
			AdjustWindowRectEx(&bounds, GetWindowLong(SDL_Window, GWL_STYLE), (GetMenu(SDL_Window) != NULL), 0);
			width = bounds.right-bounds.left;
			height = bounds.bottom-bounds.top;
			if ( center ) {
				x = (GetSystemMetrics(SM_CXSCREEN)-width)/2;
				y = (GetSystemMetrics(SM_CYSCREEN)-height)/2;
			} else if ( SDL_windowX || SDL_windowY || window ) {
				x = bounds.left;
				y = bounds.top;
			} else {
				x = y = -1;
				swp_flags |= SWP_NOMOVE;
			}
			SetWindowPos(SDL_Window, HWND_NOTOPMOST, x, y, width, height, swp_flags);
			SDL_windowX = SDL_bounds.left;
			SDL_windowY = SDL_bounds.top;
		}

	}
	ShowWindow(SDL_Window, SW_SHOW);
	SetForegroundWindow(SDL_Window);
	SDL_resizing = 0;

	/* JC 14 Mar 2006
		Flush the message loop or this can cause big problems later
		Especially if the user decides to use dialog boxes or assert()!
	*/
	WIN_FlushMessageQueue();

	/* We're live! */
	return(video);
}

struct private_hwdata {
	LPDIRECTDRAWSURFACE3 dd_surface;
	LPDIRECTDRAWSURFACE3 dd_writebuf;
};

static int DX5_AllocDDSurface(_THIS, SDL_Surface *surface, 
				LPDIRECTDRAWSURFACE3 requested, Uint32 flag)
{
	LPDIRECTDRAWSURFACE  dd_surface1;
	LPDIRECTDRAWSURFACE3 dd_surface3;
	DDSURFACEDESC ddsd;
	HRESULT result;

	/* Clear the hardware flag, in case we fail */
	surface->flags &= ~flag;

	/* Allocate the hardware acceleration data */
	surface->hwdata = (struct private_hwdata *)
					SDL_malloc(sizeof(*surface->hwdata));
	if ( surface->hwdata == NULL ) {
		SDL_OutOfMemory();
		return(-1);
	}
	dd_surface3 = NULL;

	/* Set up the surface description */
	SDL_memset(&ddsd, 0, sizeof(ddsd));
	ddsd.dwSize = sizeof(ddsd);
	ddsd.dwFlags = (DDSD_WIDTH|DDSD_HEIGHT|DDSD_CAPS|
					DDSD_PITCH|DDSD_PIXELFORMAT);
	ddsd.dwWidth = surface->w;
	ddsd.dwHeight= surface->h;
#if defined(NONAMELESSUNION)
	ddsd.u1.lPitch = surface->pitch;
#else
	ddsd.lPitch = surface->pitch;
#endif
	if ( (flag & SDL_HWSURFACE) == SDL_HWSURFACE ) {
		ddsd.ddsCaps.dwCaps =
				(DDSCAPS_OFFSCREENPLAIN|DDSCAPS_VIDEOMEMORY);
	} else {
		ddsd.ddsCaps.dwCaps =
				(DDSCAPS_OFFSCREENPLAIN|DDSCAPS_SYSTEMMEMORY);
	}
	ddsd.ddpfPixelFormat.dwSize = sizeof(ddsd.ddpfPixelFormat);
	ddsd.ddpfPixelFormat.dwFlags = DDPF_RGB;
	if ( surface->format->palette ) {
		ddsd.ddpfPixelFormat.dwFlags |= DDPF_PALETTEINDEXED8;
	}
#if defined(NONAMELESSUNION)
	ddsd.ddpfPixelFormat.u1.dwRGBBitCount = surface->format->BitsPerPixel;
	ddsd.ddpfPixelFormat.u2.dwRBitMask = surface->format->Rmask;
	ddsd.ddpfPixelFormat.u3.dwGBitMask = surface->format->Gmask;
	ddsd.ddpfPixelFormat.u4.dwBBitMask = surface->format->Bmask;
#else
	ddsd.ddpfPixelFormat.dwRGBBitCount = surface->format->BitsPerPixel;
	ddsd.ddpfPixelFormat.dwRBitMask = surface->format->Rmask;
	ddsd.ddpfPixelFormat.dwGBitMask = surface->format->Gmask;
	ddsd.ddpfPixelFormat.dwBBitMask = surface->format->Bmask;
#endif

	/* Create the DirectDraw video surface */
	if ( requested != NULL ) {
		dd_surface3 = requested;
	} else {
		result = IDirectDraw2_CreateSurface(ddraw2,
						&ddsd, &dd_surface1, NULL); 
		if ( result != DD_OK ) {
			SetDDerror("DirectDraw2::CreateSurface", result);
			goto error_end;
		}
		result = IDirectDrawSurface_QueryInterface(dd_surface1,
			&IID_IDirectDrawSurface3, (LPVOID *)&dd_surface3);
		IDirectDrawSurface_Release(dd_surface1);
		if ( result != DD_OK ) {
			SetDDerror("DirectDrawSurface::QueryInterface", result);
			goto error_end;
		}
	}

	if ( (flag & SDL_HWSURFACE) == SDL_HWSURFACE ) {
		/* Check to see whether the surface actually ended up
		   in video memory, and fail if not.  We expect the
		   surfaces we create here to actually be in hardware!
		*/
		result = IDirectDrawSurface3_GetCaps(dd_surface3,&ddsd.ddsCaps);
		if ( result != DD_OK ) {
			SetDDerror("DirectDrawSurface3::GetCaps", result);
			goto error_end;
		}
		if ( (ddsd.ddsCaps.dwCaps&DDSCAPS_VIDEOMEMORY) !=
							DDSCAPS_VIDEOMEMORY ) {
			SDL_SetError("No room in video memory");
			goto error_end;
		}
	} else {
		/* Try to hook our surface memory */
		ddsd.dwFlags = DDSD_LPSURFACE;
		ddsd.lpSurface = surface->pixels;
		result = IDirectDrawSurface3_SetSurfaceDesc(dd_surface3,
								&ddsd, 0);
		if ( result != DD_OK ) {
			SetDDerror("DirectDraw2::SetSurfaceDesc", result);
			goto error_end;
		}
	
	}

	/* Make sure the surface format was set properly */
	SDL_memset(&ddsd, 0, sizeof(ddsd));
	ddsd.dwSize = sizeof(ddsd);
	result = IDirectDrawSurface3_Lock(dd_surface3, NULL,
		&ddsd, (DDLOCK_NOSYSLOCK|DDLOCK_WAIT), NULL);
	if ( result != DD_OK ) {
		SetDDerror("DirectDrawSurface3::Lock", result);
		goto error_end;
	}
	IDirectDrawSurface3_Unlock(dd_surface3, NULL);

	if ( (flag & SDL_HWSURFACE) == SDL_SWSURFACE ) {
		if ( ddsd.lpSurface != surface->pixels ) {
			SDL_SetError("DDraw didn't use SDL surface memory");
			goto error_end;
		}
		if (
#if defined(NONAMELESSUNION)
			ddsd.u1.lPitch
#else
			ddsd.lPitch
#endif
				 != (LONG)surface->pitch ) {
			SDL_SetError("DDraw created surface with wrong pitch");
			goto error_end;
		}
	} else {
#if defined(NONAMELESSUNION)
		surface->pitch = (Uint16)ddsd.u1.lPitch;
#else
		surface->pitch = (Uint16)ddsd.lPitch;
#endif
	}
#if defined(NONAMELESSUNION)
	if ( (ddsd.ddpfPixelFormat.u1.dwRGBBitCount != 
					surface->format->BitsPerPixel) ||
	     (ddsd.ddpfPixelFormat.u2.dwRBitMask != surface->format->Rmask) ||
	     (ddsd.ddpfPixelFormat.u3.dwGBitMask != surface->format->Gmask) ||
	     (ddsd.ddpfPixelFormat.u4.dwBBitMask != surface->format->Bmask) ){
#else
	if ( (ddsd.ddpfPixelFormat.dwRGBBitCount != 
					surface->format->BitsPerPixel) ||
	     (ddsd.ddpfPixelFormat.dwRBitMask != surface->format->Rmask) ||
	     (ddsd.ddpfPixelFormat.dwGBitMask != surface->format->Gmask) ||
	     (ddsd.ddpfPixelFormat.dwBBitMask != surface->format->Bmask) ){
#endif
		SDL_SetError("DDraw didn't use SDL surface description");
		goto error_end;
	}
	if ( (ddsd.dwWidth != (DWORD)surface->w) ||
		(ddsd.dwHeight != (DWORD)surface->h) ) {
		SDL_SetError("DDraw created surface with wrong size");
		goto error_end;
	}

	/* Set the surface private data */
	surface->flags |= flag;
	surface->hwdata->dd_surface = dd_surface3;
	if ( (surface->flags & SDL_DOUBLEBUF) == SDL_DOUBLEBUF ) {
		LPDIRECTDRAWSURFACE3 dd_writebuf;

		ddsd.ddsCaps.dwCaps = DDSCAPS_BACKBUFFER;
		result = IDirectDrawSurface3_GetAttachedSurface(dd_surface3,
						&ddsd.ddsCaps, &dd_writebuf);
		if ( result != DD_OK ) {
			SetDDerror("DirectDrawSurface3::GetAttachedSurface",
								result);
		} else {
			dd_surface3 = dd_writebuf;
		}
	}
	surface->hwdata->dd_writebuf = dd_surface3;

	/* We're ready to go! */
	return(0);

	/* Okay, so goto's are cheesy, but there are so many possible
	   errors in this function, and the cleanup is the same in 
	   every single case.  Is there a better way, other than deeply
	   nesting the code?
	*/
error_end:
	if ( (dd_surface3 != NULL) && (dd_surface3 != requested) ) {
		IDirectDrawSurface_Release(dd_surface3);
	}
	SDL_free(surface->hwdata);
	surface->hwdata = NULL;
	return(-1);
}

static int DX5_AllocHWSurface(_THIS, SDL_Surface *surface)
{
	/* DDraw limitation -- you need to set cooperative level first */
	if ( SDL_primary == NULL ) {
		SDL_SetError("You must set a non-GL video mode first");
		return(-1);
	}
	return(DX5_AllocDDSurface(this, surface, NULL, SDL_HWSURFACE));
}

#ifdef DDRAW_DEBUG
void PrintSurface(char *title, LPDIRECTDRAWSURFACE3 surface, Uint32 flags)
{
	DDSURFACEDESC ddsd;

	/* Lock and load! */
	SDL_memset(&ddsd, 0, sizeof(ddsd));
	ddsd.dwSize = sizeof(ddsd);
	if ( IDirectDrawSurface3_Lock(surface, NULL, &ddsd,
			(DDLOCK_NOSYSLOCK|DDLOCK_WAIT), NULL) != DD_OK ) {
		return;
	}
	IDirectDrawSurface3_Unlock(surface, NULL);
	
	fprintf(stderr, "%s:\n", title);
	fprintf(stderr, "\tSize: %dx%d in %s at %ld bpp (pitch = %ld)\n",
		ddsd.dwWidth, ddsd.dwHeight,
		(flags & SDL_HWSURFACE) ? "hardware" : "software",
#if defined(NONAMELESSUNION)
		ddsd.ddpfPixelFormat.u1.dwRGBBitCount, ddsd.u1.lPitch);
#else
		ddsd.ddpfPixelFormat.dwRGBBitCount, ddsd.lPitch);
#endif
	fprintf(stderr, "\tR = 0x%X, G = 0x%X, B = 0x%X\n", 
#if defined(NONAMELESSUNION)
	     		ddsd.ddpfPixelFormat.u2.dwRBitMask,
	     		ddsd.ddpfPixelFormat.u3.dwGBitMask,
	     		ddsd.ddpfPixelFormat.u4.dwBBitMask);
#else
	     		ddsd.ddpfPixelFormat.dwRBitMask,
	     		ddsd.ddpfPixelFormat.dwGBitMask,
	     		ddsd.ddpfPixelFormat.dwBBitMask);
#endif
}
#endif /* DDRAW_DEBUG */

static int DX5_HWAccelBlit(SDL_Surface *src, SDL_Rect *srcrect,
					SDL_Surface *dst, SDL_Rect *dstrect)
{
	LPDIRECTDRAWSURFACE3 src_surface;
	LPDIRECTDRAWSURFACE3 dst_surface;
	DWORD flags;
	RECT rect;
	HRESULT result;

	/* Set it up.. the desination must have a DDRAW surface */
	src_surface = src->hwdata->dd_writebuf;
	dst_surface = dst->hwdata->dd_writebuf;
	rect.top    = (LONG)srcrect->y;
	rect.bottom = (LONG)srcrect->y+srcrect->h;
	rect.left   = (LONG)srcrect->x;
	rect.right  = (LONG)srcrect->x+srcrect->w;
	if ( (src->flags & SDL_SRCCOLORKEY) == SDL_SRCCOLORKEY )
		flags = DDBLTFAST_SRCCOLORKEY;
	else
		flags = DDBLTFAST_NOCOLORKEY;
	/* FIXME:  We can remove this flag for _really_ fast blit queuing,
	           but it will affect the return values of locks and flips.
	 */
	flags |= DDBLTFAST_WAIT;

	/* Do the blit! */
	result = IDirectDrawSurface3_BltFast(dst_surface,
			dstrect->x, dstrect->y, src_surface, &rect, flags);
	if ( result != DD_OK ) {
		if ( result == DDERR_SURFACELOST ) {
			result = IDirectDrawSurface3_Restore(src_surface);
			result = IDirectDrawSurface3_Restore(dst_surface);
			/* The surfaces need to be reloaded with artwork */
			SDL_SetError("Blit surfaces were lost, reload them");
			return(-2);
		}
		SetDDerror("IDirectDrawSurface3::BltFast", result);
#ifdef DDRAW_DEBUG
 fprintf(stderr, "Original dest rect: %dx%d at %d,%d\n", dstrect->w, dstrect->h, dstrect->x, dstrect->y);
 fprintf(stderr, "HW accelerated %sblit to from 0x%p to 0x%p at (%d,%d)\n",
		(src->flags & SDL_SRCCOLORKEY) ? "colorkey " : "", src, dst,
					dstrect->x, dstrect->y);
  PrintSurface("SRC", src_surface, src->flags);
  PrintSurface("DST", dst_surface, dst->flags);
 fprintf(stderr, "Source rectangle: (%d,%d) - (%d,%d)\n",
		rect.left, rect.top, rect.right, rect.bottom);
#endif
		/* Unexpected error, fall back to software blit */
		return(src->map->sw_blit(src, srcrect, dst, dstrect));
	}
	return(0);
}

static int DX5_CheckHWBlit(_THIS, SDL_Surface *src, SDL_Surface *dst)
{
	int accelerated;

	/* We need to have a DDraw surface for HW blits */
	if ( (src->flags & SDL_HWSURFACE) == SDL_SWSURFACE ) {
		/* Allocate a DDraw surface for the blit */
		if ( src->hwdata == NULL ) {
			DX5_AllocDDSurface(this, src, NULL, SDL_SWSURFACE);
		}
	}
	if ( src->hwdata == NULL ) {
		return(0);
	}

	/* Set initial acceleration on */
	src->flags |= SDL_HWACCEL;

	/* Set the surface attributes */
	if ( (src->flags & SDL_SRCCOLORKEY) == SDL_SRCCOLORKEY ) {
		if ( DX5_SetHWColorKey(this, src, src->format->colorkey) < 0 ) {
			src->flags &= ~SDL_HWACCEL;
		}
	}
	if ( (src->flags & SDL_SRCALPHA) == SDL_SRCALPHA ) {
		if ( DX5_SetHWAlpha(this, src, src->format->alpha) < 0 ) {
			src->flags &= ~SDL_HWACCEL;
		}
	}

	/* Check to see if final surface blit is accelerated */
	accelerated = !!(src->flags & SDL_HWACCEL);
	if ( accelerated ) {
#ifdef DDRAW_DEBUG
  fprintf(stderr, "Setting accelerated blit on 0x%p\n", src);
#endif
		src->map->hw_blit = DX5_HWAccelBlit;
	}
	return(accelerated);
}

static int DX5_FillHWRect(_THIS, SDL_Surface *dst, SDL_Rect *dstrect, Uint32 color)
{
	LPDIRECTDRAWSURFACE3 dst_surface;
	RECT area;
	DDBLTFX bltfx;
	HRESULT result;

#ifdef DDRAW_DEBUG
 fprintf(stderr, "HW accelerated fill at (%d,%d)\n", dstrect->x, dstrect->y);
#endif
	dst_surface = dst->hwdata->dd_writebuf;
	area.top    = (LONG)dstrect->y;
	area.bottom = (LONG)dstrect->y+dstrect->h;
	area.left   = (LONG)dstrect->x;
	area.right  = (LONG)dstrect->x+dstrect->w;
	bltfx.dwSize = sizeof(bltfx);
#if defined(NONAMELESSUNION)
	bltfx.u5.dwFillColor = color;
#else
	bltfx.dwFillColor = color;
#endif
	result = IDirectDrawSurface3_Blt(dst_surface,
			&area, NULL, NULL, DDBLT_COLORFILL|DDBLT_WAIT, &bltfx);
	if ( result == DDERR_SURFACELOST ) {
		IDirectDrawSurface3_Restore(dst_surface);
		result = IDirectDrawSurface3_Blt(dst_surface,
			&area, NULL, NULL, DDBLT_COLORFILL|DDBLT_WAIT, &bltfx);
	}
	if ( result != DD_OK ) {
		SetDDerror("IDirectDrawSurface3::Blt", result);
		return(-1);
	}
	return(0);
}

static int DX5_SetHWColorKey(_THIS, SDL_Surface *surface, Uint32 key)
{
	DDCOLORKEY colorkey;
	HRESULT result;

	/* Set the surface colorkey */
	colorkey.dwColorSpaceLowValue = key;
	colorkey.dwColorSpaceHighValue = key;
	result = IDirectDrawSurface3_SetColorKey(
			surface->hwdata->dd_surface, DDCKEY_SRCBLT, &colorkey);
	if ( result != DD_OK ) {
		SetDDerror("IDirectDrawSurface3::SetColorKey", result);
		return(-1);
	}
	return(0);
}
static int DX5_SetHWAlpha(_THIS, SDL_Surface *surface, Uint8 alpha)
{
	return(-1);
}

static int DX5_LockHWSurface(_THIS, SDL_Surface *surface)
{
	HRESULT result;
	LPDIRECTDRAWSURFACE3 dd_surface;
	DDSURFACEDESC ddsd;

	/* Lock and load! */
	dd_surface = surface->hwdata->dd_writebuf;
	SDL_memset(&ddsd, 0, sizeof(ddsd));
	ddsd.dwSize = sizeof(ddsd);
	result = IDirectDrawSurface3_Lock(dd_surface, NULL, &ddsd,
					(DDLOCK_NOSYSLOCK|DDLOCK_WAIT), NULL);
	if ( result == DDERR_SURFACELOST ) {
		result = IDirectDrawSurface3_Restore(
						surface->hwdata->dd_surface);
		result = IDirectDrawSurface3_Lock(dd_surface, NULL, &ddsd, 
					(DDLOCK_NOSYSLOCK|DDLOCK_WAIT), NULL);
	}
	if ( result != DD_OK ) {
		SetDDerror("DirectDrawSurface3::Lock", result);
		return(-1);
	}
	/* Pitch might have changed -- recalculate pitch and offset */
#if defined(NONAMELESSUNION)
	if ( surface->pitch != ddsd.u1.lPitch ) {
		surface->pitch = ddsd.u1.lPitch;
#else
	if ( surface->pitch != ddsd.lPitch ) {
		surface->pitch = (Uint16)ddsd.lPitch;
#endif
		surface->offset =
			((ddsd.dwHeight-surface->h)/2)*surface->pitch +
			((ddsd.dwWidth-surface->w)/2)*
					surface->format->BytesPerPixel;
	}
	surface->pixels = ddsd.lpSurface;
	return(0);
}

static void DX5_UnlockHWSurface(_THIS, SDL_Surface *surface)
{
	IDirectDrawSurface3_Unlock(surface->hwdata->dd_writebuf, NULL);
	surface->pixels = NULL;
}

static int DX5_FlipHWSurface(_THIS, SDL_Surface *surface)
{
	HRESULT result;
	LPDIRECTDRAWSURFACE3 dd_surface;

	dd_surface = surface->hwdata->dd_surface;

	/* to prevent big slowdown on fast computers, wait here instead of driver ring 0 code */
	/* Dmitry Yakimov (ftech@tula.net) */
	while(IDirectDrawSurface3_GetFlipStatus(dd_surface, DDGBS_ISBLTDONE) == DDERR_WASSTILLDRAWING);

	result = IDirectDrawSurface3_Flip(dd_surface, NULL, DDFLIP_WAIT);
	if ( result == DDERR_SURFACELOST ) {
		result = IDirectDrawSurface3_Restore(
						surface->hwdata->dd_surface);
		while(IDirectDrawSurface3_GetFlipStatus(dd_surface, DDGBS_ISBLTDONE) == DDERR_WASSTILLDRAWING);
		result = IDirectDrawSurface3_Flip(dd_surface, NULL, DDFLIP_WAIT);
	}
	if ( result != DD_OK ) {
		SetDDerror("DirectDrawSurface3::Flip", result);
		return(-1);
	}
	return(0);
}

static void DX5_FreeHWSurface(_THIS, SDL_Surface *surface)
{
	if ( surface->hwdata ) {
		if ( surface->hwdata->dd_surface != SDL_primary ) {
			IDirectDrawSurface3_Release(surface->hwdata->dd_surface);
		}
		SDL_free(surface->hwdata);
		surface->hwdata = NULL;
	}
}

void DX5_WindowUpdate(_THIS, int numrects, SDL_Rect *rects)
{
	HRESULT result;
	int i;
	RECT src, dst;

	for ( i=0; i<numrects; ++i ) {
		src.top    = (LONG)rects[i].y;
		src.bottom = (LONG)rects[i].y+rects[i].h;
		src.left   = (LONG)rects[i].x;
		src.right  = (LONG)rects[i].x+rects[i].w;
		dst.top    = SDL_bounds.top+src.top;
		dst.left   = SDL_bounds.left+src.left;
		dst.bottom = SDL_bounds.top+src.bottom;
		dst.right  = SDL_bounds.left+src.right;
		result = IDirectDrawSurface3_Blt(SDL_primary, &dst, 
					this->screen->hwdata->dd_surface, &src,
							DDBLT_WAIT, NULL);
		/* Doh!  Check for lost surface and restore it */
		if ( result == DDERR_SURFACELOST ) {
			IDirectDrawSurface3_Restore(SDL_primary);
			IDirectDrawSurface3_Blt(SDL_primary, &dst, 
					this->screen->hwdata->dd_surface, &src,
							DDBLT_WAIT, NULL);
		}
	}
}

void DX5_DirectUpdate(_THIS, int numrects, SDL_Rect *rects)
{
}

/* Compress a full palette into the limited number of colors given to us
   by windows.

   The "best" way to do this is to sort the colors by diversity and place
   the most diverse colors into the limited palette.  Unfortunately this
   results in widely varying colors being displayed in the interval during
   which the windows palette has been set, and the mapping of the shadow
   surface to the new palette.  This is especially noticeable during fades.

   To deal with this problem, we can copy a predetermined portion of the
   full palette, and use that as the limited palette.  This allows colors
   to fade smoothly as the remapping is very similar on each palette change.
   Unfortunately, this breaks applications which partition the palette into
   distinct and widely varying areas, expecting all colors to be available.

   I'm making them both available, chosen at compile time.
   If you want the chunk-o-palette algorithm, define SIMPLE_COMPRESSION,
   otherwise the sort-by-diversity algorithm will be used.
*/
#define SIMPLE_COMPRESSION
#define CS_CS_DIST(A, B) ({						\
	int r = (A.r - B.r);						\
	int g = (A.g - B.g);						\
	int b = (A.b - B.b);						\
	(r*r + g*g + b*b);						\
})
static void DX5_CompressPalette(_THIS, SDL_Color *colors, int ncolors, int maxcolors)
{
#ifdef SIMPLE_COMPRESSION
	int i, j;
#else
	static SDL_Color zero = { 0, 0, 0, 0 };
	int i, j;
	int max, dist;
	int prev, next;
	int *pool;
	int *seen, *order;
#endif

	/* Does this happen? */
	if ( maxcolors > ncolors ) {
		maxcolors = ncolors;
	}

#ifdef SIMPLE_COMPRESSION
	/* Just copy the first "maxcolors" colors */
	for ( j=10, i=0; i<maxcolors; ++i, ++j ) {
		SDL_colors[j].peRed = colors[i].r;
		SDL_colors[j].peGreen = colors[i].g;
		SDL_colors[j].peBlue = colors[i].b;
	}
#else
	/* Allocate memory for the arrays we use */
	pool = SDL_stack_alloc(int, 2*ncolors);
	if ( pool == NULL ) {
		/* No worries, just return */;
		return;
	}
	seen = pool;
	SDL_memset(seen, 0, ncolors*sizeof(int));
	order = pool+ncolors;

	/* Start with the brightest color */
	max = 0;
	for ( i=0; i<ncolors; ++i ) {
		dist = CS_CS_DIST(zero, colors[i]);
		if ( dist >= max ) {
			max = dist;
			next = i;
		}
	}
	j = 0;
	order[j++] = next;
	seen[next] = 1;
	prev = next;

	/* Keep going through all the colors */
	while ( j < maxcolors ) {
		max = 0;
		for ( i=0; i<ncolors; ++i ) {
			if ( seen[i] ) {
				continue;
			}
			dist = CS_CS_DIST(colors[i], colors[prev]);
			if ( dist >= max ) {
				max = dist;
				next = i;
			}
		}
		order[j++] = next;
		seen[next] = 1;
		prev = next;
	}

	/* Compress the colors to the palette */
	for ( j=10, i=0; i<maxcolors; ++i, ++j ) {
		SDL_colors[j].peRed = colors[order[i]].r;
		SDL_colors[j].peGreen = colors[order[i]].g;
		SDL_colors[j].peBlue = colors[order[i]].b;
	}
	SDL_stack_free(pool);
#endif /* SIMPLE_COMPRESSION */
}

/* Set the system colormap in both fullscreen and windowed modes */
int DX5_SetColors(_THIS, int firstcolor, int ncolors, SDL_Color *colors)
{
	int i;
	int alloct_all;

	/* Copy palette colors into display palette */
	alloct_all = 0;
	if ( SDL_palette != NULL ) {
		if ( (this->screen->flags&SDL_FULLSCREEN) == SDL_FULLSCREEN ) {
			/* We can set all entries explicitly */
			for ( i=0; i< ncolors; ++i ) {
			        int j = firstcolor + i;
				SDL_colors[j].peRed = colors[i].r;
				SDL_colors[j].peGreen = colors[i].g;
				SDL_colors[j].peBlue = colors[i].b;
			}
			/* This sends an WM_PALETTECHANGED message to us */
			colorchange_expected = 1;
			IDirectDrawPalette_SetEntries(SDL_palette, 0,
				firstcolor, ncolors, &SDL_colors[firstcolor]);
			alloct_all = 1;
		} else {
			/* Grab the 236 most diverse colors in the palette */
			DX5_CompressPalette(this, colors, ncolors, 236);
			/* This sends an WM_PALETTECHANGED message to us */
			colorchange_expected = 1;
			IDirectDrawPalette_SetEntries(SDL_palette, 0,
							0, 256, SDL_colors);
		}
	}
	return(alloct_all);
}

/* Gamma code is only available on DirectX 7 and newer */
static int DX5_SetGammaRamp(_THIS, Uint16 *ramp)
{
#ifdef IDirectDrawGammaControl_SetGammaRamp
	LPDIRECTDRAWGAMMACONTROL gamma;
	DDGAMMARAMP gamma_ramp;
	HRESULT result;
#endif

	/* In windowed or OpenGL mode, use windib gamma code */
	if ( ! DDRAW_FULLSCREEN() ) {
		return DIB_SetGammaRamp(this, ramp);
	}

#ifndef IDirectDrawGammaControl_SetGammaRamp
	SDL_SetError("SDL compiled without DirectX gamma ramp support");
	return -1;
#else
	/* Check for a video mode! */
	if ( ! SDL_primary ) {
		SDL_SetError("A video mode must be set for gamma correction");
		return(-1);
	}

	/* Get the gamma control object */
	result = IDirectDrawSurface3_QueryInterface(SDL_primary,
			&IID_IDirectDrawGammaControl, (LPVOID *)&gamma);
	if ( result != DD_OK ) {
		SetDDerror("DirectDrawSurface3::QueryInterface(GAMMA)", result);
		return(-1);
	}

	/* Set up the gamma ramp */
	SDL_memcpy(gamma_ramp.red, &ramp[0*256], 256*sizeof(*ramp));
	SDL_memcpy(gamma_ramp.green, &ramp[1*256], 256*sizeof(*ramp));
	SDL_memcpy(gamma_ramp.blue, &ramp[2*256], 256*sizeof(*ramp));
	result = IDirectDrawGammaControl_SetGammaRamp(gamma, 0, &gamma_ramp);
	if ( result != DD_OK ) {
		SetDDerror("DirectDrawGammaControl::SetGammaRamp()", result);
	}

	/* Release the interface and return */
	IDirectDrawGammaControl_Release(gamma);
	return (result == DD_OK) ? 0 : -1;
#endif /* !IDirectDrawGammaControl_SetGammaRamp */
}

static int DX5_GetGammaRamp(_THIS, Uint16 *ramp)
{
#ifdef IDirectDrawGammaControl_SetGammaRamp
	LPDIRECTDRAWGAMMACONTROL gamma;
	DDGAMMARAMP gamma_ramp;
	HRESULT result;
#endif

	/* In windowed or OpenGL mode, use windib gamma code */
	if ( ! DDRAW_FULLSCREEN() ) {
		return DIB_GetGammaRamp(this, ramp);
	}

#ifndef IDirectDrawGammaControl_SetGammaRamp
	SDL_SetError("SDL compiled without DirectX gamma ramp support");
	return -1;
#else
	/* Check for a video mode! */
	if ( ! SDL_primary ) {
		SDL_SetError("A video mode must be set for gamma correction");
		return(-1);
	}

	/* Get the gamma control object */
	result = IDirectDrawSurface3_QueryInterface(SDL_primary,
			&IID_IDirectDrawGammaControl, (LPVOID *)&gamma);
	if ( result != DD_OK ) {
		SetDDerror("DirectDrawSurface3::QueryInterface(GAMMA)", result);
		return(-1);
	}

	/* Set up the gamma ramp */
	result = IDirectDrawGammaControl_GetGammaRamp(gamma, 0, &gamma_ramp);
	if ( result == DD_OK ) {
		SDL_memcpy(&ramp[0*256], gamma_ramp.red, 256*sizeof(*ramp));
		SDL_memcpy(&ramp[1*256], gamma_ramp.green, 256*sizeof(*ramp));
		SDL_memcpy(&ramp[2*256], gamma_ramp.blue, 256*sizeof(*ramp));
	} else {
		SetDDerror("DirectDrawGammaControl::GetGammaRamp()", result);
	}

	/* Release the interface and return */
	IDirectDrawGammaControl_Release(gamma);
	return (result == DD_OK) ? 0 : -1;
#endif /* !IDirectDrawGammaControl_SetGammaRamp */
}

void DX5_VideoQuit(_THIS)
{
	int i, j;

	/* If we're fullscreen GL, we need to reset the display */
	if ( this->screen != NULL ) {
#ifndef NO_CHANGEDISPLAYSETTINGS
		if ( (this->screen->flags & (SDL_OPENGL|SDL_FULLSCREEN)) ==
		                            (SDL_OPENGL|SDL_FULLSCREEN) ) {
			ChangeDisplaySettings(NULL, 0);
			ShowWindow(SDL_Window, SW_HIDE);
		}
#endif
		if ( this->screen->flags & SDL_OPENGL ) {
			WIN_GL_ShutDown(this);
		}
	}

	/* Free any palettes we used */
	if ( SDL_palette != NULL ) {
		IDirectDrawPalette_Release(SDL_palette);
		SDL_palette = NULL;
	}

	/* Allow the primary surface to be freed */
	if ( SDL_primary != NULL ) {
		SDL_primary = NULL;
	}

	/* Free video mode lists */
	for ( i=0; i<NUM_MODELISTS; ++i ) {
		if ( SDL_modelist[i] != NULL ) {
			for ( j=0; SDL_modelist[i][j]; ++j )
				SDL_free(SDL_modelist[i][j]);
			SDL_free(SDL_modelist[i]);
			SDL_modelist[i] = NULL;
		}
	}

	/* Free the window */
	DIB_QuitGamma(this);
	if ( SDL_Window ) {
		DX5_DestroyWindow(this);
	}

	/* Free our window icon */
	if ( screen_icn ) {
		DestroyIcon(screen_icn);
		screen_icn = NULL;
	}
}

/* Exported for the windows message loop only */
void DX5_Activate(_THIS, BOOL active, BOOL minimized)
{
}
void DX5_RealizePalette(_THIS)
{
	if ( SDL_palette ) {
		IDirectDrawSurface3_SetPalette(SDL_primary, SDL_palette);
	}
}
static void DX5_Recolor8Bit(_THIS, SDL_Surface *surface, Uint8 *mapping)
{
	int row, col;
	Uint8 *pixels;

	if ( surface->w && surface->h ) {
		if ( (surface->flags & SDL_HWSURFACE) == SDL_HWSURFACE ) {
			if ( this->LockHWSurface(this, surface) < 0 ) {
				return;
			}
		}
		for ( row=0; row<surface->h; ++row ) {
			pixels = (Uint8 *)surface->pixels+row*surface->pitch;
			for ( col=0; col<surface->w; ++col, ++pixels ) {
				*pixels = mapping[*pixels];
			}
		}
		if ( (surface->flags & SDL_HWSURFACE) == SDL_HWSURFACE ) {
			this->UnlockHWSurface(this, surface);
		}
		SDL_UpdateRect(surface, 0, 0, 0, 0);
	}
}
void DX5_PaletteChanged(_THIS, HWND window)
{
	SDL_Palette *palette;
	SDL_Color *saved = NULL;
	HDC hdc;
	int i;
	PALETTEENTRY *entries;

	/* This is true when the window is closing */
	if ( (SDL_primary == NULL) || (SDL_VideoSurface == NULL) )
		return;

	/* We need to get the colors as they were set */
	palette = this->physpal;
	if(!palette)
	        palette = SDL_VideoSurface->format->palette;
	if ( palette == NULL ) { /* Sometimes we don't have a palette */
		return;
	}
	entries = SDL_stack_alloc(PALETTEENTRY, palette->ncolors);
	hdc = GetDC(window);
	GetSystemPaletteEntries(hdc, 0, palette->ncolors, entries);
	ReleaseDC(window, hdc);
	if ( ! colorchange_expected ) {
		saved = SDL_stack_alloc(SDL_Color, palette->ncolors);
		SDL_memcpy(saved, palette->colors, 
					palette->ncolors*sizeof(SDL_Color));
	}
	for ( i=0; i<palette->ncolors; ++i ) {
		palette->colors[i].r = entries[i].peRed;
		palette->colors[i].g = entries[i].peGreen;
		palette->colors[i].b = entries[i].peBlue;
	}
	SDL_stack_free(entries);
	if ( ! colorchange_expected ) {
		Uint8 mapping[256];

		SDL_memset(mapping, 0, sizeof(mapping));
		for ( i=0; i<palette->ncolors; ++i ) {
			mapping[i] = SDL_FindColor(palette,
					saved[i].r, saved[i].g, saved[i].b);
		}
		DX5_Recolor8Bit(this, SDL_VideoSurface, mapping);
		SDL_stack_free(saved);
	}
	colorchange_expected = 0;

	/* Notify all mapped surfaces of the change */
	SDL_FormatChanged(SDL_VideoSurface);
}

/* Exported for the windows message loop only */
void DX5_WinPAINT(_THIS, HDC hdc)
{
	SDL_UpdateRect(SDL_PublicSurface, 0, 0, 0, 0);
}
