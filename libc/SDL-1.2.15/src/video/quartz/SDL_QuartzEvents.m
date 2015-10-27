/*
    SDL - Simple DirectMedia Layer
    Copyright (C) 1997-2012  Sam Lantinga

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

#include "SDL_QuartzVideo.h"
#include "SDL_QuartzWM.h"

#include <IOKit/IOMessage.h> /* For wake from sleep detection */
#include <IOKit/pwr_mgt/IOPMLib.h> /* For wake from sleep detection */
#include "SDL_QuartzKeys.h"

/*
 * On Leopard, this is missing from the 64-bit headers
 */
#if defined(__LP64__) && !defined(__POWER__)
/*
 * Workaround for a bug in the 10.5 SDK: By accident, OSService.h does
 * not include Power.h at all when compiling in 64bit mode. This has
 * been fixed in 10.6, but for 10.5, we manually define UsrActivity
 * to ensure compilation works.
 */
#define UsrActivity 1
#endif

/* 
 * In Panther, this header defines device dependent masks for 
 * right side keys. These definitions only exist in Panther, but
 * the header seems to exist at least in Jaguar and probably earlier
 * versions of the OS, so this should't break anything.
 */
#include <IOKit/hidsystem/IOLLEvent.h>
/* 
 * These are not defined before Panther. To keep the code compiling
 * on systems without these, I will define if they don't exist.
 */
#ifndef NX_DEVICERCTLKEYMASK
    #define NX_DEVICELCTLKEYMASK    0x00000001
#endif
#ifndef NX_DEVICELSHIFTKEYMASK
    #define NX_DEVICELSHIFTKEYMASK  0x00000002
#endif
#ifndef NX_DEVICERSHIFTKEYMASK
    #define NX_DEVICERSHIFTKEYMASK  0x00000004
#endif
#ifndef NX_DEVICELCMDKEYMASK
    #define NX_DEVICELCMDKEYMASK    0x00000008
#endif
#ifndef NX_DEVICERCMDKEYMASK
    #define NX_DEVICERCMDKEYMASK    0x00000010
#endif
#ifndef NX_DEVICELALTKEYMASK
    #define NX_DEVICELALTKEYMASK    0x00000020
#endif
#ifndef NX_DEVICERALTKEYMASK
    #define NX_DEVICERALTKEYMASK    0x00000040
#endif
#ifndef NX_DEVICERCTLKEYMASK
    #define NX_DEVICERCTLKEYMASK    0x00002000
#endif

