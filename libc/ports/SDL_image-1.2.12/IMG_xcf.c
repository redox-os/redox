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

/* This is a XCF image file loading framework */

#include <stdio.h>
#include <ctype.h>
#include <string.h>
#include <stdlib.h>

#include "SDL_endian.h"
#include "SDL_image.h"

#ifdef LOAD_XCF

#if DEBUG
static char prop_names [][30] = {
  "end",
  "colormap",
  "active_layer",
  "active_channel",
  "selection",
  "floating_selection",
  "opacity",
  "mode",
  "visible",
  "linked",
  "preserve_transparency",
  "apply_mask",
  "edit_mask",
  "show_mask",
  "show_masked",
  "offsets",
  "color",
  "compression",
  "guides",
  "resolution",
  "tattoo",
  "parasites",
  "unit",
  "paths",
  "user_unit"
};
#endif


typedef enum
{
  PROP_END = 0,
  PROP_COLORMAP = 1,
  PROP_ACTIVE_LAYER = 2,
  PROP_ACTIVE_CHANNEL = 3,
  PROP_SELECTION = 4,
  PROP_FLOATING_SELECTION = 5,
  PROP_OPACITY = 6,
  PROP_MODE = 7,
  PROP_VISIBLE = 8,
  PROP_LINKED = 9,
  PROP_PRESERVE_TRANSPARENCY = 10,
  PROP_APPLY_MASK = 11,
  PROP_EDIT_MASK = 12,
  PROP_SHOW_MASK = 13,
  PROP_SHOW_MASKED = 14,
  PROP_OFFSETS = 15,
  PROP_COLOR = 16,
  PROP_COMPRESSION = 17,
  PROP_GUIDES = 18,
  PROP_RESOLUTION = 19,
  PROP_TATTOO = 20,
  PROP_PARASITES = 21,
  PROP_UNIT = 22,
  PROP_PATHS = 23,
  PROP_USER_UNIT = 24
} xcf_prop_type;

typedef enum {
  COMPR_NONE    = 0,
  COMPR_RLE     = 1,
  COMPR_ZLIB    = 2,
  COMPR_FRACTAL = 3
} xcf_compr_type;

typedef enum {
  IMAGE_RGB       = 0,
  IMAGE_GREYSCALE = 1,
  IMAGE_INDEXED   = 2
} xcf_image_type;

typedef struct {
  Uint32 id;
  Uint32 length;
  union {
    struct {
      Uint32 num;
      char * cmap;
    } colormap; // 1
    struct {
      Uint32 drawable_offset;
    } floating_selection; // 5
    Sint32 opacity;
    Sint32 mode;
    int    visible;
    int    linked;
    int    preserve_transparency;
    int    apply_mask;
    int    show_mask;
    struct {
      Sint32 x;
      Sint32 y;
    } offset;
    unsigned char color [3];
    Uint8 compression;
    struct {
      Sint32 x;
      Sint32 y;
    } resolution;
    struct {
      char * name;
      Uint32 flags;
      Uint32 size;
      char * data;
    } parasite;
  } data;
} xcf_prop;

typedef struct {
  char   sign [14];
  Uint32 width;
  Uint32 height;
  Sint32 image_type;
  xcf_prop * properties;

  Uint32 * layer_file_offsets;
  Uint32 * channel_file_offsets;

  xcf_compr_type compr;
  Uint32         cm_num;
  unsigned char * cm_map;
} xcf_header;

typedef struct {
  Uint32 width;
  Uint32 height;
  Sint32 layer_type;
  char * name;
  xcf_prop * properties;

  Uint32 hierarchy_file_offset;
  Uint32 layer_mask_offset;

  Uint32 offset_x;
  Uint32 offset_y;
  int visible;
} xcf_layer;

typedef struct {
  Uint32 width;
  Uint32 height;
  char * name;
  xcf_prop * properties;

  Uint32 hierarchy_file_offset;

  Uint32 color;
  Uint32 opacity;
  int selection;
  int visible;
} xcf_channel;

