/*
  SDL_image:  An example image loading library for use with SDL
  Copyright (C) 1997-2012 Sam Lantinga <slouken@libsdl.org>

  This software is provided 'as-is', without any express or implied
  warranty.  In no event will the authors be held liable for any damages
  arising from the use of this software.

  Permission is granted to anyone to use this software for any purpose,
  including commercial applications, and to alter it and redistribute it
  freely, subject to the following restrictions:

  1. The origin of this software must not be misrepresented; you must not
     claim that you wrote the original software. If you use this software
     in a product, an acknowledgment in the product documentation would be
     appreciated but is not required.
  2. Altered source versions must be plainly marked as such, and must not be
     misrepresented as being the original software.
  3. This notice may not be removed or altered from any source distribution.
*/

#if !defined(__APPLE__) || defined(SDL_IMAGE_USE_COMMON_BACKEND)

/* This is a JPEG image file loading framework */

#include <stdio.h>
#include <string.h>
#include <setjmp.h>

#include "SDL_image.h"

#ifdef LOAD_JPG

#include <jpeglib.h>

#ifdef JPEG_TRUE  /* MinGW version of jpeg-8.x renamed TRUE to JPEG_TRUE etc. */
	typedef JPEG_boolean boolean;
	#define TRUE JPEG_TRUE
	#define FALSE JPEG_FALSE
#endif

/* Define this for fast loading and not as good image quality */
/*#define FAST_JPEG*/

/* Define this for quicker (but less perfect) JPEG identification */
#define FAST_IS_JPEG

static struct {
	int loaded;
	void *handle;
	void (*jpeg_calc_output_dimensions) (j_decompress_ptr cinfo);
	void (*jpeg_CreateDecompress) (j_decompress_ptr cinfo, int version, size_t structsize);
	void (*jpeg_destroy_decompress) (j_decompress_ptr cinfo);
	boolean (*jpeg_finish_decompress) (j_decompress_ptr cinfo);
	int (*jpeg_read_header) (j_decompress_ptr cinfo, boolean require_image);
	JDIMENSION (*jpeg_read_scanlines) (j_decompress_ptr cinfo, JSAMPARRAY scanlines, JDIMENSION max_lines);
	boolean (*jpeg_resync_to_restart) (j_decompress_ptr cinfo, int desired);
	boolean (*jpeg_start_decompress) (j_decompress_ptr cinfo);
	struct jpeg_error_mgr * (*jpeg_std_error) (struct jpeg_error_mgr * err);
} lib;

