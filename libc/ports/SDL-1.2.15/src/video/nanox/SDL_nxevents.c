/*
    SDL - Simple DirectMedia Layer
    Copyright (C) 1997-2012 Sam Lantinga
    Copyright (C) 2001  Hsieh-Fu Tsai
    Copyright (C) 2002  Greg Haerr <greg@censoft.com>

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
    
    Hsieh-Fu Tsai
    clare@setabox.com
*/
#include "SDL_config.h"

#include "SDL_keysym.h"
#include "../../events/SDL_events_c.h"

#include "SDL_nxevents_c.h"
#include "SDL_nximage_c.h"

// The translation tables from a nanox keysym to a SDL keysym
static SDLKey NX_NONASCII_keymap [MWKEY_LAST + 1] ;

void NX_InitOSKeymap (_THIS)
{
    int i ;

    Dprintf ("enter NX_InitOSKeymap\n") ;

    // Map the nanox scancodes to SDL keysyms
    for (i = 0; i < SDL_arraysize (NX_NONASCII_keymap); ++ i)
        NX_NONASCII_keymap [i] = SDLK_UNKNOWN ;

    NX_NONASCII_keymap [MWKEY_LEFT        & 0xFF] = SDLK_LEFT ;
    NX_NONASCII_keymap [MWKEY_RIGHT       & 0xFF] = SDLK_RIGHT ;
    NX_NONASCII_keymap [MWKEY_UP          & 0xFF] = SDLK_UP ;
    NX_NONASCII_keymap [MWKEY_DOWN        & 0xFF] = SDLK_DOWN ;
    NX_NONASCII_keymap [MWKEY_INSERT      & 0xFF] = SDLK_INSERT ;
    NX_NONASCII_keymap [MWKEY_DELETE      & 0xFF] = SDLK_DELETE ;
    NX_NONASCII_keymap [MWKEY_HOME        & 0xFF] = SDLK_HOME ;
    NX_NONASCII_keymap [MWKEY_END         & 0xFF] = SDLK_END ;
    NX_NONASCII_keymap [MWKEY_PAGEUP      & 0xFF] = SDLK_PAGEUP ;
    NX_NONASCII_keymap [MWKEY_PAGEDOWN    & 0xFF] = SDLK_PAGEDOWN ;

    NX_NONASCII_keymap [MWKEY_KP0         & 0xFF] = SDLK_KP0 ;
    NX_NONASCII_keymap [MWKEY_KP1         & 0xFF] = SDLK_KP1 ;
    NX_NONASCII_keymap [MWKEY_KP2         & 0xFF] = SDLK_KP2 ;
    NX_NONASCII_keymap [MWKEY_KP3         & 0xFF] = SDLK_KP3 ;
    NX_NONASCII_keymap [MWKEY_KP4         & 0xFF] = SDLK_KP4 ;
    NX_NONASCII_keymap [MWKEY_KP5         & 0xFF] = SDLK_KP5 ;
    NX_NONASCII_keymap [MWKEY_KP6         & 0xFF] = SDLK_KP6 ;
    NX_NONASCII_keymap [MWKEY_KP7         & 0xFF] = SDLK_KP7 ;
    NX_NONASCII_keymap [MWKEY_KP8         & 0xFF] = SDLK_KP8 ;
    NX_NONASCII_keymap [MWKEY_KP9         & 0xFF] = SDLK_KP9 ;
    NX_NONASCII_keymap [MWKEY_KP_PERIOD   & 0xFF] = SDLK_KP_PERIOD ;
    NX_NONASCII_keymap [MWKEY_KP_DIVIDE   & 0xFF] = SDLK_KP_DIVIDE ;
    NX_NONASCII_keymap [MWKEY_KP_MULTIPLY & 0xFF] = SDLK_KP_MULTIPLY ;
    NX_NONASCII_keymap [MWKEY_KP_MINUS    & 0xFF] = SDLK_KP_MINUS ;
    NX_NONASCII_keymap [MWKEY_KP_PLUS     & 0xFF] = SDLK_KP_PLUS ;
    NX_NONASCII_keymap [MWKEY_KP_ENTER    & 0xFF] = SDLK_KP_ENTER ;
    NX_NONASCII_keymap [MWKEY_KP_EQUALS   & 0xFF] = SDLK_KP_EQUALS ;

    NX_NONASCII_keymap [MWKEY_F1          & 0xFF] = SDLK_F1 ;
    NX_NONASCII_keymap [MWKEY_F2          & 0xFF] = SDLK_F2 ;
    NX_NONASCII_keymap [MWKEY_F3          & 0xFF] = SDLK_F3 ;
    NX_NONASCII_keymap [MWKEY_F4          & 0xFF] = SDLK_F4 ;
    NX_NONASCII_keymap [MWKEY_F5          & 0xFF] = SDLK_F5 ;
    NX_NONASCII_keymap [MWKEY_F6          & 0xFF] = SDLK_F6 ;
    NX_NONASCII_keymap [MWKEY_F7          & 0xFF] = SDLK_F7 ;
    NX_NONASCII_keymap [MWKEY_F8          & 0xFF] = SDLK_F8 ;
    NX_NONASCII_keymap [MWKEY_F9          & 0xFF] = SDLK_F9 ;
    NX_NONASCII_keymap [MWKEY_F10         & 0xFF] = SDLK_F10 ;
    NX_NONASCII_keymap [MWKEY_F11         & 0xFF] = SDLK_F11 ;
    NX_NONASCII_keymap [MWKEY_F12         & 0xFF] = SDLK_F12 ;

    NX_NONASCII_keymap [MWKEY_NUMLOCK     & 0xFF] = SDLK_NUMLOCK ;
    NX_NONASCII_keymap [MWKEY_CAPSLOCK    & 0xFF] = SDLK_CAPSLOCK ;
    NX_NONASCII_keymap [MWKEY_SCROLLOCK   & 0xFF] = SDLK_SCROLLOCK ;
    NX_NONASCII_keymap [MWKEY_LSHIFT      & 0xFF] = SDLK_LSHIFT ;
    NX_NONASCII_keymap [MWKEY_RSHIFT      & 0xFF] = SDLK_RSHIFT ;
    NX_NONASCII_keymap [MWKEY_LCTRL       & 0xFF] = SDLK_LCTRL ;
    NX_NONASCII_keymap [MWKEY_RCTRL       & 0xFF] = SDLK_RCTRL ;
    NX_NONASCII_keymap [MWKEY_LALT        & 0xFF] = SDLK_LALT ;
    NX_NONASCII_keymap [MWKEY_RALT        & 0xFF] = SDLK_RALT ;
    NX_NONASCII_keymap [MWKEY_LMETA       & 0xFF] = SDLK_LMETA ;
    NX_NONASCII_keymap [MWKEY_RMETA       & 0xFF] = SDLK_RMETA ;
    NX_NONASCII_keymap [MWKEY_ALTGR       & 0xFF] = SDLK_MODE ;

    NX_NONASCII_keymap [MWKEY_PRINT       & 0xFF] = SDLK_PRINT ;
    NX_NONASCII_keymap [MWKEY_SYSREQ      & 0xFF] = SDLK_SYSREQ ;
    NX_NONASCII_keymap [MWKEY_PAUSE       & 0xFF] = SDLK_PAUSE ;
    NX_NONASCII_keymap [MWKEY_BREAK       & 0xFF] = SDLK_BREAK ;
    NX_NONASCII_keymap [MWKEY_MENU        & 0xFF] = SDLK_MENU ;

    Dprintf ("leave NX_InitOSKeymap\n") ;
}

