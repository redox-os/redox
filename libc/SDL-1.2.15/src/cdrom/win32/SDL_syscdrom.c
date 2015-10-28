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

#ifdef SDL_CDROM_WIN32

/* Functions for system-level CD-ROM audio control */

#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <mmsystem.h>

#include "SDL_cdrom.h"
#include "../SDL_syscdrom.h"

/* This really broken?? */
#define BROKEN_MCI_PAUSE	/* Pausing actually stops play -- Doh! */

/* The maximum number of CD-ROM drives we'll detect (Don't change!) */
#define MAX_DRIVES	26	

/* A list of available CD-ROM drives */
static char *SDL_cdlist[MAX_DRIVES];
static MCIDEVICEID SDL_mciID[MAX_DRIVES];
#ifdef BROKEN_MCI_PAUSE
static int SDL_paused[MAX_DRIVES];
#endif
static int SDL_CD_end_position;

/* The system-dependent CD control functions */
static const char *SDL_SYS_CDName(int drive);
static int SDL_SYS_CDOpen(int drive);
static int SDL_SYS_CDGetTOC(SDL_CD *cdrom);
static CDstatus SDL_SYS_CDStatus(SDL_CD *cdrom, int *position);
static int SDL_SYS_CDPlay(SDL_CD *cdrom, int start, int length);
static int SDL_SYS_CDPause(SDL_CD *cdrom);
static int SDL_SYS_CDResume(SDL_CD *cdrom);
static int SDL_SYS_CDStop(SDL_CD *cdrom);
static int SDL_SYS_CDEject(SDL_CD *cdrom);
static void SDL_SYS_CDClose(SDL_CD *cdrom);


/* Add a CD-ROM drive to our list of valid drives */
static void AddDrive(char *drive)
{
	int i;

	if ( SDL_numcds < MAX_DRIVES ) {
		/* Add this drive to our list */
		i = SDL_numcds;
		SDL_cdlist[i] = SDL_strdup(drive);
		if ( SDL_cdlist[i] == NULL ) {
			SDL_OutOfMemory();
			return;
		}
		++SDL_numcds;
#ifdef CDROM_DEBUG
  fprintf(stderr, "Added CD-ROM drive: %s\n", drive);
#endif
	}
}

int  SDL_SYS_CDInit(void)
{
	/* checklist: Drive 'A' - 'Z' */
	int i;
	char drive[4];

	/* Fill in our driver capabilities */
	SDL_CDcaps.Name = SDL_SYS_CDName;
	SDL_CDcaps.Open = SDL_SYS_CDOpen;
	SDL_CDcaps.GetTOC = SDL_SYS_CDGetTOC;
	SDL_CDcaps.Status = SDL_SYS_CDStatus;
	SDL_CDcaps.Play = SDL_SYS_CDPlay;
	SDL_CDcaps.Pause = SDL_SYS_CDPause;
	SDL_CDcaps.Resume = SDL_SYS_CDResume;
	SDL_CDcaps.Stop = SDL_SYS_CDStop;
	SDL_CDcaps.Eject = SDL_SYS_CDEject;
	SDL_CDcaps.Close = SDL_SYS_CDClose;

	/* Scan the system for CD-ROM drives */
	for ( i='A'; i<='Z'; ++i ) {
		SDL_snprintf(drive, SDL_arraysize(drive), "%c:\\", i);
		if ( GetDriveType(drive) == DRIVE_CDROM ) {
			AddDrive(drive);
		}
	}
	SDL_memset(SDL_mciID, 0, sizeof(SDL_mciID));
	return(0);
}

/* General ioctl() CD-ROM command function */
static int SDL_SYS_CDioctl(int id, UINT msg, DWORD flags, void *arg)
{
	MCIERROR mci_error;

	mci_error = mciSendCommand(SDL_mciID[id], msg, flags, (DWORD_PTR)arg);
	if ( mci_error ) {
		char error[256];

		mciGetErrorString(mci_error, error, 256);
		SDL_SetError("mciSendCommand() error: %s", error);
	}
	return(!mci_error ? 0 : -1);
}

static const char *SDL_SYS_CDName(int drive)
{
	return(SDL_cdlist[drive]);
}

