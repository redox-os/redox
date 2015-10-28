
SDL_image 1.2

The latest version of this library is available from:
http://www.libsdl.org/projects/SDL_image/

This is a simple library to load images of various formats as SDL surfaces.
This library supports BMP, PNM (PPM/PGM/PBM), XPM, LBM, PCX, GIF, JPEG, PNG,
TGA, and TIFF formats.

API:
#include "SDL_image.h"

	SDL_Surface *IMG_Load(const char *file);
or
	SDL_Surface *IMG_Load_RW(SDL_RWops *src, int freesrc);
or
	SDL_Surface *IMG_LoadTyped_RW(SDL_RWops *src, int freesrc, char *type);

where type is a string specifying the format (i.e. "PNG" or "pcx").
Note that IMG_Load_RW cannot load TGA images.

To create a surface from an XPM image included in C source, use:

	SDL_Surface *IMG_ReadXPMFromArray(char **xpm);

An example program 'showimage' is included, with source in showimage.c

JPEG support requires the JPEG library: http://www.ijg.org/
PNG support requires the PNG library: http://www.libpng.org/pub/png/libpng.html
    and the Zlib library: http://www.gzip.org/zlib/
TIFF support requires the TIFF library: ftp://ftp.sgi.com/graphics/tiff/

If you have these libraries installed in non-standard places, you can
try adding those paths to the configure script, e.g.
sh ./configure CPPFLAGS="-I/somewhere/include" LDFLAGS="-L/somewhere/lib"
If this works, you may need to add /somewhere/lib to your LD_LIBRARY_PATH
so shared library loading works correctly.

This library is under the zlib License, see the file "COPYING" for details.