void     QZ_InitOSKeymap (_THIS) {
    BOOL saw_layout = NO;
    UInt32 state;
    UInt32 value;
    Uint16 i;
    int world = SDLK_WORLD_0;

    for ( i=0; i<SDL_TABLESIZE(keymap); ++i )
        keymap[i] = SDLK_UNKNOWN;

    /* This keymap is almost exactly the same as the OS 9 one */
    keymap[QZ_ESCAPE] = SDLK_ESCAPE;
    keymap[QZ_F1] = SDLK_F1;
    keymap[QZ_F2] = SDLK_F2;
    keymap[QZ_F3] = SDLK_F3;
    keymap[QZ_F4] = SDLK_F4;
    keymap[QZ_F5] = SDLK_F5;
    keymap[QZ_F6] = SDLK_F6;
    keymap[QZ_F7] = SDLK_F7;
    keymap[QZ_F8] = SDLK_F8;
    keymap[QZ_F9] = SDLK_F9;
    keymap[QZ_F10] = SDLK_F10;
    keymap[QZ_F11] = SDLK_F11;
    keymap[QZ_F12] = SDLK_F12;
    keymap[QZ_F13] = SDLK_F13;
    keymap[QZ_F14] = SDLK_F14;
    keymap[QZ_F15] = SDLK_F15;
/*
    keymap[QZ_PRINT] = SDLK_PRINT;
    keymap[QZ_SCROLLOCK] = SDLK_SCROLLOCK;
    keymap[QZ_PAUSE] = SDLK_PAUSE;
*/
    keymap[QZ_POWER] = SDLK_POWER;
    keymap[QZ_BACKQUOTE] = SDLK_BACKQUOTE;
    keymap[QZ_1] = SDLK_1;
    keymap[QZ_2] = SDLK_2;
    keymap[QZ_3] = SDLK_3;
    keymap[QZ_4] = SDLK_4;
    keymap[QZ_5] = SDLK_5;
    keymap[QZ_6] = SDLK_6;
    keymap[QZ_7] = SDLK_7;
    keymap[QZ_8] = SDLK_8;
    keymap[QZ_9] = SDLK_9;
    keymap[QZ_0] = SDLK_0;
    keymap[QZ_MINUS] = SDLK_MINUS;
    keymap[QZ_EQUALS] = SDLK_EQUALS;
    keymap[QZ_BACKSPACE] = SDLK_BACKSPACE;
    keymap[QZ_INSERT] = SDLK_INSERT;
    keymap[QZ_HOME] = SDLK_HOME;
    keymap[QZ_PAGEUP] = SDLK_PAGEUP;
    keymap[QZ_NUMLOCK] = SDLK_NUMLOCK;
    keymap[QZ_KP_EQUALS] = SDLK_KP_EQUALS;
    keymap[QZ_KP_DIVIDE] = SDLK_KP_DIVIDE;
    keymap[QZ_KP_MULTIPLY] = SDLK_KP_MULTIPLY;
    keymap[QZ_TAB] = SDLK_TAB;
    keymap[QZ_q] = SDLK_q;
    keymap[QZ_w] = SDLK_w;
    keymap[QZ_e] = SDLK_e;
    keymap[QZ_r] = SDLK_r;
    keymap[QZ_t] = SDLK_t;
    keymap[QZ_y] = SDLK_y;
    keymap[QZ_u] = SDLK_u;
    keymap[QZ_i] = SDLK_i;
    keymap[QZ_o] = SDLK_o;
    keymap[QZ_p] = SDLK_p;
    keymap[QZ_LEFTBRACKET] = SDLK_LEFTBRACKET;
    keymap[QZ_RIGHTBRACKET] = SDLK_RIGHTBRACKET;
    keymap[QZ_BACKSLASH] = SDLK_BACKSLASH;
    keymap[QZ_DELETE] = SDLK_DELETE;
    keymap[QZ_END] = SDLK_END;
    keymap[QZ_PAGEDOWN] = SDLK_PAGEDOWN;
    keymap[QZ_KP7] = SDLK_KP7;
    keymap[QZ_KP8] = SDLK_KP8;
    keymap[QZ_KP9] = SDLK_KP9;
    keymap[QZ_KP_MINUS] = SDLK_KP_MINUS;
    keymap[QZ_CAPSLOCK] = SDLK_CAPSLOCK;
    keymap[QZ_a] = SDLK_a;
    keymap[QZ_s] = SDLK_s;
    keymap[QZ_d] = SDLK_d;
    keymap[QZ_f] = SDLK_f;
    keymap[QZ_g] = SDLK_g;
    keymap[QZ_h] = SDLK_h;
    keymap[QZ_j] = SDLK_j;
    keymap[QZ_k] = SDLK_k;
    keymap[QZ_l] = SDLK_l;
    keymap[QZ_SEMICOLON] = SDLK_SEMICOLON;
    keymap[QZ_QUOTE] = SDLK_QUOTE;
    keymap[QZ_RETURN] = SDLK_RETURN;
    keymap[QZ_KP4] = SDLK_KP4;
    keymap[QZ_KP5] = SDLK_KP5;
    keymap[QZ_KP6] = SDLK_KP6;
    keymap[QZ_KP_PLUS] = SDLK_KP_PLUS;
    keymap[QZ_LSHIFT] = SDLK_LSHIFT;
    keymap[QZ_RSHIFT] = SDLK_RSHIFT;
    keymap[QZ_z] = SDLK_z;
    keymap[QZ_x] = SDLK_x;
    keymap[QZ_c] = SDLK_c;
    keymap[QZ_v] = SDLK_v;
    keymap[QZ_b] = SDLK_b;
    keymap[QZ_n] = SDLK_n;
    keymap[QZ_m] = SDLK_m;
    keymap[QZ_COMMA] = SDLK_COMMA;
    keymap[QZ_PERIOD] = SDLK_PERIOD;
    keymap[QZ_SLASH] = SDLK_SLASH;
    keymap[QZ_UP] = SDLK_UP;
    keymap[QZ_KP1] = SDLK_KP1;
    keymap[QZ_KP2] = SDLK_KP2;
    keymap[QZ_KP3] = SDLK_KP3;
    keymap[QZ_KP_ENTER] = SDLK_KP_ENTER;
    keymap[QZ_LCTRL] = SDLK_LCTRL;
    keymap[QZ_LALT] = SDLK_LALT;
    keymap[QZ_LMETA] = SDLK_LMETA;
    keymap[QZ_RCTRL] = SDLK_RCTRL;
    keymap[QZ_RALT] = SDLK_RALT;
    keymap[QZ_RMETA] = SDLK_RMETA;
    keymap[QZ_SPACE] = SDLK_SPACE;
    keymap[QZ_LEFT] = SDLK_LEFT;
    keymap[QZ_DOWN] = SDLK_DOWN;
    keymap[QZ_RIGHT] = SDLK_RIGHT;
    keymap[QZ_KP0] = SDLK_KP0;
    keymap[QZ_KP_PERIOD] = SDLK_KP_PERIOD;
    keymap[QZ_IBOOK_ENTER] = SDLK_KP_ENTER;
    keymap[QZ_IBOOK_RIGHT] = SDLK_RIGHT;
    keymap[QZ_IBOOK_DOWN] = SDLK_DOWN;
    keymap[QZ_IBOOK_UP]      = SDLK_UP;
    keymap[QZ_IBOOK_LEFT] = SDLK_LEFT;

    /* 
        Up there we setup a static scancode->keysym map. However, it will not
        work very well on international keyboard. Hence we now query MacOS
        for its own keymap to adjust our own mapping table. However, this is
        basically only useful for ascii char keys. This is also the reason
        why we keep the static table, too.
     */

#if (MAC_OS_X_VERSION_MAX_ALLOWED >= 1050)
    if (TISCopyCurrentKeyboardLayoutInputSource != NULL) {
        TISInputSourceRef src = TISCopyCurrentKeyboardLayoutInputSource();
        if (src != NULL) {
            CFDataRef data = (CFDataRef)
                TISGetInputSourceProperty(src,
                    kTISPropertyUnicodeKeyLayoutData);
            if (data != NULL) {
                const UCKeyboardLayout *layout = (const UCKeyboardLayout *)
                    CFDataGetBytePtr(data);
                if (layout != NULL) {
                    const UInt32 kbdtype = LMGetKbdType();
                    saw_layout = YES;

                    /* Loop over all 127 possible scan codes */
                    for (i = 0; i < 0x7F; i++) {
                        UniChar buf[16];
                        UniCharCount count = 0;

                        /* We pretend a clean start to begin with (i.e. no dead keys active */
                        state = 0;

                        if (UCKeyTranslate(layout, i, kUCKeyActionDown, 0, kbdtype,
                                           0, &state, 16, &count, buf) != noErr) {
                            continue;
                        }

                        /* If the state become 0, it was a dead key. We need to
                           translate again, passing in the new state, to get
                           the actual key value */
                        if (state != 0) {
                            if (UCKeyTranslate(layout, i, kUCKeyActionDown, 0, kbdtype,
                                               0, &state, 16, &count, buf) != noErr) {
                                continue;
                            }
                        }

                        if (count != 1) {
                            continue;  /* no multi-char. Use SDL 1.3 instead. :) */
                        }

                        value = (UInt32) buf[0];
                        if (value >= 128) {
                            /* Some non-ASCII char, map it to SDLK_WORLD_* */
                            if (world < 0xFF) {
                                keymap[i] = world++;
                            }
                        } else if (value >= 32) {     /* non-control ASCII char */
                            keymap[i] = value;
                        }
                    }
                }
            }
            CFRelease(src);
        }
    }
#endif

#if (MAC_OS_X_VERSION_MIN_REQUIRED < 1050)
    if (!saw_layout) {
        /* Get a pointer to the systems cached KCHR */
        const void *KCHRPtr = (const void *)GetScriptManagerVariable(smKCHRCache);
        if (KCHRPtr)
        {
            /* Loop over all 127 possible scan codes */
            for (i = 0; i < 0x7F; i++)
            {
                /* We pretend a clean start to begin with (i.e. no dead keys active */
                state = 0;

                /* Now translate the key code to a key value */
                value = KeyTranslate(KCHRPtr, i, &state) & 0xff;

                /* If the state become 0, it was a dead key. We need to translate again,
                    passing in the new state, to get the actual key value */
                if (state != 0)
                    value = KeyTranslate(KCHRPtr, i, &state) & 0xff;

                /* Now we should have an ascii value, or 0. Try to figure out to which SDL symbol it maps */
                if (value >= 128) {     /* Some non-ASCII char, map it to SDLK_WORLD_* */
                    if (world < 0xFF) {
                        keymap[i] = world++;
                    }
                } else if (value >= 32) {     /* non-control ASCII char */
                    keymap[i] = value;
                }
            }
        }
    }
#endif

    /* 
        The keypad codes are re-setup here, because the loop above cannot
        distinguish between a key on the keypad and a regular key. We maybe
        could get around this problem in another fashion: NSEvent's flags
        include a "NSNumericPadKeyMask" bit; we could check that and modify
        the symbol we return on the fly. However, this flag seems to exhibit
        some weird behaviour related to the num lock key
    */
    keymap[QZ_KP0] = SDLK_KP0;
    keymap[QZ_KP1] = SDLK_KP1;
    keymap[QZ_KP2] = SDLK_KP2;
    keymap[QZ_KP3] = SDLK_KP3;
    keymap[QZ_KP4] = SDLK_KP4;
    keymap[QZ_KP5] = SDLK_KP5;
    keymap[QZ_KP6] = SDLK_KP6;
    keymap[QZ_KP7] = SDLK_KP7;
    keymap[QZ_KP8] = SDLK_KP8;
    keymap[QZ_KP9] = SDLK_KP9;
    keymap[QZ_KP_MINUS] = SDLK_KP_MINUS;
    keymap[QZ_KP_PLUS] = SDLK_KP_PLUS;
    keymap[QZ_KP_PERIOD] = SDLK_KP_PERIOD;
    keymap[QZ_KP_EQUALS] = SDLK_KP_EQUALS;
    keymap[QZ_KP_DIVIDE] = SDLK_KP_DIVIDE;
    keymap[QZ_KP_MULTIPLY] = SDLK_KP_MULTIPLY;
    keymap[QZ_KP_ENTER] = SDLK_KP_ENTER;
}

