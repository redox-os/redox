#!/usr/bin/env bash

##########################################################
# This function is simply a banner to introduce the script
##########################################################
banner()
{
	echo "|------------------------------------------|"
	echo "|----- Welcome to the redox bootstrap -----|"
	echo "|-------- for building with Podman --------|"
	echo "|------------------------------------------|"
}

###################################################################################
# This function takes care of installing a dependency via package manager of choice
# for building redox on BSDs (MacOS, FreeBSD, etc.).
# @params:    $1 package manager
#            $2 package name
#            $3 binary name (optional)
###################################################################################
install_bsd_pkg()
{
    PKG_MANAGER=$1
    PKG_NAME=$2
    BIN_NAME=$3
    if [ -z "$BIN_NAME" ]; then
        BIN_NAME=$PKG_NAME
    fi

    BIN_LOCATION=$(which $BIN_NAME || true)
    if [ -z "$BIN_LOCATION" ]; then
        echo "$PKG_MANAGER install $PKG_NAME"
        $PKG_MANAGER install "$PKG_NAME"
    else
        echo "$BIN_NAME already exists at $BIN_LOCATION, no need to install $PKG_NAME..."
    fi
}

install_macports_pkg()
{
    install_bsd_pkg "sudo port" "$1" "$2"
}

install_brew_pkg()
{
    install_bsd_pkg "brew" $@
}

install_brew_cask_pkg()
{
    install_bsd_pkg "brew cask" $@
}

install_freebsd_pkg()
{
    install_bsd_pkg "sudo pkg" $@
}

###############################################################################
# This function checks which of the supported package managers
# is available on the OSX Host.
# If a supported package manager is found, it delegates the installing work to
# the relevant function.
# Otherwise this function will exit this script with an error.
###############################################################################
osx()
{
    echo "Detected OSX!"

    if [ ! -z "$(which brew)" ]; then
        osx_homebrew $@
    elif [ ! -z "$(which port)" ]; then
        osx_macports $@
    else
        echo "Please install either Homebrew or MacPorts, if you wish to use this script"
        echo "Re-run this script once you installed one of those package managers"
        echo "Will not install, now exiting..."
        exit 1
    fi
}

###############################################################################
# This function takes care of installing all dependencies using MacPorts
# for building redox on Mac OSX
# @params:    $1 the emulator to install, virtualbox or qemu
###############################################################################
osx_macports()
{
    echo "Macports detected! Now updating..."
    sudo port -v selfupdate

    echo "Installing missing packages..."

    install_macports_pkg "git"
    install_macports_pkg "cmake"

    if [ "$1" == "qemu" ]; then
        install_macports_pkg "qemu" "qemu-system-x86_64"
    else
        install_macports_pkg "virtualbox"
    fi

    install_macports_pkg "osxfuse"
	install_macports_pkg "podman"
}

###############################################################################
# This function takes care of installing all dependencies using Homebrew
# for building redox on Mac OSX
# @params:    $1 the emulator to install, virtualbox or qemu
###############################################################################
osx_homebrew()
{
    echo "Homebrew detected! Now updating..."
    brew update

    echo "Installing missing packages..."

    install_brew_pkg "git"

    if [ "$1" == "qemu" ]; then
        install_brew_pkg "qemu" "qemu-system-x86_64"
    else
        install_brew_pkg "virtualbox"
    fi

    install_brew_pkg "make"
	install_brew_pkg "podman"
}

###############################################################################
# This function takes care of installing all dependencies using pkg
# for building redox on FreeBSD
# @params:    $1 the emulator to install, virtualbox or qemu
###############################################################################
freebsd()
{
    set -xe
    echo "FreeBSD detected!"
    echo "Installing missing packages..."

    install_freebsd_pkg "git"

    if [ "$1" == "qemu" ]; then
        install_freebsd_pkg "qemu" "qemu-system-x86_64"
    else
        install_freebsd_pkg "virtualbox"
    fi

    install_freebsd_pkg "gmake"
    install_freebsd_pkg "podman"
    set +xe
}

