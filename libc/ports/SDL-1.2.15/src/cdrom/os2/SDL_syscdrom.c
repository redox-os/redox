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

#ifdef SDL_CDROM_OS2

/* Functions for system-level CD-ROM audio control */

#define INCL_MCIOS2
#include <os2.h>
#include <os2me.h>

#include "SDL_cdrom.h"
#include "../SDL_syscdrom.h"

/* Size of MCI result buffer (in bytes) */
#define MCI_CMDRETBUFSIZE	128

/* The maximum number of CD-ROM drives we'll detect */
#define MAX_DRIVES	16	

/* A list of available CD-ROM drives */
static char *SDL_cdlist[MAX_DRIVES];
//static dev_t SDL_cdmode[MAX_DRIVES];

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

/* MCI Timing Functions */
#define	MCI_MMTIMEPERSECOND		3000
#define	FRAMESFROMMM(mmtime)		(((mmtime)*CD_FPS)/MCI_MMTIMEPERSECOND)


/* Ready for MCI CDAudio Devices */
int  SDL_SYS_CDInit(void)
{
int i; /* generig counter */
MCI_SYSINFO_PARMS		msp;	/* Structure to MCI SysInfo parameters */
CHAR 						SysInfoRet[MCI_CMDRETBUFSIZE];	/* Buffer for MCI Command result */

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

/* Get the number of CD ROMs in the System */
/* Clean SysInfo structure */
SDL_memset(&msp, 0x00, sizeof(MCI_SYSINFO_PARMS));
/* Prepare structure to Ask Numer of Audio CDs */
msp.usDeviceType = MCI_DEVTYPE_CD_AUDIO;	/* CD Audio Type */
msp.pszReturn = (PSZ)&SysInfoRet; 	/* Return Structure */
msp.ulRetSize = MCI_CMDRETBUFSIZE; 	/* Size of ret struct */
if (LOUSHORT(mciSendCommand(0,MCI_SYSINFO, MCI_SYSINFO_QUANTITY | MCI_WAIT, (PVOID)&msp, 0)) != MCIERR_SUCCESS) return(CD_ERROR);
SDL_numcds = atoi(SysInfoRet);
if (SDL_numcds > MAX_DRIVES) SDL_numcds = MAX_DRIVES; /* Limit maximum CD number */

/* Get and Add their system name to the SDL_cdlist */
msp.pszReturn = (PSZ)&SysInfoRet; 				/* Return Structure */
msp.ulRetSize = MCI_CMDRETBUFSIZE; 			/* Size of ret struct */
msp.usDeviceType = MCI_DEVTYPE_CD_AUDIO;		/* CD Audio Type */
for (i=0; i<SDL_numcds; i++)
	{
	msp.ulNumber = i+1;
	mciSendCommand(0,MCI_SYSINFO, MCI_SYSINFO_NAME | MCI_WAIT,&msp, 0);
	SDL_cdlist[i] = SDL_strdup(SysInfoRet);
	if ( SDL_cdlist[i] == NULL )
		{
		SDL_OutOfMemory();
		return(-1);
		}
	}
return(0);
}

/* Return CDAudio System Dependent Device Name - Ready for MCI*/
static const char *SDL_SYS_CDName(int drive)
{
return(SDL_cdlist[drive]);
}

/* Open CDAudio Device - Ready for MCI */
static int SDL_SYS_CDOpen(int drive)
{
MCI_OPEN_PARMS	mop;
MCI_SET_PARMS msp;
MCI_GENERIC_PARMS mgp;

/* Open the device */
mop.hwndCallback = (HWND)NULL;		// None
mop.usDeviceID = (USHORT)NULL;		// Will be returned.
mop.pszDeviceType = (PSZ)SDL_cdlist[drive];		// CDAudio Device
if (LOUSHORT(mciSendCommand(0,MCI_OPEN,MCI_WAIT,&mop, 0)) != MCIERR_SUCCESS) return(CD_ERROR);
/* Set time format */
msp.hwndCallback = (HWND)NULL;		// None
msp.ulTimeFormat = MCI_FORMAT_MSF;	// Minute : Second : Frame structure
msp.ulSpeedFormat = (ULONG)NULL;		// No change
msp.ulAudio = (ULONG)NULL;				// No Channel
msp.ulLevel = (ULONG)NULL;				// No Volume
msp.ulOver = (ULONG)NULL;				// No Delay
msp.ulItem = (ULONG)NULL;				// No item
msp.ulValue = (ULONG)NULL;				// No value for item flag
if (LOUSHORT(mciSendCommand(mop.usDeviceID,MCI_SET,MCI_WAIT | MCI_SET_TIME_FORMAT,&msp, 0)) == MCIERR_SUCCESS) return (mop.usDeviceID);
/* Error setting time format? - Close opened device */
mgp.hwndCallback = (HWND)NULL;		// None
mciSendCommand(mop.usDeviceID,MCI_CLOSE,MCI_WAIT,&mgp, 0);
return(CD_ERROR);
}

