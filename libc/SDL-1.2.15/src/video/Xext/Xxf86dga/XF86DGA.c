/* $XFree86: xc/lib/Xxf86dga/XF86DGA.c,v 3.19 2001/08/18 02:41:30 dawes Exp $ */
/*

Copyright (c) 1995  Jon Tombs
Copyright (c) 1995,1996  The XFree86 Project, Inc

*/

/* THIS IS NOT AN X CONSORTIUM STANDARD */

#ifdef __EMX__ /* needed here to override certain constants in X headers */
#define INCL_DOS
#define INCL_DOSIOCTL
#include <os2.h>
#endif

#if defined(linux)
#define HAS_MMAP_ANON
#include <sys/types.h>
#include <sys/mman.h>
/*#include <asm/page.h>*/   /* PAGE_SIZE */
#define HAS_SC_PAGESIZE /* _SC_PAGESIZE may be an enum for Linux */
#define HAS_GETPAGESIZE
#endif /* linux */

#if defined(CSRG_BASED)
#define HAS_MMAP_ANON
#define HAS_GETPAGESIZE
#include <sys/types.h>
#include <sys/mman.h>
#endif /* CSRG_BASED */

#if defined(DGUX)
#define HAS_GETPAGESIZE
#define MMAP_DEV_ZERO
#include <sys/types.h>
#include <sys/mman.h>
#include <unistd.h>
#endif /* DGUX */

#if defined(SVR4) && !defined(DGUX)
#define MMAP_DEV_ZERO
#include <sys/types.h>
#include <sys/mman.h>
#include <unistd.h>
#endif /* SVR4 && !DGUX */

#if defined(sun) && !defined(SVR4) /* SunOS */
#define MMAP_DEV_ZERO   /* doesn't SunOS have MAP_ANON ?? */
#define HAS_GETPAGESIZE
#include <sys/types.h>
#include <sys/mman.h>
#endif /* sun && !SVR4 */

#ifdef XNO_SYSCONF
#undef _SC_PAGESIZE
#endif

#define NEED_EVENTS
#define NEED_REPLIES

/* Apparently some X11 systems can't include this multiple times... */
#ifndef SDL_INCLUDED_XLIBINT_H
#define SDL_INCLUDED_XLIBINT_H 1
#include <X11/Xlibint.h>
#endif

#include "../extensions/xf86dga.h"
#include "../extensions/xf86dgastr.h"
#include "../extensions/Xext.h"
#include "../extensions/extutil.h"

extern XExtDisplayInfo* SDL_NAME(xdga_find_display)(Display*);
extern char *SDL_NAME(xdga_extension_name);

#define XF86DGACheckExtension(dpy,i,val) \
  XextCheckExtension (dpy, i, SDL_NAME(xdga_extension_name), val)

/*****************************************************************************
 *                                                                           *
 *		    public XFree86-DGA Extension routines                    *
 *                                                                           *
 *****************************************************************************/

Bool SDL_NAME(XF86DGAQueryExtension) (
    Display *dpy,
    int *event_basep,
    int *error_basep
){
    return SDL_NAME(XDGAQueryExtension)(dpy, event_basep, error_basep);
}

Bool SDL_NAME(XF86DGAQueryVersion)(
    Display* dpy,
    int* majorVersion, 
    int* minorVersion
){
    return SDL_NAME(XDGAQueryVersion)(dpy, majorVersion, minorVersion);
}

Bool SDL_NAME(XF86DGAGetVideoLL)(
    Display* dpy,
    int screen,
    int *offset,
    int *width, 
    int *bank_size, 
    int *ram_size
){
    XExtDisplayInfo *info = SDL_NAME(xdga_find_display) (dpy);
    xXF86DGAGetVideoLLReply rep;
    xXF86DGAGetVideoLLReq *req;

    XF86DGACheckExtension (dpy, info, False);

    LockDisplay(dpy);
    GetReq(XF86DGAGetVideoLL, req);
    req->reqType = info->codes->major_opcode;
    req->dgaReqType = X_XF86DGAGetVideoLL;
    req->screen = screen;
    if (!_XReply(dpy, (xReply *)&rep, 0, xFalse)) {
	UnlockDisplay(dpy);
	SyncHandle();
	return False;
    }

    *offset = /*(char *)*/rep.offset;
    *width = rep.width;
    *bank_size = rep.bank_size;
    *ram_size = rep.ram_size;
	
    UnlockDisplay(dpy);
    SyncHandle();
    return True;
}

    
Bool SDL_NAME(XF86DGADirectVideoLL)(
    Display* dpy,
    int screen,
    int enable
){
    XExtDisplayInfo *info = SDL_NAME(xdga_find_display) (dpy);
    xXF86DGADirectVideoReq *req;

    XF86DGACheckExtension (dpy, info, False);

    LockDisplay(dpy);
    GetReq(XF86DGADirectVideo, req);
    req->reqType = info->codes->major_opcode;
    req->dgaReqType = X_XF86DGADirectVideo;
    req->screen = screen;
    req->enable = enable;
    UnlockDisplay(dpy);
    SyncHandle();
    XSync(dpy,False);
    return True;
}