###############################################################################
# This function takes care of installing all dependencies for building redox on
# Arch linux
# @params:	$1 the emulator to install, virtualbox or qemu
###############################################################################
archLinux()
{
	echo "Detected Arch Linux"
	packages="git podman fuse"
	if [ "$1" == "qemu" ]; then
		packages="$packages qemu"
	elif [ "$1" == "virtualbox" ]; then
		packages="$packages virtualbox"
	fi

	# Scripts should not cause a system update in order to just install a couple
	#   of packages. If pacman -S --needed is going to fail, let it fail and the
	#   user will figure out the issues (without updating if required) and rerun
	#   the script.
	#echo "Updating system..."
	#sudo pacman -Syu

	echo "Installing packages $packages..."
	sudo pacman -S --needed $packages
}

###############################################################################
# This function takes care of installing all dependencies for building redox on
# debian based linux
# @params:	$1 the emulator to install, virtualbox or qemu
# 		$2 the package manager to use
###############################################################################
ubuntu()
{
	echo "Detected Ubuntu/Debian"
	echo "Updating system..."
	sudo "$2" update
	echo "Installing required packages..."
	sudo "$2" install \
		podman curl git make libfuse-dev
	if [ "$1" == "qemu" ]; then
		if [ -z "$(which qemu-system-x86_64)" ]; then
			echo "Installing QEMU..."
			sudo "$2" install qemu-system-x86 qemu-kvm
			sudo "$2" install qemu-efi-arm qemu-system-arm
		else
			echo "QEMU already installed!"
		fi
	else
		if [ -z "$(which virtualbox)" ]; then
			echo "Installing Virtualbox..."
			sudo "$2" install virtualbox
		else
			echo "Virtualbox already installed!"
		fi
	fi
}

###############################################################################
# This function takes care of installing all dependencies for building redox on
# fedora linux
# @params:	$1 the emulator to install, virtualbox or qemu
###############################################################################
fedora()
{
	echo "Detected Fedora"
	if [ -z "$(which git)" ]; then
		echo "Installing git..."
		sudo dnf install git-all
	fi
	if [ "$1" == "qemu" ]; then
		if [ -z "$(which qemu-system-x86_64)" ]; then
			echo "Installing QEMU..."
			sudo dnf install qemu-system-x86 qemu-kvm
		else
			echo "QEMU already installed!"
		fi
	else
		if [ -z "$(which virtualbox)" ]; then
			echo "Installing virtualbox..."
			sudo dnf install virtualbox
		else
			echo "Virtualbox already installed!"
		fi
	fi
	# Use rpm -q <package> to check if it's already installed
	PKGS=$(for pkg in podman; do rpm -q $pkg > /dev/null || echo $pkg; done)
	# If the list of packages is not empty, install missing
	COUNT=$(echo $PKGS | wc -w)
	if [ $COUNT -ne 0 ]; then
					echo "Installing necessary build tools..."
					sudo dnf install $PKGS
	fi
}

###############################################################################
# This function takes care of installing all dependencies for building redox on
# *suse linux
# @params:	$1 the emulator to install, virtualbox or qemu
###############################################################################
suse()
{
	echo "Detected SUSE Linux"
	if [ -z "$(which git)" ]; then
		echo "Installing git..."
		zypper install git
	fi
	if [ "$1" == "qemu" ]; then
		if [ -z "$(which qemu-system-x86_64)" ]; then
			echo "Installing QEMU..."
			sudo zypper install qemu-x86 qemu-kvm
		else
			echo "QEMU already installed!"
		fi
	else
		if [ -z "$(which virtualbox)" ]; then
			echo "Please install Virtualbox and re-run this script,"
			echo "or run with -e qemu"
			exit
		else
			echo "Virtualbox already installed!"
		fi
	fi
	echo "Installing necessary build tools..."
	sudo zypper install make fuse-devel podman
}

