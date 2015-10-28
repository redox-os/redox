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

#ifdef SDL_CDROM_FREEBSD

/* Functions for system-level CD-ROM audio control */

#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <errno.h>
#include <unistd.h>
#include <sys/cdio.h>

#include "SDL_cdrom.h"
#include "../SDL_syscdrom.h"


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

/* Some ioctl() errno values which occur when the tray is empty */
#define ERRNO_TRAYEMPTY(errno)	\
	((errno == EIO) || (errno == ENOENT) || (errno == EINVAL))

/* Check a drive to see if it is a CD-ROM */
static int CheckDrive(char *drive, struct stat *stbuf)
{
	int is_cd, cdfd;
	struct ioc_read_subchannel info;

	/* If it doesn't exist, return -1 */
	if ( stat(drive, stbuf) < 0 ) {
		return(-1);
	}

	/* If it does exist, verify that it's an available CD-ROM */
	is_cd = 0;
	if ( S_ISCHR(stbuf->st_mode) || S_ISBLK(stbuf->st_mode) ) {
		cdfd = open(drive, (O_RDONLY|O_EXCL|O_NONBLOCK), 0);
		if ( cdfd >= 0 ) {
			info.address_format = CD_MSF_FORMAT;
			info.data_format = CD_CURRENT_POSITION;
			info.data_len = 0;
			info.data = NULL;
			/* Under Linux, EIO occurs when a disk is not present.
			   This isn't 100% reliable, so we use the USE_MNTENT
			   code above instead.
			 */
			if ( (ioctl(cdfd, CDIOCREADSUBCHANNEL, &info) == 0) ||
						ERRNO_TRAYEMPTY(errno) ) {
				is_cd = 1;
			}
			close(cdfd);
		}
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
	/* checklist: /dev/cdrom,/dev/cd?c /dev/acd?c
			/dev/matcd?c /dev/mcd?c /dev/scd?c */
	static char *checklist[] = {
	"cdrom", "?0 cd?", "?0 acd?", "?0 matcd?", "?0 mcd?", "?0 scd?",NULL
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

/* General ioctl() CD-ROM command function */
static int SDL_SYS_CDioctl(int id, int command, void *arg)
{
	int retval;

	retval = ioctl(id, command, arg);
	if ( retval < 0 ) {
		SDL_SetError("ioctl() error: %s", strerror(errno));
	}
	return(retval);
}

static const char *SDL_SYS_CDName(int drive)
{
	return(SDL_cdlist[drive]);
}

static int SDL_SYS_CDOpen(int drive)
{
	return(open(SDL_cdlist[drive], (O_RDONLY|O_EXCL|O_NONBLOCK), 0));
}

static int SDL_SYS_CDGetTOC(SDL_CD *cdrom)
{
	struct ioc_toc_header toc;
	int i, okay;
	struct ioc_read_toc_entry entry;
	struct cd_toc_entry data;

	okay = 0;
	if ( SDL_SYS_CDioctl(cdrom->id, CDIOREADTOCHEADER, &toc) == 0 ) {
		cdrom->numtracks = toc.ending_track-toc.starting_track+1;
		if ( cdrom->numtracks > SDL_MAX_TRACKS ) {
			cdrom->numtracks = SDL_MAX_TRACKS;
		}
		/* Read all the track TOC entries */
		for ( i=0; i<=cdrom->numtracks; ++i ) {
			if ( i == cdrom->numtracks ) {
				cdrom->track[i].id = 0xAA; /* CDROM_LEADOUT */
			} else {
				cdrom->track[i].id = toc.starting_track+i;
			}
			entry.starting_track = cdrom->track[i].id;
			entry.address_format = CD_MSF_FORMAT;
			entry.data_len = sizeof(data);
			entry.data = &data;
			if ( SDL_SYS_CDioctl(cdrom->id, CDIOREADTOCENTRYS,
								&entry) < 0 ) {
				break;
			} else {
				cdrom->track[i].type = data.control;
				cdrom->track[i].offset = MSF_TO_FRAMES(
						data.addr.msf.minute,
						data.addr.msf.second,
						data.addr.msf.frame);
				cdrom->track[i].length = 0;
				if ( i > 0 ) {
					cdrom->track[i-1].length =
						cdrom->track[i].offset-
						cdrom->track[i-1].offset;
				}
			}
		}
		if ( i == (cdrom->numtracks+1) ) {
			okay = 1;
		}
	}
	return(okay ? 0 : -1);
}

/* Get CD-ROM status */
static CDstatus SDL_SYS_CDStatus(SDL_CD *cdrom, int *position)
{
	CDstatus status;
	struct ioc_toc_header toc;
	struct ioc_read_subchannel info;
	struct cd_sub_channel_info data;

	info.address_format = CD_MSF_FORMAT;
	info.data_format = CD_CURRENT_POSITION;
	info.track = 0;
	info.data_len = sizeof(data);
	info.data = &data;
	if ( ioctl(cdrom->id, CDIOCREADSUBCHANNEL, &info) < 0 ) {
		if ( ERRNO_TRAYEMPTY(errno) ) {
			status = CD_TRAYEMPTY;
		} else {
			status = CD_ERROR;
		}
	} else {
		switch (data.header.audio_status) {
			case CD_AS_AUDIO_INVALID:
			case CD_AS_NO_STATUS:
				/* Try to determine if there's a CD available */
				if (ioctl(cdrom->id,CDIOREADTOCHEADER,&toc)==0)
					status = CD_STOPPED;
				else
					status = CD_TRAYEMPTY;
				break;
			case CD_AS_PLAY_COMPLETED:
				status = CD_STOPPED;
				break;
			case CD_AS_PLAY_IN_PROGRESS:
				status = CD_PLAYING;
				break;
			case CD_AS_PLAY_PAUSED:
				status = CD_PAUSED;
				break;
			default:
				status = CD_ERROR;
				break;
		}
	}
	if ( position ) {
		if ( status == CD_PLAYING || (status == CD_PAUSED) ) {
			*position = MSF_TO_FRAMES(
					data.what.position.absaddr.msf.minute,
					data.what.position.absaddr.msf.second,
					data.what.position.absaddr.msf.frame);
		} else {
			*position = 0;
		}
	}
	return(status);
}

/* Start play */
static int SDL_SYS_CDPlay(SDL_CD *cdrom, int start, int length)
{
	struct ioc_play_msf playtime;

	FRAMES_TO_MSF(start,
		&playtime.start_m, &playtime.start_s, &playtime.start_f);
	FRAMES_TO_MSF(start+length,
		&playtime.end_m, &playtime.end_s, &playtime.end_f);
#ifdef DEBUG_CDROM
  fprintf(stderr, "Trying to play from %d:%d:%d to %d:%d:%d\n",
	playtime.cdmsf_min0, playtime.cdmsf_sec0, playtime.cdmsf_frame0,
	playtime.cdmsf_min1, playtime.cdmsf_sec1, playtime.cdmsf_frame1);
#endif
	ioctl(cdrom->id, CDIOCSTART, 0);
	return(SDL_SYS_CDioctl(cdrom->id, CDIOCPLAYMSF, &playtime));
}

/* Pause play */
static int SDL_SYS_CDPause(SDL_CD *cdrom)
{
	return(SDL_SYS_CDioctl(cdrom->id, CDIOCPAUSE, 0));
}

/* Resume play */
static int SDL_SYS_CDResume(SDL_CD *cdrom)
{
	return(SDL_SYS_CDioctl(cdrom->id, CDIOCRESUME, 0));
}

/* Stop play */
static int SDL_SYS_CDStop(SDL_CD *cdrom)
{
	return(SDL_SYS_CDioctl(cdrom->id, CDIOCSTOP, 0));
}

/* Eject the CD-ROM */
static int SDL_SYS_CDEject(SDL_CD *cdrom)
{
	return(SDL_SYS_CDioctl(cdrom->id, CDIOCEJECT, 0));
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

#endif /* SDL_CDROM_FREEBSD */
