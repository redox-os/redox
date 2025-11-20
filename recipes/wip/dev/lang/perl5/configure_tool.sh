# Toolchain detection

tryprog() {
	log "trying $1=$2"
	if command -v $2 1>/dev/null 2>/dev/null; then
		define "$1" "$2"
		result "$2"
		return 0
	else
		return 1
	fi
}

tryfromenv() {
	if [ "$mode" = "buildmini" ]; then
		getenv ev "HOST$2"
	else
		getenv ev "$2"
	fi

	if [ -n "$ev" ]; then
		tryprog $1 "$ev" && return 0
		die "Supplied $ev is not usable"
	fi

	unset ev
	return 1
}

# This is only a function for easy access to return-s
# try.out contains `$cc --version` output.
#
# Figuring out gcc is necessary to make sure -fwrapv fix gets applied.

detect_cc_version() {
	_v=`sed -ne '/^gcc version \([0-9][0-9.]*\).*/s//\1/p' try.out`

	if [ -n "$_v" ]; then
		define cctype 'gcc'
		define ccversion "$_v"
		define gccversion "$_v"
		result "gcc $_v"
		return
	fi

	_v=`sed -ne '/^clang version \([0-9][0-9.]*\).*/s//\1/p' try.out`

	if [ -n "$_v" ]; then
		define cctype 'clang'
		define ccversion "$_v"
		define gccversion "0.0"
		result "clang $_v"
		return
	fi

	define cctype 'cc'
	define ccversion ''
	define gccversion '0.0'
	result 'unknown'
}

# whichprog symbol VAR prog1 prog2
whichprog() {
	mstart "Checking for $1"
	hinted "$1" && return 0

	# Maybe we've got $CC or $HOSTCC?
	tryfromenv "$1" "$2" && return 0

	# For anything that sounds like a native compilation,
	# try no-prefix tools *first*. This is to avoid using
	# long names is case the host happens to have them.
	if [ "$mode" = 'native' -o "$mode" = 'buildmini' ]; then
		tryprog $1 "$3" && return 0
	fi

	# Finally, try $target-gcc
	test -n "$toolsprefix" && tryprog $1 "$toolsprefix$3"  && return 0
	test -n "$target"      && tryprog $1 "$target-$3"      && return 0
	test -n "$targetarch"  && tryprog $1 "$targetarch-$3"  && return 0

	result "none found"
	return 1
}

whichprog cc CC gcc || whichprog cc CC cc || die "No C compiler found"
#whichprog ld LD ld # while correct, this breaks MM library test
whichprog ar AR ar || die "Cannot find ar"
whichprog nm NM nm
whichprog ranlib RANLIB ranlib
whichprog readelf READELF readelf || die "Cannot find readelf"
whichprog objdump OBJDUMP objdump || die "Cannot find objdump"

# XXX: this looks wrong, but the usemmldlt code depends on $ld being able
# to compile try.c. What kind of moron could have written that. Oh wait.
#
# But, there was probably a reason to assume this, likely becase mainline
# Configure did and still does the same. So, ugh, leaving it as is for now.
# Speak of backward bug compatibility.
define ld "$cc"

log

mstart "Trying $cc"
if not hinted 'cctype'; then
	run $cc -v >try.out 2>&1
	try_dump_out
	detect_cc_version
fi

mstart "Checking whether $cc is a C++ compiler"
if not hinted 'd_cplusplus'; then
	try_start
	try_cat <<END
#if defined(__cplusplus)
YES
#endif
END
	try_dump
	if not run $cc $ccflags -E try.c > try.out 2>>$cfglog; then
		define d_cplusplus 'undef'
		result "probably no"
	else
		_r=`grep -v '^#' try.out | grep . | head -1 | grep '^YES'`
		if [ -n "$_r" ]; then
			define d_cplusplus 'define'
			result "yes"
		else
			define d_cplusplus 'undef'
			result 'no'
		fi
	fi
fi

mstart "Deciding how to declare external symbols"
if not hinted "extern_C"; then
	case "$d_cplusplus" in
		define)
			define "extern_C" 'extern "C"'
			result "$extern_C"
			;;
		*)
			define "extern_C" 'extern'
			result "$extern_C"
			;;
	esac
fi

# File name extensions, must be set before running any compile/link tests
define _o '.o'
define _a '.a'
define so 'so'
define _exe ''

# Used only for modules
define cccdlflags '-fPIC -Wno-unused-function'
define ccdlflags '-Wl,-E'

# Misc flags setup
predef lddlflags "-shared"	# modules
predef ccflags ''		# perl and modules
predef ldflags ''		# perl only?
predef cppflags ''		# unused?

# setfromvar what SHELLVAR
setfromenv() {
	getenv v "$2"
	test -n "$v" && append "$1" "$v"
}

