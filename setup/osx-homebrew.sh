#!/bin/bash
brew tap homebrew/versions
brew install gcc49
brew tap nashenas88/gcc_cross_compilers
brew install nashenas88/gcc_cross_compilers/i386-elf-binutils nashenas88/gcc_cross_compilers/i386-elf-gcc nasm