static void QZ_DoKey (_THIS, int state, NSEvent *event) {

    NSString *chars = NULL;
    unsigned int i, numChars;
    SDL_keysym key;
    
    /* 
        A key event can contain multiple characters,
        or no characters at all. In most cases, it
        will contain a single character. If it contains
        0 characters, we'll use 0 as the unicode. If it
        contains multiple characters, we'll use 0 as
        the scancode/keysym.
    */
    if (SDL_TranslateUNICODE && state == SDL_PRESSED) {
        [field_edit interpretKeyEvents:[NSArray arrayWithObject:event]];
        chars = [ event characters ];
        numChars = [ chars length ];
        if (numChars > 0)
            [field_edit setString:@""];
    } else {
        numChars = 0;
    }

    if (numChars == 0) {
      
        key.scancode = [ event keyCode ];
        key.sym      = keymap [ key.scancode ];
        key.unicode  = 0;
        key.mod      = KMOD_NONE;

        SDL_PrivateKeyboard (state, &key);
    }
    else if (numChars >= 1) {

        key.scancode = [ event keyCode ];
        key.sym      = keymap [ key.scancode ];
        key.unicode  = [ chars characterAtIndex:0 ];
        key.mod      = KMOD_NONE;

        SDL_PrivateKeyboard (state, &key);
      
        for (i = 1; i < numChars; i++) {

            key.scancode = 0;
            key.sym      = 0;
            key.unicode  = [ chars characterAtIndex:i];
            key.mod      = KMOD_NONE;

            SDL_PrivateKeyboard (state, &key);
        }
    }
    
    if (SDL_getenv ("SDL_ENABLEAPPEVENTS"))
        [ NSApp sendEvent:event ];
}

