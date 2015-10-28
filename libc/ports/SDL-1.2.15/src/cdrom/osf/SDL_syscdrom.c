/*
    Tru64 audio module for SDL (Simple DirectMedia Layer)
    Copyright (C) 2003

    This library is free software; you can redistribute it and/or
    modify it under the terms of the GNU Library General Public
    License as published by the Free Software Foundation; either
    version 2 of the License, or (at your option) any later version.

    This library is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
    Library General Public License for more details.

    You should have received a copy of the GNU Library General Public
    License along with this library; if not, write to the Free
    Foundation, Inc., 59 Temple Place, Suite 330, Boston, MA  02111-1307  USA


*/
#include "SDL_config.h"

#ifdef SDL_CDROM_OSF

/* Functions for system-level CD-ROM audio control */

/* #define DEBUG_CDROM 1 */

#include <sys/types.h>
#include <dirent.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <io/cam/cdrom.h>
#include <io/cam/rzdisk.h>
#include <io/common/devgetinfo.h>

#include "SDL_cdrom.h"
#include "../SDL_syscdrom.h"

/* The maximum number of CD-ROM drives we'll detect */
#define MAX_DRIVES 16

/* A list of available CD-ROM drives */
static char *SDL_cdlist[MAX_DRIVES];
static dev_t SDL_cdmode[MAX_DRIVES];

/* The system-dependent CD control functions */
static const char *SDL_SYS_CDName(int drive);
static int         SDL_SYS_CDOpen(int drive);
static int         SDL_SYS_CDGetTOC(SDL_CD *cdrom);
static CDstatus    SDL_SYS_CDStatus(SDL_CD *cdrom, int *position);
static int         SDL_SYS_CDPlay(SDL_CD *cdrom, int start, int length);
static int         SDL_SYS_CDPause(SDL_CD *cdrom);
static int         SDL_SYS_CDResume(SDL_CD *cdrom);
static int         SDL_SYS_CDStop(SDL_CD *cdrom);
static int         SDL_SYS_CDEject(SDL_CD *cdrom);
static void        SDL_SYS_CDClose(SDL_CD *cdrom);

/* Check a drive to see if it is a CD-ROM */
/* Caution!! Not tested. */ 
static int CheckDrive(char *drive, struct stat *stbuf)
{
    int cdfd, is_cd = 0;
    struct mode_sel_sns_params msp;
    struct inquiry_info inq;

#ifdef DEBUG_CDROM
    char *devtype[] = {"Disk", "Tape", "Printer", "Processor", "WORM",
	"CD-ROM", "Scanner", "Optical", "Changer", "Comm", "Unknown"};
#endif

    bzero(&msp, sizeof(msp));
    bzero(&inq, sizeof(inq));

    /* If it doesn't exist, return -1 */
    if ( stat(drive, stbuf) < 0 ) {
	return(-1);
    }

    if ( (cdfd = open(drive, (O_RDWR|O_NDELAY), 0)) >= 0 ) {
	msp.msp_addr   =   (caddr_t) &inq;
	msp.msp_pgcode =                0;
	msp.msp_pgctrl =                0;
	msp.msp_length =      sizeof(inq);
	msp.msp_setps  =                0;

	if ( ioctl(cdfd, SCSI_GET_INQUIRY_DATA, &msp) )
	    return (0);

#ifdef DEBUG_CDROM
	fprintf(stderr, "Device Type: %s\n", devtype[inq.perfdt]);
	fprintf(stderr, "Vendor: %.8s\n", inq.vndrid);
	fprintf(stderr, "Product: %.8s\n", inq.prodid);
	fprintf(stderr, "Revision: %.8s\n", inq.revlvl);
#endif
	if ( inq.perfdt == DTYPE_RODIRECT )
	    is_cd = 1;
    }

    return(is_cd);
}

