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

#ifdef SDL_CDROM_LINUX

/* Functions for system-level CD-ROM audio control */

#include <string.h>	/* For strerror() */
#include <sys/types.h>
#include <sys/stat.h>
#include <sys/ioctl.h>
#include <fcntl.h>
#include <errno.h>
#include <unistd.h>
#ifdef __LINUX__
#ifdef HAVE_LINUX_VERSION_H
/* linux 2.6.9 workaround */
#include <linux/version.h>
#if LINUX_VERSION_CODE == KERNEL_VERSION(2,6,9)
#include <asm/types.h>
#define __le64 __u64
#define __le32 __u32
#define __le16 __u16
#define __be64 __u64
#define __be32 __u32
#define __be16 __u16
#endif /* linux 2.6.9 workaround */
#endif /* HAVE_LINUX_VERSION_H */
#include <linux/cdrom.h>
#endif
#ifdef __SVR4
#include <sys/cdio.h>
#endif

/* Define this to use the alternative getmntent() code */
#ifndef __SVR4
#define USE_MNTENT
#endif

#ifdef USE_MNTENT
#if defined(__USLC__)
#include <sys/mntent.h>
#else
#include <mntent.h>
#endif

#ifndef _PATH_MNTTAB
#ifdef MNTTAB
#define _PATH_MNTTAB	MNTTAB
#else
#define _PATH_MNTTAB	"/etc/fstab"
#endif
#endif /* !_PATH_MNTTAB */

#ifndef _PATH_MOUNTED
#define _PATH_MOUNTED	"/etc/mtab"
#endif /* !_PATH_MOUNTED */

#ifndef MNTTYPE_CDROM
#define MNTTYPE_CDROM	"iso9660"
#endif
#ifndef MNTTYPE_SUPER
#define MNTTYPE_SUPER	"supermount"
#endif
#endif /* USE_MNTENT */

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
#ifndef ENOMEDIUM
#define ENOMEDIUM ENOENT
#endif
#define ERRNO_TRAYEMPTY(errno)	\
	((errno == EIO)    || (errno == ENOENT) || \
	 (errno == EINVAL) || (errno == ENOMEDIUM))

