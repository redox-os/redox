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
#include "Object.h"

using boost::shared_ptr;

#define NUMPACANIM 8

class Pacman :
	public Object
{
public:
    Pacman(shared_ptr <SDL_Surface> buf, int os, int ix, int iy, int ispdmod, int itilesize, int iheight, int iwidth, int *imap);

    void Draw();
    void Draw(int ix, int iy, int obj=3, int type=1);
    bool LoadTextures(std::string path);

    void setNextDir(int next);
    bool collision(int xtmp, int ytmp);

    void setSpeedMult( int s);

    void Update( int time );
    void nextIntersection(int &ix, int &iy);
    void reset(int ix, int iy);

    int getXpix();
    int getYpix();

private:
    int
            x,	//current position in tiles (all positions refer to upper left corner)
            y,

            dx,	//current direction in x and y
            dy,

            nextdx,	//queued direction
            nextdy,

            xpix,	//current position in pixels
            ypix,

            spdmod,	// speed modifier	- default 100?
            spdmult,

            tilesize,
            height,
            width,

            *map;

    unsigned int
            animcounter;

    float
            xfloat,	//current position as floating point based on pixel pos - allows for infinite speed
            yfloat;	// variations because xfloat can be reduced to xpix

    shared_ptr<SDL_Surface>
            pacEl[NUMPACANIM],
            pacElRot[NUMPACANIM][3];
};
