#!/usr/bin/env bash

# Alternative script for the build system Makefiles

###########################################################################
#                                                                         #
# Build the system, with a specified processor type and filesystem config #
#                                                                         #
###########################################################################

usage()
{
    echo "build.sh:      Invoke make for a particular architecture and configuration."
    echo "Usage:"
    echo "./build.sh [-X | -A | -6 | -a ARCH] [-c CONFIG] [-f FILESYSTEM_CONFIG] TARGET..."
    echo "    -X         Equivalent to -a x86_64."
    echo "    -A         Equivalent to -a aarch64."
    echo "    -6         Equivalent to -a i686."
    echo "    -a ARCH:   Processor Architecture. Normally one of x86_64, aarch64 or"
    echo "               i686. ARCH is not checked, so you can add a new architecture."
    echo "               Defaults to the directory containing the FILESYSTEM_CONFIG file,"
    echo "               or x86_64 if no FILESYSTEM_CONFIG is specified."
    echo "    -c CONFIG: The name of the config, e.g. desktop, server or demo."
    echo "               Determines the name of the image, build/ARCH/CONFIG/harddrive.img"
    echo "               e.g. build/x86_64/desktop/harddrive.img"
    echo "               Determines the name of FILESYSTEM_CONFIG if none is specified."
    echo "               Defaults to the basename of FILESYSTEM_CONFIG, or 'desktop'"
    echo "               if FILESYSTEM_CONFIG is not specified."
    echo "    -f FILESYSTEM_CONFIG:"
    echo "               The config file to use. It can be in any location."
    echo "               However, if the file is not in a directory named x86_64, aarch64"
    echo "               or i686, you must specify the architecture."
    echo "               If -f is not specified, FILESYSTEM_CONFIG is set to"
    echo "               config/ARCH/CONFIG.toml"
    echo "               If you specify both CONFIG and FILESYSTEM_CONFIG, it is not"
    echo "               necessary that they match, but it is recommended."
    echo "    Examples:  ./build.sh -c demo live - make build/x86_64/demo/livedisk.iso"
    echo "               ./build.sh -6 qemu - make build/i686/desktop/harddrive.img and"
    echo "                                    and run it in qemu"
    echo "    NOTE:      If you do not change ARCH or CONFIG very often, edit mk/config.mk"
    echo "               and set ARCH and FILESYSTEM_CONFIG. You only need to use this"
    echo "               script when you want to override them."
}

if [ "$1" == "-h" ] || [ "$1" == "--help" ]; then
	usage
    exit
fi

defaultarch="x86_64"
defaultname="desktop"
ARCH=""
CONFIG_NAME=""
FILESYSTEM_CONFIG=""

while getopts ":c:f:a:dhXA6" opt
do
	case "$opt" in
   		a) ARCH="$OPTARG";;
		c) CONFIG_NAME="$OPTARG";;
		f) FILESYSTEM_CONFIG="$OPTARG";;
        X) ARCH="x86_64";;
        A) ARCH="aarch64";;
        6) ARCH="i686";;
		h) usage;;
		\?) echo "Unknown option -$OPTARG, try -h for help"; exit;;
        :) echo "-$OPTARG requires a value"; exit;;
	esac
done
shift $((OPTIND -1))

if [ -z "$ARCH" ] && [ -n "$FILESYSTEM_CONFIG" ]; then
    dirname=`dirname "$FILESYSTEM_CONFIG"`
    ARCH=`basename $dirname`
    case "$ARCH" in
        x86_64) : ;;
        aarch64) : ;;
        i686) : ;;
        \?) ARCH=""; echo "Unknown Architecture, please specify x86_64, aarch64 or i686";;
    esac
fi

if [ -z "$config_name" ] && [ -n "$FILESYSTEM_CONFIG" ]; then
    CONFIG_NAME=`basename "$FILESYSTEM_CONFIG" .toml`
fi

if [ -z "$ARCH" ]; then
    ARCH="$defaultarch"
fi

if [ -z "$CONFIG_NAME" ]; then
    CONFIG_NAME="$defaultname"
fi

if [ -z "$FILESYSTEM_CONFIG" ]; then
    FILESYSTEM_CONFIG="config/$ARCH/$CONFIG_NAME.toml"
fi

export ARCH CONFIG_NAME FILESYSTEM_CONFIG
make $@