typedef struct {
  Uint32 width;
  Uint32 height;
  Uint32 bpp;

  Uint32 * level_file_offsets;
} xcf_hierarchy;

typedef struct {
  Uint32 width;
  Uint32 height;

  Uint32 * tile_file_offsets;
} xcf_level;

typedef unsigned char * xcf_tile;

typedef unsigned char * (* load_tile_type) (SDL_RWops *, Uint32, int, int, int);


/* See if an image is contained in a data source */
int IMG_isXCF(SDL_RWops *src)
{
	int start;
	int is_XCF;
	char magic[14];

	if ( !src )
		return 0;
	start = SDL_RWtell(src);
	is_XCF = 0;
	if ( SDL_RWread(src, magic, sizeof(magic), 1) ) {
		if (strncmp(magic, "gimp xcf ", 9) == 0) {
			is_XCF = 1;
		}
	}
	SDL_RWseek(src, start, RW_SEEK_SET);
	return(is_XCF);
}

static char * read_string (SDL_RWops * src) {
  Uint32 tmp;
  char * data;

  tmp = SDL_ReadBE32 (src);
  if (tmp > 0) {
    data = (char *) malloc (sizeof (char) * tmp);
    SDL_RWread (src, data, tmp, 1);
  }
  else {
    data = NULL;
  }

  return data;
}


static Uint32 Swap32 (Uint32 v) {
  return
    ((v & 0x000000FF) << 16)
    |  ((v & 0x0000FF00))
    |  ((v & 0x00FF0000) >> 16)
    |  ((v & 0xFF000000));
}

static void xcf_read_property (SDL_RWops * src, xcf_prop * prop) {
  prop->id = SDL_ReadBE32 (src);
  prop->length = SDL_ReadBE32 (src);

#if DEBUG
  printf ("%.8X: %s: %d\n", SDL_RWtell (src), prop->id < 25 ? prop_names [prop->id] : "unknown", prop->length);
#endif

  switch (prop->id) {
  case PROP_COLORMAP:
    prop->data.colormap.num = SDL_ReadBE32 (src);
    prop->data.colormap.cmap = (char *) malloc (sizeof (char) * prop->data.colormap.num * 3);
    SDL_RWread (src, prop->data.colormap.cmap, prop->data.colormap.num*3, 1);
    break;

  case PROP_OFFSETS:
    prop->data.offset.x = SDL_ReadBE32 (src);
    prop->data.offset.y = SDL_ReadBE32 (src);
    break;
  case PROP_OPACITY:
    prop->data.opacity = SDL_ReadBE32 (src);
    break;
  case PROP_COMPRESSION:
  case PROP_COLOR:
    SDL_RWread (src, &prop->data, prop->length, 1);
    break;
  case PROP_VISIBLE:
    prop->data.visible = SDL_ReadBE32 (src);
    break;
  default:
    //    SDL_RWread (src, &prop->data, prop->length, 1);
    SDL_RWseek (src, prop->length, RW_SEEK_CUR);
  }
}

static void free_xcf_header (xcf_header * h) {
  if (h->cm_num)
    free (h->cm_map);

  free (h);
}

static xcf_header * read_xcf_header (SDL_RWops * src) {
  xcf_header * h;
  xcf_prop prop;

  h = (xcf_header *) malloc (sizeof (xcf_header));
  SDL_RWread (src, h->sign, 14, 1);
  h->width       = SDL_ReadBE32 (src);
  h->height      = SDL_ReadBE32 (src);
  h->image_type  = SDL_ReadBE32 (src);

  h->properties = NULL;
  h->compr      = COMPR_NONE;
  h->cm_num = 0;
  h->cm_map = NULL;

  // Just read, don't save
  do {
    xcf_read_property (src, &prop);
    if (prop.id == PROP_COMPRESSION)
      h->compr = prop.data.compression;
    else if (prop.id == PROP_COLORMAP) {
      // unused var: int i;

      h->cm_num = prop.data.colormap.num;
      h->cm_map = (unsigned char *) malloc (sizeof (unsigned char) * 3 * h->cm_num);
      memcpy (h->cm_map, prop.data.colormap.cmap, 3*sizeof (char)*h->cm_num);
      free (prop.data.colormap.cmap);
    }
  } while (prop.id != PROP_END);

  return h;
}

