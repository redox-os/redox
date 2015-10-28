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

#ifdef SDL_CDROM_MACOS

/* MacOS functions for system-level CD-ROM audio control */

#include <Devices.h>
#include <Files.h>
#include <LowMem.h> /* Use entry table macros, not functions in InterfaceLib  */

#include "SDL_cdrom.h"
#include "../SDL_syscdrom.h"
#include "SDL_syscdrom_c.h"

/* Added by Matt Slot */
#if !defined(LMGetUnitTableEntryCount)
  #define LMGetUnitTableEntryCount()   *(short *)0x01D2
#endif

/* The maximum number of CD-ROM drives we'll detect */
#define MAX_DRIVES	26	

/* A list of available CD-ROM drives */
static long SDL_cdversion = 0;
static struct {
	short		dRefNum;
	short		driveNum;
	long		frames;
	char		name[256];
	Boolean		hasAudio;
	} SDL_cdlist[MAX_DRIVES];
static StringPtr gDriverName = "\p.AppleCD";

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

static short SDL_SYS_ShortToBCD(short value)
{
	return((value % 10) + (value / 10) * 0x10); /* Convert value to BCD */
}

static short SDL_SYS_BCDToShort(short value)
{
	return((value % 0x10) + (value / 0x10) * 10); /* Convert value from BCD */
}

int  SDL_SYS_CDInit(void)
{
	SInt16			dRefNum = 0;
	SInt16			first, last;

	SDL_numcds = 0;

	/* Check that the software is available */
	if (Gestalt(kGestaltAudioCDSelector, &SDL_cdversion) || 
			!SDL_cdversion) return(0);

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

	/* Walk the list, count each AudioCD driver, and save the refnums */
	first = -1;
	last = 0 - LMGetUnitTableEntryCount();
	for(dRefNum = first; dRefNum >= last; dRefNum--) {
		Str255		driverName;
		StringPtr	namePtr;
		DCtlHandle	deviceEntry;

		deviceEntry = GetDCtlEntry(dRefNum);
		if (! deviceEntry) continue;
		
		/* Is this an .AppleCD ? */
		namePtr = (*deviceEntry)->dCtlFlags & (1L << dRAMBased) ?
				((StringPtr) ((DCtlPtr) deviceEntry)->dCtlDriver + 18) :
				((StringPtr) (*deviceEntry)->dCtlDriver + 18);
		BlockMoveData(namePtr, driverName, namePtr[0]+1);
		if (driverName[0] > gDriverName[0]) driverName[0] = gDriverName[0];
		if (! EqualString(driverName, gDriverName, false, false)) continue;

		/* Record the basic info for each drive */
		SDL_cdlist[SDL_numcds].dRefNum = dRefNum;
		BlockMoveData(namePtr + 1, SDL_cdlist[SDL_numcds].name, namePtr[0]);
		SDL_cdlist[SDL_numcds].name[namePtr[0]] = 0;
		SDL_cdlist[SDL_numcds].hasAudio = false;
		SDL_numcds++;
	}
	return(0);
}

static const char *SDL_SYS_CDName(int drive)
{
	return(SDL_cdlist[drive].name);
}

static int get_drivenum(int drive)
{
	QHdr *driveQ = GetDrvQHdr();
	DrvQEl *driveElem;

	/* Update the drive number */
	SDL_cdlist[drive].driveNum = 0;
	if ( driveQ->qTail ) {
		driveQ->qTail->qLink = 0;
	}
	for ( driveElem=(DrvQEl *)driveQ->qHead; driveElem;
	      driveElem = (DrvQEl *)driveElem->qLink ) {
		if ( driveElem->dQRefNum == SDL_cdlist[drive].dRefNum ) {
			SDL_cdlist[drive].driveNum = driveElem->dQDrive;
			break;
		}
	}
	return(SDL_cdlist[drive].driveNum);
}

static int SDL_SYS_CDOpen(int drive)
{
	return(drive);
}

