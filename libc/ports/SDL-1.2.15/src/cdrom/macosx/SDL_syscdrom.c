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

#ifdef SDL_CDROM_MACOSX

#include "SDL_syscdrom_c.h"

#pragma mark -- Globals --

static FSRef**         tracks;
static FSVolumeRefNum* volumes;
static CDstatus        status;
static int             nextTrackFrame;
static int             nextTrackFramesRemaining;
static int             fakeCD;
static int             currentTrack;
static int             didReadTOC;
static int             cacheTOCNumTracks;
static int             currentDrive; /* Only allow 1 drive in use at a time */

#pragma mark -- Prototypes --

static const char *SDL_SYS_CDName   (int drive);
static int         SDL_SYS_CDOpen   (int drive);
static int         SDL_SYS_CDGetTOC (SDL_CD *cdrom);
static CDstatus    SDL_SYS_CDStatus (SDL_CD *cdrom, int *position);
static int         SDL_SYS_CDPlay   (SDL_CD *cdrom, int start, int length);
static int         SDL_SYS_CDPause  (SDL_CD *cdrom);
static int         SDL_SYS_CDResume (SDL_CD *cdrom);
static int         SDL_SYS_CDStop   (SDL_CD *cdrom);
static int         SDL_SYS_CDEject  (SDL_CD *cdrom);
static void        SDL_SYS_CDClose  (SDL_CD *cdrom);

#pragma mark -- Helper Functions --

/* Read a list of tracks from the volume */
static int LoadTracks (SDL_CD *cdrom)
{
    /* Check if tracks are already loaded */
    if  ( tracks[cdrom->id] != NULL )
        return 0;
        
    /* Allocate memory for tracks */
    tracks[cdrom->id] = (FSRef*) SDL_calloc (1, sizeof(**tracks) * cdrom->numtracks);
    if (tracks[cdrom->id] == NULL) {
        SDL_OutOfMemory ();
        return -1;
    }
    
    /* Load tracks */
    if (ListTrackFiles (volumes[cdrom->id], tracks[cdrom->id], cdrom->numtracks) < 0)
        return -1;

    return 0;
}

/* Find a file for a given start frame and length */
static FSRef* GetFileForOffset (SDL_CD *cdrom, int start, int length,  int *outStartFrame, int *outStopFrame)
{
    int i;
    
    for (i = 0; i < cdrom->numtracks; i++) {
    
        if (cdrom->track[i].offset <= start &&
            start < (cdrom->track[i].offset + cdrom->track[i].length))
            break;
    }
    
    if (i == cdrom->numtracks)
        return NULL;
        
    currentTrack = i;

    *outStartFrame = start - cdrom->track[i].offset;
    
    if ((*outStartFrame + length) < cdrom->track[i].length) {
        *outStopFrame = *outStartFrame + length;
        length = 0;
        nextTrackFrame = -1;
        nextTrackFramesRemaining = -1;
    }
    else {
        *outStopFrame = -1;
        length -= cdrom->track[i].length - *outStartFrame;
        nextTrackFrame = cdrom->track[i+1].offset;
        nextTrackFramesRemaining = length;
    }
    
    return &tracks[cdrom->id][i];
}

/* Setup another file for playback, or stop playback (called from another thread) */
static void CompletionProc (SDL_CD *cdrom)
{
    
    Lock ();
    
    if (nextTrackFrame > 0 && nextTrackFramesRemaining > 0) {
    
        /* Load the next file to play */
        int startFrame, stopFrame;
        FSRef *file;
        
        PauseFile ();
        ReleaseFile ();
                
        file = GetFileForOffset (cdrom, nextTrackFrame, 
            nextTrackFramesRemaining, &startFrame, &stopFrame);
        
        if (file == NULL) {
            status = CD_STOPPED;
            Unlock ();
            return;
        }
        
        LoadFile (file, startFrame, stopFrame);
        
        SetCompletionProc (CompletionProc, cdrom);
        
        PlayFile ();
    }
    else {
    
        /* Release the current file */
        PauseFile ();
        ReleaseFile ();
        status = CD_STOPPED;
    }
    
    Unlock ();
}


#pragma mark -- Driver Functions --

