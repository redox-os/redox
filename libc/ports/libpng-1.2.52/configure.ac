# configure.ac

dnl Process this file with autoconf to produce a configure script.
dnl
dnl Minor upgrades (compatible ABI): increment the package version
dnl (third field in two places below) and set the PNGLIB_RELEASE
dnl variable.
dnl
dnl Major upgrades (incompatible ABI): increment the package major
dnl version (second field, or first if desired), set the minor
dnl to 0, set PNGLIB_MAJOR below *and* follow the instructions in
dnl Makefile.am to upgrade the package name.

dnl This is here to prevent earlier autoconf from being used, it
dnl should not be necessary to regenerate configure if the time
dnl stamps are correct
AC_PREREQ(2.59)

dnl Version number stuff here:

AC_INIT([libpng], [1.2.52], [png-mng-implement@lists.sourceforge.net])
AM_INIT_AUTOMAKE
dnl stop configure from automagically running automake
AM_MAINTAINER_MODE

PNGLIB_VERSION=1.2.52
PNGLIB_MAJOR=1
PNGLIB_MINOR=2
PNGLIB_RELEASE=52

dnl End of version number stuff

AC_CONFIG_SRCDIR([pngget.c])
AC_CONFIG_HEADERS([config.h])

# Checks for programs.
AC_PROG_CC
AC_PROG_LD
AC_PROG_CPP
AC_CHECK_TOOL(SED, sed, :)
AC_LIBTOOL_WIN32_DLL
AC_PROG_INSTALL
AC_PROG_LN_S
AC_PROG_MAKE_SET
AC_PROG_LIBTOOL

# Checks for header files.
AC_HEADER_STDC
AC_CHECK_HEADERS([malloc.h stdlib.h string.h strings.h])

# Checks for typedefs, structures, and compiler characteristics.
AC_C_CONST
AC_TYPE_SIZE_T
AC_STRUCT_TM

# Checks for library functions.
AC_FUNC_STRTOD
AC_CHECK_FUNCS([memset], , AC_ERROR([memset not found in libc]))
AC_CHECK_FUNCS([pow], , AC_CHECK_LIB(m, pow, , AC_ERROR([cannot find pow])) )
AC_CHECK_LIB(z, zlibVersion, , AC_ERROR([zlib not installed]))

case $host_os in
  aix*)
   LIBPNG_DEFINES=-DPNG_CONFIGURE_LIBPNG -D_ALL_SOURCE;;
  *)
   LIBPNG_DEFINES=-DPNG_CONFIGURE_LIBPNG;;
esac
AC_MSG_CHECKING(
  [if assembler code in pnggccrd.c can be compiled without PNG_NO_MMX_CODE])
AC_TRY_COMPILE(
  [#include "$srcdir/pnggccrd.c"],
  [return 0;],
  AC_MSG_RESULT(yes)
  LIBPNG_NO_MMX="",
  AC_MSG_RESULT(no)
  LIBPNG_NO_MMX=-DPNG_NO_MMX_CODE)
LIBPNG_DEFINES=$LIBPNG_DEFINES\ $LIBPNG_NO_MMX
AC_SUBST(LIBPNG_DEFINES)
AC_SUBST(LIBPNG_NO_MMX)

AC_MSG_CHECKING([if libraries can be versioned])
GLD=`$LD --help < /dev/null 2>/dev/null | grep version-script`
if test "$GLD"; then
    have_ld_version_script=yes
    AC_MSG_RESULT(yes)
else
    have_ld_version_script=no
    AC_MSG_RESULT(no)
    AC_MSG_WARN(*** You have not enabled versioned symbols.)
fi
AM_CONDITIONAL(HAVE_LD_VERSION_SCRIPT, test "$have_ld_version_script" = "yes")

if test "$have_ld_version_script" = "yes"; then
    AC_MSG_CHECKING([for symbol prefix])
    SYMBOL_PREFIX=`echo "PREFIX=__USER_LABEL_PREFIX__" \
                   | ${CPP-${CC-gcc} -E} - 2>&1 \
                   | ${EGREP-grep} "^PREFIX=" \
                   | ${SED-sed} "s:^PREFIX=::"`
    AC_SUBST(SYMBOL_PREFIX)
    AC_MSG_RESULT($SYMBOL_PREFIX)
fi

# Substitutions for .in files
AC_SUBST(PNGLIB_VERSION)
AC_SUBST(PNGLIB_MAJOR)
AC_SUBST(PNGLIB_MINOR)
AC_SUBST(PNGLIB_RELEASE)

# Additional arguments (and substitutions)
# Allow the pkg-config directory to be set
AC_ARG_WITH(pkgconfigdir,
	AC_HELP_STRING([--with-pkgconfigdir],
	[Use the specified pkgconfig dir (default is libdir/pkgconfig)]),
	[pkgconfigdir=${withval}],
	[pkgconfigdir='${libdir}/pkgconfig'])

AC_SUBST([pkgconfigdir])
AC_MSG_NOTICE([pkgconfig directory is ${pkgconfigdir}])

# Make the *-config binary config scripts optional
AC_ARG_WITH(binconfigs,
	AC_HELP_STRING([--with-binconfigs],
		[Generate shell libpng-config scripts as well as pkg-config data]
		[@<:@default=yes@:>@]),
	[if test "${withval}" = no; then
		binconfigs=
		AC_MSG_NOTICE([libpng-config scripts will not be built])
	else
		binconfigs='${binconfigs}'
	fi],
	[binconfigs='${binconfigs}'])
AC_SUBST([binconfigs])

# Allow the old version number library, libpng.so, to be removed from
# the build
AC_ARG_WITH(libpng-compat,
	AC_HELP_STRING([--with-libpng-compat],
		[Generate the obsolete libpng.so library @<:@default=yes@:>@]),
	[if test "${withval}" = no; then
		compatlib=
		AC_MSG_NOTICE([libpng.so will not be built])
	else
		compatlib=libpng.la
	fi],
	[compatlib=libpng.la])
AC_SUBST([compatlib])

# Config files, substituting as above
AC_CONFIG_FILES([Makefile libpng.pc:scripts/libpng.pc-configure.in])
AC_CONFIG_FILES([libpng-config:scripts/libpng-config.in],
		[chmod +x libpng-config])

AC_OUTPUT
