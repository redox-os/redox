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

#if defined(__APPLE__) && defined(__MACH__)
#include <Carbon/Carbon.h>
#elif TARGET_API_MAC_CARBON && (UNIVERSAL_INTERFACES_VERSION > 0x0335)
#include <Carbon.h>
#else
#include <Windows.h>
#include <Strings.h>
#endif

#if SDL_MACCLASSIC_GAMMA_SUPPORT
#include <Devices.h>
#include <Files.h>
#include <MacTypes.h>
#include <QDOffscreen.h>
#include <Quickdraw.h>
#include <Video.h>
#endif

#include "SDL_stdinc.h"
#include "SDL_macwm_c.h"

void Mac_SetCaption(_THIS, const char *title, const char *icon)
{
	/* Don't convert C to P string in place, because it may be read-only */
	Str255		ptitle; /* MJS */
	ptitle[0] = strlen (title);
	SDL_memcpy(ptitle+1, title, ptitle[0]); /* MJS */
	if (SDL_Window)
		SetWTitle(SDL_Window, ptitle); /* MJS */
}

#if SDL_MACCLASSIC_GAMMA_SUPPORT
/*
 * ADC Gamma Ramp support...
 *
 * Mac Gamma Ramp code was originally from sample code provided by
 *  Apple Developer Connection, and not written specifically for SDL:
 * "Contains: Functions to enable Mac OS device gamma adjustments using 3 channel 256 element 8 bit gamma ramps
 *  Written by: Geoff Stahl (ggs)
 *  Copyright: Copyright (c) 1999 Apple Computer, Inc., All Rights Reserved
 *  Disclaimer: You may incorporate this sample code into your applications without
 *              restriction, though the sample code has been provided "AS IS" and the
 *              responsibility for its operation is 100% yours.  However, what you are
 *              not permitted to do is to redistribute the source as "DSC Sample Code"
 *              after having made changes. If you're going to re-distribute the source,
 *              we require that you make it clear in the source that the code was
 *              descended from Apple Sample Code, but that you've made changes."
 * (The sample code has been integrated into this file, and thus is modified from the original Apple sources.)
 */

typedef struct recDeviceGamma											/* storage for device handle and gamma table */
{
	GDHandle hGD;												/* handle to device */
	GammaTblPtr pDeviceGamma;									/* pointer to device gamma table */
} recDeviceGamma;
typedef recDeviceGamma * precDeviceGamma;

typedef struct recSystemGamma											/* storage for system devices and gamma tables */
{
	short numDevices;											/* number of devices */
	precDeviceGamma * devGamma;									/* array of pointers to device gamma records */
} recSystemGamma;
typedef recSystemGamma * precSystemGamma;

static Ptr CopyGammaTable (GammaTblPtr pTableGammaIn)
{
	GammaTblPtr		pTableGammaOut = NULL;
	short			tableSize, dataWidth;

	if (pTableGammaIn)												/* if there is a table to copy  */
	{
		dataWidth = (pTableGammaIn->gDataWidth + 7) / 8;			/* number of bytes per entry */
		tableSize = sizeof (GammaTbl) + pTableGammaIn->gFormulaSize +
					(pTableGammaIn->gChanCnt * pTableGammaIn->gDataCnt * dataWidth);
		pTableGammaOut = (GammaTblPtr) NewPtr (tableSize);			/* allocate new table */
		if (pTableGammaOut)											
			BlockMove( (Ptr)pTableGammaIn, (Ptr)pTableGammaOut, tableSize);	/* move everything */
	}
	return (Ptr)pTableGammaOut;										/* return whatever we allocated, could be NULL */
}