##############################################################################
# This function takes care of installing all dependencies for building redox on
# gentoo linux
# @params:	$1 the emulator to install, virtualbox or qemu
##############################################################################
gentoo()
{
	echo "Detected Gentoo Linux"
	if [ -z "$(which git)" ]; then
		echo "Installing git..."
		sudo emerge dev-vcs/git
	fi
	if [ -z "$(which fusermount 2>/dev/null)" ] && [ -z "$(which fusermount3 2>/dev/null)" ]; then
		echo "Installing fuse..."
		sudo emerge sys-fs/fuse
	fi
	if [ "$2" == "qemu" ]; then
		if [ -z "$(which qemu-system-x86_64)" ]; then
			echo "Please install QEMU and re-run this script"
			echo "Step1. Add QEMU_SOFTMMU_TARGETS=\"x86_64\" to /etc/portage/make.conf"
			echo "Step2. Execute \"sudo emerge app-emulation/qemu\""
		else
			echo "QEMU already installed!"
		fi
	fi
	if [ -z "$(which cmake)" ]; then
		echo "Installing cmake..."
		sudo emerge dev-util/cmake
	fi
	if [ -z "$(which podman)" ]; then
		echo "Please install Podman, https://wiki.gentoo.org/wiki/Podman"
	fi
}

##############################################################################
# This function takes care of installing all dependencies for building redox on
# SolusOS
# @params:	$1 the emulator to install, virtualbox or qemu
##############################################################################
solus()
{
	echo "Detected SolusOS"

	if [ "$1" == "qemu" ]; then
		if [ -z "$(which qemu-system-x86_64)" ]; then
			sudo eopkg it qemu
		else
			echo "QEMU already installed!"
		fi
	else
		if [ -z "$(which virtualbox)" ]; then
			echo "Please install Virtualbox and re-run this script,"
			echo "or run with -e qemu"
			exit
		else
			echo "Virtualbox already installed!"
		fi
	fi

	echo "Installing necessary build tools..."
	#if guards are not necessary with eopkg since it does nothing if latest version is already installed
	sudo eopkg it fuse-devel git make fuse2-devel rsync
	if [ -z "$(which podman)" ]; then
		echo "Please install Podman"
	fi
}

######################################################################
# This function outlines the different options available for bootstrap
######################################################################
usage()
{
	echo "------------------------"
	echo "|Redox bootstrap script|"
	echo "------------------------"
	echo "Usage: ./podman_bootstrap.sh"
	echo "OPTIONS:"
	echo
	echo "   -h,--help      Show this prompt"
	echo "   -u [branch]    Update git repo and update rust"
	echo "                  If blank defaults to master"
	echo "   -s             Check the status of the current travis build"
	echo "   -e [emulator]  Install specific emulator, virtualbox or qemu"
	echo "   -p [package    Choose an Ubuntu package manager, apt-fast or"
	echo "       manager]   aptitude"
	echo "   -d             Only install the dependencies, skip boot step"
	echo "EXAMPLES:"
	echo
	echo "./podman_bootstrap.sh -e qemu"
	exit
}


#############################################################
# Looks for and installs a cargo-managed binary or subcommand
#############################################################
cargoInstall() {
	if [[ "`cargo install --list`" != *"$1 v$2"* ]]; then
		cargo install --force --version "$2" "$1"
	else
		echo "You have $1 version $2 installed already!"
	fi
}