static void free_xcf_layer (xcf_layer * l) {
  free (l->name);
  free (l);
}

static xcf_layer * read_xcf_layer (SDL_RWops * src) {
  xcf_layer * l;
  xcf_prop    prop;

  l = (xcf_layer *) malloc (sizeof (xcf_layer));
  l->width  = SDL_ReadBE32 (src);
  l->height = SDL_ReadBE32 (src);
  l->layer_type = SDL_ReadBE32 (src);

  l->name = read_string (src);

  do {
    xcf_read_property (src, &prop);
    if (prop.id == PROP_OFFSETS) {
      l->offset_x = prop.data.offset.x;
      l->offset_y = prop.data.offset.y;
    } else if (prop.id == PROP_VISIBLE) {
      l->visible = prop.data.visible ? 1 : 0;
    }
  } while (prop.id != PROP_END);

  l->hierarchy_file_offset = SDL_ReadBE32 (src);
  l->layer_mask_offset     = SDL_ReadBE32 (src);

  return l;
}

static void free_xcf_channel (xcf_channel * c) {
  free (c->name);
  free (c);
}

static xcf_channel * read_xcf_channel (SDL_RWops * src) {
  xcf_channel * l;
  xcf_prop    prop;

  l = (xcf_channel *) malloc (sizeof (xcf_channel));
  l->width  = SDL_ReadBE32 (src);
  l->height = SDL_ReadBE32 (src);

  l->name = read_string (src);

  l->selection = 0;
  do {
    xcf_read_property (src, &prop);
    switch (prop.id) {
    case PROP_OPACITY:
      l->opacity = prop.data.opacity << 24;
      break;
    case PROP_COLOR:
      l->color = ((Uint32) prop.data.color[0] << 16)
	| ((Uint32) prop.data.color[1] << 8)
	| ((Uint32) prop.data.color[2]);
      break;
    case PROP_SELECTION:
      l->selection = 1;
      break;
    case PROP_VISIBLE:
      l->visible = prop.data.visible ? 1 : 0;
      break;
    default:
        ;
    }
  } while (prop.id != PROP_END);

  l->hierarchy_file_offset = SDL_ReadBE32 (src);

  return l;
}

static void free_xcf_hierarchy (xcf_hierarchy * h) {
  free (h->level_file_offsets);
  free (h);
}

static xcf_hierarchy * read_xcf_hierarchy (SDL_RWops * src) {
  xcf_hierarchy * h;
  int i;

  h = (xcf_hierarchy *) malloc (sizeof (xcf_hierarchy));
  h->width  = SDL_ReadBE32 (src);
  h->height = SDL_ReadBE32 (src);
  h->bpp    = SDL_ReadBE32 (src);

  h->level_file_offsets = NULL;
  i = 0;
  do {
    h->level_file_offsets = (Uint32 *) realloc (h->level_file_offsets, sizeof (Uint32) * (i+1));
    h->level_file_offsets [i] = SDL_ReadBE32 (src);
  } while (h->level_file_offsets [i++]);

  return h;
}

static void free_xcf_level (xcf_level * l) {
  free (l->tile_file_offsets);
  free (l);
}

static xcf_level * read_xcf_level (SDL_RWops * src) {
  xcf_level * l;
  int i;

  l = (xcf_level *) malloc (sizeof (xcf_level));
  l->width  = SDL_ReadBE32 (src);
  l->height = SDL_ReadBE32 (src);

  l->tile_file_offsets = NULL;
  i = 0;
  do {
    l->tile_file_offsets = (Uint32 *) realloc (l->tile_file_offsets, sizeof (Uint32) * (i+1));
    l->tile_file_offsets [i] = SDL_ReadBE32 (src);
  } while (l->tile_file_offsets [i++]);

  return l;
}

