/*
 * Test OSMesa interface at 8, 16 and 32 bits/channel.
 *
 * Usage: osdemo [options]
 *
 * Options:
 *   -f   generate image files
 *   -g   render gradient and print color values
 */

#include <assert.h>
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <GL/osmesa.h>
#include <GL/glu.h>
#include <orbital.h>

#define WIDTH 600
#define HEIGHT 600

static GLboolean DisplayImages = GL_FALSE;
static GLboolean WriteFiles = GL_FALSE;
static GLboolean Gradient = GL_FALSE;


static void
Sphere(float radius, int slices, int stacks)
{
   GLUquadric *q = gluNewQuadric();
   gluQuadricNormals(q, GLU_SMOOTH);
   gluSphere(q, radius, slices, stacks);
   gluDeleteQuadric(q);
}


static void
Cone(float base, float height, int slices, int stacks)
{
   GLUquadric *q = gluNewQuadric();
   gluQuadricDrawStyle(q, GLU_FILL);
   gluQuadricNormals(q, GLU_SMOOTH);
   gluCylinder(q, base, 0.0, height, slices, stacks);
   gluDeleteQuadric(q);
}


static void
Torus(float innerRadius, float outerRadius, int sides, int rings)
{
   /* from GLUT... */
   int i, j;
   GLfloat theta, phi, theta1;
   GLfloat cosTheta, sinTheta;
   GLfloat cosTheta1, sinTheta1;
   const GLfloat ringDelta = 2.0 * M_PI / rings;
   const GLfloat sideDelta = 2.0 * M_PI / sides;

   theta = 0.0;
   cosTheta = 1.0;
   sinTheta = 0.0;
   for (i = rings - 1; i >= 0; i--) {
      theta1 = theta + ringDelta;
      cosTheta1 = cos(theta1);
      sinTheta1 = sin(theta1);
      glBegin(GL_QUAD_STRIP);
      phi = 0.0;
      for (j = sides; j >= 0; j--) {
         GLfloat cosPhi, sinPhi, dist;

         phi += sideDelta;
         cosPhi = cos(phi);
         sinPhi = sin(phi);
         dist = outerRadius + innerRadius * cosPhi;

         glNormal3f(cosTheta1 * cosPhi, -sinTheta1 * cosPhi, sinPhi);
         glVertex3f(cosTheta1 * dist, -sinTheta1 * dist, innerRadius * sinPhi);
         glNormal3f(cosTheta * cosPhi, -sinTheta * cosPhi, sinPhi);
         glVertex3f(cosTheta * dist, -sinTheta * dist,  innerRadius * sinPhi);
      }
      glEnd();
      theta = theta1;
      cosTheta = cosTheta1;
      sinTheta = sinTheta1;
   }
}


static void Cube(float size)
{
   size = 0.5 * size;

   glBegin(GL_QUADS);
   /* +X face */
   glNormal3f(1, 0, 0);
   glVertex3f(size, -size,  size);
   glVertex3f(size, -size, -size);
   glVertex3f(size,  size, -size);
   glVertex3f(size,  size,  size);

   /* -X face */
   glNormal3f(-1, 0, 0);
   glVertex3f(-size,  size,  size);
   glVertex3f(-size,  size, -size);
   glVertex3f(-size, -size, -size);
   glVertex3f(-size, -size,  size);

   /* +Y face */
   glNormal3f(0, 1, 0);
   glVertex3f(-size, size,  size);
   glVertex3f( size, size,  size);
   glVertex3f( size, size, -size);
   glVertex3f(-size, size, -size);

   /* -Y face */
   glNormal3f(0, -1, 0);
   glVertex3f(-size, -size, -size);
   glVertex3f( size, -size, -size);
   glVertex3f( size, -size,  size);
   glVertex3f(-size, -size,  size);

   /* +Z face */
   glNormal3f(0, 0, 1);
   glVertex3f(-size, -size, size);
   glVertex3f( size, -size, size);
   glVertex3f( size,  size, size);
   glVertex3f(-size,  size, size);

   /* -Z face */
   glNormal3f(0, 0, -1);
   glVertex3f(-size,  size, -size);
   glVertex3f( size,  size, -size);
   glVertex3f( size, -size, -size);
   glVertex3f(-size, -size, -size);

   glEnd();
}



/**
 * Draw red/green gradient across bottom of image.
 * Read pixels to check deltas.
 */