#ifdef LOAD_JPG_DYNAMIC
int IMG_InitJPG()
{
	if ( lib.loaded == 0 ) {
		lib.handle = SDL_LoadObject(LOAD_JPG_DYNAMIC);
		if ( lib.handle == NULL ) {
			return -1;
		}
		lib.jpeg_calc_output_dimensions =
			(void (*) (j_decompress_ptr))
			SDL_LoadFunction(lib.handle, "jpeg_calc_output_dimensions");
		if ( lib.jpeg_calc_output_dimensions == NULL ) {
			SDL_UnloadObject(lib.handle);
			return -1;
		}
		lib.jpeg_CreateDecompress = 
			(void (*) (j_decompress_ptr, int, size_t))
			SDL_LoadFunction(lib.handle, "jpeg_CreateDecompress");
		if ( lib.jpeg_CreateDecompress == NULL ) {
			SDL_UnloadObject(lib.handle);
			return -1;
		}
		lib.jpeg_destroy_decompress = 
			(void (*) (j_decompress_ptr))
			SDL_LoadFunction(lib.handle, "jpeg_destroy_decompress");
		if ( lib.jpeg_destroy_decompress == NULL ) {
			SDL_UnloadObject(lib.handle);
			return -1;
		}
		lib.jpeg_finish_decompress = 
			(boolean (*) (j_decompress_ptr))
			SDL_LoadFunction(lib.handle, "jpeg_finish_decompress");
		if ( lib.jpeg_finish_decompress == NULL ) {
			SDL_UnloadObject(lib.handle);
			return -1;
		}
		lib.jpeg_read_header = 
			(int (*) (j_decompress_ptr, boolean))
			SDL_LoadFunction(lib.handle, "jpeg_read_header");
		if ( lib.jpeg_read_header == NULL ) {
			SDL_UnloadObject(lib.handle);
			return -1;
		}
		lib.jpeg_read_scanlines = 
			(JDIMENSION (*) (j_decompress_ptr, JSAMPARRAY, JDIMENSION))
			SDL_LoadFunction(lib.handle, "jpeg_read_scanlines");
		if ( lib.jpeg_read_scanlines == NULL ) {
			SDL_UnloadObject(lib.handle);
			return -1;
		}
		lib.jpeg_resync_to_restart = 
			(boolean (*) (j_decompress_ptr, int))
			SDL_LoadFunction(lib.handle, "jpeg_resync_to_restart");
		if ( lib.jpeg_resync_to_restart == NULL ) {
			SDL_UnloadObject(lib.handle);
			return -1;
		}
		lib.jpeg_start_decompress = 
			(boolean (*) (j_decompress_ptr))
			SDL_LoadFunction(lib.handle, "jpeg_start_decompress");
		if ( lib.jpeg_start_decompress == NULL ) {
			SDL_UnloadObject(lib.handle);
			return -1;
		}
		lib.jpeg_std_error = 
			(struct jpeg_error_mgr * (*) (struct jpeg_error_mgr *))
			SDL_LoadFunction(lib.handle, "jpeg_std_error");
		if ( lib.jpeg_std_error == NULL ) {
			SDL_UnloadObject(lib.handle);
			return -1;
		}
	}
	++lib.loaded;

	return 0;
}
void IMG_QuitJPG()
{
	if ( lib.loaded == 0 ) {
		return;
	}
	if ( lib.loaded == 1 ) {
		SDL_UnloadObject(lib.handle);
	}
	--lib.loaded;
}
#else
int IMG_InitJPG()
{
	if ( lib.loaded == 0 ) {
		lib.jpeg_calc_output_dimensions = jpeg_calc_output_dimensions;
		lib.jpeg_CreateDecompress = jpeg_CreateDecompress;
		lib.jpeg_destroy_decompress = jpeg_destroy_decompress;
		lib.jpeg_finish_decompress = jpeg_finish_decompress;
		lib.jpeg_read_header = jpeg_read_header;
		lib.jpeg_read_scanlines = jpeg_read_scanlines;
		lib.jpeg_resync_to_restart = jpeg_resync_to_restart;
		lib.jpeg_start_decompress = jpeg_start_decompress;
		lib.jpeg_std_error = jpeg_std_error;
	}
	++lib.loaded;

	return 0;
}
void IMG_QuitJPG()
{
	if ( lib.loaded == 0 ) {
		return;
	}
	if ( lib.loaded == 1 ) {
	}
	--lib.loaded;
}
#endif /* LOAD_JPG_DYNAMIC */

/* See if an image is contained in a data source */
int IMG_isJPG(SDL_RWops *src)
{
	int start;
	int is_JPG;
	int in_scan;
	Uint8 magic[4];

	/* This detection code is by Steaphan Greene <stea@cs.binghamton.edu> */
	/* Blame me, not Sam, if this doesn't work right. */
	/* And don't forget to report the problem to the the sdl list too! */

	if ( !src )
		return 0;
	start = SDL_RWtell(src);
	is_JPG = 0;
	in_scan = 0;
	if ( SDL_RWread(src, magic, 2, 1) ) {
		if ( (magic[0] == 0xFF) && (magic[1] == 0xD8) ) {
			is_JPG = 1;
			while (is_JPG == 1) {
				if(SDL_RWread(src, magic, 1, 2) != 2) {
					is_JPG = 0;
				} else if( (magic[0] != 0xFF) && (in_scan == 0) ) {
					is_JPG = 0;
				} else if( (magic[0] != 0xFF) || (magic[1] == 0xFF) ) {
					/* Extra padding in JPEG (legal) */
					/* or this is data and we are scanning */
					SDL_RWseek(src, -1, RW_SEEK_CUR);
				} else if(magic[1] == 0xD9) {
					/* Got to end of good JPEG */
					break;
				} else if( (in_scan == 1) && (magic[1] == 0x00) ) {
					/* This is an encoded 0xFF within the data */
				} else if( (magic[1] >= 0xD0) && (magic[1] < 0xD9) ) {
					/* These have nothing else */
				} else if(SDL_RWread(src, magic+2, 1, 2) != 2) {
					is_JPG = 0;
				} else {
					/* Yes, it's big-endian */
					Uint32 start;
					Uint32 size;
					Uint32 end;
					start = SDL_RWtell(src);
					size = (magic[2] << 8) + magic[3];
					end = SDL_RWseek(src, size-2, RW_SEEK_CUR);
					if ( end != start + size - 2 ) is_JPG = 0;
					if ( magic[1] == 0xDA ) {
						/* Now comes the actual JPEG meat */
#ifdef	FAST_IS_JPEG
						/* Ok, I'm convinced.  It is a JPEG. */
						break;
#else
						/* I'm not convinced.  Prove it! */
						in_scan = 1;
#endif
					}
				}
			}
		}
	}
	SDL_RWseek(src, start, RW_SEEK_SET);
	return(is_JPG);
}