static void free_xcf_tile (unsigned char * t) {
  free (t);
}

static unsigned char * load_xcf_tile_none (SDL_RWops * src, Uint32 len, int bpp, int x, int y) {
  unsigned char * load;

  load = (unsigned char *) malloc (len); // expect this is okay
  SDL_RWread (src, load, len, 1);

  return load;
}

static unsigned char * load_xcf_tile_rle (SDL_RWops * src, Uint32 len, int bpp, int x, int y) {
  unsigned char * load, * t, * data, * d;
  Uint32 reallen;
  int i, size, count, j, length;
  unsigned char val;

  t = load = (unsigned char *) malloc (len);
  reallen = SDL_RWread (src, t, 1, len);

  data = (unsigned char *) malloc (x*y*bpp);
  for (i = 0; i < bpp; i++) {
    d    = data + i;
    size = x*y;
    count = 0;
 
    while (size > 0) {
      val = *t++;

      length = val;
      if (length >= 128) {
	length = 255 - (length - 1);
	if (length == 128) {
	  length = (*t << 8) + t[1];
	  t += 2;
	}

	count += length;
	size -= length;

	while (length-- > 0) {
	  *d = *t++;
	  d += bpp;
	}
      }
      else {
	length += 1;
	if (length == 128) {
	  length = (*t << 8) + t[1];
	  t += 2;
	}

	count += length;
	size -= length;

	val = *t++;

	for (j = 0; j < length; j++) {
	  *d = val;
	  d += bpp;
	}
      }
    }
  }

  free (load);
  return (data);
}

static Uint32 rgb2grey (Uint32 a) {
  Uint8 l;
  l = 0.2990 * ((a && 0x00FF0000) >> 16)
    + 0.5870 * ((a && 0x0000FF00) >>  8)
    + 0.1140 * ((a && 0x000000FF));

  return (l << 16) | (l << 8) | l;
}

static void create_channel_surface (SDL_Surface * surf, xcf_image_type itype, Uint32 color, Uint32 opacity) {
  Uint32 c = 0;

  switch (itype) {
  case IMAGE_RGB:
  case IMAGE_INDEXED:
    c = opacity | color;
    break;
  case IMAGE_GREYSCALE:
    c = opacity | rgb2grey (color);
    break;
  }
  SDL_FillRect (surf, NULL, c);
}