Bool SDL_NAME(XF86DGAGetViewPortSize)(
    Display* dpy,
    int screen,
    int *width, 
    int *height
){
    XExtDisplayInfo *info = SDL_NAME(xdga_find_display) (dpy);
    xXF86DGAGetViewPortSizeReply rep;
    xXF86DGAGetViewPortSizeReq *req;

    XF86DGACheckExtension (dpy, info, False);

    LockDisplay(dpy);
    GetReq(XF86DGAGetViewPortSize, req);
    req->reqType = info->codes->major_opcode;
    req->dgaReqType = X_XF86DGAGetViewPortSize;
    req->screen = screen;
    if (!_XReply(dpy, (xReply *)&rep, 0, xFalse)) {
	UnlockDisplay(dpy);
	SyncHandle();
	return False;
    }

    *width = rep.width;
    *height = rep.height;
	
    UnlockDisplay(dpy);
    SyncHandle();
    return True;
}
    
    
Bool SDL_NAME(XF86DGASetViewPort)(
    Display* dpy,
    int screen,
    int x, 
    int y
){
    XExtDisplayInfo *info = SDL_NAME(xdga_find_display) (dpy);
    xXF86DGASetViewPortReq *req;

    XF86DGACheckExtension (dpy, info, False);

    LockDisplay(dpy);
    GetReq(XF86DGASetViewPort, req);
    req->reqType = info->codes->major_opcode;
    req->dgaReqType = X_XF86DGASetViewPort;
    req->screen = screen;
    req->x = x;
    req->y = y;
    UnlockDisplay(dpy);
    SyncHandle();
    XSync(dpy,False);
    return True;
}

    
Bool SDL_NAME(XF86DGAGetVidPage)(
    Display* dpy,
    int screen,
    int *vpage
){
    XExtDisplayInfo *info = SDL_NAME(xdga_find_display) (dpy);
    xXF86DGAGetVidPageReply rep;
    xXF86DGAGetVidPageReq *req;

    XF86DGACheckExtension (dpy, info, False);

    LockDisplay(dpy);
    GetReq(XF86DGAGetVidPage, req);
    req->reqType = info->codes->major_opcode;
    req->dgaReqType = X_XF86DGAGetVidPage;
    req->screen = screen;
    if (!_XReply(dpy, (xReply *)&rep, 0, xFalse)) {
	UnlockDisplay(dpy);
	SyncHandle();
	return False;
    }

    *vpage = rep.vpage;
    UnlockDisplay(dpy);
    SyncHandle();
    return True;
}

    
Bool SDL_NAME(XF86DGASetVidPage)(
    Display* dpy,
    int screen,
    int vpage
){
    XExtDisplayInfo *info = SDL_NAME(xdga_find_display) (dpy);
    xXF86DGASetVidPageReq *req;

    XF86DGACheckExtension (dpy, info, False);

    LockDisplay(dpy);
    GetReq(XF86DGASetVidPage, req);
    req->reqType = info->codes->major_opcode;
    req->dgaReqType = X_XF86DGASetVidPage;
    req->screen = screen;
    req->vpage = vpage;
    UnlockDisplay(dpy);
    SyncHandle();
    XSync(dpy,False);
    return True;
}

