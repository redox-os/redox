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

SDL_X11_MODULE(BASEXLIB)
SDL_X11_SYM(XClassHint*,XAllocClassHint,(void),(),return)
SDL_X11_SYM(Status,XAllocColor,(Display* a,Colormap b,XColor* c),(a,b,c),return)
SDL_X11_SYM(XSizeHints*,XAllocSizeHints,(void),(),return)
SDL_X11_SYM(XWMHints*,XAllocWMHints,(void),(),return)
SDL_X11_SYM(int,XChangePointerControl,(Display* a,Bool b,Bool c,int d,int e,int f),(a,b,c,d,e,f),return)
SDL_X11_SYM(int,XChangeProperty,(Display* a,Window b,Atom c,Atom d,int e,int f,_Xconst unsigned char* g,int h),(a,b,c,d,e,f,g,h),return)
SDL_X11_SYM(int,XChangeWindowAttributes,(Display* a,Window b,unsigned long c,XSetWindowAttributes* d),(a,b,c,d),return)
SDL_X11_SYM(Bool,XCheckTypedEvent,(Display* a,int b,XEvent* c),(a,b,c),return)
SDL_X11_SYM(int,XClearWindow,(Display* a,Window b),(a,b),return)
SDL_X11_SYM(int,XCloseDisplay,(Display* a),(a),return)
SDL_X11_SYM(Colormap,XCreateColormap,(Display* a,Window b,Visual* c,int d),(a,b,c,d),return)
SDL_X11_SYM(Cursor,XCreatePixmapCursor,(Display* a,Pixmap b,Pixmap c,XColor* d,XColor* e,unsigned int f,unsigned int g),(a,b,c,d,e,f,g),return)
SDL_X11_SYM(GC,XCreateGC,(Display* a,Drawable b,unsigned long c,XGCValues* d),(a,b,c,d),return)
SDL_X11_SYM(XImage*,XCreateImage,(Display* a,Visual* b,unsigned int c,int d,int e,char* f,unsigned int g,unsigned int h,int i,int j),(a,b,c,d,e,f,g,h,i,j),return)
SDL_X11_SYM(Pixmap,XCreatePixmap,(Display* a,Drawable b,unsigned int c,unsigned int d,unsigned int e),(a,b,c,d,e),return)
SDL_X11_SYM(Pixmap,XCreatePixmapFromBitmapData,(Display* a,Drawable b,char* c,unsigned int d,unsigned int e,unsigned long f,unsigned long g,unsigned int h),(a,b,c,d,e,f,g,h),return)
SDL_X11_SYM(Window,XCreateSimpleWindow,(Display* a,Window b,int c,int d,unsigned int e,unsigned int f,unsigned int g,unsigned long h,unsigned long i),(a,b,c,d,e,f,g,h,i),return)
SDL_X11_SYM(Window,XCreateWindow,(Display* a,Window b,int c,int d,unsigned int e,unsigned int f,unsigned int g,int h,unsigned int i,Visual* j,unsigned long k,XSetWindowAttributes* l),(a,b,c,d,e,f,g,h,i,j,k,l),return)
SDL_X11_SYM(int,XDefineCursor,(Display* a,Window b,Cursor c),(a,b,c),return)
SDL_X11_SYM(int,XDeleteProperty,(Display* a,Window b,Atom c),(a,b,c),return)
SDL_X11_SYM(int,XDestroyWindow,(Display* a,Window b),(a,b),return)
SDL_X11_SYM(char*,XDisplayName,(_Xconst char* a),(a),return)
SDL_X11_SYM(int,XEventsQueued,(Display* a,int b),(a,b),return)
SDL_X11_SYM(Bool,XFilterEvent,(XEvent *event, Window w),(event,w),return)
SDL_X11_SYM(int,XFlush,(Display* a),(a),return)
SDL_X11_SYM(int,XFree,(void*a),(a),return)
SDL_X11_SYM(int,XFreeColormap,(Display* a,Colormap b),(a,b),return)
SDL_X11_SYM(int,XFreeColors,(Display* a,Colormap b,unsigned long* c,int d,unsigned long e),(a,b,c,d,e),return)
SDL_X11_SYM(int,XFreeCursor,(Display* a,Cursor b),(a,b),return)
SDL_X11_SYM(int,XFreeGC,(Display* a,GC b),(a,b),return)
SDL_X11_SYM(int,XFreeModifiermap,(XModifierKeymap* a),(a),return)
SDL_X11_SYM(int,XFreePixmap,(Display* a,Pixmap b),(a,b),return)
SDL_X11_SYM(int,XGetErrorDatabaseText,(Display* a,_Xconst char* b,_Xconst char* c,_Xconst char* d,char* e,int f),(a,b,c,d,e,f),return)
SDL_X11_SYM(XModifierKeymap*,XGetModifierMapping,(Display* a),(a),return)
SDL_X11_SYM(int,XGetPointerControl,(Display* a,int* b,int* c,int* d),(a,b,c,d),return)
SDL_X11_SYM(XVisualInfo*,XGetVisualInfo,(Display* a,long b,XVisualInfo* c,int* d),(a,b,c,d),return)
SDL_X11_SYM(XWMHints*,XGetWMHints,(Display* a,Window b),(a,b),return)
SDL_X11_SYM(Status,XGetWindowAttributes,(Display* a,Window b,XWindowAttributes* c),(a,b,c),return)
SDL_X11_SYM(int,XGrabKeyboard,(Display* a,Window b,Bool c,int d,int e,Time f),(a,b,c,d,e,f),return)
SDL_X11_SYM(int,XGrabPointer,(Display* a,Window b,Bool c,unsigned int d,int e,int f,Window g,Cursor h,Time i),(a,b,c,d,e,f,g,h,i),return)
SDL_X11_SYM(Status,XIconifyWindow,(Display* a,Window b,int c),(a,b,c),return)
SDL_X11_SYM(int,XInstallColormap,(Display* a,Colormap b),(a,b),return)
SDL_X11_SYM(KeyCode,XKeysymToKeycode,(Display* a,KeySym b),(a,b),return)
SDL_X11_SYM(Atom,XInternAtom,(Display* a,_Xconst char* b,Bool c),(a,b,c),return)
SDL_X11_SYM(XPixmapFormatValues*,XListPixmapFormats,(Display* a,int* b),(a,b),return)
SDL_X11_SYM(int,XLookupString,(XKeyEvent* a,char* b,int c,KeySym* d,XComposeStatus* e),(a,b,c,d,e),return)
SDL_X11_SYM(int,XMapRaised,(Display* a,Window b),(a,b),return)
SDL_X11_SYM(int,XMapWindow,(Display* a,Window b),(a,b),return)
SDL_X11_SYM(int,XMaskEvent,(Display* a,long b,XEvent* c),(a,b,c),return)
SDL_X11_SYM(Status,XMatchVisualInfo,(Display* a,int b,int c,int d,XVisualInfo* e),(a,b,c,d,e),return)
SDL_X11_SYM(int,XMissingExtension,(Display* a,_Xconst char* b),(a,b),return)
SDL_X11_SYM(int,XMoveResizeWindow,(Display* a,Window b,int c,int d,unsigned int e,unsigned int f),(a,b,c,d,e,f),return)
SDL_X11_SYM(int,XMoveWindow,(Display* a,Window b,int c,int d),(a,b,c,d),return)
SDL_X11_SYM(int,XNextEvent,(Display* a,XEvent* b),(a,b),return)
SDL_X11_SYM(Display*,XOpenDisplay,(_Xconst char* a),(a),return)
SDL_X11_SYM(int,XPeekEvent,(Display* a,XEvent* b),(a,b),return)
SDL_X11_SYM(int,XPending,(Display* a),(a),return)
SDL_X11_SYM(int,XPutImage,(Display* a,Drawable b,GC c,XImage* d,int e,int f,int g,int h,unsigned int i,unsigned int j),(a,b,c,d,e,f,g,h,i,j),return)
SDL_X11_SYM(int,XQueryColors,(Display* a,Colormap b,XColor* c,int d),(a,b,c,d),return)
SDL_X11_SYM(int,XQueryKeymap,(Display* a,char *b),(a,b),return)
SDL_X11_SYM(Bool,XQueryPointer,(Display* a,Window b,Window* c,Window* d,int* e,int* f,int* g,int* h,unsigned int* i),(a,b,c,d,e,f,g,h,i),return)
SDL_X11_SYM(int,XRaiseWindow,(Display* a,Window b),(a,b),return)
SDL_X11_SYM(int,XReparentWindow,(Display* a,Window b,Window c,int d,int e),(a,b,c,d,e),return)
SDL_X11_SYM(int,XResetScreenSaver,(Display* a),(a),return)
SDL_X11_SYM(int,XResizeWindow,(Display* a,Window b,unsigned int c,unsigned int d),(a,b,c,d),return)
SDL_X11_SYM(int,XSelectInput,(Display* a,Window b,long c),(a,b,c),return)
SDL_X11_SYM(Status,XSendEvent,(Display* a,Window b,Bool c,long d,XEvent* e),(a,b,c,d,e),return)
SDL_X11_SYM(int,XSetClassHint,(Display* a,Window b,XClassHint* c),(a,b,c),return)
SDL_X11_SYM(XErrorHandler,XSetErrorHandler,(XErrorHandler a),(a),return)
SDL_X11_SYM(XIOErrorHandler,XSetIOErrorHandler,(XIOErrorHandler a),(a),return)
SDL_X11_SYM(int,XSetTransientForHint,(Display* a,Window b,Window c),(a,b,c),return)
SDL_X11_SYM(int,XSetWMHints,(Display* a,Window b,XWMHints* c),(a,b,c),return)
SDL_X11_SYM(void,XSetTextProperty,(Display* a,Window b,XTextProperty* c,Atom d),(a,b,c,d),)
SDL_X11_SYM(void,XSetWMNormalHints,(Display* a,Window b,XSizeHints* c),(a,b,c),)
SDL_X11_SYM(Status,XSetWMProtocols,(Display* a,Window b,Atom* c,int d),(a,b,c,d),return)
SDL_X11_SYM(int,XSetWindowBackground,(Display* a,Window b,unsigned long c),(a,b,c),return)
SDL_X11_SYM(int,XSetWindowBackgroundPixmap,(Display* a,Window b,Pixmap c),(a,b,c),return)
SDL_X11_SYM(int,XSetWindowColormap,(Display* a,Window b,Colormap c),(a,b,c),return)
SDL_X11_SYM(int,XStoreColors,(Display* a,Colormap b,XColor* c,int d),(a,b,c,d),return)
SDL_X11_SYM(Status,XStringListToTextProperty,(char** a,int b,XTextProperty* c),(a,b,c),return)
SDL_X11_SYM(int,XSync,(Display* a,Bool b),(a,b),return)
SDL_X11_SYM(int,XUngrabKeyboard,(Display* a,Time b),(a,b),return)
SDL_X11_SYM(int,XUngrabPointer,(Display* a,Time b),(a,b),return)
SDL_X11_SYM(int,XUnmapWindow,(Display* a,Window b),(a,b),return)
SDL_X11_SYM(int,XWarpPointer,(Display* a,Window b,Window c,int d,int e,unsigned int f,unsigned int g,int h, int i),(a,b,c,d,e,f,g,h,i),return)
SDL_X11_SYM(VisualID,XVisualIDFromVisual,(Visual* a),(a),return)
SDL_X11_SYM(XExtDisplayInfo*,XextAddDisplay,(XExtensionInfo* a,Display* b,char* c,XExtensionHooks* d,int e,XPointer f),(a,b,c,d,e,f),return)
SDL_X11_SYM(XExtensionInfo*,XextCreateExtension,(void),(),return)
SDL_X11_SYM(void,XextDestroyExtension,(XExtensionInfo* a),(a),)
SDL_X11_SYM(XExtDisplayInfo*,XextFindDisplay,(XExtensionInfo* a,Display* b),(a,b),return)
SDL_X11_SYM(int,XextRemoveDisplay,(XExtensionInfo* a,Display* b),(a,b),return)
SDL_X11_SYM(Bool,XQueryExtension,(Display* a,_Xconst char* b,int* c,int* d,int* e),(a,b,c,d,e),return)
SDL_X11_SYM(char *,XDisplayString,(Display* a),(a),return)
SDL_X11_SYM(int,XGetErrorText,(Display* a,int b,char* c,int d),(a,b,c,d),return)
SDL_X11_SYM(void,_XEatData,(Display* a,unsigned long b),(a,b),)
SDL_X11_SYM(void,_XFlush,(Display* a),(a),)
SDL_X11_SYM(void,_XFlushGCCache,(Display* a,GC b),(a,b),)
SDL_X11_SYM(int,_XRead,(Display* a,char* b,long c),(a,b,c),return)
SDL_X11_SYM(void,_XReadPad,(Display* a,char* b,long c),(a,b,c),)
SDL_X11_SYM(void,_XSend,(Display* a,_Xconst char* b,long c),(a,b,c),)
SDL_X11_SYM(Status,_XReply,(Display* a,xReply* b,int c,Bool d),(a,b,c,d),return)
SDL_X11_SYM(unsigned long,_XSetLastRequestRead,(Display* a,xGenericReply* b),(a,b),return)
SDL_X11_SYM(SDL_X11_XSynchronizeRetType,XSynchronize,(Display* a,Bool b),(a,b),return)
SDL_X11_SYM(SDL_X11_XESetWireToEventRetType,XESetWireToEvent,(Display* a,int b,SDL_X11_XESetWireToEventRetType c),(a,b,c),return)
SDL_X11_SYM(SDL_X11_XESetEventToWireRetType,XESetEventToWire,(Display* a,int b,SDL_X11_XESetEventToWireRetType c),(a,b,c),return)
SDL_X11_SYM(XExtensionErrorHandler,XSetExtensionErrorHandler,(XExtensionErrorHandler a),(a),return)

