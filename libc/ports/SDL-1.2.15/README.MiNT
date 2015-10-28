==============================================================================
Using the Simple DirectMedia Layer on Atari
==============================================================================


    If you want to build SDL from sources to create SDL programs on Atari:
        see sections I - II.
	
    If you want to create SDL programs on Atari using SDL binary build,
        download it from my web site (URL at end of this file).

    If you want to configure a program using SDL on Atari,
	    see sections IV - VI.


==============================================================================
I.  Building the Simple DirectMedia Layer libraries:
    (This step isn't necessary if you have the SDL binary distribution)

  Do the classic configure, with --disable-shared --enable-static and:

    Tos version (should run everywhere):
      --disable-threads
    Tos does not support threads.

    MiNT version (maybe Magic, only for multitasking OS):
      --disable-pthreads --enable-pth
    Mint and Magic may supports threads, so audio can be used with current
    devices, like Sun audio, or disk-writing support. Like Tos, interrupt
    audio without threads is more suited for Atari machines.

  Then you can make ; make install it.

==============================================================================
II. Building the Simple DirectMedia Layer test programs:

  Do the classic configure, then make.

  Run them !

==============================================================================
III.  Enjoy! :)

  If you have a project you'd like me to know about, or want to ask questions,
  go ahead and join the SDL developer's mailing list by sending e-mail to:

	sdl-request@libsdl.org

  and put "subscribe" into the subject of the message. Or alternatively you
  can use the web interface:

	http://www.libsdl.org/mailman/listinfo/sdl
  
==============================================================================
IV.  What is supported:

Keyboard (GEMDOS, BIOS, GEM, Ikbd)
Mouse (XBIOS, GEM, Ikbd, /dev/mouse (non working atm, disabled))
Video (XBIOS (Fullscreen), GEM (Windowed and Fullscreen))
Timer (VBL vector, GNU pth library)
Joysticks and joypads (Ikbd, Hardware)
Audio (Hardware, XBIOS, GSXB, MCSN, STFA, /dev/audio if threads enabled)
Threads (Multitasking OS only via GNU pth library)
Shared object loader (using LDG library from http://ldg.atari.org/)
Audio CD (MetaDOS)
OpenGL (using Mesa offscreen rendering driver)

- Dependent driver combinations:
Video   Kbd     Mouse   Timer   Joysticks
xbios   ikbd    ikbd    vbl(2)  ikbd
xbios   gemdos  xbios   vbl(2)  xbios
xbios   bios    xbios   vbl(2)  xbios
gem     gem     gem(1)  vbl(2)  xbios

Audio   O/S     Misc
dma8    All     Uses MFP Timer A interrupt
xbios	TOS     Uses MFP Timer A interrupt
xbios   MiNT    Uses MFP Timer A interrupt
xbios   Magic   Uses MFP Timer A interrupt
stfa    All     Uses MFP interrupt
mcsn	TOS     Uses MFP Timer A interrupt
mcsn    MiNT    Uses MiNT thread
mcsn    Magic   Disabled
gsxb    All     Uses GSXB callback

Joypad driver always uses hardware access.
OpenGL driver always uses OSMesa.

(1) GEM does not report relative mouse motion, so xbios mouse driver is used
to report this type event.
A preliminary driver for /dev/mouse device driver is present, but is disabled
till it can be used with other applications simultaneously.

(2) If you build SDL with threads using the GNU pth library, timers are
supported via the pth library.

==============================================================================
V.  Environment variables:

SDL_VIDEODRIVER:
	Set to 'xbios' to force xbios video driver
	Set to 'gem' to force gem video driver

SDL_VIDEO_GL_DRIVER:
	Set to filename to load as OpenGL library, if you use SDL_GL_LoadLibrary()

SDL_AUDIODRIVER:
	Set to 'mint_gsxb' to force Atari GSXB audio driver
	Set to 'mint_mcsn' to force Atari MCSN audio driver
	Set to 'mint_stfa' to force Atari STFA audio driver
	Set to 'mint_xbios' to force Atari Xbios audio driver
	Set to 'mint_dma8' to force Atari 8 bits DMA audio driver
	Set to 'audio' to force Sun /dev/audio audio driver
	Set to 'disk' to force disk-writing audio driver

SDL_ATARI_EVENTSDRIVER
	Set to 'ikbd' to force IKBD 6301 keyboard driver
	Set to 'gemdos' to force gemdos keyboard driver
	Set to 'bios' to force bios keyboard driver

SDL_JOYSTICK_ATARI:
	Use any of these strings in the environment variable to enable or
	disable a joystick:

	'ikbd-joy1-[on|off]' for IKBD joystick on port 1 (hardware access)
	'xbios-joy1-[on|off]' for IKBD joystick on port 1 (xbios access)
	'porta-pad-[on|off]' for joypad and/or teamtap on port A
	'porta-joy0-[on|off]' for joystick 0 on port A
	'porta-joy1-[on|off]' for joystick 1 on port A
	'porta-lp-[on|off]' for lightpen on port A
	'porta-anpad-[on|off]' for analog paddle on port A
	'portb-pad-[on|off]' for joypad and/or teamtap on port B
	'portb-joy0-[on|off]' for joystick 0 on port B
	'portb-joy1-[on|off]' for joystick 1 on port B
	'portb-anpad-[on|off]' for analog paddle on port B

	Default configuration is:
		'ikbd-joy1-on' (if IKBD events driver enabled)
		'xbios-joy1-on' (if gemdos/bios/gem events driver enabled)
		'porta-pad-on portb-pad-on' (if available on the machine)

	port[a|b]-[pad|joy?|lp|anpad]-* strings are mutually exclusives.
	On such a port, you can only use a joypad OR 1 or 2 joysticks OR
	a lightpen OR an analog paddle. You must disable joypad before
	setting another controller.

	The second joystick port on IKBD is used by the mouse, so not usable.
	Another problem with the IKBD: mouse buttons and joystick fire buttons
	are wired together at the hardware level, it means:
		port 0                port 0           port 1
		mouse left button  = joystick fire 0 = joystick fire 1
		mouse right button = joystick fire 1 = joystick fire 0

	Descriptions of joysticks/joypads:
	- Joypads: 1 hat, 17 buttons (Atari Jaguar console-like).
	- Joysticks: 1 hat, 1 button.
	- Lightpen, analog paddles: 2 axis, 2 buttons. The 2 buttons are those
	  affected to 1 button joysticks on the same port.

==============================================================================
VI.  More informations about drivers:

OpenGL:
	The default is to use the Mesa offscreen driver (osmesa.ldg). If you want
	to use an older OpenGL implementation, like mesa_gl.ldg or tiny_gl.ldg,
	your program must use SDL_GL_LoadLibrary() to do so, and retrieve the
	needed function pointers with SDL_LoadFunction(). In all cases, the OpenGL
	context is taken care of by SDL itself, you just have to use gl* functions.

	However, there is one OpenGL call that has a different prototype in the old
	implementations: glOrtho(). In the old implementations, it has 6 float as
	parameters, in the standard one, it has 6 double parameters. If you want
	to compile testdyngl, or any other SDL program that loads its OpenGL
	library, you must change the glOrtho() prototype used in this program. In
	osmesa.ldg, you can retrieve a glOrtho() with double parameters, by
	searching for the function "glOrtho6d".

Xbios video:
	Video chip is detected using the _VDO cookie.
	Screen enhancers are not supported, but could be if you know how to
	use them.

	ST, STE, Mega ST, Mega STE:
		320x200x4 bits, shades of grey, available only for the purpose
		of testing SDL.
	TT:
		320x480x8 and 320x240x8 (software double-lined mode).
	Falcon:
		All modes supported by the current monitor (RVB or VGA).
		BlowUp and Centscreen extended modes, ScreenBlaster 3 current mode.
	Milan:
		Experimental support
	Clones and any machine with monochrome monitor:
		Not supported.

Gem video:
	Automatically used if xbios not available.

	All machines:
		Only the current resolution, if 8 bits or higher depth.

IKBD keyboard, mouse and joystick driver:
	Available if _MCH cookie is ST, Mega ST, STE, Mega STE, TT or Falcon.

	Hades has an IKBD, but xbios is not available for video, so IKBD
	driver is disabled.

Gemdos and bios keyboard driver:
	Available on all machines.

Mouse and joystick xbios driver:
	Available on all machines (I think).

Joypad driver:
	Available if _MCH cookie is STE or Falcon. Supports teamtap.

PTH timer driver:
	Available with multitasking OS.

VBL timer driver:
	Available on all machines (I think).

Audio drivers:
	Cookies _SND, MCSN, STFA and GSXB used to detect supported audio
	capabilities.

	STE, Mega STE, TT:
		8 bits DMA (hardware access)
		STFA, MCSN or GSXB driver if installed
	Falcon:
		8 bits DMA (hardware access)
		Xbios functions
		STFA, MCSN or GSXB driver if installed
	Other machines:
		STFA, MCSN or GSXB driver if installed

	STFA driver:
		http://removers.free.fr/softs/stfa.html
	GSXB driver:
		http://assemsoft.atari.org/gsxb/
	MacSound driver:
		http://jf.omnis.ch/software/tos/
	MagicSound driver (MCSN,GSXB compatible):
		http://perso.wanadoo.fr/didierm/
	X-Sound driver (GSXB compatible):
		http://www.uni-ulm.de/~s_thuth/atari/xsound_e.html

-- 
Patrice Mandin <patmandin@gmail.com>
http://pmandin.atari.org/