/* This is the original behavior, before support was added for 
 * differentiating between left and right versions of the keys.
 */
static void QZ_DoUnsidedModifiers (_THIS, unsigned int newMods) {

    const int mapping[] = { SDLK_CAPSLOCK, SDLK_LSHIFT, SDLK_LCTRL, SDLK_LALT, SDLK_LMETA };

    int i;
    int bit;
    SDL_keysym key;
    
    key.scancode    = 0;
    key.sym         = SDLK_UNKNOWN;
    key.unicode     = 0;
    key.mod         = KMOD_NONE;

    /* Iterate through the bits, testing each against the current modifiers */
    for (i = 0, bit = NSAlphaShiftKeyMask; bit <= NSCommandKeyMask; bit <<= 1, ++i) {

        unsigned int currentMask, newMask;

        currentMask = current_mods & bit;
        newMask     = newMods & bit;

        if ( currentMask &&
             currentMask != newMask ) {     /* modifier up event */

             key.sym = mapping[i];
             /* If this was Caps Lock, we need some additional voodoo to make SDL happy */
             if (bit == NSAlphaShiftKeyMask)
                  SDL_PrivateKeyboard (SDL_PRESSED, &key);
             SDL_PrivateKeyboard (SDL_RELEASED, &key);
        }
        else if ( newMask &&
                  currentMask != newMask ) {     /* modifier down event */
        
             key.sym = mapping[i];
             SDL_PrivateKeyboard (SDL_PRESSED, &key);
             /* If this was Caps Lock, we need some additional voodoo to make SDL happy */
             if (bit == NSAlphaShiftKeyMask)
                  SDL_PrivateKeyboard (SDL_RELEASED, &key);
        }
    }
}

/* This is a helper function for QZ_HandleModifierSide. This 
 * function reverts back to behavior before the distinction between
 * sides was made.
 */
static void QZ_HandleNonDeviceModifier ( _THIS, unsigned int device_independent_mask, unsigned int newMods, unsigned int key_sym) {
    unsigned int currentMask, newMask;
    SDL_keysym key;
    
    key.scancode    = 0;
    key.sym         = key_sym;
    key.unicode     = 0;
    key.mod         = KMOD_NONE;
    
    /* Isolate just the bits we care about in the depedent bits so we can 
     * figure out what changed
     */ 
    currentMask = current_mods & device_independent_mask;
    newMask     = newMods & device_independent_mask;
    
    if ( currentMask &&
         currentMask != newMask ) {     /* modifier up event */
         SDL_PrivateKeyboard (SDL_RELEASED, &key);
    }
    else if ( newMask &&
          currentMask != newMask ) {     /* modifier down event */
          SDL_PrivateKeyboard (SDL_PRESSED, &key);
    }
}

/* This is a helper function for QZ_HandleModifierSide. 
 * This function sets the actual SDL_PrivateKeyboard event.
 */
static void QZ_HandleModifierOneSide ( _THIS, unsigned int newMods,
                                       unsigned int key_sym, 
                                       unsigned int sided_device_dependent_mask ) {
    
    SDL_keysym key;
    unsigned int current_dep_mask, new_dep_mask;
    
    key.scancode    = 0;
    key.sym         = key_sym;
    key.unicode     = 0;
    key.mod         = KMOD_NONE;
    
    /* Isolate just the bits we care about in the depedent bits so we can 
     * figure out what changed
     */ 
    current_dep_mask = current_mods & sided_device_dependent_mask;
    new_dep_mask     = newMods & sided_device_dependent_mask;
    
    /* We now know that this side bit flipped. But we don't know if
     * it went pressed to released or released to pressed, so we must 
     * find out which it is.
     */
    if( new_dep_mask &&
        current_dep_mask != new_dep_mask ) { 
        /* Modifier down event */
        SDL_PrivateKeyboard (SDL_PRESSED, &key);
    }
    else /* Modifier up event */ {
        SDL_PrivateKeyboard (SDL_RELEASED, &key);
    }
}

/* This is a helper function for QZ_DoSidedModifiers.
 * This function will figure out if the modifier key is the left or right side, 
 * e.g. left-shift vs right-shift. 
 */