/* Get CD Table Of Contents - Ready for MCI */
static int SDL_SYS_CDGetTOC(SDL_CD *cdrom)
{
MCI_TOC_PARMS mtp;
MCI_STATUS_PARMS msp;
MCI_TOC_REC * mtr;
INT i;

/* Correction because MCI cannot read TOC while CD is playing (it'll stop!) */
if (cdrom->status == CD_PLAYING || cdrom->status == CD_PAUSED) return 0;

/* Get Number of Tracks */
msp.hwndCallback = (HWND)NULL; /* None */
msp.ulReturn = (ULONG)NULL; /* We want this information */
msp.ulItem = MCI_STATUS_NUMBER_OF_TRACKS;
msp.ulValue = (ULONG)NULL; /* No additional information */
if (LOUSHORT(mciSendCommand(cdrom->id,MCI_STATUS,MCI_WAIT | MCI_STATUS_ITEM,&msp, 0)) != MCIERR_SUCCESS) return(CD_ERROR);
cdrom->numtracks = msp.ulReturn;
if ( cdrom->numtracks > SDL_MAX_TRACKS )
	{
	cdrom->numtracks = SDL_MAX_TRACKS;
	}
/* Alocate space for TOC data */
mtr = (MCI_TOC_REC *)SDL_malloc(cdrom->numtracks*sizeof(MCI_TOC_REC));
if ( mtr == NULL )
	{
	SDL_OutOfMemory();
	return(-1);
	}
/* Get TOC from CD */
mtp.pBuf = mtr;
mtp.ulBufSize = cdrom->numtracks*sizeof(MCI_TOC_REC);
if (LOUSHORT(mciSendCommand(cdrom->id,MCI_GETTOC,MCI_WAIT,&mtp, 0)) != MCIERR_SUCCESS)
	{
	SDL_OutOfMemory();
	SDL_free(mtr);
	return(CD_ERROR);
	}
/* Fill SDL Tracks Structure */
for (i=0; i<cdrom->numtracks; i++)
	{
	/* Set Track ID */
	cdrom->track[i].id = (mtr+i)->TrackNum;
	/* Set Track Type */
	msp.hwndCallback = (HWND)NULL; /* None */
	msp.ulReturn = (ULONG)NULL; /* We want this information */
	msp.ulItem = MCI_CD_STATUS_TRACK_TYPE;
	msp.ulValue = (ULONG)((mtr+i)->TrackNum); /* Track Number? */
	if (LOUSHORT(mciSendCommand(cdrom->id,MCI_STATUS,MCI_WAIT | MCI_TRACK | MCI_STATUS_ITEM,&msp, 0)) != MCIERR_SUCCESS)
		{
		SDL_free(mtr);
		return (CD_ERROR);
		}
	if (msp.ulReturn==MCI_CD_TRACK_AUDIO) cdrom->track[i].type = SDL_AUDIO_TRACK;
	else cdrom->track[i].type = SDL_DATA_TRACK;
	/* Set Track Length - values from MCI are in MMTIMEs - 3000 MMTIME = 1 second */
	cdrom->track[i].length = FRAMESFROMMM((mtr+i)->ulEndAddr - (mtr+i)->ulStartAddr);
	/* Set Track Offset */
	cdrom->track[i].offset = FRAMESFROMMM((mtr+i)->ulStartAddr);
	}
SDL_free(mtr);
return(0);
}


