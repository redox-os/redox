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

#ifdef SDL_CDROM_DC

/* Functions for system-level CD-ROM audio control */

#include <dc/cdrom.h>
#include <dc/spu.h>

#include "SDL_cdrom.h"
#include "../SDL_syscdrom.h"

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


int  SDL_SYS_CDInit(void)
{
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

	return(0);
}

static const char *SDL_SYS_CDName(int drive)
{
	return "/cd";
}

static int SDL_SYS_CDOpen(int drive)
{
	return(drive);
}

#define	TRACK_CDDA	0
static int SDL_SYS_CDGetTOC(SDL_CD *cdrom)
{
	CDROM_TOC toc;
	int ret,i;

	ret = cdrom_read_toc(&toc,0);
	if (ret!=ERR_OK) {
		return -1;
	}

	cdrom->numtracks = TOC_TRACK(toc.last)-TOC_TRACK(toc.first)+1;
	for(i=0;i<cdrom->numtracks;i++) {
		unsigned long entry = toc.entry[i];
		cdrom->track[i].id = i+1;
		cdrom->track[i].type = (TOC_CTRL(toc.entry[i])==TRACK_CDDA)?SDL_AUDIO_TRACK:SDL_DATA_TRACK;
		cdrom->track[i].offset = TOC_LBA(entry)-150;
		cdrom->track[i].length = TOC_LBA((i+1<toc.last)?toc.entry[i+1]:toc.leadout_sector)-TOC_LBA(entry);
	}

	return 0;
}

/* Get CD-ROM status */
static CDstatus SDL_SYS_CDStatus(SDL_CD *cdrom, int *position)
{
	int ret,dc_status,disc_type;

	ret = cdrom_get_status(&dc_status,&disc_type);
	if (ret!=ERR_OK) return CD_ERROR;

	switch(dc_status) {
//	case CD_STATUS_BUSY:
	case CD_STATUS_PAUSED:
		return CD_PAUSED;
	case CD_STATUS_STANDBY:
		return CD_STOPPED;
	case CD_STATUS_PLAYING:
		return CD_PLAYING;
//	case CD_STATUS_SEEKING:
//	case CD_STATUS_SCANING:
	case CD_STATUS_OPEN:
	case CD_STATUS_NO_DISC:
		return CD_TRAYEMPTY;
	default:
		return	CD_ERROR;
	}
}

/* Start play */
static int SDL_SYS_CDPlay(SDL_CD *cdrom, int start, int length)
{
	int ret = cdrom_cdda_play(start-150,start-150+length,1,CDDA_SECTORS);
	return ret==ERR_OK?0:-1;
}

/* Pause play */
static int SDL_SYS_CDPause(SDL_CD *cdrom)
{
	int ret=cdrom_cdda_pause();
	return ret==ERR_OK?0:-1;
}

/* Resume play */
static int SDL_SYS_CDResume(SDL_CD *cdrom)
{
	int ret=cdrom_cdda_resume();
	return ret==ERR_OK?0:-1;
}

/* Stop play */
static int SDL_SYS_CDStop(SDL_CD *cdrom)
{
	int ret=cdrom_spin_down();
	return ret==ERR_OK?0:-1;
}

/* Eject the CD-ROM */
static int SDL_SYS_CDEject(SDL_CD *cdrom)
{
	return -1;
}

/* Close the CD-ROM handle */
static void SDL_SYS_CDClose(SDL_CD *cdrom)
{
}

void SDL_SYS_CDQuit(void)
{

}

#endif /* SDL_CDROM_DC */