static void QZ_HandleModifierSide ( _THIS, int device_independent_mask, 
                                    unsigned int newMods, 
                                    unsigned int left_key_sym, 
                                    unsigned int right_key_sym,
                                    unsigned int left_device_dependent_mask, 
                                    unsigned int right_device_dependent_mask ) {
    unsigned int device_dependent_mask = 0;
    unsigned int diff_mod = 0;
    
    device_dependent_mask = left_device_dependent_mask | right_device_dependent_mask;
    /* On the basis that the device independent mask is set, but there are 
     * no device dependent flags set, we'll assume that we can't detect this 
     * keyboard and revert to the unsided behavior.
     */
    if ( (device_dependent_mask & newMods) == 0 ) {
        /* Revert to the old behavior */
        QZ_HandleNonDeviceModifier ( this, device_independent_mask, newMods, left_key_sym );
        return;
    }
        
    /* XOR the previous state against the new state to see if there's a change */
    diff_mod = (device_dependent_mask & current_mods)
        ^ (device_dependent_mask & newMods);

    if ( diff_mod ) {
        /* A change in state was found. Isolate the left and right bits 
         * to handle them separately just in case the values can simulataneously
         * change or if the bits don't both exist.
         */
        if ( left_device_dependent_mask & diff_mod ) {
            QZ_HandleModifierOneSide ( this, newMods, left_key_sym, left_device_dependent_mask );
        }
        if ( right_device_dependent_mask & diff_mod ) {
            QZ_HandleModifierOneSide ( this, newMods, right_key_sym, right_device_dependent_mask );
        }
    }
}
   
/* This is a helper function for QZ_DoSidedModifiers.
 * This function will release a key press in the case that 
 * it is clear that the modifier has been released (i.e. one side 
 * can't still be down).
 */
static void QZ_ReleaseModifierSide ( _THIS, 
                                     unsigned int device_independent_mask, 
                                     unsigned int newMods,
                                     unsigned int left_key_sym, 
                                     unsigned int right_key_sym,
                                     unsigned int left_device_dependent_mask, 
                                     unsigned int right_device_dependent_mask ) {
    unsigned int device_dependent_mask = 0;
    SDL_keysym key;
    
    key.scancode    = 0;
    key.sym         = SDLK_UNKNOWN;
    key.unicode     = 0;
    key.mod         = KMOD_NONE;
    
    device_dependent_mask = left_device_dependent_mask | right_device_dependent_mask;
    /* On the basis that the device independent mask is set, but there are 
     * no device dependent flags set, we'll assume that we can't detect this 
     * keyboard and revert to the unsided behavior.
     */
    if ( (device_dependent_mask & current_mods) == 0 ) {
        /* In this case, we can't detect the keyboard, so use the left side 
         * to represent both, and release it. 
         */
        key.sym = left_key_sym;
        SDL_PrivateKeyboard (SDL_RELEASED, &key);

        return;
    }
        
        
    /* 
     * This could have been done in an if-else case because at this point,
     * we know that all keys have been released when calling this function. 
     * But I'm being paranoid so I want to handle each separately,
     * so I hope this doesn't cause other problems.
     */
    if ( left_device_dependent_mask & current_mods ) {
        key.sym = left_key_sym;
        SDL_PrivateKeyboard (SDL_RELEASED, &key);
    }
    if ( right_device_dependent_mask & current_mods ) {
        key.sym = right_key_sym;
        SDL_PrivateKeyboard (SDL_RELEASED, &key);
    }
}

/* This is a helper function for QZ_DoSidedModifiers.
 * This function handles the CapsLock case.
 */
static void QZ_HandleCapsLock (_THIS, unsigned int newMods) {
    unsigned int currentMask, newMask;
    SDL_keysym key;
    
    key.scancode    = 0;
    key.sym         = SDLK_CAPSLOCK;
    key.unicode     = 0;
    key.mod         = KMOD_NONE;
    
    currentMask = current_mods & NSAlphaShiftKeyMask;
    newMask     = newMods & NSAlphaShiftKeyMask;

    if ( currentMask &&
         currentMask != newMask ) {     /* modifier up event */
         /* If this was Caps Lock, we need some additional voodoo to make SDL happy */
         SDL_PrivateKeyboard (SDL_PRESSED, &key);
         SDL_PrivateKeyboard (SDL_RELEASED, &key);
    }
    else if ( newMask &&
              currentMask != newMask ) {     /* modifier down event */
        /* If this was Caps Lock, we need some additional voodoo to make SDL happy */
        SDL_PrivateKeyboard (SDL_PRESSED, &key);
        SDL_PrivateKeyboard (SDL_RELEASED, &key);
    }
}

/* This function will handle the modifier keys and also determine the 
 * correct side of the key.
 */
