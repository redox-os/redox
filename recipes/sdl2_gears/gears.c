/* gears.c */

/*
 * 3-D gear wheels.  This program is in the public domain.
 *
 * Brian Paul
 */

/* Conversion to GLUT by Mark J. Kilgard */

#include <math.h>
#include <stdlib.h>
#include <GL/gl.h>
#include <GL/glu.h>
#include <SDL2/SDL.h>

#ifndef M_PI
#define M_PI 3.14159265
#endif

/**

  Draw a gear wheel.  You'll probably want to call this function when
  building a display list since we do a lot of trig here.

  Input:  inner_radius - radius of hole at center
          outer_radius - radius at center of teeth
          width - width of gear
          teeth - number of teeth
          tooth_depth - depth of tooth

 **/

static void
gear(GLfloat inner_radius, GLfloat outer_radius, GLfloat width,
     GLint teeth, GLfloat tooth_depth)
{
    GLint i;
    GLfloat r0, r1, r2;
    GLfloat angle, da;
    GLfloat u, v, len;

    r0 = inner_radius;
    r1 = outer_radius - tooth_depth / 2.0;
    r2 = outer_radius + tooth_depth / 2.0;

    da = 2.0 * M_PI / teeth / 4.0;

    glShadeModel(GL_FLAT);

    glNormal3f(0.0, 0.0, 1.0);

    /* draw front face */
    glBegin(GL_QUAD_STRIP);
    for (i = 0; i <= teeth; i++)
    {
        angle = i * 2.0 * M_PI / teeth;
        glVertex3f(r0 * cos(angle), r0 * sin(angle), width * 0.5);
        glVertex3f(r1 * cos(angle), r1 * sin(angle), width * 0.5);
        glVertex3f(r0 * cos(angle), r0 * sin(angle), width * 0.5);
        glVertex3f(r1 * cos(angle + 3 * da), r1 * sin(angle + 3 * da), width * 0.5);
    }
    glEnd();

    /* draw front sides of teeth */
    glBegin(GL_QUADS);
    da = 2.0 * M_PI / teeth / 4.0;
    for (i = 0; i < teeth; i++)
    {
        angle = i * 2.0 * M_PI / teeth;

        glVertex3f(r1 * cos(angle), r1 * sin(angle), width * 0.5);
        glVertex3f(r2 * cos(angle + da), r2 * sin(angle + da), width * 0.5);
        glVertex3f(r2 * cos(angle + 2 * da), r2 * sin(angle + 2 * da), width * 0.5);
        glVertex3f(r1 * cos(angle + 3 * da), r1 * sin(angle + 3 * da), width * 0.5);
    }
    glEnd();

    glNormal3f(0.0, 0.0, -1.0);

    /* draw back face */
    glBegin(GL_QUAD_STRIP);
    for (i = 0; i <= teeth; i++)
    {
        angle = i * 2.0 * M_PI / teeth;
        glVertex3f(r1 * cos(angle), r1 * sin(angle), -width * 0.5);
        glVertex3f(r0 * cos(angle), r0 * sin(angle), -width * 0.5);
        glVertex3f(r1 * cos(angle + 3 * da), r1 * sin(angle + 3 * da), -width * 0.5);
        glVertex3f(r0 * cos(angle), r0 * sin(angle), -width * 0.5);
    }
    glEnd();

    /* draw back sides of teeth */
    glBegin(GL_QUADS);
    da = 2.0 * M_PI / teeth / 4.0;
    for (i = 0; i < teeth; i++)
    {
        angle = i * 2.0 * M_PI / teeth;

        glVertex3f(r1 * cos(angle + 3 * da), r1 * sin(angle + 3 * da), -width * 0.5);
        glVertex3f(r2 * cos(angle + 2 * da), r2 * sin(angle + 2 * da), -width * 0.5);
        glVertex3f(r2 * cos(angle + da), r2 * sin(angle + da), -width * 0.5);
        glVertex3f(r1 * cos(angle), r1 * sin(angle), -width * 0.5);
    }
    glEnd();

    /* draw outward faces of teeth */
    glBegin(GL_QUAD_STRIP);
    for (i = 0; i < teeth; i++)
    {
        angle = i * 2.0 * M_PI / teeth;

        glVertex3f(r1 * cos(angle), r1 * sin(angle), width * 0.5);
        glVertex3f(r1 * cos(angle), r1 * sin(angle), -width * 0.5);
        u = r2 * cos(angle + da) - r1 * cos(angle);
        v = r2 * sin(angle + da) - r1 * sin(angle);
        len = sqrt(u * u + v * v);
        u /= len;
        v /= len;
        glNormal3f(v, -u, 0.0);
        glVertex3f(r2 * cos(angle + da), r2 * sin(angle + da), width * 0.5);
        glVertex3f(r2 * cos(angle + da), r2 * sin(angle + da), -width * 0.5);
        glNormal3f(cos(angle), sin(angle), 0.0);
        glVertex3f(r2 * cos(angle + 2 * da), r2 * sin(angle + 2 * da), width * 0.5);
        glVertex3f(r2 * cos(angle + 2 * da), r2 * sin(angle + 2 * da), -width * 0.5);
        u = r1 * cos(angle + 3 * da) - r2 * cos(angle + 2 * da);
        v = r1 * sin(angle + 3 * da) - r2 * sin(angle + 2 * da);
        glNormal3f(v, -u, 0.0);
        glVertex3f(r1 * cos(angle + 3 * da), r1 * sin(angle + 3 * da), width * 0.5);
        glVertex3f(r1 * cos(angle + 3 * da), r1 * sin(angle + 3 * da), -width * 0.5);
        glNormal3f(cos(angle), sin(angle), 0.0);
    }

    glVertex3f(r1 * cos(0), r1 * sin(0), width * 0.5);
    glVertex3f(r1 * cos(0), r1 * sin(0), -width * 0.5);

    glEnd();

    glShadeModel(GL_SMOOTH);

    /* draw inside radius cylinder */
    glBegin(GL_QUAD_STRIP);
    for (i = 0; i <= teeth; i++)
    {
        angle = i * 2.0 * M_PI / teeth;

        glNormal3f(-cos(angle), -sin(angle), 0.0);
        glVertex3f(r0 * cos(angle), r0 * sin(angle), -width * 0.5);
        glVertex3f(r0 * cos(angle), r0 * sin(angle), width * 0.5);
    }
    glEnd();
}