#define INPUT_BUFFER_SIZE	4096
typedef struct {
	struct jpeg_source_mgr pub;

	SDL_RWops *ctx;
	Uint8 buffer[INPUT_BUFFER_SIZE];
} my_source_mgr;

/*
 * Initialize source --- called by jpeg_read_header
 * before any data is actually read.
 */
static void init_source (j_decompress_ptr cinfo)
{
	/* We don't actually need to do anything */
	return;
}

/*
 * Fill the input buffer --- called whenever buffer is emptied.
 */
static boolean fill_input_buffer (j_decompress_ptr cinfo)
{
	my_source_mgr * src = (my_source_mgr *) cinfo->src;
	int nbytes;

	nbytes = SDL_RWread(src->ctx, src->buffer, 1, INPUT_BUFFER_SIZE);
	if (nbytes <= 0) {
		/* Insert a fake EOI marker */
		src->buffer[0] = (Uint8) 0xFF;
		src->buffer[1] = (Uint8) JPEG_EOI;
		nbytes = 2;
	}
	src->pub.next_input_byte = src->buffer;
	src->pub.bytes_in_buffer = nbytes;

	return TRUE;
}


/*
 * Skip data --- used to skip over a potentially large amount of
 * uninteresting data (such as an APPn marker).
 *
 * Writers of suspendable-input applications must note that skip_input_data
 * is not granted the right to give a suspension return.  If the skip extends
 * beyond the data currently in the buffer, the buffer can be marked empty so
 * that the next read will cause a fill_input_buffer call that can suspend.
 * Arranging for additional bytes to be discarded before reloading the input
 * buffer is the application writer's problem.
 */
static void skip_input_data (j_decompress_ptr cinfo, long num_bytes)
{
	my_source_mgr * src = (my_source_mgr *) cinfo->src;

	/* Just a dumb implementation for now.	Could use fseek() except
	 * it doesn't work on pipes.  Not clear that being smart is worth
	 * any trouble anyway --- large skips are infrequent.
	 */
	if (num_bytes > 0) {
		while (num_bytes > (long) src->pub.bytes_in_buffer) {
			num_bytes -= (long) src->pub.bytes_in_buffer;
			(void) src->pub.fill_input_buffer(cinfo);
			/* note we assume that fill_input_buffer will never
			 * return FALSE, so suspension need not be handled.
			 */
		}
		src->pub.next_input_byte += (size_t) num_bytes;
		src->pub.bytes_in_buffer -= (size_t) num_bytes;
	}
}

/*
 * Terminate source --- called by jpeg_finish_decompress
 * after all data has been read.
 */
static void term_source (j_decompress_ptr cinfo)
{
	/* We don't actually need to do anything */
	return;
}

/*
 * Prepare for input from a stdio stream.
 * The caller must have already opened the stream, and is responsible
 * for closing it after finishing decompression.
 */
static void jpeg_SDL_RW_src (j_decompress_ptr cinfo, SDL_RWops *ctx)
{
  my_source_mgr *src;

  /* The source object and input buffer are made permanent so that a series
   * of JPEG images can be read from the same file by calling jpeg_stdio_src
   * only before the first one.  (If we discarded the buffer at the end of
   * one image, we'd likely lose the start of the next one.)
   * This makes it unsafe to use this manager and a different source
   * manager serially with the same JPEG object.  Caveat programmer.
   */
  if (cinfo->src == NULL) {	/* first time for this JPEG object? */
    cinfo->src = (struct jpeg_source_mgr *)
      (*cinfo->mem->alloc_small) ((j_common_ptr) cinfo, JPOOL_PERMANENT,
				  sizeof(my_source_mgr));
    src = (my_source_mgr *) cinfo->src;
  }

  src = (my_source_mgr *) cinfo->src;
  src->pub.init_source = init_source;
  src->pub.fill_input_buffer = fill_input_buffer;
  src->pub.skip_input_data = skip_input_data;
  src->pub.resync_to_restart = lib.jpeg_resync_to_restart; /* use default method */
  src->pub.term_source = term_source;
  src->ctx = ctx;
  src->pub.bytes_in_buffer = 0; /* forces fill_input_buffer on first read */
  src->pub.next_input_byte = NULL; /* until buffer loaded */
}

struct my_error_mgr {
	struct jpeg_error_mgr errmgr;
	jmp_buf escape;
};