/* Initialize */
int SDL_SYS_CDInit (void) 
{
    /* Initialize globals */
    volumes = NULL;
    tracks  = NULL;
    status  = CD_STOPPED;
    nextTrackFrame = -1;
    nextTrackFramesRemaining = -1;
    fakeCD  = SDL_FALSE;
    currentTrack = -1;
    didReadTOC = SDL_FALSE;
    cacheTOCNumTracks = -1;
    currentDrive = -1;
    
    /* Fill in function pointers */
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

    /* 
        Read the list of "drives"
        
        This is currently a hack that infers drives from
        mounted audio CD volumes, rather than
        actual CD-ROM devices - which means it may not
        act as expected sometimes.
    */
    
    /* Find out how many cd volumes are mounted */
    SDL_numcds = DetectAudioCDVolumes (NULL, 0);

    /*
        If there are no volumes, fake a cd device
        so tray empty can be reported.
    */
    if (SDL_numcds == 0) {
    
        fakeCD = SDL_TRUE;
        SDL_numcds = 1;
        status = CD_TRAYEMPTY;
        
        return 0;
    }
    
    /* Allocate space for volumes */
    volumes = (FSVolumeRefNum*) SDL_calloc (1, sizeof(*volumes) * SDL_numcds);
    if (volumes == NULL) {
        SDL_OutOfMemory ();
        return -1;
    }
    
    /* Allocate space for tracks */
    tracks = (FSRef**) SDL_calloc (1, sizeof(*tracks) * (SDL_numcds + 1));
    if (tracks == NULL) {
        SDL_OutOfMemory ();
        return -1;
    }
    
    /* Mark the end of the tracks array */
    tracks[ SDL_numcds ] = (FSRef*)-1;
    
    /* 
        Redetect, now save all volumes for later
        Update SDL_numcds just in case it changed
    */
    {
        int numVolumes = SDL_numcds;
        
        SDL_numcds = DetectAudioCDVolumes (volumes, numVolumes);
        
        /* If more cds suddenly show up, ignore them */
        if (SDL_numcds > numVolumes) {
            SDL_SetError ("Some CD's were added but they will be ignored");
            SDL_numcds = numVolumes;
        }
    }
    
    return 0;
}

/* Shutdown and cleanup */
void SDL_SYS_CDQuit(void)
{
    ReleaseFile();
    
    if (volumes != NULL)
        free (volumes);
        
    if (tracks != NULL) {
    
        FSRef **ptr;
        for (ptr = tracks; *ptr != (FSRef*)-1; ptr++)
            if (*ptr != NULL)
                free (*ptr);
            
        free (tracks);
    }
}

/* Get the Unix disk name of the volume */
static const char *SDL_SYS_CDName (int drive)
{
    /*
     * !!! FIXME: PBHGetVolParmsSync() is gone in 10.6,
     * !!! FIXME:  replaced with FSGetVolumeParms(), which
     * !!! FIXME:  isn't available before 10.5.  :/
     */
    return "Mac OS X CD-ROM Device";

#if 0
    OSStatus     err = noErr;
    HParamBlockRec  pb;
    GetVolParmsInfoBuffer   volParmsInfo;
   
    if (fakeCD)
        return "Fake CD-ROM Device";

    pb.ioParam.ioNamePtr = NULL;
    pb.ioParam.ioVRefNum = volumes[drive];
    pb.ioParam.ioBuffer = (Ptr)&volParmsInfo;
    pb.ioParam.ioReqCount = (SInt32)sizeof(volParmsInfo);
    err = PBHGetVolParmsSync(&pb);

    if (err != noErr) {
        SDL_SetError ("PBHGetVolParmsSync returned %d", err);
        return NULL;
    }

    return volParmsInfo.vMDeviceID;
#endif
}

/* Open the "device" */
static int SDL_SYS_CDOpen (int drive)
{
    /* Only allow 1 device to be open */
    if (currentDrive >= 0) {
        SDL_SetError ("Only one cdrom is supported");
        return -1;
    }
    else
        currentDrive = drive;

    return drive;
}