static OSErr GetGammaTable (GDHandle hGD, GammaTblPtr * ppTableGammaOut)
{
	VDGammaRecord   DeviceGammaRec;
	CntrlParam		cParam;
	OSErr			err;
	
	cParam.ioCompletion = NULL;										/* set up control params */
	cParam.ioNamePtr = NULL;
	cParam.ioVRefNum = 0;
	cParam.ioCRefNum = (**hGD).gdRefNum;
	cParam.csCode = cscGetGamma;									/* Get Gamma commnd to device */
	*(Ptr *)cParam.csParam = (Ptr) &DeviceGammaRec;					/* record for gamma */

	err = PBStatusSync( (ParmBlkPtr)&cParam );						/* get gamma */
	
	*ppTableGammaOut = (GammaTblPtr)(DeviceGammaRec.csGTable);		/* pull table out of record */
	
	return err;	
}

static Ptr GetDeviceGamma (GDHandle hGD)
{
	GammaTblPtr		pTableGammaDevice = NULL;
	GammaTblPtr		pTableGammaReturn = NULL;	
	OSErr			err;
	
	err = GetGammaTable (hGD, &pTableGammaDevice);					/* get a pointer to the devices table */
	if ((noErr == err) && pTableGammaDevice)						/* if succesful */
		pTableGammaReturn = (GammaTblPtr) CopyGammaTable (pTableGammaDevice); /* copy to global */

	return (Ptr) pTableGammaReturn;
}

static void DisposeGammaTable (Ptr pGamma)
{
	if (pGamma)
		DisposePtr((Ptr) pGamma);									/* get rid of it */
}

static void DisposeSystemGammas (Ptr* ppSystemGammas)
{
	precSystemGamma pSysGammaIn;
	if (ppSystemGammas)
	{
		pSysGammaIn = (precSystemGamma) *ppSystemGammas;
		if (pSysGammaIn)
		{
			short i;
			for (i = 0; i < pSysGammaIn->numDevices; i++)		/* for all devices */
				if (pSysGammaIn->devGamma [i])						/* if pointer is valid */
				{
					DisposeGammaTable ((Ptr) pSysGammaIn->devGamma [i]->pDeviceGamma); /* dump gamma table */
					DisposePtr ((Ptr) pSysGammaIn->devGamma [i]);					   /* dump device info */
				}
			DisposePtr ((Ptr) pSysGammaIn->devGamma);				/* dump device pointer array		 */
			DisposePtr ((Ptr) pSysGammaIn);							/* dump system structure */
			*ppSystemGammas = NULL;
		}	
	}
}

static Boolean GetDeviceGammaRampGD (GDHandle hGD, Ptr pRamp)
{
	GammaTblPtr		pTableGammaTemp = NULL;
	long 			indexChan, indexEntry;
	OSErr			err;
	
	if (pRamp)															/* ensure pRamp is allocated */
	{
		err = GetGammaTable (hGD, &pTableGammaTemp);					/* get a pointer to the current gamma */
		if ((noErr == err) && pTableGammaTemp)							/* if successful */
		{															
			/* fill ramp */
			unsigned char * pEntry = (unsigned char *) &pTableGammaTemp->gFormulaData + pTableGammaTemp->gFormulaSize; /* base of table */
			short bytesPerEntry = (pTableGammaTemp->gDataWidth + 7) / 8; /* size, in bytes, of the device table entries */
			short shiftRightValue = pTableGammaTemp->gDataWidth - 8; 	 /* number of right shifts device -> ramp */
			short channels = pTableGammaTemp->gChanCnt;	
			short entries = pTableGammaTemp->gDataCnt;									
			if (3 == channels)											/* RGB format */
			{															/* note, this will create runs of entries if dest. is bigger (not linear interpolate) */
				for (indexChan = 0; indexChan < channels; indexChan++)
					for (indexEntry = 0; indexEntry < 256; indexEntry++)
						*((unsigned char *) pRamp + (indexChan * 256) + indexEntry) = 
						  *(pEntry + indexChan * entries * bytesPerEntry + indexEntry * entries * bytesPerEntry / 256) >> shiftRightValue;
			}
			else														/* single channel format */
			{
				for (indexChan = 0; indexChan < 768; indexChan += 256)	/* repeat for all 3 channels (step by ramp size) */
					for (indexEntry = 0; indexEntry < 256; indexEntry++) /* for all entries set vramp value */
						*((unsigned char *) pRamp + indexChan + indexEntry) = 
						  *(pEntry + indexEntry * entries * bytesPerEntry / 256) >> shiftRightValue;
			}
			return true;
		}
	}
	return false;
}

