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

/* Allow access to a raw mixing buffer */

#include "SDL_timer.h"
#include "SDL_audio.h"
#include "../SDL_audio_c.h"
#include "SDL_dx5audio.h"

/* Define this if you want to use DirectX 6 DirectSoundNotify interface */
//#define USE_POSITION_NOTIFY

/* DirectX function pointers for audio */
HRESULT (WINAPI *DSoundCreate)(LPGUID, LPDIRECTSOUND *, LPUNKNOWN);

/* Audio driver functions */
static int DX5_OpenAudio(_THIS, SDL_AudioSpec *spec);
static void DX5_ThreadInit(_THIS);
static void DX5_WaitAudio_BusyWait(_THIS);
#ifdef USE_POSITION_NOTIFY
static void DX6_WaitAudio_EventWait(_THIS);
#endif
static void DX5_PlayAudio(_THIS);
static Uint8 *DX5_GetAudioBuf(_THIS);
static void DX5_WaitDone(_THIS);
static void DX5_CloseAudio(_THIS);

/* Audio driver bootstrap functions */

static int Audio_Available(void)
{
	HINSTANCE DSoundDLL;
	int dsound_ok;

	/* Version check DSOUND.DLL (Is DirectX okay?) */
	dsound_ok = 0;
	DSoundDLL = LoadLibrary(TEXT("DSOUND.DLL"));
	if ( DSoundDLL != NULL ) {
		/* We just use basic DirectSound, we're okay */
		/* Yay! */
		/* Unfortunately, the sound drivers on NT have
		   higher latencies than the audio buffers used
		   by many SDL applications, so there are gaps
		   in the audio - it sounds terrible.  Punt for now.
		 */
		OSVERSIONINFO ver;
		ver.dwOSVersionInfoSize = sizeof (OSVERSIONINFO);
		GetVersionEx(&ver);
		switch (ver.dwPlatformId) {
			case VER_PLATFORM_WIN32_NT:
				if ( ver.dwMajorVersion > 4 ) {
					/* Win2K */
					dsound_ok = 1;
				} else {
					/* WinNT */
					dsound_ok = 0;
				}
				break;
			default:
				/* Win95 or Win98 */
				dsound_ok = 1;
				break;
		}
		/* Now check for DirectX 5 or better - otherwise
		 * we will fail later in DX5_OpenAudio without a chance
		 * to fall back to the DIB driver. */
		if (dsound_ok) {
			/* DirectSoundCaptureCreate was added in DX5 */
			if (!GetProcAddress(DSoundDLL, TEXT("DirectSoundCaptureCreate")))
				dsound_ok = 0;

		}
		/* Clean up.. */
		FreeLibrary(DSoundDLL);
	}
	return(dsound_ok);
}

/* Functions for loading the DirectX functions dynamically */
static HINSTANCE DSoundDLL = NULL;

static void DX5_Unload(void)
{
	if ( DSoundDLL != NULL ) {
		FreeLibrary(DSoundDLL);
		DSoundCreate = NULL;
		DSoundDLL = NULL;
	}
}
static int DX5_Load(void)
{
	int status;

	DX5_Unload();
	DSoundDLL = LoadLibrary(TEXT("DSOUND.DLL"));
	if ( DSoundDLL != NULL ) {
		DSoundCreate = (void *)GetProcAddress(DSoundDLL,
					TEXT("DirectSoundCreate"));
	}
	if ( DSoundDLL && DSoundCreate ) {
		status = 0;
	} else {
		DX5_Unload();
		status = -1;
	}
	return status;
}

static void Audio_DeleteDevice(SDL_AudioDevice *device)
{
	DX5_Unload();
	SDL_free(device->hidden);
	SDL_free(device);
}