/* Get the table of contents */
static int SDL_SYS_CDGetTOC (SDL_CD *cdrom)
{
    if (fakeCD) {
        SDL_SetError (kErrorFakeDevice);
        return -1;
    }
    
    if (didReadTOC) {
        cdrom->numtracks = cacheTOCNumTracks;
        return 0;
    }
    
    
    ReadTOCData (volumes[cdrom->id], cdrom);
    didReadTOC = SDL_TRUE;
    cacheTOCNumTracks = cdrom->numtracks;
    
    return 0;
}

/* Get CD-ROM status */
static CDstatus SDL_SYS_CDStatus (SDL_CD *cdrom, int *position)
{
    if (position) {
        int trackFrame;
        
        Lock ();
        trackFrame = GetCurrentFrame ();
        Unlock ();
    
        *position = cdrom->track[currentTrack].offset + trackFrame;
    }
    
    return status;
}

/* Start playback */
static int SDL_SYS_CDPlay(SDL_CD *cdrom, int start, int length)
{
    int startFrame, stopFrame;
    FSRef *ref;
    
    if (fakeCD) {
        SDL_SetError (kErrorFakeDevice);
        return -1;
    }
    
    Lock();
    
    if (LoadTracks (cdrom) < 0)
        return -2;
    
    if (PauseFile () < 0)
        return -3;
        
    if (ReleaseFile () < 0)
        return -4;
    
    ref = GetFileForOffset (cdrom, start, length, &startFrame, &stopFrame);
    if (ref == NULL) {
        SDL_SetError ("SDL_SYS_CDPlay: No file for start=%d, length=%d", start, length);
        return -5;
    }
    
    if (LoadFile (ref, startFrame, stopFrame) < 0)
        return -6;
    
    SetCompletionProc (CompletionProc, cdrom);
    
    if (PlayFile () < 0)
        return -7;
    
    status = CD_PLAYING;
    
    Unlock();
    
    return 0;
}

/* Pause playback */
static int SDL_SYS_CDPause(SDL_CD *cdrom)
{
    if (fakeCD) {
        SDL_SetError (kErrorFakeDevice);
        return -1;
    }
    
    Lock ();
    
    if (PauseFile () < 0) {
        Unlock ();
        return -2;
    }
    
    status = CD_PAUSED;
    
    Unlock ();
    
    return 0;
}

/* Resume playback */
static int SDL_SYS_CDResume(SDL_CD *cdrom)
{
    if (fakeCD) {
        SDL_SetError (kErrorFakeDevice);
        return -1;
    }
    
    Lock ();
    
    if (PlayFile () < 0) {
        Unlock ();
        return -2;
    }
        
    status = CD_PLAYING;
    
    Unlock ();
    
    return 0;
}

/* Stop playback */
static int SDL_SYS_CDStop(SDL_CD *cdrom)
{
    if (fakeCD) {
        SDL_SetError (kErrorFakeDevice);
        return -1;
    }
    
    Lock ();
    
    if (PauseFile () < 0) {
        Unlock ();
        return -2;
    }
        
    if (ReleaseFile () < 0) {
        Unlock ();
        return -3;
    }
        
    status = CD_STOPPED;
    
    Unlock ();
    
    return 0;
}

/* Eject the CD-ROM (Unmount the volume) */
static int SDL_SYS_CDEject(SDL_CD *cdrom)
{
    OSStatus err;
    pid_t dissenter;

    if (fakeCD) {
        SDL_SetError (kErrorFakeDevice);
        return -1;
    }
    
    Lock ();
    
    if (PauseFile () < 0) {
        Unlock ();
        return -2;
    }
        
    if (ReleaseFile () < 0) {
        Unlock ();
        return -3;
    }
    
    status = CD_STOPPED;
    
	/* Eject the volume */
	err = FSEjectVolumeSync(volumes[cdrom->id], kNilOptions, &dissenter);

	if (err != noErr) {
        Unlock ();
		SDL_SetError ("PBUnmountVol returned %d", err);
		return -4;
	}
    
    status = CD_TRAYEMPTY;

    /* Invalidate volume and track info */
    volumes[cdrom->id] = 0;
    free (tracks[cdrom->id]);
    tracks[cdrom->id] = NULL;
    
    Unlock ();
    
    return 0;
}

/* Close the CD-ROM */
static void SDL_SYS_CDClose(SDL_CD *cdrom)
{
    currentDrive = -1;
    return;
}

#endif /* SDL_CDROM_MACOSX */
