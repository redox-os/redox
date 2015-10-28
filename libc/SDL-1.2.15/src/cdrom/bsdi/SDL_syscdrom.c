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

#ifdef SDL_CDROM_BSDI

/*
 * Functions for system-level CD-ROM audio control for BSD/OS 4.x
 * This started life out as a copy of the freebsd/SDL_cdrom.c file but was
 * heavily modified.   Works for standard (MMC) SCSI and ATAPI CDrom drives.
 *
 * Steven Schultz - sms@to.gd-es.com
*/

#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <err.h>
#include <unistd.h>
#include <sys/ioctl.h>
#include </sys/dev/scsi/scsi.h>
#include </sys/dev/scsi/scsi_ioctl.h>

#include "SDL_cdrom.h"
#include "../SDL_syscdrom.h"

/*
 * The msf_to_frame and frame_to_msf were yanked from libcdrom and inlined
 * here so that -lcdrom doesn't have to be dragged in for something so simple.
*/

#define	FRAMES_PER_SECOND	75
#define	FRAMES_PER_MINUTE	(FRAMES_PER_SECOND * 60)

int
msf_to_frame(int minute, int second, int frame)
	{
	return(minute * FRAMES_PER_MINUTE + second * FRAMES_PER_SECOND + frame);
	}

void
frame_to_msf(int frame, int *minp, int *secp, int *framep)
	{
	*minp = frame / FRAMES_PER_MINUTE;
	*secp = (frame % FRAMES_PER_MINUTE) / FRAMES_PER_SECOND;
	*framep = frame % FRAMES_PER_SECOND;
	}

/* The maximum number of CD-ROM drives we'll detect */
#define MAX_DRIVES	16	

/* A list of available CD-ROM drives */
static char *SDL_cdlist[MAX_DRIVES];
static dev_t SDL_cdmode[MAX_DRIVES];

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

typedef	struct	scsi_cdb cdb_t;

static int scsi_cmd(int fd,
		struct scsi_cdb *cdb,
		int cdblen, 
		int rw,
		caddr_t data,
		int datalen,
		struct scsi_user_cdb *sus)
	{
	int	scsistatus;
	unsigned char	*cp;
	struct	scsi_user_cdb suc;

    /* safety checks */
	if	(!cdb) return(-1);
	if	(rw != SUC_READ && rw != SUC_WRITE) return(-1);

	suc.suc_flags = rw;
	suc.suc_cdblen = cdblen;
	bcopy(cdb, suc.suc_cdb, cdblen);
	suc.suc_datalen = datalen;
	suc.suc_data = data;
	suc.suc_timeout = 10;		/* 10 secs max for TUR or SENSE */
	if	(ioctl(fd, SCSIRAWCDB, &suc) == -1)
		return(-11);
	scsistatus = suc.suc_sus.sus_status;
	cp = suc.suc_sus.sus_sense;

/*
 * If a place to copy the sense data back to has been provided then the
 * caller is responsible for checking the errors and printing any information
 * out if the status was not successful.
*/
	if	(scsistatus != 0 && !sus)
		{
		fprintf(stderr,"scsistatus = %x cmd = %x\n",
			scsistatus, cdb[0]);
		fprintf(stderr, "sense %x %x %x %x %x %x %x %x %x %x %x %x %x %x %x %x\n", 
			cp[0], cp[1], cp[2], cp[3], cp[4], cp[5],
			cp[6], cp[7], cp[8], cp[9], cp[10], cp[11],
			cp[12], cp[13], cp[14], cp[15]);
		return(1);
		}
	if	(sus)
		bcopy(&suc, sus, sizeof (struct scsi_user_cdb));
	if	(scsistatus)
		return(1);	/* Return non-zero for unsuccessful status */
	return(0);
	}

/* request vendor brand and model */
unsigned char *Inquiry(int fd)
	{
	static struct scsi_cdb6 cdb = 
		{
		0x12, 
		0, 0, 0,
		56,
		0
		};
	static unsigned char Inqbuffer[56];

	if	(scsi_cmd(fd, (cdb_t *)&cdb, 6, SUC_READ, Inqbuffer, 
			sizeof(Inqbuffer), 0))
		return("\377");
	return(Inqbuffer);
	}

#define ADD_SENSECODE 12
#define ADD_SC_QUALIFIER 13