static SDL_AudioDevice *Audio_CreateDevice(int devindex)
{
	SDL_AudioDevice *this;

	/* Load DirectX */
	if ( DX5_Load() < 0 ) {
		return(NULL);
	}

	/* Initialize all variables that we clean on shutdown */
	this = (SDL_AudioDevice *)SDL_malloc(sizeof(SDL_AudioDevice));
	if ( this ) {
		SDL_memset(this, 0, (sizeof *this));
		this->hidden = (struct SDL_PrivateAudioData *)
				SDL_malloc((sizeof *this->hidden));
	}
	if ( (this == NULL) || (this->hidden == NULL) ) {
		SDL_OutOfMemory();
		if ( this ) {
			SDL_free(this);
		}
		return(0);
	}
	SDL_memset(this->hidden, 0, (sizeof *this->hidden));

	/* Set the function pointers */
	this->OpenAudio = DX5_OpenAudio;
	this->ThreadInit = DX5_ThreadInit;
	this->WaitAudio = DX5_WaitAudio_BusyWait;
	this->PlayAudio = DX5_PlayAudio;
	this->GetAudioBuf = DX5_GetAudioBuf;
	this->WaitDone = DX5_WaitDone;
	this->CloseAudio = DX5_CloseAudio;

	this->free = Audio_DeleteDevice;

	return this;
}

AudioBootStrap DSOUND_bootstrap = {
	"dsound", "Win95/98/2000 DirectSound",
	Audio_Available, Audio_CreateDevice
};

static void SetDSerror(const char *function, int code)
{
	static const char *error;
	static char  errbuf[1024];

	errbuf[0] = 0;
	switch (code) {
		case E_NOINTERFACE:
			error = 
		"Unsupported interface\n-- Is DirectX 5.0 or later installed?";
			break;
		case DSERR_ALLOCATED:
			error = "Audio device in use";
			break;
		case DSERR_BADFORMAT:
			error = "Unsupported audio format";
			break;
		case DSERR_BUFFERLOST:
			error = "Mixing buffer was lost";
			break;
		case DSERR_CONTROLUNAVAIL:
			error = "Control requested is not available";
			break;
		case DSERR_INVALIDCALL:
			error = "Invalid call for the current state";
			break;
		case DSERR_INVALIDPARAM:
			error = "Invalid parameter";
			break;
		case DSERR_NODRIVER:
			error = "No audio device found";
			break;
		case DSERR_OUTOFMEMORY:
			error = "Out of memory";
			break;
		case DSERR_PRIOLEVELNEEDED:
			error = "Caller doesn't have priority";
			break;
		case DSERR_UNSUPPORTED:
			error = "Function not supported";
			break;
		default:
			SDL_snprintf(errbuf, SDL_arraysize(errbuf),
			         "%s: Unknown DirectSound error: 0x%x",
								function, code);
			break;
	}
	if ( ! errbuf[0] ) {
		SDL_snprintf(errbuf, SDL_arraysize(errbuf), "%s: %s", function, error);
	}
	SDL_SetError("%s", errbuf);
	return;
}

/* DirectSound needs to be associated with a window */
static HWND mainwin = NULL;
/* */
void DX5_SoundFocus(HWND hwnd)
{
	mainwin = hwnd;
}

static void DX5_ThreadInit(_THIS)
{
	SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_HIGHEST);
}

static void DX5_WaitAudio_BusyWait(_THIS)
{
	DWORD status;
	DWORD cursor, junk;
	HRESULT result;

	/* Semi-busy wait, since we have no way of getting play notification
	   on a primary mixing buffer located in hardware (DirectX 5.0)
	*/
	result = IDirectSoundBuffer_GetCurrentPosition(mixbuf, &junk, &cursor);
	if ( result != DS_OK ) {
		if ( result == DSERR_BUFFERLOST ) {
			IDirectSoundBuffer_Restore(mixbuf);
		}
#ifdef DEBUG_SOUND
		SetDSerror("DirectSound GetCurrentPosition", result);
#endif
		return;
	}

	while ( (cursor/mixlen) == lastchunk ) {
		/* FIXME: find out how much time is left and sleep that long */
		SDL_Delay(1);

		/* Try to restore a lost sound buffer */
		IDirectSoundBuffer_GetStatus(mixbuf, &status);
		if ( (status&DSBSTATUS_BUFFERLOST) ) {
			IDirectSoundBuffer_Restore(mixbuf);
			IDirectSoundBuffer_GetStatus(mixbuf, &status);
			if ( (status&DSBSTATUS_BUFFERLOST) ) {
				break;
			}
		}
		if ( ! (status&DSBSTATUS_PLAYING) ) {
			result = IDirectSoundBuffer_Play(mixbuf, 0, 0, DSBPLAY_LOOPING);
			if ( result == DS_OK ) {
				continue;
			}
#ifdef DEBUG_SOUND
			SetDSerror("DirectSound Play", result);
#endif
			return;
		}

		/* Find out where we are playing */
		result = IDirectSoundBuffer_GetCurrentPosition(mixbuf,
								&junk, &cursor);
		if ( result != DS_OK ) {
			SetDSerror("DirectSound GetCurrentPosition", result);
			return;
		}
	}
}

