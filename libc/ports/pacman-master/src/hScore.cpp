/***************************************************************************
 *                                                                         *
 *   This program is free software; you can redistribute it and/or modify  *
 *   it under the terms of the GNU General Public License as published by  *
 *   the Free Software Foundation; either version 2 of the License, or     *
 *   (at your option) any later version.                                   *
 *                                                                         *
 ***************************************************************************/


#include "hScore.h"

void hScore::clear() {
    std::ofstream file;

    remove(filename);
    load();
}

int hScore::load() {
    std::ifstream file;
    int i;

    file.open(filename);
    if (!file) {
        for (i=0; i< MAXENTRIES; i++) {
            name[i]="";
            score[i]=0;

        }
        return 1;
    }
    else {
        for (i=0; i< MAXENTRIES; i++) {
            if ( ! file.eof() ) {
                getline(file, name[i], '\0');
                file >> std::setw(8) >> score[i];
            }
            else {
                name[i]="";
                score[i]=0;
            }

        }
    }
    if (file.is_open()) file.close();
    return 0;
}

int hScore::save() {
    std::ofstream file;
    int i;

    file.open(filename);
    if (!file) return 1;

    for (i=0;i<MAXENTRIES;i++) {
        file << std::setw(3) << name[i] << '\0' << std::setw(8) << score[i];
    }

    if (file.is_open() ) file.close();
    return 0;
}

bool hScore::onlist(unsigned int sc) const {
    int i;

    for (i=0; i<MAXENTRIES && sc < score[i]; i++); // DEFINE WHETHER higher score is lower number or higher number

    if (i<MAXENTRIES) return true;

    else return false;
}

void hScore::add(std::string n, unsigned int sc) {
    int i, j;

    for (i=0; i<MAXENTRIES && sc < score[i]; i++);	// DEFINE WHETHER higher score is lower number or higher number

    if (i<MAXENTRIES) {
        for (j=MAXENTRIES - 1; j>i; j--) {
            name[j]=name[j-1];
            score[j]=score[j-1];
        }
        name[i]=n;
        score[i]=sc;
    }
}

void hScore::setfilename(std::string fn) {
    int i;
    for (i=0; i<20;i++) {
        filename[i]= (fn.c_str())[i] ;
    }
}

int hScore::getScore(int i) {
    return score[i];
}
std::string hScore::getName(int i) {
    return name[i];
}

hScore::hScore() {
    int i;
    for (i=0; i<MAXENTRIES;i++) {
        name[i]="";
        score[i]=0;
    }
}

hScore::~hScore(void)
{
}