static void
render_gradient(void)
{
   GLfloat row[WIDTH][4];
   int i;

   glMatrixMode(GL_PROJECTION);
   glLoadIdentity();
   glOrtho(-1, 1, -1, 1, -1, 1);
   glMatrixMode(GL_MODELVIEW);
   glLoadIdentity();

   glBegin(GL_POLYGON);
   glColor3f(1, 0, 0);
   glVertex2f(-1, -1.0);
   glVertex2f(-1, -0.9);
   glColor3f(0, 1, 0);
   glVertex2f(1, -0.9);
   glVertex2f(1, -1.0);
   glEnd();
   glFinish();

   glReadPixels(0, 0, WIDTH, 1, GL_RGBA, GL_FLOAT, row);
   for (i = 0; i < 4; i++) {
      printf("row[i] = %f, %f, %f\n", row[i][0], row[i][1], row[i][2]);
   }
}


static void
render_image(void)
{
   static const GLfloat light_ambient[4] = { 0.0, 0.0, 0.0, 1.0 };
   static const GLfloat light_diffuse[4] = { 1.0, 1.0, 1.0, 1.0 };
   static const GLfloat light_specular[4] = { 1.0, 1.0, 1.0, 1.0 };
   static const GLfloat light_position[4] = { 1.0, 1.0, 1.0, 0.0 };
   static const GLfloat red_mat[4]   = { 1.0, 0.2, 0.2, 1.0 };
   static const GLfloat green_mat[4] = { 0.2, 1.0, 0.2, 1.0 };
   static const GLfloat blue_mat[4]  = { 0.2, 0.2, 1.0, 1.0 };
#if 0
   static const GLfloat yellow_mat[4]  = { 0.8, 0.8, 0.0, 1.0 };
#endif
   static const GLfloat purple_mat[4]  = { 0.8, 0.4, 0.8, 0.6 };

   glLightfv(GL_LIGHT0, GL_AMBIENT, light_ambient);
   glLightfv(GL_LIGHT0, GL_DIFFUSE, light_diffuse);
   glLightfv(GL_LIGHT0, GL_SPECULAR, light_specular);
   glLightfv(GL_LIGHT0, GL_POSITION, light_position);

   glEnable(GL_DEPTH_TEST);
   glEnable(GL_LIGHT0);

   glMatrixMode(GL_PROJECTION);
   glLoadIdentity();
   glFrustum(-1.0, 1.0, -1.0, 1.0, 2.0, 50.0);
   glMatrixMode(GL_MODELVIEW);
   glTranslatef(0, 0.5, -7);

   glClearColor(0.3, 0.3, 0.7, 0.0);
   glClear( GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT );

   glPushMatrix();
   glRotatef(20.0, 1.0, 0.0, 0.0);

   /* ground */
   glEnable(GL_TEXTURE_2D);
   glBegin(GL_POLYGON);
   glNormal3f(0, 1, 0);
   glTexCoord2f(0, 0);  glVertex3f(-5, -1, -5);
   glTexCoord2f(1, 0);  glVertex3f( 5, -1, -5);
   glTexCoord2f(1, 1);  glVertex3f( 5, -1,  5);
   glTexCoord2f(0, 1);  glVertex3f(-5, -1,  5);
   glEnd();
   glDisable(GL_TEXTURE_2D);

   glEnable(GL_LIGHTING);

   glPushMatrix();
   glTranslatef(-1.5, 0.5, 0.0);
   glRotatef(90.0, 1.0, 0.0, 0.0);
   glMaterialfv( GL_FRONT_AND_BACK, GL_AMBIENT_AND_DIFFUSE, red_mat );
   Torus(0.275, 0.85, 20, 20);
   glPopMatrix();

   glPushMatrix();
   glTranslatef(-1.5, -0.5, 0.0);
   glRotatef(270.0, 1.0, 0.0, 0.0);
   glMaterialfv( GL_FRONT_AND_BACK, GL_AMBIENT_AND_DIFFUSE, green_mat );
   Cone(1.0, 2.0, 16, 1);
   glPopMatrix();

   glPushMatrix();
   glTranslatef(0.95, 0.0, -0.8);
   glMaterialfv( GL_FRONT_AND_BACK, GL_AMBIENT_AND_DIFFUSE, blue_mat );
   glLineWidth(2.0);
   glPolygonMode(GL_FRONT_AND_BACK, GL_LINE);
   Sphere(1.2, 20, 20);
   glPolygonMode(GL_FRONT_AND_BACK, GL_FILL);
   glPopMatrix();

#if 0
   glPushMatrix();
   glTranslatef(0.75, 0.0, 1.3);
   glMaterialfv( GL_FRONT_AND_BACK, GL_AMBIENT_AND_DIFFUSE, yellow_mat );
   glutWireTeapot(1.0);
   glPopMatrix();
#endif

   glPushMatrix();
   glTranslatef(-0.25, 0.0, 2.5);
   glRotatef(40, 0, 1, 0);
   glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
   glEnable(GL_BLEND);
   glEnable(GL_CULL_FACE);
   glMaterialfv( GL_FRONT_AND_BACK, GL_AMBIENT_AND_DIFFUSE, purple_mat );
   Cube(1.0);
   glDisable(GL_BLEND);
   glDisable(GL_CULL_FACE);
   glPopMatrix();

   glDisable(GL_LIGHTING);

   glPopMatrix();

   glDisable(GL_DEPTH_TEST);
}


