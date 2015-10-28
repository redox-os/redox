/***************************************************************************
 *                                                                         *
 *   This program is free software; you can redistribute it and/or modify  *
 *   it under the terms of the GNU General Public License as published by  *
 *   the Free Software Foundation; either version 2 of the License, or     *
 *   (at your option) any later version.                                   *
 *                                                                         *
 ***************************************************************************/


#include "Log.h"

extern App app;

bool Log::print(std::string txt) {

    try {
        std::ofstream file(filename.c_str(), std::ios::app);

        if ( !file)
            throw Error("Unable to open logfile");

        file << txt << std::endl;

        file.close();
    }

    catch ( Error& err ) {
        std::cerr << err.getDesc();
        app.setQuit(true);
        return false;
    }
    catch ( ... ) {
        std::cerr << "Unexpected exception";
        app.setQuit(true);
        return false;
    }

    return true;
}

bool Log::setFilename(std::string fn) {


    filename = std::string(getenv("HOME")) + "/" + fn;

    try {
        std::ofstream file(filename.c_str());

        if ( !file)
            throw Error("Unable to open logfile");

        file << WNDTITLE << std::endl;

        file.close();

    }
    catch ( Error& err ) {
        std::cerr << err.getDesc();
        app.setQuit(true);
        return false;
    }
    catch ( ... ) {
        std::cerr << "Unexpected exception";
        app.setQuit(true);
        return false;
    }

    return true;
}

Log::Log()
{
}

Log::~Log()
{
}