Bool SDL_NAME(XF86DGAInstallColormap)(
    Display* dpy,
    int screen,
    Colormap cmap
){
    XExtDisplayInfo *info = SDL_NAME(xdga_find_display) (dpy);
    xXF86DGAInstallColormapReq *req;

    XF86DGACheckExtension (dpy, info, False);

    LockDisplay(dpy);
    GetReq(XF86DGAInstallColormap, req);
    req->reqType = info->codes->major_opcode;
    req->dgaReqType = X_XF86DGAInstallColormap;
    req->screen = screen;
    req->id = cmap;
    UnlockDisplay(dpy);
    SyncHandle();
    XSync(dpy,False);
    return True;
}

Bool SDL_NAME(XF86DGAQueryDirectVideo)(
    Display *dpy,
    int screen,
    int *flags
){
    XExtDisplayInfo *info = SDL_NAME(xdga_find_display) (dpy);
    xXF86DGAQueryDirectVideoReply rep;
    xXF86DGAQueryDirectVideoReq *req;

    XF86DGACheckExtension (dpy, info, False);

    LockDisplay(dpy);
    GetReq(XF86DGAQueryDirectVideo, req);
    req->reqType = info->codes->major_opcode;
    req->dgaReqType = X_XF86DGAQueryDirectVideo;
    req->screen = screen;
    if (!_XReply(dpy, (xReply *)&rep, 0, xFalse)) {
	UnlockDisplay(dpy);
	SyncHandle();
	return False;
    }
    *flags = rep.flags;
    UnlockDisplay(dpy);
    SyncHandle();
    return True;
}

Bool SDL_NAME(XF86DGAViewPortChanged)(
    Display *dpy,
    int screen,
    int n
){
    XExtDisplayInfo *info = SDL_NAME(xdga_find_display) (dpy);
    xXF86DGAViewPortChangedReply rep;
    xXF86DGAViewPortChangedReq *req;

    XF86DGACheckExtension (dpy, info, False);

    LockDisplay(dpy);
    GetReq(XF86DGAViewPortChanged, req);
    req->reqType = info->codes->major_opcode;
    req->dgaReqType = X_XF86DGAViewPortChanged;
    req->screen = screen;
    req->n = n;
    if (!_XReply(dpy, (xReply *)&rep, 0, xFalse)) {
	UnlockDisplay(dpy);
	SyncHandle();
	return False;
    }
    UnlockDisplay(dpy);
    SyncHandle();
    return rep.result;
}



/* Helper functions */

#include <X11/Xmd.h>
#include "../extensions/xf86dga.h"
#include <stdlib.h>
#include <stdio.h>
#include <fcntl.h>
#if defined(ISC) 
# define HAS_SVR3_MMAP
# include <sys/types.h>
# include <errno.h>

# include <sys/at_ansi.h>
# include <sys/kd.h>

# include <sys/sysmacros.h>
# include <sys/immu.h>
# include <sys/region.h>

# include <sys/mmap.h>
#else
# if !defined(Lynx)
#  if !defined(__EMX__)
#   include <sys/mman.h>
#  endif
# else
#  include <sys/types.h>
#  include <errno.h>
#  include <smem.h>
# endif
#endif
#include <sys/wait.h>
#include <signal.h>
#include <unistd.h>

#if defined(SVR4) && !defined(sun) && !defined(SCO325)
#define DEV_MEM "/dev/pmem"
#elif defined(SVR4) && defined(sun)
#define DEV_MEM "/dev/xsvc"
#else
#define DEV_MEM "/dev/mem"
#endif

typedef struct {
    unsigned long physaddr;	/* actual requested physical address */
    unsigned long size;		/* actual requested map size */
    unsigned long delta;	/* delta to account for page alignment */
    void *	  vaddr;	/* mapped address, without the delta */
    int		  refcount;	/* reference count */
} MapRec, *MapPtr;

typedef struct {
    Display *	display;
    int		screen;
    MapPtr	map;
} ScrRec, *ScrPtr;

static int mapFd = -1;
static int numMaps = 0;
static int numScrs = 0;
static MapPtr *mapList = NULL;
static ScrPtr *scrList = NULL;

static MapPtr
AddMap(void)
{
    MapPtr *old;

    old = mapList;
    mapList = realloc(mapList, sizeof(MapPtr) * (numMaps + 1));
    if (!mapList) {
	mapList = old;
	return NULL;
    }
    mapList[numMaps] = malloc(sizeof(MapRec));
    if (!mapList[numMaps])
	return NULL;
    return mapList[numMaps++];
}