#ifdef USE_POSITION_NOTIFY
static void DX6_WaitAudio_EventWait(_THIS)
{
	DWORD status;
	HRESULT result;

	/* Try to restore a lost sound buffer */
	IDirectSoundBuffer_GetStatus(mixbuf, &status);
	if ( (status&DSBSTATUS_BUFFERLOST) ) {
		IDirectSoundBuffer_Restore(mixbuf);
		IDirectSoundBuffer_GetStatus(mixbuf, &status);
		if ( (status&DSBSTATUS_BUFFERLOST) ) {
			return;
		}
	}
	if ( ! (status&DSBSTATUS_PLAYING) ) {
		result = IDirectSoundBuffer_Play(mixbuf, 0, 0, DSBPLAY_LOOPING);
		if ( result != DS_OK ) {
#ifdef DEBUG_SOUND
			SetDSerror("DirectSound Play", result);
#endif
			return;
		}
	}
	WaitForSingleObject(audio_event, INFINITE);
}
#endif /* USE_POSITION_NOTIFY */

static void DX5_PlayAudio(_THIS)
{
	/* Unlock the buffer, allowing it to play */
	if ( locked_buf ) {
		IDirectSoundBuffer_Unlock(mixbuf, locked_buf, mixlen, NULL, 0);
	}

}

static Uint8 *DX5_GetAudioBuf(_THIS)
{
	DWORD   cursor, junk;
	HRESULT result;
	DWORD   rawlen;

	/* Figure out which blocks to fill next */
	locked_buf = NULL;
	result = IDirectSoundBuffer_GetCurrentPosition(mixbuf, &junk, &cursor);
	if ( result == DSERR_BUFFERLOST ) {
		IDirectSoundBuffer_Restore(mixbuf);
		result = IDirectSoundBuffer_GetCurrentPosition(mixbuf,
								&junk, &cursor);
	}
	if ( result != DS_OK ) {
		SetDSerror("DirectSound GetCurrentPosition", result);
		return(NULL);
	}
	cursor /= mixlen;
#ifdef DEBUG_SOUND
	/* Detect audio dropouts */
	{ DWORD spot = cursor;
	  if ( spot < lastchunk ) {
	    spot += NUM_BUFFERS;
	  }
	  if ( spot > lastchunk+1 ) {
	    fprintf(stderr, "Audio dropout, missed %d fragments\n",
	            (spot - (lastchunk+1)));
	  }
	}
#endif
	lastchunk = cursor;
	cursor = (cursor+1)%NUM_BUFFERS;
	cursor *= mixlen;

	/* Lock the audio buffer */
	result = IDirectSoundBuffer_Lock(mixbuf, cursor, mixlen,
				(LPVOID *)&locked_buf, &rawlen, NULL, &junk, 0);
	if ( result == DSERR_BUFFERLOST ) {
		IDirectSoundBuffer_Restore(mixbuf);
		result = IDirectSoundBuffer_Lock(mixbuf, cursor, mixlen,
				(LPVOID *)&locked_buf, &rawlen, NULL, &junk, 0);
	}
	if ( result != DS_OK ) {
		SetDSerror("DirectSound Lock", result);
		return(NULL);
	}
	return(locked_buf);
}

