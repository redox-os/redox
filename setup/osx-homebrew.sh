#!/bin/bash
brew tap homebrew/versions
brew install gcc49
brew tap Nashenas88/homebrew-gcc_cross_compilers
brew install i386-elf-binutils i386-elf-gcc nasm