/* Add a CD-ROM drive to our list of valid drives */
static void AddDrive(char *drive, struct stat *stbuf)
{
    int i;

    if ( SDL_numcds < MAX_DRIVES ) {
	/* Check to make sure it's not already in our list.
	 * This can happen when we see a drive via symbolic link.
	 *
	 */
	for ( i=0; i<SDL_numcds; ++i ) {
	    if ( stbuf->st_rdev == SDL_cdmode[i] ) {
#ifdef DEBUG_CDROM
  fprintf(stderr, "Duplicate drive detected: %s == %s\n", drive, SDL_cdlist[i]);
#endif
	    return;
	    }
	}

	/* Add this drive to our list */
	i = SDL_numcds;
	SDL_cdlist[i] = SDL_strdup(drive);
	if ( SDL_cdlist[i] == NULL ) {
	    SDL_OutOfMemory();
	    return;
	}
	SDL_cdmode[i] = stbuf->st_rdev;
	++SDL_numcds;
#ifdef DEBUG_CDROM
  fprintf(stderr, "Added CD-ROM drive: %s\n", drive);
#endif
    }
}

int  SDL_SYS_CDInit(void)
{
    /* checklist:
     *
     * Tru64 5.X (/dev/rdisk/cdrom?c)
     * dir: /dev/rdisk, name: cdrom
     *
     * Digital UNIX 4.0X (/dev/rrz?c)
     * dir: /dev, name: rrz
     *
     */
    struct {
	char *dir;
	char *name;
    } checklist[] = {
	{"/dev/rdisk", "cdrom"},
	{"/dev", "rrz"},
	{NULL, NULL}};
    char drive[32];
    char *SDLcdrom;
    int i, j, exists;
    struct stat stbuf;

    /* Fill in our driver capabilities */
    SDL_CDcaps.Name   = SDL_SYS_CDName;
    SDL_CDcaps.Open   = SDL_SYS_CDOpen;
    SDL_CDcaps.GetTOC = SDL_SYS_CDGetTOC;
    SDL_CDcaps.Status = SDL_SYS_CDStatus;
    SDL_CDcaps.Play   = SDL_SYS_CDPlay;
    SDL_CDcaps.Pause  = SDL_SYS_CDPause;
    SDL_CDcaps.Resume = SDL_SYS_CDResume;
    SDL_CDcaps.Stop   = SDL_SYS_CDStop;
    SDL_CDcaps.Eject  = SDL_SYS_CDEject;
    SDL_CDcaps.Close  = SDL_SYS_CDClose;


    /* Look in the environment for our CD-ROM drive list */
    SDLcdrom = SDL_getenv("SDL_CDROM");	/* ':' separated list of devices */
    if ( SDLcdrom != NULL ) {
	char *cdpath, *delim;
	size_t len = SDL_strlen(SDLcdrom)+1;
	cdpath = SDL_stack_alloc(char, len);
	if ( cdpath != NULL ) {
	    SDL_strlcpy(cdpath, SDLcdrom, len);
	    SDLcdrom = cdpath;
	    do {
		delim = SDL_strchr(SDLcdrom, ':');
		if ( delim ) {
		    *delim++ = '\0';
		}
		if ( CheckDrive(SDLcdrom, &stbuf) > 0 ) {
		    AddDrive(SDLcdrom, &stbuf);
		}
		if ( delim ) {
		    SDLcdrom = delim;
		} else {
		    SDLcdrom = NULL;
		}
	    } while ( SDLcdrom );
	    SDL_stack_free(cdpath);
	}

	/* If we found our drives, there's nothing left to do */
	if ( SDL_numcds > 0 ) {
	    return(0);
	}
    }
    /* Scan the system for CD-ROM drives */
    for ( i = 0; checklist[i].dir; ++i) {
	DIR *devdir;
	struct dirent *devent;
	int name_len;

	devdir = opendir(checklist[i].dir);
	if (devdir) {
	    name_len = SDL_strlen(checklist[i].name);
	    while (devent = readdir(devdir))
		if (SDL_memcmp(checklist[i].name, devent->d_name, name_len) == 0)
		    if (devent->d_name[devent->d_namlen-1] == 'c') {
			SDL_snprintf(drive, SDL_arraysize(drive), "%s/%s", checklist[i].dir, devent->d_name);
#ifdef DEBUG_CDROM
			fprintf(stderr, "Try to add drive: %s\n", drive);
#endif
			if ( CheckDrive(drive, &stbuf) > 0 )
			    AddDrive(drive, &stbuf);
		    }
	    closedir(devdir);
	} else {
#ifdef DEBUG_CDROM
	    fprintf(stderr, "cannot open dir: %s\n", checklist[i].dir);
#endif
	}
    }
    return (0);
}