#if NeedWidePrototypes
SDL_X11_SYM(KeySym,XKeycodeToKeysym,(Display* a,unsigned int b,int c),(a,b,c),return)
#else
SDL_X11_SYM(KeySym,XKeycodeToKeysym,(Display* a,KeyCode b,int c),(a,b,c),return)
#endif

#ifdef X_HAVE_UTF8_STRING
SDL_X11_MODULE(UTF8)
SDL_X11_SYM(int,Xutf8TextListToTextProperty,(Display* a,char** b,int c,XICCEncodingStyle d,XTextProperty* e),(a,b,c,d,e),return)
SDL_X11_SYM(int,Xutf8LookupString,(XIC a,XKeyPressedEvent* b,char* c,int d,KeySym* e,Status* f),(a,b,c,d,e,f),return)
/*SDL_X11_SYM(XIC,XCreateIC,(XIM, ...),return)  !!! ARGH! */
SDL_X11_SYM(void,XDestroyIC,(XIC a),(a),)
SDL_X11_SYM(void,XSetICFocus,(XIC a),(a),)
SDL_X11_SYM(void,XUnsetICFocus,(XIC a),(a),)
/*SDL_X11_SYM(char*,XGetICValues,(XIC a, ...),return)*/
SDL_X11_SYM(XIM,XOpenIM,(Display* a,struct _XrmHashBucketRec* b,char* c,char* d),(a,b,c,d),return)
SDL_X11_SYM(Status,XCloseIM,(XIM a),(a),return)
SDL_X11_SYM(char*,XSetLocaleModifiers,(_Xconst char* a),(a),return)
SDL_X11_SYM(int,XRefreshKeyboardMapping,(XMappingEvent* a),(a),return)
SDL_X11_SYM(Display*,XDisplayOfIM,(XIM a),(a),return)
#endif