static int width = 800;
static int height = 600;

static SDL_Window *window = NULL;
static SDL_GLContext context = NULL;

static GLfloat view_rotx = 20.0, view_roty = 30.0, view_rotz = 0.0;
static GLint gear1, gear2, gear3;
static GLfloat angle = 0.0;
static GLfloat delta = 2.0f;

static void
draw(void)
{
    glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

    glPushMatrix();
    glRotatef(view_rotx, 1.0, 0.0, 0.0);
    glRotatef(view_roty, 0.0, 1.0, 0.0);
    glRotatef(view_rotz, 0.0, 0.0, 1.0);

    glPushMatrix();
    glTranslatef(-3.0, -2.0, 0.0);
    glRotatef(angle, 0.0, 0.0, 1.0);
    glCallList(gear1);
    glPopMatrix();

    glPushMatrix();
    glTranslatef(3.1, -2.0, 0.0);
    glRotatef(-2.0 * angle - 9.0, 0.0, 0.0, 1.0);
    glCallList(gear2);
    glPopMatrix();

    glPushMatrix();
    glTranslatef(-3.1, 4.2, 0.0);
    glRotatef(-2.0 * angle - 25.0, 0.0, 0.0, 1.0);
    glCallList(gear3);
    glPopMatrix();

    glPopMatrix();
}

static void
idle(void)
{
    angle += delta;
    if (angle > 360.0f)
        angle -= 360.0f;

    draw();

    glFlush();

    SDL_GL_SwapWindow(window);
}

/* new window size or exposure */
static void
reshape(int width, int height)
{
    GLfloat h = (GLfloat)height / (GLfloat)width;

    glViewport(0, 0, (GLint)width, (GLint)height);
    glMatrixMode(GL_PROJECTION);
    glLoadIdentity();
    glFrustum(-1.0, 1.0, -h, h, 5.0, 60.0);
    glMatrixMode(GL_MODELVIEW);
    glLoadIdentity();
    glTranslatef(0.0, 0.0, -40.0);
}

static void
init(void)
{
    static GLfloat pos[4] =
        {5.0, 5.0, 10.0, 0.0};
    static GLfloat red[4] =
        {0.8, 0.1, 0.0, 1.0};
    static GLfloat green[4] =
        {0.0, 0.8, 0.2, 1.0};
    static GLfloat blue[4] =
        {0.2, 0.2, 1.0, 1.0};

    glLightfv(GL_LIGHT0, GL_POSITION, pos);
    glEnable(GL_CULL_FACE);
    glEnable(GL_LIGHTING);
    glEnable(GL_LIGHT0);
    glEnable(GL_DEPTH_TEST);

    /* make the gears */
    gear1 = glGenLists(1);
    glNewList(gear1, GL_COMPILE);
    glMaterialfv(GL_FRONT, GL_AMBIENT_AND_DIFFUSE, red);
    gear(1.0, 4.0, 1.0, 20, 0.7);
    glEndList();

    gear2 = glGenLists(1);
    glNewList(gear2, GL_COMPILE);
    glMaterialfv(GL_FRONT, GL_AMBIENT_AND_DIFFUSE, green);
    gear(0.5, 2.0, 2.0, 10, 0.7);
    glEndList();

    gear3 = glGenLists(1);
    glNewList(gear3, GL_COMPILE);
    glMaterialfv(GL_FRONT, GL_AMBIENT_AND_DIFFUSE, blue);
    gear(1.3, 2.0, 0.5, 10, 0.7);
    glEndList();

    glEnable(GL_NORMALIZE);
}

