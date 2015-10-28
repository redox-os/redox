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

/* This is the CD-audio control API for Simple DirectMedia Layer */

#include "SDL_cdrom.h"
#include "SDL_syscdrom.h"

#if !defined(__MACOS__)
#define CLIP_FRAMES	10	/* Some CD-ROMs won't go all the way */
#endif

static int SDL_cdinitted = 0;
static SDL_CD *default_cdrom;

/* The system level CD-ROM control functions */
struct CDcaps SDL_CDcaps = {
	NULL,					/* Name */
	NULL,					/* Open */
	NULL,					/* GetTOC */
	NULL,					/* Status */
	NULL,					/* Play */
	NULL,					/* Pause */
	NULL,					/* Resume */
	NULL,					/* Stop */
	NULL,					/* Eject */
	NULL,					/* Close */
};
int SDL_numcds;

int SDL_CDROMInit(void)
{
	int retval;

	SDL_numcds = 0;
	retval = SDL_SYS_CDInit();
	if ( retval == 0 ) {
		SDL_cdinitted = 1;
	}
	default_cdrom = NULL;
	return(retval);
}

/* Check to see if the CD-ROM subsystem has been initialized */
static int CheckInit(int check_cdrom, SDL_CD **cdrom)
{
	int okay;

	okay = SDL_cdinitted;
	if ( check_cdrom && (*cdrom == NULL) ) {
		*cdrom = default_cdrom;
		if ( *cdrom == NULL ) {
			SDL_SetError("CD-ROM not opened");
			okay = 0;
		}
	}
	if ( ! SDL_cdinitted ) {
		SDL_SetError("CD-ROM subsystem not initialized");
	}
	return(okay);
}

int SDL_CDNumDrives(void)
{
	if ( ! CheckInit(0, NULL) ) {
		return(-1);
	}
	return(SDL_numcds);
}

const char *SDL_CDName(int drive)
{
	if ( ! CheckInit(0, NULL) ) {
		return(NULL);
	}
	if ( drive >= SDL_numcds ) {
		SDL_SetError("Invalid CD-ROM drive index");
		return(NULL);
	}
	if ( SDL_CDcaps.Name ) {
		return(SDL_CDcaps.Name(drive));
	} else {
		return("");
	}
}

SDL_CD *SDL_CDOpen(int drive)
{
	struct SDL_CD *cdrom;

	if ( ! CheckInit(0, NULL) ) {
		return(NULL);
	}
	if ( drive >= SDL_numcds ) {
		SDL_SetError("Invalid CD-ROM drive index");
		return(NULL);
	}
	cdrom = (SDL_CD *)SDL_malloc(sizeof(*cdrom));
	if ( cdrom == NULL ) {
		SDL_OutOfMemory();
		return(NULL);
	}
	SDL_memset(cdrom, 0, sizeof(*cdrom));
	cdrom->id = SDL_CDcaps.Open(drive);
	if ( cdrom->id < 0 ) {
		SDL_free(cdrom);
		return(NULL);
	}
	default_cdrom = cdrom;
	return(cdrom);
}

CDstatus SDL_CDStatus(SDL_CD *cdrom)
{
	CDstatus status;
	int i;
	Uint32 position;

	/* Check if the CD-ROM subsystem has been initialized */
	if ( ! CheckInit(1, &cdrom) ) {
		return(CD_ERROR);
	}

	/* Get the current status of the drive */
	cdrom->numtracks = 0;
	cdrom->cur_track = 0;
	cdrom->cur_frame = 0;
	status = SDL_CDcaps.Status(cdrom, &i);
	position = (Uint32)i;
	cdrom->status = status;

	/* Get the table of contents, if there's a CD available */
	if ( CD_INDRIVE(status) ) {
		if ( SDL_CDcaps.GetTOC(cdrom) < 0 ) {
			status = CD_ERROR;
		}
		/* If the drive is playing, get current play position */
		if ( (status == CD_PLAYING) || (status == CD_PAUSED) ) {
			for ( i=1; cdrom->track[i].offset <= position; ++i ) {
				/* Keep looking */;
			}
#ifdef DEBUG_CDROM
  fprintf(stderr, "Current position: %d, track = %d (offset is %d)\n",
				position, i-1, cdrom->track[i-1].offset);
#endif
			cdrom->cur_track = i-1;
			position -= cdrom->track[cdrom->cur_track].offset;
			cdrom->cur_frame = position;
		}
	}
	return(status);
}