SDL_keysym * NX_TranslateKey (GR_EVENT_KEYSTROKE * keystroke, SDL_keysym * keysym)
{
    GR_KEY ch = keystroke -> ch ;

    Dprintf ("enter NX_TranslateKey\n") ;

    keysym -> scancode = keystroke -> scancode ;
    keysym -> sym = SDLK_UNKNOWN ;

    if (ch & MWKEY_NONASCII_MASK) {
        keysym -> sym = NX_NONASCII_keymap [ch & 0xFF] ;
    } else {
        keysym -> sym = ch & 0x7F ;
    }

    keysym -> mod = KMOD_NONE ;
    
#if 1   //   Retrieve more mode information
    {
        GR_KEYMOD   mod = keystroke -> modifiers ;

        if (mod & MWKMOD_LSHIFT)
            keysym -> mod |= KMOD_LSHIFT ;
        if (mod & MWKMOD_RSHIFT)
            keysym -> mod |= KMOD_RSHIFT ;
        if (mod & MWKMOD_LCTRL)
            keysym -> mod |= KMOD_LCTRL ;
        if (mod & MWKMOD_RCTRL)
            keysym -> mod |= KMOD_RCTRL ;
        if (mod & MWKMOD_LALT)
            keysym -> mod |= KMOD_LALT ;
        if (mod & MWKMOD_RALT)
            keysym -> mod |= KMOD_RALT ;
        if (mod & MWKMOD_LMETA)
            keysym -> mod |= KMOD_LMETA ;
        if (mod & MWKMOD_RMETA)
            keysym -> mod |= KMOD_RMETA ;
        if (mod & MWKMOD_NUM)
            keysym -> mod |= KMOD_NUM ;
        if (mod & MWKMOD_CAPS)
            keysym -> mod |= KMOD_CAPS ;
        if (mod & MWKMOD_ALTGR)
            keysym -> mod |= KMOD_MODE ;
    }
#endif

    keysym -> unicode = ch ;

    Dprintf ("leave NX_TranslateKey\n") ;
    return keysym ;
}