static const char *SDL_SYS_CDName(int drive)
{
    return(SDL_cdlist[drive]);
}

static int SDL_SYS_CDOpen(int drive)
{
    /* O_RDWR: To use ioctl(fd, SCSI_STOP_UNIT) */
    return(open(SDL_cdlist[drive], (O_RDWR|O_NDELAY), 0));
}

static int SDL_SYS_CDGetTOC(SDL_CD *cdrom)
{
    struct cd_toc                  toc;
    struct cd_toc_header           hdr;
    struct cd_toc_entry          *cdte;
    int i;
    int okay = 0;
    if ( ioctl(cdrom->id, CDROM_TOC_HEADER, &hdr) ) {
	fprintf(stderr,"ioctl error CDROM_TOC_HEADER\n");
	return -1;
    }
    cdrom->numtracks = hdr.th_ending_track - hdr.th_starting_track + 1;
    if ( cdrom->numtracks > SDL_MAX_TRACKS ) {
	cdrom->numtracks = SDL_MAX_TRACKS;
    }
#ifdef DEBUG_CDROM
  fprintf(stderr,"hdr.th_data_len1 = %d\n", hdr.th_data_len1);
  fprintf(stderr,"hdr.th_data_len0 = %d\n", hdr.th_data_len0);
  fprintf(stderr,"hdr.th_starting_track = %d\n", hdr.th_starting_track);
  fprintf(stderr,"hdr.th_ending_track = %d\n", hdr.th_ending_track);
  fprintf(stderr,"cdrom->numtracks = %d\n", cdrom->numtracks);
#endif
    toc.toc_address_format = CDROM_LBA_FORMAT;
    toc.toc_starting_track = 0;
    toc.toc_alloc_length = (hdr.th_data_len1 << 8) +
			    hdr.th_data_len0 + sizeof(hdr);
    if ( (toc.toc_buffer = alloca(toc.toc_alloc_length)) == NULL) {
	fprintf(stderr,"cannot allocate toc.toc_buffer\n");
	return -1;
    }

    bzero (toc.toc_buffer, toc.toc_alloc_length);
    if (ioctl(cdrom->id, CDROM_TOC_ENTRYS, &toc)) {
	fprintf(stderr,"ioctl error CDROM_TOC_ENTRYS\n");
	return -1;
    }

    cdte =(struct cd_toc_entry *) ((char *) toc.toc_buffer + sizeof(hdr));
    for (i=0; i <= cdrom->numtracks; ++i) {
	if (i == cdrom->numtracks ) {
	    cdrom->track[i].id = 0xAA;;
	} else {
	    cdrom->track[i].id = hdr.th_starting_track + i;
	}

	cdrom->track[i].type =
	    cdte[i].te_control & CDROM_DATA_TRACK;
	cdrom->track[i].offset =
	    cdte[i].te_absaddr.lba.addr3 << 24 |
	    cdte[i].te_absaddr.lba.addr2 << 16 |
	    cdte[i].te_absaddr.lba.addr1 << 8  |
	    cdte[i].te_absaddr.lba.addr0;
	cdrom->track[i].length = 0;
	if ( i > 0 ) {
	    cdrom->track[i - 1].length =
		cdrom->track[i].offset -
		cdrom->track[i - 1].offset;
	}
    }
#ifdef DEBUG_CDROM
  for (i = 0; i <= cdrom->numtracks; i++) {
    fprintf(stderr,"toc_entry[%d].te_track_number = %d\n",
	    i,cdte[i].te_track_number);
    fprintf(stderr,"cdrom->track[%d].id = %d\n", i,cdrom->track[i].id);
    fprintf(stderr,"cdrom->track[%d].type = %x\n", i,cdrom->track[i].type);
    fprintf(stderr,"cdrom->track[%d].offset = %d\n", i,cdrom->track[i].offset);
    fprintf(stderr,"cdrom->track[%d].length = %d\n", i,cdrom->track[i].length);
  }
#endif
    if ( i == (cdrom->numtracks+1) ) {
	okay = 1;
    }

    return(okay ? 0 : -1);
}