/* Get CD-ROM status - Ready for MCI */
static CDstatus SDL_SYS_CDStatus(SDL_CD *cdrom, int *position)
{
CDstatus status;
MCI_STATUS_PARMS msp;

/* Get Status from MCI */
msp.hwndCallback = (HWND)NULL; /* None */
msp.ulReturn = (ULONG)NULL; /* We want this information */
msp.ulItem = MCI_STATUS_MODE;
msp.ulValue = (ULONG)NULL; /* No additional information */
if (LOUSHORT(mciSendCommand(cdrom->id,MCI_STATUS,MCI_WAIT | MCI_STATUS_ITEM,&msp, 0)) != MCIERR_SUCCESS) status = CD_ERROR;
else
	{
	switch(msp.ulReturn)
		{
		case	MCI_MODE_NOT_READY:
			status = CD_TRAYEMPTY;
			break;
		case	MCI_MODE_PAUSE:
			status = CD_PAUSED;
			break;
		case	MCI_MODE_PLAY:
			status = CD_PLAYING;
			break;
		case	MCI_MODE_STOP:
			status = CD_STOPPED;
			break;
		/* These cases should not occour */
		case	MCI_MODE_RECORD:
		case	MCI_MODE_SEEK:
		default:
			status = CD_ERROR;
			break;
		}
	}

/* Determine position */
if (position != NULL) /* The SDL $&$&%# CDROM call sends NULL pointer here! */
	{
		if ((status == CD_PLAYING) || (status == CD_PAUSED))
		{
		/* Get Position */
		msp.hwndCallback = (HWND)NULL; /* None */
		msp.ulReturn = (ULONG)NULL; /* We want this information */
		msp.ulItem = MCI_STATUS_POSITION;
		msp.ulValue = (ULONG)NULL; /* No additiona info */
		if (LOUSHORT(mciSendCommand(cdrom->id,MCI_STATUS,MCI_WAIT | MCI_STATUS_ITEM,&msp, 0)) != MCIERR_SUCCESS) return (CD_ERROR);
		/* Convert from MSF (format selected in the Open process) to Frames (format that will be returned) */
		*position = MSF_TO_FRAMES(MSF_MINUTE(msp.ulReturn),MSF_SECOND(msp.ulReturn),MSF_FRAME(msp.ulReturn));
		}
	else *position = 0;
	}
return(status);
}

/* Start play - Ready for MCI */
static int SDL_SYS_CDPlay(SDL_CD *cdrom, int start, int length)
{
MCI_GENERIC_PARMS mgp;
MCI_STATUS_PARMS msp;
MCI_PLAY_PARMS	mpp;
ULONG min,sec,frm;

/* Start MSF */
FRAMES_TO_MSF(start, &min, &sec, &frm);
MSF_MINUTE(mpp.ulFrom) = min;
MSF_SECOND(mpp.ulFrom) = sec;
MSF_FRAME(mpp.ulFrom) = frm;
/* End MSF */
FRAMES_TO_MSF(start+length, &min, &sec, &frm);
MSF_MINUTE(mpp.ulTo) = min;
MSF_SECOND(mpp.ulTo) = sec;
MSF_FRAME(mpp.ulTo) = frm;
#ifdef DEBUG_CDROM
	fprintf(stderr, "Trying to play from %d:%d:%d to %d:%d:%d\n",
	playtime.cdmsf_min0, playtime.cdmsf_sec0, playtime.cdmsf_frame0,
	playtime.cdmsf_min1, playtime.cdmsf_sec1, playtime.cdmsf_frame1);
#endif
/* Verifies if it is paused first... and if it is, unpause before stopping it. */
msp.hwndCallback = (HWND)NULL; /* None */
msp.ulReturn = (ULONG)NULL; /* We want this information */
msp.ulItem = MCI_STATUS_MODE;
msp.ulValue = (ULONG)NULL; /* No additional information */
if (LOUSHORT(mciSendCommand(cdrom->id,MCI_STATUS,MCI_WAIT | MCI_STATUS_ITEM,&msp, 0)) == MCIERR_SUCCESS)
	{
	if (msp.ulReturn == MCI_MODE_PAUSE)
		{
		mgp.hwndCallback = (HWND)NULL;		// None
		mciSendCommand(cdrom->id,MCI_RESUME,0,&mgp, 0);
		}
	}
/* Now play it. */
mpp.hwndCallback = (HWND)NULL;		// We do not want the info. temp
if (LOUSHORT(mciSendCommand(cdrom->id,MCI_PLAY,MCI_FROM | MCI_TO,&mpp, 0)) == MCIERR_SUCCESS) return 0;
return (CD_ERROR);
}

