
/* Test the SDL CD-ROM audio functions */

#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <ctype.h>

#include "SDL.h"

/* Call this instead of exit(), so we can clean up SDL: atexit() is evil. */
static void quit(int rc)
{
	SDL_Quit();
	exit(rc);
}

static void PrintStatus(int driveindex, SDL_CD *cdrom)
{
	CDstatus status;
	char *status_str;

	status = SDL_CDStatus(cdrom);
	switch (status) {
		case CD_TRAYEMPTY:
			status_str = "tray empty";
			break;
		case CD_STOPPED:
			status_str = "stopped";
			break;
		case CD_PLAYING:
			status_str = "playing";
			break;
		case CD_PAUSED:
			status_str = "paused";
			break;
		case CD_ERROR:
			status_str = "error state";
			break;
	}
	printf("Drive %d status: %s\n", driveindex, status_str);
	if ( status >= CD_PLAYING ) {
		int m, s, f;
		FRAMES_TO_MSF(cdrom->cur_frame, &m, &s, &f);
		printf("Currently playing track %d, %d:%2.2d\n",
			cdrom->track[cdrom->cur_track].id, m, s);
	}
}

static void ListTracks(SDL_CD *cdrom)
{
	int i;
	int m, s, f;
	char* trtype;

	SDL_CDStatus(cdrom);
	printf("Drive tracks: %d\n", cdrom->numtracks);
	for ( i=0; i<cdrom->numtracks; ++i ) {
		FRAMES_TO_MSF(cdrom->track[i].length, &m, &s, &f);
		if ( f > 0 )
			++s;
		switch(cdrom->track[i].type)
		{
		    case SDL_AUDIO_TRACK:
			trtype="audio";
			break;
		    case SDL_DATA_TRACK:
			trtype="data";
			break;
		    default:
			trtype="unknown";
			break;
		}
		printf("\tTrack (index %d) %d: %d:%2.2d / %d [%s track]\n", i,
					cdrom->track[i].id, m, s, cdrom->track[i].length, trtype);
	}
}

static void PrintUsage(char *argv0)
{
	fprintf(stderr, "Usage: %s [drive#] [command] [command] ...\n", argv0);
	fprintf(stderr, "Where 'command' is one of:\n");
	fprintf(stderr, "	-status\n");
	fprintf(stderr, "	-list\n");
	fprintf(stderr, "	-play [first_track] [first_frame] [num_tracks] [num_frames]\n");
	fprintf(stderr, "	-pause\n");
	fprintf(stderr, "	-resume\n");
	fprintf(stderr, "	-stop\n");
	fprintf(stderr, "	-eject\n");
	fprintf(stderr, "	-sleep <milliseconds>\n");
}

int main(int argc, char *argv[])
{
	int drive;
	int i;
	SDL_CD *cdrom;

	/* Initialize SDL first */
	if ( SDL_Init(SDL_INIT_CDROM) < 0 ) {
		fprintf(stderr, "Couldn't initialize SDL: %s\n",SDL_GetError());
		return(1);
	}

	/* Find out how many CD-ROM drives are connected to the system */
	if ( SDL_CDNumDrives() == 0 ) {
		printf("No CD-ROM devices detected\n");
		quit(0);
	}
	printf("Drives available: %d\n", SDL_CDNumDrives());
	for ( i=0; i<SDL_CDNumDrives(); ++i ) {
		printf("Drive %d:  \"%s\"\n", i, SDL_CDName(i));
	}

	/* Open the CD-ROM */
	drive = 0;
	i=1;
	if ( argv[i] && isdigit(argv[i][0]) ) {
		drive = atoi(argv[i++]);
	}
	cdrom = SDL_CDOpen(drive);
	if ( cdrom == NULL ) {
		fprintf(stderr, "Couldn't open drive %d: %s\n", drive,
							SDL_GetError());
		quit(2);
	}
#ifdef TEST_NULLCD
	cdrom = NULL;
#endif
	
	/* Find out which function to perform */
	for ( ; argv[i]; ++i ) {
		if ( strcmp(argv[i], "-status") == 0 ) {
			/* PrintStatus(drive, cdrom); */
		} else
		if ( strcmp(argv[i], "-list") == 0 ) {
			ListTracks(cdrom);
		} else
		if ( strcmp(argv[i], "-play") == 0 ) {
			int strack, sframe;
			int ntrack, nframe;

			strack = 0;
			if ( argv[i+1] && isdigit(argv[i+1][0]) ) {
				strack = atoi(argv[++i]);
			}
			sframe = 0;
			if ( argv[i+1] && isdigit(argv[i+1][0]) ) {
				sframe = atoi(argv[++i]);
			}
			ntrack = 0;
			if ( argv[i+1] && isdigit(argv[i+1][0]) ) {
				ntrack = atoi(argv[++i]);
			}
			nframe = 0;
			if ( argv[i+1] && isdigit(argv[i+1][0]) ) {
				nframe = atoi(argv[++i]);
			}
			if ( CD_INDRIVE(SDL_CDStatus(cdrom)) ) {
				if ( SDL_CDPlayTracks(cdrom, strack, sframe,
							ntrack, nframe) < 0 ) {
					fprintf(stderr,
			"Couldn't play tracks %d/%d for %d/%d: %s\n",
				strack, sframe, ntrack, nframe, SDL_GetError());
				}
			} else {
				fprintf(stderr, "No CD in drive!\n");
			}
		} else
		if ( strcmp(argv[i], "-pause") == 0 ) {
			if ( SDL_CDPause(cdrom) < 0 ) {
				fprintf(stderr, "Couldn't pause CD: %s\n",
								SDL_GetError());
			}
		} else
		if ( strcmp(argv[i], "-resume") == 0 ) {
			if ( SDL_CDResume(cdrom) < 0 ) {
				fprintf(stderr, "Couldn't resume CD: %s\n",
								SDL_GetError());
			}
		} else
		if ( strcmp(argv[i], "-stop") == 0 ) {
			if ( SDL_CDStop(cdrom) < 0 ) {
				fprintf(stderr, "Couldn't eject CD: %s\n",
								SDL_GetError());
			}
		} else
		if ( strcmp(argv[i], "-eject") == 0 ) {
			if ( SDL_CDEject(cdrom) < 0 ) {
				fprintf(stderr, "Couldn't eject CD: %s\n",
								SDL_GetError());
			}
		} else
		if ( (strcmp(argv[i], "-sleep") == 0) &&
				(argv[i+1] && isdigit(argv[i+1][0])) ) {
			SDL_Delay(atoi(argv[++i]));
			printf("Delayed %d milliseconds\n", atoi(argv[i]));
		} else {
			PrintUsage(argv[0]);
			SDL_CDClose(cdrom);
			quit(1);
		}
	}
	PrintStatus(drive, cdrom);
	SDL_CDClose(cdrom);
	SDL_Quit();

	return(0);
}