static int do_layer_surface (SDL_Surface * surface, SDL_RWops * src, xcf_header * head, xcf_layer * layer, load_tile_type load_tile) {
  xcf_hierarchy * hierarchy;
  xcf_level     * level;
  unsigned char * tile;
  Uint8  * p8;
  Uint16 * p16;
  Uint32 * p;
  int x, y, tx, ty, ox, oy, i, j;
  Uint32 *row;

  SDL_RWseek (src, layer->hierarchy_file_offset, RW_SEEK_SET);
  hierarchy = read_xcf_hierarchy (src);

  level = NULL;
  for (i = 0; hierarchy->level_file_offsets [i]; i++) {
    SDL_RWseek (src, hierarchy->level_file_offsets [i], RW_SEEK_SET);
    level = read_xcf_level (src);

    ty = tx = 0;
    for (j = 0; level->tile_file_offsets [j]; j++) {
      SDL_RWseek (src, level->tile_file_offsets [j], RW_SEEK_SET);
      ox = tx+64 > level->width ? level->width % 64 : 64;
      oy = ty+64 > level->height ? level->height % 64 : 64;

      if (level->tile_file_offsets [j+1]) {
	tile = load_tile
	  (src,
	   level->tile_file_offsets [j+1] - level->tile_file_offsets [j],
	   hierarchy->bpp,
	   ox, oy);
      }
      else {
	tile = load_tile
	  (src,
	   ox*oy*6,
	   hierarchy->bpp,
	   ox, oy);
      }

      p8  = tile;
      p16 = (Uint16 *) p8;
      p   = (Uint32 *) p8;
      for (y=ty; y < ty+oy; y++) {
	row = (Uint32 *)((Uint8 *)surface->pixels + y*surface->pitch + tx*4);
	switch (hierarchy->bpp) {
	case 4:
	  for (x=tx; x < tx+ox; x++)
	    *row++ = Swap32 (*p++);
	  break;
	case 3:
	  for (x=tx; x < tx+ox; x++) {
	    *row = 0xFF000000;
	    *row |= ((Uint32) *(p8++) << 16);
	    *row |= ((Uint32) *(p8++) << 8);
	    *row |= ((Uint32) *(p8++) << 0);
	    row++;
	  }
	  break;
	case 2: // Indexed/Greyscale + Alpha
	  switch (head->image_type) {
	  case IMAGE_INDEXED:
	    for (x=tx; x < tx+ox; x++) {
	      *row =  ((Uint32) (head->cm_map [*p8*3])     << 16);
	      *row |= ((Uint32) (head->cm_map [*p8*3+1])   << 8);
	      *row |= ((Uint32) (head->cm_map [*p8++*3+2]) << 0);
	      *row |= ((Uint32) *p8++ << 24);;
	      row++;
	    }
	    break;
	  case IMAGE_GREYSCALE:
	    for (x=tx; x < tx+ox; x++) {
	      *row = ((Uint32) *p8 << 16);
	      *row |= ((Uint32) *p8 << 8);
	      *row |= ((Uint32) *p8++ << 0);
	      *row |= ((Uint32) *p8++ << 24);;
	      row++;
	    }
	    break;	    
	  default:
	    fprintf (stderr, "Unknown Gimp image type (%d)\n", head->image_type);
	    return 1;
	  }
	  break;
	case 1: // Indexed/Greyscale
	  switch (head->image_type) {
	  case IMAGE_INDEXED:
	    for (x = tx; x < tx+ox; x++) {
	      *row++ = 0xFF000000
		| ((Uint32) (head->cm_map [*p8*3]) << 16)
		| ((Uint32) (head->cm_map [*p8*3+1]) << 8)
		| ((Uint32) (head->cm_map [*p8*3+2]) << 0);
	      p8++;
	    }
	    break;
	  case IMAGE_GREYSCALE:
	    for (x=tx; x < tx+ox; x++) {
	      *row++ = 0xFF000000
		| (((Uint32) (*p8)) << 16)
		| (((Uint32) (*p8)) << 8)
		| (((Uint32) (*p8)) << 0);
			++p8;
	    }
	    break;	    
	  default:
	    fprintf (stderr, "Unknown Gimp image type (%d)\n", head->image_type);
	    return 1;
	  }
	  break;
	}
      }
      tx += 64;
      if (tx >= level->width) {
	tx = 0;
	ty += 64;
      }
      if (ty >= level->height) {
	break;
      }

      free_xcf_tile (tile);
    }
    free_xcf_level (level);
  }

  free_xcf_hierarchy (hierarchy);
  
  return 0;
}

