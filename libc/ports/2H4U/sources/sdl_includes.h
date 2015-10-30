/*
    Copyright 2006 Pierre Lagouge, Pierre-Yves Ricau

    This file is part of 2H4U.

    2H4U is free software; you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation; either version 2 of the License, or
    (at your option) any later version.

    2H4U is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with 2H4U; if not, write to the Free Software
    Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA  02110-1301  USA
*/
//Fichier qui contient tous les includes de SDL.

//Uncomment the following line to disables sounds in 2H4U.
#define NO_SOUND_2H4U

//IF compiling with MAC
#ifdef MAC_OS

	#include <SDL_image/SDL_image.h>
	#include <SDL_ttf/SDL_ttf.h>
	#include <SDL_mixer/SDL_mixer.h>
	#include <SDL/SDL_getenv.h>

#else

	#include <SDL/SDL.h>
	#include <SDL/SDL_image.h>
	#include <SDL/SDL_ttf.h>
	#ifndef NO_SOUND_2H4U
		#include <SDL/SDL_mixer.h>
	#endif
	#include <SDL/SDL_getenv.h>

#endif

#include <cstdlib>

//Nombre de lignes dans les fichiers de langue
#define MAX_LANG 19
