#! /bin/bash
banner() {
	echo "|------------------------------------------|"
	echo "|----- Welcome to the redox bootstrap -----|"
	echo "|------------------------------------------|"
}

osx()
{
	echo "Detected OSX!"
	temp=$(brew --version)
	if [ -n "$temp" ]; then
		echo "Homebrew detected! Now updating..."
		brew update
		echo "Now installing git..."
		brew install git
		echo "Now installing virtualbox..."
		brew cask install virtualbox
	else
		echo "Homebrew does not appear to be installed! Would you like me to install it?"
		printf "(Y/n): "
		#For now assume yes
		ruby -e "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/master/install)"
	fi
	echo "Cloning Redox repo"
	git clone -b $1 --recursive https://github.com/redox-os/redox.git
	sh redox/setup/osx-homebrew.sh
	sh redox/setup/binary.sh
}

archLinux()
{
	echo "Detected Arch Linux, mah fav"
	echo "Updating system..."
	sudo pacman -Syu
	echo "Installing git..."
	sudo pacman -S git
	echo "Cloning Redox repo"
	if [ "$2" == "qemu" ]; then
		echo "Installing QEMU..."
		sudo pacman -S qemu
	fi
	git clone -b $1 --recursive https://github.com/redox-os/redox.git
	sh redox/setup/arch.sh
	sh redox/setup/binary.sh
}

ubuntu()
{
	echo "Detected Ubuntu/Debian"
	echo "Updating system..."
	sudo $3 update
	echo "Installing required packages..."
	sudo $3 install build-essential libc6-dev-i386 nasm curl file git
	if [ "$2" == "qemu" ]; then
		echo "Installing QEMU..."
		sudo $3 install qemu-system-x86 qemu-kvm
	fi
	echo "Cloning Redox repo"
	git clone -b $1 --recursive https://github.com/redox-os/redox.git
	sh redox/setup/binary.sh
}

fedora()
{
	echo "Detected Fedora"
	echo "Installing git..."
	sudo yum install git-all
	if [ "$2" == "qemu" ]; then
		echo "Installing QEMU..."
		sudo yum install qemu-system-x86 qemu-kvm
	else
		echo "Installing virtualbox..."
		sudo yum install virtualbox
	fi
	echo "Cloning Redox repo"
	git clone -b $1 --recursive https://github.com/redox-os/redox.git
	sudo dnf install gcc gcc-c++ glibc-devel.i686 nasm make
	sh redox/setup/binary.sh
}

suse()
{
	echo "Detected a suse"
	echo "Installing git..."
	zypper install git
	if [ "$2" == "qemu" ]; then
		echo "Installing QEMU..."
		sudo zypper install qemu-x86 qemu-kvm
	else
		which virtualbox && { "Virtualbox installed!"; break; }
		echo "Please install Virtualbox and re-run this script,"
		echo "or run with -e qemu"
		exit
	fi
	echo "Cloning Redox repo..."
	git clone -b $1 --recursive https://github.com/redox-os/redox.git
	sudo zypper install gcc gcc-c++ glibc-devel-32bit nasm make
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

update()
{
	git pull origin $1
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
	echo "      Good luck!"

	exit
}
if [ "$1" == "-h" ]; then
	usage
fi

if [ "$1" == "-u" ]; then
	if [ -n "$2" ]; then
		update $2
	else
		update "master"
	fi
fi

branch="master"
emulator="virtualbox"
defpackman="apt-get"
while getopts ":b:e:p:" opt
do
	case "$opt" in
		b) branch="$OPTARG";;
		e) emulator="$OPTARG";;
				p) defpackman="$OPTARG";;
		\?) echo "I don't know what to do with that option..."; exit;;
	esac
done

banner
kernel=$(uname -s)
if [ "$kernel" == "Darwin" ]; then
	osx $branch
else
	which pacman && { archLinux $branch $emulator; endMessage; }
	which apt-get && { ubuntu $branch $emulator $defpackman; endMessage; }
	which yum && { fedora $branch $emulator; endMessage; }
	which zypper && { suse $branch $emulator; endMessage; }
fi