int SDL_CDPlayTracks(SDL_CD *cdrom,
			int strack, int sframe, int ntracks, int nframes)
{
	int etrack, eframe;
	int start, length;

	/* Check if the CD-ROM subsystem has been initialized */
	if ( ! CheckInit(1, &cdrom) ) {
		return(CD_ERROR);
	}

	/* Determine the starting and ending tracks */
	if ( (strack < 0) || (strack >= cdrom->numtracks) ) {
		SDL_SetError("Invalid starting track");
		return(CD_ERROR);
	}
	if ( ! ntracks && ! nframes ) {
		etrack = cdrom->numtracks;
		eframe = 0;
	} else {
		etrack = strack+ntracks;
		if ( etrack == strack ) {
			eframe = sframe + nframes;
		} else {
			eframe = nframes;
		}
	}
	if ( etrack > cdrom->numtracks ) {
		SDL_SetError("Invalid play length");
		return(CD_ERROR);
	}

	/* Skip data tracks and verify frame offsets */
	while ( (strack <= etrack) &&
			(cdrom->track[strack].type == SDL_DATA_TRACK) ) {
		++strack;
	}
	if ( sframe >= (int)cdrom->track[strack].length ) {
		SDL_SetError("Invalid starting frame for track %d", strack);
		return(CD_ERROR);
	}
	while ( (etrack > strack) &&
			(cdrom->track[etrack-1].type == SDL_DATA_TRACK) ) {
		--etrack;
	}
	if ( eframe > (int)cdrom->track[etrack].length ) {
		SDL_SetError("Invalid ending frame for track %d", etrack);
		return(CD_ERROR);
	}

	/* Determine start frame and play length */
	start = (cdrom->track[strack].offset+sframe);
	length = (cdrom->track[etrack].offset+eframe)-start;
#ifdef CLIP_FRAMES
	/* I've never seen this necessary, but xmcd does it.. */
	length -= CLIP_FRAMES;	/* CLIP_FRAMES == 10 */
#endif
	if ( length < 0 ) {
		return(0);
	}

	/* Play! */
#ifdef DEBUG_CDROM
  fprintf(stderr, "Playing %d frames at offset %d\n", length, start);
#endif
	return(SDL_CDcaps.Play(cdrom, start, length));
}

int SDL_CDPlay(SDL_CD *cdrom, int sframe, int length)
{
	/* Check if the CD-ROM subsystem has been initialized */
	if ( ! CheckInit(1, &cdrom) ) {
		return(CD_ERROR);
	}

	return(SDL_CDcaps.Play(cdrom, sframe, length));
}

int SDL_CDPause(SDL_CD *cdrom)
{
	CDstatus status;
	int retval;

	/* Check if the CD-ROM subsystem has been initialized */
	if ( ! CheckInit(1, &cdrom) ) {
		return(CD_ERROR);
	}

	status = SDL_CDcaps.Status(cdrom, NULL);
	switch (status) {
		case CD_PLAYING:
			retval = SDL_CDcaps.Pause(cdrom);
			break;
		default:
			retval = 0;
			break;
	}
	return(retval);
}

int SDL_CDResume(SDL_CD *cdrom)
{
	CDstatus status;
	int retval;

	/* Check if the CD-ROM subsystem has been initialized */
	if ( ! CheckInit(1, &cdrom) ) {
		return(CD_ERROR);
	}

	status = SDL_CDcaps.Status(cdrom, NULL);
	switch (status) {
		case CD_PAUSED:
			retval = SDL_CDcaps.Resume(cdrom);
		default:
			retval = 0;
			break;
	}
	return(retval);
}

int SDL_CDStop(SDL_CD *cdrom)
{
	CDstatus status;
	int retval;

	/* Check if the CD-ROM subsystem has been initialized */
	if ( ! CheckInit(1, &cdrom) ) {
		return(CD_ERROR);
	}

	status = SDL_CDcaps.Status(cdrom, NULL);
	switch (status) {
		case CD_PLAYING:
		case CD_PAUSED:
			retval = SDL_CDcaps.Stop(cdrom);
		default:
			retval = 0;
			break;
	}
	return(retval);
}

int SDL_CDEject(SDL_CD *cdrom)
{
	/* Check if the CD-ROM subsystem has been initialized */
	if ( ! CheckInit(1, &cdrom) ) {
		return(CD_ERROR);
	}
	return(SDL_CDcaps.Eject(cdrom));
}

void SDL_CDClose(SDL_CD *cdrom)
{
	/* Check if the CD-ROM subsystem has been initialized */
	if ( ! CheckInit(1, &cdrom) ) {
		return;
	}
	SDL_CDcaps.Close(cdrom);
	SDL_free(cdrom);
	default_cdrom = NULL;
}

void SDL_CDROMQuit(void)
{
	SDL_SYS_CDQuit();
	SDL_cdinitted = 0;
}
