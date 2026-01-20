#include <stdint.h>
#include <math.h>
#include <stdlib.h>
#include <cairo/cairo.h>
#include <orbital.h>

#ifndef M_PI
#define M_PI 3.14159265
#endif

static int width = 800;
static int height = 600;

static void
travel_path (cairo_t *cr)
{

  cairo_pattern_t *pat;

  pat = cairo_pattern_create_linear (0.0, 0.0,  0.0, 256.0);
  cairo_pattern_add_color_stop_rgba (pat, 1, 0, 0, 0, 1);
  cairo_pattern_add_color_stop_rgba (pat, 0, 1, 1, 1, 1);
  cairo_rectangle (cr, 0, 0, 256, 256);
  cairo_set_source (cr, pat);
  cairo_fill (cr);
  cairo_pattern_destroy (pat);

  pat = cairo_pattern_create_radial (115.2, 102.4, 25.6,
                                    102.4,  102.4, 128.0);
  cairo_pattern_add_color_stop_rgba (pat, 0, 1, 1, 1, 1);
  cairo_pattern_add_color_stop_rgba (pat, 1, 0, 0, 0, 1);
  cairo_set_source (cr, pat);
  cairo_arc (cr, 128.0, 128.0, 76.8, 0, 2 * M_PI);
  cairo_fill (cr);
  cairo_pattern_destroy (pat);


  double x         = 305.6,        /* parameters like cairo_rectangle */
        y         = 25.6,
        width         = 204.8,
        height        = 204.8,
        aspect        = 1.0,     /* aspect ratio */
        corner_radius = height / 10.0;   /* and corner curvature radius */

  double radius = corner_radius / aspect;
  double degrees = M_PI / 180.0;

  cairo_new_sub_path (cr);
  cairo_arc (cr, x + width - radius, y + radius, radius, -90 * degrees, 0 * degrees);
  cairo_arc (cr, x + width - radius, y + height - radius, radius, 0 * degrees, 90 * degrees);
  cairo_arc (cr, x + radius, y + height - radius, radius, 90 * degrees, 180 * degrees);
  cairo_arc (cr, x + radius, y + radius, radius, 180 * degrees, 270 * degrees);
  cairo_close_path (cr);

  cairo_set_source_rgb (cr, 0.5, 0.5, 1);
  cairo_fill_preserve (cr);
  cairo_set_source_rgba (cr, 0.5, 0, 0, 0.5);
  cairo_set_line_width (cr, 10.0);
  cairo_stroke (cr);


  double xc = 128.0;
  double yc = 128.0;
  radius = 100.0;
  double angle1 = 45.0  * (M_PI/180.0);  /* angles are specified */
  double angle2 = 180.0 * (M_PI/180.0);  /* in radians           */

  cairo_set_line_width (cr, 10.0);
  cairo_arc (cr, xc, yc, radius, angle1, angle2);
  cairo_stroke (cr);

  /* draw helping lines */
  cairo_set_source_rgba (cr, 1, 0.2, 0.2, 0.6);
  cairo_set_line_width (cr, 6.0);

  cairo_arc (cr, xc, yc, 10.0, 0, 2*M_PI);
  cairo_fill (cr);

  cairo_arc (cr, xc, yc, radius, angle1, angle1);
  cairo_line_to (cr, xc, yc);
  cairo_arc (cr, xc, yc, radius, angle2, angle2);
  cairo_line_to (cr, xc, yc);
  cairo_stroke (cr);
}

static void 
draw (cairo_surface_t *surface)
{
  cairo_t *cr;
  cr = cairo_create (surface);
  travel_path (cr);
  cairo_destroy (cr);
}

int
main(int argc, char *argv[])
{
  void * window = orb_window_new(-1, -1, width, height, "CairoDemo");
  
  //Cairo
  uint32_t * frame_data = orb_window_data(window);
  cairo_surface_t *surface = cairo_image_surface_create_for_data((uint8_t*) frame_data, CAIRO_FORMAT_ARGB32, width, height, cairo_format_stride_for_width(CAIRO_FORMAT_ARGB32, width));
  cairo_create(surface);
  draw (surface);

  orb_window_sync(window);

  char running = 1;
  while (running) {
   void * event_iter = orb_window_events(window);

   OrbEventOption event_option;
   do {
     event_option = orb_events_next(event_iter);
     switch (event_option.tag) {
       case OrbEventOption_Quit:
         running = 0;
         break;
       default:
         break;
     }
   } while (running && event_option.tag != OrbEventOption_None);

   orb_events_destroy(event_iter);
  }
  orb_window_destroy(window);
  return 0;             /* ANSI C requires main to return int. */
}

