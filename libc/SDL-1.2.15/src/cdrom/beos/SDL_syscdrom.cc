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

#ifdef SDL_CDROM_BEOS

/* Functions for system-level CD-ROM audio control on BeOS
   (not completely implemented yet)
 */

#include <sys/types.h>
#include <sys/stat.h>
#include <unistd.h>

#include <scsi.h>
#include <Directory.h>
#include <Entry.h>
#include <Path.h>

#include "SDL_cdrom.h"
extern "C" {
#include "../SDL_syscdrom.h"
}

/* Constants to help us get at the SCSI table-of-contents info */
#define CD_NUMTRACKS(toc)	toc.toc_data[3]
#define CD_TRACK(toc, track)	(&toc.toc_data[6+(track)*8])
#define CD_TRACK_N(toc, track)	CD_TRACK(toc, track)[0]
#define CD_TRACK_M(toc, track)	CD_TRACK(toc, track)[3]
#define CD_TRACK_S(toc, track)	CD_TRACK(toc, track)[4]
#define CD_TRACK_F(toc, track)	CD_TRACK(toc, track)[5]

/* Constants to help us get at the SCSI position info */
#define POS_TRACK(pos)	pos.position[6]
#define POS_ABS_M(pos)	pos.position[9]
#define POS_ABS_S(pos)	pos.position[10]
#define POS_ABS_F(pos)	pos.position[11]
#define POS_REL_M(pos)	pos.position[13]
#define POS_REL_S(pos)	pos.position[14]
#define POS_REL_F(pos)	pos.position[15]

/* The maximum number of CD-ROM drives we'll detect */
#define MAX_DRIVES	16	

/* A list of available CD-ROM drives */
static char *SDL_cdlist[MAX_DRIVES];

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
int try_dir(const char *directory);


/* Check a drive to see if it is a CD-ROM */
static int CheckDrive(char *drive)
{
	struct stat stbuf;
	int is_cd, cdfd;
	device_geometry info;

	/* If it doesn't exist, return -1 */
	if ( stat(drive, &stbuf) < 0 ) {
		return(-1);
	}

	/* If it does exist, verify that it's an available CD-ROM */
	is_cd = 0;
	cdfd = open(drive, 0);
	if ( cdfd >= 0 ) {
		if ( ioctl(cdfd, B_GET_GEOMETRY, &info) == B_NO_ERROR ) {
			if ( info.device_type == B_CD ) {
				is_cd = 1;
			}
		}
		close(cdfd);
	} else {
		/* This can happen when the drive is open .. (?) */;
		is_cd = 1;
	}
	return(is_cd);
}

/* Add a CD-ROM drive to our list of valid drives */
static void AddDrive(char *drive)
{
	int i;
	size_t len;

	if ( SDL_numcds < MAX_DRIVES ) {
		/* Add this drive to our list */
		i = SDL_numcds;
		len = SDL_strlen(drive)+1;
		SDL_cdlist[i] = (char *)SDL_malloc(len);
		if ( SDL_cdlist[i] == NULL ) {
			SDL_OutOfMemory();
			return;
		}
		SDL_strlcpy(SDL_cdlist[i], drive, len);
		++SDL_numcds;
#ifdef CDROM_DEBUG
  fprintf(stderr, "Added CD-ROM drive: %s\n", drive);
#endif
	}
}

/* IDE bus scanning magic */
enum {
	IDE_GET_DEVICES_INFO = B_DEVICE_OP_CODES_END + 50,
};
struct ide_ctrl_info {
	bool	ide_0_present;
	bool	ide_0_master_present;
	bool	ide_0_slave_present;
	int	ide_0_master_type;
	int	ide_0_slave_type;
	bool	ide_1_present;
	bool	ide_1_master_present;
	bool	ide_1_slave_present;
	int	ide_1_master_type;
	int	ide_1_slave_type;
};

