/* gears.c */

/*
 * 3-D gear wheels.  This program is in the public domain.
 *
 * Brian Paul
 */

/* Conversion to GLUT by Mark J. Kilgard */

#include <SDL2/SDL.h>
#include <SDL2/SDL_opengl.h>
#include <SDL2/SDL_image.h>
#include <SDL2/SDL_mixer.h>
#include <SDL2/SDL_ttf.h>

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

SDL_Surface *image;
const char *IMAGE_FILE_NAME = "/games/sdl2_gears/assets/image.png";

Mix_Music *music = NULL;
const char *MUSIC_FILE_NAME = "/games/sdl2_gears/assets/music.wav";

TTF_Font *font = NULL;
const char *TTF_FILE_NAME = "/games/sdl2_gears/assets/font.ttf";

void cleanup()
{
    if (context != NULL)
    {
        SDL_GL_DeleteContext(context);
        context = NULL;
    }
    if (window != NULL)
    {
        SDL_DestroyWindow(window);
        window = NULL;
    }

    if (image != NULL)
    {
        SDL_FreeSurface(image);
        image = NULL;
        IMG_Quit();
    }

    if (music != NULL)
    {
        Mix_FreeMusic(music);
        music = NULL;
        Mix_CloseAudio();
    }

    if (font != NULL)
    {
        TTF_CloseFont(font);
        font = NULL;
    }

    // Shutdown SDL 2
    SDL_Quit();
}

int main(int argc, char *argv[])
{
    // Main
    printf("Initializing SDL\n");
    if (SDL_Init(SDL_INIT_VIDEO | SDL_INIT_AUDIO) < 0)
    {
        printf("Failed to init SDL\n");
        CheckSDLError(__LINE__);
        cleanup();
        return -1;
    }

    // Video / window
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
        cleanup();
        return -1;
    }

    printf("Creating SDL GL context\n");
    context = SDL_GL_CreateContext(window);
    if (context == NULL)
    {
        printf("Unable to create SDL GL context\n");
        CheckSDLError(__LINE__);
        cleanup();
        return -1;
    }

    init();

    reshape(width, height);

    // Image
    printf("Initializing SDL image supporting formats png and jpeg\n");
    int flags = IMG_INIT_JPG | IMG_INIT_PNG;
    int initted = IMG_Init(flags);
    if ((initted & flags) != flags)
    {
        printf("IMG_Init: Failed to init required jpg and png support: %s\n", IMG_GetError());
        CheckSDLError(__LINE__);
        cleanup();
        return -1;
    }

    image = IMG_Load(IMAGE_FILE_NAME);
    if (image == NULL)
    {
        printf("IMG_Load failed: %s\n", IMG_GetError());
        CheckSDLError(__LINE__);
        cleanup();
        return -1;
    }

    // Audio
    printf("Opening SDL mixer audio\n");
    if (Mix_OpenAudio(44100, MIX_DEFAULT_FORMAT, 2, 4096) < 0)
    {
        fprintf(stderr, "Couldn't open audio mixer: %s\n", SDL_GetError());
        CheckSDLError(__LINE__);
        cleanup();
        return -1;
    }

    music = Mix_LoadMUS(MUSIC_FILE_NAME);
    if (music == NULL)
    {
        fprintf(stderr, "Couldn't open audio file %s: %s\n", MUSIC_FILE_NAME, SDL_GetError());
        CheckSDLError(__LINE__);
        cleanup();
        return -1;
    }

    if (Mix_PlayMusic(music, -1) < 0)
    {
        fprintf(stderr, "Couldn't play music: %s\n", SDL_GetError());
        CheckSDLError(__LINE__);
        cleanup();
        return -1;
    }

    // TTF
    printf("Initializing TTF\n");
    if (TTF_Init() < 0)
    {
        printf("Failed to init TTF\n");
        CheckSDLError(__LINE__);
        cleanup();
        return -1;
    }

    font = TTF_OpenFont(TTF_FILE_NAME, 30);
    if (font == NULL)
    {
        printf("Couldn't open TTF file %s: %s\n", TTF_FILE_NAME, SDL_GetError());
        CheckSDLError(__LINE__);
        cleanup();
        return -1;
    }

    int running = 1;
    SDL_Event event;
    int playing_audio = 0;
    while (running)
    {
        idle();

        // Loop track
        Mix_PlayingMusic();

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
                    if (!Mix_PlayingMusic())
                    {
                        if (Mix_PlayMusic(music, -1) < 0)
                        {
                            fprintf(stderr, "Couldn't play music: %s\n", SDL_GetError());
                            CheckSDLError(__LINE__);
                            cleanup();
                            return -1;
                        }
                    }
                    else
                    {
                        if (Mix_PausedMusic())
                        {
                            Mix_ResumeMusic();
                        }
                        else
                        {
                            Mix_PauseMusic();
                        }
                    }
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

        SDL_Delay(10);
    }

    cleanup();

    return 0;
}
