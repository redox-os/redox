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

/* This file takes care of command line argument parsing, and stdio redirection
   in the MacOS environment. (stdio/stderr is *not* directed for Mach-O builds)
 */

#if defined(__APPLE__) && defined(__MACH__)
#include <Carbon/Carbon.h>
#elif TARGET_API_MAC_CARBON && (UNIVERSAL_INTERFACES_VERSION > 0x0335)
#include <Carbon.h>
#else
#include <Dialogs.h>
#include <Fonts.h>
#include <Events.h>
#include <Resources.h>
#include <Folders.h>
#endif

/* Include the SDL main definition header */
#include "SDL.h"
#include "SDL_main.h"
#ifdef main
#undef main
#endif

#if !(defined(__APPLE__) && defined(__MACH__))
/* The standard output files */
#define STDOUT_FILE	"stdout.txt"
#define STDERR_FILE	"stderr.txt"
#endif

#if !defined(__MWERKS__) && !TARGET_API_MAC_CARBON
	/* In MPW, the qd global has been removed from the libraries */
	QDGlobals qd;
#endif

/* Structure for keeping prefs in 1 variable */
typedef struct {
    Str255  command_line;
    Str255  video_driver_name;
    Boolean output_to_file;
}  PrefsRecord;

/* See if the command key is held down at startup */
static Boolean CommandKeyIsDown(void)
{
	KeyMap  theKeyMap;

	GetKeys(theKeyMap);

	if (((unsigned char *) theKeyMap)[6] & 0x80) {
		return(true);
	}
	return(false);
}

#if !(defined(__APPLE__) && defined(__MACH__))

/* Parse a command line buffer into arguments */
static int ParseCommandLine(char *cmdline, char **argv)
{
	char *bufp;
	int argc;

	argc = 0;
	for ( bufp = cmdline; *bufp; ) {
		/* Skip leading whitespace */
		while ( SDL_isspace(*bufp) ) {
			++bufp;
		}
		/* Skip over argument */
		if ( *bufp == '"' ) {
			++bufp;
			if ( *bufp ) {
				if ( argv ) {
					argv[argc] = bufp;
				}
				++argc;
			}
			/* Skip over word */
			while ( *bufp && (*bufp != '"') ) {
				++bufp;
			}
		} else {
			if ( *bufp ) {
				if ( argv ) {
					argv[argc] = bufp;
				}
				++argc;
			}
			/* Skip over word */
			while ( *bufp && ! SDL_isspace(*bufp) ) {
				++bufp;
			}
		}
		if ( *bufp ) {
			if ( argv ) {
				*bufp = '\0';
			}
			++bufp;
		}
	}
	if ( argv ) {
		argv[argc] = NULL;
	}
	return(argc);
}

/* Remove the output files if there was no output written */
static void cleanup_output(void)
{
	FILE *file;
	int empty;

	/* Flush the output in case anything is queued */
	fclose(stdout);
	fclose(stderr);

	/* See if the files have any output in them */
	file = fopen(STDOUT_FILE, "rb");
	if ( file ) {
		empty = (fgetc(file) == EOF) ? 1 : 0;
		fclose(file);
		if ( empty ) {
			remove(STDOUT_FILE);
		}
	}
	file = fopen(STDERR_FILE, "rb");
	if ( file ) {
		empty = (fgetc(file) == EOF) ? 1 : 0;
		fclose(file);
		if ( empty ) {
			remove(STDERR_FILE);
		}
	}
}

#endif //!(defined(__APPLE__) && defined(__MACH__))

static int getCurrentAppName (StrFileName name) {
	
    ProcessSerialNumber process;
    ProcessInfoRec      process_info;
    FSSpec              process_fsp;
    
    process.highLongOfPSN = 0;
    process.lowLongOfPSN  = kCurrentProcess;
    process_info.processInfoLength = sizeof (process_info);
    process_info.processName    = NULL;
    process_info.processAppSpec = &process_fsp;
    
    if ( noErr != GetProcessInformation (&process, &process_info) )
       return 0;
    
    SDL_memcpy(name, process_fsp.name, process_fsp.name[0] + 1);
    return 1;
}