int TestForMedium(int fd)
	{
	int	sts, asc, ascq;
	struct	scsi_user_cdb sus;
	static struct scsi_cdb6 cdb =
		{
		CMD_TEST_UNIT_READY, /* command */
		0,	/* reserved */
		0,	/* reserved */
		0,	/* reserved */
		0,	/* reserved */
		0	/* reserved */
		};

again:	sts = scsi_cmd(fd, (cdb_t *)&cdb, 6, SUC_READ, 0, 0, &sus);
	asc = sus.suc_sus.sus_sense[ADD_SENSECODE];
	ascq = sus.suc_sus.sus_sense[ADD_SC_QUALIFIER];
	if	(asc == 0x3a && ascq == 0x0)	/* no medium */
		return(0);
	if	(asc == 0x28 && ascq == 0x0)	/* medium changed */
		goto again;
	if	(asc == 0x4 && ascq == 0x1 )	/* coming ready */
		{
		sleep(2);
		goto again;
		}
	return(1);
	}

/* Check a drive to see if it is a CD-ROM */
static int CheckDrive(char *drive, struct stat *stbuf)
{
	int is_cd = 0, cdfd;
	char *p;

	/* If it doesn't exist, return -1 */
	if ( stat(drive, stbuf) < 0 ) {
		return(-1);
	}

	/* If it does exist, verify that it's an available CD-ROM */
	cdfd = open(drive, (O_RDONLY|O_EXCL|O_NONBLOCK), 0);
	if ( cdfd >= 0 ) {
		p = Inquiry(cdfd);
		if (*p == TYPE_ROM)
			is_cd = 1;
		close(cdfd);
	}
	return(is_cd);
}