static ScrPtr
AddScr(void)
{
    ScrPtr *old;

    old = scrList;
    scrList = realloc(scrList, sizeof(ScrPtr) * (numScrs + 1));
    if (!scrList) {
	scrList = old;
	return NULL;
    }
    scrList[numScrs] = malloc(sizeof(ScrRec));
    if (!scrList[numScrs])
	return NULL;
    return scrList[numScrs++];
}

static MapPtr
FindMap(unsigned long address, unsigned long size)
{
    int i;

    for (i = 0; i < numMaps; i++) {
	if (mapList[i]->physaddr == address &&
	    mapList[i]->size == size)
	    return mapList[i];
    }
    return NULL;
}

static ScrPtr
FindScr(Display *display, int screen)
{
    int i;

    for (i = 0; i < numScrs; i++) {
	if (scrList[i]->display == display &&
	    scrList[i]->screen == screen)
	    return scrList[i];
    }
    return NULL;
}

static void *
MapPhysAddress(unsigned long address, unsigned long size)
{
    unsigned long offset, delta;
    int pagesize = -1;
    void *vaddr;
    MapPtr mp;
#if defined(ISC) && defined(HAS_SVR3_MMAP)
    struct kd_memloc mloc;
#elif defined(__EMX__)
    APIRET rc;
    ULONG action;
    HFILE hfd;
#endif

    if ((mp = FindMap(address, size))) {
	mp->refcount++;
	return (void *)((unsigned long)mp->vaddr + mp->delta);
    }

#if defined(_SC_PAGESIZE) && defined(HAS_SC_PAGESIZE)
    pagesize = sysconf(_SC_PAGESIZE);
#endif
#ifdef _SC_PAGE_SIZE
    if (pagesize == -1)
	pagesize = sysconf(_SC_PAGE_SIZE);
#endif
#ifdef HAS_GETPAGESIZE
    if (pagesize == -1)
	pagesize = getpagesize();
#endif
#ifdef PAGE_SIZE
    if (pagesize == -1)
	pagesize = PAGE_SIZE;
#endif
    if (pagesize == -1)
	pagesize = 4096;

   delta = address % pagesize;
   offset = address - delta;

#if defined(ISC) && defined(HAS_SVR3_MMAP)
    if (mapFd < 0) {
	if ((mapFd = open("/dev/mmap", O_RDWR)) < 0)
	    return NULL;
    }
    mloc.vaddr = (char *)0;
    mloc.physaddr = (char *)offset;
    mloc.length = size + delta;
    mloc.ioflg=1;

    if ((vaddr = (void *)ioctl(mapFd, MAP, &mloc)) == (void *)-1)
	return NULL;
#elif defined (__EMX__)
    /*
     * Dragon warning here! /dev/pmap$ is never closed, except on progam exit.
     * Consecutive calling of this routine will make PMAP$ driver run out
     * of memory handles. Some umap/close mechanism should be provided
     */

    rc = DosOpen("/dev/pmap$", &hfd, &action, 0, FILE_NORMAL, FILE_OPEN,
		 OPEN_ACCESS_READWRITE | OPEN_SHARE_DENYNONE, (PEAOP2)NULL);
    if (rc != 0)
	return NULL;
    {
	struct map_ioctl {
		union {
			ULONG phys;
			void* user;
		} a;
		ULONG size;
	} pmap,dmap;
	ULONG plen,dlen;
#define XFREE86_PMAP	0x76
#define PMAP_MAP	0x44

	pmap.a.phys = offset;
	pmap.size = size + delta;
	rc = DosDevIOCtl(hfd, XFREE86_PMAP, PMAP_MAP,
			 (PULONG)&pmap, sizeof(pmap), &plen,
			 (PULONG)&dmap, sizeof(dmap), &dlen);
	if (rc == 0) {
		vaddr = dmap.a.user;
	}
   }
   if (rc != 0)
	return NULL;
#elif defined (Lynx)
    vaddr = (void *)smem_create("XF86DGA", (char *)offset, 
				size + delta, SM_READ|SM_WRITE);
#else
#ifndef MAP_FILE
#define MAP_FILE 0
#endif
    if (mapFd < 0) {
	if ((mapFd = open(DEV_MEM, O_RDWR)) < 0)
	    return NULL;
    }
    vaddr = (void *)mmap(NULL, size + delta, PROT_READ | PROT_WRITE,
                        MAP_FILE | MAP_SHARED, mapFd, (off_t)offset);
    if (vaddr == (void *)-1)
	return NULL;
#endif

    if (!vaddr) {
	if (!(mp = AddMap()))
	    return NULL;
	mp->physaddr = address;
	mp->size = size;
	mp->delta = delta;
	mp->vaddr = vaddr;
	mp->refcount = 1;
    }
    return (void *)((unsigned long)vaddr + delta);
}