if [ "$mode" = 'target' -o "$mode" = 'native' ]; then
	setfromenv ccflags CFLAGS
	setfromenv ldflags LDFLAGS
	if [ -n "$sysroot" ]; then
		msg "Adding --sysroot to {cc,ld}flags"
		prepend ccflags "--sysroot=$sysroot"
		prepend ldflags "--sysroot=$sysroot"
		# While cccdlflags are used together with ccflags,
		# ld is always called with lddlflags *instead*of* ldflags
		prepend lddlflags "--sysroot=$sysroot"
		# Same for cpp
		prepend cppflags "--sysroot=$sysroot"
	fi
elif [ "$mode" = 'buildmini' ]; then
	setfromenv ccflags HOSTCFLAGS
	setfromenv ldflags HOSTLDFLAGS
fi

# Use $ldflags as default value for $lddlflags, together with whatever
# hints provided, but avoid re-setting anyting specified in the command line
if [ -n "$ldflags" -a "$x_lddlflags" != "user" ]; then
	append lddlflags "$ldflags"
fi

# enddef ccflags # done later in _hdrs because of LARGEFILE_SOURCE
enddef ldflags
enddef lddlflags
enddef cppflags

mstart "Checking whether ld supports scripts"
if not hinted 'ld_can_script'; then
	cat > try.c <<EOM
void foo() {}
void bar() {}
EOM
	cat > try.h <<EOM
LIBTEST_42 {
 global:
  foo;
 local: *;
 };
EOM
	log "try.c"
	try_dump
	log "try.h"
	try_dump_h
	rm -f a.out 2>/dev/null

	if run $cc $cccdlflags $ccdlflags $ccflags $lddlflags -o a.out try.c \
		-Wl,--version-script=try.h >/dev/null 2>&1 \
		&& test -s a.out
	then
		define ld_can_script 'define'
		result "yes"
	else
		define ld_can_script 'undef'
		result "no"
	fi
fi

# Guessing OS is better done with the toolchain available.
# CC output is crucial here -- Android toolchains come with
# generic armeabi prefix and "android" is one of the few osname
# values that make difference later.

mstart "Trying to guess target OS"
if not hinted 'osname'; then
	run $cc -v > try.out 2>&1
	try_dump_out

	_ct=`sed -ne '/^Target: /s///p' try.out`
	test -z "$_ct" && _ct="$targetarch"

	case "$_ct" in
		*-mingw32)
			define osname "MSWin32"
			result "MSWin32"
			;;
		*-android|*-androideabi)
			define osname "android"
			result "Android"
			;;
		*-linux*)
			define osname "linux"
			result "Linux"
			;;
		*-netbsd*)
			define osname "netbsd"
			result "NetBSD"
			;;
		*-bsd*)
			define osname "bsd"
			result "BSD"
			;;
		*-gnu*)
			define osname "gnu"
			result "GNU"
			;;
		*-midipix*)
			define osname "midipix"
			result "Midipix"
			;;
		*-redox*)
			define osname "redox"
			result "Redox"
			;;
		*)
			result "no"
			;;
	esac
fi

# Check whether debugging should be enabled
# Allow -DEBUGGING as well (sets EBUGGING=define)
case "$DEBUGGING:$EBUGGING" in
	:*)
		DEBUGGING=$EBUGGING
		;;
esac

mstart "Checking whether to enable -g"
predef optimize ''
case "$DEBUGGING" in
	both|define)
		append optimize "-g"
		result "yes" ;;
	*)
		result "no" ;;
esac

mstart "Checking whether to use -DDEBUGGING"
case "$DEBUGGING" in
	both|define)
		append optimize '-DDEBUGGING'
		result "yes" ;;
	*)
		result "no" ;;
esac

# gcc 4.9 and above does some optimizations that break perl.
# see perl ticket 121505.
if [ "$cctype" = 'gcc' ]; then
	case "$ccversion" in
		1.*|2.*|3.*) ;;
		4.9*) append 'optimize' '-fwrapv -fno-strict-aliasing' ;;
		4.*) ;;
		*) append 'optimize' '-fwrapv -fno-strict-aliasing' ;;
	esac
fi
enddef optimize

# These are kind-of part of toolchain, but we do not test them

# For newer gcc-s, -E alone is *not* enough! Perl expects cpp not to break
# lines, but gcc injects #line directives in-between tokens, subtly breaking
# try_preproc and Errno.pm
define cpp "$cc -E -P"
define cpprun "$cpp"
define cppstdin "$cpp"

define cpplast -
define cppminus -
define cppsymbols

define nm_opt
define nm_so_opt

# cperl wants to know this for some reason
mstart "Checking whether address sanitizer is enabled"
if not hinted sanitize_address 'yes' 'no'; then
	case "$ccflags" in
		*-fsanitize=address*|*-faddress-sanitizer*)
			define sanitize_address 'define'
			result 'yes'
			;;
		*)
			define sanitize_address 'undef'
			result 'no'
			;;
	esac
fi