static int SDL_SYS_CDGetTOC(SDL_CD *cdrom)
{
	CDCntrlParam		cdpb;
	CDTrackData			tracks[SDL_MAX_TRACKS];
	long				i, leadout;

	/* Get the number of tracks on the CD by examining the TOC */
	SDL_memset(&cdpb, 0, sizeof(cdpb));
	cdpb.ioVRefNum = SDL_cdlist[cdrom->id].driveNum;
	cdpb.ioCRefNum = SDL_cdlist[cdrom->id].dRefNum;
	cdpb.csCode = kReadTOC;
	cdpb.csParam.words[0] = kGetTrackRange;
	if ( PBControlSync((ParmBlkPtr)&cdpb) != noErr ) {
		SDL_SetError("PBControlSync() failed");
		return(-1);
	}

	cdrom->numtracks = 
			SDL_SYS_BCDToShort(cdpb.csParam.bytes[1]) - 
			SDL_SYS_BCDToShort(cdpb.csParam.bytes[0]) + 1;
	if ( cdrom->numtracks > SDL_MAX_TRACKS )
		cdrom->numtracks = SDL_MAX_TRACKS;
	cdrom->status = CD_STOPPED;
	cdrom->cur_track = 0; /* Apparently these are set elsewhere */
	cdrom->cur_frame = 0; /* Apparently these are set elsewhere */


	/* Get the lead out area of the CD by examining the TOC */
	SDL_memset(&cdpb, 0, sizeof(cdpb));
	cdpb.ioVRefNum = SDL_cdlist[cdrom->id].driveNum;
	cdpb.ioCRefNum = SDL_cdlist[cdrom->id].dRefNum;
	cdpb.csCode = kReadTOC;
	cdpb.csParam.words[0] = kGetLeadOutArea;
	if ( PBControlSync((ParmBlkPtr)&cdpb) != noErr ) {
		SDL_SetError("PBControlSync() failed");
		return(-1);
	}

	leadout = MSF_TO_FRAMES(
			SDL_SYS_BCDToShort(cdpb.csParam.bytes[0]),
			SDL_SYS_BCDToShort(cdpb.csParam.bytes[1]),
			SDL_SYS_BCDToShort(cdpb.csParam.bytes[2]));

	/* Get an array of track locations by examining the TOC */
	SDL_memset(tracks, 0, sizeof(tracks));
	SDL_memset(&cdpb, 0, sizeof(cdpb));
	cdpb.ioVRefNum = SDL_cdlist[cdrom->id].driveNum;
	cdpb.ioCRefNum = SDL_cdlist[cdrom->id].dRefNum;
	cdpb.csCode = kReadTOC;
	cdpb.csParam.words[0] = kGetTrackEntries;	/* Type of Query */
	* ((long *) (cdpb.csParam.words+1)) = (long) tracks;				
	cdpb.csParam.words[3] = cdrom->numtracks * sizeof(tracks[0]);		
	* ((char *) (cdpb.csParam.words+4)) = 1;	/* First track */
	if ( PBControlSync((ParmBlkPtr)&cdpb) != noErr ) {
		SDL_SetError("PBControlSync() failed");
		return(-1);
	}

	/* Read all the track TOC entries */
	SDL_cdlist[cdrom->id].hasAudio = false;
	for ( i=0; i<cdrom->numtracks; ++i ) 
		{
		cdrom->track[i].id = i+1;
		if (tracks[i].entry.control & kDataTrackMask)
			cdrom->track[i].type = SDL_DATA_TRACK;
		else
			{
			cdrom->track[i].type = SDL_AUDIO_TRACK;
			SDL_cdlist[SDL_numcds].hasAudio = true;
			}
		
		cdrom->track[i].offset = MSF_TO_FRAMES(
				SDL_SYS_BCDToShort(tracks[i].entry.min),
				SDL_SYS_BCDToShort(tracks[i].entry.min),
				SDL_SYS_BCDToShort(tracks[i].entry.frame));
		cdrom->track[i].length = MSF_TO_FRAMES(
				SDL_SYS_BCDToShort(tracks[i+1].entry.min),
				SDL_SYS_BCDToShort(tracks[i+1].entry.min),
				SDL_SYS_BCDToShort(tracks[i+1].entry.frame)) -
				cdrom->track[i].offset;
		}
	
	/* Apparently SDL wants a fake last entry */
	cdrom->track[i].offset = leadout;
	cdrom->track[i].length = 0;

	return(0);
}

