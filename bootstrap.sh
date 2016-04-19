#! /bin/bash
banner() {
	echo "|------------------------------------------|"
	echo "|----- Welcome to the redox bootstrap -----|"
	echo "|------------------------------------------|"
}

gitClone() {
	git clone https://github.com/redox-os/redox.git --origin upstream --recursive
	git submodule update --recursive --init
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
	echo "Running Redox setup script..."
	brew tap homebrew/versions
	brew install gcc49
	brew tap Nashenas88/homebrew-gcc_cross_compilers
	brew install i386-elf-binutils i386-elf-gcc nasm
	brew install Caskroom/cask/osxfuse
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

	echo "Installing fuse..."
	sudo pacman -S fuse
	echo "Running Redox setup scripts..."
	sh redox/setup/arch.sh
}

ubuntu()
{
	echo "Detected Ubuntu/Debian"
	echo "Updating system..."
	sudo "$3" update
	echo "Installing required packages..."
	sudo "$3" install build-essential libc6-dev-i386 nasm curl file git libfuse-dev
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
	gitClone
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
	gitClone
	echo "Installing necessary build tools..."
	sudo dnf install gcc gcc-c++ glibc-devel.i686 nasm make libfuse-dev
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
	gitClone
	echo "Installing necessary build tools..."
	sudo zypper install gcc gcc-c++ glibc-devel-32bit nasm make libfuse
}

gentoo()
{
	echo "Detected Gentoo Linux"
	if [ -z "$(which nasm)" ]; then
		echo "Installing nasm..."
		sudo emerge dev-lang/nasm
	fi
	if [ -z "$(which git)" ]; then
		echo "Installing git..."
		sudo emerge dev-vcs/git
	fi
	echo "Installing fuse..."
	sudo emerge sys-fs/fuse
	if [ "$2" == "qemu" ]; then
		if [ -z "$(which qemu-system-i386)" ]; then
			echo "Please install QEMU and re-run this script"
			echo "Step1. Add QEMU_SOFTMMU_TARGETS=\"i386\" to /etc/portage/make.conf"
			echo "Step2. Execute \"sudo emerge app-emulation/qemu\""
		else
			echo "QEMU already installed!"
		fi
	fi
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

rustInstall() {
	if [ -z "$(which multirust)" ]; then
		echo "You do not have multirust installed."
		echo "We HIGHLY reccomend using multirust."
		echo "Would you like to install it now?"
		printf "(y/N): "
		read mrust
		if echo "$mrust" | grep -iq "^y" ;then
			#install multirust
			echo "Multirust will be installed!"
			curl -sf https://raw.githubusercontent.com/brson/multirust/master/quick-install.sh | sh
			multirust override nightly
		else
			echo "Multirust will not be installed!"
		fi
	fi
	if [ -z "$(which rustc)" ]; then
		echo "Rust is not installed"
		echo "Please either run the script again, accepting multirust install"
		echo "or install rustc nightly manually (not reccomended) via:"
		echo "\#curl -sSf https://static.rust-lang.org/rustup.sh | sh -s -- --channel=nightly"
		exit 1
	fi
	if echo "$(rustc --version)" | grep -viq "nightly" ;then
		echo "It appears that you have rust installed, but it"
		echo "is not the nightly version, please either install"
		echo "the nightly manually (not reccomended) or run this"
		echo "script again, accepting the multirust install"
		echo
	else
		echo "Your rust install looks good!"
		echo
	fi
}

statusCheck() {
	for i in $(echo "$(curl -sf https://api.travis-ci.org/repositories/redox-os/redox.json)" | tr "," "\n")
	do
	  if echo "$i" | grep -iq "last_build_status" ;then
	    if echo "$i" | grep -iq "0" ;then
				echo; echo;
				echo "********************************************"
	      echo "Travis reports that the last build succeded!"
	      echo "Looks like you are good to go!"
				echo "********************************************"
	    else
				echo; echo;
				echo "**************************************************"
	      echo "Travis reports that the last build *FAILED* :("
	      echo "Might want to check out the issues before building"
				echo "**************************************************"
	    fi
	  fi
	done
}

endMessage()
{
	echo "Cloning github repo..."
	gitClone
	rustInstall
	echo "Cleaning up..."
	rm bootstrap.sh
	echo
	echo "---------------------------------------"
	echo "Well it looks like you are ready to go!"
	echo "---------------------------------------"
	statusCheck
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
	git pull origin master
	multirust update nightly
	exit
fi

emulator="qemu"
defpackman="apt-get"
while getopts ":e:p:" opt
do
	case "$opt" in
		e) emulator="$OPTARG";;
		p) defpackman="$OPTARG";;
		\?) echo "I don't know what to do with that option, try -h for help"; exit;;
	esac
done

banner
if [ "Darwin" == "$(uname -s)" ]; then
	osx "$emulator"
else
	# Arch linux
	if hash 2>/dev/null pacman; then
		archLinux "$emulator"
	fi
	# Debian or any derivative of it
	if hash 2>/dev/null apt-get; then
		ubuntu "$emulator" "$defpackman"
	fi
	# Fedora
	if hash 2>/dev/null yum; then
		fedora "$emulator"
	fi
	# Suse and derivatives
	if hash 2>/dev/null zypper; then
		suse "$emulator"
	fi
	# Gentoo
	if hash 2>/dev/null emerge; then
		gentoo "$emulator"
 	fi
fi
endMessage