static void my_error_exit(j_common_ptr cinfo)
{
	struct my_error_mgr *err = (struct my_error_mgr *)cinfo->err;
	longjmp(err->escape, 1);
}

static void output_no_message(j_common_ptr cinfo)
{
	/* do nothing */
}

/* Load a JPEG type image from an SDL datasource */
SDL_Surface *IMG_LoadJPG_RW(SDL_RWops *src)
{
	int start;
	struct jpeg_decompress_struct cinfo;
	JSAMPROW rowptr[1];
	SDL_Surface *volatile surface = NULL;
	struct my_error_mgr jerr;

	if ( !src ) {
		/* The error message has been set in SDL_RWFromFile */
		return NULL;
	}
	start = SDL_RWtell(src);

	if ( !IMG_Init(IMG_INIT_JPG) ) {
		return NULL;
	}

	/* Create a decompression structure and load the JPEG header */
	cinfo.err = lib.jpeg_std_error(&jerr.errmgr);
	jerr.errmgr.error_exit = my_error_exit;
	jerr.errmgr.output_message = output_no_message;
	if(setjmp(jerr.escape)) {
		/* If we get here, libjpeg found an error */
		lib.jpeg_destroy_decompress(&cinfo);
		if ( surface != NULL ) {
			SDL_FreeSurface(surface);
		}
		SDL_RWseek(src, start, RW_SEEK_SET);
		IMG_SetError("JPEG loading error");
		return NULL;
	}

	lib.jpeg_create_decompress(&cinfo);
	jpeg_SDL_RW_src(&cinfo, src);
	lib.jpeg_read_header(&cinfo, TRUE);

	if(cinfo.num_components == 4) {
		/* Set 32-bit Raw output */
		cinfo.out_color_space = JCS_CMYK;
		cinfo.quantize_colors = FALSE;
		lib.jpeg_calc_output_dimensions(&cinfo);

		/* Allocate an output surface to hold the image */
		surface = SDL_AllocSurface(SDL_SWSURFACE,
		        cinfo.output_width, cinfo.output_height, 32,
#if SDL_BYTEORDER == SDL_LIL_ENDIAN
		                   0x00FF0000, 0x0000FF00, 0x000000FF, 0xFF000000);
#else
		                   0x0000FF00, 0x00FF0000, 0xFF000000, 0x000000FF);
#endif
	} else {
		/* Set 24-bit RGB output */
		cinfo.out_color_space = JCS_RGB;
		cinfo.quantize_colors = FALSE;
#ifdef FAST_JPEG
		cinfo.scale_num   = 1;
		cinfo.scale_denom = 1;
		cinfo.dct_method = JDCT_FASTEST;
		cinfo.do_fancy_upsampling = FALSE;
#endif
		lib.jpeg_calc_output_dimensions(&cinfo);

		/* Allocate an output surface to hold the image */
		surface = SDL_AllocSurface(SDL_SWSURFACE,
		        cinfo.output_width, cinfo.output_height, 24,
#if SDL_BYTEORDER == SDL_LIL_ENDIAN
		                   0x0000FF, 0x00FF00, 0xFF0000,
#else
		                   0xFF0000, 0x00FF00, 0x0000FF,
#endif
		                   0);
	}

	if ( surface == NULL ) {
		lib.jpeg_destroy_decompress(&cinfo);
		SDL_RWseek(src, start, RW_SEEK_SET);
		IMG_SetError("Out of memory");
		return NULL;
	}

	/* Decompress the image */
	lib.jpeg_start_decompress(&cinfo);
	while ( cinfo.output_scanline < cinfo.output_height ) {
		rowptr[0] = (JSAMPROW)(Uint8 *)surface->pixels +
		                    cinfo.output_scanline * surface->pitch;
		lib.jpeg_read_scanlines(&cinfo, rowptr, (JDIMENSION) 1);
	}
	lib.jpeg_finish_decompress(&cinfo);
	lib.jpeg_destroy_decompress(&cinfo);

	return(surface);
}

#else

int IMG_InitJPG()
{
	IMG_SetError("JPEG images are not supported");
	return(-1);
}

void IMG_QuitJPG()
{
}

/* See if an image is contained in a data source */
int IMG_isJPG(SDL_RWops *src)
{
	return(0);
}

/* Load a JPEG type image from an SDL datasource */
SDL_Surface *IMG_LoadJPG_RW(SDL_RWops *src)
{
	return(NULL);
}

#endif /* LOAD_JPG */

#endif /* !defined(__APPLE__) || defined(SDL_IMAGE_USE_COMMON_BACKEND) */