/* Get CD-ROM status */
static CDstatus SDL_SYS_CDStatus(SDL_CD *cdrom, int *position)
{
	CDCntrlParam cdpb;
	CDstatus status = CD_ERROR;
	Boolean spinning = false;

	if (position) *position = 0;

	/* Get the number of tracks on the CD by examining the TOC */
	if ( ! get_drivenum(cdrom->id) ) {
		return(CD_TRAYEMPTY);
	}
	SDL_memset(&cdpb, 0, sizeof(cdpb));
	cdpb.ioVRefNum = SDL_cdlist[cdrom->id].driveNum;
	cdpb.ioCRefNum = SDL_cdlist[cdrom->id].dRefNum;
	cdpb.csCode = kReadTOC;
	cdpb.csParam.words[0] = kGetTrackRange;
	if ( PBControlSync((ParmBlkPtr)&cdpb) != noErr ) {
		SDL_SetError("PBControlSync() failed");
		return(CD_ERROR);
	}

	cdrom->numtracks = 
			SDL_SYS_BCDToShort(cdpb.csParam.bytes[1]) - 
			SDL_SYS_BCDToShort(cdpb.csParam.bytes[0]) + 1;
	if ( cdrom->numtracks > SDL_MAX_TRACKS )
		cdrom->numtracks = SDL_MAX_TRACKS;
	cdrom->cur_track = 0; /* Apparently these are set elsewhere */
	cdrom->cur_frame = 0; /* Apparently these are set elsewhere */


	if (1 || SDL_cdlist[cdrom->id].hasAudio) {
		/* Get the current playback status */
		SDL_memset(&cdpb, 0, sizeof(cdpb));
		cdpb.ioVRefNum = SDL_cdlist[cdrom->id].driveNum;
		cdpb.ioCRefNum = SDL_cdlist[cdrom->id].dRefNum;
		cdpb.csCode = kAudioStatus;
		if ( PBControlSync((ParmBlkPtr)&cdpb) != noErr ) {
			SDL_SetError("PBControlSync() failed");
			return(-1);
		}
	
		switch(cdpb.csParam.cd.status) {
			case kStatusPlaying:
				status = CD_PLAYING;
				spinning = true;
				break;
			case kStatusPaused:
				status = CD_PAUSED;
				spinning = true;
				break;
			case kStatusMuted:
				status = CD_PLAYING; /* What should I do here? */
				spinning = true;
				break;
			case kStatusDone:
				status = CD_STOPPED;
				spinning = true;
				break;
			case kStatusStopped:
				status = CD_STOPPED;
				spinning = false;
				break;
			case kStatusError:
			default:
				status = CD_ERROR;
				spinning = false;
				break;
			}

		if (spinning && position) *position = MSF_TO_FRAMES(
				SDL_SYS_BCDToShort(cdpb.csParam.cd.minute),
				SDL_SYS_BCDToShort(cdpb.csParam.cd.second),
				SDL_SYS_BCDToShort(cdpb.csParam.cd.frame));
		}
	else
		status = CD_ERROR; /* What should I do here? */

	return(status);
}