static int getPrefsFile (FSSpec *prefs_fsp, int create) {

    /* The prefs file name is the application name, possibly truncated, */
    /* plus " Preferences */
    
    #define  SUFFIX   " Preferences"
    #define  MAX_NAME 19             /* 31 - strlen (SUFFIX) */
    
    short  volume_ref_number;
    long   directory_id;
    StrFileName  prefs_name;
    StrFileName  app_name;
    
    /* Get Preferences folder - works with Multiple Users */
    if ( noErr != FindFolder ( kOnSystemDisk, kPreferencesFolderType, kDontCreateFolder,
                               &volume_ref_number, &directory_id) )
        exit (-1);
    
    if ( ! getCurrentAppName (app_name) )
        exit (-1);
    
    /* Truncate if name is too long */
    if (app_name[0] > MAX_NAME )
        app_name[0] = MAX_NAME;
        
    SDL_memcpy(prefs_name + 1, app_name + 1, app_name[0]);    
    SDL_memcpy(prefs_name + app_name[0] + 1, SUFFIX, strlen (SUFFIX));
    prefs_name[0] = app_name[0] + strlen (SUFFIX);
   
    /* Make the file spec for prefs file */
    if ( noErr != FSMakeFSSpec (volume_ref_number, directory_id, prefs_name, prefs_fsp) ) {
        if ( !create )
            return 0;
        else {
            /* Create the prefs file */
            SDL_memcpy(prefs_fsp->name, prefs_name, prefs_name[0] + 1);
            prefs_fsp->parID   = directory_id;
            prefs_fsp->vRefNum = volume_ref_number;
                
            FSpCreateResFile (prefs_fsp, 0x3f3f3f3f, 'pref', 0); // '????' parsed as trigraph
            
            if ( noErr != ResError () )
                return 0;
        }
     }
    return 1;
}

static int readPrefsResource (PrefsRecord *prefs) {
    
    Handle prefs_handle;
    
    prefs_handle = Get1Resource( 'CLne', 128 );

	if (prefs_handle != NULL) {
		int offset = 0;
//		int j      = 0;
		
		HLock(prefs_handle);
		
		/* Get command line string */	
		SDL_memcpy(prefs->command_line, *prefs_handle, (*prefs_handle)[0]+1);

		/* Get video driver name */
		offset += (*prefs_handle)[0] + 1;	
		SDL_memcpy(prefs->video_driver_name, *prefs_handle + offset, (*prefs_handle)[offset] + 1);		
		
		/* Get save-to-file option (1 or 0) */
		offset += (*prefs_handle)[offset] + 1;
		prefs->output_to_file = (*prefs_handle)[offset];
		
		ReleaseResource( prefs_handle );
    
        return ResError() == noErr;
    }

    return 0;
}

static int writePrefsResource (PrefsRecord *prefs, short resource_file) {

    Handle prefs_handle;
    
    UseResFile (resource_file);
    
    prefs_handle = Get1Resource ( 'CLne', 128 );
    if (prefs_handle != NULL)
        RemoveResource (prefs_handle);
    
    prefs_handle = NewHandle ( prefs->command_line[0] + prefs->video_driver_name[0] + 4 );
    if (prefs_handle != NULL) {
    
        int offset;
        
        HLock (prefs_handle);
        
        /* Command line text */
        offset = 0;
        SDL_memcpy(*prefs_handle, prefs->command_line, prefs->command_line[0] + 1);
        
        /* Video driver name */
        offset += prefs->command_line[0] + 1;
        SDL_memcpy(*prefs_handle + offset, prefs->video_driver_name, prefs->video_driver_name[0] + 1);
        
        /* Output-to-file option */
        offset += prefs->video_driver_name[0] + 1;
        *( *((char**)prefs_handle) + offset)     = (char)prefs->output_to_file;
        *( *((char**)prefs_handle) + offset + 1) = 0;
              
        AddResource   (prefs_handle, 'CLne', 128, "\pCommand Line");
        WriteResource (prefs_handle);
        UpdateResFile (resource_file);
        DisposeHandle (prefs_handle);
        
        return ResError() == noErr;
    }
    
    return 0;
}

