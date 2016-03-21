#! /bin/bash
banner() {
	echo "|------------------------------------------|"
	echo "|----- Welcome to the redox bootstrap -----|"
	echo "|------------------------------------------|"
}

osx()
{
	echo "Detected OSX!"
	if [ ! -z "$(which brew)" ]; then
		echo "Homebrew detected! Now updating..."
		brew update
		if [ -z "$(which git)" ]; then
			echo "Now installing git..."
			brew install git
		fi
		if [ "$2" == "qemu" ]; then
			if [ -z "$(which qemu-system-i386)" ]; then
				echo "Installing qemu..."
				brew install qemu
			else
				echo "QEMU already installed!"
			fi
		else
			if [ -z "$(which virtualbox)" ]; then
				echo "Now installing virtualbox..."
				brew cask install virtualbox
			else
				echo "Virtualbox already installed!"
			fi
		fi
	else
		echo "Homebrew does not appear to be installed! Would you like me to install it?"
		printf "(Y/n): "
		read -r installit
		if [ "$installit" == "Y" ]; then
			ruby -e "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install)"
		else
			echo "Will not install, now exiting..."
			exit
		fi
	fi
	echo "Cloning Redox repo"
	git clone -b "$1" --recursive https://github.com/redox-os/redox.git
	echo "Running Redox setup script..."
	brew tap homebrew/versions
	brew install gcc49
	brew tap Nashenas88/homebrew-gcc_cross_compilers
	brew install i386-elf-binutils i386-elf-gcc nasm
	echo "Running rust install script"
	sh redox/setup/binary.sh
	endMessage
}

archLinux()
{
	echo "Detected Arch Linux"
	echo "Updating system..."
	sudo pacman -Syu
    if [ -z "$(which nasm)" ]; then
        echo "Installing nasm..."
        sudo pacman -S nasm
    fi
	if [ -z "$(which git)" ]; then
		echo "Installing git..."
		sudo pacman -S git
	fi
	if [ "$2" == "qemu" ]; then
		if [ -z "$(which qemu-system-i386)" ]; then
			echo "Installing QEMU..."
			sudo pacman -S qemu
		else
			echo "QEMU already installed!"
		fi
	fi
	echo "Cloning redox repo..."
	git clone -b "$1" --recursive https://github.com/redox-os/redox.git
	echo "Running Redox setup scripts..."
	sh redox/setup/arch.sh
	echo "Running rust installer..."
	sh redox/setup/binary.sh
}

ubuntu()
{
	echo "Detected Ubuntu/Debian"
	echo "Updating system..."
	sudo "$3" update
	echo "Installing required packages..."
	sudo "$3" install build-essential libc6-dev-i386 nasm curl file git
	if [ "$2" == "qemu" ]; then
		if [ -z "$(which qemu-system-i386)" ]; then
			echo "Installing QEMU..."
			sudo "$3" install qemu-system-x86 qemu-kvm
		else
			echo "QEMU already installed!"
		fi
	else
		if [ -z "$(which virtualbox)" ]; then
			echo "Installing Virtualbox..."
			sudo "$3" install virtualbox
		else
			echo "Virtualbox already installed!"
		fi
	fi
	echo "Cloning Redox repo"
	git clone -b "$1" --recursive https://github.com/redox-os/redox.git
	echo "Running rust installer..."
	sh redox/setup/binary.sh
}

fedora()
{
	echo "Detected Fedora"
    	if [ -z "$(which git)" ]; then
		echo "Installing git..."
		sudo yum install git-all
	fi
	if [ "$2" == "qemu" ]; then
		if [ -z "$(which qemu-system-i386)" ]; then
			echo "Installing QEMU..."
			sudo yum install qemu-system-x86 qemu-kvm
		else
			echo "QEMU already installed!"
		fi
	else
		if [ -z "$(which virtualbox)" ]; then
			echo "Installing virtualbox..."
			sudo yum install virtualbox
		else
			echo "Virtualbox already installed!"
		fi
	fi
	echo "Cloning Redox repo"
	git clone -b "$1" --recursive https://github.com/redox-os/redox.git
	echo "Installing necessary build tools..."
	sudo dnf install gcc gcc-c++ glibc-devel.i686 nasm make
	echo "Running rust installer"
	sh redox/setup/binary.sh
}

suse()
{
	echo "Detected a suse"
	if [ -z "$(which git)" ]; then
		echo "Installing git..."
		zypper install git
	fi
	if [ "$2" == "qemu" ]; then
		if [ -z "$(which qemu-system-i386)" ]; then
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
	echo "Cloning Redox repo..."
	git clone -b "$1" --recursive https://github.com/redox-os/redox.git
	echo "Installing necessary build tools..."
	sudo zypper install gcc gcc-c++ glibc-devel-32bit nasm make
	echo "Running rust installer"
	sh redox/setup/binary.sh
}

usage()
{
	echo "------------------------"
	echo "|Redox bootstrap script|"
	echo "------------------------"
	echo "Usage: ./bootstrap.sh"
	echo "OPTIONS:"
	echo
	echo "   -h,--help      Show this prompt"
	echo "   -b [branch]    Specify a branch of redox to clone"
	echo "   -u [branch]    Update git repo and update rust"
	echo "                  If blank defaults to master"
	echo "   -e [emulator]  Install specific emulator, virtualbox or qemu"
	echo "   -p [package    Choose an Ubuntu package manager, apt-fast or"
	echo "       manager]   aptitude"
	echo "EXAMPLES:"
	echo
	echo "./bootstrap.sh -b buddy -e qemu"
	exit
}

updater()
{
	git pull origin "$1"
	sh setup/binary.sh
	exit
}

endMessage()
{
	echo "Cleaning up..."
	rm bootstrap.sh
	echo "---------------------------------------"
	echo "Well it looks like you are ready to go!"
	echo "---------------------------------------"
	echo "		cd redox"
	echo "		make all"
	echo "		make virtualbox or qemu"
	echo
	echo "If make qemu fails complaining about kvm"
	echo "run \'make qemu kvm=no\'"
	echo
	echo "      Good luck!"

	exit
}
if [ "$1" == "-h" ]; then
	usage
fi

if [ "$1" == "-u" ]; then
	if [ -n "$2" ]; then
		updater "$2"
	else
		updater "master"
	fi
fi

branch="master"
emulator="qemu"
defpackman="apt-get"
while getopts ":b:e:p:" opt
do
	case "$opt" in
		b) branch="$OPTARG";;
		e) emulator="$OPTARG";;
		p) defpackman="$OPTARG";;
		\?) echo "I don't know what to do with that option, try -h for help"; exit;;
	esac
done

banner
if [ "Darwin" == "$(uname -s)" ]; then
	osx "$branch" "$emulator"
else
	which pacman && { archLinux "$branch" "$emulator"; endMessage; }
	which apt-get && { ubuntu "$branch" "$emulator" "$defpackman"; endMessage; }
	which yum && { fedora "$branch" "$emulator"; endMessage; }
	which zypper && { suse "$branch" "$emulator"; endMessage; }
fi
