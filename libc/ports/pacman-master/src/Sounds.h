/***************************************************************************
 *                                                                         *
 *   This program is free software; you can redistribute it and/or modify  *
 *   it under the terms of the GNU General Public License as published by  *
 *   the Free Software Foundation; either version 2 of the License, or     *
 *   (at your option) any later version.                                   *
 *                                                                         *
 ***************************************************************************/


#pragma once

#include <boost/shared_ptr.hpp>

#include "Main.h"

using boost::shared_ptr;

#define NUMOFSOUNDS 13

class Sounds
{
public:
    Sounds();
    ~Sounds();
    bool init();
    void play(int i, bool looped=0, int volume=128);
//    void modify( int sound, long freq, long volume=0, long pan=0 );
    void stop(int i);
    void stop();
    void toggleSounds();
    bool on;
private:
    shared_ptr<Mix_Chunk>
            snd[NUMOFSOUNDS];

    std::string
            sndPaths[NUMOFSOUNDS];

    bool
            isinit;
};