#ifndef NO_SHARED_MEMORY
SDL_X11_MODULE(SHM)
SDL_X11_SYM(Status,XShmAttach,(Display* a,XShmSegmentInfo* b),(a,b),return)
SDL_X11_SYM(Status,XShmDetach,(Display* a,XShmSegmentInfo* b),(a,b),return)
SDL_X11_SYM(Status,XShmPutImage,(Display* a,Drawable b,GC c,XImage* d,int e,int f,int g,int h,unsigned int i,unsigned int j,Bool k),(a,b,c,d,e,f,g,h,i,j,k),return)
SDL_X11_SYM(XImage*,XShmCreateImage,(Display* a,Visual* b,unsigned int c,int d,char* e,XShmSegmentInfo* f,unsigned int g,unsigned int h),(a,b,c,d,e,f,g,h),return)
SDL_X11_SYM(Bool,XShmQueryExtension,(Display* a),(a),return)
#endif

/*
 * Not required...these only exist in code in headers on some 64-bit platforms,
 *  and are removed via macros elsewhere, so it's safe for them to be missing.
 */
#ifdef LONG64
SDL_X11_MODULE(IO_32BIT)
SDL_X11_SYM(int,_XData32,(Display *dpy,register long *data,unsigned len),(dpy,data,len),return)
SDL_X11_SYM(void,_XRead32,(Display *dpy,register long *data,long len),(dpy,data,len),)
#endif

