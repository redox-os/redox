/*
    SDL - Simple DirectMedia Layer
    Copyright (C) 1997-2012 Sam Lantinga

    This library is free software; you can redistribute it and/or
    modify it under the terms of the GNU Lesser General Public
    License as published by the Free Software Foundation; either
    version 2.1 of the License, or (at your option) any later version.

    This library is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
    Lesser General Public License for more details.

    You should have received a copy of the GNU Lesser General Public
    License along with this library; if not, write to the Free Software
    Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA  02110-1301  USA

    Sam Lantinga
    slouken@libsdl.org
*/
#include "SDL_config.h"

#ifdef SDL_LOADSO_WIN32

/* * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * */
/* System dependent library loading routines                           */

#define WIN32_LEAN_AND_MEAN
#include <windows.h>

#include "SDL_loadso.h"

void *SDL_LoadObject(const char *sofile)
{
	void *handle = NULL;
	const char *loaderror = "Unknown error";

#if defined(_WIN32_WCE)
	char errbuf[512];

	wchar_t *errbuf_t = SDL_malloc(512 * sizeof(wchar_t));
	wchar_t *sofile_t = SDL_malloc((MAX_PATH+1) * sizeof(wchar_t));

	MultiByteToWideChar(CP_ACP, MB_PRECOMPOSED, sofile, -1, sofile_t, MAX_PATH);
	handle = (void *)LoadLibrary(sofile_t);

	/* Generate an error message if all loads failed */
	if ( handle == NULL ) {
		FormatMessage((FORMAT_MESSAGE_IGNORE_INSERTS |
					FORMAT_MESSAGE_FROM_SYSTEM),
				NULL, GetLastError(), 
				MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT),
				errbuf_t, SDL_arraysize(errbuf), NULL);
		WideCharToMultiByte(CP_ACP, 0, errbuf_t, -1, errbuf, 511, NULL, NULL);
		loaderror = errbuf;
	}

	SDL_free(sofile_t);
	SDL_free(errbuf_t);

#else /*if defined(__WIN32__)*/
	char errbuf[512];

	handle = (void *)LoadLibrary(sofile);

	/* Generate an error message if all loads failed */
	if ( handle == NULL ) {
		FormatMessage((FORMAT_MESSAGE_IGNORE_INSERTS |
					FORMAT_MESSAGE_FROM_SYSTEM),
				NULL, GetLastError(), 
				MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT),
				errbuf, SDL_arraysize(errbuf), NULL);
		loaderror = errbuf;
	}
#endif

	if ( handle == NULL ) {
		SDL_SetError("Failed loading %s: %s", sofile, loaderror);
	}
	return(handle);
}

void *SDL_LoadFunction(void *handle, const char *name)
{
	void *symbol = NULL;
	const char *loaderror = "Unknown error";

#if defined(_WIN32_WCE)
	char errbuf[512];
	int length = SDL_strlen(name);

	wchar_t *name_t = SDL_malloc((length + 1) * sizeof(wchar_t));
	wchar_t *errbuf_t = SDL_malloc(512 * sizeof(wchar_t));

	MultiByteToWideChar(CP_ACP, MB_PRECOMPOSED, name, -1, name_t, length+1);

	symbol = (void *)GetProcAddress((HMODULE)handle, name_t);
	if ( symbol == NULL ) {
		FormatMessage((FORMAT_MESSAGE_IGNORE_INSERTS |
					FORMAT_MESSAGE_FROM_SYSTEM),
				NULL, GetLastError(), 
				MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT),
				errbuf_t, SDL_arraysize(errbuf), NULL);
		WideCharToMultiByte(CP_ACP, 0, errbuf_t, -1, errbuf, 511, NULL, NULL);
		loaderror = errbuf;
	}

	SDL_free(name_t);
	SDL_free(errbuf_t);

#else /*if defined(WIN32)*/
	char errbuf[512];

	symbol = (void *)GetProcAddress((HMODULE)handle, name);
	if ( symbol == NULL ) {
		FormatMessage((FORMAT_MESSAGE_IGNORE_INSERTS |
					FORMAT_MESSAGE_FROM_SYSTEM),
				NULL, GetLastError(), 
				MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT),
				errbuf, SDL_arraysize(errbuf), NULL);
		loaderror = errbuf;
	}
#endif

	if ( symbol == NULL ) {
		SDL_SetError("Failed loading %s: %s", name, loaderror);
	}
	return(symbol);
}

void SDL_UnloadObject(void *handle)
{
	if ( handle != NULL ) {
		FreeLibrary((HMODULE)handle);
	}
}

#endif /* SDL_LOADSO_WIN32 */
