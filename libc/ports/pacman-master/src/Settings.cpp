/***************************************************************************
 *                                                                         *
 *   This program is free software; you can redistribute it and/or modify  *
 *   it under the terms of the GNU General Public License as published by  *
 *   the Free Software Foundation; either version 2 of the License, or     *
 *   (at your option) any later version.                                   *
 *                                                                         *
 ***************************************************************************/


#include "Settings.h"

extern Log logtxt;

int Settings::setPath(int mode,std::string str) {
    int i;

    switch (mode) {
    case MODE_LEVELS:
        for (i=0;i<lvlpathcount;i++) {
            if (lvlpath[i]=="./levels/" + str + "/") {
                lvlpathcurrent=i;
                return 0;
            }
        }
        break;
    case MODE_SKINS:
        for (i=0;i<skinspathcount;i++) {
            if (skinspath[i]=="./skins/" + str + "/") {
                skinspathcurrent=i;
                return 0;
            }
        }
        break;
    default:
        break;
    }

    return 1;
}

string Settings::getFile(string filename) {
    struct stat fileInfo;
    for(int i = 0; i < searchpaths.size(); i++) {
        string path = searchpaths[i] + "/" + filename;
        if (stat(path.c_str(), &fileInfo) == 0) {
            return path;
        }
    }
    throw new Error("File not found: " + filename);
}

bool Settings::LoadSettings(std::string filename) {

    filename = getFile(filename);

    std::ifstream	file( filename.c_str() );
    std::string		buffer,
    tmpstr;
    char			c=';' ;
    int				pos;

    if (!file) {
        logtxt.print( "Could not open settings file" );
        return false;
    }

    while (file)	{
        do {
            if ( file.eof() ) break;
            c=file.get();
        } while (c < 'A' || c > 'z');

        pos= file.tellg();
        pos--;

        if (pos<0) pos=0;

        file.seekg(pos);

        getline(file, buffer, '=');
        if (! file.eof() ) {
            if (buffer == "WIDTH") file >> width;
            else if (buffer == "HEIGHT") file >> height;
            else if (buffer == "FIELDWIDTH") file >> fieldwidth;
            else if (buffer == "FIELDHEIGHT") file >> fieldheight;
            else if (buffer == "TILESIZE") file >> tilesize;
            else if (buffer == "PACSTARTX") file >> pacstartx;
            else if (buffer == "PACSTARTY") file >> pacstarty;
            else if (buffer == "PACSPEED") file >> pacspeed;
            else if (buffer == "BADDIESTARTX") file >> baddiestartx;
            else if (buffer == "BADDIESTARTY") file >> baddiestarty;
            else if (buffer == "BADDIESPEED") file >> baddiespeed;
            else if (buffer == "BADDIEIQ") file >> baddieiq;
            else if (buffer == "VULN_DURATION") file >> vuln_duration;
            else if (buffer == "GATEX") file >> gatex;
            else if (buffer == "GATEY") file >> gatey;
            else if (buffer == "LEVEL_PATH") {
                getline(file, tmpstr, ';');
                lvlpath.push_back("./levels/" + tmpstr + "/");
                lvlpathcount++;
            }
            else if (buffer == "SKINS_PATH") {
                getline(file, tmpstr, ';');
                skinspath.push_back("./skins/" + tmpstr + "/");
                skinspathcount++;
            }
        }
    }

    file.close();

    logtxt.print(filename + " loaded");

    return true;
}

Settings::Settings() {
    width = 640;
    height = 480;

    lvlpathcount = 0;
    lvlpathcurrent = 0;
    skinspathcount = 0;
    skinspathcurrent = 0;

    gatex = 0;
    gatey = 0;
    fieldwidth = 0;
    fieldheight = 0;
    tilesize = 0;
    pacstartx = 0;
    pacstarty = 0;
    pacspeed = 0;
    baddiestartx = 0;
    baddiestarty = 0;
    baddiespeed = 0;
    baddieiq = 0;
    vuln_duration = 0;

    searchpaths.push_back(".");
    searchpaths.push_back(string(getenv("HOME")) + "/" HOME_CONF_PATH);
    searchpaths.push_back(APP_PATH);
}

Settings::~Settings() {}