static int SDL_SYS_CDOpen(int drive)
{
	MCI_OPEN_PARMS mci_open;
	MCI_SET_PARMS mci_set;
	char device[3];
	DWORD flags;

	/* Open the requested device */
	mci_open.lpstrDeviceType = (LPCSTR) MCI_DEVTYPE_CD_AUDIO;
	device[0] = *SDL_cdlist[drive];
	device[1] = ':';
	device[2] = '\0';
	mci_open.lpstrElementName = device;
	flags =
	  (MCI_OPEN_TYPE|MCI_OPEN_SHAREABLE|MCI_OPEN_TYPE_ID|MCI_OPEN_ELEMENT);
	if ( SDL_SYS_CDioctl(0, MCI_OPEN, flags, &mci_open) < 0 ) {
		flags &= ~MCI_OPEN_SHAREABLE;
		if ( SDL_SYS_CDioctl(0, MCI_OPEN, flags, &mci_open) < 0 ) {
			return(-1);
		}
	}
	SDL_mciID[drive] = mci_open.wDeviceID;

	/* Set the minute-second-frame time format */
	mci_set.dwTimeFormat = MCI_FORMAT_MSF;
	SDL_SYS_CDioctl(drive, MCI_SET, MCI_SET_TIME_FORMAT, &mci_set);

#ifdef BROKEN_MCI_PAUSE
	SDL_paused[drive] = 0;
#endif
	return(drive);
}

static int SDL_SYS_CDGetTOC(SDL_CD *cdrom)
{
	MCI_STATUS_PARMS mci_status;
	int i, okay;
	DWORD flags;

	okay = 0;
	mci_status.dwItem = MCI_STATUS_NUMBER_OF_TRACKS;
	flags = MCI_STATUS_ITEM | MCI_WAIT;
	if ( SDL_SYS_CDioctl(cdrom->id, MCI_STATUS, flags, &mci_status) == 0 ) {
		cdrom->numtracks = mci_status.dwReturn;
		if ( cdrom->numtracks > SDL_MAX_TRACKS ) {
			cdrom->numtracks = SDL_MAX_TRACKS;
		}
		/* Read all the track TOC entries */
		flags = MCI_STATUS_ITEM | MCI_TRACK | MCI_WAIT;
		for ( i=0; i<cdrom->numtracks; ++i ) {
			cdrom->track[i].id = i+1;
			mci_status.dwTrack = cdrom->track[i].id;
#ifdef MCI_CDA_STATUS_TYPE_TRACK
			mci_status.dwItem = MCI_CDA_STATUS_TYPE_TRACK;
			if ( SDL_SYS_CDioctl(cdrom->id, MCI_STATUS, flags,
							&mci_status) < 0 ) {
				break;
			}
			if ( mci_status.dwReturn == MCI_CDA_TRACK_AUDIO ) {
				cdrom->track[i].type = SDL_AUDIO_TRACK;
			} else {
				cdrom->track[i].type = SDL_DATA_TRACK;
			}
#else
			cdrom->track[i].type = SDL_AUDIO_TRACK;
#endif
			mci_status.dwItem = MCI_STATUS_POSITION;
			if ( SDL_SYS_CDioctl(cdrom->id, MCI_STATUS, flags,
							&mci_status) < 0 ) {
				break;
			}
			cdrom->track[i].offset = MSF_TO_FRAMES(
					MCI_MSF_MINUTE(mci_status.dwReturn),
					MCI_MSF_SECOND(mci_status.dwReturn),
					MCI_MSF_FRAME(mci_status.dwReturn));
			cdrom->track[i].length = 0;
			if ( i > 0 ) {
				cdrom->track[i-1].length =
						cdrom->track[i].offset-
						cdrom->track[i-1].offset;
			}
		}
		if ( i == cdrom->numtracks ) {
			mci_status.dwTrack = cdrom->track[i - 1].id;
			mci_status.dwItem = MCI_STATUS_LENGTH;
			if ( SDL_SYS_CDioctl(cdrom->id, MCI_STATUS, flags,
							&mci_status) == 0 ) {
				cdrom->track[i - 1].length = MSF_TO_FRAMES(
					MCI_MSF_MINUTE(mci_status.dwReturn),
					MCI_MSF_SECOND(mci_status.dwReturn),
					MCI_MSF_FRAME(mci_status.dwReturn));
				/* compute lead-out offset */
				cdrom->track[i].offset = cdrom->track[i - 1].offset +
					cdrom->track[i - 1].length;
				cdrom->track[i].length = 0;
				okay = 1;
			}
		}
	}
	return(okay ? 0 : -1);
}