/* Start play */
static int SDL_SYS_CDPlay(SDL_CD *cdrom, int start, int length)
{
	CDCntrlParam cdpb;

	/* Pause the current audio playback to avoid audible artifacts */
	if ( SDL_SYS_CDPause(cdrom) < 0 ) {
		return(-1);
	}

	/* Specify the AudioCD playback mode */
	SDL_memset(&cdpb, 0, sizeof(cdpb));
	cdpb.ioVRefNum = SDL_cdlist[cdrom->id].driveNum;
	cdpb.ioCRefNum = SDL_cdlist[cdrom->id].dRefNum;
	cdpb.csCode = kSetPlayMode;
	cdpb.csParam.bytes[0] = false;			/* Repeat? */
	cdpb.csParam.bytes[1] = kPlayModeSequential;	/* Play mode */
	/* ¥¥¥ÊTreat as soft error, NEC Drive doesnt support this call ¥¥¥ */
	PBControlSync((ParmBlkPtr) &cdpb);

#if 1
	/* Specify the end of audio playback */
	SDL_memset(&cdpb, 0, sizeof(cdpb));
	cdpb.ioVRefNum = SDL_cdlist[cdrom->id].driveNum;
	cdpb.ioCRefNum = SDL_cdlist[cdrom->id].dRefNum;
	cdpb.csCode = kAudioStop;
	cdpb.csParam.words[0] = kBlockPosition;		/* Position Mode */
	*(long *) (cdpb.csParam.words + 1) = start+length-1; /* Search Address */
	if ( PBControlSync((ParmBlkPtr)&cdpb) != noErr ) {
		SDL_SetError("PBControlSync() failed");
		return(-1);
	}

	/* Specify the start of audio playback, and start it */
	SDL_memset(&cdpb, 0, sizeof(cdpb));
	cdpb.ioVRefNum = SDL_cdlist[cdrom->id].driveNum;
	cdpb.ioCRefNum = SDL_cdlist[cdrom->id].dRefNum;
	cdpb.csCode = kAudioPlay;
	cdpb.csParam.words[0] = kBlockPosition;			/* Position Mode */
	*(long *) (cdpb.csParam.words + 1) = start+1;	/* Search Address */
	cdpb.csParam.words[3] = false;					/* Stop address? */
	cdpb.csParam.words[4] = kStereoPlayMode;		/* Audio Play Mode */
	if ( PBControlSync((ParmBlkPtr)&cdpb) != noErr ) {
		SDL_SetError("PBControlSync() failed");
		return(-1);
	}
#else
	/* Specify the end of audio playback */
	FRAMES_TO_MSF(start+length, &m, &s, &f);
	SDL_memset(&cdpb, 0, sizeof(cdpb));
	cdpb.ioVRefNum = SDL_cdlist[cdrom->id].driveNum;
	cdpb.ioCRefNum = SDL_cdlist[cdrom->id].dRefNum;
	cdpb.csCode = kAudioStop;
	cdpb.csParam.words[0] = kTrackPosition;			/* Position Mode */
	cdpb.csParam.words[1] = 0;						/* Search Address (hiword)*/
	cdpb.csParam.words[2] = 						/* Search Address (loword)*/
			SDL_SYS_ShortToBCD(cdrom->numtracks);	
	if ( PBControlSync((ParmBlkPtr)&cdpb) != noErr ) {
		SDL_SetError("PBControlSync() failed");
		return(-1);
	}

	/* Specify the start of audio playback, and start it */
	FRAMES_TO_MSF(start, &m, &s, &f);
	SDL_memset(&cdpb, 0, sizeof(cdpb));
	cdpb.ioVRefNum = SDL_cdlist[cdrom->id].driveNum;
	cdpb.ioCRefNum = SDL_cdlist[cdrom->id].dRefNum;
	cdpb.csCode = kAudioPlay;
	cdpb.csParam.words[0] = kTrackPosition;			/* Position Mode */
	cdpb.csParam.words[1] = 0;						/* Search Address (hiword)*/
	cdpb.csParam.words[2] = SDL_SYS_ShortToBCD(1);	/* Search Address (loword)*/
	cdpb.csParam.words[3] = false;					/* Stop address? */
	cdpb.csParam.words[4] = kStereoPlayMode;		/* Audio Play Mode */
	if ( PBControlSync((ParmBlkPtr)&cdpb) != noErr ) {
		SDL_SetError("PBControlSync() failed");
		return(-1);
	}
#endif

	return(0);
}

/* Pause play */
static int SDL_SYS_CDPause(SDL_CD *cdrom)
{
	CDCntrlParam cdpb;

	SDL_memset(&cdpb, 0, sizeof(cdpb));
	cdpb.ioVRefNum = SDL_cdlist[cdrom->id].driveNum;
	cdpb.ioCRefNum = SDL_cdlist[cdrom->id].dRefNum;
	cdpb.csCode = kAudioPause;
	cdpb.csParam.words[0] = 0;	/* Pause/Continue Flag (hiword) */
	cdpb.csParam.words[1] = 1;	/* Pause/Continue Flag (loword) */
	if ( PBControlSync((ParmBlkPtr)&cdpb) != noErr ) {
		SDL_SetError("PBControlSync() failed");
		return(-1);
	}
	return(0);
}