static void
init_context(void)
{
   const GLint texWidth = 64, texHeight = 64;
   GLubyte *texImage;
   int i, j;

   /* checker image */
   texImage = (GLubyte *)malloc(texWidth * texHeight * 4);
   for (i = 0; i < texHeight; i++) {
      for (j = 0; j < texWidth; j++) {
         int k = (i * texWidth + j) * 4;
         if ((i % 5) == 0 || (j % 5) == 0) {
            texImage[k+0] = 200;
            texImage[k+1] = 200;
            texImage[k+2] = 200;
            texImage[k+3] = 255;
         }
         else {
            if ((i % 5) == 1 || (j % 5) == 1) {
               texImage[k+0] = 50;
               texImage[k+1] = 50;
               texImage[k+2] = 50;
               texImage[k+3] = 255;
            }
            else {
               texImage[k+0] = 100;
               texImage[k+1] = 100;
               texImage[k+2] = 100;
               texImage[k+3] = 255;
            }
         }
      }
   }

   glTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA, texWidth, texHeight, 0,
                GL_RGBA, GL_UNSIGNED_BYTE, texImage);
   glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST);
   glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST);

   free(texImage);
}

static void
display_image(const char *filename, const GLubyte *buffer, int width, int height)
{
  void * window = orb_window_new(-1, -1, width, height, filename);

  uint32_t * frame_data = orb_window_data(window);
  uint32_t * image_data = (uint32_t *)buffer;

  int x, y;
  for(y = 0; y < height; y++) {
   for(x = 0; x < width; x++) {
     frame_data[y * width + x] = image_data[(height - 1 - y) * width + x] | 0xFF000000;
   }
  }

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
}

static void
write_ppm(const char *filename, const GLubyte *buffer, int width, int height)
{
   const int binary = 0;
   FILE *f = fopen( filename, "w" );
   if (f) {
      int i, x, y;
      const GLubyte *ptr = buffer;
      if (binary) {
         fprintf(f,"P6\n");
         fprintf(f,"# ppm-file created by osdemo.c\n");
         fprintf(f,"%i %i\n", width,height);
         fprintf(f,"255\n");
         fclose(f);
         f = fopen( filename, "ab" );  /* reopen in binary append mode */
         for (y=height-1; y>=0; y--) {
            for (x=0; x<width; x++) {
               i = (y*width + x) * 4;
               fputc(ptr[i], f);   /* write red */
               fputc(ptr[i+1], f); /* write green */
               fputc(ptr[i+2], f); /* write blue */
            }
         }
      }
      else {
         /*ASCII*/
         int counter = 0;
         fprintf(f,"P3\n");
         fprintf(f,"# ascii ppm file created by osdemo.c\n");
         fprintf(f,"%i %i\n", width, height);
         fprintf(f,"255\n");
         for (y=height-1; y>=0; y--) {
            for (x=0; x<width; x++) {
               i = (y*width + x) * 4;
               fprintf(f, " %3d %3d %3d", ptr[i], ptr[i+1], ptr[i+2]);
               counter++;
               if (counter % 5 == 0)
                  fprintf(f, "\n");
            }
         }
      }
      fclose(f);
   }
}