static void DX5_WaitDone(_THIS)
{
	Uint8 *stream;

	/* Wait for the playing chunk to finish */
	stream = this->GetAudioBuf(this);
	if ( stream != NULL ) {
		SDL_memset(stream, silence, mixlen);
		this->PlayAudio(this);
	}
	this->WaitAudio(this);

	/* Stop the looping sound buffer */
	IDirectSoundBuffer_Stop(mixbuf);
}

static void DX5_CloseAudio(_THIS)
{
	if ( sound != NULL ) {
		if ( mixbuf != NULL ) {
			/* Clean up the audio buffer */
			IDirectSoundBuffer_Release(mixbuf);
			mixbuf = NULL;
		}
		if ( audio_event != NULL ) {
			CloseHandle(audio_event);
			audio_event = NULL;
		}
		IDirectSound_Release(sound);
		sound = NULL;
	}
}

#ifdef USE_PRIMARY_BUFFER
/* This function tries to create a primary audio buffer, and returns the
   number of audio chunks available in the created buffer.
*/
static int CreatePrimary(LPDIRECTSOUND sndObj, HWND focus, 
	LPDIRECTSOUNDBUFFER *sndbuf, WAVEFORMATEX *wavefmt, Uint32 chunksize)
{
	HRESULT result;
	DSBUFFERDESC format;
	DSBCAPS caps;
	int numchunks;

	/* Try to set primary mixing privileges */
	result = IDirectSound_SetCooperativeLevel(sndObj, focus,
							DSSCL_WRITEPRIMARY);
	if ( result != DS_OK ) {
#ifdef DEBUG_SOUND
		SetDSerror("DirectSound SetCooperativeLevel", result);
#endif
		return(-1);
	}

	/* Try to create the primary buffer */
	SDL_memset(&format, 0, sizeof(format));
	format.dwSize = sizeof(format);
	format.dwFlags=(DSBCAPS_PRIMARYBUFFER|DSBCAPS_GETCURRENTPOSITION2);
	format.dwFlags |= DSBCAPS_STICKYFOCUS;
#ifdef USE_POSITION_NOTIFY
	format.dwFlags |= DSBCAPS_CTRLPOSITIONNOTIFY;
#endif
	result = IDirectSound_CreateSoundBuffer(sndObj, &format, sndbuf, NULL);
	if ( result != DS_OK ) {
#ifdef DEBUG_SOUND
		SetDSerror("DirectSound CreateSoundBuffer", result);
#endif
		return(-1);
	}

	/* Check the size of the fragment buffer */
	SDL_memset(&caps, 0, sizeof(caps));
	caps.dwSize = sizeof(caps);
	result = IDirectSoundBuffer_GetCaps(*sndbuf, &caps);
	if ( result != DS_OK ) {
#ifdef DEBUG_SOUND
		SetDSerror("DirectSound GetCaps", result);
#endif
		IDirectSoundBuffer_Release(*sndbuf);
		return(-1);
	}
	if ( (chunksize > caps.dwBufferBytes) ||
				((caps.dwBufferBytes%chunksize) != 0) ) {
		/* The primary buffer size is not a multiple of 'chunksize'
		   -- this hopefully doesn't happen when 'chunksize' is a 
		      power of 2.
		*/
		IDirectSoundBuffer_Release(*sndbuf);
		SDL_SetError(
"Primary buffer size is: %d, cannot break it into chunks of %d bytes\n",
					caps.dwBufferBytes, chunksize);
		return(-1);
	}
	numchunks = (caps.dwBufferBytes/chunksize);

	/* Set the primary audio format */
	result = IDirectSoundBuffer_SetFormat(*sndbuf, wavefmt);
	if ( result != DS_OK ) {
#ifdef DEBUG_SOUND
		SetDSerror("DirectSound SetFormat", result);
#endif
		IDirectSoundBuffer_Release(*sndbuf);
		return(-1);
	}
	return(numchunks);
}
#endif /* USE_PRIMARY_BUFFER */

