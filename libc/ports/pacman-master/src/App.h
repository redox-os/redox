/***************************************************************************
 *                                                                         *
 *   This program is free software; you can redistribute it and/or modify  *
 *   it under the terms of the GNU General Public License as published by  *
 *   the Free Software Foundation; either version 2 of the License, or     *
 *   (at your option) any later version.                                   *
 *                                                                         *
 ***************************************************************************/


#pragma once
#include "Main.h"
#include <boost/shared_ptr.hpp>

using boost::shared_ptr;

class Sounds;

class App
{
public:
    App();
    ~App();             //SDL systems are shut down in destructor

    void InitApp();     //SDL systems are initialized
    void InitWindow();  //creates window with parameters from loaded settings file
    void InitSound();
    void PrepareShutdown();

    //////////////
    //Global
    //////////////

    inline bool getQuit() const { return quit; }
    void setQuit( bool q ) { quit = q; }
    shared_ptr<SDL_Surface> getScreen() const { return screen; }
    shared_ptr<SDL_Surface> getBuf() const { return buf; }
    Sounds* getSnd() { return snd; }

private:

    //////////////
    //Global
    //////////////

    bool
            quit;       //if true then quit application

    shared_ptr<SDL_Surface>
            screen,    //screen surface
            buf;       //buffer surface

    Sounds
            *snd;

};
