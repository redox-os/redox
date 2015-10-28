
/* Test program to compare the compile-time version of SDL with the linked
   version of SDL
*/

#include <stdio.h>
#include <stdlib.h>

#include "SDL.h"

int main(int argc, char *argv[])
{
	SDL_version compiled;

	/* Initialize SDL */
	if ( SDL_Init(0) < 0 ) {
		fprintf(stderr, "Couldn't initialize SDL: %s\n",SDL_GetError());
		exit(1);
	}
#ifdef DEBUG
	fprintf(stderr, "SDL initialized\n");
#endif
#if SDL_VERSION_ATLEAST(1, 2, 0)
	printf("Compiled with SDL 1.2 or newer\n");
#else
	printf("Compiled with SDL older than 1.2\n");
#endif
	SDL_VERSION(&compiled);
	printf("Compiled version: %d.%d.%d\n",
			compiled.major, compiled.minor, compiled.patch);
	printf("Linked version: %d.%d.%d\n",
			SDL_Linked_Version()->major,
			SDL_Linked_Version()->minor,
			SDL_Linked_Version()->patch);
	SDL_Quit();
	return(0);
}