/* Pause play - Ready for MCI */
static int SDL_SYS_CDPause(SDL_CD *cdrom)
{
MCI_GENERIC_PARMS mgp;

mgp.hwndCallback = (HWND)NULL;		// None
if (LOUSHORT(mciSendCommand(cdrom->id,MCI_PAUSE,MCI_WAIT,&mgp, 0)) == MCIERR_SUCCESS) return 0;
return(CD_ERROR);
}

/* Resume play - Ready for MCI */
static int SDL_SYS_CDResume(SDL_CD *cdrom)
{
MCI_GENERIC_PARMS mgp;

mgp.hwndCallback = (HWND)NULL;		// None
if (LOUSHORT(mciSendCommand(cdrom->id,MCI_RESUME,MCI_WAIT,&mgp, 0)) == MCIERR_SUCCESS) return 0;
return(CD_ERROR);
}

/* Stop play - Ready for MCI */
static int SDL_SYS_CDStop(SDL_CD *cdrom)
{
MCI_GENERIC_PARMS mgp;
MCI_STATUS_PARMS msp;

/* Verifies if it is paused first... and if it is, unpause before stopping it. */
msp.hwndCallback = (HWND)NULL; /* None */
msp.ulReturn = (ULONG)NULL; /* We want this information */
msp.ulItem = MCI_STATUS_MODE;
msp.ulValue = (ULONG)NULL; /* No additional information */
if (LOUSHORT(mciSendCommand(cdrom->id,MCI_STATUS,MCI_WAIT | MCI_STATUS_ITEM,&msp, 0)) == MCIERR_SUCCESS)
	{
	if (msp.ulReturn == MCI_MODE_PAUSE)
		{
		mgp.hwndCallback = (HWND)NULL;		// None
		mciSendCommand(cdrom->id,MCI_RESUME,0,&mgp, 0);
		}
	}
/* Now stops the media */
mgp.hwndCallback = (HWND)NULL;		// None
if (LOUSHORT(mciSendCommand(cdrom->id,MCI_STOP,MCI_WAIT,&mgp, 0)) == MCIERR_SUCCESS) return 0;
return(CD_ERROR);
}

/* Eject the CD-ROM - Ready for MCI */
static int SDL_SYS_CDEject(SDL_CD *cdrom)
{
MCI_SET_PARMS msp;

msp.hwndCallback = (HWND)NULL;		// None
msp.ulTimeFormat = (ULONG)NULL;		// No change
msp.ulSpeedFormat = (ULONG)NULL;		// No change
msp.ulAudio = (ULONG)NULL;				// No Channel
msp.ulLevel = (ULONG)NULL;				// No Volume
msp.ulOver = (ULONG)NULL;				// No Delay
msp.ulItem = (ULONG)NULL;					// No item
msp.ulValue = (ULONG)NULL;					// No value for item flag
if (LOUSHORT(mciSendCommand(cdrom->id,MCI_SET,MCI_WAIT | MCI_SET_DOOR_OPEN,&msp, 0)) == MCIERR_SUCCESS) return 0;
return(CD_ERROR);
}

/* Close the CD-ROM handle - Ready for MCI */
static void SDL_SYS_CDClose(SDL_CD *cdrom)
{
MCI_GENERIC_PARMS mgp;

mgp.hwndCallback = (HWND)NULL;		// None
mciSendCommand(cdrom->id,MCI_CLOSE,MCI_WAIT,&mgp, 0);
}

/* Finalize CDROM Subsystem - Ready for MCI */
void SDL_SYS_CDQuit(void)
{
int i;

if ( SDL_numcds > 0 )
	{
	for ( i=0; i<SDL_numcds; ++i )
		{
		SDL_free(SDL_cdlist[i]);
		}
	SDL_numcds = 0;
	}
}

#endif /* SDL_CDROM_OS2 */