/* This function tries to create a secondary audio buffer, and returns the
   number of audio chunks available in the created buffer.
*/
static int CreateSecondary(LPDIRECTSOUND sndObj, HWND focus,
	LPDIRECTSOUNDBUFFER *sndbuf, WAVEFORMATEX *wavefmt, Uint32 chunksize)
{
	const int numchunks = 8;
	HRESULT result;
	DSBUFFERDESC format;
	LPVOID pvAudioPtr1, pvAudioPtr2;
	DWORD  dwAudioBytes1, dwAudioBytes2;

	/* Try to set primary mixing privileges */
	if ( focus ) {
		result = IDirectSound_SetCooperativeLevel(sndObj,
					focus, DSSCL_PRIORITY);
	} else {
		result = IDirectSound_SetCooperativeLevel(sndObj,
					GetDesktopWindow(), DSSCL_NORMAL);
	}
	if ( result != DS_OK ) {
#ifdef DEBUG_SOUND
		SetDSerror("DirectSound SetCooperativeLevel", result);
#endif
		return(-1);
	}

	/* Try to create the secondary buffer */
	SDL_memset(&format, 0, sizeof(format));
	format.dwSize = sizeof(format);
	format.dwFlags = DSBCAPS_GETCURRENTPOSITION2;
#ifdef USE_POSITION_NOTIFY
	format.dwFlags |= DSBCAPS_CTRLPOSITIONNOTIFY;
#endif
	if ( ! focus ) {
		format.dwFlags |= DSBCAPS_GLOBALFOCUS;
	} else {
		format.dwFlags |= DSBCAPS_STICKYFOCUS;
	}
	format.dwBufferBytes = numchunks*chunksize;
	if ( (format.dwBufferBytes < DSBSIZE_MIN) ||
	     (format.dwBufferBytes > DSBSIZE_MAX) ) {
		SDL_SetError("Sound buffer size must be between %d and %d",
				DSBSIZE_MIN/numchunks, DSBSIZE_MAX/numchunks);
		return(-1);
	}
	format.dwReserved = 0;
	format.lpwfxFormat = wavefmt;
	result = IDirectSound_CreateSoundBuffer(sndObj, &format, sndbuf, NULL);
	if ( result != DS_OK ) {
		SetDSerror("DirectSound CreateSoundBuffer", result);
		return(-1);
	}
	IDirectSoundBuffer_SetFormat(*sndbuf, wavefmt);

	/* Silence the initial audio buffer */
	result = IDirectSoundBuffer_Lock(*sndbuf, 0, format.dwBufferBytes,
	                                 (LPVOID *)&pvAudioPtr1, &dwAudioBytes1,
	                                 (LPVOID *)&pvAudioPtr2, &dwAudioBytes2,
	                                 DSBLOCK_ENTIREBUFFER);
	if ( result == DS_OK ) {
		if ( wavefmt->wBitsPerSample == 8 ) {
			SDL_memset(pvAudioPtr1, 0x80, dwAudioBytes1);
		} else {
			SDL_memset(pvAudioPtr1, 0x00, dwAudioBytes1);
		}
		IDirectSoundBuffer_Unlock(*sndbuf,
		                          (LPVOID)pvAudioPtr1, dwAudioBytes1,
		                          (LPVOID)pvAudioPtr2, dwAudioBytes2);
	}

	/* We're ready to go */
	return(numchunks);
}