static int check_boundary (_THIS, int x, int y)
{
    if (x < OffsetX || y < OffsetY || x > OffsetX + this -> screen -> w ||
        y > OffsetY + this -> screen -> h)
        return 0 ;
            
    return 1 ;
}

void NX_PumpEvents (_THIS)
{
    GR_EVENT         event ;
    static GR_BUTTON last_button_down = 0 ;

    GrCheckNextEvent (& event) ;
    while (event.type != GR_EVENT_TYPE_NONE) {

        // dispatch event
        switch (event.type) {
            case GR_EVENT_TYPE_MOUSE_ENTER :
            {
                Dprintf ("mouse enter\n") ;
                SDL_PrivateAppActive (1, SDL_APPMOUSEFOCUS) ;
                break ;
            }

            case GR_EVENT_TYPE_MOUSE_EXIT :
            {
                Dprintf ("mouse exit\n") ;
                SDL_PrivateAppActive (0, SDL_APPMOUSEFOCUS) ;
                break ;
            }

            case GR_EVENT_TYPE_FOCUS_IN :
            {
                Dprintf ("focus in\n") ;
                SDL_PrivateAppActive (1, SDL_APPINPUTFOCUS) ;
                break ;
            }

            case GR_EVENT_TYPE_FOCUS_OUT :
            {
                Dprintf ("focus out\n") ;
                SDL_PrivateAppActive (0, SDL_APPINPUTFOCUS) ;
                break ;
            }

            case GR_EVENT_TYPE_MOUSE_MOTION :
            {               
                Dprintf ("mouse motion\n") ;

                if (SDL_VideoSurface) {
                    if (currently_fullscreen) {
                        if (check_boundary (this, event.button.x, event.button.y)) {
                            SDL_PrivateMouseMotion (0, 0, event.button.x - OffsetX, 
                                event.button.y - OffsetY) ;
                        }
                    } else {
                        SDL_PrivateMouseMotion (0, 0, event.button.x, event.button.y) ;
                    }
                }
                break ;
            }

            case GR_EVENT_TYPE_BUTTON_DOWN :
            {
                int button = event.button.buttons ;
                
                Dprintf ("button down\n") ;

                switch (button) {
                    case MWBUTTON_L :
                        button = 1 ;
                        break ;
                    case MWBUTTON_M :
                        button = 2 ;
                        break ;
                    case MWBUTTON_R :
                        button = 3 ;
                        break ;
                    default :
                        button = 0 ;
                }
                last_button_down = button ;
                
                if (currently_fullscreen) {
                    if (check_boundary (this, event.button.x, event.button.y)) {
                        SDL_PrivateMouseButton (SDL_PRESSED, button, 
                            event.button.x - OffsetX, event.button.y - OffsetY) ;
                    }
                } else {
                    SDL_PrivateMouseButton (SDL_PRESSED, button, 
                        event.button.x, event.button.y) ;
                }
                break ;
            }

            // do not konw which button is released
            case GR_EVENT_TYPE_BUTTON_UP :
            {   
                Dprintf ("button up\n") ;

                if (currently_fullscreen) {
                    if (check_boundary (this, event.button.x, event.button.y)) {
                        SDL_PrivateMouseButton (SDL_RELEASED, last_button_down, 
                            event.button.x - OffsetX, event.button.y - OffsetY) ;
                    }
                } else {
                    SDL_PrivateMouseButton (SDL_RELEASED, last_button_down, 
                        event.button.x, event.button.y) ;
                }
                last_button_down = 0 ;
                break ;
            }

            case GR_EVENT_TYPE_KEY_DOWN :
            {
                SDL_keysym keysym ;

                Dprintf ("key down\n") ;
                SDL_PrivateKeyboard (SDL_PRESSED,
                    NX_TranslateKey (& event.keystroke, & keysym)) ;
                break ;
            }

            case GR_EVENT_TYPE_KEY_UP :
            {
                SDL_keysym keysym ;

                Dprintf ("key up\n") ;
                SDL_PrivateKeyboard (SDL_RELEASED,
                    NX_TranslateKey (& event.keystroke, & keysym)) ;
                break ;
            }

            case GR_EVENT_TYPE_CLOSE_REQ :
            {
                Dprintf ("close require\n") ;
                SDL_PrivateQuit () ;
                break ;
            }

            case GR_EVENT_TYPE_EXPOSURE :
            {
                Dprintf ("event_type_exposure\n") ;
                if (SDL_VideoSurface) {
                    NX_RefreshDisplay (this) ;//, & event.exposure) ;
                }
                break ;
            }

            case GR_EVENT_TYPE_UPDATE :
            {
                switch (event.update.utype) {
                    case GR_UPDATE_MAP :
                    {
                        Dprintf ("GR_UPDATE_MAP\n") ;
                        // If we're not active, make ourselves active
                        if (!(SDL_GetAppState () & SDL_APPACTIVE)) {
                            // Send an internal activate event
                            SDL_PrivateAppActive (1, SDL_APPACTIVE) ;
                        }
                        if (SDL_VideoSurface) {
                            NX_RefreshDisplay (this) ;
                        }
                        break ;
                    }
                    
                    case GR_UPDATE_UNMAP :
                    case GR_UPDATE_UNMAPTEMP :
                    {
                        Dprintf ("GR_UPDATE_UNMAP or GR_UPDATE_UNMAPTEMP\n") ;
                        // If we're active, make ourselves inactive
                        if (SDL_GetAppState () & SDL_APPACTIVE) {
                            // Send an internal deactivate event
                            SDL_PrivateAppActive (0, SDL_APPACTIVE | SDL_APPINPUTFOCUS) ;
                        }
                        break ; 
                    }
                    
                    case GR_UPDATE_SIZE :
                    {
                        Dprintf ("GR_UPDATE_SIZE\n") ;
                        SDL_PrivateResize (event.update.width, event.update.height) ;
                        break ; 
                    }

                    case GR_UPDATE_MOVE :
		    case GR_UPDATE_REPARENT :
                    {
                        Dprintf ("GR_UPDATE_MOVE or GR_UPDATE_REPARENT\n") ;
#ifdef ENABLE_NANOX_DIRECT_FB
			if (Clientfb) {
			    /* Get current window position and fb pointer*/
			    if (currently_fullscreen) 
				GrGetWindowFBInfo(FSwindow, &fbinfo);
			    else
				GrGetWindowFBInfo(SDL_Window, &fbinfo);
			}
#endif
                        break ; 
                    }
                    
                    default :
                        Dprintf ("unknown GR_EVENT_TYPE_UPDATE\n") ;
                        break ; 
                }
                break ; 
            }
                
            default :
            {
                Dprintf ("pump event default\n") ;
            }
        }

        GrCheckNextEvent (& event) ;
    }
}