/* Add a CD-ROM drive to our list of valid drives */
static void AddDrive(char *drive, struct stat *stbuf)
{
	int i;

	if ( SDL_numcds < MAX_DRIVES ) {
		/* Check to make sure it's not already in our list.
	 	   This can happen when we see a drive via symbolic link.
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
	/* checklist: /dev/rsr?c */
	static char *checklist[] = {
	"?0 rsr?", NULL
	};
	char *SDLcdrom;
	int i, j, exists;
	char drive[32];
	struct stat stbuf;

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
	for ( i=0; checklist[i]; ++i ) {
		if ( checklist[i][0] == '?' ) {
			char *insert;
			exists = 1;
			for ( j=checklist[i][1]; exists; ++j ) {
				SDL_snprintf(drive, SDL_arraysize(drive), "/dev/%sc", &checklist[i][3]);
				insert = SDL_strchr(drive, '?');
				if ( insert != NULL ) {
					*insert = j;
				}
				switch (CheckDrive(drive, &stbuf)) {
					/* Drive exists and is a CD-ROM */
					case 1:
						AddDrive(drive, &stbuf);
						break;
					/* Drive exists, but isn't a CD-ROM */
					case 0:
						break;
					/* Drive doesn't exist */
					case -1:
						exists = 0;
						break;
				}
			}
		} else {
			SDL_snprintf(drive, SDL_arraysize(drive), "/dev/%s", checklist[i]);
			if ( CheckDrive(drive, &stbuf) > 0 ) {
				AddDrive(drive, &stbuf);
			}
		}
	}
	return(0);
}

static const char *SDL_SYS_CDName(int drive)
{
	return(SDL_cdlist[drive]);
}

static int SDL_SYS_CDOpen(int drive)
{
	return(open(SDL_cdlist[drive], O_RDONLY | O_NONBLOCK | O_EXCL, 0));
}

static int SDL_SYS_CDGetTOC(SDL_CD *cdrom)
	{
	u_char cdb[10], buf[4], *p, *toc;
	struct scsi_user_cdb sus;
	int i, sts, first_track, last_track, ntracks, toc_size;

	bzero(cdb, sizeof (cdb));
	cdb[0] = 0x43;		/* Read TOC */
	cdb[1] = 0x2;		/* MSF */
	cdb[8] = 4;		/* size TOC header */
	sts = scsi_cmd(cdrom->id, (cdb_t *)cdb, 10, SUC_READ, buf, 4, &sus);
	if	(sts < 0)
		return(-1);
	first_track = buf[2];
	last_track = buf[3];
	ntracks = last_track - first_track + 1;
	cdrom->numtracks = ntracks;
	toc_size = 4 + (ntracks + 1) * 8;
	toc = (u_char *)SDL_malloc(toc_size);
	if	(toc == NULL)
		return(-1);
	bzero(cdb, sizeof (cdb));
	cdb[0] = 0x43;
	cdb[1] = 0x2;
	cdb[6] = first_track;
	cdb[7] = toc_size >> 8;
	cdb[8] = toc_size & 0xff;
	sts = scsi_cmd(cdrom->id, (cdb_t *)cdb, 10, SUC_READ, toc, toc_size, 
			&sus);
	if	(sts < 0)
		{
		SDL_free(toc);
		return(-1);
		}

	for	(i = 0, p = toc+4; i <= ntracks; i++, p+= 8)
		{
		if	(i == ntracks)
			cdrom->track[i].id = 0xAA;	/* Leadout */
		else
			cdrom->track[i].id = first_track + i;
		if	(p[1] & 0x20)
			cdrom->track[i].type = SDL_DATA_TRACK;
		else
			cdrom->track[i].type = SDL_AUDIO_TRACK;
		cdrom->track[i].offset = msf_to_frame(p[5], p[6], p[7]);
		cdrom->track[i].length = 0;
		if	(i > 0)
			cdrom->track[i-1].length = cdrom->track[i].offset -
						   cdrom->track[i-1].offset;
		}
	SDL_free(toc);
	return(0);
	}

/* Get CD-ROM status */
static CDstatus SDL_SYS_CDStatus(SDL_CD *cdrom, int *position)
	{
	CDstatus status;
	u_char	cdb[10], buf[16];
	int	sts;
	struct	scsi_user_cdb sus;

	bzero(cdb, sizeof (cdb));
	cdb[0] = 0x42;		/* read subq */
	cdb[1] = 0x2;		/* MSF */
	cdb[2] = 0x40;		/* q channel */
	cdb[3] = 1;		/* current pos */
	cdb[7] = sizeof (buf) >> 8;
	cdb[8] = sizeof (buf) & 0xff;
	sts = scsi_cmd(cdrom->id, (cdb_t *)cdb, 10, SUC_READ, buf, sizeof (buf),
			&sus);
	if	(sts < 0)
		return(-1);
	if	(sts)
		{
		if	(TestForMedium(cdrom->id) == 0)
			status = CD_TRAYEMPTY;
		else
			status = CD_ERROR;
		}
	else
		{
		switch	(buf[1])
			{
			case	0x11:
				status = CD_PLAYING;
				break;
			case	0x12:
				status = CD_PAUSED;
				break;
			case	0x13:
			case	0x14:
			case	0x15:
				status = CD_STOPPED;
				break;
			default:
				status = CD_ERROR;
				break;
			}
		}
	if	(position)
		{
		if	( status == CD_PLAYING || (status == CD_PAUSED) )
			*position = msf_to_frame(buf[9], buf[10], buf[11]);
		else
			*position = 0;
		}
	return(status);
	}

/* Start play */
static int SDL_SYS_CDPlay(SDL_CD *cdrom, int start, int length)
	{
	u_char	cdb[10];
	int	sts, minute, second, frame, eminute, esecond, eframe;
	struct	scsi_user_cdb sus;

	bzero(cdb, sizeof(cdb));
	cdb[0] = 0x47;		/* Play */
	frame_to_msf(start, &minute, &second, &frame);
	frame_to_msf(start + length, &eminute, &esecond, &eframe);
	cdb[3] = minute;
	cdb[4] = second;
	cdb[5] = frame;
	cdb[6] = eminute;
	cdb[7] = esecond;
	cdb[8] = eframe;
	sts = scsi_cmd(cdrom->id, (cdb_t *)cdb, 10, SUC_READ, 0, 0, &sus);
	return(sts);
	}

static	int
pauseresume(SDL_CD *cdrom, int flag)
	{
	u_char	cdb[10];
	struct	scsi_user_cdb sus;

	bzero(cdb, sizeof (cdb));
	cdb[0] = 0x4b;
	cdb[8] = flag & 0x1;
	return(scsi_cmd(cdrom->id, (cdb_t *)cdb, 10, SUC_READ, 0, 0, &sus));
	}

/* Pause play */
static int SDL_SYS_CDPause(SDL_CD *cdrom)
{
	return(pauseresume(cdrom, 0));
}

/* Resume play */
static int SDL_SYS_CDResume(SDL_CD *cdrom)
{
	return(pauseresume(cdrom, 1));
}

/* Stop play */
static int SDL_SYS_CDStop(SDL_CD *cdrom)
{
	u_char cdb[6];
	struct	scsi_user_cdb sus;

	bzero(cdb, sizeof (cdb));
	cdb[0] = 0x1b;		/* stop */
	cdb[1] = 1;		/* immediate */
	return(scsi_cmd(cdrom->id, (cdb_t *)cdb, 6, SUC_READ, 0, 0, &sus));
}

/* Eject the CD-ROM */
static int SDL_SYS_CDEject(SDL_CD *cdrom)
{
	u_char cdb[6];
	struct	scsi_user_cdb sus;

	bzero(cdb, sizeof (cdb));
	cdb[0] = 0x1b;		/* stop */
	cdb[1] = 1;		/* immediate */
	cdb[4] = 2;		/* eject */
	return(scsi_cmd(cdrom->id, (cdb_t *)cdb, 6, SUC_READ, 0, 0, &sus));
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
		}
	SDL_numcds = 0;
}

#endif /* SDL_CDROM_BSDI */
