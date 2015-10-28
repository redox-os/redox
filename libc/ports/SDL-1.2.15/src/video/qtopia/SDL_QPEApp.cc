/*
    SDL - Simple DirectMedia Layer
    Copyright (C) 1997-2012 Sam Lantinga

    This library is free software; you can redistribute it and/or
    modify it under the terms of the GNU Library General Public
    License as published by the Free Software Foundation; either
    version 2 of the License, or (at your option) any later version.

    This library is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
    Library General Public License for more details.

    You should have received a copy of the GNU Library General Public
    License along with this library; if not, write to the Free
    Foundation, Inc., 59 Temple Place, Suite 330, Boston, MA  02111-1307  USA

    Sam Lantinga
    slouken@libsdl.org
*/
#include "SDL_config.h"

#include <qpe/qpeapplication.h>
#include <qapplication.h>
#include <qevent.h>

#include "SDL_thread.h"
#include "SDL_timer.h"
#include "SDL_error.h"

/* Flag to tell whether or not the Be application is active or not */
int SDL_QPEAppActive = 0;
static QPEApplication *app;

int SDL_InitQPEApp() {
  if(SDL_QPEAppActive <= 0) {
    if(!qApp) {
      int argc = 1;
      char *argv[] = { { "SDLApp" } };
      app = new QPEApplication(argc, argv);
      QWidget dummy;
      app->showMainWidget(&dummy);
    } else {
      app = (QPEApplication*)qApp;
    }
    SDL_QPEAppActive++;
  }
  return 0;  
}

/* Quit the QPE Application, if there's nothing left to do */
void SDL_QuitQPEApp(void)
{
  /* Decrement the application reference count */
  SDL_QPEAppActive--;
  /* If the reference count reached zero, clean up the app */
  if ( SDL_QPEAppActive == 0 && app) {
    delete app;
    app = 0;
    qApp = 0;
  }
}