####################################################################
# This function gets the current build status from travis and prints
# a message to the user
####################################################################
statusCheck() {
	for i in $(echo "$(curl -sf https://api.travis-ci.org/repositories/redox-os/redox.json)" | tr "," "\n")
	do
		if echo "$i" | grep -iq "last_build_status" ;then
			if echo "$i" | grep -iq "0" ;then
				echo
				echo "********************************************"
				echo "Travis reports that the last build succeeded!"
				echo "Looks like you are good to go!"
				echo "********************************************"
			elif echo "$i" | grep -iq "null" ;then
				echo
				echo "******************************************************************"
				echo "The Travis build did not finish, this is an error with its config."
				echo "I cannot reliably determine whether the build is succeeding or not."
				echo "Consider checking for and maybe opening an issue on gitlab"
				echo "******************************************************************"
			else
				echo
				echo "**************************************************"
				echo "Travis reports that the last build *FAILED* :("
				echo "Might want to check out the issues before building"
				echo "**************************************************"
			fi
		fi
	done
}

###########################################################################
# This function is the main logic for the bootstrap; it clones the git repo
# then it installs the dependent packages
###########################################################################
boot()
{
	echo "Cloning gitlab repo..."
	git clone https://gitlab.redox-os.org/redox-os/redox.git --origin upstream --recursive
	echo "Creating .config with PODMAN_BUILD=1"
	echo 'PODMAN_BUILD?=1' > redox/.config
	echo "Cleaning up..."
	rm podman_bootstrap.sh
	echo
	echo "---------------------------------------"
	echo "Well it looks like you are ready to go!"
	echo "---------------------------------------"
	statusCheck
	echo "The file redox/.config was created with PODMAN_BUILD=1."
	echo "Run the following commands to build redox using Podman:"
	echo
	echo "cd redox"
	MAKE="make"
	if [[ "$(uname)" == "FreeBSD" ]]; then
    	MAKE="gmake"
    	echo "kldload fuse.ko # This loads the kernel module for fuse"
    fi
	echo "$MAKE all"
	echo "$MAKE virtualbox or qemu"
	echo
	echo "You can also edit mk/config.mk and change PODMAN_BUILD to 1 so"
	echo "you don't need to specify it on the command line."
	echo
	echo "      Good luck!"

	exit
}

if [ "$1" == "-h" ] || [ "$1" == "--help" ]; then
	usage
elif [ "$1" == "-u" ]; then
	git pull upstream master
	git submodule update --recursive --init
	rustup update nightly
	exit
elif [ "$1" == "-s" ]; then
	statusCheck
	exit
fi

emulator="qemu"
defpackman="apt-get"
dependenciesonly=false
update=false
while getopts ":e:p:udhs" opt
do
	case "$opt" in
		e) emulator="$OPTARG";;
		p) defpackman="$OPTARG";;
		d) dependenciesonly=true;;
		u) update=true;;
		h) usage;;
		s) statusCheck && exit;;
		\?) echo "I don't know what to do with that option, try -h for help"; exit;;
	esac
done

banner

if [ "$update" == "true" ]; then
	git pull upstream master
	git submodule update --recursive --init
	exit
fi

if [ "Darwin" == "$(uname -s)" ]; then
	osx "$emulator"
else
	# Here we will use package managers to determine which operating system the user is using.

	# Suse and derivatives
	if hash 2>/dev/null zypper; then
		suse "$emulator"
	# Debian or any derivative of it
	elif hash 2>/dev/null apt-get; then
		ubuntu "$emulator" "$defpackman"
	# Fedora
	elif hash 2>/dev/null dnf; then
		fedora "$emulator"
	# Gentoo
	elif hash 2>/dev/null emerge; then
		gentoo "$emulator"
	# SolusOS
	elif hash 2>/dev/null eopkg; then
		solus "$emulator"
	# Arch linux
	elif hash 2>/dev/null pacman; then
		archLinux "$emulator"
	# FreeBSD
	elif hash 2>/dev/null pkg; then
		freebsd "$emulator"
	# Unsupported platform
	else
    	printf "\e[31;1mFatal error: \e[0;31mUnsupported platform, please open an issue\[0m"
	fi
fi

if [ "$dependenciesonly" = false ]; then
	boot
fi

echo "Redox bootstrap complete!"
