/*
    SDL - Simple DirectMedia Layer
    Copyright (C) 1997-2012 Sam Lantinga

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

    Sam Lantinga
    slouken@libsdl.org
*/
#include "SDL_config.h"

/* This is the MacOS specific header for the SDL CD-ROM API
   Contributed by Matt Slot
 */

/* AppleCD Control calls */
#define kVerifyTheDisc   	  5		/* Returns noErr if there is disc inserted */
#define kEjectTheDisc   	  7		/* Eject disc from drive */
#define kUserEject    		 80		/* Enable/disable the CD-ROM eject button */
#define kReadTOC    		100		/* Extract various TOC information from the disc */
#define kReadQ   			101		/* Extract Q subcode info for the current track */
#define kAudioTrackSearch   103		/* Start playback from the indicated position */
#define kAudioPlay    		104		/* Start playback from the indicated position */
#define kAudioPause    		105		/* Pause/continue the playback */
#define kAudioStop    		106		/* Stop playback at the indicated position */
#define kAudioStatus    	107		/* Return audio play status */
#define kAudioControl    	109		/* Set the output volume for the audio channels */
#define kReadAudioVolume   	112		/* Get the output volume for the audio channels */
#define kSetTrackList   	122		/* Set the track program for the audio CD to play */
#define kGetTrackList   	123		/* Get the track program the audio CD is playing */
#define kGetTrackIndex   	124		/* Get the track index the audio CD is playing */
#define kSetPlayMode   		125		/* Set the audio tracks play mode */
#define kGetPlayMode   		126		/* Get the audio tracks play mode */

/* AppleCD Status calls */
#define kGetDriveType   	 96		/* Get the type of the physical CD-ROM drive */
#define kWhoIsThere    		 97		/* Get a bitmap of SCSI IDs the driver controls */
#define kGetBlockSize    	 98		/* Get current block size of the CD-ROM drive */
	
/* AppleCD other constants */
#define kBlockPosition    	  0		/* Position at the specified logical block number */
#define kAbsMSFPosition    	  1		/* Position at the specified Min/Sec/Frame (in BCD) */
#define kTrackPosition    	  2		/* Position at the specified track number (in BCD) */
#define kIndexPosition    	  3		/* Position at the nth track in program (in BCD) */

#define kMutedPlayMode   	  0		/* Play the audio track with no output */
#define kStereoPlayMode   	  9		/* Play the audio track in normal stereo */

#define kControlFieldMask  	0x0D	/* Bits 3,2,0 in the nibble */
#define kDataTrackMask   	0x04	/* Indicates Data Track */

#define kGetTrackRange    	  1		/* Query TOC for track numbers */
#define kGetLeadOutArea    	  2		/* Query TOC for "Lead Out" end of audio data */
#define kGetTrackEntries   	  3		/* Query TOC for track starts and data types */

#define kStatusPlaying		  0		/* Audio Play operation in progress */
#define kStatusPaused		  1		/* CD-ROM device in Hold Track ("Pause") state */
#define kStatusMuted		  2		/* MUTING-ON operation in progress */
#define kStatusDone			  3		/* Audio Play completed */
#define kStatusError		  4		/* Error occurred during audio play operation */
#define kStatusStopped		  5		/* Audio play operation not requested */

#define kPlayModeSequential	  0		/*  Play tracks in order */
#define kPlayModeShuffled	  1		/* Play tracks randomly */
#define kPlayModeProgrammed   2		/* Use custom playlist */

/* AppleCD Gestalt selectors */
#define kGestaltAudioCDSelector    'aucd'
#define kDriverVersion52   		0x00000520
#define kDriverVersion51   		0x00000510
#define kDriverVersion50   		0x00000500

/* Drive type constants */
#define kDriveAppleCD_SC   				  1
#define kDriveAppleCD_SCPlus_or_150   	  2
#define kDriveAppleCD_300_or_300Plus   	  3

/* Misc constants */
#define kFirstSCSIDevice   	 -33
#define kLastSCSIDevice    	 -40

#if PRAGMA_STRUCT_ALIGN
	#pragma options align=mac68k
#endif

/* AppleCD driver parameter block */
typedef struct CDCntrlParam {
	QElemPtr				qLink;
	short					qType;
	short					ioTrap;
	Ptr						ioCmdAddr;
	IOCompletionUPP			ioCompletion;
	OSErr					ioResult;
	StringPtr				ioNamePtr;
	short					ioVRefNum;
	short					ioCRefNum;
	short					csCode;
	
	union {
		long				longs[6];
		short				words[11];
		unsigned char		bytes[22];
		struct {
			unsigned char	status;
			unsigned char	play;
			unsigned char	control;
			unsigned char	minute;
			unsigned char	second;
			unsigned char	frame;
			} cd;
		} csParam;

	} CDCntrlParam, *CDCntrlParamPtr;

typedef union CDTrackData {
	long				value;			/* Treat as a longword value */
	struct {
		unsigned char	reserved : 4;	/* Unused by AppleCD driver  */
		unsigned char	control : 4;	/* Track flags (data track?) */
		unsigned char	min;			/* Start of track (BCD)      */
		unsigned char	sec;			/* Start of track (BCD)      */
		unsigned char	frame;			/* Start of track (BCD)      */
		} entry;						/* Broken into fields        */
	} CDTrackData, *CDTrackPtr;
	
#if PRAGMA_STRUCT_ALIGN
	#pragma options align=reset
#endif