int  SDL_SYS_CDInit(void)
{
	char *SDLcdrom;

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
				if ( CheckDrive(SDLcdrom) > 0 ) {
					AddDrive(SDLcdrom);
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
	try_dir("/dev/disk");
	return 0;
}


int try_dir(const char *directory)
{ 
	BDirectory dir; 
	dir.SetTo(directory); 
	if(dir.InitCheck() != B_NO_ERROR) { 
		return false; 
	} 
	dir.Rewind(); 
	BEntry entry; 
	while(dir.GetNextEntry(&entry) >= 0) { 
		BPath path; 
		const char *name; 
		entry_ref e; 
		
		if(entry.GetPath(&path) != B_NO_ERROR) 
			continue; 
		name = path.Path(); 
		
		if(entry.GetRef(&e) != B_NO_ERROR) 
			continue; 

		if(entry.IsDirectory()) { 
			if(SDL_strcmp(e.name, "floppy") == 0) 
				continue; /* ignore floppy (it is not silent)  */
			int devfd = try_dir(name);
			if(devfd >= 0)
				return devfd;
		} 
		else { 
			int devfd; 
			device_geometry g; 

			if(SDL_strcmp(e.name, "raw") != 0) 
				continue; /* ignore partitions */

			devfd = open(name, O_RDONLY); 
			if(devfd < 0) 
				continue; 

			if(ioctl(devfd, B_GET_GEOMETRY, &g, sizeof(g)) >= 0) {
				if(g.device_type == B_CD)
				{
				AddDrive(strdup(name));
				}
			}
			close(devfd);
		} 
	}
	return B_ERROR;
}


/* General ioctl() CD-ROM command function */
static int SDL_SYS_CDioctl(int index, int command, void *arg)
{
	int okay;
	int fd;

	okay = 0;
	fd = open(SDL_cdlist[index], 0);
	if ( fd >= 0 ) {
		if ( ioctl(fd, command, arg) == B_NO_ERROR ) {
			okay = 1;
		}
		close(fd);
	}
	return(okay ? 0 : -1);
}

static const char *SDL_SYS_CDName(int drive)
{
	return(SDL_cdlist[drive]);
} 

static int SDL_SYS_CDOpen(int drive)
{
	return(drive);
}

static int SDL_SYS_CDGetTOC(SDL_CD *cdrom)
{
	int i;
	scsi_toc toc;

	if ( SDL_SYS_CDioctl(cdrom->id, B_SCSI_GET_TOC, &toc) == 0 ) {
		cdrom->numtracks = CD_NUMTRACKS(toc);
		if ( cdrom->numtracks > SDL_MAX_TRACKS ) {
			cdrom->numtracks = SDL_MAX_TRACKS;
		}
		for ( i=0; i<=cdrom->numtracks; ++i ) {
			cdrom->track[i].id = CD_TRACK_N(toc, i);
			/* FIXME:  How do we tell on BeOS? */
			cdrom->track[i].type = SDL_AUDIO_TRACK;
			cdrom->track[i].offset = MSF_TO_FRAMES(
							CD_TRACK_M(toc, i),
							CD_TRACK_S(toc, i),
							CD_TRACK_F(toc, i));
			cdrom->track[i].length = 0;
			if ( i > 0 ) {
				cdrom->track[i-1].length =
						cdrom->track[i].offset-
						cdrom->track[i-1].offset;
			}
		}
		return(0);
	} else {
		return(-1);
	}
}

/* Get CD-ROM status */
static CDstatus SDL_SYS_CDStatus(SDL_CD *cdrom, int *position)
{
	CDstatus status;
	int fd;
	int cur_frame;
	scsi_position pos;

	fd = open(SDL_cdlist[cdrom->id], 0);
	cur_frame = 0;
	if ( fd >= 0 ) {
		if ( ioctl(fd, B_SCSI_GET_POSITION, &pos) == B_NO_ERROR ) {
			cur_frame = MSF_TO_FRAMES(
				POS_ABS_M(pos), POS_ABS_S(pos), POS_ABS_F(pos));
		}
		if ( ! pos.position[1] || (pos.position[1] >= 0x13) ||
			((pos.position[1] == 0x12) && (!pos.position[6])) ) {
			status = CD_STOPPED;
		} else
		if ( pos.position[1] == 0x11 ) {
			status = CD_PLAYING;
		} else {
			status = CD_PAUSED;
		}
		close(fd);
	} else {
		status = CD_TRAYEMPTY;
	}
	if ( position ) {
		*position = cur_frame;
	}
	return(status);
}

/* Start play */
static int SDL_SYS_CDPlay(SDL_CD *cdrom, int start, int length)
{
	int okay;
	int fd;
	scsi_play_position pos;

	okay = 0;
	fd = open(SDL_cdlist[cdrom->id], 0);
	if ( fd >= 0 ) {
		FRAMES_TO_MSF(start, &pos.start_m, &pos.start_s, &pos.start_f);
		FRAMES_TO_MSF(start+length, &pos.end_m, &pos.end_s, &pos.end_f);
		if ( ioctl(fd, B_SCSI_PLAY_POSITION, &pos) == B_NO_ERROR ) {
			okay = 1;
		}
		close(fd);
	}
	return(okay ? 0 : -1);
}

/* Pause play */
static int SDL_SYS_CDPause(SDL_CD *cdrom)
{
	return(SDL_SYS_CDioctl(cdrom->id, B_SCSI_PAUSE_AUDIO, 0));
}

/* Resume play */
static int SDL_SYS_CDResume(SDL_CD *cdrom)
{
	return(SDL_SYS_CDioctl(cdrom->id, B_SCSI_RESUME_AUDIO, 0));
}

/* Stop play */
static int SDL_SYS_CDStop(SDL_CD *cdrom)
{
	return(SDL_SYS_CDioctl(cdrom->id, B_SCSI_STOP_AUDIO, 0));
}

/* Eject the CD-ROM */
static int SDL_SYS_CDEject(SDL_CD *cdrom)
{
	return(SDL_SYS_CDioctl(cdrom->id, B_SCSI_EJECT, 0));
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

#endif /* SDL_CDROM_BEOS */
