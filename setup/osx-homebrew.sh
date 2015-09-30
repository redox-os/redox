#!/bin/bash
brew install homebrew/versions/gcc49
brew tap altkatz/homebrew-gcc_cross_compilers
brew install i386-elf-gcc i386-elf-binutils nasm