SDL_Surface *IMG_LoadXCF_RW(SDL_RWops *src)
{
  int start;
  const char *error = NULL;
  SDL_Surface *surface, *lays;
  xcf_header * head;
  xcf_layer  * layer;
  xcf_channel ** channel;
  int chnls, i, offsets;
  Uint32 offset, fp;

  unsigned char * (* load_tile) (SDL_RWops *, Uint32, int, int, int);

  if ( !src ) {
    /* The error message has been set in SDL_RWFromFile */
    return NULL;
  }
  start = SDL_RWtell(src);

  /* Initialize the data we will clean up when we're done */
  surface = NULL;

  head = read_xcf_header (src);

  switch (head->compr) {
  case COMPR_NONE:
    load_tile = load_xcf_tile_none;
    break;
  case COMPR_RLE:
    load_tile = load_xcf_tile_rle;
    break;
  default:
    fprintf (stderr, "Unsupported Compression.\n");
    free_xcf_header (head);
    return NULL;
  }

  /* Create the surface of the appropriate type */
  surface = SDL_AllocSurface(SDL_SWSURFACE, head->width, head->height, 32,
			     0x00FF0000,0x0000FF00,0x000000FF,0xFF000000);

  if ( surface == NULL ) {
    error = "Out of memory";
    goto done;
  }

  head->layer_file_offsets = NULL;
  offsets = 0;

  while ((offset = SDL_ReadBE32 (src))) {
    head->layer_file_offsets = (Uint32 *) realloc (head->layer_file_offsets, sizeof (Uint32) * (offsets+1));
    head->layer_file_offsets [offsets] = offset;
    offsets++;
  }
  fp = SDL_RWtell (src);
 
  lays = SDL_AllocSurface(SDL_SWSURFACE, head->width, head->height, 32,
			  0x00FF0000,0x0000FF00,0x000000FF,0xFF000000);

  if ( lays == NULL ) {
    error = "Out of memory";
    goto done;
  }

  // Blit layers backwards, because Gimp saves them highest first
  for (i = offsets; i > 0; i--) {
    SDL_Rect rs, rd;
    SDL_RWseek (src, head->layer_file_offsets [i-1], RW_SEEK_SET);

    layer = read_xcf_layer (src);
    do_layer_surface (lays, src, head, layer, load_tile);
    rs.x = 0;
    rs.y = 0;
    rs.w = layer->width;
    rs.h = layer->height;
    rd.x = layer->offset_x;
    rd.y = layer->offset_y;
    rd.w = layer->width;
    rd.h = layer->height;

    if (layer->visible)
      SDL_BlitSurface (lays, &rs, surface, &rd);
    free_xcf_layer (layer);
  }

  SDL_FreeSurface (lays);

  SDL_RWseek (src, fp, RW_SEEK_SET);

  // read channels
  channel = NULL;
  chnls   = 0;
  while ((offset = SDL_ReadBE32 (src))) {
    channel = (xcf_channel **) realloc (channel, sizeof (xcf_channel *) * (chnls+1));
    fp = SDL_RWtell (src);
    SDL_RWseek (src, offset, RW_SEEK_SET);
    channel [chnls++] = (read_xcf_channel (src));
    SDL_RWseek (src, fp, RW_SEEK_SET);    
  }

  if (chnls) {
    SDL_Surface * chs;

    chs = SDL_AllocSurface(SDL_SWSURFACE, head->width, head->height, 32,
			   0x00FF0000,0x0000FF00,0x000000FF,0xFF000000);

    if (chs == NULL) {
      error = "Out of memory";
      goto done;
    }
    for (i = 0; i < chnls; i++) {
      //      printf ("CNLBLT %i\n", i);
      if (!channel [i]->selection && channel [i]->visible) {
	create_channel_surface (chs, head->image_type, channel [i]->color, channel [i]->opacity);
	SDL_BlitSurface (chs, NULL, surface, NULL);
      }
      free_xcf_channel (channel [i]);
    }

    SDL_FreeSurface (chs);
  }

done:
  free_xcf_header (head);
  if ( error ) {
    SDL_RWseek(src, start, RW_SEEK_SET);
    if ( surface ) {
      SDL_FreeSurface(surface);
      surface = NULL;
    }
    IMG_SetError(error);
  }

  return(surface);
}

#else

/* See if an image is contained in a data source */
int IMG_isXCF(SDL_RWops *src)
{
  return(0);
}

/* Load a XCF type image from an SDL datasource */
SDL_Surface *IMG_LoadXCF_RW(SDL_RWops *src)
{
  return(NULL);
}

#endif /* LOAD_XCF */