static Ptr GetSystemGammas (void)
{
	precSystemGamma pSysGammaOut;									/* return pointer to system device gamma info */
	short devCount = 0;												/* number of devices attached */
	Boolean fail = false;
	GDHandle hGDevice;
	
	pSysGammaOut = (precSystemGamma) NewPtr (sizeof (recSystemGamma)); /* allocate for structure */
	
	hGDevice = GetDeviceList ();							/* top of device list */
	do																/* iterate */
	{
		devCount++;													/* count devices					 */
		hGDevice = GetNextDevice (hGDevice);						/* next device */
	} while (hGDevice);
	
	pSysGammaOut->devGamma = (precDeviceGamma *) NewPtr (sizeof (precDeviceGamma) * devCount); /* allocate for array of pointers to device records */
	if (pSysGammaOut)
	{
		pSysGammaOut->numDevices = devCount;						/* stuff count */
		
		devCount = 0;												/* reset iteration */
		hGDevice = GetDeviceList ();
		do
		{
			pSysGammaOut->devGamma [devCount] = (precDeviceGamma) NewPtr (sizeof (recDeviceGamma));	  /* new device record */
			if (pSysGammaOut->devGamma [devCount])					/* if we actually allocated memory */
			{
				pSysGammaOut->devGamma [devCount]->hGD = hGDevice;										  /* stuff handle */
				pSysGammaOut->devGamma [devCount]->pDeviceGamma = (GammaTblPtr)GetDeviceGamma (hGDevice); /* copy gamma table */
			}
			else													/* otherwise dump record on exit */
			 fail = true;
			devCount++;												/* next device */
			hGDevice = GetNextDevice (hGDevice);						
		} while (hGDevice);
	}
	if (!fail)														/* if we did not fail */
		return (Ptr) pSysGammaOut;									/* return pointer to structure */
	else
	{
		DisposeSystemGammas ((Ptr *) &pSysGammaOut);					/* otherwise dump the current structures (dispose does error checking) */
		return NULL;												/* could not complete */
	}
}

static void RestoreDeviceGamma (GDHandle hGD, Ptr pGammaTable)
{
	VDSetEntryRecord setEntriesRec;
	VDGammaRecord	gameRecRestore;
	CTabHandle      hCTabDeviceColors;
	Ptr				csPtr;
	OSErr			err = noErr;
	
	if (pGammaTable)												/* if we have a table to restore								 */
	{
		gameRecRestore.csGTable = pGammaTable;						/* setup restore record */
		csPtr = (Ptr) &gameRecRestore;
		err = Control((**hGD).gdRefNum, cscSetGamma, (Ptr) &csPtr);	/* restore gamma */

		if ((noErr == err) && (8 == (**(**hGD).gdPMap).pixelSize))	/* if successful and on an 8 bit device */
		{
			hCTabDeviceColors = (**(**hGD).gdPMap).pmTable;			/* do SetEntries to force CLUT update */
			setEntriesRec.csTable = (ColorSpec *) &(**hCTabDeviceColors).ctTable;
			setEntriesRec.csStart = 0;
			setEntriesRec.csCount = (**hCTabDeviceColors).ctSize;
			csPtr = (Ptr) &setEntriesRec;
			
			err = Control((**hGD).gdRefNum, cscSetEntries, (Ptr) &csPtr); /* SetEntries in CLUT */
		}
	}
}

