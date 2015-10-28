/***************************************************************************
 *                                                                         *
 *   This program is free software; you can redistribute it and/or modify  *
 *   it under the terms of the GNU General Public License as published by  *
 *   the Free Software Foundation; either version 2 of the License, or     *
 *   (at your option) any later version.                                   *
 *                                                                         *
 ***************************************************************************/


#pragma once
#include <vector>
#include <string>
#include <cstdlib>
#include <sys/stat.h>
#include "Main.h"

using std::string;

class Settings {
public:
    Settings();
    ~Settings();

    bool LoadSettings(string filename);

    //searches for str in level/skinspaths; if successful, sets currently selected path.
    //returns 0 on success, 1 on failure
    int setPath(int mode, string str);

    //////////////////////////////
    // VARIABLES	- APP
    //////////////////////////////
    int
            width,
            height;

    //////////////////////////////
    // VARIABLES	- GAME
    //////////////////////////////

    int
            fieldwidth,
            fieldheight,
            tilesize,
            gatex,
            gatey,
            pacstartx,
            pacstarty,
            pacspeed,
            baddiestartx,
            baddiestarty,
            baddiespeed,
            baddieiq,
            vuln_duration,

            lvlpathcount,
            lvlpathcurrent,
            skinspathcount,
            skinspathcurrent;

    std::vector<string>
            lvlpath,
            skinspath,
            searchpaths;

    /* look for file in search paths and return first instance */
    string getFile(string filename);
};