static GLboolean
test(GLenum type, GLint bits, const char *filename)
{
   const GLint z = 16, stencil = 0, accum = 0;
   OSMesaContext ctx;
   void *buffer;
   GLint cBits;

   assert(bits == 8 ||
          bits == 16 ||
          bits == 32);

   assert(type == GL_UNSIGNED_BYTE ||
          type == GL_UNSIGNED_SHORT ||
          type == GL_FLOAT);

   ctx = OSMesaCreateContextExt(OSMESA_BGRA, z, stencil, accum, NULL );
   if (!ctx) {
      printf("OSMesaCreateContextExt() failed!\n");
      return 0;
   }

   /* Allocate the image buffer */
   buffer = malloc(WIDTH * HEIGHT * 4 * bits / 8);
   if (!buffer) {
      printf("Alloc image buffer failed!\n");
      return 0;
   }

   /* Bind the buffer to the context and make it current */
   if (!OSMesaMakeCurrent( ctx, buffer, type, WIDTH, HEIGHT )) {
      printf("OSMesaMakeCurrent (%d bits/channel) failed!\n", bits);
      free(buffer);
      OSMesaDestroyContext(ctx);
      return 0;
   }

   /* sanity checks */
   glGetIntegerv(GL_RED_BITS, &cBits);
   if (cBits != bits) {
      fprintf(stderr, "Unable to create %d-bit/channel renderbuffer.\n", bits);
      fprintf(stderr, "May need to recompile Mesa with CHAN_BITS=16 or 32.\n");
      return 0;
   }
   glGetIntegerv(GL_GREEN_BITS, &cBits);
   assert(cBits == bits);
   glGetIntegerv(GL_BLUE_BITS, &cBits);
   assert(cBits == bits);
   glGetIntegerv(GL_ALPHA_BITS, &cBits);
   assert(cBits == bits);

   if (WriteFiles)
      printf("Rendering %d bit/channel image: %s\n", bits, filename);
   else
      printf("Rendering %d bit/channel image\n", bits);

   OSMesaColorClamp(GL_TRUE);

   init_context();
   render_image();
   if (Gradient)
      render_gradient();

   /* Make sure buffered commands are finished! */
   glFinish();

   if (DisplayImages && filename != NULL) {
      if (type == GL_UNSIGNED_SHORT) {
         GLushort *buffer16 = (GLushort *) buffer;
         GLubyte *buffer8 = (GLubyte *) malloc(WIDTH * HEIGHT * 4);
         int i;
         for (i = 0; i < WIDTH * HEIGHT * 4; i++)
            buffer8[i] = buffer16[i] >> 8;
         display_image(filename, buffer8, WIDTH, HEIGHT);
         free(buffer8);
      }
      else if (type == GL_FLOAT) {
         GLfloat *buffer32 = (GLfloat *) buffer;
         GLubyte *buffer8 = (GLubyte *) malloc(WIDTH * HEIGHT * 4);
         int i;
         /* colors may be outside [0,1] so we need to clamp */
         for (i = 0; i < WIDTH * HEIGHT * 4; i++)
            buffer8[i] = (GLubyte) (buffer32[i] * 255.0);
         display_image(filename, buffer8, WIDTH, HEIGHT);
         free(buffer8);
      }
      else {
         display_image(filename, (const GLubyte *)buffer, WIDTH, HEIGHT);
      }
   }

   if (WriteFiles && filename != NULL) {
      if (type == GL_UNSIGNED_SHORT) {
         GLushort *buffer16 = (GLushort *) buffer;
         GLubyte *buffer8 = (GLubyte *) malloc(WIDTH * HEIGHT * 4);
         int i;
         for (i = 0; i < WIDTH * HEIGHT * 4; i++)
            buffer8[i] = buffer16[i] >> 8;
         write_ppm(filename, buffer8, WIDTH, HEIGHT);
         free(buffer8);
      }
      else if (type == GL_FLOAT) {
         GLfloat *buffer32 = (GLfloat *) buffer;
         GLubyte *buffer8 = (GLubyte *) malloc(WIDTH * HEIGHT * 4);
         int i;
         /* colors may be outside [0,1] so we need to clamp */
         for (i = 0; i < WIDTH * HEIGHT * 4; i++)
            buffer8[i] = (GLubyte) (buffer32[i] * 255.0);
         write_ppm(filename, buffer8, WIDTH, HEIGHT);
         free(buffer8);
      }
      else {
         write_ppm(filename, (const GLubyte *)buffer, WIDTH, HEIGHT);
      }
   }

   OSMesaDestroyContext(ctx);

   free(buffer);

   return 1;
}


int
main( int argc, char *argv[] )
{
   int i;

   printf("Use -f to write image files\n");

   for (i = 1; i < argc; i++) {
      if (strcmp(argv[i], "-d") == 0)
         DisplayImages = GL_TRUE;
      else if (strcmp(argv[i], "-f") == 0)
         WriteFiles = GL_TRUE;
      else if (strcmp(argv[i], "-g") == 0)
         Gradient = GL_TRUE;
   }

   test(GL_UNSIGNED_BYTE, 8, "image8.ppm");
   test(GL_UNSIGNED_SHORT, 16, "image16.ppm");
   test(GL_FLOAT, 32, "image32.ppm");

   return 0;
}