/* Resume play */
static int SDL_SYS_CDResume(SDL_CD *cdrom)
{
	CDCntrlParam cdpb;

	SDL_memset(&cdpb, 0, sizeof(cdpb));
	cdpb.ioVRefNum = SDL_cdlist[cdrom->id].driveNum;
	cdpb.ioCRefNum = SDL_cdlist[cdrom->id].dRefNum;
	cdpb.csCode = kAudioPause;
	cdpb.csParam.words[0] = 0;	/* Pause/Continue Flag (hiword) */
	cdpb.csParam.words[1] = 0;	/* Pause/Continue Flag (loword) */
	if ( PBControlSync((ParmBlkPtr)&cdpb) != noErr ) {
		SDL_SetError("PBControlSync() failed");
		return(-1);
	}
	return(0);
}

/* Stop play */
static int SDL_SYS_CDStop(SDL_CD *cdrom)
{
	CDCntrlParam cdpb;

	SDL_memset(&cdpb, 0, sizeof(cdpb));
	cdpb.ioVRefNum = SDL_cdlist[cdrom->id].driveNum;
	cdpb.ioCRefNum = SDL_cdlist[cdrom->id].dRefNum;
	cdpb.csCode = kAudioStop;
	cdpb.csParam.words[0] = 0;		/* Position Mode */
	cdpb.csParam.words[1] = 0;		/* Search Address (hiword) */
	cdpb.csParam.words[2] = 0;		/* Search Address (loword) */
	if ( PBControlSync((ParmBlkPtr)&cdpb) != noErr ) {
		SDL_SetError("PBControlSync() failed");
		return(-1);
	}
	return(0);
}

/* Eject the CD-ROM */
static int SDL_SYS_CDEject(SDL_CD *cdrom)
{
	Boolean	disk = false;
	QHdr *driveQ = GetDrvQHdr();
	DrvQEl *driveElem;
	HParamBlockRec hpb;
	ParamBlockRec cpb;

	for ( driveElem = (DrvQEl *) driveQ->qHead; driveElem; driveElem = 
			  (driveElem) ? ((DrvQEl *) driveElem->qLink) : 
			  ((DrvQEl *) driveQ->qHead) ) {
		if ( driveQ->qTail ) {
			driveQ->qTail->qLink = 0;
		}
		if ( driveElem->dQRefNum != SDL_cdlist[cdrom->id].dRefNum ) {
			continue;
		}
	
		/* Does drive contain mounted volume? If not, skip */
		SDL_memset(&hpb, 0, sizeof(hpb));
		hpb.volumeParam.ioVRefNum = driveElem->dQDrive;
		if ( PBHGetVInfoSync(&hpb) != noErr ) {
			continue;
		}
		if ( (UnmountVol(0, driveElem->dQDrive) == noErr) && 
		     (Eject(0, driveElem->dQDrive) == noErr) ) {
			driveElem = 0; /* Clear pointer to reset our loop */
			disk = true;
		}
	}

	/* If no disk is present, just eject the tray */
	if (! disk) {
		SDL_memset(&cpb, 0, sizeof(cpb));
		cpb.cntrlParam.ioVRefNum = 0; /* No Drive */
		cpb.cntrlParam.ioCRefNum = SDL_cdlist[cdrom->id].dRefNum;
		cpb.cntrlParam.csCode = kEjectTheDisc;
		if ( PBControlSync((ParmBlkPtr)&cpb) != noErr ) {
			SDL_SetError("PBControlSync() failed");
			return(-1);
		}
	}
	return(0);
}

/* Close the CD-ROM handle */
static void SDL_SYS_CDClose(SDL_CD *cdrom)
{
	return;
}

void SDL_SYS_CDQuit(void)
{
	while(SDL_numcds--) 
		SDL_memset(SDL_cdlist + SDL_numcds, 0, sizeof(SDL_cdlist[0]));
}

#endif /* SDL_CDROM_MACOS */