static int readPreferences (PrefsRecord *prefs) {

    int    no_error = 1;
    FSSpec prefs_fsp;

    /* Check for prefs file first */
    if ( getPrefsFile (&prefs_fsp, 0) ) {
    
        short  prefs_resource;
        
        prefs_resource = FSpOpenResFile (&prefs_fsp, fsRdPerm);
        if ( prefs_resource == -1 ) /* this shouldn't happen, but... */
            return 0;
    
        UseResFile   (prefs_resource);
        no_error = readPrefsResource (prefs);     
        CloseResFile (prefs_resource);
    }
    
    /* Fall back to application's resource fork (reading only, so this is safe) */
    else {
    
          no_error = readPrefsResource (prefs);
     }

    return no_error;
}

static int writePreferences (PrefsRecord *prefs) {
    
    int    no_error = 1;
    FSSpec prefs_fsp;
    
    /* Get prefs file, create if it doesn't exist */
    if ( getPrefsFile (&prefs_fsp, 1) ) {
    
        short  prefs_resource;
        
        prefs_resource = FSpOpenResFile (&prefs_fsp, fsRdWrPerm);
        if (prefs_resource == -1)
            return 0;
        no_error = writePrefsResource (prefs, prefs_resource);
        CloseResFile (prefs_resource);
    }
    
    return no_error;
}

