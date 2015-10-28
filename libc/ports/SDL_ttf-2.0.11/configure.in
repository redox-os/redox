dnl Process this file with autoconf to produce a configure script.
AC_INIT(README)

dnl Set various version strings - taken gratefully from the GTk sources

# Making releases:
#   MICRO_VERSION += 1;
#   INTERFACE_AGE += 1;
#   BINARY_AGE += 1;
# if any functions have been added, set INTERFACE_AGE to 0.
# if backwards compatibility has been broken,
# set BINARY_AGE and INTERFACE_AGE to 0.

MAJOR_VERSION=2
MINOR_VERSION=0
MICRO_VERSION=11
INTERFACE_AGE=1
BINARY_AGE=11
VERSION=$MAJOR_VERSION.$MINOR_VERSION.$MICRO_VERSION

AC_SUBST(MAJOR_VERSION)
AC_SUBST(MINOR_VERSION)
AC_SUBST(MICRO_VERSION)
AC_SUBST(INTERFACE_AGE)
AC_SUBST(BINARY_AGE)
AC_SUBST(VERSION)

# libtool versioning
LT_INIT([win32-dll])

LT_RELEASE=$MAJOR_VERSION.$MINOR_VERSION
LT_CURRENT=`expr $MICRO_VERSION - $INTERFACE_AGE`
LT_REVISION=$INTERFACE_AGE
LT_AGE=`expr $BINARY_AGE - $INTERFACE_AGE`

AC_SUBST(LT_RELEASE)
AC_SUBST(LT_CURRENT)
AC_SUBST(LT_REVISION)
AC_SUBST(LT_AGE)

dnl Detect the canonical build and host environments
AC_CANONICAL_HOST

dnl Setup for automake
AM_INIT_AUTOMAKE(SDL_ttf, $VERSION)

dnl Check for tools
AC_PROG_LIBTOOL
AC_PROG_CC
AC_PROG_INSTALL
AC_PROG_MAKE_SET
if test -z "$host_alias"; then
    hostaliaswindres=
else
    hostaliaswindres="$host_alias-windres"
fi
AC_CHECK_PROGS(WINDRES, [windres $hostaliaswindres $host_os-windres])

case "$host" in
    *-*-cygwin* | *-*-mingw32*)
	;;
    *)
	AC_FUNC_ALLOCA
	;;
esac


dnl Check for iconv (character conversion library; see iconv.m4)
dnl This isn't available on many systems
dnl AM_ICONV

case "$host" in
    *-*-beos*)
        ac_default_prefix=/boot/develop/tools/gnupro
        ;;
    *-*-cygwin* | *-*-mingw32*)
        if test "$build" != "$host"; then # cross-compiling
            # Default cross-compile location
            ac_default_prefix=/usr/local/cross-tools/i386-mingw32
        else
            # Look for the location of the tools and install there
            if test "$BUILD_PREFIX" != ""; then
                ac_default_prefix=$BUILD_PREFIX
            fi
        fi
        if test x$WINDRES != x; then
            use_version_rc=true
        fi
        ;;
esac
AM_CONDITIONAL(USE_VERSION_RC, test x$use_version_rc = xtrue)

dnl Check for the FreeType 2 library
dnl
dnl Get the cflags and libraries from the freetype-config script
dnl
AC_ARG_WITH(freetype-prefix,[  --with-freetype-prefix=PFX   Prefix where FREETYPE is 
installed (optional)],
            freetype_prefix="$withval", freetype_prefix="")
AC_ARG_WITH(freetype-exec-prefix,[  --with-freetype-exec-prefix=PFX Exec prefix 
where FREETYPE is installed (optional)],
            freetype_exec_prefix="$withval", freetype_exec_prefix="")

if test x$freetype_exec_prefix != x ; then
     freetype_args="$freetype_args --exec-prefix=$freetype_exec_prefix"
     if test x${FREETYPE_CONFIG+set} != xset ; then
        FREETYPE_CONFIG=$freetype_exec_prefix/bin/freetype-config
     fi
fi
if test x$freetype_prefix != x ; then
     freetype_args="$freetype_args --prefix=$freetype_prefix"
     if test x${FREETYPE_CONFIG+set} != xset ; then
        FREETYPE_CONFIG=$freetype_prefix/bin/freetype-config
     fi
fi
AC_PATH_PROG(FREETYPE_CONFIG, freetype-config, no)
no_freetype=""
if test "$FREETYPE_CONFIG" = "no" ; then
    AC_MSG_ERROR([
*** Unable to find FreeType2 library (http://www.freetype.org/)
])
else
    CFLAGS="$CFLAGS `$FREETYPE_CONFIG $freetypeconf_args --cflags`"
    LIBS="$LIBS `$FREETYPE_CONFIG $freetypeconf_args --libs`"
fi

dnl Check for SDL
SDL_VERSION=1.2.4
AC_SUBST(SDL_VERSION)
AM_PATH_SDL($SDL_VERSION,
            :,
	    AC_MSG_ERROR([*** SDL version $SDL_VERSION not found!])
)
CFLAGS="$CFLAGS $SDL_CFLAGS"
LIBS="$LIBS $SDL_LIBS"

dnl Check for OpenGL
case "$host" in
    *-*-cygwin* | *-*-mingw32*)
        MATHLIB=""
        SYS_GL_LIBS="-lopengl32"
        ;;
    *-*-beos*)
        MATHLIB=""
        SYS_GL_LIBS="-lGL"
        ;;
    *-*-darwin*)
        MATHLIB=""
        SYS_GL_LIBS="-Wl,-framework,OpenGL"
        ;;
    *-*-aix*)
        if test x$ac_cv_prog_gcc = xyes; then
            CFLAGS="-mthreads"
	fi
        SYS_GL_LIBS=""
        ;;
    *)
        MATHLIB="-lm"
        AC_PATH_X
        AC_PATH_XTRA
        if test x$have_x = xyes; then
            CFLAGS="$CFLAGS $X_CFLAGS"
            SYS_GL_LIBS="$X_LIBS -lGL"
        else
            SYS_GL_LIBS="-lGL"
        fi
        ;;
esac
AC_MSG_CHECKING(for OpenGL support)
have_opengl=no
AC_TRY_COMPILE([
 #include "SDL_opengl.h"
],[
 GLuint texture;
],[
have_opengl=yes
])
AC_MSG_RESULT($have_opengl)
if test x$have_opengl = xyes; then
    CFLAGS="$CFLAGS -DHAVE_OPENGL"
    GL_LIBS="$SYS_GL_LIBS"
else
    GL_LIBS=""
fi
AC_SUBST([GL_LIBS])
AC_SUBST([MATHLIB])
AC_SUBST([WINDRES])

# Finally create all the generated files
AC_OUTPUT([
Makefile
SDL_ttf.spec
SDL_ttf.pc
])