/* Get CD-ROM status */
static CDstatus SDL_SYS_CDStatus(SDL_CD *cdrom, int *position)
{
	CDstatus status;
	MCI_STATUS_PARMS mci_status;
	DWORD flags;

	flags = MCI_STATUS_ITEM | MCI_WAIT;
	mci_status.dwItem = MCI_STATUS_MODE;
	if ( SDL_SYS_CDioctl(cdrom->id, MCI_STATUS, flags, &mci_status) < 0 ) {
		status = CD_ERROR;
	} else {
		switch (mci_status.dwReturn) {
			case MCI_MODE_NOT_READY:
			case MCI_MODE_OPEN:
				status = CD_TRAYEMPTY;
				break;
			case MCI_MODE_STOP:
#ifdef BROKEN_MCI_PAUSE
				if ( SDL_paused[cdrom->id] ) {
					status = CD_PAUSED;
				} else {
					status = CD_STOPPED;
				}
#else
				status = CD_STOPPED;
#endif /* BROKEN_MCI_PAUSE */
				break;
			case MCI_MODE_PLAY:
#ifdef BROKEN_MCI_PAUSE
				if ( SDL_paused[cdrom->id] ) {
					status = CD_PAUSED;
				} else {
					status = CD_PLAYING;
				}
#else
				status = CD_PLAYING;
#endif /* BROKEN_MCI_PAUSE */
				break;
			case MCI_MODE_PAUSE:
				status = CD_PAUSED;
				break;
			default:
				status = CD_ERROR;
				break;
		}
	}
	if ( position ) {
		if ( status == CD_PLAYING || (status == CD_PAUSED) ) {
			mci_status.dwItem = MCI_STATUS_POSITION;
			if ( SDL_SYS_CDioctl(cdrom->id, MCI_STATUS, flags,
							&mci_status) == 0 ) {
				*position = MSF_TO_FRAMES(
					MCI_MSF_MINUTE(mci_status.dwReturn),
					MCI_MSF_SECOND(mci_status.dwReturn),
					MCI_MSF_FRAME(mci_status.dwReturn));
			} else {
				*position = 0;
			}
		} else {
			*position = 0;
		}
	}
	return(status);
}

/* Start play */
static int SDL_SYS_CDPlay(SDL_CD *cdrom, int start, int length)
{
	MCI_PLAY_PARMS mci_play;
	int m, s, f;
	DWORD flags;

	flags = MCI_FROM | MCI_TO | MCI_NOTIFY;
	mci_play.dwCallback = 0;
	FRAMES_TO_MSF(start, &m, &s, &f);
	mci_play.dwFrom = MCI_MAKE_MSF(m, s, f);
	FRAMES_TO_MSF(start+length, &m, &s, &f);
	mci_play.dwTo = MCI_MAKE_MSF(m, s, f);
	SDL_CD_end_position = mci_play.dwTo;
	return(SDL_SYS_CDioctl(cdrom->id, MCI_PLAY, flags, &mci_play));
}

/* Pause play */
static int SDL_SYS_CDPause(SDL_CD *cdrom)
{
#ifdef BROKEN_MCI_PAUSE
	SDL_paused[cdrom->id] = 1;
#endif
	return(SDL_SYS_CDioctl(cdrom->id, MCI_PAUSE, MCI_WAIT, NULL));
}

/* Resume play */
static int SDL_SYS_CDResume(SDL_CD *cdrom)
{
#ifdef BROKEN_MCI_PAUSE
	MCI_STATUS_PARMS mci_status;
	int okay;
	int flags;

	okay = 0;
	/* Play from the current play position to the end position set earlier */
	flags = MCI_STATUS_ITEM | MCI_WAIT;
	mci_status.dwItem = MCI_STATUS_POSITION;
	if ( SDL_SYS_CDioctl(cdrom->id, MCI_STATUS, flags, &mci_status) == 0 ) {
		MCI_PLAY_PARMS mci_play;

		flags = MCI_FROM | MCI_TO | MCI_NOTIFY;
		mci_play.dwCallback = 0;
		mci_play.dwFrom = mci_status.dwReturn;
		mci_play.dwTo = SDL_CD_end_position;
		if (SDL_SYS_CDioctl(cdrom->id,MCI_PLAY,flags,&mci_play) == 0) {
			okay = 1;
			SDL_paused[cdrom->id] = 0;
		}
	}
	return(okay ? 0 : -1);
#else
	return(SDL_SYS_CDioctl(cdrom->id, MCI_RESUME, MCI_WAIT, NULL));
#endif /* BROKEN_MCI_PAUSE */
}

/* Stop play */
static int SDL_SYS_CDStop(SDL_CD *cdrom)
{
	return(SDL_SYS_CDioctl(cdrom->id, MCI_STOP, MCI_WAIT, NULL));
}

/* Eject the CD-ROM */
static int SDL_SYS_CDEject(SDL_CD *cdrom)
{
	return(SDL_SYS_CDioctl(cdrom->id, MCI_SET, MCI_SET_DOOR_OPEN, NULL));
}

/* Close the CD-ROM handle */
static void SDL_SYS_CDClose(SDL_CD *cdrom)
{
	SDL_SYS_CDioctl(cdrom->id, MCI_CLOSE, MCI_WAIT, NULL);
}

void SDL_SYS_CDQuit(void)
{
	int i;

	if ( SDL_numcds > 0 ) {
		for ( i=0; i<SDL_numcds; ++i ) {
			SDL_free(SDL_cdlist[i]);
			SDL_cdlist[i] = NULL;
		}
		SDL_numcds = 0;
	}
}

#endif /* SDL_CDROM_WIN32 */