static void RestoreSystemGammas (Ptr pSystemGammas)
{
	short i;
	precSystemGamma pSysGammaIn = (precSystemGamma) pSystemGammas;
	if (pSysGammaIn)
		for (i = 0; i < pSysGammaIn->numDevices; i++)			/* for all devices */
			RestoreDeviceGamma (pSysGammaIn->devGamma [i]->hGD, (Ptr) pSysGammaIn->devGamma [i]->pDeviceGamma);	/* restore gamma */
}

static Ptr CreateEmptyGammaTable (short channels, short entries, short bits)
{
	GammaTblPtr		pTableGammaOut = NULL;
	short			tableSize, dataWidth;

	dataWidth = (bits + 7) / 8;										/* number of bytes per entry */
	tableSize = sizeof (GammaTbl) + (channels * entries * dataWidth);
	pTableGammaOut = (GammaTblPtr) NewPtrClear (tableSize);			/* allocate new tabel */

	if (pTableGammaOut)												/* if we successfully allocated */
	{
		pTableGammaOut->gVersion = 0;								/* set parameters based on input */
		pTableGammaOut->gType = 0;
		pTableGammaOut->gFormulaSize = 0;
		pTableGammaOut->gChanCnt = channels;
		pTableGammaOut->gDataCnt = entries;
		pTableGammaOut->gDataWidth = bits;
	}
	return (Ptr)pTableGammaOut;										/* return whatever we allocated */
}