/* Check a drive to see if it is a CD-ROM */
static int CheckDrive(char *drive, char *mnttype, struct stat *stbuf)
{
	int is_cd, cdfd;
	struct cdrom_subchnl info;

	/* If it doesn't exist, return -1 */
	if ( stat(drive, stbuf) < 0 ) {
		return(-1);
	}

	/* If it does exist, verify that it's an available CD-ROM */
	is_cd = 0;
	if ( S_ISCHR(stbuf->st_mode) || S_ISBLK(stbuf->st_mode) ) {
		cdfd = open(drive, (O_RDONLY|O_NONBLOCK), 0);
		if ( cdfd >= 0 ) {
			info.cdsc_format = CDROM_MSF;
			/* Under Linux, EIO occurs when a disk is not present.
			 */
			if ( (ioctl(cdfd, CDROMSUBCHNL, &info) == 0) ||
						ERRNO_TRAYEMPTY(errno) ) {
				is_cd = 1;
			}
			close(cdfd);
		}
#ifdef USE_MNTENT
		/* Even if we can't read it, it might be mounted */
		else if ( mnttype && (SDL_strcmp(mnttype, MNTTYPE_CDROM) == 0) ) {
			is_cd = 1;
		}
#endif
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

#ifdef USE_MNTENT
static void CheckMounts(const char *mtab)
{
	FILE *mntfp;
	struct mntent *mntent;
	struct stat stbuf;

	mntfp = setmntent(mtab, "r");
	if ( mntfp != NULL ) {
		char *tmp;
		char *mnt_type;
		size_t mnt_type_len;
		char *mnt_dev;
		size_t mnt_dev_len;

		while ( (mntent=getmntent(mntfp)) != NULL ) {
			mnt_type_len = SDL_strlen(mntent->mnt_type) + 1;
			mnt_type = SDL_stack_alloc(char, mnt_type_len);
			if (mnt_type == NULL)
				continue;  /* maybe you'll get lucky next time. */

			mnt_dev_len = SDL_strlen(mntent->mnt_fsname) + 1;
			mnt_dev = SDL_stack_alloc(char, mnt_dev_len);
			if (mnt_dev == NULL) {
				SDL_stack_free(mnt_type);
				continue;
			}

			SDL_strlcpy(mnt_type, mntent->mnt_type, mnt_type_len);
			SDL_strlcpy(mnt_dev, mntent->mnt_fsname, mnt_dev_len);

			/* Handle "supermount" filesystem mounts */
			if ( SDL_strcmp(mnt_type, MNTTYPE_SUPER) == 0 ) {
				tmp = SDL_strstr(mntent->mnt_opts, "fs=");
				if ( tmp ) {
					SDL_stack_free(mnt_type);
					mnt_type = SDL_strdup(tmp + SDL_strlen("fs="));
					if ( mnt_type ) {
						tmp = SDL_strchr(mnt_type, ',');
						if ( tmp ) {
							*tmp = '\0';
						}
					}
				}
				tmp = SDL_strstr(mntent->mnt_opts, "dev=");
				if ( tmp ) {
					SDL_stack_free(mnt_dev);
					mnt_dev = SDL_strdup(tmp + SDL_strlen("dev="));
					if ( mnt_dev ) {
						tmp = SDL_strchr(mnt_dev, ',');
						if ( tmp ) {
							*tmp = '\0';
						}
					}
				}
			}
			if ( SDL_strcmp(mnt_type, MNTTYPE_CDROM) == 0 ) {
#ifdef DEBUG_CDROM
  fprintf(stderr, "Checking mount path from %s: %s mounted on %s of %s\n",
	mtab, mnt_dev, mntent->mnt_dir, mnt_type);
#endif
				if (CheckDrive(mnt_dev, mnt_type, &stbuf) > 0) {
					AddDrive(mnt_dev, &stbuf);
				}
			}
			SDL_stack_free(mnt_dev);
			SDL_stack_free(mnt_type);
		}
		endmntent(mntfp);
	}
}
#endif /* USE_MNTENT */

int  SDL_SYS_CDInit(void)
{
	/* checklist: /dev/cdrom, /dev/hd?, /dev/scd? /dev/sr? */
	static char *checklist[] = {
		"cdrom", "?a hd?", "?0 scd?", "?0 sr?", NULL
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
#ifdef DEBUG_CDROM
  fprintf(stderr, "Checking CD-ROM drive from SDL_CDROM: %s\n", SDLcdrom);
#endif
				if ( CheckDrive(SDLcdrom, NULL, &stbuf) > 0 ) {
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

#ifdef USE_MNTENT
	/* Check /dev/cdrom first :-) */
	if (CheckDrive("/dev/cdrom", NULL, &stbuf) > 0) {
		AddDrive("/dev/cdrom", &stbuf);
	}

	/* Now check the currently mounted CD drives */
	CheckMounts(_PATH_MOUNTED);

	/* Finally check possible mountable drives in /etc/fstab */
	CheckMounts(_PATH_MNTTAB);

	/* If we found our drives, there's nothing left to do */
	if ( SDL_numcds > 0 ) {
		return(0);
	}
#endif /* USE_MNTENT */

	/* Scan the system for CD-ROM drives.
	   Not always 100% reliable, so use the USE_MNTENT code above first.
	 */
	for ( i=0; checklist[i]; ++i ) {
		if ( checklist[i][0] == '?' ) {
			char *insert;
			exists = 1;
			for ( j=checklist[i][1]; exists; ++j ) {
				SDL_snprintf(drive, SDL_arraysize(drive), "/dev/%s", &checklist[i][3]);
				insert = SDL_strchr(drive, '?');
				if ( insert != NULL ) {
					*insert = j;
				}
#ifdef DEBUG_CDROM
  fprintf(stderr, "Checking possible CD-ROM drive: %s\n", drive);
#endif
				switch (CheckDrive(drive, NULL, &stbuf)) {
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
#ifdef DEBUG_CDROM
  fprintf(stderr, "Checking possible CD-ROM drive: %s\n", drive);
#endif
			if ( CheckDrive(drive, NULL, &stbuf) > 0 ) {
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
	return(open(SDL_cdlist[drive], (O_RDONLY|O_NONBLOCK), 0));
}

static int SDL_SYS_CDGetTOC(SDL_CD *cdrom)
{
	struct cdrom_tochdr toc;
	int i, okay;
	struct cdrom_tocentry entry;

	okay = 0;
	if ( SDL_SYS_CDioctl(cdrom->id, CDROMREADTOCHDR, &toc) == 0 ) {
		cdrom->numtracks = toc.cdth_trk1-toc.cdth_trk0+1;
		if ( cdrom->numtracks > SDL_MAX_TRACKS ) {
			cdrom->numtracks = SDL_MAX_TRACKS;
		}
		/* Read all the track TOC entries */
		for ( i=0; i<=cdrom->numtracks; ++i ) {
			if ( i == cdrom->numtracks ) {
				cdrom->track[i].id = CDROM_LEADOUT;
			} else {
				cdrom->track[i].id = toc.cdth_trk0+i;
			}
			entry.cdte_track = cdrom->track[i].id;
			entry.cdte_format = CDROM_MSF;
			if ( SDL_SYS_CDioctl(cdrom->id, CDROMREADTOCENTRY,
								&entry) < 0 ) {
				break;
			} else {
				if ( entry.cdte_ctrl & CDROM_DATA_TRACK ) {
					cdrom->track[i].type = SDL_DATA_TRACK;
				} else {
					cdrom->track[i].type = SDL_AUDIO_TRACK;
				}
				cdrom->track[i].offset = MSF_TO_FRAMES(
						entry.cdte_addr.msf.minute,
						entry.cdte_addr.msf.second,
						entry.cdte_addr.msf.frame);
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
	struct cdrom_tochdr toc;
	struct cdrom_subchnl info;

	info.cdsc_format = CDROM_MSF;
	if ( ioctl(cdrom->id, CDROMSUBCHNL, &info) < 0 ) {
		if ( ERRNO_TRAYEMPTY(errno) ) {
			status = CD_TRAYEMPTY;
		} else {
			status = CD_ERROR;
		}
	} else {
		switch (info.cdsc_audiostatus) {
			case CDROM_AUDIO_INVALID:
			case CDROM_AUDIO_NO_STATUS:
				/* Try to determine if there's a CD available */
				if (ioctl(cdrom->id, CDROMREADTOCHDR, &toc)==0)
					status = CD_STOPPED;
				else
					status = CD_TRAYEMPTY;
				break;
			case CDROM_AUDIO_COMPLETED:
				status = CD_STOPPED;
				break;
			case CDROM_AUDIO_PLAY:
				status = CD_PLAYING;
				break;
			case CDROM_AUDIO_PAUSED:
				/* Workaround buggy CD-ROM drive */
				if ( info.cdsc_trk == CDROM_LEADOUT ) {
					status = CD_STOPPED;
				} else {
					status = CD_PAUSED;
				}
				break;
			default:
				status = CD_ERROR;
				break;
		}
	}
	if ( position ) {
		if ( status == CD_PLAYING || (status == CD_PAUSED) ) {
			*position = MSF_TO_FRAMES(
					info.cdsc_absaddr.msf.minute,
					info.cdsc_absaddr.msf.second,
					info.cdsc_absaddr.msf.frame);
		} else {
			*position = 0;
		}
	}
	return(status);
}

/* Start play */
static int SDL_SYS_CDPlay(SDL_CD *cdrom, int start, int length)
{
	struct cdrom_msf playtime;

	FRAMES_TO_MSF(start,
	   &playtime.cdmsf_min0, &playtime.cdmsf_sec0, &playtime.cdmsf_frame0);
	FRAMES_TO_MSF(start+length,
	   &playtime.cdmsf_min1, &playtime.cdmsf_sec1, &playtime.cdmsf_frame1);
#ifdef DEBUG_CDROM
  fprintf(stderr, "Trying to play from %d:%d:%d to %d:%d:%d\n",
	playtime.cdmsf_min0, playtime.cdmsf_sec0, playtime.cdmsf_frame0,
	playtime.cdmsf_min1, playtime.cdmsf_sec1, playtime.cdmsf_frame1);
#endif
	return(SDL_SYS_CDioctl(cdrom->id, CDROMPLAYMSF, &playtime));
}

/* Pause play */
static int SDL_SYS_CDPause(SDL_CD *cdrom)
{
	return(SDL_SYS_CDioctl(cdrom->id, CDROMPAUSE, 0));
}

/* Resume play */
static int SDL_SYS_CDResume(SDL_CD *cdrom)
{
	return(SDL_SYS_CDioctl(cdrom->id, CDROMRESUME, 0));
}

/* Stop play */
static int SDL_SYS_CDStop(SDL_CD *cdrom)
{
	return(SDL_SYS_CDioctl(cdrom->id, CDROMSTOP, 0));
}

/* Eject the CD-ROM */
static int SDL_SYS_CDEject(SDL_CD *cdrom)
{
	return(SDL_SYS_CDioctl(cdrom->id, CDROMEJECT, 0));
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

#endif /* SDL_CDROM_LINUX */