static void QZ_DoSidedModifiers (_THIS, unsigned int newMods) {
	/* Set up arrays for the key syms for the left and right side. */
    const unsigned int left_mapping[]  = { SDLK_LSHIFT, SDLK_LCTRL, SDLK_LALT, SDLK_LMETA };
    const unsigned int right_mapping[] = { SDLK_RSHIFT, SDLK_RCTRL, SDLK_RALT, SDLK_RMETA };
	/* Set up arrays for the device dependent masks with indices that 
     * correspond to the _mapping arrays 
     */
    const unsigned int left_device_mapping[]  = { NX_DEVICELSHIFTKEYMASK, NX_DEVICELCTLKEYMASK, NX_DEVICELALTKEYMASK, NX_DEVICELCMDKEYMASK };
    const unsigned int right_device_mapping[] = { NX_DEVICERSHIFTKEYMASK, NX_DEVICERCTLKEYMASK, NX_DEVICERALTKEYMASK, NX_DEVICERCMDKEYMASK };

    unsigned int i;
    unsigned int bit;
    
    /* Handle CAPSLOCK separately because it doesn't have a left/right side */
    QZ_HandleCapsLock ( this, newMods );
        
    /* Iterate through the bits, testing each against the current modifiers */
    for (i = 0, bit = NSShiftKeyMask; bit <= NSCommandKeyMask; bit <<= 1, ++i) {
		
        unsigned int currentMask, newMask;
		
        currentMask = current_mods & bit;
        newMask     = newMods & bit;
		
        /* If the bit is set, we must always examine it because the left
         * and right side keys may alternate or both may be pressed.
         */
        if ( newMask ) {
            QZ_HandleModifierSide ( this, bit, newMods, 
                                       left_mapping[i],
                                       right_mapping[i],
                                       left_device_mapping[i],
                                       right_device_mapping[i] );
        }
        /* If the state changed from pressed to unpressed, we must examine
            * the device dependent bits to release the correct keys.
            */
        else if ( currentMask &&
                  currentMask != newMask ) { /* modifier up event */
                  QZ_ReleaseModifierSide ( this, bit, newMods,
                                           left_mapping[i],
                                           right_mapping[i],
                                           left_device_mapping[i],
                                           right_device_mapping[i] );
        }
    }
}

/* This function is called to handle the modifiers.
 * It will try to distinguish between the left side and right side 
 * of the keyboard for those modifiers that qualify if the 
 * operating system version supports it. Otherwise, the code 
 * will not try to make the distinction.
 */
static void QZ_DoModifiers (_THIS, unsigned int newMods) {
	
    if (current_mods == newMods)
    	return;
    
    /* 
     * Starting with Panther (10.3.0), the ability to distinguish between 
     * left side and right side modifiers is available.
     */
    if( system_version >= 0x1030 ) {
        QZ_DoSidedModifiers (this, newMods);
    }
    else {
        QZ_DoUnsidedModifiers (this, newMods);
    }
    
    current_mods = newMods;
}

static void QZ_GetMouseLocation (_THIS, NSPoint *p) {
    *p = [ NSEvent mouseLocation ]; /* global coordinates */
    if (qz_window)
        QZ_PrivateGlobalToLocal (this, p);
    QZ_PrivateCocoaToSDL (this, p);
}

void QZ_DoActivate (_THIS) {

    SDL_PrivateAppActive (1, SDL_APPINPUTFOCUS | (QZ_IsMouseInWindow (this) ? SDL_APPMOUSEFOCUS : 0));

    QZ_UpdateCursor(this);

    /* Regrab input, only if it was previously grabbed */
    if ( current_grab_mode == SDL_GRAB_ON ) {
        
        /* Restore cursor location if input was grabbed */
        QZ_PrivateWarpCursor (this, cursor_loc.x, cursor_loc.y);
        QZ_ChangeGrabState (this, QZ_ENABLE_GRAB);
    }
    else {
        /* Update SDL's mouse location */
        NSPoint p;
        QZ_GetMouseLocation (this, &p);
        SDL_PrivateMouseMotion (0, 0, p.x, p.y);
    }

    QZ_UpdateCursor(this);
}

void QZ_DoDeactivate (_THIS) {
    
    SDL_PrivateAppActive (0, SDL_APPINPUTFOCUS | SDL_APPMOUSEFOCUS);

    /* Get the current cursor location, for restore on activate */
    QZ_GetMouseLocation (this, &cursor_loc);
    
    /* Reassociate mouse and cursor */
    CGAssociateMouseAndMouseCursorPosition (1);

    QZ_UpdateCursor(this);
}

void QZ_SleepNotificationHandler (void * refcon,
                                  io_service_t service,
                                  natural_t messageType,
                                  void * messageArgument )
{
     SDL_VideoDevice *this = (SDL_VideoDevice*)refcon;
     
     switch(messageType)
     {
         case kIOMessageSystemWillSleep:
             IOAllowPowerChange(power_connection, (long) messageArgument);
             break;
         case kIOMessageCanSystemSleep:
             IOAllowPowerChange(power_connection, (long) messageArgument);
             break;
         case kIOMessageSystemHasPoweredOn:
            /* awake */
            SDL_PrivateExpose();
            break;
     }
}

void QZ_RegisterForSleepNotifications (_THIS)
{
     CFRunLoopSourceRef rls;
     IONotificationPortRef thePortRef;
     io_object_t notifier;

     power_connection = IORegisterForSystemPower (this, &thePortRef, QZ_SleepNotificationHandler, &notifier);

     if (power_connection == 0)
         NSLog(@"SDL: QZ_SleepNotificationHandler() IORegisterForSystemPower failed.");

     rls = IONotificationPortGetRunLoopSource (thePortRef);
     CFRunLoopAddSource (CFRunLoopGetCurrent(), rls, kCFRunLoopDefaultMode);
     CFRelease (rls);
}