/*
 * Still need to find a clean way of detecting the death of a DGA app
 * and returning things to normal - Jon
 * This is here to help debugging without rebooting... Also C-A-BS
 * should restore text mode.
 */

int
SDL_NAME(XF86DGAForkApp)(int screen)
{
    pid_t pid;
    int status;
    int i;

     /* fork the app, parent hangs around to clean up */
    if ((pid = fork()) > 0) {
	ScrPtr sp;

	waitpid(pid, &status, 0);
	for (i = 0; i < numScrs; i++) {
	    sp = scrList[i];
	    SDL_NAME(XF86DGADirectVideoLL)(sp->display, sp->screen, 0);
	    XSync(sp->display, False);
	}
        if (WIFEXITED(status))
	    _exit(0);
	else
	    _exit(-1);
    }
    return pid;
}


Bool
SDL_NAME(XF86DGADirectVideo)(
    Display *dis,
    int screen,
    int enable
){
    ScrPtr sp;
    MapPtr mp = NULL;

    if ((sp = FindScr(dis, screen)))
	mp = sp->map;

    if (enable & XF86DGADirectGraphics) {
#if !defined(ISC) && !defined(HAS_SVR3_MMAP) && !defined(Lynx) \
	&& !defined(__EMX__)
	if (mp && mp->vaddr)
	    mprotect(mp->vaddr, mp->size + mp->delta, PROT_READ | PROT_WRITE);
#endif
    } else {
#if !defined(ISC) && !defined(HAS_SVR3_MMAP) && !defined(Lynx) \
	&& !defined(__EMX__)
	if (mp && mp->vaddr)
	    mprotect(mp->vaddr, mp->size + mp->delta, PROT_READ);
#elif defined(Lynx)
	/* XXX this doesn't allow enable after disable */
	smem_create(NULL, mp->vaddr, mp->size + mp->delta, SM_DETACH);
	smem_remove("XF86DGA");
#endif
    }

    SDL_NAME(XF86DGADirectVideoLL)(dis, screen, enable);
    return 1;
}


static void
XF86cleanup(int sig)
{
    ScrPtr sp;
    int i;
    static char beenhere = 0;

    if (beenhere)
	_exit(3);
    beenhere = 1;

    for (i = 0; i < numScrs; i++) {
	sp = scrList[i];
	SDL_NAME(XF86DGADirectVideo)(sp->display, sp->screen, 0);
	XSync(sp->display, False);
    }
    _exit(3);
}

Bool
SDL_NAME(XF86DGAGetVideo)(
    Display *dis,
    int screen,
    char **addr,
    int *width, 
    int *bank, 
    int *ram
){
    /*unsigned long*/ int offset;
    static int beenHere = 0;
    ScrPtr sp;
    MapPtr mp;

    if (!(sp = FindScr(dis, screen))) {
	if (!(sp = AddScr())) {
	    fprintf(stderr, "XF86DGAGetVideo: malloc failure\n");
	    exit(-2);
	}
	sp->display = dis;
	sp->screen = screen;
	sp->map = NULL;
    }

    SDL_NAME(XF86DGAGetVideoLL)(dis, screen , &offset, width, bank, ram);

    *addr = MapPhysAddress(offset, *bank);
    if (*addr == NULL) {
	fprintf(stderr, "XF86DGAGetVideo: failed to map video memory (%s)\n",
		strerror(errno));
	exit(-2);
    }

    if ((mp = FindMap(offset, *bank)))
	sp->map = mp;

    if (!beenHere) {
	beenHere = 1;
	atexit((void(*)(void))XF86cleanup);
	/* one shot XF86cleanup attempts */
	signal(SIGSEGV, XF86cleanup);
#ifdef SIGBUS
	signal(SIGBUS, XF86cleanup);
#endif
	signal(SIGHUP, XF86cleanup);
	signal(SIGFPE, XF86cleanup);  
    }

    return 1;
}