/* Get CD-ROM status */
static CDstatus SDL_SYS_CDStatus(SDL_CD *cdrom, int *position)
{
    CDstatus                     status;
    struct cd_sub_channel            sc;
    struct cd_subc_channel_data     scd;

    sc.sch_address_format = CDROM_LBA_FORMAT;
    sc.sch_data_format    = CDROM_CURRENT_POSITION;
    sc.sch_track_number   = 0;
    sc.sch_alloc_length   = sizeof(scd);
    sc.sch_buffer         = (caddr_t)&scd;
    if ( ioctl(cdrom->id, CDROM_READ_SUBCHANNEL, &sc) ) {
	status = CD_ERROR;
	fprintf(stderr,"ioctl error CDROM_READ_SUBCHANNEL \n");
    } else {
	switch (scd.scd_header.sh_audio_status) {
	    case AS_AUDIO_INVALID:
		status = CD_STOPPED;
		break;
	    case AS_PLAY_IN_PROGRESS:
		status = CD_PLAYING;
		break;
	    case AS_PLAY_PAUSED:
		status = CD_PAUSED;
		break;
	    case AS_PLAY_COMPLETED:
		status = CD_STOPPED;
		break;
	    case AS_PLAY_ERROR:
		status = CD_ERROR;
		break;
	    case AS_NO_STATUS:
		status = CD_STOPPED;
		break;
	    default:
		status = CD_ERROR;
		break;
	}
#ifdef DEBUG_CDROM
  fprintf(stderr,"scd.scd_header.sh_audio_status = %x\n",
	scd.scd_header.sh_audio_status);
#endif
    }
    if (position) {
	if (status == CD_PLAYING || (status == CD_PAUSED) ) {
	    *position =
		scd.scd_position_data.scp_absaddr.lba.addr3 << 24 |
		scd.scd_position_data.scp_absaddr.lba.addr2 << 16 |
		scd.scd_position_data.scp_absaddr.lba.addr1 << 8  |
		scd.scd_position_data.scp_absaddr.lba.addr0;
	} else {
	    *position = 0;
	}
    }

    return status;
}

/* Start play */
static int SDL_SYS_CDPlay(SDL_CD *cdrom, int start, int length)
{
/*
 * Play MSF
 */
    struct cd_play_audio_msf msf;
    int end;

    bzero(&msf, sizeof(msf));
    end = start +length;
    FRAMES_TO_MSF(start + 150, /* LBA = 4500*M + 75*S + F - 150 */
		  &msf.msf_starting_M_unit,
		  &msf.msf_starting_S_unit,
		  &msf.msf_starting_F_unit);
    FRAMES_TO_MSF(end + 150, /* LBA = 4500*M + 75*S + F - 150 */
		  &msf.msf_ending_M_unit,
		  &msf.msf_ending_S_unit,
		  &msf.msf_ending_F_unit);

    return(ioctl(cdrom->id, CDROM_PLAY_AUDIO_MSF, &msf));
}

/* Pause play */
static int SDL_SYS_CDPause(SDL_CD *cdrom)
{
    return(ioctl(cdrom->id, CDROM_PAUSE_PLAY));
}

/* Resume play */
static int SDL_SYS_CDResume(SDL_CD *cdrom)
{
    return(ioctl(cdrom->id, CDROM_RESUME_PLAY));
}

/* Stop play */
static int SDL_SYS_CDStop(SDL_CD *cdrom)
{
    return(ioctl(cdrom->id, SCSI_STOP_UNIT));
}

/* Eject the CD-ROM */
static int SDL_SYS_CDEject(SDL_CD *cdrom)
{
    return(ioctl(cdrom->id, CDROM_EJECT_CADDY));
}

/* Close the CD-ROM handle */
static void SDL_SYS_CDClose(SDL_CD *cdrom)
{
    close(cdrom->id);
}

void SDL_SYS_CDQuit(void)
{
    int i;

    if ( SDL_numcds > 0 ) {
	for ( i=0; i<SDL_numcds; ++i ) {
	    SDL_free(SDL_cdlist[i]);
	}
	SDL_numcds = 0;
    }
}

#endif /* SDL_CDROM_OSF */