/*
 * libX11 1.4.99.1 added _XGetRequest, and macros use it behind the scenes.
 */
SDL_X11_MODULE(XGETREQUEST)
SDL_X11_SYM(void *,_XGetRequest,(Display* a,CARD8 b,size_t c),(a,b,c),return)

/*
 * These only show up on some variants of Unix.
 */
#if defined(__osf__)
SDL_X11_MODULE(OSF_ENTRY_POINTS)
SDL_X11_SYM(void,_SmtBufferOverflow,(Display *dpy,register smtDisplayPtr p),(dpy,p),)
SDL_X11_SYM(void,_SmtIpError,(Display *dpy,register smtDisplayPtr p, int i),(dpy,p,i),)
SDL_X11_SYM(int,ipAllocateData,(ChannelPtr a, IPCard b, IPDataPtr * c),(a,b,c),return)
SDL_X11_SYM(int,ipUnallocateAndSendData,(ChannelPtr a, IPCard b),(a,b),return)
#endif

/* Xrandr support. */
#if SDL_VIDEO_DRIVER_X11_XRANDR
SDL_X11_MODULE(XRANDR)
SDL_X11_SYM(Status,XRRQueryVersion,(Display *dpy,int *major_versionp,int *minor_versionp),(dpy,major_versionp,minor_versionp),return)
SDL_X11_SYM(XRRScreenConfiguration *,XRRGetScreenInfo,(Display *dpy,Drawable draw),(dpy,draw),return)
SDL_X11_SYM(SizeID,XRRConfigCurrentConfiguration,(XRRScreenConfiguration *config,Rotation *rotation),(config,rotation),return)
SDL_X11_SYM(XRRScreenSize *,XRRConfigSizes,(XRRScreenConfiguration *config, int *nsizes),(config,nsizes),return)
SDL_X11_SYM(Status,XRRSetScreenConfig,(Display *dpy, XRRScreenConfiguration *config, Drawable draw, int size_index, Rotation rotation, Time timestamp),(dpy,config,draw,size_index,rotation,timestamp),return)
SDL_X11_SYM(void,XRRFreeScreenConfigInfo,(XRRScreenConfiguration *config),(config),)
#endif

/* end of SDL_x11sym.h ... */