/* This function tries to set position notify events on the mixing buffer */
#ifdef USE_POSITION_NOTIFY
static int CreateAudioEvent(_THIS)
{
	LPDIRECTSOUNDNOTIFY notify;
	DSBPOSITIONNOTIFY *notify_positions;
	int i, retval;
	HRESULT result;

	/* Default to fail on exit */
	retval = -1;
	notify = NULL;

	/* Query for the interface */
	result = IDirectSoundBuffer_QueryInterface(mixbuf,
			&IID_IDirectSoundNotify, (void *)&notify);
	if ( result != DS_OK ) {
		goto done;
	}

	/* Allocate the notify structures */
	notify_positions = (DSBPOSITIONNOTIFY *)SDL_malloc(NUM_BUFFERS*
					sizeof(*notify_positions));
	if ( notify_positions == NULL ) {
		goto done;
	}

	/* Create the notify event */
	audio_event = CreateEvent(NULL, FALSE, FALSE, NULL);
	if ( audio_event == NULL ) {
		goto done;
	}

	/* Set up the notify structures */
	for ( i=0; i<NUM_BUFFERS; ++i ) {
		notify_positions[i].dwOffset = i*mixlen;
		notify_positions[i].hEventNotify = audio_event;
	}
	result = IDirectSoundNotify_SetNotificationPositions(notify,
					NUM_BUFFERS, notify_positions);
	if ( result == DS_OK ) {
		retval = 0;
	}
done:
	if ( notify != NULL ) {
		IDirectSoundNotify_Release(notify);
	}
	return(retval);
}
#endif /* USE_POSITION_NOTIFY */

static int DX5_OpenAudio(_THIS, SDL_AudioSpec *spec)
{
	HRESULT      result;
	WAVEFORMATEX waveformat;

	/* Set basic WAVE format parameters */
	SDL_memset(&waveformat, 0, sizeof(waveformat));
	waveformat.wFormatTag = WAVE_FORMAT_PCM;

	/* Determine the audio parameters from the AudioSpec */
	switch ( spec->format & 0xFF ) {
		case 8:
			/* Unsigned 8 bit audio data */
			spec->format = AUDIO_U8;
			silence = 0x80;
			waveformat.wBitsPerSample = 8;
			break;
		case 16:
			/* Signed 16 bit audio data */
			spec->format = AUDIO_S16;
			silence = 0x00;
			waveformat.wBitsPerSample = 16;
			break;
		default:
			SDL_SetError("Unsupported audio format");
			return(-1);
	}
	waveformat.nChannels = spec->channels;
	waveformat.nSamplesPerSec = spec->freq;
	waveformat.nBlockAlign =
		waveformat.nChannels * (waveformat.wBitsPerSample/8);
	waveformat.nAvgBytesPerSec = 
		waveformat.nSamplesPerSec * waveformat.nBlockAlign;

	/* Update the fragment size as size in bytes */
	SDL_CalculateAudioSpec(spec);

	/* Open the audio device */
	result = DSoundCreate(NULL, &sound, NULL);
	if ( result != DS_OK ) {
		SetDSerror("DirectSoundCreate", result);
		return(-1);
	}

	/* Create the audio buffer to which we write */
	NUM_BUFFERS = -1;
#ifdef USE_PRIMARY_BUFFER
	if ( mainwin ) {
		NUM_BUFFERS = CreatePrimary(sound, mainwin, &mixbuf,
						&waveformat, spec->size);
	}
#endif /* USE_PRIMARY_BUFFER */
	if ( NUM_BUFFERS < 0 ) {
		NUM_BUFFERS = CreateSecondary(sound, mainwin, &mixbuf,
						&waveformat, spec->size);
		if ( NUM_BUFFERS < 0 ) {
			return(-1);
		}
#ifdef DEBUG_SOUND
		fprintf(stderr, "Using secondary audio buffer\n");
#endif
	}
#ifdef DEBUG_SOUND
	else
		fprintf(stderr, "Using primary audio buffer\n");
#endif

	/* The buffer will auto-start playing in DX5_WaitAudio() */
	lastchunk = 0;
	mixlen = spec->size;

#ifdef USE_POSITION_NOTIFY
	/* See if we can use DirectX 6 event notification */
	if ( CreateAudioEvent(this) == 0 ) {
		this->WaitAudio = DX6_WaitAudio_EventWait;
	} else {
		this->WaitAudio = DX5_WaitAudio_BusyWait;
	}
#endif
	return(0);
}

