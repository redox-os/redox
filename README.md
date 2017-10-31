# cookbook
A collection of package recipes for Redox.

[![Travis Build Status](https://travis-ci.org/redox-os/cookbook.svg?branch=master)](https://travis-ci.org/redox-os/cookbook)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

## Setup

### Ubuntu and other Debian based systems

To install the toolchain, run the following commands:
```bash
# Get the Redox OS APT key
sudo apt-key adv --keyserver keyserver.ubuntu.com --recv-keys AA12E97F0881517F

# Install the APT repository
sudo add-apt-repository 'deb https://static.redox-os.org/toolchain/apt /'

# Update your package lists
sudo apt update

# Install the cross compiler
sudo apt install x86-64-unknown-redox-gcc
```

### Arch Linux
To install the toolchain, run the following commands:
 ```bash
 # Clone libc
 git clone --recursive git@github.com:redox-os/libc

 # Go to the packages
 cd libc/packages/arch

 # Start with binutils
 cd binutils
 makepkg -si

 # Then autoconf
 cd ../autoconf
 makepkg -si

 # Then gcc-freestanding
 cd ../gcc-freestanding
 makepkg -si

 # Then newlib
 cd ../newlib
 makepkg -si

 # Finally gcc
 cd ../gcc
 makepkg -si
 ```

### Gentoo Linux
```bash
 # Clone libc
 git clone --recursive git@github.com:redox-os/libc

 # Install needed tools
 emerge -a =sys-devel/autoconf-2.64 =sys-devel/automake-1.11.6-r2

 # Run the setup script
 cd libc
 PREFIX=<your preferred toolchain prefix> ./setup.sh all

 # Add the tools to your path
 export PATH=$PATH:<toolchain prefix>/bin
```

### Other distros/Mac OS X
To install the toolchain, run the following commands:
 ```bash
 # Clone libc
 git clone --recursive git@github.com:redox-os/libc

 # Run the setup script
 cd libc
 ./setup.sh all

 # Add the tools to your path
 export PATH=$PATH:/path/to/libc/build/prefix/bin
 ```