static Boolean SetDeviceGammaRampGD (GDHandle hGD, Ptr pRamp)
{
	VDSetEntryRecord setEntriesRec;
	VDGammaRecord	gameRecRestore;
	GammaTblPtr		pTableGammaNew;
	GammaTblPtr		pTableGammaCurrent = NULL;
	CTabHandle      hCTabDeviceColors;
	Ptr				csPtr;
	OSErr			err;
	short 			dataBits, entries, channels = 3;						/* force three channels in the gamma table */
	
	if (pRamp)																/* ensure pRamp is allocated */
	{
		err= GetGammaTable (hGD, &pTableGammaCurrent);						/* get pointer to current table */
		if ((noErr == err) && pTableGammaCurrent)
		{
			dataBits = pTableGammaCurrent->gDataWidth;						/* table must have same data width */
			entries = pTableGammaCurrent->gDataCnt;							/* table must be same size */
			pTableGammaNew = (GammaTblPtr) CreateEmptyGammaTable (channels, entries, dataBits); /* our new table */
			if (pTableGammaNew)												/* if successful fill table */
			{	
				unsigned char * pGammaBase = (unsigned char *) &pTableGammaNew->gFormulaData + pTableGammaNew->gFormulaSize; /* base of table */
				if ((256 == entries) && (8 == dataBits)) 						/* simple case: direct mapping */
					BlockMove ((Ptr)pRamp, (Ptr)pGammaBase, channels * entries); /* move everything */
				else														/* tough case handle entry, channel and data size disparities */
				{
					short indexChan, indexEntry;
					short bytesPerEntry = (dataBits + 7) / 8; 				/* size, in bytes, of the device table entries */
					short shiftRightValue = 8 - dataBits;					/* number of right shifts ramp -> device */
					shiftRightValue += ((bytesPerEntry - 1) * 8);  			/* multibyte entries and the need to map a byte at a time most sig. to least sig. */
					for (indexChan = 0; indexChan < channels; indexChan++) /* for all the channels */
						for (indexEntry = 0; indexEntry < entries; indexEntry++) /* for all the entries */
						{
							short currentShift = shiftRightValue;			/* reset current bit shift */
							long temp = *((unsigned char *)pRamp + (indexChan << 8) + (indexEntry << 8) / entries); /* get data from ramp */
							short indexByte;
							for (indexByte = 0; indexByte < bytesPerEntry; indexByte++) /* for all bytes */
							{
								if (currentShift < 0)						/* shift data correctly for current byte */
									*(pGammaBase++) = temp << -currentShift;
								else
									*(pGammaBase++) = temp >> currentShift;
								currentShift -= 8;							/* increment shift to align to next less sig. byte */
							}
						}
				}
				
				/* set gamma */
				gameRecRestore.csGTable = (Ptr) pTableGammaNew;				/* setup restore record */
				csPtr = (Ptr) &gameRecRestore;
				err = Control((**hGD).gdRefNum, cscSetGamma, (Ptr) &csPtr);	/* restore gamma (note, display drivers may delay returning from this until VBL) */
				
				if ((8 == (**(**hGD).gdPMap).pixelSize) && (noErr == err))	/* if successful and on an 8 bit device */
				{
					hCTabDeviceColors = (**(**hGD).gdPMap).pmTable;			/* do SetEntries to force CLUT update */
					setEntriesRec.csTable = (ColorSpec *) &(**hCTabDeviceColors).ctTable;
					setEntriesRec.csStart = 0;
					setEntriesRec.csCount = (**hCTabDeviceColors).ctSize;
					csPtr = (Ptr) &setEntriesRec;
					err = Control((**hGD).gdRefNum, cscSetEntries, (Ptr) &csPtr);	/* SetEntries in CLUT */
				}
				DisposeGammaTable ((Ptr) pTableGammaNew);					/* dump table */
				if (noErr == err)
					return true;
			}
		}
	}
	else																	/* set NULL gamma -> results in linear map */
	{
		gameRecRestore.csGTable = (Ptr) NULL;								/* setup restore record */
		csPtr = (Ptr) &gameRecRestore;
		err = Control((**hGD).gdRefNum, cscSetGamma, (Ptr) &csPtr);			/* restore gamma */
		
		if ((8 == (**(**hGD).gdPMap).pixelSize) && (noErr == err))			/* if successful and on an 8 bit device */
		{
			hCTabDeviceColors = (**(**hGD).gdPMap).pmTable;					/* do SetEntries to force CLUT update */
			setEntriesRec.csTable = (ColorSpec *) &(**hCTabDeviceColors).ctTable;
			setEntriesRec.csStart = 0;
			setEntriesRec.csCount = (**hCTabDeviceColors).ctSize;
			csPtr = (Ptr) &setEntriesRec;
			err = Control((**hGD).gdRefNum, cscSetEntries, (Ptr) &csPtr);	/* SetEntries in CLUT */
		}
		if (noErr == err)
			return true;
	}
	return false;															/* memory allocation or device control failed if we get here */
}

/* end of ADC Gamma Ramp support code... */

static Ptr systemGammaPtr;

void Mac_QuitGamma(_THIS)
{
	if (systemGammaPtr)
	{
		RestoreSystemGammas(systemGammaPtr);
		DisposeSystemGammas(&systemGammaPtr);
	}
}

static unsigned char shiftedRamp[3 * 256];

int Mac_SetGammaRamp(_THIS, Uint16 *ramp)
{
	int i;
	if (!systemGammaPtr)
		systemGammaPtr = GetSystemGammas();
	for (i = 0; i < 3 * 256; i++)
	{
		shiftedRamp[i] = ramp[i] >> 8;
	}

	if (SetDeviceGammaRampGD(GetMainDevice(), (Ptr) shiftedRamp))
		return 0;
	else
		return -1;
}

int Mac_GetGammaRamp(_THIS, Uint16 *ramp)
{
	if (GetDeviceGammaRampGD(GetMainDevice(), (Ptr) shiftedRamp))
	{
		int i;
		for (i = 0; i < 3 * 256; i++)
		{
			ramp[i] = shiftedRamp[i] << 8;
		}
		return 0;
	}
	else
		return -1;
}

#endif  /* SDL_MACCLASSIC_GAMMA_SUPPORT */


