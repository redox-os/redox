/***************************************************************************
 *                                                                         *
 *   This program is free software; you can redistribute it and/or modify  *
 *   it under the terms of the GNU General Public License as published by  *
 *   the Free Software Foundation; either version 2 of the License, or     *
 *   (at your option) any later version.                                   *
 *                                                                         *
 ***************************************************************************/


//
//Pacman clone written by Jakob Gruber in C++ using Allegro. June 2007
//

#include "BHeap.h"

void BHeap::reset() {
    while ( nrofitems != 0) remove();
    squaresChecked=0;
    nrofitems=0;
}

int BHeap::find(int x, int y) {
    int i;
    for (i=1;i<=nrofitems;i++) {
        if (openList[i] ) {
            if ( openX[ openList[i] ] == x && openY[ openList[i] ] == y ) return i;
        }
    }
    return 0;
}

bool BHeap::changeF(int pos, unsigned int newF) {	//position in openlist and new value of F
    int i, tmp;
    if (newF < Fcost[ openList[pos] ] && pos > 0) {
        Fcost[ openList[pos] ] = newF;
        i=pos;
        if (i > 1) {
            while ( Fcost[ openList[i] ] < Fcost[openList[i/2]] && i > 1 )  {
                tmp=openList[i];
                openList[i]=openList[i/2];
                openList[i/2]=tmp;
                i=i/2;
            }
        }
        return 0;
    }
    return 1;
}

void BHeap::remove() {
    int i, j, tmp;

    openList[1]=openList[nrofitems];
    openList[nrofitems]=0;
    --nrofitems;

    if (nrofitems) {	//not empty?
        i=1;

        while (1) {
            j=i;

            if (2*j+1 <= nrofitems && openList[2*j] && openList[2*j+1]) {	//both children

                if ( Fcost[openList[j]] >= Fcost[openList[j*2]] ) i=2*j;
                if ( Fcost[openList[i]] >= Fcost[openList[j*2+1]] ) i=2*j+1;
            }
            else if (2*j+1 <= nrofitems && openList[2*j+1]) {			//only right child
                if ( Fcost[openList[i]] >= Fcost[openList[i*2+1]] ) i=2*j+1;
            }
            else if (2*j <=nrofitems && openList[2*j]) {	//only left child
                if ( Fcost[openList[i]] >= Fcost[openList[i*2]] ) i=2*j;
            }
            if (j!=i) {
                tmp=openList[i];		//problem here
                openList[i]=openList[j];
                openList[j]=tmp;
            }
            else break;
        }
    }
}

void BHeap::add(int x, int y, int f) {
    int i;

    ++squaresChecked;
    ++nrofitems;

    Fcost[squaresChecked]=f;
    openX[squaresChecked]=x;
    openY[squaresChecked]=y;
    openList[nrofitems]=squaresChecked;

    i=nrofitems;
    if (i>1) {
        while ( Fcost[squaresChecked] < Fcost[openList[i/2]] && i > 1 )  {
            openList[i]=openList[i/2];
            openList[i/2]=squaresChecked;
            i=i/2;
        }
    }
}

bool BHeap::isEmpty() {
    if ( nrofitems == 0) return true;
    else return false;
}

BHeap::BHeap(int width, int height): squaresChecked(0), nrofitems(0)
{
    int i;

    openList= new unsigned int[width*height];
    Fcost= new unsigned int[width*height];
    openX= new int[width*height];
    openY= new int[width*height];

    for (i=0;i<width*height;i++) {
        openList[i]=0;
        Fcost[i]=0;
        openX[i]=0;
        openY[i]=0;
    }
}

BHeap::~BHeap(void)
{
    delete[] openList;
    delete[] Fcost;
    delete[] openX;
    delete[] openY;
}