void CheckSDLError(int line)
{
    const char *error = SDL_GetError();
    if (error != "")
    {
        printf("SLD Error: %s\n", error);

        if (line != -1)
            printf("\nLine: %d\n", line);

        SDL_ClearError();
    }
}

void audio_callback(void *userdata, Uint8 *stream, int len);
static Uint8 *audio_pos; // global pointer to the audio buffer to be played
static Uint32 audio_len; // remaining length of the sample we have to play

void audio_callback(void *userdata, Uint8 *stream, int len)
{
    if (audio_len == 0)
        return;

    len = (len > audio_len ? audio_len : len);
    SDL_memcpy (stream, audio_pos, len); 					// simply copy from one buffer into the other
    //SDL_MixAudio(stream, audio_pos, len, SDL_MIX_MAXVOLUME); //FIXME: broken in redox

    audio_pos += len;
    audio_len -= len;
}

int main(int argc, char *argv[])
{
    printf("Initializing SDL\n");
    if (SDL_Init(SDL_INIT_VIDEO | SDL_INIT_AUDIO) < 0)
    {
        printf("Failed to init SDL\n");
        CheckSDLError(__LINE__);
        return -1;
    }

    printf("Creating SDL window\n");
    window = SDL_CreateWindow(
        "Gears",
        -1,
        -1,
        width,
        height,
        SDL_WINDOW_OPENGL);
    if (window == NULL)
    {
        printf("Unable to create window\n");
        CheckSDLError(__LINE__);
        return -1;
    }

    printf("Creating SDL GL context\n");
    context = SDL_GL_CreateContext(window);
    if (context == NULL)
    {
        printf("Unable to create SDL GL context\n");
        CheckSDLError(__LINE__);
        return -1;
    }

    init();

    reshape(width, height);

    // Audio
    // local variables
    static Uint32 wav_length = 0;    // length of our sample
    static Uint8 *wav_buffer = NULL; // buffer containing our audio file
    static SDL_AudioSpec wav_spec;   // the specs of our piece of music

    // Load the WAV
    // the specs, length and buffer of our wav are filled
    const char* audio_file_name = "./test.wav";
    if (SDL_LoadWAV(audio_file_name, &wav_spec, &wav_buffer, &wav_length) == NULL)
    {
        fprintf(stderr, "Couldn't open audio file %s: %s\n", audio_file_name, SDL_GetError());
        return -1;
    }

    // set the callback function
    wav_spec.callback = audio_callback;
    wav_spec.userdata = NULL;
    // set our global static variables
    audio_pos = wav_buffer; // copy sound buffer
    audio_len = wav_length; // copy file length

    /* Open the audio device */
    if (SDL_OpenAudio(&wav_spec, NULL) < 0)
    {
        fprintf(stderr, "Couldn't open audio: %s\n", SDL_GetError());
        return -1;
    }

    int running = 1;
    SDL_Event event;
    int playing_audio = 0;
    while (running)
    {
        idle();

        // Loop track
        if (audio_len <= 0) {
            audio_pos = wav_buffer;
            audio_len = wav_length;
        }

        while (SDL_PollEvent(&event))
        {
            if (event.type == SDL_QUIT)
                running = 0;

            if (event.type == SDL_KEYDOWN)
            {
                switch (event.key.keysym.sym)
                {
                case SDLK_p:
                {
                    if (playing_audio)
                    {
                        fprintf(stderr, "Pausing SDL audio\n");
                    }
                    else
                    {
                        fprintf(stderr, "Playing SDL audio\n");
                    }
                    SDL_PauseAudio(playing_audio);
                    playing_audio = playing_audio > 0 ? 0 : 1;
                    break;
                }
                case SDLK_a:
                case SDLK_LEFT:
                {
                    delta -= 0.2f;
                    break;
                }
                case SDLK_d:
                case SDLK_RIGHT:
                {
                    delta += 0.2f;
                    break;
                }
                case SDLK_ESCAPE:
                {
                    running = 0;
                    break;
                }
                default:
                    break;
                }
            }

            if (event.type == SDL_MOUSEBUTTONDOWN)
            {
                if (event.button.button == SDL_BUTTON_LEFT)
                {
                    printf("Left mouse btn pressed at position %d,%d\n", event.button.x, event.button.y);
                }
                else if (event.button.button == SDL_BUTTON_MIDDLE)
                {
                    printf("Middle mouse btn pressed at position %d,%d\n", event.button.x, event.button.y);
                }
                else if (event.button.button == SDL_BUTTON_RIGHT)
                {
                    printf("Right mouse btn pressed at position %d,%d\n", event.button.x, event.button.y);
                }
            }
        }
    }

    SDL_GL_DeleteContext(context);
    SDL_DestroyWindow(window);

    SDL_CloseAudio();
    SDL_FreeWAV(wav_buffer);

    // Shutdown SDL 2
    SDL_Quit();

    return 0;
}