/* Try to map Quartz mouse buttons to SDL's lingo... */
static int QZ_OtherMouseButtonToSDL(int button)
{
    switch (button)
    {
        case 0:
            return(SDL_BUTTON_LEFT);   /* 1 */
        case 1:
            return(SDL_BUTTON_RIGHT);  /* 3 */
        case 2:
            return(SDL_BUTTON_MIDDLE); /* 2 */
    }

    /* >= 3: skip 4 & 5, since those are the SDL mousewheel buttons. */
    return(button + 3);
}


void QZ_PumpEvents (_THIS)
{
    int32_t dx, dy;

    NSDate *distantPast;
    NSEvent *event;
    NSRect winRect;
    NSAutoreleasePool *pool;

    if (!SDL_VideoSurface)
        return;  /* don't do anything if there's no screen surface. */

    /* Update activity every five seconds to prevent screensaver. --ryan. */
    if (!allow_screensaver) {
        static Uint32 screensaverTicks;
        Uint32 nowTicks = SDL_GetTicks();
        if ((nowTicks - screensaverTicks) > 5000)
        {
            UpdateSystemActivity(UsrActivity);
            screensaverTicks = nowTicks;
        }
    }

    pool = [ [ NSAutoreleasePool alloc ] init ];
    distantPast = [ NSDate distantPast ];

    winRect = NSMakeRect (0, 0, SDL_VideoSurface->w, SDL_VideoSurface->h);
    
    /* while grabbed, accumulate all mouse moved events into one SDL mouse event */
    dx = 0;
    dy = 0;
    
    do {
    
        /* Poll for an event. This will not block */
        event = [ NSApp nextEventMatchingMask:NSAnyEventMask
                                    untilDate:distantPast
                                    inMode: NSDefaultRunLoopMode dequeue:YES ];
        if (event != nil) {

            int button;
            unsigned int type;
            BOOL isInGameWin;
            
            #define DO_MOUSE_DOWN(button) do {                                               \
                            if ( SDL_GetAppState() & SDL_APPMOUSEFOCUS ) {                   \
                                SDL_PrivateMouseButton (SDL_PRESSED, button, 0, 0);          \
                                expect_mouse_up |= 1<<button;                                \
                            }                                                                \
                            [ NSApp sendEvent:event ];                                       \
            } while(0)
            
            #define DO_MOUSE_UP(button) do {                                            \
                            if ( expect_mouse_up & (1<<button) ) {                      \
                                SDL_PrivateMouseButton (SDL_RELEASED, button, 0, 0);    \
                                expect_mouse_up &= ~(1<<button);                        \
                            }                                                           \
                            [ NSApp sendEvent:event ];                                  \
            } while(0)
            
            type = [ event type ];
            isInGameWin = QZ_IsMouseInWindow (this);

            QZ_DoModifiers(this, [ event modifierFlags ] );

            switch (type) {
                case NSLeftMouseDown:
                    if ( SDL_getenv("SDL_HAS3BUTTONMOUSE") ) {
                        DO_MOUSE_DOWN (SDL_BUTTON_LEFT);
                    } else {
                        if ( NSCommandKeyMask & current_mods ) {
                            last_virtual_button = SDL_BUTTON_RIGHT;
                            DO_MOUSE_DOWN (SDL_BUTTON_RIGHT);
                        }
                        else if ( NSAlternateKeyMask & current_mods ) {
                            last_virtual_button = SDL_BUTTON_MIDDLE;
                            DO_MOUSE_DOWN (SDL_BUTTON_MIDDLE);
                        }
                        else {
                            DO_MOUSE_DOWN (SDL_BUTTON_LEFT);
                        }
                    }
                    break;

                case NSLeftMouseUp:
                    if ( last_virtual_button != 0 ) {
                        DO_MOUSE_UP (last_virtual_button);
                        last_virtual_button = 0;
                    }
                    else {
                        DO_MOUSE_UP (SDL_BUTTON_LEFT);
                    }
                    break;

                case NSOtherMouseDown:
                case NSRightMouseDown:
                    button = QZ_OtherMouseButtonToSDL([ event buttonNumber ]);
                    DO_MOUSE_DOWN (button);
                    break;

                case NSOtherMouseUp:
                case NSRightMouseUp:
                    button = QZ_OtherMouseButtonToSDL([ event buttonNumber ]);
                    DO_MOUSE_UP (button);
                    break;

                case NSSystemDefined:
                    /*
                        Future: up to 32 "mouse" buttons can be handled.
                        if ([event subtype] == 7) {
                            unsigned int buttons;
                            buttons = [ event data2 ];
                    */
                    break;
                case NSLeftMouseDragged:
                case NSRightMouseDragged:
                case NSOtherMouseDragged: /* usually middle mouse dragged */
                case NSMouseMoved:
                    if ( grab_state == QZ_INVISIBLE_GRAB ) {
                
                        /*
                            If input is grabbed+hidden, the cursor doesn't move,
                            so we have to call the lowlevel window server
                            function. This is less accurate but works OK.                         
                        */
                        int32_t dx1, dy1;
                        CGGetLastMouseDelta (&dx1, &dy1);
                        dx += dx1;
                        dy += dy1;
                    }
                    else {
                        
                        /*
                            Get the absolute mouse location. This is not the
                            mouse location after the currently processed event,
                            but the *current* mouse location, i.e. after all
                            pending events. This means that if there are
                            multiple mouse moved events in the queue, we make
                            multiple identical calls to SDL_PrivateMouseMotion(),
                            but that's no problem since the latter only
                            generates SDL events for nonzero movements. In my
                            experience on PBG4/10.4.8, this rarely happens anyway.
                        */
                        NSPoint p;
                        QZ_GetMouseLocation (this, &p);
                        SDL_PrivateMouseMotion (0, 0, p.x, p.y);
                    }
                    
                    /* 
                        Handle grab input+cursor visible by warping the cursor back
                        into the game window. This still generates a mouse moved event,
                        but not as a result of the warp (so it's in the right direction).
                    */
                    if ( grab_state == QZ_VISIBLE_GRAB && !isInGameWin ) {
                       
                        NSPoint p;
                        QZ_GetMouseLocation (this, &p);

                        if ( p.x < 0.0 ) 
                            p.x = 0.0;
                        
                        if ( p.y < 0.0 ) 
                            p.y = 0.0;
                        
                        if ( p.x >= winRect.size.width ) 
                            p.x = winRect.size.width-1;
                        
                        if ( p.y >= winRect.size.height ) 
                            p.y = winRect.size.height-1;
                        
                        QZ_PrivateWarpCursor (this, p.x, p.y);
                    }
                    else
                    if ( !isInGameWin && (SDL_GetAppState() & SDL_APPMOUSEFOCUS) ) {
                    
                        SDL_PrivateAppActive (0, SDL_APPMOUSEFOCUS);

                        if (grab_state == QZ_INVISIBLE_GRAB)
                            /*The cursor has left the window even though it is
                              disassociated from the mouse (and therefore
                              shouldn't move): this can happen with Wacom
                              tablets, and it effectively breaks the grab, since
                              mouse down events now go to background
                              applications. The only possibility to avoid this
                              seems to be talking to the tablet driver
                              (AppleEvents) to constrain its mapped area to the
                              window, which may not be worth the effort. For
                              now, handle the condition more gracefully than
                              before by reassociating cursor and mouse until the
                              cursor enters the window again, making it obvious
                              to the user that the grab is broken.*/
                            CGAssociateMouseAndMouseCursorPosition (1);

                        QZ_UpdateCursor(this);
                    }
                    else
                    if ( isInGameWin && (SDL_GetAppState() & (SDL_APPMOUSEFOCUS | SDL_APPINPUTFOCUS)) == SDL_APPINPUTFOCUS ) {
                    
                        SDL_PrivateAppActive (1, SDL_APPMOUSEFOCUS);

                        QZ_UpdateCursor(this);

                        if (grab_state == QZ_INVISIBLE_GRAB) { /*see comment above*/
                            QZ_PrivateWarpCursor (this, SDL_VideoSurface->w / 2, SDL_VideoSurface->h / 2);
                            CGAssociateMouseAndMouseCursorPosition (0);
                        }
                    }
                    break;
                case NSScrollWheel:
                    if ( isInGameWin ) {
                        float dy, dx;
                        Uint8 button;
                        dy = [ event deltaY ];
                        dx = [ event deltaX ];
                        if ( dy > 0.0 ) /* Scroll up */
                            button = SDL_BUTTON_WHEELUP;
                        else if ( dy < 0.0 ) /* Scroll down */
                            button = SDL_BUTTON_WHEELDOWN;
                        else
                            break; /* Horizontal scroll */
                        /* For now, wheel is sent as a quick down+up */
                        SDL_PrivateMouseButton (SDL_PRESSED, button, 0, 0);
                        SDL_PrivateMouseButton (SDL_RELEASED, button, 0, 0);
                    }
                    break;
                case NSKeyUp:
                    QZ_DoKey (this, SDL_RELEASED, event);
                    break;
                case NSKeyDown:
                    QZ_DoKey (this, SDL_PRESSED, event);
                    break;
                case NSFlagsChanged:
                    break;
                case NSAppKitDefined:
                    [ NSApp sendEvent:event ];
                    if ([ event subtype ] == NSApplicationActivatedEventType && (mode_flags & SDL_FULLSCREEN)) {
                        /* the default handling of this event seems to reset any cursor set by [NSCursor set] (used by SDL_SetCursor() in fullscreen mode) to the default system arrow cursor */
                        SDL_Cursor *sdlc = SDL_GetCursor();
                        if (sdlc != NULL && sdlc->wm_cursor != NULL) {
                            [ sdlc->wm_cursor->nscursor set ];
                        }
                    }
                    break;
                    /* case NSApplicationDefined: break; */
                    /* case NSPeriodic: break; */
                    /* case NSCursorUpdate: break; */
                default:
                    [ NSApp sendEvent:event ];
            }
        }
    } while (event != nil);
    
    /* handle accumulated mouse moved events */
    if (dx != 0 || dy != 0)
        SDL_PrivateMouseMotion (0, 1, dx, dy);
    
    [ pool release ];
}

void QZ_UpdateMouse (_THIS)
{
    NSPoint p;
    QZ_GetMouseLocation (this, &p);
    SDL_PrivateAppActive (QZ_IsMouseInWindow (this), SDL_APPMOUSEFOCUS);
    SDL_PrivateMouseMotion (0, 0, p.x, p.y);
}
