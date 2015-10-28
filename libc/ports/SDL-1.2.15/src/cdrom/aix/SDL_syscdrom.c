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

    Carsten Griwodz
    griff@kom.tu-darmstadt.de

    based on linux/SDL_syscdrom.c by Sam Lantinga
*/
#include "SDL_config.h"

#ifdef SDL_CDROM_AIX

/* Functions for system-level CD-ROM audio control */

/*#define DEBUG_CDROM 1*/

#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <errno.h>
#include <unistd.h>

#include <sys/ioctl.h>
#include <sys/devinfo.h>
#include <sys/mntctl.h>
#include <sys/statfs.h>
#include <sys/vmount.h>
#include <fstab.h>
#include <sys/scdisk.h>

#include "SDL_cdrom.h"
#include "../SDL_syscdrom.h"

/* The maximum number of CD-ROM drives we'll detect */
#define MAX_DRIVES	16	

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
static int         SDL_SYS_CDioctl(int id, int command, void *arg);

/* Check a drive to see if it is a CD-ROM */
static int CheckDrive(char *drive, struct stat *stbuf)
{
    int is_cd;
    int cdfd;
    int ret;
    struct devinfo info;

    /* If it doesn't exist, return -1 */
    if ( stat(drive, stbuf) < 0 ) {
        return -1;
    }

    /* If it does exist, verify that it's an available CD-ROM */
    is_cd = 0;
    if ( S_ISCHR(stbuf->st_mode) || S_ISBLK(stbuf->st_mode) ) {
        cdfd = open(drive, (O_RDONLY|O_EXCL|O_NONBLOCK), 0);
        if ( cdfd >= 0 ) {
            ret = SDL_SYS_CDioctl( cdfd, IOCINFO, &info );
	    if ( ret < 0 ) {
		/* Some kind of error */
		is_cd = 0;
	    } else {
		if ( info.devtype == DD_CDROM ) {
		    is_cd = 1;
		} else {
		    is_cd = 0;
		}
	    }
            close(cdfd);
	}
#ifdef DEBUG_CDROM
	else
	{
            fprintf(stderr, "Could not open drive %s (%s)\n", drive, strerror(errno));
	}
#endif
    }
    return is_cd;
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

static void CheckMounts()
{
    char*          buffer;
    int            bufsz;
    struct vmount* ptr;
    int            ret;

    buffer = (char*)SDL_malloc(10);
    bufsz  = 10;
    if ( buffer==NULL )
    {
        fprintf(stderr, "Could not allocate 10 bytes in aix/SDL_syscdrom.c:CheckMounts\n" );
	exit ( -10 );
    }

    do
    {
	/* mntctrl() returns an array of all mounted filesystems */
        ret = mntctl ( MCTL_QUERY, bufsz, buffer );
        if ( ret == 0 )
        {
				   /* Buffer was too small, realloc.    */
            bufsz = *(int*)buffer; /* Required size is in first word.   */
				   /* (whatever a word is in AIX 4.3.3) */
				   /* int seems to be OK in 32bit mode. */
            SDL_free(buffer);
            buffer = (char*)SDL_malloc(bufsz);
            if ( buffer==NULL )
            {
                fprintf(stderr,
			"Could not allocate %d bytes in aix/SDL_syscdrom.c:CheckMounts\n",
			bufsz );
	        exit ( -10 );
            }
        }
	else if ( ret < 0 )
	{
#ifdef DEBUG_CDROM
            fprintf(stderr, "Error reading vmount structures\n");
#endif
	    return;
	}
    }
    while ( ret == 0 );

#ifdef DEBUG_CDROM
    fprintf ( stderr, "Read %d vmount structures\n",ret );
#endif
    ptr = (struct vmount*)buffer;
    do
    {
            switch(ptr->vmt_gfstype)
            {
            case MNT_CDROM :
                {
		    struct stat stbuf;
		    char*       text;

		    text = (char*)ptr + ptr->vmt_data[VMT_OBJECT].vmt_off;
#ifdef DEBUG_CDROM
  fprintf(stderr, "Checking mount path: %s mounted on %s\n",
	text, (char*)ptr + ptr->vmt_data[VMT_STUB].vmt_off );
#endif
		    if ( CheckDrive( text, &stbuf) > 0)
		    {
		        AddDrive( text, &stbuf);
		    }
                }
                break;
            default :
                break;
            }
            ptr = (struct vmount*)((char*)ptr + ptr->vmt_length);
            ret--;
    }
    while ( ret > 0 );

    free ( buffer );
}

static int CheckNonmounts()
{
#ifdef _THREAD_SAFE
    AFILE_t      fsFile = NULL;
    int          passNo = 0;
    int          ret;
    struct fstab entry;
    struct stat  stbuf;

    ret = setfsent_r( &fsFile, &passNo );
    if ( ret != 0 ) return -1;
    do
    {
        ret = getfsent_r ( &entry, &fsFile, &passNo );
        if ( ret == 0 ) {
            char* l = SDL_strrchr(entry.fs_spec,'/');
            if ( l != NULL ) {
                if ( !SDL_strncmp("cd",++l,2) ) {
#ifdef DEBUG_CDROM
                    fprintf(stderr,
			    "Found unmounted CD ROM drive with device name %s\n",
			    entry.fs_spec);
#endif
		    if ( CheckDrive( entry.fs_spec, &stbuf) > 0)
		    {
		        AddDrive( entry.fs_spec, &stbuf);
		    }
                }
            }
        }
    }
    while ( ret == 0 );
    ret = endfsent_r ( &fsFile );
    if ( ret != 0 ) return -1;
    return 0;
#else
    struct fstab* entry;
    struct stat  stbuf;

    setfsent();
    do
    {
        entry = getfsent();
        if ( entry != NULL ) {
            char* l = SDL_strrchr(entry->fs_spec,'/');
            if ( l != NULL ) {
                if ( !SDL_strncmp("cd",++l,2) ) {
#ifdef DEBUG_CDROM
                    fprintf(stderr,"Found unmounted CD ROM drive with device name %s", entry->fs_spec);
#endif
		    if ( CheckDrive( entry->fs_spec, &stbuf) > 0)
		    {
		        AddDrive( entry->fs_spec, &stbuf);
		    }
                }
            }
        }
    }
    while ( entry != NULL );
    endfsent();
#endif
}

int  SDL_SYS_CDInit(void)
{
	char *SDLcdrom;
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

	CheckMounts();
	CheckNonmounts();

	return 0;
}

/* General ioctl() CD-ROM command function */
static int SDL_SYS_CDioctl(int id, int command, void *arg)
{
    int retval;

    retval = ioctl(id, command, arg);
    if ( retval < 0 ) {
        SDL_SetError("ioctl() error: %s", strerror(errno));
    }
    return retval;
}

static const char *SDL_SYS_CDName(int drive)
{
	return(SDL_cdlist[drive]);
}

static int SDL_SYS_CDOpen(int drive)
{
    int   fd;
    char* lastsl;
    char* cdromname;
    size_t len;

    /*
     * We found /dev/cd? drives and that is in our list. But we can
     * open only the /dev/rcd? versions of those devices for Audio CD.
     */
    len = SDL_strlen(SDL_cdlist[drive])+2;
    cdromname = (char*)SDL_malloc(len);
    SDL_strlcpy(cdromname,SDL_cdlist[drive],len);
    lastsl = SDL_strrchr(cdromname,'/');
    if (lastsl) {
	*lastsl = 0;
	SDL_strlcat(cdromname,"/r",len);
	lastsl = SDL_strrchr(SDL_cdlist[drive],'/');
	if (lastsl) {
	    lastsl++;
	    SDL_strlcat(cdromname,lastsl,len);
	}
    }

#ifdef DEBUG_CDROM
    fprintf(stderr, "Should open drive %s, opening %s\n", SDL_cdlist[drive], cdromname);
#endif

    /*
     * Use exclusive access. Don't use SC_DIAGNOSTICS as xmcd does because they
     * require root priviledges, and we don't want that. SC_SINGLE provides
     * exclusive access with less trouble.
     */
    fd = openx(cdromname, O_RDONLY, NULL, SC_SINGLE);
    if ( fd < 0 )
    {
#ifdef DEBUG_CDROM
            fprintf(stderr, "Could not open drive %s (%s)\n", cdromname, strerror(errno));
#endif
    }
    else
    {
	struct mode_form_op cdMode;
	int                 ret;
#ifdef DEBUG_CDROM
	cdMode.action = CD_GET_MODE;
	ret = SDL_SYS_CDioctl(fd, DK_CD_MODE, &cdMode);
	if ( ret < 0 ) {
            fprintf(stderr,
	            "Could not get drive mode for %s (%s)\n",
		    cdromname, strerror(errno));
	} else {
	    switch(cdMode.cd_mode_form) {
		case CD_MODE1 :
                    fprintf(stderr,
	                "Drive mode for %s is %s\n",
		        cdromname, "CD-ROM Data Mode 1");
		    break;
		case CD_MODE2_FORM1 :
                    fprintf(stderr,
	                "Drive mode for %s is %s\n",
		        cdromname, "CD-ROM XA Data Mode 2 Form 1");
		    break;
		case CD_MODE2_FORM2 :
                    fprintf(stderr,
	                "Drive mode for %s is %s\n",
		        cdromname, "CD-ROM XA Data Mode 2 Form 2");
		    break;
		case CD_DA :
                    fprintf(stderr,
	                "Drive mode for %s is %s\n",
		        cdromname, "CD-DA");
		    break;
		default :
                    fprintf(stderr,
	                "Drive mode for %s is %s\n",
		        cdromname, "unknown");
		    break;
	    }
	}
#endif

	cdMode.action       = CD_CHG_MODE;
	cdMode.cd_mode_form = CD_DA;
	ret = SDL_SYS_CDioctl(fd, DK_CD_MODE, &cdMode);
	if ( ret < 0 ) {
#ifdef DEBUG_CDROM
            fprintf(stderr,
	            "Could not set drive mode for %s (%s)\n",
		    cdromname, strerror(errno));
#endif
            SDL_SetError("ioctl() error: Could not set CD drive mode, %s",
	                 strerror(errno));
	} else {
#ifdef DEBUG_CDROM
            fprintf(stderr,
	            "Drive mode for %s set to CD_DA\n",
		    cdromname);
#endif
	}
    }
    SDL_free(cdromname);
    return fd;
}

static int SDL_SYS_CDGetTOC(SDL_CD *cdrom)
{
    struct cd_audio_cmd cmd;
    struct cd_audio_cmd entry;
    int                 i;
    int                 okay;

    cmd.audio_cmds = CD_TRK_INFO_AUDIO;
    cmd.msf_flag   = FALSE;
    if ( SDL_SYS_CDioctl(cdrom->id, DKAUDIO, &cmd) < 0 ) {
	return -1;
    }

    okay = 0;
    cdrom->numtracks = cmd.indexing.track_index.last_track
		     - cmd.indexing.track_index.first_track+1;
    if ( cdrom->numtracks > SDL_MAX_TRACKS ) {
        cdrom->numtracks = SDL_MAX_TRACKS;
    }

    /* Read all the track TOC entries */
    for ( i=0; i<=cdrom->numtracks; ++i ) {
        if ( i == cdrom->numtracks ) {
            cdrom->track[i].id = 0xAA;;
        } else {
            cdrom->track[i].id = cmd.indexing.track_index.first_track+i;
        }
        entry.audio_cmds         = CD_GET_TRK_MSF;
	entry.indexing.track_msf.track = cdrom->track[i].id;
	if ( SDL_SYS_CDioctl(cdrom->id, DKAUDIO, &entry) < 0 ) {
            break;
        } else {
            cdrom->track[i].type = 0;    /* don't know how to detect 0x04 data track */
            cdrom->track[i].offset = MSF_TO_FRAMES(
                entry.indexing.track_msf.mins,
                entry.indexing.track_msf.secs,
                entry.indexing.track_msf.frames);
            cdrom->track[i].length = 0;
            if ( i > 0 ) {
                cdrom->track[i-1].length = cdrom->track[i].offset
		                         - cdrom->track[i-1].offset;
            }
        }
    }
    if ( i == (cdrom->numtracks+1) ) {
        okay = 1;
    }
    return(okay ? 0 : -1);
}

/* Get CD-ROM status */
static CDstatus SDL_SYS_CDStatus(SDL_CD *cdrom, int *position)
{
    CDstatus            status;
    struct cd_audio_cmd cmd;
    cmd.audio_cmds = CD_INFO_AUDIO;

    if ( SDL_SYS_CDioctl(cdrom->id, DKAUDIO, &cmd) < 0 ) {
#ifdef DEBUG_CDROM
    fprintf(stderr, "ioctl failed in SDL_SYS_CDStatus (%s)\n", SDL_GetError());
#endif
        status = CD_ERROR;
    } else {
        switch (cmd.status) {
            case CD_NO_AUDIO:
            case CD_COMPLETED:
                status = CD_STOPPED;
                break;
            case CD_PLAY_AUDIO:
                status = CD_PLAYING;
                break;
            case CD_PAUSE_AUDIO:
                status = CD_PAUSED;
                break;
            case CD_NOT_VALID:
#ifdef DEBUG_CDROM
    fprintf(stderr, "cdStatus failed with CD_NOT_VALID\n");
#endif
                status = CD_ERROR;
                break;
            case CD_STATUS_ERROR:
#ifdef DEBUG_CDROM
    fprintf(stderr, "cdStatus failed with CD_STATUS_ERROR\n");
#endif
                status = CD_ERROR;
                break;
            default:
#ifdef DEBUG_CDROM
    fprintf(stderr, "cdStatus failed with unknown error\n");
#endif
                status = CD_ERROR;
                break;
        }
    }
    if ( position ) {
        if ( status == CD_PLAYING || (status == CD_PAUSED) ) {
            *position = MSF_TO_FRAMES( cmd.indexing.info_audio.current_mins,
                                       cmd.indexing.info_audio.current_secs,
                                       cmd.indexing.info_audio.current_frames);
        } else {
            *position = 0;
        }
    }
    return status;
}

/* Start play */
static int SDL_SYS_CDPlay(SDL_CD *cdrom, int start, int length)
{
    struct cd_audio_cmd cmd;

    /*
     * My CD Rom is muted by default. I think I read that this is new with
     * AIX 4.3. SDL does not change the volume, so I need a kludge. Maybe
     * its better to do this elsewhere?
     */
    cmd.audio_cmds = CD_PLAY_AUDIO | CD_SET_VOLUME;
    cmd.msf_flag   = TRUE;
    FRAMES_TO_MSF(start,
                  &cmd.indexing.msf.first_mins,
                  &cmd.indexing.msf.first_secs,
                  &cmd.indexing.msf.first_frames);
    FRAMES_TO_MSF(start+length,
                  &cmd.indexing.msf.last_mins,
                  &cmd.indexing.msf.last_secs,
                  &cmd.indexing.msf.last_frames);
    cmd.volume_type     = CD_VOLUME_ALL;
    cmd.all_channel_vol = 255;   /* This is a uchar. What is a good value? No docu! */
    cmd.out_port_0_sel  = CD_AUDIO_CHNL_0;
    cmd.out_port_1_sel  = CD_AUDIO_CHNL_1;
    cmd.out_port_2_sel  = CD_AUDIO_CHNL_2;
    cmd.out_port_3_sel  = CD_AUDIO_CHNL_3;

#ifdef DEBUG_CDROM
  fprintf(stderr, "Trying to play from %d:%d:%d to %d:%d:%d\n",
	cmd.indexing.msf.first_mins,
	cmd.indexing.msf.first_secs,
	cmd.indexing.msf.first_frames,
	cmd.indexing.msf.last_mins,
	cmd.indexing.msf.last_secs,
	cmd.indexing.msf.last_frames);
#endif
	return(SDL_SYS_CDioctl(cdrom->id, DKAUDIO, &cmd));
}

/* Pause play */
static int SDL_SYS_CDPause(SDL_CD *cdrom)
{
    struct cd_audio_cmd cmd;
    cmd.audio_cmds = CD_PAUSE_AUDIO;
    return(SDL_SYS_CDioctl(cdrom->id, DKAUDIO, &cmd));
}

/* Resume play */
static int SDL_SYS_CDResume(SDL_CD *cdrom)
{
    struct cd_audio_cmd cmd;
    cmd.audio_cmds = CD_RESUME_AUDIO;
    return(SDL_SYS_CDioctl(cdrom->id, DKAUDIO, &cmd));
}

/* Stop play */
static int SDL_SYS_CDStop(SDL_CD *cdrom)
{
    struct cd_audio_cmd cmd;
    cmd.audio_cmds = CD_STOP_AUDIO;
    return(SDL_SYS_CDioctl(cdrom->id, DKAUDIO, &cmd));
}

/* Eject the CD-ROM */
static int SDL_SYS_CDEject(SDL_CD *cdrom)
{
    return(SDL_SYS_CDioctl(cdrom->id, DKEJECT, 0));
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

#endif /* SDL_CDROM_AIX */