/* This is where execution begins */
int main(int argc, char *argv[])
{

#if !(defined(__APPLE__) && defined(__MACH__))
#pragma unused(argc, argv)
#endif
	
#define DEFAULT_ARGS "\p"                /* pascal string for default args */
#define DEFAULT_VIDEO_DRIVER "\ptoolbox" /* pascal string for default video driver name */	
#define DEFAULT_OUTPUT_TO_FILE 1         /* 1 == output to file, 0 == no output */

#define VIDEO_ID_DRAWSPROCKET 1          /* these correspond to popup menu choices */
#define VIDEO_ID_TOOLBOX      2

    PrefsRecord prefs = { DEFAULT_ARGS, DEFAULT_VIDEO_DRIVER, DEFAULT_OUTPUT_TO_FILE }; 
	
#if !(defined(__APPLE__) && defined(__MACH__))
	int     nargs;
	char   **args;
	char   *commandLine;
	
	StrFileName  appNameText;
#endif
	int     videodriver     = VIDEO_ID_TOOLBOX;
    int     settingsChanged = 0;
    
    long	i;

	/* Kyle's SDL command-line dialog code ... */
#if !TARGET_API_MAC_CARBON
	InitGraf    (&qd.thePort);
	InitFonts   ();
	InitWindows ();
	InitMenus   ();
	InitDialogs (nil);
#endif
	InitCursor ();
	FlushEvents(everyEvent,0);
#if !TARGET_API_MAC_CARBON
	MaxApplZone ();
#endif
	MoreMasters ();
	MoreMasters ();
#if 0
	/* Intialize SDL, and put up a dialog if we fail */
	if ( SDL_Init (0) < 0 ) {

#define kErr_OK		1
#define kErr_Text	2

        DialogPtr errorDialog;
        short	  dummyType;
    	Rect	  dummyRect;
	    Handle    dummyHandle;
	    short     itemHit;
	
		errorDialog = GetNewDialog (1001, nil, (WindowPtr)-1);
		if (errorDialog == NULL)
		    return -1;
		DrawDialog (errorDialog);
		
		GetDialogItem (errorDialog, kErr_Text, &dummyType, &dummyHandle, &dummyRect);
		SetDialogItemText (dummyHandle, "\pError Initializing SDL");
		
#if TARGET_API_MAC_CARBON
		SetPort (GetDialogPort(errorDialog));
#else
		SetPort (errorDialog);
#endif
		do {
			ModalDialog (nil, &itemHit);
		} while (itemHit != kErr_OK);
		
		DisposeDialog (errorDialog);
		exit (-1);
	}
	atexit(cleanup_output);
	atexit(SDL_Quit);
#endif

/* Set up SDL's QuickDraw environment  */
#if !TARGET_API_MAC_CARBON
	SDL_InitQuickDraw(&qd);
#endif

	 if ( readPreferences (&prefs) ) {
		
        if (SDL_memcmp(prefs.video_driver_name+1, "DSp", 3) == 0)
            videodriver = 1;
        else if (SDL_memcmp(prefs.video_driver_name+1, "toolbox", 7) == 0)
            videodriver = 2;
	 }
	 	
	if ( CommandKeyIsDown() ) {

#define kCL_OK		1
#define kCL_Cancel	2
#define kCL_Text	3
#define kCL_File	4
#define kCL_Video   6
       
        DialogPtr commandDialog;
        short	  dummyType;
        Rect	  dummyRect;
        Handle    dummyHandle;
        short     itemHit;
   #if TARGET_API_MAC_CARBON
        ControlRef control;
   #endif
        
        /* Assume that they will change settings, rather than do exhaustive check */
        settingsChanged = 1;
        
        /* Create dialog and display it */
        commandDialog = GetNewDialog (1000, nil, (WindowPtr)-1);
    #if TARGET_API_MAC_CARBON
        SetPort ( GetDialogPort(commandDialog) );
    #else
        SetPort (commandDialog);
     #endif
           
        /* Setup controls */
    #if TARGET_API_MAC_CARBON
        GetDialogItemAsControl(commandDialog, kCL_File, &control);
        SetControlValue (control, prefs.output_to_file);
    #else
        GetDialogItem   (commandDialog, kCL_File, &dummyType, &dummyHandle, &dummyRect); /* MJS */
        SetControlValue ((ControlHandle)dummyHandle, prefs.output_to_file );
    #endif

        GetDialogItem     (commandDialog, kCL_Text, &dummyType, &dummyHandle, &dummyRect);
        SetDialogItemText (dummyHandle, prefs.command_line);

    #if TARGET_API_MAC_CARBON
        GetDialogItemAsControl(commandDialog, kCL_Video, &control);
        SetControlValue (control, videodriver);
   #else
        GetDialogItem   (commandDialog, kCL_Video, &dummyType, &dummyHandle, &dummyRect);
        SetControlValue ((ControlRef)dummyHandle, videodriver);
     #endif

        SetDialogDefaultItem (commandDialog, kCL_OK);
        SetDialogCancelItem  (commandDialog, kCL_Cancel);

        do {
        		
        	ModalDialog(nil, &itemHit); /* wait for user response */
            
            /* Toggle command-line output checkbox */	
        	if ( itemHit == kCL_File ) {
        #if TARGET_API_MAC_CARBON
        		GetDialogItemAsControl(commandDialog, kCL_File, &control);
        		SetControlValue (control, !GetControlValue(control));
        #else
        		GetDialogItem(commandDialog, kCL_File, &dummyType, &dummyHandle, &dummyRect); /* MJS */
        		SetControlValue((ControlHandle)dummyHandle, !GetControlValue((ControlHandle)dummyHandle) );
        #endif
        	}

        } while (itemHit != kCL_OK && itemHit != kCL_Cancel);

        /* Get control values, even if they did not change */
        GetDialogItem     (commandDialog, kCL_Text, &dummyType, &dummyHandle, &dummyRect); /* MJS */
        GetDialogItemText (dummyHandle, prefs.command_line);

    #if TARGET_API_MAC_CARBON
        GetDialogItemAsControl(commandDialog, kCL_File, &control);
        prefs.output_to_file = GetControlValue(control);
	#else
        GetDialogItem (commandDialog, kCL_File, &dummyType, &dummyHandle, &dummyRect); /* MJS */
        prefs.output_to_file = GetControlValue ((ControlHandle)dummyHandle);
 	#endif

    #if TARGET_API_MAC_CARBON
        GetDialogItemAsControl(commandDialog, kCL_Video, &control);
        videodriver = GetControlValue(control);
    #else
        GetDialogItem (commandDialog, kCL_Video, &dummyType, &dummyHandle, &dummyRect);
        videodriver = GetControlValue ((ControlRef)dummyHandle);
     #endif

        DisposeDialog (commandDialog);

        if (itemHit == kCL_Cancel ) {
        	exit (0);
        }
	}
    
    /* Set pseudo-environment variables for video driver, update prefs */
	switch ( videodriver ) {
	   case VIDEO_ID_DRAWSPROCKET: 
	      SDL_putenv("SDL_VIDEODRIVER=DSp");
	      SDL_memcpy(prefs.video_driver_name, "\pDSp", 4);
	      break;
	   case VIDEO_ID_TOOLBOX:
	      SDL_putenv("SDL_VIDEODRIVER=toolbox");
	      SDL_memcpy(prefs.video_driver_name, "\ptoolbox", 8);
	      break;
	}

#if !(defined(__APPLE__) && defined(__MACH__))
    /* Redirect standard I/O to files */
	if ( prefs.output_to_file ) {
		freopen (STDOUT_FILE, "w", stdout);
		freopen (STDERR_FILE, "w", stderr);
	} else {
		fclose (stdout);
		fclose (stderr);
	}
#endif
   
    if (settingsChanged) {
        /* Save the prefs, even if they might not have changed (but probably did) */
        if ( ! writePreferences (&prefs) )
            fprintf (stderr, "WARNING: Could not save preferences!\n");
    }
   
#if !(defined(__APPLE__) && defined(__MACH__))
    appNameText[0] = 0;
    getCurrentAppName (appNameText); /* check for error here ? */

    commandLine = (char*) malloc (appNameText[0] + prefs.command_line[0] + 2);
    if ( commandLine == NULL ) {
       exit(-1);
    }

    /* Rather than rewrite ParseCommandLine method, let's replace  */
    /* any spaces in application name with underscores,            */
    /* so that the app name is only 1 argument                     */   
    for (i = 1; i < 1+appNameText[0]; i++)
        if ( appNameText[i] == ' ' ) appNameText[i] = '_';

    /* Copy app name & full command text to command-line C-string */      
    SDL_memcpy(commandLine, appNameText + 1, appNameText[0]);
    commandLine[appNameText[0]] = ' ';
    SDL_memcpy(commandLine + appNameText[0] + 1, prefs.command_line + 1, prefs.command_line[0]);
    commandLine[ appNameText[0] + 1 + prefs.command_line[0] ] = '\0';

    /* Parse C-string into argv and argc */
    nargs = ParseCommandLine (commandLine, NULL);
    args = (char **)malloc((nargs+1)*(sizeof *args));
    if ( args == NULL ) {
		exit(-1);
	}
	ParseCommandLine (commandLine, args);
        
	/* Run the main application code */
	SDL_main(nargs, args);
	free (args);
	free (commandLine);
   
   	/* Remove useless stdout.txt and stderr.txt */
   	cleanup_output ();
#else // defined(__APPLE__) && defined(__MACH__)
	SDL_main(argc, argv);
#endif
   	
	/* Exit cleanly, calling atexit() functions */
	exit (0);    

	/* Never reached, but keeps the compiler quiet */
	return (0);
}
