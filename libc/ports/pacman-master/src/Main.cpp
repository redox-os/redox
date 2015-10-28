/***************************************************************************
 *                                                                         *
 *   This program is free software; you can redistribute it and/or modify  *
 *   it under the terms of the GNU General Public License as published by  *
 *   the Free Software Foundation; either version 2 of the License, or     *
 *   (at your option) any later version.                                   *
 *                                                                         *
 ***************************************************************************/


#include "Main.h"

//////////////////////////////////////////////////////
//	GLOBAL VARS
//////////////////////////////////////////////////////

Log		logtxt;
App		app;
Game	game;
Settings settings;

//////////////////////////////////////////////////////
//	MAIN
//////////////////////////////////////////////////////

int main( int argc, char** argv ) {
    std::string str="",level="",skin="",editfile="";
    bool editor=false;

    for (int i = 1;i<argc;i++) {
        str=argv[i];
        if (str=="--help") {
            std::cout << "pacman usage:\n\ncommandline arguments\n--help:\t\tshow this message\n"
                    << "--level <lvl>:\tstart with selected level\n--skin <skin>:\tstart with selected skin\n"
                    << "--editor <lvl>:\tstart in editor mode. \n\t\tif a levelname is given the editor loads and saves to that level\n\n"
                    << "ingame\nesc/q:\tquit\narrows:\tmovement\nspace:\tboost\n"
                    << "p:\ttoggle pause\nn:\tnew game\nl:\tswitch level\ns:\tswitch skin\n"
                    << "e:\tenter editor\nw:\t(in editor) save map\nf:\ttoggle fps display\nh:\tview highscore\n";
            return 0;
        }
        else if (str=="--level") {
            if (++i<argc) {
                str=argv[i];
                if (str[0]=='-') {
                    std::cerr << "no level name given. exiting...\n";
                    return 1;
                }
                else
                    level=str;
            }
            else {
                std::cerr << "no level name given. exiting...\n";
                return 1;
            }
        }
        else  if (str=="--skin")
            if (++i<argc) {
            str=argv[i];
            if (str[0]=='-') {
                std::cerr << "no skin name given. exiting...\n";
                return 1;
            }
            else
                skin=str;
        }
        else {
            std::cerr << "no skin name given. exiting...\n";
            return 1;
        }
        else if (str=="--editor") {
            editor=true;
            if (i+1<argc) {
                str=argv[i+1];
                if (str[0]!='-') {
                    i++;
                    editfile=str;
                }
            }
        }
        else
            std::cerr << "unrecognized commandline option\n";
    }

    srand( (unsigned int)time(NULL) );

    //init log
    logtxt.setFilename(".pacman_sdl");

    //init settings
    if ( !app.getQuit() ) settings.LoadSettings(SETTINGSFILE);
    if ( !app.getQuit() ) settings.LoadSettings( (settings.lvlpath[settings.lvlpathcurrent] + CFGFILE) );

    //init SDL
    if ( !app.getQuit() ) app.InitApp();

    //init window
    if ( !app.getQuit() ) app.InitWindow();

    //init sound
    if ( !app.getQuit() ) app.InitSound();

    //set editorpath
    if ( editfile!="" ) {
        game.setEditorPath("./levels/" + editfile + "/");
    }

    //init game class
    if ( !app.getQuit() ) game.gameInit(level,skin,editor);


    //main loop
    while ( ! app.getQuit() ) {

        game.emptyMsgPump();

        if ( ! app.getQuit() )
            game.processLogic();

        if ( ! app.getQuit() )
            game.render();
    }

    //shutdown
    logtxt.print( "Shutdown" );

    return 0;
}
